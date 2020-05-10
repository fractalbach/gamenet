use std::f64;
use std::collections::{VecDeque, HashSet, HashMap};
use std::sync::Arc;
use std::usize;

use cgmath::{Vector2, Vector3, vec2, vec3};
use cgmath::InnerSpace;
use serde::Serialize;

use tectonic::{TectonicLayer, TectonicInfo};
use util::{hash_indices, sphere_uv_vec, TangentPlane};
use util::cache::InteriorCache;
use river::common::RiverInfo;
use river::hex::HexGraph;
use river::river_graph::{RiverGraph, Mouth, Node, NearRiverInfo, RiverSettings};
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
    region_cache: InteriorCache<Vector3<i64>, Region>,
}

/// River region.
///
/// A River region is associated with a single tectonic cell, and
/// handles height generation due to river action within its bounds.
///
/// A Tectonic cell is an ideal boundary for a river region because it
/// is likely to either border an ocean, or else be bordered by a
/// mountain range which would realistically separate river basins.
#[derive(Serialize)]
struct Region {
    nucleus: Vector3<f64>,
    graph: RiverGraph
}


// --------------------------------------------------------------------


impl RiverLayer {
    pub const REGION_CACHE_SIZE: usize = 100;

    // Construction

    fn new(seed: u32) -> RiverLayer {
        RiverLayer {
            seed,
            region_cache: InteriorCache::new(Self::REGION_CACHE_SIZE),
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
        &self,
        v: Vector3<f64>,
        tectonic_info: TectonicInfo,
        tectonic: &mut TectonicLayer,
    ) -> RiverInfo {
        self.region(v, tectonic_info, tectonic).height(v)
    }

    /// Gets Region for passed position vector and tectonic data.
    ///
    /// If Region has not yet been created, it is generated. If it has
    /// already been generated, a reference to the cached region
    /// is provided.
    ///
    /// # Arguments
    /// * `v` - Position in 3d space relative to world center.
    ///             Will be normalized.
    /// * `tectonic_info` - Tectonic information for the passed point
    /// * `tectonic` - Mutable reference to tectonic layer.
    ///
    /// # Returns
    /// Mutable Region reference.
    fn region(
        &self,
        v: Vector3<f64>,
        tectonic_info: TectonicInfo,
        tectonic: &mut TectonicLayer,
    ) -> Arc<Region> {
        let indices: Vector3<i64> = tectonic_info.indices;
        self.region_cache.get(&indices, ||{
            let region_hash = hash_indices(self.seed, indices);
            Region::new(region_hash, tectonic, tectonic_info.clone())
        })
    }
}


// --------------------------------------------------------------------


impl Region {
    pub const NODE_MEAN_SEPARATION: f64 = 10_000.0;
    const MIN_STRAHLER: i8 = 2;
    const BIAS_MAGNITUDE: f64 = 0.05;  // Should be < 0.1

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
        let nodes = Self::create_nodes(seed, tectonic, &tectonic_info);
        let mouths = Self::find_mouths(&nodes);

        let settings =  RiverSettings { max_influence_r: 4000.0 };

        Region {
            nucleus: tectonic_info.nucleus,
            graph: RiverGraph::new(
                nodes, &mouths, Self::MIN_STRAHLER, settings
            ),
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
        tectonic_info: &TectonicInfo,
    ) -> Vec<Node> {

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
            tectonic_info: &TectonicInfo,
            hex_graph: &HexGraph,
            origin: Vector3<f64>,
        ) -> (Vec<Node>, HashMap<Vector2<i64>, usize>) {
            {
                let xyz = TangentPlane::new(origin).xyz(vec2(0.0, 0.0));
                let first_indices = tectonic.height(xyz).indices;
                debug_assert!(tectonic_info.indices == first_indices);
            }
            let mut nodes = Vec::new();
            let mut included = HashMap::with_capacity(100);
            let mut frontier = VecDeque::with_capacity(100);
            let mut visited = HashSet::with_capacity(100);
            let first = vec2(0, 0);
            frontier.push_back(first);  // Add first.
            visited.insert(first);
            while !frontier.is_empty() {
                // TODO: Debug
                let indices = frontier.pop_front().unwrap();
                let uv = hex_graph.pos(indices);  // TODO: randomize.
                let xyz = TangentPlane::new(origin).xyz(uv);
                let node_info = tectonic.height(xyz);
                if node_info.indices != tectonic_info.indices {
                    continue;
                }

                // Add indices to included map and append node to vec.
                debug_assert!(!included.contains_key(&indices));
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
                    if !visited.contains(hex_neighbor) {
                        // Add to 'visited' set here, so that the same
                        // set of indices are not added more than once.
                        visited.insert(*hex_neighbor);
                        frontier.push_back(*hex_neighbor);
                    }
                }
            }

            // TODO: Log max frontier, visited, and included capacities.

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

        // Find center node position.
        let origin = tectonic.raw_v(tectonic_info.nucleus);

        // Find nodes in cell.
        let (mut nodes, included) = explore_cell(
            tectonic,
            &tectonic_info,
            &hex_graph,
            origin,
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
    fn find_mouths(nodes: &Vec<Node>) -> Vec<Mouth> {
        // TODO:
        // Find Plate-wide bias.
        // This provides a somewhat more consistent direction for rivers.

        let mut mouths = Vec::with_capacity(100);
        for (i, node) in nodes.iter().enumerate() {
            // If node is not in an ocean, continue search.
            if node.h >= 0.0 {
                continue;
            }

            // Check if any neighbor is on land.
            for &neighbor in &node.neighbors {
                if neighbor != usize::MAX && nodes[neighbor].h >= 0.0 {

                    let dir = (nodes[neighbor].uv - node.uv).normalize();
                    let bias = dir * Self::BIAS_MAGNITUDE;

                    mouths.push(Mouth{i, bias});
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
        let graph_info = self.graph.info(uv);

        assert!(false);
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
        assert!(false);
        vec2(0.0, 0.0)  // Todo: Replace placeholder
    }
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use std::collections::{HashSet, HashMap};
    use std::fs;

    use cgmath::{Vector2, Vector3, vec2, vec3};
    use serde_json;

    use tectonic::{TectonicLayer, TectonicInfo};
    use test_util::serialize_to;
    use river::major::*;
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
                    vec2(0, 0),
                    vec2(0.0, 0.0),
                    -13.0
                )
            },
            Node {
                neighbors: [0, 2, usize::MAX],
                ..Node::new(
                    1,
                    vec2(0, 0),
                    vec2(0.0, 0.0),
                    -24.0
                )
            },
            Node {
                neighbors: [0, 1, 3],
                ..Node::new(
                    2,  // Index of Node in river graph.
                    vec2(0, 0),
                    vec2(0.0, 0.0),
                    -11.0,  // Height above mean sea level.
                )
            },
            Node {
                neighbors: [0, 2, usize::MAX],
                ..Node::new(
                    3,  // Index of Node in river graph.
                    vec2(0, 0),
                    vec2(0.0, 0.0),
                    18.0,  // Height above mean sea level.
                )
            }
        );

        let mouths = Region::find_mouths(&nodes);

        assert_eq!(0, mouths[0].i);
        assert_eq!(2, mouths[1].i);
    }

    #[test]
    fn test_region_graph() {
        let mut tectonic = TectonicLayer::new(13);
        let mut river = RiverLayer::new(13);

        // Find good tectonic cell with height above sea level.
        let mut tectonic_info = TectonicInfo {
            height: -1.0,
            indices: vec3(0, 0, 0),
            nucleus: vec3(0.0, 0.0, 0.0),
            mod_input: vec3(0.0, 0.0, 0.0),
        };
        let mut v = vec3(0.0, 0.0, 0.0);
        for (x, y, z) in iproduct!(-10..11, -10..11, -10..11) {
            if x == 0 && y == 0 && z == 0 {
                continue;
            }
            v = vec3(x as f64, y as f64, z as f64);
            tectonic_info = tectonic.height(v);
            if tectonic_info.height > 0.0 {
                break;
            }
        }
        assert_gt!(v.magnitude(), 0.0);
        assert_gt!(tectonic_info.height, 0.0);

        // Get river region
        let region = river.region(v, tectonic_info, &mut tectonic);

        assert_gt!(region.graph.len(), 0);

        // Serialize graph.
        serialize_to(region.as_ref(), "test_region_graph.json");
    }

    #[test]
    fn test_vector_is_unique_in_set() {
        let mut map = HashSet::with_capacity(100);
        let v1 = vec2(1, 2);
        let v2 = vec2(1, 2);
        let v3 = vec2(1, 2);
        map.insert(v1);
        map.insert(v2);
        assert_eq!(1, map.len());
        assert!(map.contains(&v3));
    }

    #[test]
    fn test_vector_is_unique_in_map() {
        let mut map = HashMap::with_capacity(100);
        let v1 = vec2(1, 2);
        let v2 = vec2(1, 2);
        let v3 = vec2(1, 2);
        map.insert(v1, 5);
        map.insert(v2, 7);
        assert_eq!(1, map.len());
        assert!(map.contains_key(&v3));
    }
}
