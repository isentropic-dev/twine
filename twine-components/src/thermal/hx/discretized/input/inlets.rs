use twine_thermo::State;

/// Inlet thermodynamic states for the top and bottom streams.
///
/// The "top" and "bottom" labels refer to the physical stream assignment,
/// not necessarily the hot/cold side of the heat exchanger.
#[derive(Debug, Clone)]
pub struct Inlets<TopFluid, BottomFluid> {
    /// Inlet state of the top stream.
    pub top: State<TopFluid>,

    /// Inlet state of the bottom stream.
    pub bottom: State<BottomFluid>,
}
