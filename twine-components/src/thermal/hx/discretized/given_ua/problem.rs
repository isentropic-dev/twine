//! Problem formulation for iterative UA matching.

use std::{convert::Infallible, marker::PhantomData};

use twine_core::model::Model;
use twine_solve::equation::EquationProblem;
use uom::si::{
    f64::{ThermalConductance, ThermodynamicTemperature},
    thermal_conductance::watt_per_kelvin,
    thermodynamic_temperature::kelvin,
};

use crate::thermal::hx::discretized::{
    DiscretizedHx, Given, Known, Results, SolveError,
    traits::{DiscretizedArrangement, DiscretizedHxThermoModel},
};

/// Model adapter for target-based UA solving.
///
/// Wraps the base discretized solver and exposes the top stream outlet
/// temperature as the sole input variable to the model.
pub(super) struct GivenUaModel<
    'a,
    Arrangement,
    TopFluid,
    BottomFluid,
    TopThermo,
    BottomThermo,
    const N: usize,
> {
    known: &'a Known<TopFluid, BottomFluid>,
    thermo_top: &'a TopThermo,
    thermo_bottom: &'a BottomThermo,
    _arrangement: PhantomData<Arrangement>,
}

impl<'a, Arrangement, TopFluid, BottomFluid, TopThermo, BottomThermo, const N: usize>
    GivenUaModel<'a, Arrangement, TopFluid, BottomFluid, TopThermo, BottomThermo, N>
{
    pub(super) fn new(
        known: &'a Known<TopFluid, BottomFluid>,
        thermo_top: &'a TopThermo,
        thermo_bottom: &'a BottomThermo,
    ) -> Self {
        Self {
            known,
            thermo_top,
            thermo_bottom,
            _arrangement: PhantomData,
        }
    }
}

impl<Arrangement, TopFluid, BottomFluid, TopThermo, BottomThermo, const N: usize> Model
    for GivenUaModel<'_, Arrangement, TopFluid, BottomFluid, TopThermo, BottomThermo, N>
where
    Arrangement: DiscretizedArrangement + Default,
    TopFluid: Clone,
    BottomFluid: Clone,
    TopThermo: DiscretizedHxThermoModel<TopFluid>,
    BottomThermo: DiscretizedHxThermoModel<BottomFluid>,
{
    type Input = ThermodynamicTemperature;
    type Output = Results<TopFluid, BottomFluid, N>;
    type Error = SolveError;

    fn call(&self, input: &Self::Input) -> Result<Self::Output, Self::Error> {
        let given = Given::TopOutletTemp(*input);
        DiscretizedHx::<Arrangement, N>::solve(
            self.known,
            given,
            self.thermo_top,
            self.thermo_bottom,
        )
    }
}

/// Equation problem definition for UA matching.
///
/// Computes the residual as `achieved_ua - target_ua`.
pub(super) struct GivenUaProblem<TopFluid, BottomFluid, const N: usize> {
    target_ua: ThermalConductance,
    _fluids: PhantomData<(TopFluid, BottomFluid)>,
}

impl<TopFluid, BottomFluid, const N: usize> GivenUaProblem<TopFluid, BottomFluid, N> {
    pub(super) fn new(target_ua: ThermalConductance) -> Self {
        Self {
            target_ua,
            _fluids: PhantomData,
        }
    }
}

impl<TopFluid, BottomFluid, const N: usize> EquationProblem<1>
    for GivenUaProblem<TopFluid, BottomFluid, N>
{
    type Input = ThermodynamicTemperature;
    type Output = Results<TopFluid, BottomFluid, N>;
    type InputError = Infallible;
    type ResidualError = Infallible;

    fn input(&self, x: &[f64; 1]) -> Result<Self::Input, Self::InputError> {
        Ok(ThermodynamicTemperature::new::<kelvin>(x[0]))
    }

    fn residuals(
        &self,
        _input: &Self::Input,
        output: &Self::Output,
    ) -> Result<[f64; 1], Self::ResidualError> {
        let ua = output.ua.get::<watt_per_kelvin>();
        let target = self.target_ua.get::<watt_per_kelvin>();
        Ok([ua - target])
    }
}
