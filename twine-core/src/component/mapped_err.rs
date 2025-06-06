use std::{error::Error as StdError, marker::PhantomData};

use super::Component;

/// A wrapper that transforms a component's error type.
///
/// Internally used by `.map_err()` to map one error type to another.
pub(crate) struct MappedErr<C, ErrorMap, NewError>
where
    C: Component,
    ErrorMap: Fn(C::Error) -> NewError,
    NewError: StdError + Send + Sync + 'static,
{
    component: C,
    error_map: ErrorMap,
    _marker: PhantomData<NewError>,
}

impl<C, ErrorMap, NewError> MappedErr<C, ErrorMap, NewError>
where
    C: Component,
    ErrorMap: Fn(C::Error) -> NewError,
    NewError: StdError + Send + Sync + 'static,
{
    /// Creates a new component with an adapted error type.
    pub(crate) fn new(component: C, error_map: ErrorMap) -> Self {
        Self {
            component,
            error_map,
            _marker: PhantomData,
        }
    }
}

impl<C, ErrorMap, NewError> Component for MappedErr<C, ErrorMap, NewError>
where
    C: Component,
    ErrorMap: Fn(C::Error) -> NewError,
    NewError: StdError + Send + Sync + 'static,
{
    type Input = C::Input;
    type Output = C::Output;
    type Error = NewError;

    /// Calls the wrapped component and transforms the error.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.component.call(input).map_err(&self.error_map)
    }
}
