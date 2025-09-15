use std::{array, convert::Infallible, time::Duration};

use twine_core::{DurationExt, Simulation, State, TimeIntegrable};

use super::{ModelInput, TankModel};

/// A simulation of the `TankModel` using forward Euler integration.
#[derive(Debug)]
pub(super) struct TankSimulation<const N: usize> {
    model: TankModel<N>,
}

impl<const N: usize> TankSimulation<N> {
    pub(super) fn new(model: TankModel<N>) -> Self {
        Self { model }
    }
}

impl<const N: usize> Simulation<TankModel<N>> for TankSimulation<N> {
    type StepError = Infallible;

    fn model(&self) -> &TankModel<N> {
        &self.model
    }

    fn advance_time(
        &mut self,
        state: &State<TankModel<N>>,
        dt: Duration,
    ) -> Result<ModelInput<N>, Self::StepError> {
        let State { input, output } = state;
        let dt_time = dt.as_time();

        Ok(ModelInput {
            datetime: input.datetime + dt,
            element_state: output.element_state,
            t_tank: array::from_fn(|i| {
                output.tank.temperatures[i].step(output.tank.derivatives[i], dt_time)
            }),
            ..input.clone()
        })
    }
}
