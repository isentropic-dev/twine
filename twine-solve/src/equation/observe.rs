/// Receives solver events and decides how the iteration should proceed.
///
/// Observers let callers monitor or steer a solver without changing its API,
/// enabling logging, early stopping, or custom control policies.
///
/// The `observe` method returns `Option<A>`, where `Some(action)` requests a
/// solver-specific action and `None` lets the solver continue unchanged.
///
/// Closures automatically implement `Observer`, and a built-in impl for `()`
/// provides a no-op observer that always returns `None`.
pub trait Observer<E, A> {
    /// Observes a solver event and optionally returns a control action.
    fn observe(&mut self, event: &E) -> Option<A>;
}

/// Blanket implementation for observer closures.
impl<E, A, F> Observer<E, A> for F
where
    F: FnMut(&E) -> Option<A>,
{
    fn observe(&mut self, event: &E) -> Option<A> {
        self(event)
    }
}

/// A no-op observer that always returns `None`.
impl<E, A> Observer<E, A> for () {
    fn observe(&mut self, _event: &E) -> Option<A> {
        None
    }
}
