use std::convert::Infallible;

use twine_core::Component;
use uom::{
    si::{
        ISQ, Quantity, SI,
        acceleration::meter_per_second_squared,
        f64::{Acceleration, Force, Length, Mass, Velocity},
        force::newton,
        length::meter,
        mass::kilogram,
        velocity::meter_per_second,
    },
    typenum::{N2, P1, Z0},
};

/// A simple harmonic oscillator component.
///
/// Models a mass-spring system without damping. Computes the time derivatives
/// of position and velocity (i.e., velocity and acceleration) from the current
/// state (position and velocity) and physical parameters (stiffness and mass).
pub struct Oscillator;

/// Input to the oscillator component.
///
/// Includes the system's physical parameters and current state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Input {
    pub parameters: Parameters,
    pub state: State,
}

/// Physical parameters of the oscillator.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Parameters {
    pub mass: Mass,
    pub stiffness: Stiffness,
}

/// The oscillator's dynamic state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State {
    pub position: Length,
    pub velocity: Velocity,
}

/// Output from the oscillator component.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Output {
    pub velocity: Velocity,
    pub acceleration: Acceleration,
}

/// Stiffness is a force per unit length (N/m or kg/s²).
pub type Stiffness = Quantity<ISQ<Z0, P1, N2, Z0, Z0, Z0, Z0>, SI<f64>, f64>;

impl Input {
    /// Sets mass from a `uom::Mass`.
    #[must_use]
    pub fn mass(mut self, mass: Mass) -> Self {
        self.parameters.mass = mass;
        self
    }

    /// Sets mass in SI units (kg).
    #[must_use]
    pub fn mass_si(self, mass: f64) -> Self {
        self.mass(Mass::new::<kilogram>(mass))
    }

    /// Sets stiffness from a `Stiffness` quantity.
    #[must_use]
    pub fn stiffness(mut self, stiffness: Stiffness) -> Self {
        self.parameters.stiffness = stiffness;
        self
    }

    /// Sets stiffness in SI units (N/m).
    #[must_use]
    pub fn stiffness_si(self, stiffness: f64) -> Self {
        self.stiffness(Force::new::<newton>(stiffness) / Length::new::<meter>(1.0))
    }

    /// Sets position from a `uom::Length`.
    #[must_use]
    pub fn position(mut self, position: Length) -> Self {
        self.state.position = position;
        self
    }

    /// Sets position in SI units (m).
    #[must_use]
    pub fn position_si(self, position: f64) -> Self {
        self.position(Length::new::<meter>(position))
    }

    /// Sets velocity from a `uom::Velocity`.
    #[must_use]
    pub fn velocity(mut self, velocity: Velocity) -> Self {
        self.state.velocity = velocity;
        self
    }

    /// Sets velocity in SI units (m/s).
    #[must_use]
    pub fn velocity_si(self, velocity: f64) -> Self {
        self.velocity(Velocity::new::<meter_per_second>(velocity))
    }
}

impl Output {
    /// Creates an `Output` from `uom::Velocity` and `uom::Acceleration`.
    #[must_use]
    pub fn new(velocity: Velocity, acceleration: Acceleration) -> Self {
        Self {
            velocity,
            acceleration,
        }
    }

    /// Creates an `Output` from raw SI values (m/s and m/s²).
    #[must_use]
    pub fn from_si(velocity: f64, acceleration: f64) -> Self {
        Self::new(
            Velocity::new::<meter_per_second>(velocity),
            Acceleration::new::<meter_per_second_squared>(acceleration),
        )
    }
}

impl Component for Oscillator {
    type Input = Input;
    type Output = Output;
    type Error = Infallible;

    /// Computes velocity and acceleration from the current state and parameters.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        let Input {
            parameters: Parameters { mass, stiffness },
            state: State { position, velocity },
        } = input;

        Ok(Output {
            velocity,
            acceleration: -stiffness / mass * position,
        })
    }
}

impl Default for Input {
    /// Creates a default oscillator input with zero state and unit parameters.
    fn default() -> Self {
        Self {
            parameters: Parameters {
                mass: Mass::new::<kilogram>(1.0),
                stiffness: Force::new::<newton>(1.0) / Length::new::<meter>(1.0),
            },
            state: State {
                position: Length::new::<meter>(0.0),
                velocity: Velocity::new::<meter_per_second>(0.0),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_oscillator() {
        // At rest with zero position and velocity.
        let input = Input::default();
        let expected_output = Output::from_si(0.0, 0.0);
        assert_eq!(Oscillator.call(input).unwrap(), expected_output);

        // Displaced with nonzero position and velocity.
        let input = Input::default() //
            .position_si(2.0)
            .velocity_si(1.0);
        let expected_output = Output::from_si(1.0, -2.0);
        assert_eq!(Oscillator.call(input).unwrap(), expected_output);

        // Changing stiffness and mass affects the acceleration.
        let input = Input::default()
            .position_si(2.0)
            .velocity_si(1.0)
            .stiffness_si(0.5)
            .mass_si(4.0);
        let expected_output = Output::from_si(1.0, -0.25);
        assert_eq!(Oscillator.call(input).unwrap(), expected_output);
    }
}
