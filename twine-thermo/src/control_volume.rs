use twine_core::{
    TimeDerivative, TimeIntegrable,
    constraint::{Constrained, ConstraintError, StrictlyPositive},
};
use uom::{
    ConstZero,
    num_traits::Zero,
    si::f64::{MassDensity, MassRate, Power, Volume},
};

use crate::{
    HeatFlow, MassFlow, PropertyError, State, StateDerivative, WorkFlow,
    capability::{HasCv, HasEnthalpy, HasInternalEnergy, ThermoModel},
};

/// A finite control volume representing a well-mixed region of fluid.
///
/// The internal fluid state is assumed to be spatially uniform,
/// and any mass leaving the volume is at the current internal state.
/// Changes in kinetic and potential energy are neglected.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlVolume<Fluid> {
    volume: Constrained<Volume, StrictlyPositive>,
    state: State<Fluid>,
}

/// Represents mass, heat, or work flow across the boundary of a control volume.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoundaryFlow<Fluid> {
    Mass(MassFlow<Fluid>),
    Heat(HeatFlow),
    Work(WorkFlow),
}

impl<Fluid> ControlVolume<Fluid> {
    /// Creates a new [`ControlVolume`] from a volume and initial state.
    ///
    /// # Errors
    ///
    /// Returns a [`ConstraintError`] if `volume` is not strictly positive.
    pub fn new(volume: Volume, state: State<Fluid>) -> Result<Self, ConstraintError> {
        let volume = Constrained::new(volume)?;
        Ok(Self::from_constrained(volume, state))
    }

    /// Creates a new [`ControlVolume`] from a pre-validated positive volume and state.
    pub fn from_constrained(
        volume: Constrained<Volume, StrictlyPositive>,
        state: State<Fluid>,
    ) -> Self {
        Self { volume, state }
    }

    /// Returns the net mass flow rate into the control volume.
    ///
    /// Inflow contributions are positive.
    /// Outflow contributions are negative.
    ///
    /// Only [`BoundaryFlow::Mass`] entries affect the result.
    pub fn net_mass_flow<'a, I>(flows: I) -> MassRate
    where
        I: IntoIterator<Item = &'a BoundaryFlow<Fluid>>,
        Fluid: 'a,
    {
        flows
            .into_iter()
            .fold(MassRate::ZERO, |m_dot_net, flow| match flow {
                BoundaryFlow::Mass(mass_flow) => m_dot_net + mass_flow.signed_mass_rate(),
                _ => m_dot_net,
            })
    }

    /// Returns the net energy flow rate into the control volume.
    ///
    /// Inflow contributions are positive.
    /// Outflow contributions are negative.
    ///
    /// # Parameters
    ///
    /// - `flows`: Iterator over boundary flows.
    /// - `model`: Model used to compute thermodynamic properties.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if any required enthalpy cannot be computed.
    pub fn net_energy_flow<'a, I, Model>(
        &self,
        flows: I,
        model: &Model,
    ) -> Result<Power, PropertyError>
    where
        Model: ThermoModel<Fluid = Fluid> + HasEnthalpy,
        I: IntoIterator<Item = &'a BoundaryFlow<Fluid>>,
        Fluid: 'a,
    {
        let h_cv = model.enthalpy(&self.state)?;
        flows.into_iter().try_fold(Power::ZERO, |q_dot_net, flow| {
            let q_dot_flow = match flow {
                BoundaryFlow::Mass(MassFlow::In(stream)) => stream.enthalpy_flow(model)?,
                BoundaryFlow::Mass(MassFlow::Out(m_dot)) => -m_dot.into_inner() * h_cv,
                BoundaryFlow::Mass(MassFlow::None) => Power::ZERO,
                BoundaryFlow::Heat(heat_flow) => heat_flow.signed(),
                BoundaryFlow::Work(work_flow) => work_flow.signed(),
            };
            Ok(q_dot_net + q_dot_flow)
        })
    }
}

impl<Fluid> ControlVolume<Fluid>
where
    Fluid: TimeIntegrable<Derivative = ()>,
{
    /// Returns the time derivative of the control volume's internal state.
    ///
    /// The model applies transient mass and energy balances to a fixed-volume,
    /// well-mixed control volume with negligible kinetic and potential energy changes.
    ///
    /// # Mass and Energy Balances
    ///
    /// Conservation of mass and energy yield the following, using a
    /// positive-into-the-system sign convention for both heat and work:
    ///
    /// ```text
    /// dM/dt = V · dρ/dt = ∑ṁ_in − ∑ṁ_out
    ///
    /// dU/dt = Q̇_net + Ẇ_net + ∑(ṁ_in · h_in) − ∑(ṁ_out · h_out)
    /// ```
    ///
    /// The total time derivative of internal energy in the fixed volume is:
    ///
    /// ```text
    /// dU/dt = V · (ρ · du/dt + u · dρ/dt)
    ///       = V · (ρ · cv · dT/dt + u · dρ/dt)
    /// ```
    ///
    /// Substituting and solving for `dρ/dt` and `dT/dt`:
    ///
    /// ```text
    /// dρ/dt = (∑ṁ_in − ∑ṁ_out) / V
    ///
    /// dT/dt = (Q̇_net + Ẇ_net + ∑(ṁ_in · h_in) − ∑(ṁ_out · h_out) − u · V · dρ/dt)
    ///         / (ρ · V · cv)
    /// ```
    ///
    /// Where:
    ///
    /// - `dρ/dt` = rate of change of fluid density (kg/m³·s)
    /// - `dT/dt` = rate of change of fluid temperature (K/s)
    /// - `dU/dt` = rate of change of total internal energy in the volume (W)
    /// - `ṁ_in`  = mass inflow rate (kg/s)
    /// - `ṁ_out` = mass outflow rate (kg/s)
    /// - `h_in`  = specific enthalpy of the inflow (J/kg)
    /// - `h_out` = specific enthalpy of the outflow (J/kg)
    /// - `Q̇_net` = net heat transfer rate into the system (W)
    /// - `Ẇ_net` = net work rate into the system (W)
    /// - `u`     = specific internal energy of the fluid (J/kg)
    /// - `ρ`     = fluid density (kg/m³)
    /// - `cv`    = specific heat at constant volume (J/kg·K)
    /// - `V`     = volume of the control region (m³)
    ///
    /// Note that SI units are shown for clarity.
    /// All computations use unit-safe types via the [`uom`] system,
    /// which enforces dimensional consistency at compile time.
    ///
    /// # Parameters
    ///
    /// - `flows`: Boundary flows affecting the control volume.
    /// - `model`: Model used to compute thermodynamic properties.
    ///
    /// # Errors
    ///
    /// Returns a [`PropertyError`] if any required property cannot be computed.
    pub fn state_derivative<Model>(
        &self,
        flows: &[BoundaryFlow<Fluid>],
        model: &Model,
    ) -> Result<StateDerivative<Fluid>, PropertyError>
    where
        Model: ThermoModel<Fluid = Fluid> + HasCv + HasEnthalpy + HasInternalEnergy,
    {
        let volume = self.volume.into_inner();
        let heat_capacity = volume * self.state.density * model.cv(&self.state)?;

        let m_dot_net = Self::net_mass_flow(flows);
        let q_dot_net = self.net_energy_flow(flows, model)?;

        let (rho_dt, temp_dt) = if m_dot_net.is_zero() {
            (
                TimeDerivative::<MassDensity>::ZERO,
                q_dot_net / heat_capacity,
            )
        } else {
            let u = model.internal_energy(&self.state)?;
            (
                m_dot_net / volume,
                (q_dot_net - m_dot_net * u) / heat_capacity,
            )
        };

        Ok(StateDerivative::<Fluid> {
            temperature: temp_dt,
            density: rho_dt,
            fluid: (),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use twine_core::TimeIntegrable;
    use twine_core::constraint::Constrained;
    use uom::si::{
        f64::{
            MassDensity, MassRate, SpecificHeatCapacity, ThermodynamicTemperature, Time, Volume,
        },
        mass_density::kilogram_per_cubic_meter,
        mass_rate::kilogram_per_second,
        power::watt,
        specific_heat_capacity::joule_per_kilogram_kelvin,
        thermodynamic_temperature::kelvin,
        volume::cubic_meter,
    };

    use crate::{
        BoundaryFlow, ControlVolume, HeatFlow, MassFlow, State, Stream, WorkFlow,
        model::perfect_gas::{PerfectGas, PerfectGasFluid, PerfectGasParameters},
        units::SpecificGasConstant,
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    struct MockGas;

    impl TimeIntegrable for MockGas {
        type Derivative = ();

        fn step(self, _derivative: Self::Derivative, _dt: Time) -> Self {
            self
        }
    }

    impl PerfectGasFluid for MockGas {
        fn parameters() -> PerfectGasParameters {
            PerfectGasParameters::new(
                SpecificGasConstant::new::<joule_per_kilogram_kelvin>(400.0),
                SpecificHeatCapacity::new::<joule_per_kilogram_kelvin>(1000.0),
            )
        }
    }

    fn mock_gas_model() -> PerfectGas<MockGas> {
        PerfectGas::<MockGas>::new().expect("mock gas parameters must be physically valid")
    }

    #[test]
    fn equal_inflow_and_outflow_conserves_mass_and_energy() {
        let thermo = mock_gas_model();

        let volume = Volume::new::<cubic_meter>(2.0);

        let state = State::new(
            ThermodynamicTemperature::new::<kelvin>(300.0),
            MassDensity::new::<kilogram_per_cubic_meter>(1.0),
            MockGas,
        );

        let (inflow, outflow) = MassFlow::balanced_pair(
            Stream::new(
                MassRate::new::<kilogram_per_second>(0.3),
                state.with_temperature(ThermodynamicTemperature::new::<kelvin>(350.0)),
            )
            .unwrap(),
        );

        let derivative = ControlVolume::new(volume, state)
            .unwrap()
            .state_derivative(
                &[BoundaryFlow::Mass(inflow), BoundaryFlow::Mass(outflow)],
                &thermo,
            )
            .unwrap();

        // Mass balance:
        //   dρ/dt = (ṁ_in − ṁ_out) / V = 0
        assert_relative_eq!(derivative.density.value, 0.0);

        // Energy balance:
        //   Q̇_net = ṁ · cp · (T_in − T) = 0.3 · 1000 · 50 = 15,000
        //   C = ρ · V · cv = 1 · 2 · 600 = 1200
        //   dT/dt = Q̇_net / C = 15000 / 1200 = 12.5
        assert_relative_eq!(derivative.temperature.value, 12.5);
    }

    #[test]
    fn adiabatic_outflow_decreases_temperature_and_density() {
        let thermo = mock_gas_model();

        let volume = Volume::new::<cubic_meter>(2.0);

        let state = State::new(
            ThermodynamicTemperature::new::<kelvin>(300.0),
            MassDensity::new::<kilogram_per_cubic_meter>(1.0),
            MockGas,
        );

        let m_dot = Constrained::new(MassRate::new::<kilogram_per_second>(0.2)).unwrap();

        let derivative = ControlVolume::new(volume, state)
            .unwrap()
            .state_derivative(&[BoundaryFlow::Mass(MassFlow::Out(m_dot))], &thermo)
            .unwrap();

        // Mass balance:
        //   dρ/dt = −ṁ / V = −0.2 / 2 = −0.1
        assert_relative_eq!(derivative.density.value, -0.1);

        // Energy balance for ideal gas adiabatic blowdown:
        //   dT/dt = −ṁ · R · T / (cv · ρ · V)
        //         = −0.2 · 400 · 300 / (600 · 1 · 2) = −20
        assert_relative_eq!(derivative.temperature.value, -20.0);
    }

    #[test]
    fn heat_input_without_mass_flow_increases_temperature() {
        let thermo = mock_gas_model();

        let volume = Volume::new::<cubic_meter>(1.0);

        let state = State::new(
            ThermodynamicTemperature::new::<kelvin>(300.0),
            MassDensity::new::<kilogram_per_cubic_meter>(2.0),
            MockGas,
        );

        let derivative = ControlVolume::new(volume, state)
            .unwrap()
            .state_derivative(
                &[BoundaryFlow::Heat(
                    HeatFlow::incoming(Power::new::<watt>(600.0)).unwrap(),
                )],
                &thermo,
            )
            .unwrap();

        // dρ/dt = 0 (no mass flow)
        assert_relative_eq!(derivative.density.value, 0.0);

        // dT/dt = Q̇ / (ρ · V · cv) = 600 / (2 · 1 · 600) = 0.5
        assert_relative_eq!(derivative.temperature.value, 0.5);
    }

    #[test]
    fn net_power_out_without_mass_flow_decreases_temperature() {
        let thermo = mock_gas_model();

        let volume = Volume::new::<cubic_meter>(3.0);

        let state = State::new(
            ThermodynamicTemperature::new::<kelvin>(290.0),
            MassDensity::new::<kilogram_per_cubic_meter>(1.2),
            MockGas,
        );

        let derivative = ControlVolume::new(volume, state)
            .unwrap()
            .state_derivative(
                &[
                    BoundaryFlow::Heat(HeatFlow::incoming(Power::new::<watt>(60.0)).unwrap()),
                    BoundaryFlow::Work(WorkFlow::outgoing(Power::new::<watt>(5460.0)).unwrap()),
                ],
                &thermo,
            )
            .unwrap();

        // dρ/dt = 0 (no mass flow)
        assert_relative_eq!(derivative.density.value, 0.0);

        // Energy storage capacity:
        //   C = ρ · V · cv = 1.2 · 3 · 600 = 2,160
        //
        // Net power into CV:
        //   Q̇_net = 60 − 5,460 = −5,400
        //
        // Temperature rate:
        //   dT/dt = Q̇_net / C = −5,400 / 2,160 = −2.5
        assert_relative_eq!(derivative.temperature.value, -2.5);
    }
}
