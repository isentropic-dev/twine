mod step;

use std::{convert::Infallible, fmt::Debug, ops::Range};

use thiserror::Error;
use twine_core::Component;

pub use step::{EmptyRangeError, Step};

/// Threshold below which [`StepSchedule::value_at`] uses linear search.
///
/// For schedules with fewer than this many steps, a linear scan is used.
/// Otherwise, binary search is performed for faster lookups in larger schedules.
const LINEAR_SEARCH_THRESHOLD: usize = 32;

/// Associates values with distinct, non-overlapping time ranges.
///
/// A `StepSchedule` is a collection of [`Step`]s, each mapping a value to a
/// non-empty, non-overlapping half-open range `[start, end)`.
///
/// The range type `T` must implement [`Ord`], and usually represents time,
/// though any ordered type (e.g., numbers or indices) is supported.
///
/// # Examples
///
/// ```
/// use twine_components::schedule::step_schedule::{Step, StepSchedule};
///
/// let schedule = StepSchedule::new([
///     Step::new(0..10, "low").unwrap(),
///     Step::new(10..20, "medium").unwrap(),
///     Step::new(20..30, "high").unwrap(),
/// ]).unwrap();
///
/// assert_eq!(schedule.value_at(&-1), None);
/// assert_eq!(schedule.value_at(&0), Some(&"low"));
/// assert_eq!(schedule.value_at(&15), Some(&"medium"));
/// assert_eq!(schedule.value_at(&25), Some(&"high"));
/// assert_eq!(schedule.value_at(&30), None);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StepSchedule<T, V> {
    steps: Vec<Step<T, V>>,
}

/// Error returned when attempting to add a [`Step`] that overlaps an existing one.
///
/// This error can occur when creating a [`StepSchedule`] with overlapping steps,
/// or when pushing a new step into an existing schedule.
/// It contains the ranges of the two conflicting steps.
#[derive(Debug, Error)]
#[error("steps overlap: {first:?} and {second:?}")]
pub struct OverlappingStepsError<T: Debug + Ord> {
    /// The range of the existing step.
    pub first: Range<T>,
    /// The range of the conflicting step.
    pub second: Range<T>,
}

impl<T: Debug + Clone + Ord, V> StepSchedule<T, V> {
    /// Creates a new `StepSchedule` from an iterator over [`Step`]s.
    ///
    /// Accepts any type convertible into an iterator, such as a vector or array.
    /// The resulting schedule is ordered by increasing start time.
    ///
    /// # Errors
    ///
    /// Returns an [`OverlappingStepsError`] if any steps have overlapping ranges.
    pub fn new<I>(steps: I) -> Result<Self, OverlappingStepsError<T>>
    where
        I: IntoIterator<Item = Step<T, V>>,
    {
        let mut steps: Vec<_> = steps.into_iter().collect();
        steps.sort_by(|a, b| a.start().cmp(b.start()));

        if let Some(index) = steps.windows(2).position(|pair| pair[0].overlaps(&pair[1])) {
            let first = steps[index].range().clone();
            let second = steps[index + 1].range().clone();
            return Err(OverlappingStepsError { first, second });
        }

        Ok(StepSchedule { steps })
    }

    /// Attempts to add a step to the schedule.
    ///
    /// # Errors
    ///
    /// Returns an [`OverlappingStepsError`] if the new step's range overlaps
    /// with any existing steps in the schedule.
    pub fn try_push(&mut self, step: Step<T, V>) -> Result<(), OverlappingStepsError<T>> {
        let index = self.steps.partition_point(|s| s.start() < step.start());

        if index > 0 && self.steps[index - 1].overlaps(&step) {
            return Err(OverlappingStepsError {
                first: self.steps[index - 1].range().clone(),
                second: step.range().clone(),
            });
        }

        if index < self.steps.len() && self.steps[index].overlaps(&step) {
            return Err(OverlappingStepsError {
                first: self.steps[index].range().clone(),
                second: step.range().clone(),
            });
        }

        self.steps.insert(index, step);
        Ok(())
    }

    /// Returns a slice of all steps in this schedule, in order of increasing start time.
    #[must_use]
    pub fn steps(&self) -> &[Step<T, V>] {
        &self.steps
    }

    /// Returns the value associated with the given `time`, if any.
    ///
    /// Searches for the [`Step`] whose range contains `time` and returns a
    /// reference to its associated value.
    /// Returns `None` if no such step exists.
    ///
    /// For schedules with fewer than [`LINEAR_SEARCH_THRESHOLD`] steps,
    /// a linear scan is used.
    /// Otherwise, binary search is performed.
    ///
    /// # Examples
    ///
    /// ```
    /// use twine_components::schedule::step_schedule::{Step, StepSchedule};
    ///
    /// let schedule = StepSchedule::new(vec![
    ///     Step::new(0..10, 1).unwrap(),
    ///     Step::new(10..20, 2).unwrap(),
    /// ]).unwrap();
    ///
    /// assert_eq!(schedule.value_at(&5), Some(&1));
    /// assert_eq!(schedule.value_at(&15), Some(&2));
    /// assert_eq!(schedule.value_at(&20), None);
    /// ```
    pub fn value_at(&self, time: &T) -> Option<&V> {
        if self.steps.len() < LINEAR_SEARCH_THRESHOLD {
            self.steps.iter().find_map(|step| step.value_at(time))
        } else {
            self.steps
                .binary_search_by(|step| step.cmp_to_time(time))
                .ok()
                .map(|index| self.steps[index].value())
        }
    }
}

impl<T: Debug + Clone + Ord, V: Clone> Component for StepSchedule<T, V> {
    type Input = T;
    type Output = Option<V>;
    type Error = Infallible;

    /// Invokes the schedule with the given input `time`, returning the associated value if any.
    ///
    /// Returns `Ok(Some(value))` if a step covers the input, or `Ok(None)` if no step matches.
    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self.value_at(&input).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter;

    #[test]
    fn new_succeeds_with_non_overlapping_steps() {
        let schedule = StepSchedule::new([
            Step::new(10..20, "a").unwrap(),
            Step::new(20..30, "b").unwrap(),
        ])
        .unwrap();

        assert_eq!(schedule.steps().len(), 2);
        assert_eq!(schedule.value_at(&15), Some(&"a"));
        assert_eq!(schedule.value_at(&25), Some(&"b"));
    }

    #[test]
    fn new_rejects_overlapping_steps() {
        let result = StepSchedule::new([
            Step::new(0..10, "a").unwrap(),
            Step::new(9..15, "b").unwrap(),
        ]);

        assert!(result.is_err());
    }

    #[test]
    fn try_push_adds_step_successfully() {
        let mut schedule = StepSchedule::new([
            Step::new(-10..0, "a").unwrap(),
            Step::new(0..10, "b").unwrap(),
        ])
        .unwrap();

        let new_step = Step::new(10..20, "c").unwrap();

        assert!(schedule.try_push(new_step).is_ok());
        assert_eq!(schedule.steps().len(), 3);
        assert_eq!(schedule.value_at(&15), Some(&"c"));
    }

    #[test]
    fn steps_are_sorted_by_start_time() {
        let schedule = StepSchedule::new([
            Step::new(20..30, "later").unwrap(),
            Step::new(0..10, "early").unwrap(),
        ])
        .unwrap();

        let steps = schedule.steps();
        assert_eq!(steps[0].start(), &0);
        assert_eq!(steps[1].start(), &20);
    }

    #[test]
    fn value_at_handles_empty_schedule() {
        let schedule: StepSchedule<i32, &str> = StepSchedule::default();
        assert_eq!(schedule.value_at(&5), None);
    }

    #[test]
    fn value_at_handles_exact_start_and_end_bounds() {
        let step = Step::new(5..10, "only").unwrap();
        let schedule = StepSchedule::new(iter::once(step)).unwrap();

        assert_eq!(schedule.value_at(&4), None);
        assert_eq!(schedule.value_at(&5), Some(&"only"));
        assert_eq!(schedule.value_at(&9), Some(&"only"));
        assert_eq!(schedule.value_at(&10), None);
    }

    #[test]
    fn value_at_works_for_large_schedules() {
        let steps = (0..100).map(|i| Step::new(i * 10..(i + 1) * 10, i).unwrap());
        let schedule = StepSchedule::new(steps).unwrap();

        assert_eq!(schedule.value_at(&0), Some(&0));
        assert_eq!(schedule.value_at(&42), Some(&4));
        assert_eq!(schedule.value_at(&999), Some(&99));
        assert_eq!(schedule.value_at(&1000), None);
    }
}
