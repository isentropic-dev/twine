use std::marker::PhantomData;

use crate::Component;

use super::TwineError;

/// A component that returns its input unchanged.
///
/// This serves as the required starting point for all [`Twine`] chains.
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
    type Error = TwineError;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(input)
    }
}
