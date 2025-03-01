use std::marker::PhantomData;

use super::Component;

/// A wrapper that transforms a component’s input and output.
///
/// Internally used by `.map()` to modify how a component interacts with its
/// surrounding context.
pub(crate) struct Mapped<C, InputMap, OutputMap, In, Out>
where
    C: Component,
    InputMap: Fn(&In) -> C::Input,
    OutputMap: Fn(In, C::Output) -> Out,
{
    component: C,
    input_map: InputMap,
    output_map: OutputMap,
    _marker: PhantomData<(In, Out)>,
}

impl<C, InputMap, OutputMap, In, Out> Mapped<C, InputMap, OutputMap, In, Out>
where
    C: Component,
    InputMap: Fn(&In) -> C::Input,
    OutputMap: Fn(In, C::Output) -> Out,
{
    /// Creates a new mapped component with input and output transformations.
    pub(crate) fn new(component: C, input_map: InputMap, output_map: OutputMap) -> Self {
        Self {
            component,
            input_map,
            output_map,
            _marker: PhantomData,
        }
    }
}

impl<C, InputMap, OutputMap, In, Out> Component for Mapped<C, InputMap, OutputMap, In, Out>
where
    C: Component,
    InputMap: Fn(&In) -> C::Input,
    OutputMap: Fn(In, C::Output) -> Out,
{
    type Input = In;
    type Output = Out;
    type Error = C::Error;

    /// Calls the wrapped component with a transformed input and applies
    /// the output mapping function.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let mapped_input = (self.input_map)(&input);
        let output = self.component.call(mapped_input)?;
        Ok((self.output_map)(input, output))
    }
}
