use crate::Component;

/// Defines a collection of connected components and their input/output types.
///
/// `Composition` is usually implemented on an empty marker struct to define
/// how subcomponents fit together. When combined with a generic struct that
/// holds instances, it allows reusing the same component names across different
/// contexts—whether referring to instances, inputs, or outputs.
pub trait Composition {
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

/// Represents a composed component built from a `Composition`.
///
/// The `Composed` trait combines multiple subcomponents from a `Composition`
/// into a single [`Component`]. It allows implementers to define execution
/// order and map the composed component's input and the outputs of previously
/// executed subcomponents into the inputs of the next ones in the sequence.
pub trait Composed: Sized {
    /// The input type for this composed component.
    type Input;

    /// The `Composition` that defines the subcomponents.
    type Components: Composition;

    /// Constructs a new composed instance from subcomponents.
    ///
    /// This method defines the composition logic, which includes:
    /// - Accepting a `Self::Components` struct of instantiated components.
    /// - Specifying the execution order of subcomponents.
    /// - Mapping inputs and outputs between subcomponents, ensuring values
    ///   are available at the correct execution stage.
    ///
    /// The execution order and input/output mapping are typically implemented
    /// using a `Twine` builder.
    fn new(components: <Self::Components as Composition>::Components) -> Self;

    /// Provides access to the composed processing chain as a [`Component`].
    fn component(
        &self,
    ) -> &dyn Component<
        Input = Self::Input,
        Output = <Self::Components as Composition>::ComponentOutputs,
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
    type Output = <T::Components as Composition>::ComponentOutputs;

    fn call(&self, input: Self::Input) -> Self::Output {
        self.component().call(input)
    }
}
