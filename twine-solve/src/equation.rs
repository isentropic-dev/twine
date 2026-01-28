mod evaluate;
mod problem;

pub mod bisection;

pub use evaluate::{EvalError, EvaluateResult, Evaluation, evaluate};
pub use problem::EquationProblem;
