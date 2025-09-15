use uom::si::f64::ThermodynamicTemperature;

/// Ambient temperatures surrounding the tank.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Environment {
    /// Temperature below the tank.
    pub bottom: ThermodynamicTemperature,

    /// Temperature at the sides of the tank.
    pub side: ThermodynamicTemperature,

    /// Temperature above the tank.
    pub top: ThermodynamicTemperature,
}
