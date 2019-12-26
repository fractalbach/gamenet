//! Module containing StreetBuilder which produces street map.
//!
//! Levels:
//! * Quarter
//! * Block
//!
use std::collections::VecDeque;

use cgmath::{Vector2, vec2};
use cgmath::InnerSpace;

use pop::streets::builder::{Builder, StreetSegmentBuilder};
use pop::streets::map::{TownMap, NodeId, Node};


#[derive(Clone, Debug)]
pub struct StreetBuilderSettings {
    pub base_edge_len: f64,
    pub max_edge_len_ratio: f64,
    pub min_edge_len_ratio: f64,
    // As const settings are required, they should be added here.
}

#[derive(Debug)]
pub struct StreetBuilder {
    settings: StreetBuilderSettings,
    quarter_builder: QuarterBuilder,
    block_builder: BlockBuilder,
}

/// Struct responsible for building quarters in a town/city.
///
/// Produces Major-order streets radiating outwards, then minor-order
/// streets to connect them. The areas enclosed by the produced streets
/// become quarters and are then assigned characteristics.
#[derive(Debug)]
struct QuarterBuilder {}

#[derive(Debug)]
struct BlockBuilder {}


// --------------------------------------------------------------------


impl StreetBuilder {
    pub fn new(settings: StreetBuilderSettings) -> StreetBuilder {
        StreetBuilder {
            settings,
            quarter_builder: QuarterBuilder::new(),
            block_builder: BlockBuilder::new(),
        }
    }

    pub fn build(&self, map: &mut TownMap) {
        self.quarter_builder.build(map, &self.settings);
        self.block_builder.build(map, &self.settings);
    }
}


// --------------------------------------------------------------------


impl QuarterBuilder {
    fn new() -> QuarterBuilder { QuarterBuilder {} }

    fn build(&self, map: &mut TownMap, settings: &StreetBuilderSettings) {
        // TODO Produce major vector street start nodes
        // Iterate over all road nodes until land value falls
        // below threshold.

        // TODO Form major vector streets along value falloff direction.
        // Continue until ...
        //    * Street is blocked.
        //    * Another road or street is encountered.
        //    * Land value falls below threshold.

        // TODO Form minor vector streets across value falloff direction.

    }

    fn find_major_vector_start_nodes(
        &self, map: &TownMap, settings: &StreetBuilderSettings
    ) -> Vec<NodeId> {
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

    fn create_minor_vector_streets(
        &self, map: &TownMap, settings: &StreetBuilderSettings
    ) -> Vec<StreetSegmentBuilder> {
        vec!()
    }
}

impl BlockBuilder {
    fn new() -> BlockBuilder { BlockBuilder {} }

    fn build(&self, map: &mut TownMap, settings: &StreetBuilderSettings) {
        // TODO
    }
}
