//! `field` blocks as they were in Rust before ND3b moved them into formula
//! files; kept for the test that the formulas give the same faces and bits.

use crate::common::*;
use crate::light::{MOMENT, MU0, TESLA};
use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const SHUE98: &str = "Shue, J.-H. et al. 1998, Magnetopause location under extreme solar wind conditions, J. Geophys. Res. 103, 17691: r = r₀ (2 / (1 + cos θ))^α, α = (0.58 − 0.007 B_z)(1 + 0.024 ln P_dyn), B_z in nT, P_dyn in nPa. doi:10.1029/98JA01103";
const KIV98: &str = "Kivelson, M. G. et al. 1998, Ganymede's magnetosphere: magnetometer overview, J. Geophys. Res. 103, 19963: a moon's own field stands off its host's field and plasma about two moon radii out. doi:10.1029/98JE00227";
const BRACE91: &str = "Brace, L. H. and Kliore, A. J. 1991, The structure of the Venus ionosphere, Space Sci. Rev. 55, 81: the ionopause sits where the ionosphere's pressure balances the solar wind's, a few hundred km up on Venus. doi:10.1007/BF00177136";
const LUH95: &str = "Luhmann, J. G. and Cravens, T. E. 1991, Magnetic fields in the ionosphere of Venus, Space Sci. Rev. 55, 201: induced magnetospheres of unmagnetised bodies with air. doi:10.1007/BF00177137";
const ZIM00: &str = "Zimmer, C., Khurana, K. K. and Kivelson, M. G. 2000, Subsurface oceans on Europa and Callisto: constraints from Galileo magnetometer observations, Icarus 147, 329: a conducting shell answers a changing field with a dipole of relative strength A. doi:10.1006/icar.2000.6456";
const KIV00: &str = "Kivelson, M. G. et al. 2000, Galileo magnetometer measurements: a stronger case for a subsurface ocean at Europa, Science 289, 1340: induced response close to that of a perfect conductor (A about 0.97). doi:10.1126/science.289.5483.1340";

/// Height, m, at which the air's own pressure P₀ exp(−h/H) falls to `k`
/// times the wind's ram pressure: a stand-in for the ionopause (ionospheric
/// plasma pressure is a small, fairly steady share of the neutral pressure
/// there). Zero if the air cannot hold the wind off above the ground.
pub fn ionopause_altitude(surface_pressure: f64, scale_height: f64, ram: f64, k: f64) -> f64 {
    if surface_pressure <= 0.0 || ram <= 0.0 {
        return 0.0;
    }
    (scale_height * m::ln(surface_pressure / (k * ram))).max(0.0)
}

/// Shue et al. 1998 flaring exponent, from the wind's ram pressure (Pa) and
/// the north-south field (T, positive northward).
pub fn shue_alpha(ram_pa: f64, bz_t: f64) -> f64 {
    let p_npa = (ram_pa * 1e9).max(1e-6);
    (0.58 - 0.007 * bz_t * 1e9) * (1.0 + 0.024 * m::ln(p_npa))
}

/// Shue boundary distance at angle θ from the nose.
pub fn shue_radius(r0: f64, alpha: f64, theta: f64) -> f64 {
    r0 * m::pow(2.0 / (1.0 + m::cos(theta)), alpha)
}

/// Stand-off of a dipole moment M (A m²) in an outside field B (T) and
/// plasma pressure p (Pa): the compressed dipole (2 B_dip) balances
/// B² / 2μ₀ + p.
pub fn standoff_in_field(moment: f64, outside_b: f64, plasma_pa: f64) -> f64 {
    let b_need = m::sqrt(outside_b * outside_b + 2.0 * MU0 * plasma_pa.max(0.0));
    if moment <= 0.0 || b_need <= 0.0 {
        return 0.0;
    }
    m::cbrt(MU0 * moment / (2.0 * m::PI * b_need))
}

/// Skin depth, m, of a conductor σ (S/m) at angular frequency ω.
pub fn skin_depth(sigma: f64, omega: f64) -> f64 {
    m::sqrt(2.0 / (MU0 * sigma * omega))
}

/// Induced response of a conducting shell (radius r, thickness h, σ) to a
/// field changing with period T: thin-shell form A = κ / √(1 + κ²),
/// κ = μ₀ σ ω r h_eff / 3, with the thickness capped at the skin depth.
pub fn induction_amplitude(sigma: f64, thickness: f64, radius: f64, period: f64) -> f64 {
    if sigma <= 0.0 || thickness <= 0.0 || period <= 0.0 {
        return 0.0;
    }
    let omega = 2.0 * m::PI / period;
    let h = thickness.min(skin_depth(sigma, omega));
    let k = MU0 * sigma * omega * radius * h / 3.0;
    k / m::sqrt(1.0 + k * k)
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let conductivity = D([-3, -1, 3, 2, 0, 0, 0]);
    vec![
        Def {
            id: "sse.field.ionopause_altitude",
            version: "1.0.0",
            inputs: vec![
                port("pressure", "atmosphere.surface_pressure", D::PRESSURE, None),
                port("scale_height", "atmosphere.scale_height", D::LENGTH, None),
                port("density", "wind.density", D::DENSITY, None),
                port("speed", "wind.speed", D::VELOCITY, None),
            ],
            outputs: vec![port("altitude", "body.ionopause_altitude", D::LENGTH, None)],
            params: vec![param("k", D::NONE, 1.0, (0.01, 100.0), "air pressure at the boundary over the wind's ram pressure (stand-in for the ionosphere's share)")],
            cost: CostClass::Trivial,
            references: &[BRACE91, LUH95],
            curious: "A world with air but no magnetic field still holds the wind off: the wind lights up the top of its air and pushes against that. Venus and Mars do this a few hundred kilometres up.",
            science: "The wind's ram pressure meets the upper atmosphere's pressure in an isothermal column. A stand-in for the ionospheric pressure balance, which needs the ionosphere's own profile.",
            formula: "h = H ln(P₀ / (k ρ v²))",
            f: |i, p| Ok(vec![ionopause_altitude(i[0].max(0.0), positive(i[1], "scale height")?, i[2].max(0.0) * i[3] * i[3], p[0])]),
        }
        .build(),
        Def {
            id: "sse.field.shue_flaring",
            version: "1.0.0",
            inputs: vec![
                port("density", "wind.density", D::DENSITY, None),
                port("speed", "wind.speed", D::VELOCITY, None),
            ],
            outputs: vec![port("alpha", "magnetosphere.flaring", D::NONE, None)],
            params: vec![param("bz_nt", D::NONE, 0.0, (-50.0, 50.0), "north-south field in the wind, nT (southward negative)")],
            cost: CostClass::Trivial,
            references: &[SHUE98],
            curious: "The bubble is blunt at the nose and flares out behind into a long tail; a stronger wind or a southward field opens it wider.",
            science: "Shue's empirical boundary: distance r₀ at the nose, flaring with angle from the nose by the exponent α.",
            formula: "α = (0.58 − 0.007 B_z)(1 + 0.024 ln P_dyn)",
            f: |i, p| Ok(vec![shue_alpha(i[0].max(0.0) * i[1] * i[1], p[0] * 1e-9)]),
        }
        .build(),
        Def {
            id: "sse.field.standoff_in_field",
            version: "1.0.0",
            inputs: vec![
                port("moment", "body.magnetic_moment", MOMENT, None),
                port("outside", "host.field_at_orbit", TESLA, None),
                port("plasma", "host.plasma_pressure_at_orbit", D::PRESSURE, None),
            ],
            outputs: vec![port("radius", "body.magnetopause_radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[KIV98],
            curious: "A moon with its own field inside a planet's bubble makes a bubble within a bubble. Ganymede's reaches about two of its own widths out against Jupiter's field.",
            science: "Inside a host's bubble the pressure outside is the host's field and plasma, not the star's wind. The moon's compressed dipole balances them.",
            formula: "R = (μ₀ M / (2π √(B² + 2μ₀ p)))^⅓",
            f: |i, _| Ok(vec![standoff_in_field(i[0].max(0.0), i[1].max(0.0), i[2].max(0.0))]),
        }
        .build(),
        Def {
            id: "sse.induction.response",
            version: "1.0.0",
            inputs: vec![
                port("conductivity", "layer.electrical_conductivity", conductivity, None),
                port("thickness", "layer.thickness", D::LENGTH, None),
                port("radius", "layer.outer_radius", D::LENGTH, None),
                port("period", "field.change_period", D::TIME, None),
            ],
            outputs: vec![port("amplitude", "induction.amplitude", D::NONE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[ZIM00, KIV00],
            curious: "A salty ocean under ice is a conductor. As a moon swings through its planet's tilted field, the ocean answers with a field of its own, and a magnetometer flying past can find the hidden sea.",
            science: "A thin conducting shell in a field changing at angular frequency ω answers with a dipole of relative strength κ/√(1+κ²), κ = μ₀σωrh/3; a shell thicker than the skin depth acts as one skin depth thick.",
            formula: "A = κ / √(1 + κ²),  κ = μ₀ σ ω r min(h, δ) / 3,  δ = √(2 / μ₀σω)",
            f: |i, _| Ok(vec![induction_amplitude(i[0].max(0.0), i[1].max(0.0), positive(i[2], "radius")?, positive(i[3], "period")?)]),
        }
        .build(),
    ]
}
