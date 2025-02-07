use std::marker::PhantomData;

use super::Component;

/// A wrapper that adapts a component by transforming its input and output.
///
/// This struct is used internally by `.map()` to modify how a component
/// interacts with its surrounding context.
pub(crate) struct Mapped<C, FI, FO, I, O> {
    component: C,
    input_map: FI,
    output_map: FO,
    _marker: PhantomData<(I, O)>,
}

impl<C, FI, FO, I, O> Mapped<C, FI, FO, I, O> {
    /// Creates a new mapped component with input and output transformations.
    pub(crate) fn new(component: C, input_map: FI, output_map: FO) -> Self {
        Self {
            component,
            input_map,
            output_map,
            _marker: PhantomData,
        }
    }
}

impl<C, FI, FO, I, O> Component for Mapped<C, FI, FO, I, O>
where
    C: Component,
    FI: Fn(&I) -> C::Input,
    FO: Fn((I, C::Output)) -> O,
{
    type Input = I;
    type Output = O;

    /// Calls the wrapped component with a transformed input and applies the
    /// output mapping function.
    fn call(&self, input: Self::Input) -> Self::Output {
        let mapped_input = (self.input_map)(&input);
        let output = self.component.call(mapped_input);
        (self.output_map)((input, output))
    }
}
