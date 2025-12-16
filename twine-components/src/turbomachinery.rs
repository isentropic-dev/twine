//! Turbomachinery component models.
//!
//! This module contains component-level models for devices like compressors and turbines.
//! Outputs are designed to be unambiguous for systems modeling. For example,
//! turbomachinery work is reported as a non-negative work type (e.g., [`CompressionWork`]),
//! rather than a signed value with a convention-dependent meaning.

pub mod compressor;
pub mod turbine;
mod work;

pub use work::{CompressionWork, ExpansionWork};

#[cfg(test)]
pub(crate) mod test_utils;
