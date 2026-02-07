mod equation;
mod ode;
mod optimization;

pub use equation::EquationProblem;
pub use ode::OdeProblem;
pub use optimization::{MaximizationProblem, MinimizationProblem};
