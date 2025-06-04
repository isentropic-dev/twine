use std::marker::PhantomData;

use super::Component;

/// A wrapper that transforms a component's output type.
///
/// Internally used by `.map_output()` to adapt a component so it produces a
/// different output type.
pub(crate) struct MappedOutput<C, OutputMap, NewOutput>
where
    C: Component,
    OutputMap: Fn(C::Output) -> NewOutput,
{
    component: C,
    output_map: OutputMap,
    _marker: PhantomData<NewOutput>,
}

impl<C, OutputMap, NewOutput> MappedOutput<C, OutputMap, NewOutput>
where
    C: Component,
    OutputMap: Fn(C::Output) -> NewOutput,
{
    /// Creates a new component with an adapted output type.
    pub(crate) fn new(component: C, output_map: OutputMap) -> Self {
        Self {
            component,
            output_map,
            _marker: PhantomData,
        }
    }
}

impl<C, OutputMap, NewOutput> Component for MappedOutput<C, OutputMap, NewOutput>
where
    C: Component,
    OutputMap: Fn(C::Output) -> NewOutput,
{
    type Input = C::Input;
    type Output = NewOutput;
    type Error = C::Error;

    /// Calls the wrapped component and transforms the output.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        self.component.call(input).map(&self.output_map)
    }
}
