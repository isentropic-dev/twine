use twine_core::Snapshot;

/// Event emitted by the Euler solver for each snapshot.
///
/// Step 0 is the initial state before any integration.
/// Steps 1..N are emitted after each integration step.
#[derive(Debug, Clone)]
pub struct Event<I, O> {
    /// The step number (0 for initial, 1..N for integration steps).
    pub step: usize,

    /// Snapshot of the model input and output at this step.
    pub snapshot: Snapshot<I, O>,
}
