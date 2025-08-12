//! Stratified thermal storage tank component.
//!
//! Models a tank divided into `N` fully mixed vertical layers,
//! with energy and mass exchange via configured inlet ports,
//! auxiliary heat sources, and ambient boundary conditions.

#![allow(dead_code)]

mod buoyancy;
mod environment;
mod node;
mod port_flow;

use twine_core::TimeDerivative;
use twine_thermo::{HeatFlow, model::incompressible::IncompressibleFluid};
use uom::si::f64::ThermodynamicTemperature;

pub use environment::Environment;
pub use port_flow::PortFlow;

/// A stratified thermal energy storage tank.
///
/// Represents a fixed-geometry tank with `N` fully mixed vertical layers.
/// Supports energy exchange through `P` port pairs and `Q` auxiliary heat sources.
///
/// A **port pair** models a real-world connection between the tank and an
/// external hydraulic circuit:
/// - One end returns fluid to the tank at a known temperature.
/// - The other end draws fluid out of the tank at the same volumetric rate.
///
/// This pairing maintains mass balance in the tank.
/// The outlet temperature for the port pair comes from the layer(s) where the
/// outflow is taken.
///
/// The location of port pairs and auxiliary heat sources are fixed when the
/// tank is constructed; each may apply to a single layer or be split across
/// multiple layers.
///
/// Generic over:
/// - `F`: fluid type (must implement [`IncompressibleFluid`])
/// - `N`: number of layers
/// - `P`: number of port pairs
/// - `Q`: number of auxiliary heat sources
pub struct StratifiedTank<F: IncompressibleFluid, const N: usize, const P: usize, const Q: usize> {
    fluid: F,
}

/// Input to the stratified tank component.
///
/// Captures the runtime state needed to evaluate the tank's thermal response.
/// Locations of ports and auxiliary heat sources, and how they are distributed
/// across layers, are fixed when the tank is constructed.
/// This struct holds only the values that change at runtime.
///
/// Generic over:
/// - `N`: Number of stratified layers
/// - `P`: Number of port pairs
/// - `Q`: Number of auxiliary heat sources
pub struct Input<const N: usize, const P: usize, const Q: usize> {
    /// Temperatures of the `N` layers, from bottom to top.
    ///
    /// Each layer is fully mixed.
    /// Values do not need to be stratified; if warmer layers appear below
    /// cooler ones, the model mixes adjacent layers before computing energy balances.
    pub temperatures: [ThermodynamicTemperature; N],

    /// Flow rates and temperatures for the `P` port pairs.
    pub port_flows: [PortFlow; P],

    /// Heat input or extraction for the `Q` auxiliary sources.
    pub aux_heat_flows: [HeatFlow; Q],

    /// Ambient temperatures outside the tank.
    pub environment: Environment,
}

/// Output from the stratified tank component.
///
/// Captures the thermally stable layer temperatures and their time derivatives
/// after applying mass and energy balances.
///
/// Generic over:
/// - `N`: Number of stratified layers
pub struct Output<const N: usize> {
    /// Temperatures of the `N` layers, from bottom to top.
    ///
    /// These values are guaranteed to be thermally stable.
    pub temperatures: [ThermodynamicTemperature; N],

    /// Time derivatives of temperature in each layer.
    ///
    /// Includes the effects of mass flow, auxiliary heat input, and
    /// environmental heat loss or gain.
    pub derivatives: [TimeDerivative<ThermodynamicTemperature>; N],
}

impl<F: IncompressibleFluid, const N: usize, const P: usize, const Q: usize>
    StratifiedTank<F, N, P, Q>
{
    /// Evaluates the tank's thermal response at a single point in time.
    ///
    /// Enforces thermal stability by mixing unstable layers, then applies port
    /// flows, auxiliary heat input, and environmental effects.
    pub fn call(&self, _input: Input<N, P, Q>) -> Output<N> {
        todo!()
    }
}
