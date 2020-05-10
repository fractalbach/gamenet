//! Module containing astar utilities for use with various graphs.
//!
//! The function provided in this module expands on the basic astar
//! algorithm by being able to add nodes and edges to the graph, as the
//! graph is explored.

use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use std::collections::{BinaryHeap, HashMap, HashSet, LinkedList};
use std::f64::{consts, INFINITY};
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Index;

use cgmath::{Vector2, vec2, InnerSpace, MetricSpace};
use geo::Line;
use petgraph::graph::{UnGraph, NodeIndex};
use petgraph::visit::{IntoNeighbors, IntoEdges, Visitable, EdgeRef};

use quad::{QuadMap, ItemId, Rect, Spatial};
use util::cw_angle_pos;
use util::line::LineOps;
use util::vec::VecMap;
use util::vec2::{VecOps, ToVec2};


struct DynAstar<'a, W: Fn(Vector2<f64>, Vector2<f64>) -> Option<f64>> {
    graph: &'a UnGraph<Vector2<f64>, f64>,
    node_map: QuadMap<NodeMapEntry>,
    edge_map: QuadMap<Line<f64>>,
    index_map: HashMap<NodeIndex, ItemId>,
    weight: W,
    step: f64,
    max_step: f64,
}

#[derive(Debug, Clone, Copy)]
struct NodeMapEntry {
    index: NodeIndex,
    uv: Vector2<f64>,
}

#[derive(Hash, Debug, Clone, Copy)]
enum NodeRef {
    Graph(NodeIndex),
    Grid(Vector2<i32>),
}

struct FrontierItem {
    index: NodeRef,
    priority: f64,
}


/// Finds optimal route from start to dest, adding nodes where needed.
/// 
/// This function expands upon the basic astar algorithm by being able 
/// to add nodes and edges to the graph, as the graph is explored.
///
/// If the initial state of the graph needs to be preserved, a copy
/// should be made and passed to this function. Copying will have a
/// small performance cost compared to the dynamic astar
/// implementation itself.
///
/// # Arguments
/// * graph - Mutable graph to explore, and add to.
/// * bounds - Bounds which contain all points and edges.
/// * weight - Function which produces the weight for an edge between
///         any two points.
/// * start - NodeIndex of start node.
/// * dest - NodeIndex of destination node.
/// * step - Distance from origin node at which new nodes
///         will be created.
///
/// # Return
/// Vec containing NodeIndex of each node in the path from start
/// to dest.
pub fn dyn_astar<W: Fn(Vector2<f64>, Vector2<f64>) -> Option<f64>>(
    graph: &UnGraph<Vector2<f64>, f64>,
    bounds: Rect,
    weight: W,
    start: NodeIndex,
    dest: NodeIndex,
    step: f64,
) -> Vec<Vector2<f64>>
where
    W: FnMut(Vector2<f64>, Vector2<f64>) -> Option<f64>,
{
    let mut astar_data = DynAstar::new(graph, bounds, weight, step);
    let node_references = astar_data.astar(start, dest);
    node_references.map(|&(reference, _)| astar_data.pos(reference))
}

impl<'a, W> DynAstar<'a, W>
where W: Fn(Vector2<f64>, Vector2<f64>) -> Option<f64> {
    fn new(
        graph: &'a UnGraph<Vector2<f64>, f64>,
        bounds: Rect,
        weight: W,
        step: f64,
    ) -> DynAstar<'a, W> {
        let (index_map, node_map) = Self::create_node_map(graph, bounds);
        let edge_map = Self::create_edge_map(graph, bounds);
        DynAstar {
            graph,
            node_map,
            edge_map,
            index_map,
            weight,
            step,
            max_step: step * consts::SQRT_2,
        }
    }

    fn create_node_map(
        graph: &UnGraph<Vector2<f64>, f64>, bounds: Rect
    ) -> (HashMap<NodeIndex, ItemId>, QuadMap<NodeMapEntry>) {
        let mut index_map = HashMap::with_capacity(graph.node_count());
        let mut quad_map = QuadMap::default(bounds);
        for index in graph.node_indices() {
            let pos = graph[index];
            let map_index = quad_map.insert(NodeMapEntry::new(pos, index));
            index_map.insert(index, map_index);
        }
        (index_map, quad_map)
    }

    fn create_edge_map(
        graph: &UnGraph<Vector2<f64>, f64>, bounds: Rect
    ) -> QuadMap<Line<f64>> {
        let mut map = QuadMap::default(bounds);
        for index in graph.node_indices() {
            for edge in graph.edges(index) {
                let pos_a = graph.index(edge.source());
                let pos_b = graph.index(edge.target());
                map.insert(Line::new(pos_a.to_point(), pos_b.to_point()));
            }
        }
        map
    }

    /// Finds best path from A to B indices.
    fn astar(&self, start: NodeIndex, dest: NodeIndex) -> Vec<(NodeRef, f64)> {
        let grid_result = self.grid_astar(start, dest);
        let elisions = self.find_elisions(&grid_result);
        Self::elide(&grid_result, &elisions)
        // grid_result
    }

    /// Locate nodes to be removed from route in post-processing.
    fn find_elisions(&self, nodes: &Vec<(NodeRef, f64)>) -> Vec<usize> {
        // Remove nodes that do not reduce path weight.
        let mut elisions = vec!();
        let mut previous = 0;
        for examined in 1..(nodes.len() - 1) {
            let next = examined + 1;
            let (previous_node, previous_cost) = nodes[previous];
            let (next_node, next_cost) = nodes[next];
            let previous_pos = self.pos(previous_node);
            let next_pos = self.pos(next_node);
            let current_weight = next_cost - previous_cost;
            let potential_weight =
                self.edge_weight(previous_pos, next_pos).unwrap_or(INFINITY);

            // If route could be improved by eliding examined node, do so.
            if current_weight >= potential_weight {
                elisions.push(examined);
            } else {
                previous = examined;
            }
        }
        elisions
    }

    fn elide(
        nodes: &Vec<(NodeRef, f64)>, elisions: &Vec<usize>
    ) -> Vec<(NodeRef, f64)> {
        if elisions.is_empty() {
            return nodes.clone();
        }
        let mut elisions_i = 0;
        let mut result = vec!();
        for (i, &node) in nodes.iter().enumerate() {
            if elisions_i < elisions.len() && i == elisions[elisions_i] {
                elisions_i += 1;
            } else {
                result.push(node);
            }
        }
        result
    }

    fn grid_astar(
        &self, start: NodeIndex, dest: NodeIndex
    ) -> Vec<(NodeRef, f64)> {
        // Create collections.
        let mut frontier: BinaryHeap<FrontierItem> =
            BinaryHeap::with_capacity(self.graph.node_count());
        let mut previous_nodes: HashMap<NodeRef, NodeRef> = HashMap::new();
        let mut costs: HashMap<NodeRef, f64> = HashMap::new();

        // Initialize.
        frontier.push(FrontierItem{index: NodeRef::Graph(start), priority: 0.0});
        costs.insert(NodeRef::Graph(start), 0.);
        let max_heuristic = self.max_heuristic(start, dest);

        // Search
        while !frontier.is_empty() {
            let current = frontier.pop().unwrap().index;
            if current == NodeRef::Graph(dest) {
                break;
            }

            // Iterate over neighbors.
            let current_cost = costs[&current];
            for (next, weight) in self.neighbors(current) {
                let potential_cost = current_cost + weight;
                let existing_cost = *costs.get(&next).unwrap_or(&INFINITY);
                if potential_cost >= existing_cost {
                    continue;
                }
                let next_cost = potential_cost;
                costs.insert(next, next_cost);
                let next_pos = self.pos(next);
                let heuristic = self.heuristic(next_pos, start, dest);
                if heuristic > max_heuristic {
                    continue;
                }
                let priority = -(next_cost + heuristic);
                frontier.push(FrontierItem{index: next, priority});
                previous_nodes.insert(next, current);
            }
        }

        // Unpack route taken.
        Self::unpack_route(
            &previous_nodes,
            &costs,
            NodeRef::Graph(start),
            NodeRef::Graph(dest)
        )
    }

    fn unpack_route(
        previous: &HashMap<NodeRef, NodeRef>,
        costs: &HashMap<NodeRef, f64>,
        start: NodeRef,
        dest: NodeRef
    ) -> Vec<(NodeRef, f64)> {
        let mut route = vec!();
        if !previous.contains_key(&dest) {
            return route;
        }
        let mut current = dest;
        loop {
            route.push((current, costs[&current]));
            if current == start {
                break;
            }
            current = previous[&current];
        }
        route.reverse();
        route
    }

    fn heuristic(
        &self, pos: Vector2<f64>, start: NodeIndex, dest: NodeIndex
    ) -> f64 {
        pos.distance(self.pos(NodeRef::Graph(dest)))
    }

    fn max_heuristic(&self, start: NodeIndex, dest: NodeIndex) -> f64 {
        let start_pos = self.pos(NodeRef::Graph(start));
        let dest_pos = self.pos(NodeRef::Graph(dest));
        let distance = start_pos.distance(dest_pos) * 3.;
        let floor = self.step * 50.;
        if distance < floor { floor } else { distance }
    }

    fn pos(&self, node: NodeRef) -> Vector2<f64> {
        match node {
            NodeRef::Graph(index) => self.graph[index],
            NodeRef::Grid(indices) => {
                self.node_map.bounding_box().midpoint() + vec2(
                    indices.x as f64 * self.step, indices.y as f64 * self.step
                )
            }
        }
    }

    /// Gets neighbor nodes and the weight of the edge connecting to them.
    fn neighbors(&self, node: NodeRef) -> Vec<(NodeRef, f64)> {
        let pos = self.pos(node);
        let mut nodes = vec!();

        // Collect nearby graph nodes.
        for nearby_node in self.node_map.query_radius(pos, self.max_step).map(
                |(node_entry, _, _, _)| NodeRef::Graph(node_entry.index)
        ) {
            if nearby_node != node {
                nodes.push(nearby_node)
            }
        }

        // Collect grid node neighbors.
        nodes.append(&mut match node {
            NodeRef::Graph(index) => {
                let step_pos = self.pos(node) / self.step;
                let min_indices = vec2(step_pos.x as i32, step_pos.y as i32);
                let other_x = if step_pos.x < 0. {
                    min_indices.x - 1
                } else {
                    min_indices.x + 1
                };
                let other_y = if step_pos.y < 0. {
                    min_indices.y - 1
                } else {
                    min_indices.y + 1
                };
                vec!(
                    NodeRef::Grid(vec2(min_indices.x, min_indices.y)),
                    NodeRef::Grid(vec2(min_indices.x, other_y)),
                    NodeRef::Grid(vec2(other_x, min_indices.y)),
                    NodeRef::Grid(vec2(other_x, other_y)),
                )
            },
            NodeRef::Grid(indices) => {
                vec!(
                    vec2(indices.x, indices.y + 1),  // North.
                    vec2(indices.x + 1, indices.y),  // East.
                    vec2(indices.x, indices.y - 1),  // South.
                    vec2(indices.x - 1, indices.y),  // West.
                    vec2(indices.x + 1, indices.y + 1),  // Northeast.
                    vec2(indices.x + 1, indices.y - 1),  // Southeast.
                    vec2(indices.x - 1, indices.y - 1),  // Southwest.
                    vec2(indices.x - 1, indices.y + 1),  // Northwest.
                    vec2(indices.x + 1, indices.y + 2),  // North-northeast.
                    vec2(indices.x + 2, indices.y + 1),  // East-northeast.
                    vec2(indices.x + 2, indices.y - 1),  // East-southeast.
                    vec2(indices.x + 1, indices.y - 2),  // South-southeast.
                    vec2(indices.x - 1, indices.y - 2),  // South-southwest.
                    vec2(indices.x - 2, indices.y - 1),  // West-southwest.
                    vec2(indices.x - 2, indices.y + 1),  // West-northwest.
                    vec2(indices.x - 1, indices.y + 2),  // North-northwest.
                ).map(|&indices| NodeRef::Grid(indices))
            }
        });
        let mut result = vec!();
        for node in nodes {
            match self.edge_weight(pos, self.pos(node)) {
                Some(weight) => result.push((node, weight)),
                None => ()
            }
        }

        // Collect edges from graph.
        match node {
            NodeRef::Graph(index) => {
                result.extend(self.graph.edges(index).map(|edge|{
                    if index == edge.source() {
                        (NodeRef::Graph(edge.target()), *edge.weight())
                    } else {
                        (NodeRef::Graph(edge.source()), *edge.weight())
                    }
                }));
            }
            NodeRef::Grid(_) => ()
        }

        result
    }

    // Internal utilities.

    /// Returns < 0 if no connection can be made.
    fn edge_weight(&self, a: Vector2<f64>, b: Vector2<f64>) -> Option<f64> {
        match self.path_is_clear(a, b) {
            true => None,
            false => (self.weight)(a, b)
        }
    }

    /// Checks that path between A and B is clear.
    ///
    /// This function returns false if any other edges would block the edge
    /// between A and B.
    fn path_is_clear(&self, a: Vector2<f64>, b: Vector2<f64>) -> bool {
        let line = Line::new(a.to_point(), b.to_point());
        let query_res = self.edge_map.query(Rect::from_points(a, b));
        query_res.iter().any(|(res_line, _, _)| res_line.crosses(&line))
    }
}

impl NodeMapEntry {
    fn new(pos: Vector2<f64>, index: NodeIndex) -> NodeMapEntry {
        NodeMapEntry { index, uv: pos}
    }

    fn uv(&self) -> Vector2<f64> {
        self.uv
    }
}

impl Spatial for NodeMapEntry {
    fn aabb(&self) -> Rect {
        Rect::null_at(self.uv)
    }
}

impl PartialEq for FrontierItem {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Eq for FrontierItem {}

impl Ord for FrontierItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.partial_cmp(&other.priority).unwrap_or(Equal)
    }
}

impl PartialOrd for FrontierItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NodeRef {
    fn eq(&self, other: &Self) -> bool {
        match self {
            NodeRef::Graph(node_index) => match other {
                NodeRef::Graph(other_index) => node_index == other_index,
                NodeRef::Grid(_) => false,
            }
            NodeRef::Grid(grid_indices) => match other {
                NodeRef::Graph(_) => false,
                NodeRef::Grid(other_indices) => grid_indices == other_indices
            }
        }
    }
}

impl Eq for NodeRef {}


#[cfg(test)]
mod tests {
    use cgmath::vec2;
    use petgraph::graph::{UnGraph, NodeIndex};

    use ::util::astar::*;
    use ::util::vec::VecMap;
    use ::util::vec2::VecOps;
    use quad::Rect;
    use test_util::serialize_to;
    use std::iter::FromIterator;
    use wasm_bindgen::__rt::std::collections::BinaryHeap;
    use petgraph::dot::Config::NodeIndexLabel;

    #[test]
    fn test_minimal_dyn_astar_path() {
        let mut graph = UnGraph::new_undirected();
        let start = graph.add_node(vec2(100.0, 0.0));
        let dest = graph.add_node(vec2(0.0, 100.0));
        let path = dyn_astar(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| Some(a.distance(b)),
            start,
            dest,
            10.,
        );
        serialize_to(&path, "dyn_astar_minimal_path.json");

        assert_gt!(path.len(), 0);
        assert_vec2_near!(path.last().unwrap(), graph[dest]);
    }

    #[test]
    fn test_dyn_astar_adjusted_weight_path() {
        let mut graph = UnGraph::new_undirected();
        let start = graph.add_node(vec2(100.0, 0.0));
        let dest = graph.add_node(vec2(100.0, 150.0));
        let path = dyn_astar(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| {
                let distance = a.distance(b);
                let mean_x = (a.x + b.x) / 2.;
                if distance < 50. {
                    Some(a.distance(b) * (1. + mean_x.abs() / 10.))
                } else {
                    None
                }
            },
            start,
            dest,
            10.,
        );
        serialize_to(&path, "dyn_astar_adjusted_path.json");

        assert_gt!(path.len(), 0);
        assert_vec2_near!(path.last().unwrap(), graph[dest]);
    }

    #[test]
    fn test_dyn_astar_path_with_existing_edges_in_graph() {
        let mut graph = UnGraph::new_undirected();
        let start = graph.add_node(vec2(0., 0.0));
        let dest = graph.add_node(vec2(0., 100.0));
        
        // Add parallel series of edges which should be utilized in the
        // resulting path.
        let c = graph.add_node(vec2(15., 0.));
        let d = graph.add_node(vec2(15., 10.));
        let e = graph.add_node(vec2(15., 20.));
        let f = graph.add_node(vec2(15., 30.));
        let g = graph.add_node(vec2(15., 40.));
        let h = graph.add_node(vec2(15., 50.));
        let i = graph.add_node(vec2(15., 60.));
        let j = graph.add_node(vec2(15., 70.));
        let k = graph.add_node(vec2(15., 80.));
        let l = graph.add_node(vec2(15., 90.));
        let m = graph.add_node(vec2(15., 100.));
        graph.add_edge(c, d, 10.);
        graph.add_edge(d, e, 10.);
        graph.add_edge(e, f, 10.);
        graph.add_edge(f, g, 10.);
        graph.add_edge(g, h, 10.);
        graph.add_edge(h, i, 10.);
        graph.add_edge(i, j, 10.);
        graph.add_edge(j, k, 10.);
        graph.add_edge(k, l, 10.);
        graph.add_edge(l, m, 10.);

        let path = dyn_astar(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| Some(a.distance(b) * 2.),
            start,
            dest,
            10.,
        );
        serialize_to(&path, "dyn_astar_with_previous_edges_path.json");

        assert_gt!(path.len(), 0);
        assert_vec2_near!(path.last().unwrap(), graph[dest]);
    }

    #[test]
    fn test_dyn_astar_path_with_obstacle() {
        let mut graph = UnGraph::new_undirected();
        let start = graph.add_node(vec2(0., 0.0));
        let dest = graph.add_node(vec2(0., 100.0));

        // Add parallel series of edges which should be utilized in the
        // resulting path.
        let obstacle_end_a = graph.add_node(vec2(-50., 50.));
        let obstacle_end_b = graph.add_node(vec2(50., 50.));
        graph.add_edge(obstacle_end_a, obstacle_end_b, 1000.);

        let path = dyn_astar(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| Some(a.distance(b) * 1.5),
            start,
            dest,
            10.,
        );
        serialize_to(&path, "dyn_astar_obstacle_path.json");

        assert_gt!(path.len(), 0);
        assert_vec2_near!(path.last().unwrap(), graph[dest]);
    }

    #[test]
    fn test_dyn_astar_path_with_long_obstacle() {
        let mut graph = UnGraph::new_undirected();
        let start = graph.add_node(vec2(0., 0.0));
        let dest = graph.add_node(vec2(0., 100.0));

        // Add parallel series of edges which should be utilized in the
        // resulting path.
        let obstacle_end_a = graph.add_node(vec2(-100., 50.));
        let obstacle_end_b = graph.add_node(vec2(100., 50.));
        graph.add_edge(obstacle_end_a, obstacle_end_b, 1000.);

        let path = dyn_astar(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| Some(a.distance(b) * 1.5),
            start,
            dest,
            10.,
        );
        serialize_to(&path, "dyn_astar_long_obstacle_path.json");

        assert_gt!(path.len(), 0);
        assert_vec2_near!(path.last().unwrap(), graph[dest]);
    }

    #[test]
    fn test_frontier_order_in_bin_heap() {
        let mut heap = BinaryHeap::new();
        heap.push(FrontierItem {
            index: NodeRef::Graph(NodeIndex::new(0)), priority: -1.0
        });
        heap.push(FrontierItem {
            index: NodeRef::Graph(NodeIndex::new(1)), priority: -2.0
        });
        assert!(heap.peek().is_some());
        assert_eq!(heap.peek().unwrap().index, NodeRef::Graph(NodeIndex::new(0)));
    }
}
