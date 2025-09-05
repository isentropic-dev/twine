/// Options for specifying tank insulation.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Insulation {
    /// Tank is perfectly insulated with no heat transfer to its environment.
    Adiabatic,
}
