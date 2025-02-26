use crate::Component;

/// Defines a collection of connected components and their input/output types.
///
/// `ComponentGroup` is typically implemented on an empty marker struct to
/// specify how multiple subcomponents fit together. When paired with a generic
/// struct that holds actual instances, this approach enables reusing the same
/// component names in different contexts—whether referring to instances, their
/// input values, or their output values.
pub trait ComponentGroup {
    /// Specifies the concrete types of subcomponent instances.
    ///
    /// Typically a struct like `MyComponents<Comp1, Comp2, ...>`.
    type Components;

    /// Specifies the input types for each subcomponent.
    ///
    /// Typically structured as `<Comp as Component>::Input` for each field.
    type ComponentInputs;

    /// Specifies the output types for each subcomponent.
    ///
    /// Typically structured as `<Comp as Component>::Output` for each field.
    type ComponentOutputs;
}

/// Represents a composed component built from a `ComponentGroup`.
///
/// The `Composed` trait combines multiple subcomponents from a `ComponentGroup`
/// into a single [`Component`]. It allows implementers to define execution
/// order and map the composed component's input and the outputs of previously
/// executed subcomponents into the inputs of the next ones in the sequence.
pub trait Composed: Sized {
    /// The input type for this composed component.
    type Input;

    /// The `ComponentGroup` that defines the subcomponents.
    type Components: ComponentGroup;

    /// Constructs a new composed instance from subcomponents.
    ///
    /// This method defines the composition logic, which includes:
    /// - Accepting a `Self::Components` struct of instantiated components.
    /// - Specifying the execution order of subcomponents.
    /// - Mapping inputs and outputs between subcomponents, respecting execution
    ///   order to ensure values are available when needed.
    ///
    /// The execution order and input/output mapping are typically implemented
    /// using a `Twine` builder.
    fn new(components: <Self::Components as ComponentGroup>::Components) -> Self;

    /// Provides access to the composed processing chain as a [`Component`].
    fn component(
        &self,
    ) -> &dyn Component<
        Input = Self::Input,
        Output = <Self::Components as ComponentGroup>::ComponentOutputs,
    >;
}

/// Implements [`Component`] for all [`Composed`] types.
///
/// This blanket implementation ensures that any type implementing `Composed`
/// also implements [`Component`]. Once `new` and `component` are implemented,
/// the composed type can be used like any other `Component`.
impl<T> Component for T
where
    T: Composed,
{
    type Input = T::Input;
    type Output = <T::Components as ComponentGroup>::ComponentOutputs;

    fn call(&self, input: Self::Input) -> Self::Output {
        self.component().call(input)
    }
}
