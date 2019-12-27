//! Town generation module.
//!
//! Produces Town/City street and building layout.
//!
use cgmath::{Vector2};

use pop::streets::builder::Builder;
use pop::streets::map::{TownMap, TownMapSettings};
use pop::streets::radial_builder::{RadialBuilder, StreetBuilderSettings};


pub struct TownPlanSettings<'a> {
    pub map_settings: TownMapSettings,
    pub street_settings: StreetBuilderSettings<'a>
}


/// Struct containing town layout information.
///
/// TownPlan is higher level than TownMap, which contains the raw data
/// (nodes, edges, obstacles, etc) produced by the TownPlan.
pub struct TownPlan {
    map: TownMap
}

impl TownPlan {
    const DEFAULT_SETTINGS: TownPlanSettings<'static> = TownPlanSettings {
        map_settings: TownMapSettings {
            node_merge_dist: 0.1,
        },
        street_settings: StreetBuilderSettings {
            base_edge_len: 100.0,
            max_edge_len_ratio: 1.5,
            min_edge_len_ratio: 0.5,
            cost_mod_fn: &Self::default_cost_fn,
        }
    };

    pub fn new(settings: TownPlanSettings) -> TownPlan {
        let mut map = TownMap::new(settings.map_settings);

        RadialBuilder::new(settings.street_settings.clone()).build(&mut map);

        TownPlan { map }
    }

    pub fn default() -> TownPlan {
        Self::new(Self::DEFAULT_SETTINGS)
    }

    // Accessors

    fn mut_map(&mut self) -> &mut TownMap {
        &mut self.map
    }

    fn default_cost_fn(a: Vector2<f64>, b: Vector2<f64>) -> f64 { 1.0  }
}


#[cfg(test)]
mod tests {
    use std::fs;

    use cgmath::{vec2, Vector2};

    use pop::streets::builder::StreetSegmentBuilder;
    use pop::streets::map::{TownMap, TownMapSettings, Node};
    use pop::streets::plan::{TownPlan, TownPlanSettings};
    use pop::streets::radial_builder::{RadialBuilder, StreetBuilderSettings};

    fn default_cost_fn(a: Vector2<f64>, b: Vector2<f64>) -> f64 { 1.0 }

    fn get_default_test_settings() -> TownPlanSettings<'static> {
        TownPlanSettings {
            map_settings: TownMapSettings {
                node_merge_dist: 0.1,
            },
            street_settings: StreetBuilderSettings {
                base_edge_len: 25.0,
                max_edge_len_ratio: 1.5,
                min_edge_len_ratio: 0.5,
                cost_mod_fn: &default_cost_fn,
            }
        }
    }

    #[test]
    fn test_simple_initialization() {
        let mut plan = TownPlan::new(get_default_test_settings());
        let mut map = plan.mut_map();
        let step = 50.0;
        for i in -50..50 {
            let u0 = i as f64 * step;
            let u1 = u0 - step;
            let v = 0.0;

            let mut road_segment =
                StreetSegmentBuilder::new(vec2(u0, v), vec2(u1, v), 1.0);
            map.add(&mut road_segment);
        }
        assert!(true);  // Placeholder

        // Serialize map.
        let s = serde_json::to_string_pretty(&map).unwrap();
        fs::write("test_simple_town_graph.json", &s).expect("Unable to write");
    }
}
