mod evaluate;
mod observe;
mod problem;

pub mod bisection;

pub use evaluate::{EvalError, Evaluation, evaluate};
pub use observe::Observer;
pub use problem::EquationProblem;
