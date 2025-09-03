use twine_core::constraint::{Constrained, UnitInterval};

#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum Location {
    HeightFraction(Constrained<f64, UnitInterval>),
}

#[derive(Debug, Clone, Copy)]
pub struct PortPairLocation {
    pub inlet: Location,
    pub outlet: Location,
}
