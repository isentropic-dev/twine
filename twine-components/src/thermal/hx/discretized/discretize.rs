//! Core discretization algorithm for heat exchangers.
//!
//! This module converts resolved boundary conditions into node arrays by:
//! 1. Computing linear pressure drops and energy-balance enthalpies
//! 2. Converting (P, h) pairs into thermodynamic states

use std::mem::MaybeUninit;

use twine_thermo::{State, capability::StateFrom, units::SpecificEnthalpy};
use uom::si::f64::{Power, Pressure};

use super::{
    error::SolveError,
    resolve::Resolved,
    traits::{DiscretizedArrangement, DiscretizedHxThermoModel},
};

/// Discretized node arrays for a solved heat exchanger.
#[derive(Debug)]
pub(super) struct Nodes<TopFluid, BottomFluid, const N: usize> {
    pub(super) top: [State<TopFluid>; N],
    pub(super) bottom: [State<BottomFluid>; N],
    pub(super) top_enthalpies: [SpecificEnthalpy; N],
    pub(super) bottom_enthalpies: [SpecificEnthalpy; N],
}

impl<TopFluid, BottomFluid, const N: usize> Nodes<TopFluid, BottomFluid, N> {
    /// Discretizes the heat exchanger into N nodes.
    ///
    /// This function breaks the heat exchanger into segments and computes node
    /// states from the resolved endpoint conditions.
    pub(super) fn new<Arrangement>(
        resolved: &Resolved<TopFluid, BottomFluid>,
        thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
        thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
    ) -> Result<Self, SolveError>
    where
        Arrangement: DiscretizedArrangement,
        TopFluid: Clone,
        BottomFluid: Clone,
    {
        let q_signed = resolved.q_dot.signed_top_to_bottom();

        // Step 1: Compute node arrays (pressures and enthalpies)
        let arrays = compute_node_arrays::<Arrangement, N>(resolved, q_signed);

        // Step 2: Build State objects from (P, h) pairs using thermo models
        let states = build_node_states::<Arrangement, TopFluid, BottomFluid, N>(
            resolved,
            thermo_top,
            thermo_bottom,
            &arrays,
        )?;

        Ok(Nodes {
            top: states.top,
            bottom: states.bottom,
            top_enthalpies: arrays.top_enthalpies,
            bottom_enthalpies: arrays.bottom_enthalpies,
        })
    }
}

/// Node property arrays computed during discretization.
struct NodeArrays<const N: usize> {
    top_pressures: [Pressure; N],
    bottom_pressures: [Pressure; N],
    top_enthalpies: [SpecificEnthalpy; N],
    bottom_enthalpies: [SpecificEnthalpy; N],
}

/// Paired node state arrays for top and bottom streams.
struct NodeStates<TopFluid, BottomFluid, const N: usize> {
    top: [State<TopFluid>; N],
    bottom: [State<BottomFluid>; N],
}

/// Computes pressure and enthalpy arrays for all nodes.
fn compute_node_arrays<Arrangement, const N: usize>(
    resolved: &Resolved<impl Clone, impl Clone>,
    q_signed: Power,
) -> NodeArrays<N>
where
    Arrangement: DiscretizedArrangement,
{
    // Compute outlet enthalpies from energy balance
    let h_top_out = resolved.top.h_in - q_signed / resolved.top.m_dot;
    let h_bottom_out = resolved.bottom.h_in + q_signed / resolved.bottom.m_dot;

    // Pressure arrays: linear interpolation from inlet to outlet
    let top_pressures = linear_array(resolved.top.p_in, resolved.top.p_out);
    let bottom_pressures = Arrangement::bottom_select(
        linear_array(resolved.bottom.p_in, resolved.bottom.p_out),
        linear_array(resolved.bottom.p_out, resolved.bottom.p_in),
    );

    // Enthalpy arrays: linear interpolation from inlet to outlet
    let top_enthalpies = linear_array(resolved.top.h_in, h_top_out);
    let bottom_enthalpies = Arrangement::bottom_select(
        linear_array(resolved.bottom.h_in, h_bottom_out),
        linear_array(h_bottom_out, resolved.bottom.h_in),
    );

    NodeArrays {
        top_pressures,
        bottom_pressures,
        top_enthalpies,
        bottom_enthalpies,
    }
}

/// Creates an array via linear interpolation: `[start, ..., end]` with N evenly-spaced points.
#[inline]
fn linear_array<T, const N: usize>(start: T, end: T) -> [T; N]
where
    T: Copy
        + std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<f64, Output = T>,
{
    #[allow(clippy::cast_precision_loss)]
    let segments = (N - 1) as f64;
    let span = end - start;
    std::array::from_fn(|i| {
        #[allow(clippy::cast_precision_loss)]
        let i = i as f64;
        start + span * (i / segments)
    })
}

/// Builds state arrays from computed node properties.
fn build_node_states<Arrangement, TopFluid, BottomFluid, const N: usize>(
    resolved: &Resolved<TopFluid, BottomFluid>,
    thermo_top: &impl DiscretizedHxThermoModel<TopFluid>,
    thermo_bottom: &impl DiscretizedHxThermoModel<BottomFluid>,
    arrays: &NodeArrays<N>,
) -> Result<NodeStates<TopFluid, BottomFluid, N>, SolveError>
where
    Arrangement: DiscretizedArrangement,
    TopFluid: Clone,
    BottomFluid: Clone,
{
    let top_states = build_states(
        thermo_top,
        "top",
        &resolved.top.inlet,
        &resolved.top.outlet,
        &arrays.top_pressures,
        &arrays.top_enthalpies,
        true,
    )?;

    let bottom_states = build_states(
        thermo_bottom,
        "bottom",
        &resolved.bottom.inlet,
        &resolved.bottom.outlet,
        &arrays.bottom_pressures,
        &arrays.bottom_enthalpies,
        Arrangement::bottom_select(true, false),
    )?;

    Ok(NodeStates {
        top: top_states,
        bottom: bottom_states,
    })
}

/// Builds a state array from pressure and enthalpy arrays.
///
/// The inlet and outlet states are placed at their respective array positions
/// based on `inlet_at_start`. All other positions are computed from thermodynamic
/// models using `StateFrom<(Fluid, Pressure, SpecificEnthalpy)>`.
///
/// # Performance
///
/// Uses `MaybeUninit` to build the array without heap allocation while
/// preserving the ability to return early on thermodynamic errors.
/// This is the idiomatic approach for fallible array construction in Rust.
fn build_states<Fluid, const N: usize>(
    thermo: &impl StateFrom<(Fluid, Pressure, SpecificEnthalpy), Fluid = Fluid>,
    side: &'static str,
    inlet: &State<Fluid>,
    outlet: &State<Fluid>,
    pressures: &[Pressure; N],
    enthalpies: &[SpecificEnthalpy; N],
    inlet_at_start: bool,
) -> Result<[State<Fluid>; N], SolveError>
where
    Fluid: Clone,
{
    let (inlet_index, outlet_index) = if inlet_at_start {
        (0, N - 1)
    } else {
        (N - 1, 0)
    };

    // Initialize uninitialized array on the stack - zero heap allocation
    let mut states: [MaybeUninit<State<Fluid>>; N] = unsafe { MaybeUninit::uninit().assume_init() };

    // Place known inlet and outlet states at their indices
    states[inlet_index] = MaybeUninit::new(inlet.clone());
    states[outlet_index] = MaybeUninit::new(outlet.clone());

    // Compute intermediate states from (P, h) pairs, with early return on error
    for i in 1..(N - 1) {
        let state = thermo
            .state_from((inlet.fluid.clone(), pressures[i], enthalpies[i]))
            .map_err(|err| {
                SolveError::thermo_failed(format!("state_from({side} node {i})"), err)
            })?;
        states[i] = MaybeUninit::new(state);
    }

    // Safety: All N elements have been initialized:
    // - Endpoints at inlet_index and outlet_index (always 0 and N-1)
    // - Interior nodes at 1..(N-1) in the loop
    // MaybeUninit<T> has the same memory layout as T, so this cast is safe.
    let states_ptr = (&raw const states).cast::<[State<Fluid>; N]>();
    Ok(unsafe { states_ptr.read() })
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::{MassRate, Power},
        mass_rate::kilogram_per_second,
        power::kilowatt,
        thermodynamic_temperature::kelvin,
    };

    use crate::thermal::hx::{
        arrangement::CounterFlow,
        discretized::{
            Given, HeatTransferRate, Inlets, Known, MassFlows, PressureDrops,
            resolve::Resolved,
            test_support::{TestThermoModel, state},
        },
    };

    #[test]
    fn counterflow_orders_bottom_from_right_to_left() {
        let model = TestThermoModel::new();

        let known = Known {
            inlets: Inlets {
                top: state(400.0),
                bottom: state(300.0),
            },
            m_dot: MassFlows::new_unchecked(
                MassRate::new::<kilogram_per_second>(1.0),
                MassRate::new::<kilogram_per_second>(1.0),
            ),
            dp: PressureDrops::default(),
        };

        let q_dot = HeatTransferRate::TopToBottom(Power::new::<kilowatt>(30.0));
        let resolved = Resolved::new(&known, Given::HeatTransferRate(q_dot), &model, &model)
            .expect("resolution should succeed");

        let nodes = Nodes::<_, _, 3>::new::<CounterFlow>(&resolved, &model, &model)
            .expect("discretization should succeed");

        assert_relative_eq!(nodes.bottom[0].temperature.get::<kelvin>(), 330.0);
        assert_relative_eq!(nodes.bottom[2].temperature.get::<kelvin>(), 300.0);
    }
}
