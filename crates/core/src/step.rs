/// A trait for types that can be stepped using their derivative.
///
/// Implementing this trait enables generic ODE solvers to work with the type by
/// stepping it via `derivative * delta`, where the derivative is with respect
/// to `Delta`. While typically used for time integration, `Delta` can represent
/// any independent variable (spatial coordinate, arc length, etc.).
///
/// `Delta` can be a plain scalar like `f64` or a dimensioned type like
/// `uom::Time` for compile-time unit checking.
pub trait StepIntegrable<Delta> {
    /// The derivative of the type with respect to `Delta`.
    type Derivative;

    /// Returns the value after stepping with a derivative and step size.
    #[must_use]
    fn step(&self, derivative: Self::Derivative, delta: Delta) -> Self;
}

/// Type alias for the derivative of a `StepIntegrable` type.
///
/// This is a convenience for accessing the [`StepIntegrable::Derivative`]
/// associated type without writing out the fully qualified syntax.
pub type DerivativeOf<T, Delta> = <T as StepIntegrable<Delta>>::Derivative;

#[cfg(test)]
mod tests {
    use super::*;

    // Scalar state and derivative
    #[derive(Debug, PartialEq)]
    struct Position(f64);
    struct Velocity(f64);

    impl StepIntegrable<f64> for Position {
        type Derivative = Velocity;

        fn step(&self, derivative: Velocity, delta: f64) -> Self {
            Position(self.0 + derivative.0 * delta)
        }
    }

    // Vector state and derivative
    #[derive(Debug, PartialEq)]
    struct StateVector(Vec<f64>);
    struct DerivativeVector(Vec<f64>);

    impl StepIntegrable<f64> for StateVector {
        type Derivative = DerivativeVector;

        fn step(&self, derivative: DerivativeVector, delta: f64) -> Self {
            let next = self
                .0
                .iter()
                .zip(derivative.0.iter())
                .map(|(s, d)| s + d * delta)
                .collect();
            StateVector(next)
        }
    }

    #[test]
    fn step_scalar_state() {
        let pos = Position(0.0);
        let vel = Velocity(2.0);
        let dt = 0.5;

        let next = pos.step(vel, dt);

        assert_eq!(next, Position(1.0));
    }

    #[test]
    fn step_vector_state() {
        let state = StateVector(vec![1.0, 2.0, 3.0]);
        let deriv = DerivativeVector(vec![0.1, 0.2, 0.3]);
        let dt = 10.0;

        let next = state.step(deriv, dt);

        assert_eq!(next, StateVector(vec![2.0, 4.0, 6.0]));
    }
}
