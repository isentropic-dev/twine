use std::marker::PhantomData;

use thiserror::Error;
use twine_thermo::{State, capability::ThermoModel};
use uom::si::f64::{Power, ThermalConductance};

use crate::thermal::hx::discretized::{Given, HeatTransferRate, Known};

/// Entry point for solving a discretized heat exchanger.
///
/// The arrangement and node count are fixed by generics.
/// This type also provides the natural home for higher-level solve helpers,
/// such as iterating on UA to match target outlet states.
#[derive(Debug, Clone, Copy)]
pub struct DiscretizedHx<Arrangement, const N: usize> {
    _arrangement: PhantomData<Arrangement>,
}

/// Node states and performance metrics for a discretized heat exchanger.
///
/// Node arrays follow the physical layout from left (0) to right (N-1).
/// The top stream always flows from node 0 to node N-1.
/// The bottom stream flows from node 0 to node N-1 for parallel flow and from
/// node N-1 to node 0 for counterflow.
#[derive(Debug, Clone)]
pub struct Results<TopFluid, BottomFluid, const N: usize> {
    /// Top stream node states, ordered from left (0) to right (N-1).
    pub top: [State<TopFluid>; N],

    /// Bottom stream node states, ordered from left (0) to right (N-1).
    pub bottom: [State<BottomFluid>; N],

    /// Heat transfer rate.
    pub q_dot: HeatTransferRate,

    /// Total heat exchanger conductance.
    pub ua: ThermalConductance,
}

/// Errors that can occur while solving a discretized heat exchanger.
#[derive(Debug, Error)]
pub enum SolveError<TopFluid, BottomFluid, const N: usize> {
    /// A computed heat transfer rate was invalid.
    #[error("computed heat transfer rate is invalid: {q_dot:?}")]
    InvalidHeatTransferRate {
        /// Heat transfer rate that failed validation.
        q_dot: Power,
    },

    /// A Second Law violation occurred.
    #[error("second law violation")]
    SecondLawViolation {
        /// Top stream node states, ordered left to right.
        top: [State<TopFluid>; N],
        /// Bottom stream node states, ordered left to right.
        bottom: [State<BottomFluid>; N],
        /// Heat transfer rate.
        q_dot: HeatTransferRate,
    },

    /// A thermodynamic model operation failed.
    ///
    /// This failure can be from property evaluation or state construction.
    #[error("thermodynamic model failed: {context}")]
    ThermoModelFailed {
        /// Operation context for the thermodynamic model failure.
        context: String,
        /// Underlying thermodynamic model error.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

impl<Arrangement, const N: usize> DiscretizedHx<Arrangement, N> {
    /// Solve a discretized heat exchanger with a fixed arrangement and node count.
    ///
    /// # Errors
    ///
    /// Returns a [`SolveError`] if the solver fails to converge or violates
    /// physical constraints.
    pub fn solve<TopFluid, BottomFluid, TopModel, BottomModel>(
        known: &Known<TopFluid, BottomFluid>,
        given: Given,
        thermo_top: &TopModel,
        thermo_bottom: &BottomModel,
    ) -> Result<Results<TopFluid, BottomFluid, N>, SolveError<TopFluid, BottomFluid, N>>
    where
        Arrangement: 'static,
        TopFluid: Copy,
        BottomFluid: Copy,
        TopModel: ThermoModel<Fluid = TopFluid>,
        BottomModel: ThermoModel<Fluid = BottomFluid>,
    {
        solve::<Arrangement, N, TopFluid, BottomFluid, TopModel, BottomModel>(
            known,
            given,
            thermo_top,
            thermo_bottom,
        )
    }

    /// Solve a discretized heat exchanger when both streams share the same thermo model.
    ///
    /// This is a convenience wrapper around [`DiscretizedHx::solve`].
    ///
    /// # Errors
    ///
    /// Returns a [`SolveError`] if the solver fails to converge or violates
    /// physical constraints.
    pub fn solve_same<Fluid, Model>(
        known: &Known<Fluid, Fluid>,
        given: Given,
        thermo: &Model,
    ) -> Result<Results<Fluid, Fluid, N>, SolveError<Fluid, Fluid, N>>
    where
        Arrangement: 'static,
        Fluid: Copy,
        Model: ThermoModel<Fluid = Fluid>,
    {
        Self::solve(known, given, thermo, thermo)
    }
}

/// Internal solver implementation used by [`DiscretizedHx::solve`].
///
/// `known` supplies inlet states, mass flow rates, and pressure drops.
/// `given` supplies the additional constraint used to close the energy balance.
///
/// # Errors
///
/// Returns a [`SolveError`] if the solver fails to converge or violates
/// physical constraints.
///
/// # Panics
///
/// Panics unconditionally; this solver is not yet implemented.
#[allow(clippy::unimplemented, clippy::extra_unused_type_parameters)]
pub(crate) fn solve<Arrangement, const N: usize, TopFluid, BottomFluid, TopModel, BottomModel>(
    _known: &Known<TopFluid, BottomFluid>,
    _given: Given,
    _thermo_top: &TopModel,
    _thermo_bottom: &BottomModel,
) -> Result<Results<TopFluid, BottomFluid, N>, SolveError<TopFluid, BottomFluid, N>>
where
    Arrangement: 'static,
    TopFluid: Copy,
    BottomFluid: Copy,
    TopModel: ThermoModel<Fluid = TopFluid>,
    BottomModel: ThermoModel<Fluid = BottomFluid>,
{
    unimplemented!()
}
