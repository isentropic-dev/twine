use std::marker::PhantomData;

use super::Component;

/// A wrapper that transforms a component’s input type.
///
/// Internally used by `.map_input()` to adapt a component so it can accept a
/// different input type.
pub(crate) struct MappedInput<C, InputMap, NewInput>
where
    C: Component,
    InputMap: Fn(NewInput) -> C::Input,
{
    component: C,
    input_map: InputMap,
    _marker: PhantomData<NewInput>,
}

impl<C, InputMap, NewInput> MappedInput<C, InputMap, NewInput>
where
    C: Component,
    InputMap: Fn(NewInput) -> C::Input,
{
    /// Creates a new component with an adapted input type.
    pub(crate) fn new(component: C, input_map: InputMap) -> Self {
        Self {
            component,
            input_map,
            _marker: PhantomData,
        }
    }
}

impl<C, InputMap, NewInput> Component for MappedInput<C, InputMap, NewInput>
where
    C: Component,
    InputMap: Fn(NewInput) -> C::Input,
{
    type Input = NewInput;
    type Output = C::Output;
    type Error = C::Error;

    /// Calls the wrapped component with a transformed input value.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let mapped_input = (self.input_map)(input);
        self.component.call(mapped_input)
    }
}
