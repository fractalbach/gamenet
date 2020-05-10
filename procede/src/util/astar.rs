//! Module containing astar utilities for use with various graphs.
//!
//! The function provided in this module expands on the basic astar
//! algorithm by being able to add nodes and edges to the graph, as the
//! graph is explored.

use std::collections::{BinaryHeap, HashMap, HashSet};
use std::f64::{consts, INFINITY};
use std::hash::{Hash, Hasher};
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
use num_traits::real::Real;
use std::cmp::Ordering;
use std::cmp::Ordering::Equal;
use itertools::rev;


struct DynAstar<'a, W: Fn(Vector2<f64>, Vector2<f64>) -> Option<f64>> {
    graph: &'a mut UnGraph<Vector2<f64>, f64>,
    node_map: QuadMap<NodeMapEntry>,
    edge_map: QuadMap<Line<f64>>,
    index_map: HashMap<NodeIndex, ItemId>,
    calculated: HashSet<NodeIndex>,
    weight: W,
    min_sep: f64,
    sample_angle_gain: f64,
    initial_sample_angle: f64,
    step: f64,
    max_step: f64,
}

#[derive(Debug, Clone, Copy)]
struct NodeMapEntry {
    index: NodeIndex,
    uv: Vector2<f64>,
}

struct FrontierItem {
    index: NodeIndex,
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
    graph: &mut UnGraph<Vector2<f64>, f64>,
    bounds: Rect,
    weight: W,
    start: NodeIndex,
    dest: NodeIndex,
    step: f64,
) -> Vec<NodeIndex>
where
    W: FnMut(Vector2<f64>, Vector2<f64>) -> Option<f64>,
{
    let mut astar_data = DynAstar::new(graph, bounds, weight, step);
    astar_data.astar(start, dest)
}

impl<'a, W> DynAstar<'a, W>
where W: Fn(Vector2<f64>, Vector2<f64>) -> Option<f64> {
    const DEFAULT_SAMPLE_GAIN: f64 = 1.4;
    const DEFAULT_MIN_SEP_RATIO: f64 = 0.2;

    fn new(
        graph: &'a mut UnGraph<Vector2<f64>, f64>,
        bounds: Rect,
        weight: W,
        step: f64,
    ) -> DynAstar<'a, W> {
        let min_sep = step * Self::DEFAULT_MIN_SEP_RATIO;
        let n_nodes = graph.node_count();
        let (index_map, node_map) = Self::create_node_map(graph, bounds);
        let edge_map = Self::create_edge_map(graph, bounds);
        DynAstar {
            graph,
            node_map,
            edge_map,
            index_map,
            calculated: HashSet::with_capacity(n_nodes),
            weight,
            min_sep,
            sample_angle_gain: Self::DEFAULT_SAMPLE_GAIN,
            initial_sample_angle: min_sep / step * consts::PI,
            step,
            max_step: step * 1.9,
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

    /// Finds best path from A to B indices, optionally adding new nodes.
    fn astar(&mut self, start: NodeIndex, dest: NodeIndex) -> Vec<NodeIndex> {
        // Create collections.
        let mut frontier: BinaryHeap<FrontierItem> =
            BinaryHeap::with_capacity(self.graph.node_count());
        let mut previous_nodes: HashMap<NodeIndex, NodeIndex> = HashMap::new();
        let mut costs: HashMap<NodeIndex, f64> = HashMap::new();

        // Initialize.
        frontier.push(FrontierItem{index: start, priority: 0.0});
        costs.insert(start, 0.);
        let dest_pos = self.graph[dest];

        // Search
        while !frontier.is_empty() {
            let current = frontier.pop().unwrap().index;
            if current == dest {
                break;
            }

            // Populate current node's neighbors.
            let current_pos = self.graph[current];
            let dir_to_dest = dest_pos - current_pos;
            let mut preferred_dir = Vec::with_capacity(2);
            preferred_dir.push(dir_to_dest);
            match previous_nodes.get(&current) {
                Some(&previous) => {
                    let prev_pos = self.graph[previous];
                    let dir_from_previous = current_pos - prev_pos;
                    preferred_dir.push(dir_from_previous);
                }
                None => ()
            }
            self.create_neighbors(current, &preferred_dir);

            // Iterate over neighbors.
            let current_cost = costs[&current];
            for edge in self.graph.edges(current) {
                let next = if edge.target() != current {
                    edge.target()
                } else {
                    edge.source()
                };
                let potential_cost = current_cost + edge.weight();
                let existing_cost = *costs.get(&next).unwrap_or(&INFINITY);
                if potential_cost >= existing_cost {
                    continue;
                }
                let next_cost = potential_cost;
                costs.insert(next, next_cost);
                let next_pos = self.graph[next];
                let heuristic = next_pos.distance(dest_pos);
                let priority = -(next_cost + heuristic);
                frontier.push(FrontierItem{index: next, priority});
                previous_nodes.insert(next, current);
            }
        }

        // Unpack route taken.
        if !previous_nodes.contains_key(&dest) {
            return vec!();
        }
        let mut route = vec!();
        {
            let mut current = dest;
            loop {
                if current == start {
                    break;
                }
                route.push(current);
                current = previous_nodes[&current];
            }
            route.reverse()
        }
        route
    }


    // Internal utilities.

    /// Produces neighbors for the passed node in a graph.
    fn create_neighbors(
        &mut self,
        node: NodeIndex,
        preferred_dir: &Vec<Vector2<f64>>,
    ) {
        if self.calculated.contains(&node) {
            return;  // Already calculated.
        }

        // Make connections to existing neighbors.
        let &node_pos = self.graph.index(node);
        let nearby = self.node_map.query_radius(node_pos, self.max_step).map(
            |(node_info, _, _, _)| node_info.index
        );
        for index in nearby {
            if node != index {
                self.try_add_edge(node, index);
            }
        }

        // Generate new nodes to explore.
        for preferred in preferred_dir {
            for sample_dir in self.sample_directions(preferred) {
                let sample_pos = node_pos + sample_dir * self.step;
                self.try_add_node(sample_pos, node);
            }
        }
        self.calculated.insert(node);
    }

    fn sample_directions(&self, preferred_dir: &Vector2<f64>) -> Vec<Vector2<f64>> {
        let preferred_dir = preferred_dir.normalize();
        let mut result = Vec::with_capacity(32);
        // result.push(preferred_dir);
        // let mut sample_angle = self.initial_sample_angle;
        // while sample_angle < consts::PI {
        //     result.push(preferred_dir.rotate(sample_angle));
        //     result.push(preferred_dir.rotate(-sample_angle));
        //     sample_angle *= self.sample_angle_gain;
        // }
        for i in 0..8 {
            result.push(preferred_dir.rotate(i as f64 * (2. * consts::PI / 8.)));
        }
        result
    }

    fn try_add_node(
        &mut self, pos: Vector2<f64>, previous: NodeIndex
    ) -> Option<NodeIndex> {
        // Check if node is within min_sep distance.
        if self.node_map.nearest(pos, self.min_sep).is_some() {
            return None;
        }

        // Check if edge connection can be formed.
        let prev_pos = self.graph[previous];
        let weight = match self.edge_weight(pos, prev_pos) {
            Some(weight) => weight,
            None => return None,
        };

        // Add Node.
        let node_graph_index = self.graph.add_node(pos);
        let map_index = self.node_map.insert(
            NodeMapEntry::new(pos, node_graph_index)
        );
        self.index_map.insert(node_graph_index, map_index);
        self.edge_map.insert(Line::new(prev_pos.to_point(), pos.to_point()));
        self.graph.add_edge(previous, node_graph_index, weight);
        Some(node_graph_index)
    }

    /// Attempts to connect two existing nodes.
    ///
    /// (Nodes added via try_add_node() do not need to be connected by
    /// this method)
    ///
    /// Returns None if no connection can be made.
    fn try_add_edge(&mut self, a: NodeIndex, b: NodeIndex) -> Option<f64> {
        // Check if connection already exists.
        if self.graph.contains_edge(a, b) {
            return None;
        }

        let pos_a = self.graph[a];
        let pos_b = self.graph[b];
        // Check if path is clear between two points.
        let weight = match self.edge_weight(pos_a, pos_b) {
            Some(weight) => weight,
            None => return None
        };
        let line = Line::new(pos_a.to_point(), pos_b.to_point());
        self.graph.add_edge(a, b, weight);
        self.edge_map.insert(line);
        Some(weight)
    }

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
    fn test_sample_directions() {
        let mut graph = UnGraph::new_undirected();
        let astar = DynAstar::new(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| Some(1.0),
            10.,
        );
        let dir = astar.sample_directions(&vec2(0., 1.));
        serialize_to(&dir, "default_astar_sample_dir.json");
    }

    #[test]
    fn test_minimal_dyn_astar_path() {
        let mut graph = UnGraph::new_undirected();
        let start = graph.add_node(vec2(100.0, 0.0));
        let dest = graph.add_node(vec2(0.0, 100.0));
        let path_nodes = dyn_astar(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| Some(a.distance(b)),
            start,
            dest,
            10.,
        );
        let path = path_nodes.map(|node| graph[*node].to_point());
        serialize_to(&path, "dyn_astar_minimal_path.json");

        let all_nodes = Vec::from_iter(
            graph.node_indices().map(|index| graph[index])
        );
        serialize_to(&all_nodes, "dyn_astar_minimal_all_nodes.json");

        assert_gt!(path.len(), 0);
        assert_vec2_near!(path.last().unwrap().to_vec(), graph[dest]);
    }

    #[test]
    fn test_dyn_astar_adjusted_weight_path() {
        let mut graph = UnGraph::new_undirected();
        let start = graph.add_node(vec2(100.0, 0.0));
        let dest = graph.add_node(vec2(100.0, 100.0));
        let path_nodes = dyn_astar(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| Some(a.distance(b) + b.x.abs()),
            start,
            dest,
            10.,
        );
        let path = path_nodes.map(|node| graph[*node].to_point());
        serialize_to(&path, "dyn_astar_adjusted_path.json");

        let all_nodes = Vec::from_iter(
            graph.node_indices().map(|index| graph[index])
        );
        serialize_to(&all_nodes, "dyn_astar_adjusted_all_nodes.json");

        assert_gt!(path.len(), 0);
        assert_vec2_near!(path.last().unwrap().to_vec(), graph[dest]);
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

        let path_nodes = dyn_astar(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| Some(a.distance(b) * 2.),
            start,
            dest,
            10.,
        );
        let path = path_nodes.map(|node| graph[*node].to_point());
        serialize_to(&path, "dyn_astar_with_previous_edges_path.json");

        let all_nodes = Vec::from_iter(
            graph.node_indices().map(|index| graph[index])
        );
        serialize_to(&all_nodes, "dyn_astar_with_previous_edges_all_nodes.json");

        assert_gt!(path.len(), 0);
        assert_vec2_near!(path.last().unwrap().to_vec(), graph[dest]);
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

        let path_nodes = dyn_astar(
            &mut graph,
            Rect::centered_with_radius(vec2(0., 0.), 1000.),
            |a, b| Some(a.distance(b) * 1.5),
            start,
            dest,
            10.,
        );
        let path = path_nodes.map(|node| graph[*node].to_point());
        serialize_to(&path, "dyn_astar_obstacle_path.json");

        let all_nodes = Vec::from_iter(
            graph.node_indices().map(|index| graph[index])
        );
        serialize_to(&all_nodes, "dyn_astar_obstacle_all_nodes.json");

        assert_gt!(path.len(), 0);
        assert_vec2_near!(path.last().unwrap().to_vec(), graph[dest]);
    }

    #[test]
    fn test_frontier_order_in_bin_heap() {
        let mut heap = BinaryHeap::new();
        heap.push(FrontierItem { index: NodeIndex::new(0), priority: -1.0 });
        heap.push(FrontierItem { index: NodeIndex::new(1), priority: -2.0 });
        assert!(heap.peek().is_some());
        assert_eq!(heap.peek().unwrap().index, NodeIndex::new(0));
    }
}
