use std::{cmp::Ordering, convert::Infallible};

use jiff::civil::Time;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use twine_core::Component;

const LINEAR_SEARCH_THRESHOLD: usize = 32;

#[derive(Debug, Clone)]
pub struct StepSchedule<T> {
    segments: Vec<Segment<T>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment<T> {
    pub start: Time,
    pub end: Time,
    pub value: T,
}

#[derive(Debug, Error)]
pub enum StepScheduleError {
    #[error("Segment {index} is invalid: start ({start}) >= end ({end})")]
    InvalidSegment {
        index: usize,
        start: Time,
        end: Time,
    },

    #[error(
        "Segment {prev_index} (end: {prev_end}) overlaps with segment {curr_index} (start: {curr_start})"
    )]
    OverlappingSegments {
        prev_index: usize,
        curr_index: usize,
        prev_end: Time,
        curr_start: Time,
    },
}

impl<T> StepSchedule<T> {
    pub fn new(mut segments: Vec<Segment<T>>) -> Result<Self, StepScheduleError> {
        // Validate each segment.
        for (index, segment) in segments.iter().enumerate() {
            if segment.start >= segment.end {
                return Err(StepScheduleError::InvalidSegment {
                    index,
                    start: segment.start,
                    end: segment.end,
                });
            }
        }

        // Sort the segments.
        segments.sort_by_key(|seg| seg.start);

        // Check for overlapping segments.
        for i in 1..segments.len() {
            let prev = &segments[i - 1];
            let curr = &segments[i];
            if prev.end > curr.start {
                return Err(StepScheduleError::OverlappingSegments {
                    prev_index: i - 1,
                    curr_index: i,
                    prev_end: prev.end,
                    curr_start: curr.start,
                });
            }
        }

        Ok(StepSchedule { segments })
    }

    /// Returns a reference to the value at the given time, or None.
    #[must_use]
    pub fn value_at(&self, time: Time) -> Option<&T> {
        if self.segments.len() <= LINEAR_SEARCH_THRESHOLD {
            self.segments
                .iter()
                .find(|segment| segment.start <= time && time < segment.end)
                .map(|segment| &segment.value)
        } else {
            self.segments
                .binary_search_by(|segment| {
                    if time < segment.start {
                        Ordering::Greater
                    } else if time >= segment.end {
                        Ordering::Less
                    } else {
                        Ordering::Equal
                    }
                })
                .ok()
                .map(|index| &self.segments[index].value)
        }
    }
}

impl<T: Clone> Component for StepSchedule<T> {
    type Input = Time;
    type Output = Option<T>;
    type Error = Infallible;

    fn call(&self, input: Self::Input) -> Result<Self::Output, Self::Error> {
        Ok(self.value_at(input).cloned())
    }
}
