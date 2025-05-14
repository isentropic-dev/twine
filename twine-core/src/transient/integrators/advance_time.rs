use std::{convert::Infallible, fmt::Debug};

use uom::si::f64::Time;

use crate::{
    transient::{Integrator, Simulation, Temporal},
    Component,
};

/// An integrator that advances simulation time without modifying state.
///
/// `AdvanceTime` implements the [`Integrator`] trait by incrementing the
/// input’s timestamp by a fixed time step `dt`, leaving other fields unchanged.
/// It is useful for time-driven systems that do not depend on internal state or
/// feedback for evolution.
#[derive(Debug)]
pub struct AdvanceTime;

impl<C> Integrator<C> for AdvanceTime
where
    C: Component,
    C::Input: Clone + Temporal,
{
    type Error = Infallible;

    /// Computes the next input by advancing the simulation time.
    ///
    /// This integrator increments the input’s timestamp by `dt`.
    /// All other fields are left unchanged.
    fn propose_input(&self, simulation: &Simulation<C>, dt: Time) -> Result<C::Input, Self::Error> {
        let current_step = simulation.current_step();
        let current_time = simulation.current_time();

        let new_time = current_time + dt;
        let next_input = current_step.input.clone().with_time(new_time);

        Ok(next_input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{
        f64::Time,
        time::{hour, minute},
    };

    use crate::transient::test_utils::EchoTime;

    #[test]
    fn advance_time_increments_simulation_time() {
        let start_time = Time::new::<hour>(1.0);
        let sim = Simulation::new(EchoTime, start_time).unwrap();

        let integrator = AdvanceTime;
        let dt = Time::new::<minute>(5.0);
        let next_input = integrator.propose_input(&sim, dt).unwrap();

        assert_eq!(next_input, Time::new::<minute>(65.0));
    }
}
