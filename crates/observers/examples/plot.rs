//! Interactive visualizations of Twine solvers.
//!
//! Each mode solves a simple mathematical problem and opens an interactive plot
//! window showing what the solver did.
//!
//! # Usage
//!
//! ```text
//! cargo run --example plot --features plot -- bisect
//! cargo run --example plot --features plot -- minimize
//! cargo run --example plot --features plot -- ode
//! ```
//!
//! # Modes
//!
//! - **bisect** — Find the Dottie number (the unique fixed point of cos x).
//!   Shows signed residual and bracket width converging to zero.
//!
//! - **minimize** — Find the minimum of x⁴ − 4x² on \[0, 3\].
//!   Shows both interior points chasing the minimum at x = √2.
//!
//! - **ode** — Integrate an undamped simple harmonic oscillator with forward
//!   Euler. Overlays the analytical solution to show energy drift over time —
//!   a vivid illustration of why step size matters.

use std::{convert::Infallible, error::Error};

use twine_core::{
    DerivativeOf, EquationProblem, Model, OdeProblem, OptimizationProblem, StepIntegrable,
};
use twine_observers::{PlotObserver, show_traces};
use twine_solvers::{equation::bisection, optimization::golden_section, transient::euler};

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "bisect".into());
    match mode.as_str() {
        "bisect" => bisect(),
        "minimize" => minimize(),
        "ode" => ode(),
        other => {
            eprintln!("Unknown mode: {other}");
            eprintln!("Usage: plot [bisect|minimize|ode]");
            std::process::exit(1);
        }
    }
}

// --- Bisect ------------------------------------------------------------------

/// A model that passes its input through unchanged.
struct Passthrough;

impl Model for Passthrough {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn call(&self, input: &f64) -> Result<f64, Infallible> {
        Ok(*input)
    }
}

/// Residual: cos(x) − x.
///
/// Finding the root gives the Dottie number ≈ 0.7391.
struct CosMinusX;

impl EquationProblem<1> for CosMinusX {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn input(&self, x: &[f64; 1]) -> Result<f64, Infallible> {
        Ok(x[0])
    }

    fn residuals(&self, input: &f64, _output: &f64) -> Result<[f64; 1], Infallible> {
        Ok([input.cos() - input])
    }
}

/// Find the Dottie number via bisection and plot convergence.
///
/// Bisection events carry lifetime parameters, so we collect data manually in a
/// closure and pass it to [`show_traces`] rather than using [`PlotObserver`]
/// directly.
fn bisect() -> Result<(), Box<dyn Error>> {
    let mut iter = 0_u32;
    let mut residuals: Vec<[f64; 2]> = Vec::new();
    let mut brackets: Vec<[f64; 2]> = Vec::new();

    bisection::solve(
        &Passthrough,
        &CosMinusX,
        [0.0, 2.0],
        &bisection::Config::default(),
        |event: &bisection::Event<'_, Passthrough, CosMinusX>| {
            let n = f64::from(iter);
            iter += 1;

            if let Ok(eval) = event.result() {
                residuals.push([n, eval.residuals[0]]);
            }
            if let bisection::Event::Midpoint { bracket, .. } = event {
                brackets.push([n, bracket.width()]);
            }

            None
        },
    )?;

    show_traces(
        "Bisection: cos(x) = x  →  Dottie number ≈ 0.7391",
        vec![
            ("Residual".into(), residuals),
            ("Bracket width".into(), brackets),
        ],
        true,
    )?;

    Ok(())
}

// --- Minimize ----------------------------------------------------------------

/// Model that evaluates f(x) = x⁴ − 4x².
struct Quartic;

impl Model for Quartic {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn call(&self, input: &f64) -> Result<f64, Infallible> {
        let x = *input;
        Ok(x.powi(4) - 4.0 * x.powi(2))
    }
}

/// Optimization problem that uses the model output directly as the objective.
struct DirectObjective;

impl OptimizationProblem<1> for DirectObjective {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn input(&self, x: &[f64; 1]) -> Result<f64, Infallible> {
        Ok(x[0])
    }

    fn objective(&self, _input: &f64, output: &f64) -> Result<f64, Infallible> {
        Ok(*output)
    }
}

/// Minimize x⁴ − 4x² on [0, 3] and plot both interior points converging.
///
/// The `other` point in each golden section event is always the current best,
/// so plotting both `point.objective` and `other.objective` shows the two
/// interior points chasing the minimum together.
fn minimize() -> Result<(), Box<dyn Error>> {
    let mut iter = 0_u32;
    let mut current: Vec<[f64; 2]> = Vec::new();
    let mut best: Vec<[f64; 2]> = Vec::new();

    golden_section::minimize(
        &Quartic,
        &DirectObjective,
        [0.0, 3.0],
        &golden_section::Config::default(),
        |event: &golden_section::Event<'_, Quartic, DirectObjective>| {
            let n = f64::from(iter);
            iter += 1;

            if let golden_section::Event::Evaluated { point, other, .. } = event {
                current.push([n, point.objective]);
                best.push([n, other.objective]);
            }

            None
        },
    )?;

    show_traces(
        "Minimize: x⁴ − 4x²  →  minimum at x = √2 ≈ 1.414",
        vec![
            ("Current point".into(), current),
            ("Best so far".into(), best),
        ],
        true,
    )?;

    Ok(())
}

// --- ODE ---------------------------------------------------------------------

/// State of the oscillator: position and velocity.
#[derive(Clone, Debug)]
struct OscState {
    position: f64,
    velocity: f64,
}

/// Time derivative of the oscillator state.
#[derive(Clone, Debug)]
struct OscDerivative {
    d_position: f64,
    d_velocity: f64,
}

impl StepIntegrable<f64> for OscState {
    type Derivative = OscDerivative;

    fn step(&self, deriv: OscDerivative, dt: f64) -> Self {
        OscState {
            position: self.position + deriv.d_position * dt,
            velocity: self.velocity + deriv.d_velocity * dt,
        }
    }
}

/// Full model input: oscillator state plus current time.
#[derive(Clone, Debug)]
struct OscInput {
    state: OscState,
    t: f64,
}

/// Model output: time derivatives of position and velocity.
#[derive(Clone, Debug)]
struct OscOutput {
    d_position: f64,
    d_velocity: f64,
}

/// Model for the simple harmonic oscillator: ẋ = v, v̇ = −ω₀²x.
struct OscModel {
    omega0: f64,
}

impl Model for OscModel {
    type Input = OscInput;
    type Output = OscOutput;
    type Error = Infallible;

    fn call(&self, input: &OscInput) -> Result<OscOutput, Infallible> {
        Ok(OscOutput {
            d_position: input.state.velocity,
            d_velocity: -self.omega0.powi(2) * input.state.position,
        })
    }
}

/// ODE problem that wires up the oscillator state, derivative, and input.
struct OscProblem;

impl OdeProblem for OscProblem {
    type Input = OscInput;
    type Output = OscOutput;
    type Delta = f64;
    type State = OscState;
    type Error = Infallible;

    fn state(&self, input: &OscInput) -> Result<OscState, Infallible> {
        Ok(input.state.clone())
    }

    fn derivative(
        &self,
        _input: &OscInput,
        output: &OscOutput,
    ) -> Result<DerivativeOf<OscState, f64>, Infallible> {
        Ok(OscDerivative {
            d_position: output.d_position,
            d_velocity: output.d_velocity,
        })
    }

    fn build_input(
        &self,
        base: &OscInput,
        state: &OscState,
        dt: &f64,
    ) -> Result<OscInput, Infallible> {
        Ok(OscInput {
            state: state.clone(),
            t: base.t + dt,
        })
    }
}

/// Integrate an undamped SHO with forward Euler and compare to the analytical
/// solution cos(t).
///
/// Forward Euler introduces a small energy gain at every step. Over many steps
/// this accumulates visibly: the numerical amplitude grows while the analytical
/// solution stays bounded. The longer the simulation runs, the more dramatic
/// the divergence.
///
/// [`PlotObserver`] is used here directly since euler events carry no lifetime
/// parameters.
fn ode() -> Result<(), Box<dyn Error>> {
    let model = OscModel { omega0: 1.0 };
    let initial = OscInput {
        state: OscState {
            position: 1.0,
            velocity: 0.0,
        },
        t: 0.0,
    };

    let mut observer =
        PlotObserver::new(|event: &euler::Event<OscInput, OscOutput>| Some(event.snapshot.input.t))
            .trace("Euler (numerical)", |event| {
                Some(event.snapshot.input.state.position)
            })
            .trace("Analytical cos(t)", |event| {
                Some(event.snapshot.input.t.cos())
            })
            .with_legend();

    // dt=0.1, 500 steps → 50 seconds. Energy grows by (1 + dt²)^steps ≈ 142x,
    // so amplitude grows by √142 ≈ 12x. Unmistakable.
    euler::solve(&model, &OscProblem, initial, 0.1_f64, 500, &mut observer)?;

    observer.show("ODE: Undamped SHO — Euler energy drift vs. analytical cos(t)")?;

    Ok(())
}
