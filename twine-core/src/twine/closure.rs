use std::marker::PhantomData;

use crate::Component;

/// A wrapper that allows using functions as components.
pub(crate) struct Closure<F, I, O> {
    function: F,
    _marker: PhantomData<(I, O)>,
}

impl<F, I, O> Closure<F, I, O> {
    /// Creates a new closure-based component.
    pub(crate) const fn new(function: F) -> Self {
        Self {
            function,
            _marker: PhantomData,
        }
    }
}

impl<F, I, O> Component for Closure<F, I, O>
where
    F: Fn(I) -> O,
{
    type Input = I;
    type Output = O;

    fn call(&self, input: Self::Input) -> Self::Output {
        (self.function)(input)
    }
}

impl<F, I, O> From<F> for Closure<F, I, O>
where
    F: Fn(I) -> O,
{
    /// Converts a function into a `Closure` component.
    fn from(func: F) -> Self {
        Closure::new(func)
    }
}
