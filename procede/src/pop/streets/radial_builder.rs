//! Module containing Radial which produces the major-order roads
//! radiating outwards from the city center (generally) and/or away from
//! traversing roadways.
use std::collections::VecDeque;
use std::fmt;

use cgmath::{Vector2, vec2};
use cgmath::InnerSpace;

use pop::streets::builder::{Builder, StreetSegmentBuilder};
use pop::streets::map::{TownMap, NodeId, Node};
use pop::streets::poly::Poly;


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
/// When built on a TownMap, produces a collection of bounds which may
/// be used to generate a town subdivision.
#[derive(Debug)]
pub struct RadialBuilder<'a> {
    settings: StreetBuilderSettings<'a>,
    sections: Vec<Poly>,
}


// --------------------------------------------------------------------


impl<'a> RadialBuilder<'a> {
    pub fn new(settings: StreetBuilderSettings) -> RadialBuilder {
        RadialBuilder {
            settings,
            sections: vec!(),
        }
    }

    fn find_start_nodes(&self, map: &TownMap) -> Vec<NodeId> {
        let mut nodes = vec!();
        let mut frontier = VecDeque::with_capacity(20);

        match Self::find_highest_value_node(map) {
            Some(id) => frontier.push_back(id),
            None    => return nodes
        };

        nodes
    }

    /// Find node with highest influence value.
    fn find_highest_value_node(map: &TownMap) -> Option<NodeId> {
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

    fn build_major_streets(
        &self, map: &mut TownMap, start_nodes: &Vec<NodeId>
    ) -> Vec<NodeId> {
        let mut nodes = vec!();

        for &start in start_nodes {
            nodes.append(&mut self.build_street(map, start));
        }

        nodes
    }

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

