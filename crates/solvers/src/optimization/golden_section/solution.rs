use std::marker::PhantomData;

/// The result of a golden section search.
#[derive(Debug, Clone)]
pub struct Solution<I, O> {
    _marker: PhantomData<(I, O)>,
}
