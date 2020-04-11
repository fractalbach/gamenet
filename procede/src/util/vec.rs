use std::iter::FromIterator;

/// Additional utility operations for Polygon.
pub trait VecMap<T> {
    fn map<B, F>(&self, f: F) -> Vec<B> where F: FnMut(&T) -> B;
}

impl<T> VecMap<T> for Vec<T> {
    fn map<B, F>(&self, f: F) -> Vec<B> where F: FnMut(&T) -> B {
        Vec::from_iter(self.iter().map(f))
    }
}
