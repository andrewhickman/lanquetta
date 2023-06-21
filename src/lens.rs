use druid::{Data, Lens};

/// A lens that gives access to a computed value from a type. Does not support writes.
pub struct Project<F> {
    f: F,
}

impl<F> Project<F> {
    pub fn new(f: F) -> Self {
        Project { f }
    }
}

impl<T, U, F> Lens<T, U> for Project<F>
where
    T: Data,
    F: Fn(&T) -> U,
{
    fn with<V, G: FnOnce(&U) -> V>(&self, data: &T, g: G) -> V {
        g(&(self.f)(data))
    }

    fn with_mut<V, G: FnOnce(&mut U) -> V>(&self, data: &mut T, g: G) -> V {
        g(&mut (self.f)(data))
    }
}
