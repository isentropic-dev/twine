use std::marker::PhantomData;

use crate::Component;

/// A no-op component that returns its input unchanged.
pub(crate) struct Identity<T> {
    _marker: PhantomData<T>,
}

impl<T> Identity<T> {
    /// Creates a new identity component.
    pub(crate) const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Component for Identity<T> {
    type Input = T;
    type Output = T;

    fn call(&self, input: T) -> T {
        input
    }
}
