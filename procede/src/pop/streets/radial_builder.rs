//! Module containing Radial which produces the major-order roads
//! radiating outwards from the city center (generally) and/or away from
//! traversing roadways.
use std::collections::VecDeque;
use std::fmt;

use cgmath::{Vector2, vec2};
use cgmath::MetricSpace;
use cgmath::InnerSpace;

use pop::streets::builder::{Builder, StreetSegmentBuilder};
use pop::streets::map::{TownMap, NodeId, Node};
use pop::streets::poly::Poly;
use wasm_bindgen::__rt::std::collections::HashSet;


#[derive(Clone)]
pub struct StreetBuilderSettings<'a> {
    pub base_edge_len: f64,
    pub max_edge_len_ratio: f64,
    pub min_edge_len_ratio: f64,
    pub cost_mod_fn: &'a Fn(Vector2<f64>, Vector2<f64>) -> f64,
    // As const settings are required, they should be added here.
}

/// Struct responsible for building quarters in a town/city.
///
/// Produces Major-order streets radiating outwards, then minor-order
/// streets to connect them. The areas enclosed by the produced streets
/// become quarters.
///
/// When built on a TownMap, returns a collection of bounds which may
/// be used to generate a town subdivision.
#[derive(Debug)]
pub struct RadialBuilder<'a> {
    settings: StreetBuilderSettings<'a>,
    sections: Vec<Poly>,
}


// --------------------------------------------------------------------


impl<'a> RadialBuilder<'a> {
    /// Create new RadialBuilder.
    ///
    /// # Arguments
    /// * `settings` - Generic StreetBuilderSettings.
    ///
    /// # Return
    /// RadialBuilder
    pub fn new(settings: StreetBuilderSettings) -> RadialBuilder {
        RadialBuilder {
            settings,
            sections: vec!(),
        }
    }

    /// Find nodes on existing roads where streets may branch.
    ///
    /// The returned candidate nodes are possible start points for roads
    /// to branch from.
    ///
    /// # Arguments
    /// * `map` - TownMap with existing road nodes and influence field.
    ///
    /// # Return
    /// Vector of NodeId's which are candidates to branch streets from.
    fn find_start_nodes(&self, map: &TownMap) -> Vec<NodeId> {
        let mut nodes = vec!();
        let mut frontier = VecDeque::with_capacity(20);
        let mut visited = HashSet::with_capacity(map.n_nodes());
        let base_edge_len = self.settings.base_edge_len;

        // Start search at node with highest influence.
        match Self::find_highest_value_node(map) {
            Some(id) => frontier.push_back((id, base_edge_len)),
            None    => return nodes
        };

        // Explore existing road nodes finding start nodes.
        while !frontier.is_empty() {
            // Get next unvisited road node.
            let (id, d) = frontier.pop_front().unwrap();
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);

            // TODO: Check influence value at node exceeds threshold.

            // Add node to start node vec if appropriate.
            if d >= base_edge_len {
                nodes.push(id);
            }

            // Add node's connections to frontier.
            let node = map.node(id);
            for &(_edge_id, other_id, other_uv) in node.edges() {
                if visited.contains(&other_id) {
                    continue;
                }
                let d_to_other = node.uv().distance(other_uv);
                frontier.push_back((other_id, d + d_to_other));
            }
        }

        nodes
    }

    /// Find node with highest influence value.
    ///
    /// If multiple nodes have equal influence value, the first NodeId
    /// will be returned.
    ///
    /// # Arguments
    /// * `map` - TownMap with existing road nodes and influence field.
    ///
    /// # Return
    /// Option<NodeId>
    /// * `None` if no nodes exist in map.
    /// * `NodeId` if any node was found.
    fn find_highest_value_node(map: &TownMap) -> Option<NodeId> {
        if map.n_nodes() == 0 {
            return None;
        }
        let (_, (first, _)) = &map.nodes().iter().nth(0)?;
        let mut highest_v = map.value_map().sample(first.uv()).magnitude();
        let mut highest_id = first.id();
        for (_, (node, _)) in map.nodes().iter().skip(1) {
            let v = map.value_map().sample(node.uv()).magnitude();
            if v > highest_v {
                highest_id = node.id();
                highest_v = v;
            }
        }
        Some(highest_id)
    }

    /// Produce major-order streets
    ///
    /// # Arguments
    /// * `map` - TownMap to which streets will be added.
    /// * `start_nodes` - Id's of nodes where streets will branch from.
    ///
    /// # Return
    /// Vec of NodeId's added to the map.
    fn build_major_streets(
        &self, map: &mut TownMap, start_nodes: &Vec<NodeId>
    ) -> Vec<NodeId> {
        let mut nodes = vec!();

        for &start in start_nodes {
            nodes.append(&mut self.build_street(map, start));
        }

        nodes
    }

    /// Produce major-order street
    ///
    /// # Arguments
    /// * `map` - TownMap to which streets will be added.
    /// * `start` - Id of node where street will branch from.
    ///
    /// # Return
    /// Vec of NodeId's added to the map.
    fn build_street(&self, map: &mut TownMap, start: NodeId) -> Vec<NodeId> {
        let nodes = vec!();

        nodes
    }
}

impl<'a> Builder for RadialBuilder<'a> {
    /// Adds nodes, edges, and obstacle lines for roads radiating
    /// outward from centers of influence in a town.
    fn build(&mut self, map: &mut TownMap) {
        // Produce major vector street start nodes
        // Iterate over all road nodes until land value falls
        // below threshold
        let start_nodes = self.find_start_nodes(map);
        if start_nodes.len() == 0 {
            return;
        }

        // TODO Form major vector streets along value falloff direction.
        // Continue until ...
        //    * Street is blocked.
        //    * Another road or street is encountered.
        //    * Land value falls below threshold.
        let major_nodes = self.build_major_streets(map, &start_nodes);

        // TODO Form minor vector streets across value falloff direction.


    }
}

impl<'a> fmt::Debug for StreetBuilderSettings<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "StreetBuilderSettings {{ \
            base_edge_len: {}, min / max edge len ratios: {} / {} \
            }}",
            self.base_edge_len,
            self.min_edge_len_ratio,
            self.max_edge_len_ratio,
        )
    }
}


// --------------------------------------------------------------------

