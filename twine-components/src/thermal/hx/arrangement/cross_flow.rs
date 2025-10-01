use crate::thermal::hx::effectiveness_ntu::{EffectivenessRelation, NtuRelation};

struct CrossFlow<T: MixState, U: MixState>(T, U);

struct Mixed;
struct Unmixed;

trait MixState {}
impl MixState for Mixed {}
impl MixState for Unmixed {}

impl EffectivenessRelation for CrossFlow<Mixed, Mixed> {
    fn effectiveness(
        &self,
        ntu: crate::thermal::hx::Ntu,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Effectiveness {
        todo!()
    }
}

impl EffectivenessRelation for CrossFlow<Unmixed, Unmixed> {
    fn effectiveness(
        &self,
        ntu: crate::thermal::hx::Ntu,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Effectiveness {
        todo!()
    }
}

impl EffectivenessRelation for CrossFlow<Mixed, Unmixed> {
    fn effectiveness(
        &self,
        ntu: crate::thermal::hx::Ntu,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Effectiveness {
        todo!()
    }
}

impl EffectivenessRelation for CrossFlow<Unmixed, Mixed> {
    fn effectiveness(
        &self,
        ntu: crate::thermal::hx::Ntu,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Effectiveness {
        todo!()
    }
}

impl NtuRelation for CrossFlow<Mixed, Unmixed> {
    fn ntu(
        &self,
        effectiveness: crate::thermal::hx::Effectiveness,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Ntu {
        todo!()
    }
}

impl NtuRelation for CrossFlow<Unmixed, Mixed> {
    fn ntu(
        &self,
        effectiveness: crate::thermal::hx::Effectiveness,
        capacitance_rates: [crate::thermal::hx::CapacitanceRate; 2],
    ) -> crate::thermal::hx::Ntu {
        todo!()
    }
}
