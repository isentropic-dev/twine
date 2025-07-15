mod air;
mod carbon_dioxide;
mod custom;
mod water;

pub use air::Air;
pub use carbon_dioxide::CarbonDioxide;
pub use custom::{IdealGasCustom, IncompressibleCustom};
pub use water::Water;

/// Marker trait for zero-sized fluid types.
///
/// A `MarkerFluid` represents a canonical fluid type with no runtime data,
/// such as [`Air`] or [`Water`].
///
/// Implement this trait for any zero-sized `Fluid` type to enable improved
/// ergonomics and simplified logic when working with [`State<Fluid>`] or
/// other fluid-specific types.
///
/// # Example
///
/// ```
/// use twine_thermo::fluid::MarkerFluid;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// struct MyFluid;
///
/// impl MarkerFluid for MyFluid {}
/// ```
pub trait MarkerFluid: std::fmt::Debug + Clone + Copy + PartialEq + Eq + Default {}
