#[derive(Debug, Clone, Copy, Default)]
pub(super) struct Adjacent<T> {
    pub(super) bottom: T,
    pub(super) side: T,
    pub(super) top: T,
}

impl<T: Copy> Adjacent<T> {
    pub(super) fn from_value(value: T) -> Self {
        Self {
            bottom: value,
            side: value,
            top: value,
        }
    }

    pub(super) fn with_bottom(self, bottom: T) -> Self {
        Self { bottom, ..self }
    }

    pub(super) fn with_side(self, side: T) -> Self {
        Self { side, ..self }
    }

    pub(super) fn with_top(self, top: T) -> Self {
        Self { top, ..self }
    }
}
