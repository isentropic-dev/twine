mod evaluate;
mod goal;
mod problem;

pub use evaluate::{EvalError, EvaluateResult, Evaluation, evaluate};
pub use goal::{Goal, Maximize, Minimize};
pub use problem::OptimizationProblem;
