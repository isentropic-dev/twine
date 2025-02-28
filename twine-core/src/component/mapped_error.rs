use std::{error::Error as StdError, marker::PhantomData};

use super::Component;

/// A wrapper that adapts a component by transforming its error type.
///
/// This is used internally by `.map_error()` to modify how a component
/// reports errors.
pub(crate) struct MappedError<C, ErrorMap, NewError> {
    component: C,
    error_map: ErrorMap,
    _marker: PhantomData<NewError>,
}

impl<C, ErrorMap, NewError> MappedError<C, ErrorMap, NewError> {
    /// Creates a new component with a transformed error type.
    pub(crate) fn new(component: C, error_map: ErrorMap) -> Self {
        Self {
            component,
            error_map,
            _marker: PhantomData,
        }
    }
}

impl<C, ErrorMap, NewError> Component for MappedError<C, ErrorMap, NewError>
where
    C: Component,
    ErrorMap: Fn(C::Error) -> NewError,
    NewError: StdError + Send + Sync + 'static,
{
    type Input = C::Input;
    type Output = C::Output;
    type Error = NewError;

    /// Calls the wrapped component and applies the error mapping if it fails.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.component
            .call(input)
            .map_err(|error| (self.error_map)(error))
    }
}
