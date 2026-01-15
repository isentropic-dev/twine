mod evaluate;
mod problem;

pub mod bisection;

pub use evaluate::{EvalError, Evaluation, evaluate};
pub use problem::EquationProblem;
