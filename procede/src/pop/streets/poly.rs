//! Module containing a utility class for interacting with regions
//! described by a boundary composed of edges.

use cgmath::{Vector2, vec2};
use pop::streets::map::{TownMap, NodeId};


#[derive(Debug)]
pub struct Poly {
    vertices: Vec<Vertex>
}

#[derive(Debug, Clone, Copy)]
pub struct Vertex {
    id: NodeId,
    uv: Vector2<f64>,
}

impl Poly {
    pub fn new(map: &TownMap, bounds: Vec<NodeId>) -> Poly {
        let mut vertices = Vec::new();

        for id in bounds {
            let node = map.node(id);
            let vert = Vertex {
                id: node.id(),
                uv: node.uv(),
            };
            vertices.push(vert);
        }

        Poly { vertices }
    }
}
