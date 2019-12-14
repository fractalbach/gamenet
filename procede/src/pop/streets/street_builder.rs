//! Module containing StreetBuilder which produces street map.
//!
//! Levels:
//! * District
//! * Block
//!

use cgmath::{Vector2, vec2};

use pop::streets::builder::Builder;


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
}


impl StreetBuilder {
    pub fn new(settings: StreetBuilderSettings) -> StreetBuilder {
        StreetBuilder {
            settings,
        }
    }
}
