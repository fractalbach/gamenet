//! Town generation module.
//!
//! Produces Town/City street and building layout.

use pop::streets::builder::Builder;
use pop::streets::map::{TownMap, TownMapSettings};
use pop::streets::street_builder::{StreetBuilder, StreetBuilderSettings};


pub struct TownPlanSettings {
    pub map_settings: TownMapSettings,
    pub street_settings: StreetBuilderSettings
}


pub struct TownPlan {
    map: TownMap
}


#[cfg(test)]
mod tests {
    use cgmath::vec2;

    use pop::streets::builder::StreetSegmentBuilder;
    use pop::streets::map::{TownMap, TownMapSettings, Node};
    use pop::streets::plan::{TownPlan, TownPlanSettings};
    use pop::streets::street_builder::{StreetBuilder, StreetBuilderSettings};

    fn get_default_test_settings() -> TownPlanSettings {
        TownPlanSettings {
            map_settings: TownMapSettings {
                node_merge_dist: 0.1,
            },
            street_settings: StreetBuilderSettings {
                base_edge_len: 100.0,
                max_edge_len_ratio: 1.5,
                min_edge_len_ratio: 0.5,
            }
        }
    }

    #[test]
    fn test_simple_initialization() {
        let step = 100.0;
        for i in 0..100 {
            let u0 = i as f64 * step;
            let u1 = u0 - step;
            let v = 0.0;

            let street_segment =
                StreetSegmentBuilder::new(vec2(u0, v), vec2(u1, v), 1.0);
        }
        assert!(true);  // Placeholder
    }
}
