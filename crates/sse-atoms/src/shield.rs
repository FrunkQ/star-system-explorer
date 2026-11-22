//! Cosmic-ray shielding and dose (magnetic fields design 6.3, stage MF7
//! part 1). A dipole turns away charged particles below a cut-off rigidity
//! that depends on magnetic latitude and distance (Størmer); air then
//! absorbs what gets through. Each world's dose is computed from its own
//! field and air column, never from a "shielded" switch: shielding is an
//! effect, and a thick air with no field can shield better than a field
//! with no air.

use crate::common::*;
use crate::light::{MOMENT, MU0};
use sse_core::math as m;
use sse_core::quantity::consts::C;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

/// Volt (rigidity is momentum per charge, in volts).
pub const VOLT: D = D([2, 1, -3, -1, 0, 0, 0]);
/// Mass per area (an air column), kg/m².
pub const COLUMN: D = D([-2, 1, 0, 0, 0, 0, 0]);
/// Dose rate, Sv/s (J/kg/s).
pub const DOSE_RATE: D = D([2, 0, -3, 0, 0, 0, 0]);

const SMART05: &str = "Smart, D. F. and Shea, M. A. 2005, A review of geomagnetic cutoff rigidities for earth-orbiting spacecraft, Adv. Space Res. 36, 2012: Størmer's cut-off; for vertical arrival R_c = (μ₀ M c / 16π r²) cos⁴λ, about 14.9 GV at Earth's equator. doi:10.1016/j.asr.2004.09.015";
const PDG24: &str = "Navas, S. et al. (Particle Data Group) 2024, Review of Particle Physics: Cosmic rays, Phys. Rev. D 110, 030001: primary nucleon intensity falls as about E^−2.7 above a few GeV (integral index about 1.7). doi:10.1103/PhysRevD.110.030001";
const CLEM00: &str = "Clem, J. M. and Dorman, L. I. 2000, Neutron monitor response functions, Space Sci. Rev. 93, 335: ground count rate against cut-off follows N(R_c) = N₀ (1 − exp(−α R_c^−k)); attenuation in air about 140 g/cm² (barometric coefficient about 0.7 % per hPa). doi:10.1023/A:1026508915269";
const ATRI13: &str = "Atri, D., Hariharan, B. and Grießmeier, J.-M. 2013, Galactic cosmic ray-induced radiation dose on terrestrial exoplanets, Astrobiology 13, 910: the air column, not the field, is the main shield for a world with thick air. doi:10.1089/ast.2013.1052";
const ZEIT13: &str = "Zeitlin, C. et al. 2013, Measurements of energetic particle radiation in transit to Mars on the Mars Science Laboratory, Science 340, 1080: 1.84 mSv per day in interplanetary space near 1 AU. doi:10.1126/science.1235989";
const UNSC08: &str = "UNSCEAR 2008, Sources and effects of ionizing radiation, Annex B: cosmic rays at sea level give about 0.39 mSv a year (0.28 directly ionising, mostly muons, and 0.11 from neutrons). https://www.unscear.org/unscear/en/publications/2008_1.html";

/// Størmer vertical cut-off rigidity, V, at distance r (m) from a dipole of
/// moment M (A m²), at magnetic latitude λ (radians).
pub fn cutoff_rigidity(moment: f64, r: f64, lat: f64) -> f64 {
    if moment <= 0.0 || r <= 0.0 {
        return 0.0;
    }
    let c = m::cos(lat).abs();
    MU0 * moment * C / (16.0 * m::PI * r * r) * (c * c * c * c)
}

/// Share of the free-space cosmic-ray dose carried by particles above
/// rigidity R (V): (1 + R/R₀)^−γ, a high-rigidity power law with the
/// solar-modulation turnover at R₀.
pub fn space_share(rigidity: f64, r0_gv: f64, gamma: f64) -> f64 {
    m::pow(1.0 + rigidity.max(0.0) * 1e-9 / r0_gv, -gamma)
}

/// Share of the polar ground rate left at cut-off R (V): the Dorman
/// function 1 − exp(−α R_c^−k), R_c in GV; 1 at no cut-off.
pub fn ground_share(rigidity: f64, alpha: f64, k: f64) -> f64 {
    let gv = rigidity * 1e-9;
    if gv <= 1e-6 {
        return 1.0;
    }
    1.0 - m::exp(-alpha * m::pow(gv, -k))
}

/// Settings for the air's dose block.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AirDose {
    /// Attenuation length of the shower's nucleonic part, kg/m².
    pub lambda_n: f64,
    /// Attenuation length of its penetrating (muon) part, kg/m².
    pub lambda_mu: f64,
    /// Share of the top-of-air dose that turns into the penetrating part.
    pub muon_share: f64,
}

/// Dose rate below an air column X (kg/m²), from the free-space dose D₀ and
/// the shares of it that pass the cut-off in space and at the ground. The
/// shower's own cut-off response moves from the space share (thin air) to
/// the ground share (thick air) as the nucleonic part is absorbed. `sky`
/// is the share of the sky not blocked by the body itself.
pub fn air_dose(free: f64, column: f64, space: f64, ground: f64, sky: f64, a: &AirDose) -> f64 {
    let x = column.max(0.0);
    let w = m::exp(-x / a.lambda_n);
    let share = w * space + (1.0 - w) * ground;
    free * sky
        * ((1.0 - a.muon_share) * w * share + a.muon_share * m::exp(-x / a.lambda_mu) * ground)
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    vec![
        Def {
            id: "sse.shield.cutoff",
            version: "1.0.0",
            inputs: vec![
                port("moment", "body.magnetic_moment", MOMENT, None),
                port("distance", "shield.distance", D::LENGTH, None),
                port("latitude", "shield.magnetic_latitude", D::NONE, Some((-m::PI / 2.0, m::PI / 2.0))),
            ],
            outputs: vec![port("rigidity", "shield.cutoff_rigidity", VOLT, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[SMART05],
            curious: "A magnetic field turns cosmic rays aside, most strongly at its equator. Near the magnetic poles the field lines point straight down and let them in, which is why aurorae and the highest radiation doses are found there.",
            science: "Størmer's result for a dipole: a particle needs at least the cut-off rigidity (momentum per charge) to arrive vertically at magnetic latitude λ and distance r. It falls as 1/r², so a field shields less with height.",
            formula: "R_c = (μ₀ M c / 16π r²) cos⁴λ  (vertical arrival)",
            f: |i, _| Ok(vec![cutoff_rigidity(i[0].max(0.0), positive(i[1], "distance")?, i[2])]),
        }
        .build(),
        Def {
            id: "sse.shield.space_share",
            version: "1.0.0",
            inputs: vec![port("rigidity", "shield.cutoff_rigidity", VOLT, None)],
            outputs: vec![port("share", "shield.space_share", D::NONE, Some((0.0, 1.0)))],
            params: vec![
                param("r0_gv", D::NONE, 2.0, (0.1, 20.0), "rigidity, GV, below which the star's wind has already thinned the cosmic rays"),
                param("gamma", D::NONE, 1.7, (1.0, 3.0), "integral spectral index of cosmic rays at high rigidity"),
            ],
            cost: CostClass::Trivial,
            references: &[PDG24],
            curious: "Cosmic rays come in all energies, many more weak ones than strong. A field that turns away the weak ones removes most of the dose in space.",
            science: "The integral cosmic-ray spectrum above rigidity R falls as a power of R with index about 1.7; the star's wind already removes most particles below a few GV, so the share bends over below R₀.",
            formula: "share = (1 + R_c / R₀)^−γ",
            f: |i, p| Ok(vec![space_share(i[0], p[0], p[1])]),
        }
        .build(),
        Def {
            id: "sse.shield.ground_share",
            version: "1.0.0",
            inputs: vec![port("rigidity", "shield.cutoff_rigidity", VOLT, None)],
            outputs: vec![port("share", "shield.ground_share", D::NONE, Some((0.0, 1.0)))],
            params: vec![
                param("alpha", D::NONE, 10.0, (1.0, 30.0), "Dorman function α (sea level)"),
                param("k", D::NONE, 1.0, (0.5, 1.5), "Dorman function k (sea level)"),
            ],
            cost: CostClass::Trivial,
            references: &[CLEM00],
            curious: "Under a thick air, only the strongest cosmic rays make showers that reach the ground, and the field cannot turn those aside. So the field changes the ground dose by about half, not a hundredfold.",
            science: "Neutron monitors show how a ground rate falls with cut-off: the Dorman function, fitted at sea level, integrates the shower yield over the cosmic-ray spectrum.",
            formula: "share = 1 − exp(−α R_c^−k),  R_c in GV",
            f: |i, p| Ok(vec![ground_share(i[0], p[0], p[1])]),
        }
        .build(),
        Def {
            id: "sse.shield.air_dose",
            version: "1.0.0",
            inputs: vec![
                port("free", "shield.free_space_dose_rate", DOSE_RATE, None),
                port("column", "atmosphere.column_mass", COLUMN, None),
                port("space", "shield.space_share", D::NONE, Some((0.0, 1.0))),
                port("ground", "shield.ground_share", D::NONE, Some((0.0, 1.0))),
                port("sky", "shield.open_sky", D::NONE, Some((0.0, 1.0))),
            ],
            outputs: vec![port("dose", "shield.dose_rate", DOSE_RATE, None)],
            params: vec![
                param("lambda_n", COLUMN, 1400.0, (500.0, 3000.0), "attenuation length of the shower's nucleonic part, kg/m² (140 g/cm²)"),
                param("lambda_mu", COLUMN, 6000.0, (2000.0, 20000.0), "attenuation length of its penetrating muon part, kg/m²"),
                param("muon_share", D::NONE, 0.0045, (0.0, 0.1), "share of the top-of-air dose that becomes penetrating muons"),
            ],
            cost: CostClass::Trivial,
            references: &[ATRI13, CLEM00, ZEIT13, UNSC08],
            curious: "Air is a shield. Earth's column of air weighs as much as ten metres of water, and cuts the cosmic-ray dose about a thousandfold; Mars's thin air barely helps.",
            science: "Two-part attenuation: the nucleonic cascade falls off with about 140 g/cm²; muons, made in it, penetrate much further and dominate at sea level. The body itself blocks the sky below the horizon.",
            formula: "D = D₀ s [(1 − f_μ) w (w S_space + (1 − w) S_ground) + f_μ e^(−X/λ_μ) S_ground],  w = e^(−X/λ_n)",
            f: |i, p| {
                let a = AirDose { lambda_n: p[0], lambda_mu: p[1], muon_share: p[2] };
                Ok(vec![air_dose(i[0].max(0.0), i[1], i[2], i[3], i[4], &a)])
            },
        }
        .build(),
    ]
}
