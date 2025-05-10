use crate::{thermo::units::HasTimeDerivative, Component};

pub trait StatefulComponent: Component {
    type State: HasTimeDerivative;

    fn extract_state(input: &Self::Input) -> Self::State;

    fn extract_derivative(
        output: &Self::Output,
    ) -> <Self::State as HasTimeDerivative>::TimeDerivative;

    fn apply_state(input: &Self::Input, state: Self::State) -> Self::Input;
}
