use crate::Component;

/// Defines a set of related components that can be composed.
///
/// The `Composable` trait is typically implemented for a generic struct that
/// groups multiple named components. It provides associated types for:
/// - `Inputs`: A variant where each field represents the component's input type.
/// - `Outputs`: A variant where each field represents the component's output type.
pub trait Composable {
    /// Defines the expected input types for each subcomponent.
    ///
    /// Typically structured as `<CompType as Component>::Input` for each field.
    type Inputs;

    /// Defines the computed output types for each subcomponent.
    ///
    /// Typically structured as `<CompType as Component>::Output` for each field.
    type Outputs;
}

/// Represents a component built from a `Composable` set of subcomponents.
///
/// The `Composed` trait enables combining multiple subcomponents into a single
/// [`Component`]. It provides methods for constructing the execution chain and
/// accessing the final composed component.
pub trait Composed: Sized {
    /// The input type for the composed component.
    type Input;

    /// The grouped subcomponents.
    type Components: Composable;

    /// Constructs a new composed instance from subcomponents.
    ///
    /// This method defines the composition logic, which includes:
    /// - Accepting a `Self::Components` struct of instantiated subcomponents.
    /// - Defining the execution order of subcomponents.
    /// - Mapping inputs and outputs between subcomponents to ensure values
    ///   are available at the correct execution stage.
    ///
    /// The execution order and input/output mapping are typically implemented
    /// using a `Twine` builder.
    fn new(components: Self::Components) -> Self;

    /// Returns a reference to the composed processing chain as a [`Component`].
    fn component(
        &self,
    ) -> &dyn Component<Input = Self::Input, Output = <Self::Components as Composable>::Outputs>;
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
    type Output = <T::Components as Composable>::Outputs;

    fn call(&self, input: Self::Input) -> Self::Output {
        self.component().call(input)
    }
}
