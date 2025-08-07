//! Stratified thermal storage tank component.
//!
//! Models a tank divided into `N` fully mixed vertical layers,
//! with energy and mass exchange via configured inlet ports,
//! auxiliary heat sources, and ambient boundary conditions.

#![allow(dead_code)]

mod buoyancy;
mod environment;
mod port_flow;

use twine_core::TimeDerivative;
use twine_thermo::{HeatFlow, model::incompressible::IncompressibleFluid};
use uom::si::f64::ThermodynamicTemperature;

pub use environment::Environment;
pub use port_flow::PortFlow;

/// A stratified thermal energy storage tank.
///
/// Represents a fixed-geometry tank with `N` fully mixed vertical layers.
/// Supports energy exchange through `P` inlet port pairs and `Q` auxiliary heat sources.
///
/// Each port pair models a real-world connection where fluid enters the tank
/// at a configured location with a known temperature and flow rate, and leaves
/// from another configured location to preserve mass balance.
///
/// Generic over:
/// - `F`: Fluid type (must implement [`IncompressibleFluid`])
/// - `N`: Number of stratified layers
/// - `P`: Number of port pairs
/// - `Q`: Number of auxiliary heat sources
pub struct StratifiedTank<F: IncompressibleFluid, const N: usize, const P: usize, const Q: usize> {
    fluid: F,
}

/// Input to the stratified tank component.
///
/// Captures the full runtime state used to evaluate the tank's thermal behavior.
/// Layer geometry, port positions, and auxiliary heat source locations are
/// fixed at tank instantiation and not represented here.
///
/// Generic over:
/// - `N`: Number of stratified layers
/// - `P`: Number of inlet port pairs
/// - `Q`: Number of auxiliary heat sources
pub struct Input<const N: usize, const P: usize, const Q: usize> {
    /// Temperatures of the `N` vertical layers, from bottom to top.
    ///
    /// Each layer is assumed to be fully mixed.
    ///
    /// These values do not need to be stratified.
    /// If any warmer layers appear below cooler ones, the model will
    /// immediately mix adjacent layers to restore thermal stability before
    /// computing mass and energy balances.
    pub temperatures: [ThermodynamicTemperature; N],

    /// Inlet conditions for `P` configured port pairs.
    ///
    /// Each [`PortFlow`] specifies the volume rate and temperature of inflow to
    /// a preconfigured layer.
    ///
    /// A corresponding outflow at the same rate occurs at another preconfigured layer,
    /// which may be the same or different from the inflow layer, preserving mass balance
    /// within the tank.
    pub port_flows: [PortFlow; P],

    /// Auxiliary heat input or extraction applied to configured layers.
    ///
    /// Typical examples include immersion heaters or internal heat exchangers.
    pub aux_heat_flows: [HeatFlow; Q],

    /// Environmental temperatures affecting boundary heat exchange.
    ///
    /// Used to model thermal losses or gains to surrounding conditions.
    pub environment: Environment,
}

/// Output from the stratified tank component.
///
/// Represents the thermally stable layer temperatures and their time derivatives.
///
/// Generic over:
/// - `N`: Number of stratified layers
pub struct Output<const N: usize> {
    /// Resolved temperatures of the `N` vertical layers, from bottom to top.
    ///
    /// These values are guaranteed to be thermally stable (i.e., stratified).
    pub temperatures: [ThermodynamicTemperature; N],

    /// Time derivatives of temperature in each layer.
    ///
    /// Includes effects from mass flow, auxiliary heat input, and environmental exchange.
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
