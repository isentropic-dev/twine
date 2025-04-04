#[cfg(test)]
mod oscillator;

// TODO: Create a component that wraps `ode_solvers`, using `Oscillator` as a
//       test case to drive development.

#[cfg(test)]
mod tests {
    use super::*;

    use crate::Component;

    #[test]
    fn call_oscillator() {
        let component = oscillator::Oscillator;

        // At rest with zero position and velocity.
        let input = oscillator::Input::default();
        let expected_derivative = oscillator::Derivative::new_si(0.0, 0.0);
        assert_eq!(component.call(input).unwrap(), expected_derivative);

        // Displaced with nonzero position and velocity.
        let input = oscillator::Input::default() //
            .position_si(2.0)
            .velocity_si(1.0);
        let expected_derivative = oscillator::Derivative::new_si(1.0, -2.0);
        assert_eq!(component.call(input).unwrap(), expected_derivative);

        // Changing stiffness and mass affects the velocity derivative.
        let input = oscillator::Input::default()
            .position_si(2.0)
            .velocity_si(1.0)
            .stiffness_si(0.5)
            .mass_si(4.0);
        let expected_derivative = oscillator::Derivative::new_si(1.0, -0.25);
        assert_eq!(component.call(input).unwrap(), expected_derivative);
    }
}
