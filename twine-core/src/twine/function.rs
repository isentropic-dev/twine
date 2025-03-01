use std::marker::PhantomData;

use crate::Component;

use super::TwineError;

/// A component that integrates functions into [`Twine`] chains.
pub(crate) struct Function<F, I, O>
where
    F: Fn(I) -> O,
{
    function: F,
    _marker: PhantomData<(I, O)>,
}

impl<F, I, O> Function<F, I, O>
where
    F: Fn(I) -> O,
{
    /// Creates a new function-based component.
    pub(crate) const fn new(function: F) -> Self {
        Self {
            function,
            _marker: PhantomData,
        }
    }
}

impl<F, I, O> Component for Function<F, I, O>
where
    F: Fn(I) -> O,
{
    type Input = I;
    type Output = O;
    type Error = TwineError;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok((self.function)(input))
    }
}
