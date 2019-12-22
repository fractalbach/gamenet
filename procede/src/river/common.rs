use cgmath::Vector2;

/// Struct used to return height and related information about
/// a position from the RiverLayer.
pub struct RiverInfo {
    pub height: f64,
}

pub const MAX_STRAHLER: i8 = 12;

/// Gets base width of a river with a given strahler number.
///
/// The base width is the mean width of the river before noise, curves,
/// or other varying features.
///
/// # Arguments
/// * `strahler` - Strahler number.
///
/// # Return
/// River base width in meters.
pub const fn get_base_width(strahler: i8) -> f64 {
    // Width table based on real-world measurements.
    const LOOKUP: [f64; 13] = [
        1.0,  // 0
        1.5,  // 1
        2.0,  // 2
        5.0,  // 3
        10.0,  // 4
        50.0,  // 5
        100.0,  // 6
        180.0,  // 7
        400.0,  // 8
        800.0,  // 9
        1000.0,  // 10
        2000.0,  // 11
        4000.0,  // 12
    ];

    LOOKUP[strahler as usize]
}


// --------------------------------------------------------------------


#[cfg(test)]
mod tests {
    use river::common::*;

    #[test]
    fn test_base_width() {
        assert_in_range!(0.75, get_base_width(0), 1.5);
        assert_in_range!(1.0, get_base_width(1), 2.0);
        assert_in_range!(5.0, get_base_width(4), 50.0);
        assert_in_range!(700.0, get_base_width(10), 2000.0);
        assert_in_range!(3000.0, get_base_width(12), 8000.0);
    }
}
