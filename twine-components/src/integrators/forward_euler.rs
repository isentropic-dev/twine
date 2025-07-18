//! A first-order explicit integrator using the forward Euler method.
//!
//! This integrator is best suited for simple dynamic systems where
//! computational efficiency takes priority over numerical accuracy.

use std::{convert::Infallible, marker::PhantomData};

use twine_core::{Component, TimeIntegrable};
use uom::si::f64::Time;

/// A [`Component`] that performs a single forward Euler integration step.
///
/// Takes a `(value, derivative, dt)` tuple and returns the integrated result.
#[derive(Debug, Clone, Copy, Default)]
pub struct ForwardEuler<T> {
    _marker: PhantomData<T>,
}

impl<T> ForwardEuler<T> {
    /// Creates a new [`ForwardEuler`] component.
    #[must_use]
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T: TimeIntegrable> Component for ForwardEuler<T> {
    type Input = (T, T::Derivative, Time);
    type Output = T;
    type Error = Infallible;

    fn call(&self, (value, derivative, dt): Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(value.step(derivative, dt))
    }
}
