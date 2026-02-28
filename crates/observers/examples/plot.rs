//! Interactive visualizations of Twine solvers.
//!
//! Each mode solves a simple mathematical problem and opens an interactive plot
//! window showing what the solver did.
//!
//! # Usage
//!
//! ```text
//! cargo run --example plot --features plot -- bisect
//! cargo run --example plot --features plot -- maximize
//! cargo run --example plot --features plot -- ode
//! cargo run --example plot --features plot -- ode 0.2
//! ```
//!
//! # Modes
//!
//! - **bisect** — Find the Dottie number (the unique fixed point of cos x).
//!   Shows x and residual converging as bisection homes in on ≈ 0.7391.
//!
//! - **maximize** — Find the maximum of sin(x) on \[0, π\].
//!   Shows evaluated points on the curve clustering around (π/2, 1).
//!
//! - **ode [dt]** — Integrate a damped oscillator with forward Euler over 30
//!   seconds. Overlays the analytical solution; drift accumulates with larger
//!   step sizes. Try `0.05` (default), `0.2`, `0.5` to see the difference.

use std::{convert::Infallible, error::Error};

use twine_core::{
    DerivativeOf, EquationProblem, Model, OdeProblem, OptimizationProblem, StepIntegrable,
};
use twine_observers::{PlotObserver, ShowConfig};
use twine_solvers::{equation::bisection, optimization::golden_section, transient::euler};

fn main() -> Result<(), Box<dyn Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "bisect".into());
    match mode.as_str() {
        "bisect" => bisect(),
        "maximize" => maximize(),
        "ode" => {
            let dt = std::env::args()
                .nth(2)
                .as_deref()
                .map(str::parse::<f64>)
                .transpose()
                .unwrap_or_else(|_| {
                    eprintln!("Invalid step size — expected a number, e.g. 0.1");
                    std::process::exit(1);
                })
                .unwrap_or(0.05);
            ode(dt)
        }
        other => {
            eprintln!("Unknown mode: {other}");
            eprintln!("Usage: plot [bisect|maximize|ode [dt]]");
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
/// Both x and the residual are plotted on a linear scale, showing
/// convergence toward the root.
fn bisect() -> Result<(), Box<dyn Error>> {
    let mut obs = PlotObserver::<2>::new(["x", "Residual"]);
    let mut iter = 0_u32;

    bisection::solve(
        &Passthrough,
        &CosMinusX,
        [0.0, 2.0],
        &bisection::Config::default(),
        |event: &bisection::Event<'_, Passthrough, CosMinusX>| {
            let n = f64::from(iter);
            iter += 1;
            let residual = event.result().as_ref().ok().map(|e| e.residuals[0].abs());
            obs.record(n, [Some(event.x()), residual]);
            None
        },
    )?;

    obs.show(
        ShowConfig::new()
            .title("Bisection: cos(x) = x  →  Dottie number ≈ 0.7391")
            .legend(),
    )?;

    Ok(())
}

// --- Maximize ----------------------------------------------------------------

/// Model that evaluates sin(x).
struct Sine;

impl Model for Sine {
    type Input = f64;
    type Output = f64;
    type Error = Infallible;

    fn call(&self, input: &f64) -> Result<f64, Infallible> {
        Ok(input.sin())
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

/// Maximize sin(x) on [0, π] and show where golden section samples the curve.
///
/// Golden section converges quickly, so the evaluations are sparse. A dense
/// background trace shows the full sine curve for context; the evaluated points
/// sit on top of it, converging toward the peak at (π/2, 1). Each point is
/// numbered in evaluation order so the solver's sampling strategy is visible.
fn maximize() -> Result<(), Box<dyn Error>> {
    let mut obs = PlotObserver::<2>::new(["sin(x)", "Evaluated points"]);

    // Pre-load the background sine curve as trace 0.
    for i in 0_u32..=300 {
        let x = std::f64::consts::PI * f64::from(i) / 300.0;
        obs.record(x, [Some(x.sin()), None]);
    }

    let mut iter = 1_u32;
    golden_section::maximize(
        &Sine,
        &DirectObjective,
        [0.0, std::f64::consts::PI],
        &golden_section::Config::default(),
        |event: &golden_section::Event<'_, Sine, DirectObjective>| {
            if let golden_section::Event::Evaluated { point, .. } = event {
                obs.record(point.x, [None, Some(point.objective)]);
                obs.label(point.x, point.objective, iter.to_string());
                iter += 1;
            }
            None
        },
    )?;

    obs.show(
        ShowConfig::new()
            .title("Maximize: sin(x) on [0, π]  →  maximum at (π/2, 1) ≈ (1.571, 1)")
            .legend(),
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

/// Model for the damped harmonic oscillator: ẋ = v, v̇ = −2ζω₀v − ω₀²x.
struct OscModel {
    zeta: f64,
    omega0: f64,
}

impl Model for OscModel {
    type Input = OscInput;
    type Output = OscOutput;
    type Error = Infallible;

    fn call(&self, input: &OscInput) -> Result<OscOutput, Infallible> {
        Ok(OscOutput {
            d_position: input.state.velocity,
            d_velocity: -2.0 * self.zeta * self.omega0 * input.state.velocity
                - self.omega0.powi(2) * input.state.position,
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

/// Integrate a damped oscillator with forward Euler and compare to the
/// analytical solution.
///
/// With ζ=0.1 the system oscillates through several cycles before settling.
/// Euler tracks the shape well but accumulates a small phase and amplitude
/// error over time — visible by the end of the run as the two traces drift
/// slightly apart.
fn ode(dt: f64) -> Result<(), Box<dyn Error>> {
    let zeta = 0.1_f64;
    let omega0 = 1.0_f64;
    let omega_d = (omega0.powi(2) - zeta.powi(2)).sqrt();

    let model = OscModel { zeta, omega0 };
    let initial = OscInput {
        state: OscState {
            position: 1.0,
            velocity: 0.0,
        },
        t: 0.0,
    };

    // Analytical solution for x(0)=1, v(0)=0:
    // x(t) = e^(-ζt) · [cos(ω_d·t) + (ζ/ω_d)·sin(ω_d·t)]
    let analytical = move |t: f64| {
        (-zeta * t).exp() * ((omega_d * t).cos() + (zeta / omega_d) * (omega_d * t).sin())
    };

    let mut obs = PlotObserver::<2>::new(["Euler (numerical)", "Analytical"]);

    // Simulate 30 seconds regardless of step size.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    // TODO: remove once we have a cleaner way to derive step count from duration
    let steps = (30.0 / dt).round() as usize;
    euler::solve(
        &model,
        &OscProblem,
        initial,
        dt,
        steps,
        |event: &euler::Event<OscInput, OscOutput>| {
            let t = event.snapshot.input.t;
            obs.record(
                t,
                [
                    Some(event.snapshot.input.state.position),
                    Some(analytical(t)),
                ],
            );
            None
        },
    )?;

    obs.show(
        ShowConfig::new()
            .title(&format!(
                "ODE: Damped oscillator (ζ=0.1, dt={dt}) — Euler vs. analytical"
            ))
            .legend(),
    )?;

    Ok(())
}
