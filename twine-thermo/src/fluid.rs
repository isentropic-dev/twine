mod air;
mod carbon_dioxide;
mod custom;
mod water;

pub use air::Air;
pub use carbon_dioxide::CarbonDioxide;
pub use custom::{IdealGasCustom, IncompressibleCustom};
pub use water::Water;

/// A marker trait for stateless fluid types.
///
/// A fluid type is `Stateless` if its identity is determined entirely by its
/// type and it carries no runtime-specific configuration or state.
/// This pattern typically applies to canonical fluids like [`Air`] or [`Water`],
/// which are zero-sized and fully described by their type.
///
/// Implement this trait for any fluid type with a fixed identity and no
/// per-instance variation.
/// Doing so enables improved ergonomics and simplifies logic when working
/// with [`State<Fluid>`] or other fluid-specific abstractions.
///
/// # Example
///
/// ```
/// use twine_thermo::fluid::Stateless;
///
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// struct MyFluid;
///
/// impl Stateless for MyFluid {}
/// ```
pub trait Stateless: std::fmt::Debug + Clone + Copy + PartialEq + Eq + Default {}
