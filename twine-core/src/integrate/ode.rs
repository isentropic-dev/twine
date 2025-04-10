#![allow(dead_code)]

#[cfg(test)]
mod oscillator;

use std::convert::Infallible;

use ode_solvers::{SVector, System};

use crate::Component;

/// A trait for components that define an ODE model with `N` state variables.
///
/// This trait allows a component to be used with an ODE solver by describing
/// how to construct the component input from the solver state and how to
/// extract state derivatives from the component's output.
trait OdeModel<const N: usize>: Component {
    fn apply_state(initial_conditions: &Self::Input, x: f64, y: [f64; N]) -> Self::Input;

    fn extract_state(input: &Self::Input) -> [f64; N];

    fn extract_derivative(output: &Self::Output) -> [f64; N];
}

struct OdeSolver<C: OdeModel<N>, const N: usize> {
    component: C,
}

impl<C: OdeModel<N>, const N: usize> OdeSolver<C, N> {
    fn new(component: C) -> Self {
        Self { component }
    }

    fn call_component(&self, input: C::Input) -> Result<C::Output, C::Error> {
        self.component.call(input)
    }
}

#[derive(Debug)]
enum Method {
    /// Classic fixed-step 4th-order Runge–Kutta method.
    ///
    /// A widely used method that provides a good balance between accuracy
    /// and simplicity. Suitable for problems where high precision is not
    /// critical or where step size is controlled externally. Does not adapt
    /// step size based on local error, so it may be inefficient for stiff or
    /// rapidly-changing systems.
    Rk4,

    /// Adaptive Dormand–Prince 5(4) Runge–Kutta method.
    ///
    /// An explicit embedded method that computes both 5th and 4th order
    /// solutions to estimate local truncation error. The solver adjusts the
    /// step size to keep the error within specified `abs_tol` and `rel_tol`
    /// bounds. Efficient for many non-stiff problems and commonly used as a
    /// general-purpose adaptive integrator.
    Dopri5 { abs_tol: f64, rel_tol: f64 },

    /// Adaptive Dormand–Prince 8(5,3) Runge–Kutta method.
    ///
    /// A higher-order embedded Runge–Kutta method that provides 8th, 5th, and
    /// 3rd order solutions for precise error control. Offers high accuracy per
    /// step and is particularly useful for long integration intervals or when
    /// very low error tolerance is required. Typically more computationally
    /// expensive per step than `Dopri5`, but often more efficient overall due
    /// to fewer steps needed.
    Dop853 { abs_tol: f64, rel_tol: f64 },
}

#[derive(Debug)]
struct OdeSolverInput<C: OdeModel<N>, const N: usize> {
    initial_conditions: C::Input,
    x_start: f64,
    x_end: f64,
    x_step: f64,
    method: Method,
}

#[derive(Debug)]
struct OdeSolverOutput<C: OdeModel<N>, const N: usize> {
    call_count: u32,
    initial_conditions: C::Input,
    steps: Vec<Step<N>>,
}

impl<C: OdeModel<N>, const N: usize> OdeSolverOutput<C, N> {
    fn input_at_step(&self, step: &Step<N>) -> C::Input {
        let Step { x, y } = *step;
        C::apply_state(&self.initial_conditions, x, y)
    }
}

#[derive(Debug, Clone, Copy)]
struct Step<const N: usize> {
    /// The independent variable (usually time).
    x: f64,
    /// The dependent variable(s) at this `x`.
    y: [f64; N],
}

impl<C: OdeModel<N>, const N: usize> Component for OdeSolver<C, N> {
    type Input = OdeSolverInput<C, N>;
    type Output = OdeSolverOutput<C, N>;
    type Error = Infallible; // TODO: let's ignore errors for now

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let OdeSolverInput {
            initial_conditions,
            x_start,
            x_end,
            x_step,
            method,
        } = input;

        let system = OdeStepper {
            component: &self.component,
            initial_conditions: &initial_conditions,
        };

        let y_start = C::extract_state(&initial_conditions).into();

        let (stats, x_out, y_out) = match method {
            Method::Rk4 => {
                let mut stepper = ode_solvers::Rk4::new(system, x_start, y_start, x_end, x_step);

                let stats = stepper.integrate().expect("error handling to come later");
                let x_out = stepper.x_out().clone();
                let y_out = stepper.y_out().clone();

                (stats, x_out, y_out)
            }
            Method::Dopri5 { abs_tol, rel_tol } => {
                let mut stepper = ode_solvers::Dopri5::new(
                    system, x_start, x_end, x_step, y_start, rel_tol, abs_tol,
                );

                let stats = stepper.integrate().expect("error handling to come later");
                let x_out = stepper.x_out().clone();
                let y_out = stepper.y_out().clone();

                (stats, x_out, y_out)
            }

            Method::Dop853 { abs_tol, rel_tol } => {
                let mut stepper = ode_solvers::Dop853::new(
                    system, x_start, x_end, x_step, y_start, rel_tol, abs_tol,
                );

                let stats = stepper.integrate().expect("error handling to come later");
                let x_out = stepper.x_out().clone();
                let y_out = stepper.y_out().clone();

                (stats, x_out, y_out)
            }
        };

        let steps = x_out
            .into_iter()
            .zip(y_out)
            .map(|(x, y)| Step { x, y: y.into() })
            .collect();

        Ok(OdeSolverOutput {
            call_count: stats.num_eval,
            initial_conditions,
            steps,
        })
    }
}

struct OdeStepper<'a, C: OdeModel<N>, const N: usize> {
    component: &'a C,
    initial_conditions: &'a C::Input,
}

impl<C: OdeModel<N>, const N: usize> System<f64, SVector<f64, N>> for OdeStepper<'_, C, N> {
    fn system(&self, x: f64, y: &SVector<f64, N>, dy: &mut SVector<f64, N>) {
        let input = C::apply_state(self.initial_conditions, x, (*y).into());

        match self.component.call(input) {
            Ok(output) => {
                let derivative = C::extract_derivative(&output);
                *dy = SVector::from_row_slice(&derivative);
            }
            Err(_) => {
                *dy = SVector::from_element(f64::NAN);
            }
        }
    }

    fn solout(&mut self, _x: f64, _y: &SVector<f64, N>, _dy: &SVector<f64, N>) -> bool {
        // TODO: have a call_failed on &self that we set to true if NAN is in dy?
        false
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use crate::Component;

    use super::*;

    impl OdeModel<2> for oscillator::Oscillator {
        fn apply_state(input: &Self::Input, _x: f64, y: [f64; 2]) -> Self::Input {
            (*input).position_si(y[0]).velocity_si(y[1])
        }

        fn extract_state(input: &Self::Input) -> [f64; 2] {
            [input.state.position.value, input.state.velocity.value]
        }

        fn extract_derivative(output: &Self::Output) -> [f64; 2] {
            [output.position.value, output.velocity.value]
        }
    }

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

    #[test]
    fn integrate_oscillator() {
        let ode_solver = OdeSolver::new(oscillator::Oscillator);

        let initial_input = oscillator::Input::default() //
            .position_si(2.0)
            .velocity_si(1.0);

        let input = OdeSolverInput::<oscillator::Oscillator, 2> {
            initial_conditions: initial_input,
            x_start: 0.0,
            x_end: 2.0 * PI,
            x_step: PI * 0.1,
            // method: Method::Rk4,
            // method: Method::Dopri5 { abs_tol: 1e-9, rel_tol: 1e-9 },
            method: Method::Dop853 {
                abs_tol: 1e-9,
                rel_tol: 1e-9,
            },
        };

        let output = ode_solver.call(input).expect("sure");

        println!("ode output: {output:#?}");

        let final_step = output.steps.last().unwrap();
        let final_input = output.input_at_step(final_step);
        let final_output = ode_solver.call_component(final_input).unwrap();

        println!("\ninput at final step: {final_input:#?}",);
        println!("\noutput at final step: {final_output:#?}");
    }
}
