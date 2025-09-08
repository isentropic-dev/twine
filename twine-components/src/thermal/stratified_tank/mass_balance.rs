use std::array;

use uom::{ConstZero, si::f64::VolumeRate};

/// Compute upward node-to-node flows from bottom to top, positive upward.
///
/// Returns an array of length `N`.
/// Each entry `0..N-2` is the flow from node `i` to node `i+1`.
/// Entry `N-1` is the final residual and should be 0 if mass is conserved.
/// This invariant is checked in debug builds to within 1e-12 m³/s.
///
/// # Parameters
///
/// - `port_flow_rates`: Flow rate for each port pair.
/// - `port_flow_weights`: Inlet and outlet weight pairs for each port at each node.
pub(super) fn compute_upward_flows<const N: usize, const P: usize>(
    port_flow_rates: &[VolumeRate; P],
    port_flow_weights: &[[(f64, f64); P]; N],
) -> [VolumeRate; N] {
    let mut flow_up = VolumeRate::ZERO;

    let upward_flows: [VolumeRate; N] = array::from_fn(|i| {
        // Net inflow to node i from ports:
        // Σ_k[ v_dot_port[k] * (w_in[i][k] - w_out[i][k]) ]
        let net_port_inflow = port_flow_rates
            .iter()
            .zip(port_flow_weights[i].iter())
            .fold(VolumeRate::ZERO, |acc, (&v_dot_port, (wi, wo))| {
                acc + v_dot_port * (wi - wo)
            });

        // Node i volume balance (density constant):
        // flow_up[i] = flow_up[i-1] + net_port_inflow
        flow_up += net_port_inflow;

        // Upward flow across the boundary above node i (positive = upward).
        // Negative means downward (from i+1 to i).
        flow_up
    });

    #[cfg(debug_assertions)]
    {
        let residual = upward_flows[N - 1].value; // m^3/s
        assert!(
            residual.abs() < 1e-12,
            "Mass is not conserved; residual at top boundary = {residual}",
        );
    }

    upward_flows
}

#[cfg(test)]
mod tests {
    use super::*;

    use approx::assert_relative_eq;
    use uom::si::{f64::VolumeRate, volume_rate::cubic_meter_per_second};

    fn rate(v_m3s: f64) -> VolumeRate {
        VolumeRate::new::<cubic_meter_per_second>(v_m3s)
    }

    #[test]
    fn single_port_bottom_in_top_out() {
        // N=3 nodes, P=1 port pair
        let port_flow_rates = [rate(1.0)];

        // Inlet all into node 0; outlet all from node 2
        let weights = [
            [(1.0, 0.0)], // bottom
            [(0.0, 0.0)], // middle
            [(0.0, 1.0)], // top
        ];

        let flow_up = compute_upward_flows(&port_flow_rates, &weights);

        assert_relative_eq!(flow_up[0].value, 1.0);
        assert_relative_eq!(flow_up[1].value, 1.0);
        assert_relative_eq!(flow_up[2].value, 0.0); // residual
    }

    #[test]
    fn inlet_and_outlet_on_same_node_produces_no_vertical_flow() {
        // N=3 nodes, P=1 port pair
        const N: usize = 3;
        const P: usize = 1;

        let port_flow_rates = [rate(0.8)];

        // Both inlet and outlet entirely at node 1 → no net source anywhere.
        let weights = [
            [(0.0, 0.0)], // bottom
            [(1.0, 1.0)], // middle
            [(0.0, 0.0)], // top
        ];

        let flow_up = compute_upward_flows::<N, P>(&port_flow_rates, &weights);

        // No vertical transport required; cumulative stays zero and residual is zero.
        assert_relative_eq!(flow_up[0].value, 0.0);
        assert_relative_eq!(flow_up[1].value, 0.0);
        assert_relative_eq!(flow_up[2].value, 0.0); // residual
    }

    #[test]
    fn two_ports_mixed_distribution() {
        // N=3 nodes, P=2 port pairs
        const N: usize = 3;
        const P: usize = 2;

        let port_flow_rates = [rate(0.3), rate(0.5)];

        let weights = [
            [(1.0, 0.0), (0.0, 1.0)], // 100% of p0 in, 100% of p1 out
            [(0.0, 0.0), (0.6, 0.0)], // 60% of p1 in, nothing out
            [(0.0, 1.0), (0.4, 0.0)], // 40% of p1 in, 100% of p0 out
        ];

        // s0 = +0.3 - 0.5 = -0.2
        // s1 = +0.5*0.6 = +0.3
        // s2 = +0.5*0.4 - 0.3 = -0.1
        // cumulative: [-0.2, +0.1, 0.0]
        let flow_up = compute_upward_flows::<N, P>(&port_flow_rates, &weights);

        assert_relative_eq!(flow_up[0].value, -0.2);
        assert_relative_eq!(flow_up[1].value, 0.1);
        assert_relative_eq!(flow_up[2].value, 0.0); // residual
    }
}
