use std::{array, ops::RangeInclusive};
use uom::si::{
    f64::{ThermodynamicTemperature, Volume},
    thermodynamic_temperature::kelvin,
    volume::cubic_meter,
};

/// Restores thermal stability in a stack of layers by mixing unstable adjacents.
///
/// Unstable pairs (warmer below cooler) are recursively merged until the
/// resulting profile is monotonically increasing from bottom to top.
///
/// Returns a new array of temperatures with the same shape as the input.
///
/// This operation **conserves thermal energy** under the assumption of:
/// - Constant fluid density
/// - Constant specific heat capacity
///
/// The model does not simulate physical mixing over time, only instantaneous
/// buoyancy-driven correction at a single point in time.
fn apply_buoyancy<const N: usize>(layers: [Layer; N]) -> [ThermodynamicTemperature; N] {
    let mut stack: [Block; N] = array::from_fn(|_| Block::default());
    let mut stack_index = 0;

    for (i, layer) in layers.into_iter().enumerate() {
        let mut block = Block::new(i, layer);

        // Merge downward until the block below is not warmer.
        while stack_index > 0 && stack[stack_index - 1].temp > block.temp {
            block = block.merge_with_below(&stack[stack_index - 1]);
            stack_index -= 1;
        }

        // Add this block to the stack.
        stack[stack_index] = block;
        stack_index += 1;
    }

    // Build the resulting temperature array.
    let mut temperatures = [ThermodynamicTemperature::default(); N];
    for block in &stack[..stack_index] {
        for i in block.range.clone() {
            temperatures[i] = block.temperature();
        }
    }

    temperatures
}

/// A contiguous group of fully mixed layers.
///
/// Used as a temporary data structure during stabilization.
#[derive(Debug, Clone)]
struct Block {
    range: RangeInclusive<usize>,
    temp: f64,
    vol: f64,
}

impl Block {
    fn new(index: usize, layer: Layer) -> Self {
        Self {
            range: index..=index,
            temp: layer.temperature.get::<kelvin>(),
            vol: layer.volume.get::<cubic_meter>(),
        }
    }

    fn temperature(&self) -> ThermodynamicTemperature {
        ThermodynamicTemperature::new::<kelvin>(self.temp)
    }

    /// Merges `self` with the block below it, returning a new block.
    ///
    /// Temperature is updated using a volume-weighted average.
    fn merge_with_below(self, below: &Block) -> Self {
        let total_vol = self.vol + below.vol;
        let mixed_temp = (self.temp * self.vol + below.temp * below.vol) / total_vol;

        Self {
            range: *below.range.start()..=*self.range.end(),
            temp: mixed_temp,
            vol: total_vol,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            range: 0..=0,
            temp: 0.0,
            vol: 0.0,
        }
    }
}

/// A single thermal layer.
#[derive(Debug, Clone, Copy, Default)]
struct Layer {
    temperature: ThermodynamicTemperature,
    volume: Volume,
}

impl Layer {
    fn new(temperature: ThermodynamicTemperature, volume: Volume) -> Self {
        Self {
            temperature,
            volume,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::thermodynamic_temperature::degree_celsius;

    fn uniform_layers<const N: usize>(temps: [f64; N]) -> [Layer; N] {
        temps.map(|temp| {
            Layer::new(
                ThermodynamicTemperature::new::<degree_celsius>(temp),
                Volume::new::<cubic_meter>(1.0),
            )
        })
    }

    fn temperatures<const N: usize>(temps: [f64; N]) -> [ThermodynamicTemperature; N] {
        temps.map(ThermodynamicTemperature::new::<degree_celsius>)
    }

    #[test]
    fn no_mixing_needed() {
        let layers = uniform_layers([30.0, 40.0, 50.0]);
        assert_eq!(apply_buoyancy(layers), temperatures([30.0, 40.0, 50.0]));
    }

    #[test]
    fn all_mixed() {
        let layers = uniform_layers([50.0, 40.0, 30.0]);
        assert_eq!(apply_buoyancy(layers), temperatures([40.0, 40.0, 40.0]));
    }

    #[test]
    fn some_mixing() {
        let layers = uniform_layers([20.0, 30.0, 50.0, 40.0, 42.0]);
        assert_eq!(
            apply_buoyancy(layers),
            temperatures([20.0, 30.0, 44.0, 44.0, 44.0]),
        );
    }

    #[test]
    fn some_mixing_uneven_volumes() {
        let layers = [
            Layer::new(
                ThermodynamicTemperature::new::<degree_celsius>(2.0),
                Volume::new::<cubic_meter>(1.0),
            ),
            Layer::new(
                ThermodynamicTemperature::new::<degree_celsius>(10.0),
                Volume::new::<cubic_meter>(4.0),
            ),
            Layer::new(
                ThermodynamicTemperature::new::<degree_celsius>(5.0),
                Volume::new::<cubic_meter>(1.0),
            ),
        ];
        assert_eq!(apply_buoyancy(layers), temperatures([2.0, 9.0, 9.0]));
    }
}
