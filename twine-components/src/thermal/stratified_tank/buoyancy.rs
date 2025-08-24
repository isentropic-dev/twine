use std::{array, ops::RangeInclusive};
use uom::si::{
    f64::{Mass, MassDensity, ThermodynamicTemperature, Volume},
    temperature_interval, thermodynamic_temperature,
};

/// Restores thermal stability in a stack of fluid layers through buoyancy mixing.
///
/// This function merges any unstable adjacent layers so that denser fluid is
/// always below lighter fluid.
///
/// Within each merged block:
/// - temperature becomes uniform (mass-weighted average),
/// - total mass and volume are conserved, and
/// - the block's mass is redistributed back to the original layers in
///   proportion to their volumes (volumes themselves are unchanged).
///
/// This process represents an instantaneous stabilization step;
/// it does not simulate transient mixing dynamics.
///
/// Returns the stabilized temperatures and per-layer masses.
pub(super) fn stabilize<const N: usize>(
    mut temp: [ThermodynamicTemperature; N],
    dens: &[MassDensity; N],
    vol: &[Volume; N],
) -> ([ThermodynamicTemperature; N], [Mass; N]) {
    // Fast path: already stable if every adjacent pair has ρ_below ≥ ρ_above.
    let already_stable = dens.windows(2).all(|w| w[0] >= w[1]);
    if already_stable {
        return (temp, array::from_fn(|i| dens[i] * vol[i]));
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

    // Assign each layer its block's temperature and proportional share of block mass.
    let mut mass = [Mass::default(); N];
    for block in &stack[..stack_index] {
        for i in block.range.clone() {
            temp[i] = block.temp;
            mass[i] = block.mass * (vol[i] / block.vol);
        }
    }

    (temp, mass)
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

    use approx::{assert_relative_eq, relative_eq};
    use uom::si::{
        f64::MassDensity, mass::kilogram, mass_density::kilogram_per_cubic_meter,
        thermodynamic_temperature::degree_celsius, volume::cubic_meter,
    };

    /// An arbitrary density model that ensures cold is slightly denser than warm.
    fn rho(temp: ThermodynamicTemperature) -> MassDensity {
        let t = temp.get::<degree_celsius>();
        let d = 1000.0 - t * 1e-12;
        MassDensity::new::<kilogram_per_cubic_meter>(d)
    }

    fn ts<const N: usize>(temps_c: [f64; N]) -> [ThermodynamicTemperature; N] {
        temps_c.map(ThermodynamicTemperature::new::<degree_celsius>)
    }

    fn vs<const N: usize>(vols_m3: [f64; N]) -> [Volume; N] {
        vols_m3.map(Volume::new::<cubic_meter>)
    }

    fn t_in_c(temp: ThermodynamicTemperature) -> f64 {
        temp.get::<degree_celsius>()
    }

    fn m_in_kg(mass: Mass) -> f64 {
        mass.get::<kilogram>()
    }

    #[test]
    fn no_mixing_needed() {
        let vol = vs([1.0; 3]);
        let temp = ts([30.0, 40.0, 50.0]);
        let dens = temp.map(rho);

        let (temp, _) = stabilize(temp, &dens, &vol);
        assert_eq!(temp, ts([30.0, 40.0, 50.0]));
    }

    #[test]
    fn all_mixed() {
        let vol = vs([1.0; 3]);
        let temp = ts([50.0, 40.0, 30.0]);
        let dens = temp.map(rho);

        let (temp, mass) = stabilize(temp, &dens, &vol);

        assert_relative_eq!(t_in_c(temp[0]), 40.0, epsilon = 1e-12);
        assert_relative_eq!(t_in_c(temp[1]), 40.0, epsilon = 1e-12);
        assert_relative_eq!(t_in_c(temp[2]), 40.0, epsilon = 1e-12);

        assert_relative_eq!(m_in_kg(mass[0]), 1000.0, epsilon = 1e-10);
        assert_relative_eq!(m_in_kg(mass[1]), 1000.0, epsilon = 1e-10);
        assert_relative_eq!(m_in_kg(mass[2]), 1000.0, epsilon = 1e-10);
    }

    #[test]
    fn some_mixing() {
        let vol = vs([1.0; 5]);
        let temp = ts([20.0, 30.0, 50.0, 40.0, 42.0]);
        let dens = temp.map(rho);

        let (temp, mass) = stabilize(temp, &dens, &vol);

        assert_relative_eq!(t_in_c(temp[0]), 20.0, epsilon = 1e-12);
        assert_relative_eq!(t_in_c(temp[1]), 30.0, epsilon = 1e-12);
        assert_relative_eq!(t_in_c(temp[2]), 44.0, epsilon = 1e-12);
        assert_relative_eq!(t_in_c(temp[3]), 44.0, epsilon = 1e-12);
        assert_relative_eq!(t_in_c(temp[4]), 44.0, epsilon = 1e-12);

        assert!(
            mass.iter()
                .all(|m| relative_eq!(m.get::<kilogram>(), 1000.0, epsilon = 1e-10))
        );
    }

    #[test]
    fn some_mixing_uneven_volumes() {
        let vol = vs([1.0, 4.0, 2.0]);
        let temp = ts([2.0, 10.0, 4.0]);
        let dens = temp.map(rho);

        let (temp, mass) = stabilize(temp, &dens, &vol);

        assert_relative_eq!(t_in_c(temp[0]), 2.0, epsilon = 1e-12);
        assert_relative_eq!(t_in_c(temp[1]), 8.0, epsilon = 1e-12);
        assert_relative_eq!(t_in_c(temp[2]), 8.0, epsilon = 1e-12);

        assert_relative_eq!(m_in_kg(mass[0]), 1000.0, epsilon = 1e-10);
        assert_relative_eq!(m_in_kg(mass[1]), 4000.0, epsilon = 1e-10);
        assert_relative_eq!(m_in_kg(mass[2]), 2000.0, epsilon = 1e-10);
    }
}
