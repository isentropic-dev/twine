/// Values at the bottom, side, and top faces.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(super) struct Adjacent<T> {
    pub(super) bottom: T,
    pub(super) side: T,
    pub(super) top: T,
}

impl<T: Copy> Adjacent<T> {
    /// Constructs an `Adjacent` with the same value for all faces.
    #[allow(dead_code)]
    pub(super) fn from_value(value: T) -> Self {
        Self {
            bottom: value,
            side: value,
            top: value,
        }
    }
}
