//! Module containing a utility iterator for assisting in street growth.
use std::f64;
use std::iter;

use super::map::Node;
use cgmath::{Vector2, Rotation, Rotation2, Basis2, Rad};


/// Utility iterator that yields directions from the node that are
/// candidates for street development.
///
/// Not all returned vectors will be valid. Returned values should be
/// filtered by the user.
pub struct OpenDir<'a> {
    node: &'a Node,
    i: usize,
    max_i: usize,
    offset: f64,
    offset_step: f64,
    un_continued: Vec<usize>
}

impl<'a> OpenDir<'a> {
    pub fn new(node: &'a Node, offset_step: f64) -> OpenDir<'a> {
        debug_assert!(offset_step == offset_step);
        let un_continued = Self::find_un_continued(node);
        OpenDir {
            node,
            i: 0,
            max_i: node.edges().len() + un_continued.len(),
            offset: 0.0,
            offset_step,
            un_continued,
        }
    }

    /// Find Node's edges which have not been continued on the
    /// opposite side of the node by another edge.
    fn find_un_continued(node: &Node) -> Vec<usize> {
        const COS45: f64 = f64::consts::FRAC_1_SQRT_2;
        let mut un_continued = vec!();
        for i in 0..node.edges().len() {
            let reciprocal = node.edge_dir(i) * -1.0;
            let (nearest_i, cos_theta) = node.nearest_edge(reciprocal);
            if cos_theta < COS45 {
                un_continued.push(i);
            }
        }
        un_continued
    }
}

impl Iterator for OpenDir<'_> {
    type Item = Vector2<f64>;

    /// Yields next open direction from the Node.
    ///
    /// Continuations of existing streets will be preferred, after which
    /// directions will be yielded that bisect gaps between edges
    /// connected to the node.
    ///
    /// # Return
    /// Direction vector pointed away from node.
    fn next(&mut self) -> Option<Vector2<f64>> {
        debug_assert!(self.max_i >= 1);

        // If any edge has no pair with a difference in angle of less
        // than 45deg, it will be considered not continued and its
        // direct continuation direction will be yielded first.
        const DEG45: f64 = f64::consts::PI / 4.0;

        let res;
        if self.i < self.un_continued.len() {
            // Return permutation of continuation.
            let continued_edge: usize = self.un_continued[self.i];
            let continuation_dir = -self.node.edge_dir(continued_edge);
            let rot = Basis2::from_angle(Rad(self.offset));
            res = Some(rot.rotate_vector(continuation_dir));
        } else {
            // Return gap mid-line permutation.
            let edge_i = self.i - self.un_continued.len();
            let rot = Basis2::from_angle(
                -Rad(self.node.gap_angle(edge_i) / 2.0 + self.offset)
            );
            res = Some(rot.rotate_vector(self.node.edge_dir(edge_i)));
        }

        self.i += 1;
        if self.i == self.max_i {
            if self.offset >= 0.0 {
                self.offset += self.offset_step;
            } else {
                self.offset *= -1.0;
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use cgmath::{vec2};

    use pop::streets::map::{TownMap, Node};
    use pop::streets::open_dir::OpenDir;


    #[test]
    fn test_open_dir_yields_continuation_of_first_edge_first() {
        let mut map = TownMap::default();
        let a = map.add_node(Node::new(vec2(0.0, 0.0))).id();
        let b = map.add_node(Node::new(vec2(1.0, 0.0))).id();
        let c = map.add_node(Node::new(vec2(1.0, -1.1))).id();
        let d = map.add_node(Node::new(vec2(1.0, 1.0))).id();
        let ab = map.add_edge_between(a, b, 1.0);
        let bc = map.add_edge_between(b, c, 1.0);
        let bd = map.add_edge_between(b, d, 1.0);

        let first = OpenDir::new(map.node(b), 0.1).nth(0);
        assert_eq!(first.unwrap(), vec2(1.0, 0.0));
    }
}
