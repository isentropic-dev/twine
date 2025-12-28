use std::sync::PoisonError;

use thiserror::Error;

use crate::PropertyError;

/// Errors returned by the [`CoolProp`](super::CoolProp) model.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum CoolPropError {
    #[error(transparent)]
    Rfluids(#[from] rfluids::native::CoolPropError),
    #[error("CoolProp abstract state mutex poisoned")]
    Poisoned,
}

impl<T> From<PoisonError<T>> for CoolPropError {
    fn from(_: PoisonError<T>) -> Self {
        CoolPropError::Poisoned
    }
}

impl From<CoolPropError> for PropertyError {
    fn from(error: CoolPropError) -> Self {
        match error {
            CoolPropError::Rfluids(message) => map_error_message(&message.to_string()),
            CoolPropError::Poisoned => PropertyError::Calculation {
                context: "CoolProp abstract state mutex poisoned".to_string(),
            },
        }
    }
}

/// The `rfluids::native::CoolPropError` type reports errors as strings,
/// and the wording may change over time.
/// This mapping aims to provide the most useful `PropertyError` variant.
/// Unknown error strings default to `PropertyError::Calculation`.
fn map_error_message(message: &str) -> PropertyError {
    const UNDEFINED_MARKERS: &[&str] = &["not defined"];
    const OUT_OF_DOMAIN_MARKERS: &[&str] = &[
        "not in range",
        "out of range",
        "outside the range of validity",
        "must be in range",
        "must be between",
        "quality must be",
    ];
    const INVALID_STATE_MARKERS: &[&str] =
        &["not a valid number", "invalid state", "invalid number"];

    let lowered = message.to_lowercase();
    let context = message.to_string();

    if contains_any(&lowered, UNDEFINED_MARKERS) {
        PropertyError::Undefined { context }
    } else if contains_any(&lowered, OUT_OF_DOMAIN_MARKERS) {
        PropertyError::OutOfDomain { context }
    } else if contains_any(&lowered, INVALID_STATE_MARKERS) {
        PropertyError::InvalidState { context }
    } else {
        PropertyError::Calculation { context }
    }
}

/// Returns `true` if any needle is found in the haystack.
fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_error_message_returns_undefined() {
        let error = map_error_message("inputs are not defined for this region");
        assert!(matches!(error, PropertyError::Undefined { .. }));
    }

    #[test]
    fn map_error_message_returns_out_of_domain() {
        let error = map_error_message("input 3.14 is out of range");
        assert!(matches!(error, PropertyError::OutOfDomain { .. }));
    }

    #[test]
    fn map_error_message_returns_invalid_state() {
        let error = map_error_message("p is not a valid number");
        assert!(matches!(error, PropertyError::InvalidState { .. }));
    }

    #[test]
    fn map_error_message_returns_calculation_by_default() {
        let error = map_error_message("some other failure");
        assert!(matches!(error, PropertyError::Calculation { .. }));
    }
}
