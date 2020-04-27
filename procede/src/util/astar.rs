//! Module containing astar utilities for use with various graphs.

use std::collections::{BinaryHeap, HashMap};
use std::f64::consts;
use std::hash::{Hash, Hasher};
use std::ops::Index;

use cgmath::Vector2;
use petgraph::graph::{UnGraph, NodeIndex};
use petgraph::visit::{IntoNeighbors, IntoEdges, Visitable};

use quad::{QuadMap, ItemId, Rect, Spatial};
use util::vec2::VecOps;


/// Finds optimal route from start to dest, adding nodes where needed.
// pub fn dyn_astar<H, W>(
//     graph: &mut UnGraph<Vector2<f64>, f64>,
//     bounds: Rect,
//     weight: W,
//     start: NodeIndex,
//     dest: NodeIndex,
// ) -> Vec<NodeIndex>
// where
//     W: FnMut(Vector2<f64>, Vector2<f64>) -> f64,
// {
//     // Create structures
//     let mut map = QuadMap::default(bounds);
//     let mut frontier = BinaryHeap::with_capacity(graph.node_count());
//     let mut previous_nodes = HashMap::new();
//     let mut costs = HashMap::new();
//
//     // Initialize map.
//     for index in graph.node_indices() {
//         map.insert(graph.index(index));
//     }
//
//     // Initialize.
//     frontier.push(start);
//     costs.insert(start, 0.);
//
//     // Search
//     while !frontier.is_empty() {
//         let current = frontier.pop();
//         if current == dest {
//             break;
//         }
//
//         for next in
//     }
// }

fn neighbors(
    graph: &mut Ungraph<Vector2<f64>, f64>,
    node: NodeIndex,
    map: &mut QuadMap<vector2<f64>>,
    preferred_dir: &Vec<Vector2<f64>>,
    samples: u32,
) {
    
}
