//! Discretized counterflow and parallel-flow heat exchanger modeling.
//!
//! A discretized heat exchanger divides the flow into a linear series of
//! constant-property sub-exchangers so thermodynamic properties can vary
//! along a linear array of nodes, supporting real-fluid behavior.

mod given_ua;
mod heat_transfer_rate;
mod input;
mod metrics;
mod results;
mod solve;
mod traits;

#[cfg(test)]
mod test_support;

pub use given_ua::{GivenUaConfig, GivenUaError};
pub use heat_transfer_rate::HeatTransferRate;
pub use input::{Given, Inlets, Known, MassFlows, PressureDrops};
pub use results::{MinDeltaT, Results};
pub use solve::SolveError;

use std::marker::PhantomData;

use twine_core::constraint::{Constrained, NonNegative};
use uom::si::f64::ThermalConductance;

use given_ua::given_ua;
use solve::solve;
use traits::{DiscretizedArrangement, DiscretizedHxThermoModel};

/// Entry point for solving a discretized heat exchanger.
///
/// The arrangement and node count are fixed by generics.
/// This type also provides the natural home for higher-level solve helpers,
/// such as iterating on UA to match target outlet states.
///
/// # Minimum Node Count
///
/// The node count `N` must be at least 2 (inlet and outlet).
/// This constraint is enforced at compile time via const assertions.
///
/// ```compile_fail
/// # use twine_components::thermal::hx::{arrangement::ParallelFlow, discretized::DiscretizedHx};
/// // This will fail to compile: N must be >= 2
/// let _ = DiscretizedHx::<ParallelFlow, 1>::solve(todo!(), todo!(), todo!(), todo!());
/// ```
///
/// # Examples
///
/// Basic solve with known heat transfer rate:
///
/// ```rust
/// use twine_components::thermal::hx::{
///     arrangement::CounterFlow,
///     discretized::{DiscretizedHx, Given, HeatTransferRate, Inlets, Known, MassFlows, PressureDrops},
/// };
/// use twine_thermo::{fluid::CarbonDioxide, model::perfect_gas::PerfectGas};
/// use uom::si::f64::{MassRate, Pressure};
/// use uom::si::{mass_rate::kilogram_per_second, pressure::pascal};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let thermo = PerfectGas::<CarbonDioxide>::new()?;
/// let inlet = thermo.reference_state(CarbonDioxide);
///
/// let known = Known {
///     inlets: Inlets {
///         top: inlet,
///         bottom: inlet,
///     },
///     m_dot: MassFlows::new(
///         MassRate::new::<kilogram_per_second>(1.0),
///         MassRate::new::<kilogram_per_second>(1.0),
///     )?,
///     dp: PressureDrops::zero(),
/// };
///
/// let given = Given::HeatTransferRate(HeatTransferRate::None);
///
/// let _result = DiscretizedHx::<CounterFlow, 20>::solve(&known, given, &thermo, &thermo)?;
/// # Ok(())
/// # }
/// ```
///
/// Iterative solve to match a target conductance (UA):
///
/// ```rust
/// use twine_components::thermal::hx::{
///     arrangement::CounterFlow,
///     discretized::{DiscretizedHx, GivenUaConfig, Inlets, Known, MassFlows, PressureDrops},
/// };
/// use twine_core::constraint::NonNegative;
/// use twine_thermo::{capability::StateFrom, fluid::CarbonDioxide, model::perfect_gas::PerfectGas};
/// use uom::si::f64::{MassRate, Pressure, ThermalConductance, ThermodynamicTemperature};
/// use uom::si::{
///     mass_rate::kilogram_per_second,
///     pressure::pascal,
///     thermal_conductance::watt_per_kelvin,
///     thermodynamic_temperature::kelvin,
/// };
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let thermo = PerfectGas::<CarbonDioxide>::new()?;
///
/// // Create inlet states at different temperatures
/// let p = Pressure::new::<pascal>(101325.0);
/// let top_inlet = thermo.state_from((
///     CarbonDioxide,
///     ThermodynamicTemperature::new::<kelvin>(400.0),
///     p,
/// ))?;
/// let bottom_inlet = thermo.state_from((
///     CarbonDioxide,
///     ThermodynamicTemperature::new::<kelvin>(300.0),
///     p,
/// ))?;
///
/// let known = Known {
///     inlets: Inlets {
///         top: top_inlet,
///         bottom: bottom_inlet,
///     },
///     m_dot: MassFlows::new(
///         MassRate::new::<kilogram_per_second>(1.0),
///         MassRate::new::<kilogram_per_second>(1.0),
///     )?,
///     dp: PressureDrops::zero(),
/// };
///
/// let target_ua = NonNegative::new(ThermalConductance::new::<watt_per_kelvin>(1000.0))?;
///
/// let _result = DiscretizedHx::<CounterFlow, 20>::given_ua_same(
///     &known,
///     target_ua,
///     GivenUaConfig::default(),
///     &thermo,
/// )?;
/// # Ok(())
/// # }
/// ```
pub struct DiscretizedHx<Arrangement, const N: usize> {
    _arrangement: PhantomData<Arrangement>,
}

impl<Arrangement, const N: usize> DiscretizedHx<Arrangement, N> {
    /// Solves a discretized heat exchanger with a fixed arrangement and node count.
    ///
    /// # Errors
    ///
    /// Returns a [`SolveError`] on non-physical results or thermodynamic model failures.
    pub fn solve<TopFluid, BottomFluid>(
        known: &Known<TopFluid, BottomFluid>,
        given: Given,
        thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
        thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
    ) -> Result<Results<TopFluid, BottomFluid, N>, SolveError>
    where
        Arrangement: DiscretizedArrangement + Default,
        TopFluid: Clone,
        BottomFluid: Clone,
    {
        solve::<Arrangement, _, _, N>(known, given, thermo_top, thermo_bottom)
    }

    /// Solves a discretized heat exchanger when both streams share the same thermo model.
    ///
    /// This is a convenience wrapper around [`DiscretizedHx::solve`].
    ///
    /// # Errors
    ///
    /// Returns a [`SolveError`] on non-physical results or thermodynamic model failures.
    pub fn solve_same<Fluid, Model>(
        known: &Known<Fluid, Fluid>,
        given: Given,
        thermo: &Model,
    ) -> Result<Results<Fluid, Fluid, N>, SolveError>
    where
        Arrangement: DiscretizedArrangement + Default,
        Fluid: Clone,
        Model: DiscretizedHxThermoModel<Fluid>,
    {
        solve::<Arrangement, _, _, N>(known, given, thermo, thermo)
    }

    /// Solves a discretized heat exchanger given a target conductance (UA).
    ///
    /// Iterates on the top outlet temperature to achieve the specified
    /// thermal conductance.
    ///
    /// # Errors
    ///
    /// Returns a [`GivenUaError`] on non-physical results, thermodynamic model failures,
    /// or if the solver fails to converge.
    pub fn given_ua<TopFluid, BottomFluid>(
        known: &Known<TopFluid, BottomFluid>,
        target_ua: Constrained<ThermalConductance, NonNegative>,
        config: GivenUaConfig,
        thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
        thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
    ) -> Result<Results<TopFluid, BottomFluid, N>, GivenUaError>
    where
        Arrangement: DiscretizedArrangement + Default,
        TopFluid: Clone,
        BottomFluid: Clone,
    {
        given_ua::<Arrangement, _, _, N>(known, target_ua, config, thermo_top, thermo_bottom)
    }

    /// Solves a discretized heat exchanger given a target UA when both streams share the same thermo model.
    ///
    /// This is a convenience wrapper around [`DiscretizedHx::given_ua`].
    ///
    /// # Errors
    ///
    /// Returns a [`GivenUaError`] on non-physical results, thermodynamic model failures,
    /// or if the solver fails to converge.
    pub fn given_ua_same<Fluid, Model>(
        known: &Known<Fluid, Fluid>,
        target_ua: Constrained<ThermalConductance, NonNegative>,
        config: GivenUaConfig,
        thermo: &Model,
    ) -> Result<Results<Fluid, Fluid, N>, GivenUaError>
    where
        Arrangement: DiscretizedArrangement + Default,
        Fluid: Clone,
        Model: DiscretizedHxThermoModel<Fluid>,
    {
        given_ua::<Arrangement, _, _, N>(known, target_ua, config, thermo, thermo)
    }
}
