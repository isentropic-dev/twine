//! A simple harmonic oscillator.
//!
//! This lives here for now because it will be useful for developing and testing
//! a component that solves ODEs.

use std::convert::Infallible;

use uom::{
    si::{
        acceleration::meter_per_second_squared,
        f64::{Acceleration, Force, Length, Mass, Velocity},
        force::newton,
        length::meter,
        mass::kilogram,
        velocity::meter_per_second,
        Quantity, ISQ, SI,
    },
    typenum::{N2, P1, Z0},
};

use crate::Component;

/// Stiffness: force per unit length (N/m or kg/s²)
type Stiffness = Quantity<ISQ<Z0, P1, N2, Z0, Z0, Z0, Z0>, SI<f64>, f64>;

/// State of the oscillator: position and velocity.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct State {
    pub(crate) position: Length,
    pub(crate) velocity: Velocity,
}

/// Derivative of state: velocity and acceleration.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Derivative {
    pub(crate) position: Velocity,
    pub(crate) velocity: Acceleration,
}

/// Input to the oscillator: state and physical parameters.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Input {
    pub(crate) state: State,
    pub(crate) parameters: Parameters,
}

/// Oscillator parameters: stiffness and mass.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Parameters {
    pub(crate) stiffness: Stiffness,
    pub(crate) mass: Mass,
}

/// A component that computes oscillator state derivatives.
pub(crate) struct Oscillator;

impl Component for Oscillator {
    type Input = Input;
    type Output = Derivative;
    type Error = Infallible;

    /// Evaluates the derivative of the oscillator's state.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let Input {
            state: State { position, velocity },
            parameters: Parameters { stiffness, mass },
        } = input;

        Ok(Derivative {
            position: velocity,
            velocity: -stiffness / mass * position,
        })
    }
}

#[allow(dead_code)]
impl Input {
    /// Sets position using a `uom::Length`.
    pub(crate) fn position(mut self, position: Length) -> Self {
        self.state.position = position;
        self
    }

    /// Sets position in SI units (m).
    pub(crate) fn position_si(mut self, position: f64) -> Self {
        self.state.position = Length::new::<meter>(position);
        self
    }

    /// Sets velocity using a `uom::Velocity`.
    pub(crate) fn velocity(mut self, velocity: Velocity) -> Self {
        self.state.velocity = velocity;
        self
    }

    /// Sets velocity in SI units (m/s).
    pub(crate) fn velocity_si(mut self, velocity: f64) -> Self {
        self.state.velocity = Velocity::new::<meter_per_second>(velocity);
        self
    }

    /// Sets stiffness using a `Stiffness` quantity.
    pub(crate) fn stiffness(mut self, stiffness: Stiffness) -> Self {
        self.parameters.stiffness = stiffness;
        self
    }

    /// Sets stiffness in SI units (N/m).
    pub(crate) fn stiffness_si(mut self, stiffness: f64) -> Self {
        self.parameters.stiffness = Force::new::<newton>(stiffness) / Length::new::<meter>(1.0);
        self
    }

    /// Sets mass using a `uom::Mass`.
    pub(crate) fn mass(mut self, mass: Mass) -> Self {
        self.parameters.mass = mass;
        self
    }

    /// Sets mass in SI units (kg).
    pub(crate) fn mass_si(mut self, mass: f64) -> Self {
        self.parameters.mass = Mass::new::<kilogram>(mass);
        self
    }
}

impl Derivative {
    /// Creates a derivative using raw SI values (m/s and m/s²).
    pub(crate) fn new_si(position: f64, velocity: f64) -> Self {
        Self {
            position: Velocity::new::<meter_per_second>(position),
            velocity: Acceleration::new::<meter_per_second_squared>(velocity),
        }
    }
}

impl Default for Input {
    /// Creates a default oscillator input with zero state and unit parameters.
    fn default() -> Self {
        Self {
            state: State {
                position: Length::new::<meter>(0.0),
                velocity: Velocity::new::<meter_per_second>(0.0),
            },
            parameters: Parameters {
                stiffness: Force::new::<newton>(1.0) / Length::new::<meter>(1.0),
                mass: Mass::new::<kilogram>(1.0),
            },
        }
    }
}
