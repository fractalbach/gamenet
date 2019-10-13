//! Module containing river procedural generation structs
//! and functions.
mod common;
mod hex;
mod river_graph;
mod segment;

use std::f64;
use std::collections::{VecDeque, HashSet, HashMap};
use std::usize;

use cgmath::{Vector2, Vector3};
use cgmath::InnerSpace;
use lru_cache::LruCache;

use tectonic::{TectonicLayer, TectonicInfo};
use util::{hash_indices, sphere_uv_vec};
use river::common::RiverInfo;
use river::hex::HexGraph;
use river::river_graph::{RiverGraph, Node};
use river::segment::Segment;


// --------------------------------------------------------------------


/// Struct handling generation of major river effects on the map
///
/// The RiverInfo produced by this layer includes an updated height
/// value that includes the effects of river valley formation, as well
/// as other major-river related information.
///
/// The .height() method is the main public method exposed by
/// a RiverLayer. When passed a position, and TectonicInfo struct, the
/// RiverLayer will generate or retrieve a river Region, and then yield
/// a RiverInfo struct instance for that position.
pub struct RiverLayer {
    seed: u32,
    region_cache: LruCache<Vector3<i64>, Region>,
}

/// River region.
///
/// A River region is associated with a single tectonic cell, and
/// handles height generation due to river action within its bounds.
///
/// A Tectonic cell is an ideal boundary for a river region because it
/// is likely to either border an ocean, or else be bordered by a
/// mountain range which would realistically separate river basins.
struct Region {
    graph: RiverGraph
}


// --------------------------------------------------------------------


impl RiverLayer {
    pub const REGION_CACHE_SIZE: usize = 1_00;

    // Construction

    fn new(seed: u32) -> RiverLayer {
        RiverLayer {
            seed,
            region_cache: LruCache::new(Self::REGION_CACHE_SIZE),
        }
    }

    // Height methods

    /// Produces height and related information for a position.
    ///
    /// # Arguments
    /// * `v` - Position in 3d space relative to world center.
    ///             Will be normalized.
    /// * `tectonic_info` - Tectonic information for the passed point
    /// * `tectonic` - Mutable reference to tectonic layer.
    ///
    /// # Returns
    /// RiverInfo containing height and related information.
    pub fn height(
            &mut self,
            v: Vector3<f64>,
            tectonic_info: TectonicInfo,
            tectonic: &mut TectonicLayer,
    ) -> RiverInfo {
        let indices: Vector3<i64> = tectonic_info.indices;
        if !self.region_cache.contains_key(&indices) {
            let region_hash = hash_indices(self.seed, indices);
            let region = Region::new(region_hash, tectonic, tectonic_info);
            self.region_cache.insert(indices, region);
        }
        let region = self.region_cache.get_mut(&indices).unwrap();
        region.height(v)
    }
}


// --------------------------------------------------------------------


impl Region {
    pub const NODE_MEAN_SEPARATION: f64 = 10_000.0;
    const CONTROL_POINT_DIST: f64 = Self::NODE_MEAN_SEPARATION * 0.2;

    fn new(
        seed: u32,
        tectonic: &mut TectonicLayer,
        tectonic_info: TectonicInfo,
    ) -> Region {
        // Get nucleus surface position mapping
        let center3d = tectonic.surface.surf_pos(
            tectonic_info.nucleus.normalize()
        );
        let (u_vec, v_vec) = sphere_uv_vec(center3d);

        // Create river nodes.
        let nodes = Self::create_nodes(seed, tectonic, tectonic_info);
        let mouths = Self::find_mouths(&nodes);

        Region {
            graph: RiverGraph::new(nodes, &mouths),
        }
    }

    /// Creates nodes that lie within a river region.
    ///
    /// # Arguments
    /// * `seed` - Seed for node graph.
    /// * `tectonic` - Mutable reference to TectonicLayer.
    /// * `tectonic_info` - used to indicate the region.
    ///
    /// # Return
    /// Vec of nodes with all fields except inlets, outlet, and
    /// strahler set.
    fn create_nodes(
        seed: u32,
        tectonic: &mut TectonicLayer,
        tectonic_info: TectonicInfo,
    ) -> Vec<Node> {
        /// Finds the first hex index contained within the cell.
        ///
        /// # Arguments
        /// * `tectonic` - Tectonic layer whose cell is being explored.
        /// * `cell_indices` - Indices of cell being explored.
        /// * `hex_graph`- HexGraph used to generate base
        ///             node positions.
        ///
        /// # Return
        /// HexGraph indices of node within the cell from which
        /// exploration will start.
        fn find_first(
            tectonic: &mut TectonicLayer,
            cell_indices: Vector3<i64>,
            hex_graph: &HexGraph
        ) -> Vector2<i64> {
            // TODO: Ensure first node is in cell. If not, do quick search.
            Vector2::new(0, 0)
        }

        /// Runs BFS Search until all nodes that are in cell are added
        /// to nodes Vec.
        ///
        /// Included indices are added to included set for
        /// quick checking.
        ///
        /// # Arguments
        /// * `tectonic` - Reference to tectonic layer used to generate
        ///             heights for nodes.
        /// * `cell_indices` - Indices of cell which is being explored.
        /// * `hex_graph` - HexGraph used to generate nodes and lookup
        ///             node positions.
        /// * `first` - HexGraph indices of node at which to
        ///             start exploration.
        ///
        /// # Return
        /// * Vec of Nodes which are contained by cell.
        /// * HashMap with the index of each node in the Node Vec,
        ///             stored with the node's HexGraph indices as key.
        fn explore_cell(
            tectonic: &mut TectonicLayer,
            cell_indices: Vector3<i64>,
            hex_graph: &HexGraph,
            first: Vector2<i64>,
        ) -> (Vec<Node>, HashMap<Vector2<i64>, usize>) {
            let mut nodes = Vec::new();
            let mut included = HashMap::with_capacity(100);
            let mut frontier = VecDeque::with_capacity(100);
            let mut visited = HashSet::with_capacity(100);
            frontier.push_back(first);
            visited.insert(first);
            while !frontier.is_empty() {
                let indices = frontier.pop_front().unwrap();
                let uv = hex_graph.pos(indices);  // TODO: randomize
                let xyz = Region::uv_to_xyz_norm(uv);
                let node_info = tectonic.height(xyz);
                if node_info.indices != cell_indices {
                    continue;
                }

                // Add indices to included map and append node to vec.
                included.insert(indices, nodes.len());
                let node_i = nodes.len();
                nodes.push(Node::new(
                    node_i,
                    indices,
                    uv,
                    node_info.height,
               ));

                // Add hex neighbors to frontier.
                for hex_neighbor in &hex_graph.neighbors(indices) {
                    if !visited.contains(&hex_neighbor) {
                        frontier.push_back(*hex_neighbor);
                    }
                }
            }

            (nodes, included)
        }

        /// Sets node neighbors.
        ///
        /// These are the nodes which are within the cell. If a cell has
        /// fewer than three neighbors, one or more index will be set to
        /// -1 (since usize is unsigned, this will be usize::MAX)
        ///
        /// The nodes within the neighbors are ordered clockwise.
        ///
        /// This function modifies the nodes in-place. It does not
        /// return a useful value.
        ///
        /// # Arguments
        /// * `nodes` - Vec of nodes in cell.
        /// * `included` - Map of Node Vec indices stored by their
        ///             HexGraph indices.
        /// * `hex_graph` - HexGraph used to generate nodes.
        ///
        fn set_neighbors(
                nodes: &mut Vec<Node>,
                included: HashMap<Vector2<i64>, usize>,
                hex_graph: &HexGraph,
        ) {
            for node in nodes {
                let hex_neighbors = hex_graph.neighbors(node.indices);
                for (i, neighbor_indices) in hex_neighbors.iter().enumerate() {
                    // `included` contains node index stored by hex indices key.
                    let node_index = included.get(neighbor_indices);
                    if node_index.is_some() {
                        node.neighbors[i] = *node_index.unwrap();
                    }
                }
            }
        }

        // ------------------------

        // Create hex graph to produce nodes.
        let hex_graph = HexGraph::new(Self::NODE_MEAN_SEPARATION);

        // Find center node.
        let first = find_first(tectonic, tectonic_info.indices, &hex_graph);

        // Find nodes in cell.
        let (mut nodes, included) = explore_cell(
            tectonic,
            tectonic_info.indices,
            &hex_graph,
            first,
        );
        set_neighbors(&mut nodes, included, &hex_graph);

        nodes
    }

    /// Finds river mouth nodes.
    ///
    /// River mouth nodes are nodes that are within an ocean but which
    /// are adjacent to one or more nodes on land.
    ///
    /// # Arguments
    /// * `nodes` River nodes to search for mouths.
    ///
    /// # Return
    /// Vec of river node indices that are river mouths.
    fn find_mouths(nodes: &Vec<Node>) -> Vec<usize> {
        let mut mouths = Vec::new();
        for (i, node) in nodes.iter().enumerate() {
            // If node is not in an ocean, continue search.
            if node.h >= 0.0 {
                continue;
            }

            // Check if any neighbor is on land.
            for neighbor in &node.neighbors {
                if *neighbor != usize::MAX && nodes[*neighbor].h >= 0.0 {
                    mouths.push(i);
                    break;
                }
            }
        }

        mouths
    }

    // --------------

    /// Gets height at passed position
    ///
    /// # Arguments
    /// * `v` - Position relative to world center.
    fn height(&self, v: Vector3<f64>) -> RiverInfo {
        let uv = self.xyz_to_uv(v);
        let (d, nearest_seg) = self.nearest_segment(uv);

        RiverInfo {
            height: 0.0, // TODO: Replace placeholder
        }
    }

    /// Converts a 3d position vector to a 2d uv position.
    ///
    /// The produced vector can be used to identify a position
    /// in the 2d river graph.
    ///
    /// # Arguments
    /// * `v` - Position relative to world center.
    ///
    /// # Return
    /// 2d UV position in plane tangential to region origin.
    fn xyz_to_uv(&self, v: Vector3<f64>) -> Vector2<f64> {
        Vector2::new(0.0, 0.0)  // Todo: Replace placeholder
    }

    /// Converts a uv position vector to a 3d world position.
    ///
    /// The produced vector identifies a point in 3d space relative
    /// to the world center.
    ///
    /// # Arguments:
    /// * `uv` - 2d uv position vector.
    ///
    /// # Return
    /// Normalized vector identifying point on surface of world sphere.
    fn uv_to_xyz_norm(uv: Vector2<f64>) -> Vector3<f64> {
        Vector3::new(0.0, 0.0, 0.0)  // Todo: Replace placeholder
    }

    /// Finds the nearest river segment to a position.
    ///
    /// # Arguments:
    /// * `uv` - Position in uv space relative to Region center.
    ///
    /// # Returns
    /// * Distance to nearest segment.
    /// * Segment nearest the passed point.
    fn nearest_segment(&self, uv: Vector2<f64>) -> (f64, Segment) {
        let node= Node::new(
            usize::MAX,
            Vector2::new(-1, -1),
            Vector2::new(-1.0, -1.0),
            -1.0
        );
        (  // TODO: Get distance, nearest segment.
            -1.0,
            // TODO: Replace placeholder.
            Segment::new(&node, &node)
        )
    }
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use cgmath::Vector2;

    use river::*;
    use river::river_graph::Node;

    // ----------------------------------------------------------------
    // Region

    #[test]
    fn test_find_mouths() {
        let nodes = vec!(
            Node {
                neighbors: [1, 2, 3],
                ..Node::new(
                    0,
                    Vector2::new(0, 0),
                    Vector2::new(0.0, 0.0),
                    -13.0
                )
            },
            Node {
                neighbors: [0, 2, usize::MAX],
                ..Node::new(
                    1,
                    Vector2::new(0, 0),
                    Vector2::new(0.0, 0.0),
                    -24.0
                )
            },
            Node {
                neighbors: [0, 1, 3],
                ..Node::new(
                    2,  // Index of Node in river graph.
                    Vector2::new(0, 0),
                    Vector2::new(0.0, 0.0),
                    -11.0,  // Height above mean sea level.
                )
            },
            Node {
                neighbors: [0, 2, usize::MAX],
                ..Node::new(
                    3,  // Index of Node in river graph.
                    Vector2::new(0, 0),
                    Vector2::new(0.0, 0.0),
                    18.0,  // Height above mean sea level.
                )
            }
        );

        let mouths = Region::find_mouths(&nodes);

        assert_eq!(mouths, vec!(0, 2));
    }
}
