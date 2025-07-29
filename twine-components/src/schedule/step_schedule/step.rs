use std::{cmp::Ordering, fmt::Debug, ops::Range};

use thiserror::Error;

/// Associates a value with a non-empty, half-open time range.
///
/// A `Step` pairs a value with the range `[start, end)`, where `start < end`.
/// The type `T` must implement [`Ord`] and typically represents time.
///
/// Steps are used as building blocks for schedules like [`StepSchedule`],
/// which assign values to non-overlapping intervals.
///
/// # Examples
///
/// ```
/// use twine_components::schedule::step_schedule::Step;
///
/// let step = Step::new(0..10, "active").unwrap();
/// assert!(step.contains(&5));
/// assert_eq!(step.value(), &"active");
///
/// assert!(Step::new(2..2, "empty").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step<T, V> {
    range: Range<T>,
    value: V,
}

/// Error returned when attempting to create a [`Step`] with an empty range.
#[derive(Debug, Error)]
#[error("empty range: start ({start:?}) >= end ({end:?})")]
pub struct EmptyRangeError<T: Debug> {
    pub start: T,
    pub end: T,
}

impl<T: Debug + Ord, V> Step<T, V> {
    /// Creates a new `Step` with the given range and value.
    ///
    /// # Errors
    ///
    /// Returns an [`EmptyRangeError`] if the provided range is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use twine_components::schedule::step_schedule::Step;
    ///
    /// let step = Step::new(0..10, 42.0).unwrap();
    /// assert_eq!(step.start(), &0);
    /// assert_eq!(step.end(), &10);
    /// assert_eq!(step.value(), &42.0);
    ///
    /// assert!(Step::new(5..1, "invalid range").is_err());
    /// ```
    pub fn new(range: Range<T>, value: V) -> Result<Self, EmptyRangeError<T>> {
        if range.is_empty() {
            Err(EmptyRangeError {
                start: range.start,
                end: range.end,
            })
        } else {
            Ok(Self { range, value })
        }
    }

    /// Returns a reference to the range covered by this step.
    pub fn range(&self) -> &Range<T> {
        &self.range
    }

    /// Returns a reference to the value associated with this step.
    pub fn value(&self) -> &V {
        &self.value
    }

    /// Returns a reference to the inclusive start bound of the range.
    pub fn start(&self) -> &T {
        &self.range.start
    }

    /// Returns a reference to the exclusive end bound of the range.
    pub fn end(&self) -> &T {
        &self.range.end
    }

    /// Returns `true` if `time` falls within the range of this step.
    ///
    /// Equivalent to `self.range.contains(time)`.
    pub fn contains(&self, time: &T) -> bool {
        self.range.contains(time)
    }

    /// Returns `true` if this step's range overlaps with `other`'s range.
    ///
    /// Two steps overlap if their ranges share any values.
    ///
    /// # Examples
    ///
    /// ```
    /// use twine_components::schedule::step_schedule::Step;
    ///
    /// let a = Step::new(0..5, "a").unwrap();
    /// let b = Step::new(4..8, "b").unwrap();
    /// let c = Step::new(8..10, "c").unwrap();
    ///
    /// assert!(a.overlaps(&b));
    /// assert!(!b.overlaps(&c));
    /// ```
    pub fn overlaps(&self, other: &Self) -> bool {
        self.range.start < other.range.end && other.range.start < self.range.end
    }

    /// Returns a reference to the value if `time` is within this step's range.
    ///
    /// Returns `None` if `time` is outside the range.
    pub fn value_at(&self, time: &T) -> Option<&V> {
        if self.contains(time) {
            Some(&self.value)
        } else {
            None
        }
    }

    /// Returns how this step's range relates to a given time value.
    ///
    /// - [`Ordering::Less`] if the step ends at or before `time`
    /// - [`Ordering::Greater`] if the step starts after `time`
    /// - [`Ordering::Equal`] if `time` is within the step's range
    ///
    /// Useful for efficient searching (e.g., with `binary_search_by`).
    pub fn cmp_to_time(&self, time: &T) -> Ordering {
        if self.end() <= time {
            Ordering::Less
        } else if self.start() > time {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl<T: Debug + Ord, V> TryFrom<(Range<T>, V)> for Step<T, V> {
    type Error = EmptyRangeError<T>;

    fn try_from((range, value): (Range<T>, V)) -> Result<Self, Self::Error> {
        Step::new(range, value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_creation() {
        let step = Step::new(1..5, "a").unwrap();
        assert_eq!(step.start(), &1);
        assert_eq!(step.end(), &5);
        assert_eq!(step.value(), &"a");

        let err = Step::new(3..3, "x").unwrap_err();
        assert_eq!(err.start, 3);
        assert_eq!(err.end, 3);
    }

    #[test]
    fn value_at_works() {
        let step = Step::new(10..20, "middle").unwrap();
        assert_eq!(step.value_at(&0), None);
        assert_eq!(step.value_at(&10), Some(&"middle"));
        assert_eq!(step.value_at(&15), Some(&"middle"));
        assert_eq!(step.value_at(&20), None);
        assert_eq!(step.value_at(&25), None);
    }

    #[test]
    fn overlaps_works() {
        let a = Step::new(0..5, "a").unwrap();
        let b = Step::new(4..8, "b").unwrap();
        let c = Step::new(8..10, "c").unwrap();

        assert!(a.overlaps(&b));
        assert!(!b.overlaps(&c));
    }

    #[test]
    fn cmp_to_time_works() {
        let step = Step::new(10..20, "x").unwrap();
        assert_eq!(step.cmp_to_time(&5), std::cmp::Ordering::Greater);
        assert_eq!(step.cmp_to_time(&15), std::cmp::Ordering::Equal);
        assert_eq!(step.cmp_to_time(&25), std::cmp::Ordering::Less);
    }
}
