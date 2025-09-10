use std::{array, ops::RangeInclusive};
use uom::si::{
    f64::{ThermodynamicTemperature, Volume},
    temperature_interval, thermodynamic_temperature,
};

/// Restores thermal stability in a stack of fluid nodes through buoyancy mixing.
///
/// This function merges any unstable adjacent nodes so that colder fluid is
/// always below warmer fluid.
/// The temperature of a merged nodes is computed using a mass-weighted average,
/// which for an incompressible fluid is equivalent to a volume-weighted average.
///
/// This process represents an instantaneous stabilization step;
/// it does not simulate transient mixing dynamics.
pub(super) fn stabilize<const N: usize>(
    temp: &mut [ThermodynamicTemperature; N],
    vol: &[Volume; N],
) {
    // Fast path: already stable if every adjacent pair has T_below ≤ T_above.
    if temp.windows(2).all(|w| w[0] <= w[1]) {
        return;
    }

    // Build a stack of blocks from bottom to top, recursively merging unstable pairs.
    let mut stack: [Block; N] = array::from_fn(|_| Block::default());
    let mut stack_index = 0;
    for i in 0..N {
        let mut block = Block::new(i, temp[i], vol[i]);

        // Merge downward while the block below is less dense than this one.
        while stack_index > 0 && should_merge(&block, &stack[stack_index - 1]) {
            block = merge(&block, &stack[stack_index - 1]);
            stack_index -= 1;
        }

        stack[stack_index] = block;
        stack_index += 1;
    }

    for block in &stack[..stack_index] {
        for i in block.range.clone() {
            temp[i] = block.temp;
        }
    }
}

/// A block of adjacent nodes with uniform temperature and total volume.
#[derive(Debug, Clone)]
struct Block {
    range: RangeInclusive<usize>,
    temp: ThermodynamicTemperature,
    vol: Volume,
}

/// Returns `true` if the block pair (below, above) is unstable.
///
/// A block pair is unstable if `T_below > T_above`.
fn should_merge(above: &Block, below: &Block) -> bool {
    below.temp > above.temp
}

/// Merges the block pair (below, above) into a new `Block`.
///
/// Note that the `(V*T)/V` arithmetic yields a `TemperatureInterval`,
/// but it's safe to rewrap into absolute temperature.
fn merge(above: &Block, below: &Block) -> Block {
    let total_vol = above.vol + below.vol;

    let t_mix = (above.vol * above.temp + below.vol * below.temp) / total_vol;
    let t_k = t_mix.get::<temperature_interval::kelvin>();
    let mixed_temp = ThermodynamicTemperature::new::<thermodynamic_temperature::kelvin>(t_k);

    Block {
        range: *below.range.start()..=*above.range.end(),
        temp: mixed_temp,
        vol: total_vol,
    }
}

impl Block {
    fn new(index: usize, temp: ThermodynamicTemperature, vol: Volume) -> Self {
        Self {
            range: index..=index,
            temp,
            vol,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            range: 0..=0,
            temp: ThermodynamicTemperature::default(),
            vol: Volume::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use uom::si::{thermodynamic_temperature::degree_celsius, volume::cubic_meter};

    fn ts<const N: usize>(temps_c: [f64; N]) -> [ThermodynamicTemperature; N] {
        temps_c.map(ThermodynamicTemperature::new::<degree_celsius>)
    }

    fn vs<const N: usize>(vols_m3: [f64; N]) -> [Volume; N] {
        vols_m3.map(Volume::new::<cubic_meter>)
    }

    #[test]
    fn no_mixing_needed() {
        let vol = vs([1.0; 3]);
        let mut temp = ts([30.0, 40.0, 50.0]);

        stabilize(&mut temp, &vol);
        assert_eq!(temp, ts([30.0, 40.0, 50.0]));
    }

    #[test]
    fn all_mixed() {
        let vol = vs([1.0; 3]);
        let mut temp = ts([50.0, 40.0, 30.0]);

        stabilize(&mut temp, &vol);
        assert_eq!(temp, ts([40.0, 40.0, 40.0]));
    }

    #[test]
    fn some_mixing() {
        let vol = vs([1.0; 5]);
        let mut temp = ts([20.0, 30.0, 50.0, 40.0, 42.0]);

        stabilize(&mut temp, &vol);
        assert_eq!(temp, ts([20.0, 30.0, 44.0, 44.0, 44.0]));
    }

    #[test]
    fn some_mixing_uneven_volumes() {
        let vol = vs([1.0, 4.0, 2.0]);
        let mut temp = ts([2.0, 10.0, 4.0]);

        stabilize(&mut temp, &vol);
        assert_eq!(temp, ts([2.0, 8.0, 8.0]));
    }
}
