//! B1: one body, physics by condition. Any object, from a rocky world to a
//! hundred-solar-mass star, asks the same question here: does it shine by
//! itself, and why? The answer comes from its mass and age on one mass axis,
//! never from what it is called.
//!
//! - Above the hydrogen-burning limit: the life line (Hurley, Pols and Tout
//!   2000), so fusion, giants and remnants as F0 built them.
//! - Between deuterium burning (about 13 Jupiter masses) and hydrogen
//!   burning: a brown dwarf, burning deuterium while young and then cooling.
//! - Below that, with a deep hydrogen and helium envelope: a giant planet,
//!   cooling and contracting on the same law (Burrows et al. 2001 eq. 3 is
//!   one fit across brown dwarfs and giant planets), so the two meet with no
//!   seam at the deuterium limit.
//! - Without an envelope: no light of its own beyond heat flow, which the
//!   heat budget (P1) carries.

use crate::hurley::Fits;
use crate::lifeline::{teff, LifeLine, Phase, BROWN_DWARF_MIN, MASS_RANGE};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Mass above which a world keeps a deep hydrogen and helium envelope and
/// cools like a giant: about 10 Earth masses, solar masses (the core mass
/// for runaway gas accretion, Pollack et al. 1996).
pub const ENVELOPE_MIN_MSUN: f64 = 3.0e-5;
/// Jupiter's radius, solar radii: giant planets and old brown dwarfs sit
/// near it whatever their mass (electron degeneracy against Coulomb).
pub const R_JUPITER_RSUN: f64 = sse_core::sheet::R_JUP_EQUATORIAL / sse_core::sheet::R_SUN;
/// Deuterium burning lasts about this long at the deuterium limit, Myr,
/// growing as mass squared above it (Spiegel, Burrows and Milsom 2011).
pub const DEUTERIUM_MYR_AT_LIMIT: f64 = 10.0;

/// What makes a body shine.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LightSource {
    /// Hydrogen (and later helium and heavier) fusion.
    Fusion,
    /// Deuterium burning in a young brown dwarf.
    Deuterium,
    /// Gravitational contraction before the main sequence.
    Contraction,
    /// Leftover heat of formation leaking out: brown dwarfs, giant planets,
    /// white dwarfs and young neutron stars.
    Cooling,
    /// Nothing of its own (a black hole without a disc, a rocky world).
    None,
}

/// A body's own light.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SelfLight {
    /// Own luminosity, solar.
    pub luminosity_lsun: f64,
    /// Effective temperature of that light, K (0 when there is none).
    pub teff_k: f64,
    pub radius_rsun: f64,
    pub source: LightSource,
    /// The life line's phase where there is one (main sequence, giant, ...),
    /// "giant planet" or "rocky" below the deuterium limit.
    pub phase: String,
    pub why: String,
}

/// The light of any body of `mass_msun` at `age_myr` since it formed.
/// `radius_rsun` is the body's own radius where formation gave one (giant
/// planets); stars and brown dwarfs use the life line's.
pub fn self_light(
    fits: &Fits,
    mass_msun: f64,
    age_myr: f64,
    radius_rsun: Option<f64>,
) -> SelfLight {
    let age = age_myr.max(0.0);
    if mass_msun >= BROWN_DWARF_MIN && mass_msun <= MASS_RANGE.1 {
        let life = LifeLine::new(fits, mass_msun).expect("inside the life line's range");
        let s = life.state_at(fits, age);
        let source = match s.phase {
            Phase::PreMainSequence => LightSource::Contraction,
            Phase::MainSequence | Phase::HertzsprungGap | Phase::Giant | Phase::WolfRayet => {
                LightSource::Fusion
            }
            Phase::BrownDwarf if age < deuterium_myr(mass_msun) => LightSource::Deuterium,
            Phase::BrownDwarf | Phase::WhiteDwarf | Phase::NeutronStar => LightSource::Cooling,
            Phase::BlackHole => LightSource::None,
        };
        let why = match source {
            LightSource::Deuterium => format!(
                "{:.1} Jupiter masses: burning deuterium, which lasts about {:.0} Myr",
                mass_msun / MJ_MSUN,
                deuterium_myr(mass_msun)
            ),
            LightSource::Cooling if s.phase == Phase::BrownDwarf => format!(
                "{:.1} Jupiter masses: too light to burn hydrogen; its deuterium is spent and it cools",
                mass_msun / MJ_MSUN
            ),
            _ => format!("{}: on the life line", s.phase.label()),
        };
        return SelfLight {
            luminosity_lsun: s.luminosity,
            teff_k: s.teff,
            radius_rsun: s.radius,
            source,
            phase: s.phase.label().to_string(),
            why,
        };
    }
    if (ENVELOPE_MIN_MSUN..BROWN_DWARF_MIN).contains(&mass_msun) {
        let r = radius_rsun.unwrap_or(R_JUPITER_RSUN);
        let l = cooling_luminosity(mass_msun, age);
        return SelfLight {
            luminosity_lsun: l,
            teff_k: teff(l, r),
            radius_rsun: r,
            source: LightSource::Cooling,
            phase: "giant planet".into(),
            why: format!(
                "{:.2} Jupiter masses: below the deuterium limit, it shines only by the heat of its formation leaking out",
                mass_msun / MJ_MSUN
            ),
        };
    }
    SelfLight {
        luminosity_lsun: 0.0,
        teff_k: 0.0,
        radius_rsun: radius_rsun.unwrap_or(0.0),
        source: LightSource::None,
        phase: if mass_msun < ENVELOPE_MIN_MSUN {
            "rocky".into()
        } else {
            "beyond the life line".into()
        },
        why: if mass_msun < ENVELOPE_MIN_MSUN {
            "no deep envelope: its own heat leaks out as heat flow, not light".into()
        } else {
            format!("{mass_msun} solar masses is beyond the life line's fits")
        },
    }
}

/// Jupiter's mass in solar masses.
pub const MJ_MSUN: f64 = sse_core::sheet::GM_JUP / sse_core::sheet::GM_SUN;

/// How long deuterium burning lasts for a brown dwarf, Myr.
pub fn deuterium_myr(mass_msun: f64) -> f64 {
    let x = mass_msun / BROWN_DWARF_MIN;
    DEUTERIUM_MYR_AT_LIMIT * x * x
}

/// Burrows et al. 2001 eq. 3 for brown dwarfs and giant planets, solar
/// luminosities: L = 4×10⁻⁵ (t / Gyr)^−1.3 (M / 0.05 M_sun)^2.64.
pub fn cooling_luminosity(mass_msun: f64, age_myr: f64) -> f64 {
    let gyr = (age_myr / 1000.0).max(0.001);
    4.0e-5 * m::pow(gyr, -1.3) * m::pow(mass_msun / 0.05, 2.64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fits() -> Fits {
        Fits::new(0.0142)
    }

    #[test]
    fn one_mass_axis_from_rock_to_star() {
        let f = fits();
        let sun = self_light(&f, 1.0, 4570.0, None);
        assert_eq!(sun.source, LightSource::Fusion);
        assert!((sun.luminosity_lsun - 1.0).abs() < 0.15, "{sun:?}");
        let earth = self_light(&f, 3.0e-6, 4570.0, None);
        assert_eq!(earth.source, LightSource::None);
        assert_eq!(earth.phase, "rocky");
        // Jupiter: Burrows' fit gives about 1.6e-10 against the measured
        // 8.7e-10 L_sun (docs/fudges.md): helium rain is not modelled.
        let jup = self_light(&f, MJ_MSUN, 4570.0, Some(R_JUPITER_RSUN));
        assert_eq!(jup.source, LightSource::Cooling);
        assert!(
            jup.luminosity_lsun > 1.0e-10 && jup.luminosity_lsun < 1.0e-9,
            "{jup:?}"
        );
        assert!(jup.teff_k > 60.0 && jup.teff_k < 110.0, "{}", jup.teff_k);
        // A 20 Jupiter-mass brown dwarf at 1 Gyr is a T dwarf (500 to 1300 K).
        let bd = self_light(&f, 20.0 * MJ_MSUN, 1000.0, None);
        assert_eq!(bd.source, LightSource::Cooling);
        assert!(bd.teff_k > 500.0 && bd.teff_k < 1300.0, "{bd:?}");
        // Young, it burns deuterium.
        let young = self_light(&f, 20.0 * MJ_MSUN, 5.0, None);
        assert_eq!(young.source, LightSource::Deuterium);
    }

    #[test]
    fn no_seam_at_the_deuterium_limit() {
        let f = fits();
        let below = self_light(&f, BROWN_DWARF_MIN * 0.999, 3000.0, Some(0.1));
        let above = self_light(&f, BROWN_DWARF_MIN * 1.001, 3000.0, None);
        let ratio = above.luminosity_lsun / below.luminosity_lsun;
        assert!(ratio > 0.99 && ratio < 1.02, "{ratio}");
        assert_eq!(below.phase, "giant planet");
        assert_eq!(above.phase, "brown dwarf");
    }
}
