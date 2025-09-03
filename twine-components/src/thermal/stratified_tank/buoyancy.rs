use std::{array, ops::RangeInclusive};
use uom::si::{
    f64::{Mass, MassDensity, ThermodynamicTemperature, Volume},
    temperature_interval, thermodynamic_temperature,
};

use super::DensityModel;

/// Restores thermal stability in a stack of fluid layers through buoyancy mixing.
///
/// This function merges any unstable adjacent layers so that denser fluid is
/// always below lighter fluid.
/// The temperature of a merged layer is computed using a mass-weighted average.
///
/// This process represents an instantaneous stabilization step;
/// it does not simulate transient mixing dynamics.
pub(super) fn stabilize<const N: usize>(
    temp: &[ThermodynamicTemperature; N],
    vol: &[Volume; N],
    dens_model: &impl DensityModel,
) -> [(ThermodynamicTemperature, Mass); N] {
    let dens = temp.map(|t| dens_model.density(t));

    // Fast path: already stable if every adjacent pair has ρ_below ≥ ρ_above.
    let already_stable = dens.windows(2).all(|w| w[0] >= w[1]);
    if already_stable {
        return array::from_fn(|i| (temp[i], dens[i] * vol[i]));
    }

    // Build a stack of blocks from bottom to top, recursively merging unstable pairs.
    let mut stack: [Block; N] = array::from_fn(|_| Block::default());
    let mut stack_index = 0;
    for i in 0..N {
        let mut block = Block::new(i, temp[i], dens[i], vol[i]);

        // Merge downward while the block below is less dense than this one.
        while stack_index > 0 && should_merge(&block, &stack[stack_index - 1]) {
            block = merge(&block, &stack[stack_index - 1]);
            stack_index -= 1;
        }

        stack[stack_index] = block;
        stack_index += 1;
    }

    let mut layers = [Default::default(); N];
    for block in &stack[..stack_index] {
        let block_temp = block.temp;
        let block_dens = dens_model.density(block_temp);

        for i in block.range.clone() {
            layers[i] = (block_temp, block_dens * vol[i]);
        }
    }

    layers
}

/// A block of adjacent layers with uniform temperature and total mass/volume.
#[derive(Debug, Clone)]
struct Block {
    range: RangeInclusive<usize>,
    temp: ThermodynamicTemperature,
    vol: Volume,
    mass: Mass,
}

/// Returns `true` if the block pair (below, above) is unstable.
///
/// A block pair is unstable if:
///
/// ```text
///   ρ_below < ρ_above
/// ⇔ m_below / v_below < m_above / v_above
/// ⇔ m_below * v_above < m_above * v_below
/// ```
///
/// The cross-multiply avoids divisions and is numerically friendly.
fn should_merge(above: &Block, below: &Block) -> bool {
    (below.mass * above.vol) < (above.mass * below.vol)
}

/// Merges the block pair (below, above) into a new `Block`.
///
/// Volume and mass are conserved.
/// Temperature is updated using a mass-weighted average,
/// which conserves energy under the constant specific heat assumption.
///
/// Note that the `(m*T)/m` arithmetic yields a `TemperatureInterval`,
/// but it's safe to rewrap into absolute temperature.
fn merge(above: &Block, below: &Block) -> Block {
    let total_mass = above.mass + below.mass;
    let total_vol = above.vol + below.vol;

    let t_mix = (above.mass * above.temp + below.mass * below.temp) / total_mass;
    let t_k = t_mix.get::<temperature_interval::kelvin>();
    let mixed_temp = ThermodynamicTemperature::new::<thermodynamic_temperature::kelvin>(t_k);

    Block {
        range: *below.range.start()..=*above.range.end(),
        temp: mixed_temp,
        vol: total_vol,
        mass: total_mass,
    }
}

impl Block {
    fn new(index: usize, temp: ThermodynamicTemperature, dens: MassDensity, vol: Volume) -> Self {
        Self {
            range: index..=index,
            temp,
            vol,
            mass: vol * dens,
        }
    }
}

impl Default for Block {
    fn default() -> Self {
        Self {
            range: 0..=0,
            temp: ThermodynamicTemperature::default(),
            vol: Volume::default(),
            mass: Mass::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{
        f64::MassDensity, mass::kilogram, mass_density::kilogram_per_cubic_meter,
        thermodynamic_temperature::degree_celsius, volume::cubic_meter,
    };

    /// An arbitrary density model that ensures cold is slightly denser than warm.
    struct NearlyConstant;
    impl DensityModel for NearlyConstant {
        fn density(&self, temp: ThermodynamicTemperature) -> MassDensity {
            let t = temp.get::<degree_celsius>();
            let d = 1000.0 - t * 1e-12;
            MassDensity::new::<kilogram_per_cubic_meter>(d)
        }
    }

    fn ts<const N: usize>(temps_c: [f64; N]) -> [ThermodynamicTemperature; N] {
        temps_c.map(ThermodynamicTemperature::new::<degree_celsius>)
    }

    fn vs<const N: usize>(vols_m3: [f64; N]) -> [Volume; N] {
        vols_m3.map(Volume::new::<cubic_meter>)
    }

    fn layer_temp(layers: &[(ThermodynamicTemperature, Mass)], index: usize) -> f64 {
        layers[index].0.get::<degree_celsius>()
    }

    fn layer_mass(layers: &[(ThermodynamicTemperature, Mass)], index: usize) -> f64 {
        layers[index].1.get::<kilogram>()
    }

    #[test]
    fn no_mixing_needed() {
        let vol = vs([1.0; 3]);
        let temp = ts([30.0, 40.0, 50.0]);

        let layers = stabilize(&temp, &vol, &NearlyConstant);
        assert_eq!(layers.map(|l| l.0), ts([30.0, 40.0, 50.0]));
    }

    #[test]
    fn all_mixed() {
        let vol = vs([1.0; 3]);
        let temp = ts([50.0, 40.0, 30.0]);

        let layers = stabilize(&temp, &vol, &NearlyConstant);

        assert_relative_eq!(layer_temp(&layers, 0), 40.0, epsilon = 1e-12);
        assert_relative_eq!(layer_temp(&layers, 1), 40.0, epsilon = 1e-12);
        assert_relative_eq!(layer_temp(&layers, 2), 40.0, epsilon = 1e-12);

        assert_relative_eq!(layer_mass(&layers, 0), 1000.0, epsilon = 1e-10);
        assert_relative_eq!(layer_mass(&layers, 1), 1000.0, epsilon = 1e-10);
        assert_relative_eq!(layer_mass(&layers, 2), 1000.0, epsilon = 1e-10);
    }

    #[test]
    fn some_mixing() {
        let vol = vs([1.0; 5]);
        let temp = ts([20.0, 30.0, 50.0, 40.0, 42.0]);

        let layers = stabilize(&temp, &vol, &NearlyConstant);

        assert_relative_eq!(layer_temp(&layers, 0), 20.0, epsilon = 1e-12);
        assert_relative_eq!(layer_temp(&layers, 1), 30.0, epsilon = 1e-12);
        assert_relative_eq!(layer_temp(&layers, 2), 44.0, epsilon = 1e-12);
        assert_relative_eq!(layer_temp(&layers, 3), 44.0, epsilon = 1e-12);
        assert_relative_eq!(layer_temp(&layers, 4), 44.0, epsilon = 1e-12);
    }

    #[test]
    fn some_mixing_uneven_volumes() {
        let vol = vs([1.0, 4.0, 2.0]);
        let temp = ts([2.0, 10.0, 4.0]);

        let layers = stabilize(&temp, &vol, &NearlyConstant);

        assert_relative_eq!(layer_temp(&layers, 0), 2.0, epsilon = 1e-12);
        assert_relative_eq!(layer_temp(&layers, 1), 8.0, epsilon = 1e-12);
        assert_relative_eq!(layer_temp(&layers, 2), 8.0, epsilon = 1e-12);

        assert_relative_eq!(layer_mass(&layers, 0), 1000.0, epsilon = 1e-10);
        assert_relative_eq!(layer_mass(&layers, 1), 4000.0, epsilon = 1e-10);
        assert_relative_eq!(layer_mass(&layers, 2), 2000.0, epsilon = 1e-10);
    }
}
