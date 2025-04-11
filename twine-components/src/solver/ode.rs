use std::{cell::RefCell, rc::Rc};

use ode_solvers::{dop_shared::IntegrationError, SVector, System};
use thiserror::Error;
use twine_core::{
    solve::ode::{Integratable, State},
    Component,
};

/// Solves an [`Integratable`] component that defines a system of ODEs.
pub struct Solver<C: Integratable<N>, const N: usize> {
    component: C,
}

/// Input for [`Solver::call`].
#[derive(Debug)]
pub struct SolverInput<C: Integratable<N>, const N: usize> {
    /// The initial conditions for integration.
    ///
    /// This input defines the initial system [`State`] using
    /// [`Integratable::extract_state`] and is used to reconstruct component
    /// inputs during integration with [`Integratable::apply_state`].
    pub initial_conditions: C::Input,

    /// The endpoint of the integration interval.
    pub x_end: f64,

    /// The step size used by the solver.
    pub x_step: f64,

    /// The numerical integration method to use.
    pub method: Method,
}

/// Output for [`Solver::call`].
#[derive(Debug)]
pub struct SolverOutput<C: Integratable<N>, const N: usize> {
    /// Number of times the component was called during integration.
    pub component_calls: u32,

    /// The original input, used to reconstruct component inputs from solver states.
    pub initial_conditions: C::Input,

    /// The full sequence of integration states produced by the solver.
    pub steps: Vec<State<N>>,
}

/// Error returned by [`Solver::call`].
///
/// Wraps integration errors and component call failures.
#[derive(Debug, Error)]
pub enum SolverError {
    #[error(transparent)]
    IntegrationError(#[from] IntegrationError),

    #[error("Component call failed")]
    ComponentError {
        #[source]
        error: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}

/// Supported numerical integration methods for the solver.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Method {
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

impl<C: Integratable<N>, const N: usize> Solver<C, N> {
    /// Creates a new ODE solver for the given component.
    pub fn new(component: C) -> Self {
        Self { component }
    }

    /// Returns the component input at the final integration step.
    ///
    /// This method reconstructs the input to the component from the final state
    /// in the solver output. This is useful when the downstream logic depends
    /// on the component's view of the state, not just the raw state vector.
    ///
    /// # Parameters
    ///
    /// - `output`: A `SolverOutput` produced by a successful solver run.
    ///
    /// # Returns
    ///
    /// The reconstructed component input corresponding to the final state.
    ///
    /// # Panics
    ///
    /// Panics if `output.steps` is empty.
    pub fn final_component_input(&self, output: &SolverOutput<C, N>) -> C::Input {
        let state = *output.steps.last().expect("SolverOutput has no steps");
        C::apply_state(&output.initial_conditions, state)
    }

    /// Evaluates the component at the final integration step.
    ///
    /// This method reconstructs the component input from the final state
    /// and calls the component to obtain its output. It is typically used to
    /// extract the final computed value after integration.
    ///
    /// # Parameters
    ///
    /// - `output`: A `SolverOutput` produced by a successful solver run.
    ///
    /// # Returns
    ///
    /// The component's output for the final integration step.
    ///
    /// # Errors
    ///
    /// Returns an error if the component call at the final step fails.
    ///
    /// # Panics
    ///
    /// Panics if `output.steps` is empty.
    pub fn final_component_output(
        &self,
        output: &SolverOutput<C, N>,
    ) -> Result<C::Output, C::Error> {
        let input = self.final_component_input(output);
        self.component.call(input)
    }
}

impl<C: Integratable<N>, const N: usize> Component for Solver<C, N> {
    type Input = SolverInput<C, N>;
    type Output = SolverOutput<C, N>;
    type Error = SolverError;

    /// Integrates the component over the specified domain using the selected method.
    ///
    /// This method evaluates the wrapped component as an ODE system and returns
    /// a sequence of integration steps. If an error occurs during evaluation or
    /// numerical integration, the method returns an appropriate `SolverError`.
    ///
    /// # Parameters
    ///
    /// - `input`: Solver configuration and initial conditions.
    ///
    /// # Returns
    ///
    /// A `SolverOutput` containing the full integration trace, or a `SolverError`
    /// if the component fails or the integration does not complete successfully.
    ///
    /// # Errors
    ///
    /// Returns `SolverError::IntegrationError` if the numerical solver fails, or
    /// `SolverError::ComponentError` if any component call fails during integration.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let SolverInput {
            initial_conditions,
            x_end,
            x_step,
            method,
        } = input;

        let component_error = Rc::new(RefCell::new(None));
        let system = OdeSystem {
            component: &self.component,
            initial_conditions: &initial_conditions,
            call_error: Rc::clone(&component_error),
        };

        let State { x: x_start, y } = C::extract_state(&initial_conditions);
        let y_start = y.into();

        let (stats, x_out, y_out) = match method {
            Method::Rk4 => {
                let mut stepper = ode_solvers::Rk4::new(system, x_start, y_start, x_end, x_step);
                stepper.integrate().map(|stats| {
                    let x_out = stepper.x_out().clone();
                    let y_out = stepper.y_out().clone();
                    (stats, x_out, y_out)
                })?
            }
            Method::Dopri5 { abs_tol, rel_tol } => {
                let mut stepper = ode_solvers::Dopri5::new(
                    system, x_start, x_end, x_step, y_start, rel_tol, abs_tol,
                );
                stepper.integrate().map(|stats| {
                    let x_out = stepper.x_out().clone();
                    let y_out = stepper.y_out().clone();
                    (stats, x_out, y_out)
                })?
            }
            Method::Dop853 { abs_tol, rel_tol } => {
                let mut stepper = ode_solvers::Dop853::new(
                    system, x_start, x_end, x_step, y_start, rel_tol, abs_tol,
                );
                stepper.integrate().map(|stats| {
                    let x_out = stepper.x_out().clone();
                    let y_out = stepper.y_out().clone();
                    (stats, x_out, y_out)
                })?
            }
        };

        if let Some(err) = component_error.borrow_mut().take() {
            return Err(SolverError::ComponentError {
                error: Box::new(err),
            });
        }

        let steps = x_out
            .into_iter()
            .zip(y_out)
            .map(|(x, y)| State { x, y: y.into() })
            .collect();

        Ok(SolverOutput {
            component_calls: stats.num_eval,
            initial_conditions,
            steps,
        })
    }
}

/// Internal wrapper that adapts a component into an ODE solver system.
struct OdeSystem<'a, C: Integratable<N>, const N: usize> {
    component: &'a C,
    initial_conditions: &'a C::Input,
    call_error: Rc<RefCell<Option<C::Error>>>,
}

impl<C: Integratable<N>, const N: usize> System<f64, SVector<f64, N>> for OdeSystem<'_, C, N> {
    fn system(&self, x: f64, y: &SVector<f64, N>, dy: &mut SVector<f64, N>) {
        let state = State { x, y: (*y).into() };
        let input = C::apply_state(self.initial_conditions, state);

        match self.component.call(input) {
            Ok(output) => {
                let derivative = C::extract_derivative(&output);
                *dy = SVector::from_row_slice(&derivative);
            }
            Err(e) => {
                *self.call_error.borrow_mut() = Some(e);
                *dy = SVector::from_element(f64::NAN);
            }
        }
    }

    fn solout(&mut self, _x: f64, _y: &SVector<f64, N>, _dy: &SVector<f64, N>) -> bool {
        // Stop integration early if a component call failed.
        self.call_error.borrow().is_some()
    }
}

#[cfg(test)]
mod tests {
    use std::{convert::Infallible, f64::consts::PI};

    use approx::{assert_abs_diff_eq, assert_relative_eq};
    use uom::si::{
        acceleration::meter_per_second_squared, length::meter, velocity::meter_per_second,
    };

    use crate::example::oscillator;

    use super::*;

    /// A test component representing a one-dimensional linear ODE: dy/dx = slope.
    struct Linear {
        slope: f64,
    }

    impl Component for Linear {
        type Input = State<1>;
        type Output = f64;
        type Error = Infallible;

        fn call(&self, _input: Self::Input) -> Result<Self::Output, Self::Error> {
            Ok(self.slope)
        }
    }

    impl Integratable<1> for Linear {
        fn apply_state(_initial_conditions: &Self::Input, state: State<1>) -> Self::Input {
            state
        }

        fn extract_state(input: &Self::Input) -> State<1> {
            *input
        }

        fn extract_derivative(output: &Self::Output) -> [f64; 1] {
            [*output]
        }
    }

    /// Implements `Integratable` for the `Oscillator` component.
    ///
    /// The oscillator component does not carry its own time information, so
    /// this implementation assumes that initial conditions are always specified
    /// at `x = 0` (i.e., time = 0). This is sufficient for test cases where the
    /// solver is responsible for advancing time from zero.
    impl Integratable<2> for oscillator::Oscillator {
        fn apply_state(initial_conditions: &Self::Input, state: State<2>) -> Self::Input {
            let position = state.y[0];
            let velocity = state.y[1];
            (*initial_conditions)
                .position_si(position)
                .velocity_si(velocity)
        }

        fn extract_state(input: &Self::Input) -> State<2> {
            State {
                x: 0.0,
                y: [input.state.position.value, input.state.velocity.value],
            }
        }

        fn extract_derivative(output: &Self::Output) -> [f64; 2] {
            [output.velocity.value, output.acceleration.value]
        }
    }

    /// A simple constant-rate ODE model: dy/dx = 2.
    ///
    /// Given initial y(0) = 4, we expect y(x) = 2x + 4.
    /// This test checks exact values at x = 0.0, 0.5, and 1.0.
    #[test]
    fn solve_a_linear_ode() {
        let solver = Solver::new(Linear { slope: 2.0 });

        let input = SolverInput {
            initial_conditions: State { x: 0.0, y: [4.0] },
            x_end: 1.0,
            x_step: 0.5,
            method: Method::Rk4,
        };

        let output = solver.call(input).unwrap();

        let [first, middle, last] = output.steps.as_slice() else {
            panic!("Expected exactly 3 steps");
        };

        assert_relative_eq!(first.y[0], 4.0);
        assert_relative_eq!(middle.y[0], 5.0);
        assert_relative_eq!(last.y[0], 6.0);
    }

    /// Simulates a simple harmonic oscillator with unit mass and stiffness.
    ///
    /// The system starts at position = 1.0, velocity = 0.0. This is equivalent to:
    ///     x(t) = cos(t)
    ///     v(t) = -sin(t)
    ///     a(t) = -x(t)
    ///
    /// We integrate from t = 0 to t = π, so we expect:
    ///     position ≈ -1.0 (cos(π))
    ///     velocity ≈  0.0 (sin(π))
    ///     acceleration ≈ 1.0 (–cos(π))
    #[test]
    fn solve_a_harmonic_oscillator() {
        let solver = Solver::new(oscillator::Oscillator);

        let input = SolverInput {
            initial_conditions: oscillator::Input::default()
                .position_si(1.0)
                .velocity_si(0.0),
            x_end: PI,
            x_step: PI * 0.01,
            method: Method::Dopri5 {
                abs_tol: 1e-9,
                rel_tol: 1e-6,
            },
        };

        let output = solver.call(input).unwrap();

        let final_component_input = solver.final_component_input(&output);
        let final_component_output = solver.final_component_output(&output).unwrap();

        assert_relative_eq!(
            final_component_input.state.position.get::<meter>(),
            -1.0,
            max_relative = 1e-6
        );
        assert_abs_diff_eq!(
            final_component_output.velocity.get::<meter_per_second>(),
            0.0,
            epsilon = 1e-6
        );
        assert_relative_eq!(
            final_component_output
                .acceleration
                .get::<meter_per_second_squared>(),
            1.0,
            max_relative = 1e-6
        );
    }
}
