use uom::si::f64::Time;

pub trait HasTime: Sized {
    fn get_time(&self) -> Time;
    fn with_time(self, time: Time) -> Self;
}
