pub mod equation;
pub mod ode;
pub mod optimization;

pub use equation::EquationProblem;
pub use ode::OdeProblem;
pub use optimization::{MaximizationProblem, MinimizationProblem};
