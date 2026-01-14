use crate::model::Snapshot;

/// The result of evaluating a model at a given `x`.
#[derive(Debug, Clone)]
pub struct Evaluation<I, O, const N: usize> {
    pub x: [f64; N],
    pub residuals: [f64; N],
    pub snapshot: Snapshot<I, O>,
}
