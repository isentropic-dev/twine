/// A value that can be used as input or output to a `Component`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Value {
    /// A boolean value (`true` or `false`).
    Boolean(bool),
    /// A 32-bit signed integer.
    Integer(i32),
    /// A 64-bit floating-point number.
    Number(f64),
}

/// Represents the kind of a `Value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ValueKind {
    Boolean,
    Integer,
    Number,
}

/// An error indicating slice length or kind mismatches during validation.
#[derive(Debug)]
pub(crate) enum SliceKindError {
    /// The lengths of the actual and expected slices do not match.
    LengthMismatch { expected: usize, actual: usize },
    /// A mismatch in kinds was found at a specific index.
    KindMismatch {
        index: usize,
        expected: ValueKind,
        actual: ValueKind,
    },
}

/// Validates that two iterators of `ValueKind` have the same length and matching kinds.
///
/// # Parameters
///
/// - `actual`: An iterator of the actual kinds.
/// - `expected`: An iterator of the expected kinds.
///
/// # Errors
///
/// - Returns `SliceKindError::LengthMismatch` if lengths differ.
/// - Returns `SliceKindError::KindMismatch` if any kind differs by index.
pub(crate) fn validate_kinds<I1, I2>(actual: I1, expected: I2) -> Result<(), SliceKindError>
where
    I1: ExactSizeIterator<Item = ValueKind>,
    I2: ExactSizeIterator<Item = ValueKind>,
{
    if actual.len() != expected.len() {
        return Err(SliceKindError::LengthMismatch {
            actual: actual.len(),
            expected: expected.len(),
        });
    }

    for (i, (actual_kind, expected_kind)) in actual.zip(expected).enumerate() {
        if actual_kind != expected_kind {
            return Err(SliceKindError::KindMismatch {
                index: i,
                actual: actual_kind,
                expected: expected_kind,
            });
        }
    }

    Ok(())
}

impl From<&Value> for ValueKind {
    fn from(value: &Value) -> Self {
        match value {
            Value::Boolean(_) => ValueKind::Boolean,
            Value::Integer(_) => ValueKind::Integer,
            Value::Number(_) => ValueKind::Number,
        }
    }
}

impl From<bool> for Value {
    fn from(val: bool) -> Self {
        Value::Boolean(val)
    }
}

impl From<i32> for Value {
    fn from(val: i32) -> Self {
        Value::Integer(val)
    }
}

impl From<f64> for Value {
    fn from(val: f64) -> Self {
        Value::Number(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_kinds_success() {
        let actual = [ValueKind::Boolean, ValueKind::Integer, ValueKind::Number];
        let expected = [ValueKind::Boolean, ValueKind::Integer, ValueKind::Number];

        let result = validate_kinds(actual.iter().copied(), expected.iter().copied());
        assert!(result.is_ok());
    }

    #[test]
    fn validate_kinds_length_mismatch() {
        let actual = [ValueKind::Boolean, ValueKind::Integer];
        let expected = [ValueKind::Boolean, ValueKind::Integer, ValueKind::Number];

        let err = validate_kinds(actual.iter().copied(), expected.iter().copied()).unwrap_err();
        match err {
            SliceKindError::LengthMismatch { expected, actual } => {
                assert_eq!(expected, 3);
                assert_eq!(actual, 2);
            }
            SliceKindError::KindMismatch { .. } => panic!("Unexpected error type"),
        }
    }

    #[test]
    fn validate_kinds_kind_mismatch() {
        let actual = [ValueKind::Boolean, ValueKind::Integer, ValueKind::Boolean];
        let expected = [ValueKind::Boolean, ValueKind::Integer, ValueKind::Number];

        let err = validate_kinds(actual.iter().copied(), expected.iter().copied()).unwrap_err();
        match err {
            SliceKindError::KindMismatch {
                index,
                expected,
                actual,
            } => {
                assert_eq!(index, 2);
                assert_eq!(expected, ValueKind::Number);
                assert_eq!(actual, ValueKind::Boolean);
            }
            SliceKindError::LengthMismatch { .. } => panic!("Unexpected error type"),
        }
    }
}
