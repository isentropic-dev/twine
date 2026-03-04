use thiserror::Error;

/// Errors that can occur when creating a [`Bracket`] or validating bounds.
#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum BracketError {
    /// One or both endpoints are non-finite.
    #[error("non-finite endpoint(s)")]
    NonFinite,
    /// Endpoints are equal, giving zero width.
    #[error("zero width")]
    ZeroWidth,
    /// Left endpoint is greater than right endpoint.
    #[error("left > right")]
    Inverted,
    /// Residual signs do not bracket a root.
    #[error("no sign change")]
    NoSignChange,
}

/// Current bracket bounds and their residual signs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bracket {
    left: f64,
    right: f64,
    left_sign: Sign,
    right_sign: Sign,
}

impl Bracket {
    /// Creates a validated bracket from left and right endpoint–sign pairs.
    ///
    /// Left must be strictly less than right.
    ///
    /// # Errors
    ///
    /// Returns [`BracketError::NonFinite`] if either endpoint is non-finite,
    /// [`BracketError::ZeroWidth`] if the endpoints are equal,
    /// [`BracketError::Inverted`] if left > right, or
    /// [`BracketError::NoSignChange`] if the signs are the same.
    pub fn new(left: (f64, Sign), right: (f64, Sign)) -> Result<Self, BracketError> {
        let (left, left_sign) = left;
        let (right, right_sign) = right;

        if !left.is_finite() || !right.is_finite() {
            return Err(BracketError::NonFinite);
        }

        // Exact equality is intentional — zero-width brackets are invalid.
        #[allow(clippy::float_cmp)]
        if left == right {
            return Err(BracketError::ZeroWidth);
        }

        if left > right {
            return Err(BracketError::Inverted);
        }

        if left_sign == right_sign {
            return Err(BracketError::NoSignChange);
        }

        Ok(Self {
            left,
            right,
            left_sign,
            right_sign,
        })
    }

    /// Creates a bracket from pre-validated, pre-ordered bounds and signs.
    ///
    /// This skips endpoint validation (finiteness, ordering) since `Bounds`
    /// already enforces those invariants. Only validates sign opposition.
    pub(crate) fn from_bounds(
        bounds: Bounds,
        left_sign: Sign,
        right_sign: Sign,
    ) -> Result<Self, BracketError> {
        if left_sign == right_sign {
            return Err(BracketError::NoSignChange);
        }

        Ok(Self {
            left: bounds.left,
            right: bounds.right,
            left_sign,
            right_sign,
        })
    }

    /// Returns the bracket bounds as an array.
    #[must_use]
    pub fn as_array(&self) -> [f64; 2] {
        [self.left, self.right]
    }

    /// Returns the midpoint of the bracket.
    #[must_use]
    pub fn midpoint(&self) -> f64 {
        0.5 * (self.left + self.right)
    }

    /// Returns the bracket width.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.right - self.left
    }

    /// Returns true if the bracket width satisfies the x tolerances.
    #[must_use]
    pub fn is_x_converged(&self, x_abs_tol: f64, x_rel_tol: f64) -> bool {
        let mid = self.midpoint();
        self.width() <= x_abs_tol + x_rel_tol * mid.abs()
    }

    /// Shrinks the bracket using a new endpoint and its residual sign.
    pub(crate) fn shrink(&mut self, x: f64, sign: Sign) {
        if self.left_sign == sign {
            self.left = x;
            self.left_sign = sign;
        } else {
            self.right = x;
            self.right_sign = sign;
        }
    }
}

/// The sign of a residual for bracket logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    /// Residual is positive (or zero).
    Positive,
    /// Residual is negative.
    Negative,
}

impl Sign {
    /// Returns the sign of a residual value.
    #[must_use]
    pub fn of(value: f64) -> Self {
        if value >= 0.0 {
            Sign::Positive
        } else {
            Sign::Negative
        }
    }
}

/// Ordered finite bounds for a bracket.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Bounds {
    left: f64,
    right: f64,
}

impl Bounds {
    /// Validates and orders the bracket endpoints.
    ///
    /// # Errors
    ///
    /// Returns `BracketError` if endpoints are non-finite or zero width.
    pub(crate) fn new(bracket: [f64; 2]) -> Result<Self, BracketError> {
        let [left, right] = bracket;

        if !left.is_finite() || !right.is_finite() {
            return Err(BracketError::NonFinite);
        }

        // Exact equality is intentional — zero-width brackets are invalid.
        #[allow(clippy::float_cmp)]
        if left == right {
            return Err(BracketError::ZeroWidth);
        }

        if left < right {
            Ok(Self { left, right })
        } else {
            Ok(Self {
                left: right,
                right: left,
            })
        }
    }

    /// Returns the bounds as an array.
    pub(crate) fn as_array(&self) -> [f64; 2] {
        [self.left, self.right]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;

    #[test]
    fn bounds_reorders_bracket() {
        let bounds = Bounds::new([3.0, 1.0]).expect("valid bracket");
        assert_relative_eq!(bounds.left, 1.0);
        assert_relative_eq!(bounds.right, 3.0);
    }

    #[test]
    fn bounds_rejects_non_finite() {
        assert!(matches!(
            Bounds::new([f64::NAN, 1.0]),
            Err(BracketError::NonFinite)
        ));
        assert!(matches!(
            Bounds::new([0.0, f64::INFINITY]),
            Err(BracketError::NonFinite)
        ));
    }

    #[test]
    fn bounds_rejects_zero_width() {
        assert!(matches!(
            Bounds::new([2.0, 2.0]),
            Err(BracketError::ZeroWidth)
        ));
    }

    #[test]
    fn new_rejects_non_finite() {
        assert!(matches!(
            Bracket::new((f64::NAN, Sign::Negative), (1.0, Sign::Positive)),
            Err(BracketError::NonFinite)
        ));
    }

    #[test]
    fn new_rejects_zero_width() {
        assert!(matches!(
            Bracket::new((2.0, Sign::Negative), (2.0, Sign::Positive)),
            Err(BracketError::ZeroWidth)
        ));
    }

    #[test]
    fn new_rejects_inverted() {
        assert!(matches!(
            Bracket::new((10.0, Sign::Negative), (0.0, Sign::Positive)),
            Err(BracketError::Inverted)
        ));
    }

    #[test]
    fn new_rejects_no_sign_change() {
        assert!(matches!(
            Bracket::new((0.0, Sign::Positive), (1.0, Sign::Positive)),
            Err(BracketError::NoSignChange)
        ));
    }

    #[test]
    fn shrink_shifts_bounds() {
        let mut bracket =
            Bracket::new((0.0, Sign::Negative), (2.0, Sign::Positive)).expect("valid bracket");

        bracket.shrink(1.0, Sign::Negative);
        let [left, right] = bracket.as_array();
        assert_relative_eq!(left, 1.0);
        assert_relative_eq!(right, 2.0);

        bracket.shrink(1.5, Sign::Positive);
        let [left, right] = bracket.as_array();
        assert_relative_eq!(left, 1.0);
        assert_relative_eq!(right, 1.5);
    }
}
