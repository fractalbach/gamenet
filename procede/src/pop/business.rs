

/// A type of generated business
struct Business {
    key: str,
    display_name: str,
    ratio: f32,
}


impl Business {
    pub fn new(k: String, name: String, ratio: f32) -> Business {
        Business {
            key: k,
            display_name: name,
            ratio
        }
    }
}
