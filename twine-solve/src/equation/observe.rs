/// Receives solver events and decides how the iteration should proceed.
///
/// Observers let callers monitor or steer a solver without changing its API,
/// enabling logging, early stopping, or custom control policies.
///
/// Closures automatically implement `Observer`, and a built-in impl for `()`
/// provides a no-op observer that always returns `Decision::Continue`.
pub trait Observer<E, A> {
    fn observe(&mut self, event: &E) -> Decision<A>;
}

/// Observer decisions for solver iteration.
pub enum Decision<A> {
    Continue,
    Action(A),
    Stop,
}

/// Blanket implementation for observer closures.
impl<E, A, F> Observer<E, A> for F
where
    F: FnMut(&E) -> Decision<A>,
{
    fn observe(&mut self, event: &E) -> Decision<A> {
        self(event)
    }
}

/// A no-op observer that always returns `Decision::Continue`.
impl<E, A> Observer<E, A> for () {
    fn observe(&mut self, _event: &E) -> Decision<A> {
        Decision::Continue
    }
}
