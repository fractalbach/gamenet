//! Module containing Radial which produces the major-order roads
//! radiating outwards from the city center (generally) and/or away from
//! traversing roadways.
use std::collections::{VecDeque, HashSet};
use std::fmt;

use cgmath::{Vector2, vec2};
use cgmath::MetricSpace;
use cgmath::InnerSpace;

use pop::streets::builder::{Builder, StreetSegmentBuilder};
use pop::streets::map::{TownMap, NodeId, Node};
use pop::streets::open_dir::OpenDir;
use pop::streets::poly::Poly;
use util;


#[derive(Clone)]
pub struct StreetBuilderSettings<'a> {
    pub base_edge_len: f64,
    pub max_edge_len_ratio: f64,
    pub min_edge_len_ratio: f64,
    pub base_min_influence: f64,
    pub min_fork_angle: f64,
    pub cost_mod_fn: &'a dyn Fn(Vector2<f64>, Vector2<f64>) -> f64,
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
    settings: &'a StreetBuilderSettings<'a>,
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
    pub fn new(settings: &'a StreetBuilderSettings) -> RadialBuilder<'a> {
        RadialBuilder {
            settings,
            sections: vec!(),
        }
    }
}

impl<'a> Builder for RadialBuilder<'a> {
    /// Adds nodes, edges, and obstacle lines for roads radiating
    /// outward from centers of influence in a town.
    fn build(&mut self, map: &mut TownMap) {
        // TODO
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

