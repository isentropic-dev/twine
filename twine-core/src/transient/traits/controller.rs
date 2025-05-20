use crate::{
    transient::{Simulation, Temporal},
    Component,
};

/// A trait for modifying simulation inputs before component evaluation.
///
/// A `Controller` adjusts the input proposed by an [`Integrator`] during each
/// simulation step, enabling feedback control, constraint enforcement, or
/// other domain-specific transformations.
///
/// # Common Use Cases
///
/// - Closed-loop feedback control  
/// - Enforcing physical or logical constraints  
/// - Open-loop simulation using [`NoController`]
///
/// [`NoController`]: crate::transient::controllers::NoController
pub trait Controller<C>
where
    C: Component,
    C::Input: Clone + Temporal,
{
    /// The error type returned if control logic fails.
    type Error: std::error::Error + Send + Sync + 'static;

    /// Adjusts the proposed input before component evaluation.
    ///
    /// # Errors
    ///
    /// Returns `Err(Self::Error)` if the input is invalid or control logic fails.
    fn adjust_input(
        &self,
        simulation: &Simulation<C>,
        input: C::Input,
    ) -> Result<C::Input, Self::Error>;
}
