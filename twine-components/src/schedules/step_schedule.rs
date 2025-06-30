use std::{cmp::Ordering, convert::Infallible, fmt::Display};

use jiff::civil::Time;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use twine_core::Component;

const LINEAR_SEARCH_THRESHOLD: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StepSchedule<T, V> {
    steps: Vec<Step<T, V>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Step<T, V> {
    pub start: T,
    pub end: T,
    pub value: V,
}

#[derive(Debug, PartialEq, Error)]
pub enum StepScheduleError<T, V>
where
    T: Display,
{
    #[error("Step is invalid: start ({}) >= end ({})", step.start, step.end)]
    InvalidStep { step: Step<T, V> },

    #[error("Steps overlap: [{}, {}) and [{}, {})", first.start, first.end, second.start, second.end)]
    OverlappingSteps {
        first: Step<T, V>,
        second: Step<T, V>,
    },
}

struct WeeklyStepSchedule<V> {
    monday: StepSchedule<Time, V>,
}

impl<T, V> StepSchedule<T, V>
where
    T: Clone + Ord + Display,
{
    #[allow(clippy::missing_errors_doc)]
    pub fn new(mut steps: Vec<Step<T, V>>) -> Result<Self, StepScheduleError<T, V>> {
        if let Some(index) = steps.iter().position(|step| step.start >= step.end) {
            let bad_step = steps.remove(index);
            return Err(StepScheduleError::InvalidStep { step: bad_step });
        }

        // TODO: See if I can use a reference instead so we don't need T: Clone
        steps.sort_by_key(|step| step.start.clone());

        if let Some(index) = steps
            .windows(2)
            .position(|pair| pair[0].end > pair[1].start)
        {
            let second = steps.remove(index + 1);
            let first = steps.remove(index);
            return Err(StepScheduleError::OverlappingSteps { first, second });
        }

        Ok(StepSchedule { steps })
    }

    /// Returns a reference to the value at the given time, or None.
    #[must_use]
    pub fn value_at(&self, time: &T) -> Option<&V> {
        if self.steps.len() <= LINEAR_SEARCH_THRESHOLD {
            self.steps
                .iter()
                .find(|step| step.start <= *time && *time < step.end)
                .map(|step| &step.value)
        } else {
            self.steps
                .binary_search_by(|step| {
                    if *time < step.start {
                        Ordering::Greater
                    } else if *time >= step.end {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    }
                })
                .ok()
                .map(|index| &self.steps[index].value)
        }
    }
}

impl<T, V> Step<T, V> {
    pub fn new(start: T, end: T, value: V) -> Self {
        Self { start, end, value }
    }
}

impl<T, V> Component for StepSchedule<T, V>
where
    T: Clone + Ord + Display,
    V: Clone,
{
    type Input = T;
    type Output = Option<V>;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self.value_at(&input).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use jiff::{ToSpan, civil::Time};

    fn t(hour: i8, minute: i8) -> Time {
        Time::constant(hour, minute, 0, 0)
    }

    #[test]
    fn valid_schedule_is_ok() {
        let schedule = StepSchedule::new(vec![
            Step::new(8, 12, "morning"),
            Step::new(12, 18, "afternoon"),
        ]);

        assert!(schedule.is_ok());
    }

    #[test]
    fn error_on_invalid_step() {
        let schedule = StepSchedule::new(vec![
            Step::new(5, 7, "ok"),
            Step::new(9, 10, "ok"),
            Step::new(8, 8, "bad"),
        ]);

        if let Err(StepScheduleError::InvalidStep { step }) = schedule {
            assert_eq!(step.value, "bad");
        } else {
            panic!("Expected InvalidStep error")
        }
    }

    #[test]
    fn error_on_overlapping_steps() {
        let schedule = StepSchedule::new(vec![
            Step::new(t(7, 0), t(8, 0), 1),
            Step::new(t(8, 0), t(10, 0), 2),
            Step::new(t(9, 30), t(11, 0), 3),
        ]);

        if let Err(StepScheduleError::OverlappingSteps { first, second }) = schedule {
            assert_eq!(first.start, t(8, 0));
            assert_eq!(second.start, t(9, 30));
        } else {
            panic!("Expected OverlappingSteps error")
        }
    }

    #[test]
    fn value_at_returns_none_for_empty_schedule() {
        let schedule: StepSchedule<i32, i32> = StepSchedule::new(vec![]).unwrap();

        assert!(schedule.value_at(&0).is_none());
    }

    #[test]
    fn value_at_finds_correct_step() {
        let schedule = StepSchedule::new(vec![
            Step::new(t(8, 0), t(10, 0), "early"),
            Step::new(t(10, 0), t(12, 0), "late"),
        ])
        .unwrap();

        assert_eq!(schedule.value_at(&t(7, 0)), None);
        assert_eq!(schedule.value_at(&t(8, 0)), Some(&"early"));
        assert_eq!(schedule.value_at(&t(9, 59)), Some(&"early"));
        assert_eq!(schedule.value_at(&t(10, 0)), Some(&"late"));
        assert_eq!(schedule.value_at(&t(11, 59)), Some(&"late"));
        assert_eq!(schedule.value_at(&t(12, 0)), None);
        assert_eq!(schedule.value_at(&t(14, 0)), None);
    }

    #[test]
    fn value_at_works_with_many_steps() {
        let steps: Vec<_> = (0..100)
            .map(|i| {
                let start = Time::midnight() + (i * 10).minutes();
                let end = start + 5.minutes();
                Step {
                    start,
                    end,
                    value: i,
                }
            })
            .collect();

        let schedule = StepSchedule::new(steps).unwrap();

        assert_eq!(schedule.value_at(&t(0, 0)), Some(&0));
        assert_eq!(
            schedule.value_at(&Time::constant(0, 4, 59, 999_999_999)),
            Some(&0)
        );
        assert_eq!(schedule.value_at(&t(0, 5)), None);

        assert_eq!(schedule.value_at(&t(0, 0)), Some(&0));
        assert_eq!(schedule.value_at(&t(0, 5)), None);
    }
}
