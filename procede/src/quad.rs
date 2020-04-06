//! Adapted from aabb-quadtree.
//!
//! A simple spacial partitioning data structure that allows fast queries for
//! 2-dimensional objects.
//!
//! As the name implies, the tree is a mapping from axis-aligned-bounding-box => object.

use std::cmp::{Ord, Ordering};
use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::ops::{Index, IndexMut};

use cgmath::{Vector2, vec2};
use cgmath::MetricSpace;
use fnv::FnvHasher;
use serde::{Deserialize, Serialize};
use serde::ser::SerializeStruct;

type FnvHashMap<K, V> = HashMap<K, V, BuildHasherDefault<FnvHasher>>;

/// An object that has a bounding box.
///
/// Implementing this trait is not required, but can make insertions easier.
pub trait Spatial {
    /// Returns the bounding box for the object.
    fn aabb(&self) -> Rect;
}

trait Close<Rhs: ?Sized = Self> {
    fn is_close(&self, other: &Self, eps: f64) -> bool;
}

/// An ID unique to a single QuadTree.  This is the object that is
/// returned from queries, and can be used to access the elements stored
/// in the quad tree.
///
/// DO NOT use an ItemId on a quadtree unless the ItemId came from that tree.
#[derive(
    Eq, PartialEq, Ord, PartialOrd,
    Hash, Clone, Copy, Debug,
    Serialize, Deserialize
)]
pub struct ItemId(u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QuadTreeConfig {
    allow_duplicates: bool,
    max_children: usize,
    min_children: usize,
    max_depth: usize,
    epsilon: f64,
}


#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct Rect {
    minimums: Vector2<f64>,
    maximums: Vector2<f64>,
}

/// The main QuadTree structure.  Mainly supports inserting, removing,
/// and querying objects in 2d space.
#[derive(Debug, Clone)]
pub struct QuadMap<T> {
    root: QuadNode,
    config: QuadTreeConfig,
    id: u32,
    elements: FnvHashMap<ItemId, (T, Rect)>,
}

#[derive(Debug, Serialize, Deserialize)]
enum QuadNode {
    Branch {
        aabb: Rect,
        children: [(Rect, Box<QuadNode>); 4],
        in_all: Vec<(ItemId, Rect)>,
        element_count: usize,
        depth: usize,
    },
    Leaf {
        aabb: Rect,
        elements: Vec<(ItemId, Rect)>,
        depth: usize,
    },
}

impl Clone for QuadNode {
    fn clone(&self) -> QuadNode {
        match self {
            &QuadNode::Branch {
                ref aabb,
                ref children,
                ref in_all,
                ref element_count,
                ref depth,
            } => {
                let children = [
                    children[0].clone(),
                    children[1].clone(),
                    children[2].clone(),
                    children[3].clone(),
                ];
                QuadNode::Branch {
                    aabb: aabb.clone(),
                    children: children,
                    in_all: in_all.clone(),
                    element_count: element_count.clone(),
                    depth: depth.clone(),
                }
            }
            &QuadNode::Leaf {
                ref aabb,
                ref elements,
                ref depth,
            } => QuadNode::Leaf {
                aabb: aabb.clone(),
                elements: elements.clone(),
                depth: depth.clone(),
            },
        }
    }
}

impl<T> QuadMap<T> {
    /// Constructs a new QuadTree with customizable options.
    ///
    /// * `size`: the enclosing space for the quad-tree.
    /// * `allow_duplicates`: if false, the quadtree will remove objects
    ///             that have the same bounding box.
    /// * `min_children`: the minimum amount of children that a tree
    ///             node will have.
    /// * `max_children`: the maximum amount of children that a tree
    ///             node will have before it gets split.
    /// * `max_depth`: the maximum depth that the tree can grow before
    ///             it stops.
    pub fn new(
        size: Rect,
        allow_duplicates: bool,
        min_children: usize,
        max_children: usize,
        max_depth: usize
    ) -> QuadMap<T> {
        QuadMap {
            root: QuadNode::Leaf {
                aabb: size,
                elements: Vec::with_capacity(max_children),
                depth: 0,
            },
            config: QuadTreeConfig {
                allow_duplicates,
                max_children,
                min_children,
                max_depth,
                epsilon: 0.0001,
            },
            id: 0,
            elements: HashMap::with_capacity_and_hasher(
                max_children * 16, Default::default()
            ),
        }
    }

    /// Constructs a new QuadTree with customizable options.
    ///
    /// * `size`: the enclosing space for the quad-tree.
    /// ### Defauts
    /// * `allow_duplicates`: true
    /// * `min_children`: 4
    /// * `max_children`: 16
    /// * `max_depth`: 8
    pub fn default(size: Rect) -> QuadMap<T> {
        QuadMap::new(size, true, 4, 16, 8)
    }

    /// Inserts an element with the provided bounding box.
    pub fn insert_with_box(&mut self, t: T, aabb: Rect) -> ItemId {
        let &mut QuadMap {
            ref mut root,
            ref config,
            ref mut id,
            ref mut elements,
        } = self;

        let item_id = ItemId(*id);
        *id += 1;

        if root.insert(item_id, aabb, config) {
            elements.insert(item_id, (t, aabb));
        }

        item_id
    }

    /// Returns an ItemId for the first element that was inserted into
    /// the tree.
    pub fn first(&self) -> Option<ItemId> {
        self.elements.iter().next().map(|(id, _)| *id)
    }

    /// Inserts an element into the tree.
    pub fn insert(&mut self, t: T) -> ItemId
        where
            T: Spatial,
    {
        let b = t.aabb();
        self.insert_with_box(t, b)
    }

    /// Retrieves an element by looking it up from the ItemId.
    pub fn get(&self, id: ItemId) -> Option<&T> {
        self.elements.get(&id).map(|&(ref a, _)| a)
    }

    pub fn get_mut(&mut self, id: ItemId) -> Option<&mut T> {
        self.elements.get_mut(&id).map(|&mut (ref mut a, _)| a)
    }

    /// Returns an iterator of (element, bounding-box, id) for each element
    /// whose bounding box intersects with `bounding_box`.
    pub fn query(&self, bounding_box: Rect) -> Vec<(&T, &Rect, ItemId)>
        where
            T: ::std::fmt::Debug,
    {
        let mut ids = vec![];
        self.root.query(bounding_box, &mut ids);
        ids.sort_by_key(|&(id, _)| id);
        ids.dedup();
        ids.iter()
            .map(|&(id, _)| {
                let &(ref t, ref rect) = match self.elements.get(&id) {
                    Some(e) => e,
                    None => {
                        panic!("looked for {:?}", id);
                    }
                };
                (t, rect, id)
            })
            .collect()
    }

    /// Gets element nearest to a set of UV coordinates within a radius.
    ///
    /// Distance is measured from the center of the element's rectangle.
    ///
    /// # Arguments
    /// * `uv` - Vector2<f64> specifying the center of the search area.
    /// * `r` - Radius around the position specified by `uv` within
    ///             which to search for the nearest Node.
    ///
    /// # Returns
    /// Tuple of:
    /// * Reference to the nearest item.
    /// * Item's bounding Rect.
    /// * ItemId.
    /// * Distance to the nearest item.
    pub fn nearest(
        &self, uv: Vector2<f64>, r: f64
    ) -> Option<(&T, Rect, ItemId, f64)> where T: ::std::fmt::Debug {
        let rect = Rect::centered_with_radius(uv, r);

        // Query Nodes within rect.
        let query_res = self.query(rect);
        if query_res.is_empty() {
            return None;
        }

        // Find which result is closest.
        let first_rect = query_res[0].1;
        let mut nearest_d2 = first_rect.midpoint().distance2(uv);
        let mut nearest_i = 0usize;
        for i in 1..query_res.len() {
            let res = query_res[i];
            let item_rect: &Rect = res.1;
            let d2 = item_rect.midpoint().distance2(uv);
            if d2 < nearest_d2 {
                nearest_d2 = d2;
                nearest_i = i;
            }
        }

        // Check that distance to nearest item is less than r.
        // Otherwise, return None.
        let d = nearest_d2.sqrt() as f64;
        if d > r {
            return None;
        }

        let (nearest, &rect, id) = query_res[nearest_i];
        Option::Some((nearest, rect, id, d))
    }

    /// Attempts to remove the item with id `item_id` from the tree.  If that
    /// item was present, it returns a tuple of (element, bounding-box)
    pub fn remove(&mut self, item_id: ItemId) -> Option<(T, Rect)> {
        match self.elements.remove(&item_id) {
            Some((item, aabb)) => {
                self.root.remove(item_id, aabb, &self.config);
                Some((item, aabb))
            }
            None => None,
        }
    }

    /// Returns an iterator over all the items in the tree.
    pub fn iter(
        &self
    ) -> ::std::collections::hash_map::Iter<ItemId, (T, Rect)> {
        self.elements.iter()
    }

    /// Calls `f` repeatedly for every node in the tree with these arguments
    ///
    /// * `&Rect`: The boudning box of that tree node
    /// * `usize`: The current depth
    /// * `bool`: True if the node is a leaf-node, False if the node is a
    ///             branch node.
    pub fn inspect<F: FnMut(&Rect, usize, bool)>(&self, mut f: F) {
        self.root.inspect(&mut f);
    }

    /// Returns the number of elements in the tree
    pub fn len(&self) -> usize { self.elements.len() }

    /// Returns true if the tree is empty.
    pub fn is_empty(&self) -> bool { self.elements.is_empty() }

    /// Returns the enclosing bounding-box for the entire tree.
    pub fn bounding_box(&self) -> Rect {
        self.root.bounding_box()
    }
}

impl QuadNode {
    fn bounding_box(&self) -> Rect {
        match self {
            &QuadNode::Branch { ref aabb, .. } => aabb.clone(),
            &QuadNode::Leaf { ref aabb, .. } => aabb.clone(),
        }
    }

    fn new_leaf(aabb: Rect, depth: usize, config: &QuadTreeConfig) -> QuadNode {
        QuadNode::Leaf {
            aabb: aabb,
            elements: Vec::with_capacity(config.max_children / 2),
            depth: depth,
        }
    }

    fn inspect<F: FnMut(&Rect, usize, bool)>(&self, f: &mut F) {
        match self {
            &QuadNode::Branch {
                depth,
                ref aabb,
                ref children,
                ..
            } => {
                f(aabb, depth, false);
                for child in children {
                    child.1.inspect(f);
                }
            }
            &QuadNode::Leaf { depth, ref aabb, .. } => {
                f(aabb, depth, true);
            }
        }
    }

    fn insert(
        &mut self, item_id: ItemId, item_aabb: Rect, config: &QuadTreeConfig
    ) -> bool {
        let mut into = None;
        let mut did_insert = false;
        match self {
            &mut QuadNode::Branch {
                ref aabb,
                ref mut in_all,
                ref mut children,
                ref mut element_count,
                ..
            } => {
                if item_aabb.contains(&aabb.midpoint()) {
                    // Only insert if there isn't another item with a very
                    // similar aabb.
                    if config.allow_duplicates || !in_all.iter().any(
                        |&(_, ref e_bb)| e_bb.is_close(
                            &item_aabb, config.epsilon
                        )
                    ) {
                        in_all.push((item_id, item_aabb));
                        did_insert = true;
                        *element_count += 1;
                    }
                } else {
                    for &mut (ref aabb, ref mut child) in children {
                        if aabb.does_intersect(&item_aabb) {
                            if child.insert(item_id, item_aabb, config) {
                                *element_count += 1;
                                did_insert = true;
                            }
                        }
                    }
                }
            }

            &mut QuadNode::Leaf {
                ref aabb,
                ref mut elements,
                ref depth,
            } => {
                if elements.len() == config.max_children &&
                        *depth != config.max_depth {
                    // STEAL ALL THE CHILDREN MUAHAHAHAHA
                    let mut extracted_children = Vec::new();
                    ::std::mem::swap(&mut extracted_children, elements);
                    extracted_children.push((item_id, item_aabb));
                    did_insert = true;

                    let split = aabb.split_quad();
                    into = Some((
                        extracted_children,
                        QuadNode::Branch {
                            aabb: *aabb,
                            in_all: Vec::new(),
                            children: [
                                (split[0], Box::new(QuadNode::new_leaf(
                                    split[0], depth + 1, config))),
                                (split[1], Box::new(QuadNode::new_leaf(
                                    split[1], depth + 1, config))),
                                (split[2], Box::new(QuadNode::new_leaf(
                                    split[2], depth + 1, config))),
                                (split[3], Box::new(QuadNode::new_leaf(
                                    split[3], depth + 1, config))),
                            ],
                            element_count: 0,
                            depth: *depth,
                        },
                    ));
                } else {
                    if config.allow_duplicates ||
                        !elements
                            .iter()
                            .any(|&(_, ref e_bb)| e_bb.is_close(
                                &item_aabb, config.epsilon)
                            )
                    {
                        elements.push((item_id, item_aabb));
                        did_insert = true;
                    }
                }
            }
        }

        // If we transitioned from a leaf node to a branch node, we
        // need to update ourself and re-add all the children that
        // we used to have
        // in our this leaf into our new leaves.
        if let Some((extracted_children, new_node)) = into {
            *self = new_node;
            for (child_id, child_aabb) in extracted_children {
                self.insert(child_id, child_aabb, config);
            }
        }

        did_insert
    }

    fn remove(
        &mut self, item_id: ItemId, item_aabb: Rect, config: &QuadTreeConfig
    ) -> bool {
        fn remove_from(v: &mut Vec<(ItemId, Rect)>, item: ItemId) -> bool {
            if let Some(index) =
                    v.iter().position(|a| a.0 == item) {
                v.swap_remove(index);
                true
            } else {
                false
            }
        }

        let mut compact = None;
        let removed = match self {
            &mut QuadNode::Branch {
                ref depth,
                ref aabb,
                ref mut in_all,
                ref mut children,
                ref mut element_count,
                ..
            } => {
                let mut did_remove = false;

                if item_aabb.contains(&aabb.midpoint()) {
                    did_remove = remove_from(in_all, item_id);
                } else {
                    for &mut (ref child_aabb, ref mut child_tree) in children {
                        if child_aabb.does_intersect(&item_aabb) {
                            did_remove |= child_tree.remove(item_id, item_aabb, config);
                        }
                    }
                }

                if did_remove {
                    *element_count -= 1;
                    if *element_count < config.min_children {
                        compact = Some((*element_count, *aabb, *depth));
                    }
                }
                did_remove
            }

            &mut QuadNode::Leaf {
                ref mut elements, ..
            } => remove_from(elements, item_id),
        };

        if let Some((size, aabb, depth)) = compact {
            let mut elements = Vec::with_capacity(size);
            self.query(aabb, &mut elements);
            elements.sort_by(|&(id1, _), &(ref id2, _)| id1.cmp(id2));
            elements.dedup();
            *self = QuadNode::Leaf {
                aabb: aabb,
                elements: elements,
                depth: depth,
            };
        }
        removed
    }

    fn query(&self, query_aabb: Rect, out: &mut Vec<(ItemId, Rect)>) {
        fn match_all(
            elements: &Vec<(ItemId, Rect)>,
            query_aabb: Rect,
            out: &mut Vec<(ItemId, Rect)>
        ) {
            for &(ref child_id, ref child_aabb) in elements {
                if query_aabb.does_intersect(child_aabb) {
                    out.push((*child_id, *child_aabb))
                }
            }
        }

        match self {
            &QuadNode::Branch { ref in_all, ref children, .. } => {
                match_all(in_all, query_aabb, out);

                for &(ref child_aabb, ref child_tree) in children {
                    if query_aabb.does_intersect(&child_aabb) {
                        child_tree.query(query_aabb, out);
                    }
                }
            }
            &QuadNode::Leaf {
                ref elements, ..
            } => match_all(elements, query_aabb, out),
        }
    }
}


impl<T> Index<ItemId> for QuadMap<T> {
    type Output = T;

    fn index(&self, id: ItemId) -> &Self::Output {
        &self.elements[&id].0
    }
}

impl<T> IndexMut<ItemId> for QuadMap<T> {
    fn index_mut(&mut self, id: ItemId) -> &mut Self::Output {
        self.get_mut(id).unwrap()
    }
}

impl<T> Serialize for QuadMap<T> where T: serde::Serialize{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: serde::Serializer
    {
        let mut state = serializer.serialize_struct("QuadMap", 2)?;
        state.serialize_field("config", &self.config)?;
        state.serialize_field("elements", &self.elements)?;
        state.end()
    }
}

impl Rect {
    pub const fn from_min_max(min: Vector2<f64>, max: Vector2<f64>) -> Rect {
        Rect { minimums: min, maximums: max }
    }

    pub fn centered_with_radius(p1: Vector2<f64>, r: f64) -> Rect {
        let v = vec2(r, r);
        Rect::from_points(p1 - v, p1 + v)
    }

    pub fn from_points(p1: Vector2<f64>, p2: Vector2<f64>) -> Rect {
        let mut r = Rect::null_at(p1);
        r.expand_to_include(p2);
        r
    }

    pub fn from_point_and_size(point: Vector2<f64>, size: Vector2<f64>) -> Rect {
        assert!(size.x > 0.0);
        assert!(size.y > 0.0);
        Rect {
            minimums: point,
            maximums: point + size,
        }
    }

    pub const fn null() -> Rect {
        let nan = ::std::f64::NAN;
        Rect {
            minimums: vec2(nan, nan),
            maximums: vec2(nan, nan),
        }
    }

    pub const fn null_at(point: Vector2<f64>) -> Rect {
        Rect {
            minimums: point,
            maximums: point,
        }
    }

    pub fn expand(&self, left: f64, bottom: f64, right: f64, top: f64) -> Rect {
        let minimums_vec = vec2(left, bottom);
        let maximums_vec = vec2(right, top);
        Rect {
            minimums: self.minimums - minimums_vec,
            maximums: self.maximums + maximums_vec,
        }
    }

    pub fn width(&self) -> f64 { self.maximums.x - self.minimums.x }

    pub fn height(&self) -> f64 { self.maximums.y - self.minimums.y }

    pub fn left(&self) -> f64 { self.minimums.x }

    pub fn right(&self) -> f64 { self.maximums.x }

    pub fn top(&self) -> f64 { self.maximums.y }

    pub fn bottom(&self) -> f64 { self.minimums.y }

    pub fn minimums(&self) -> Vector2<f64> { self.minimums }

    pub fn maximums(&self) -> Vector2<f64> { self.maximums }

    pub fn top_left(&self) -> Vector2<f64> {
        vec2(self.minimums().x, self.maximums().y)
    }

    pub fn bottom_right(&self) -> Vector2<f64> {
        vec2(self.maximums().x, self.minimums().y)
    }

    pub fn north(&self) -> Vector2<f64> {
        vec2(
            self.left() + self.width() / 2.0,
            self.top(),
        )
    }

    pub fn south(&self) -> Vector2<f64> {
        vec2(
            self.left() + self.width() / 2.0,
            self.bottom()
        )
    }

    pub fn west(&self) -> Vector2<f64> {
        vec2(
            self.left(),
            self.top() + self.height() / 2.0,
        )
    }

    pub fn east(&self) -> Vector2<f64> {
        vec2(
            self.right(),
            self.top() + self.height() / 2.0,
        )
    }

    pub fn expanded_by(&self, point: Vector2<f64>) -> Rect {
        let mut r = self.clone();
        r.expand_to_include(point);
        r
    }

    pub fn is_null(&self) -> bool {
        (
            self.minimums.x.is_nan() ||
            self.minimums.y.is_nan() ||
            self.maximums.x.is_nan() ||
            self.maximums.y.is_nan()
        )
    }

    pub fn expand_to_include(&mut self, point: Vector2<f64>) {
        fn min(a: f64, b: f64) -> f64 {
            if a.is_nan() { return b; }
            if b.is_nan() { return a; }
            if a < b { return a; }
            return b;
        }

        fn max(a: f64, b: f64) -> f64 {
            if a.is_nan() { return b; }
            if b.is_nan() { return a; }
            if a > b { return a; }
            return b;
        }

        self.minimums.x = min(self.minimums.x, point.x);
        self.minimums.y = min(self.minimums.y, point.y);

        self.maximums.x = max(self.maximums.x, point.x);
        self.maximums.y = max(self.maximums.y, point.y);
    }

    pub fn union_with(&self, other: &Rect) -> Rect {
        let mut r = self.clone();
        r.expand_to_include(other.minimums);
        r.expand_to_include(other.maximums);
        r
    }

    pub fn contains(&self, p: &Vector2<f64>) -> bool {
        (
            p.x >= self.minimums.x &&
            p.x < self.maximums.x &&
            p.y >= self.minimums.y &&
            p.y < self.maximums.y
        )
    }

    pub fn does_intersect(&self, other: &Rect) -> bool {
        let r1 = self;
        let r2 = other;

        // From stack overflow:
        // http://gamedev.stackexchange.com/a/913
        !(
            r2.left() > r1.right() ||
            r2.right() < r1.left() ||
            r2.top() < r1.bottom() ||
            r2.bottom() > r1.top()
        )
    }

    pub fn intersect_with(&self, other: &Rect) -> Rect {
        if !self.does_intersect(other) {
            return Rect::null();
        }
        let left = self.left().max(other.left());
        let right = self.right().min(other.right());

        let top = self.top().max(other.top());
        let bottom = self.bottom().min(other.bottom());

        Rect::from_points(vec2(left, top), vec2(right, bottom))
    }

    pub fn midpoint(&self) -> Vector2<f64> {
        let half = vec2(self.width() / 2.0, self.height() / 2.0);
        self.minimums() + half
    }

    pub fn split_vert(&self) -> (Rect, Rect) {
        let half_size = vec2(
            self.width() / 2.0,
            self.height(),
        );
        let half_offset = vec2(self.width() / 2.0, 0.0);
        (
            Rect::from_point_and_size(self.minimums, half_size),
            Rect::from_point_and_size((self.minimums + half_offset), half_size),
        )
    }

    pub fn split_hori(&self) -> (Rect, Rect) {
        let half_size = vec2(
            self.width(),
            self.height() / 2.0
        );
        let half_offset = vec2( 0.0, self.height() / 2.0);
        (
            Rect::from_point_and_size(self.minimums, half_size),
            Rect::from_point_and_size((self.minimums + half_offset), half_size),
        )
    }

    pub fn split_quad(&self) -> [Rect; 4] {
        let half = vec2(self.width() / 2.0, self.height() / 2.0);
        [
            // x _
            // _ _
            Rect::from_point_and_size(self.minimums, half),
            // _ x
            // _ _
            Rect::from_point_and_size(
                Vector2{
                    x: self.minimums.x + half.x,
                    ..self.minimums
                },
                half,
            ),
            // _ _
            // x _
            Rect::from_point_and_size(
                Vector2 {
                    y: self.minimums.y + half.y,
                    ..self.minimums
                },
                half,
            ),
            // _ _
            // _ x
            Rect::from_point_and_size((self.minimums + half), half),
        ]
    }
}  // Rect

impl Close for Rect {
    fn is_close(&self, other: &Rect, epsilon: f64) -> bool {
        self.minimums.is_close(&other.minimums, epsilon) &&
            self.maximums.is_close(&other.maximums, epsilon)
    }
}


impl PartialOrd for Rect {
    fn partial_cmp(&self, other: &Rect) -> Option<Ordering> {
        self.minimums.x.partial_cmp(&other.minimums.x).or_else(
            || self.minimums.y.partial_cmp(&other.minimums.y).or_else(
                || self.maximums.x.partial_cmp(&other.minimums.x).or_else(
                    || self.maximums.y.partial_cmp(&other.minimums.y)
                )
            )
        )
    }
}


impl PartialEq for Rect {
    fn eq(&self, other: &Rect) -> bool {
        self.minimums == other.minimums &&
            self.maximums == other.maximums
    }
}

impl Eq for Rect {}


impl Spatial for Rect {
    fn aabb(&self) -> Rect { *self }
}

impl Spatial for Vector2<f64> {
    fn aabb(&self) -> Rect { Rect::null_at(*self) }
}

impl Close for Vector2<f64> {
    fn is_close(&self, other: &Vector2<f64>, eps: f64) -> bool {
        self.distance2(*other) < eps * eps
    }
}


#[cfg(test)]
mod tests {
    use cgmath::{Vector2, vec2};

    use quad::{QuadMap, Rect};

    #[test]
    fn similar_points() {
        let mut quad_tree: QuadMap<Vector2<f64>> = QuadMap::new(
            Rect::centered_with_radius(vec2(0.0, 0.0), 10.0), false, 1, 5, 2
        );

        let p = vec2(0.0, 0.0);
        quad_tree.insert(p);
        quad_tree.insert(p);
        assert_eq!(quad_tree.elements.len(), 1);
    }

    /// Test that the nearest item to a passed position can be found.
    #[test]
    fn test_find_nearest_node() {
        let mut map = QuadMap::default(
            Rect::centered_with_radius(vec2(0.0, 0.0), 2000.0)
        );

        map.insert(vec2(0.0, 1000.0));
        map.insert(vec2(0.0, 0.0));  // Should be nearest.
        map.insert(vec2(1000.0, 0.0));
        map.insert(vec2(-500.0, -500.0));
        map.insert(vec2(100.0, -200.0));
        map.insert(vec2(-200.0, 100.0));

        let v = map.nearest(vec2(200.0, 200.0), 300.0).unwrap().0;

        assert_vec2_near!(v, vec2(0.0, 0.0));
    }

    /// Test that the nearest item to a passed position is not returned
    /// if the radius is too small.
    #[test]
    fn test_find_nearest_node_returns_none_if_radius_too_small() {
        let mut map = QuadMap::default(
            Rect::centered_with_radius(vec2(0.0, 0.0), 2000.0)
        );

        map.insert(vec2(0.0, 1000.0));
        map.insert(vec2(0.0, 0.0));  // Nearest.
        map.insert(vec2(1000.0, 0.0));

        assert!(map.nearest(vec2(200.0, 200.0), 220.0).is_none());
    }
}
