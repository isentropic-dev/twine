pub trait StateIntegrator<C>
where
    C: StatefulComponent,
    C::Input: Clone + HasTime,
{
    fn step(
        &self,
        component: &C,
        input0: C::Input,
        output0: C::Output,
        dt: Time,
    ) -> Result<(C::Input, C::Output), C::Error>;
}
