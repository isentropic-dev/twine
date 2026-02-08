/// Events emitted during golden section search.
#[derive(Debug)]
pub enum Event<'a, M, P> {
    // TODO: Define events (e.g., Iteration with current bounds and evaluations)
    _Phantom(std::marker::PhantomData<(&'a M, &'a P)>),
}
