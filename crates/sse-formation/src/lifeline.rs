//! A star's whole life line, drawn once (H6 rule 6): every date is a read of it.
//!
//! Main sequence: Hurley, Pols and Tout 2000 (calculated, "standard" level).
//! Pre-main sequence, Hertzsprung gap, giant phases: simple rules on the same
//! fits ("sketch" level, replaced by F7's tabulated tracks and binary rules).
//! Remnants: published initial-final mass relation and cooling laws.

use crate::hurley::Fits;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Solar effective temperature, K (IAU 2015 B3 nominal).
pub const TEFF_SUN: f64 = 5772.0;
/// Chandrasekhar mass used by the white-dwarf radius law (HPT00).
pub const M_CH: f64 = 1.44;
/// Stars: the hydrogen-burning limit (about 0.075, Chabrier et al. 2000) to
/// 100. HPT00 fit 0.1 to 100; 0.08 to 0.1 is a short, labelled extrapolation.
pub const MASS_RANGE: (f64, f64) = (0.08, 100.0);
/// Brown dwarfs: the deuterium-burning limit (13 Jupiter masses) up to the stars.
pub const BROWN_DWARF_MIN: f64 = 0.0124;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Phase {
    PreMainSequence,
    MainSequence,
    HertzsprungGap,
    Giant,
    /// Below the hydrogen-burning limit: never a main-sequence star.
    BrownDwarf,
    WhiteDwarf,
    NeutronStar,
    BlackHole,
}

impl Phase {
    pub fn is_remnant(self) -> bool {
        matches!(
            self,
            Phase::WhiteDwarf | Phase::NeutronStar | Phase::BlackHole
        )
    }
    pub fn label(self) -> &'static str {
        match self {
            Phase::PreMainSequence => "pre-main-sequence",
            Phase::MainSequence => "main sequence",
            Phase::HertzsprungGap => "subgiant",
            Phase::Giant => "giant",
            Phase::BrownDwarf => "brown dwarf",
            Phase::WhiteDwarf => "white dwarf",
            Phase::NeutronStar => "neutron star",
            Phase::BlackHole => "black hole",
        }
    }
}

/// How a value was obtained, and at which fidelity level (design section 4).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    Sketch,
    Standard,
    Detailed,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StarState {
    pub phase: Phase,
    /// Solar luminosities.
    pub luminosity: f64,
    /// Solar radii.
    pub radius: f64,
    /// Kelvin.
    pub teff: f64,
    /// Solar masses (no wind mass loss before the remnant at this level).
    pub mass: f64,
    pub level: Level,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct LifeLine {
    pub initial_mass: f64,
    pub z: f64,
    /// Birth to zero-age main sequence (Kelvin-Helmholtz time), Myr.
    pub t_zams: f64,
    /// Ages from birth, Myr.
    pub t_tms: f64,
    pub t_bgb: f64,
    pub t_end: f64,
    pub remnant: Phase,
    pub remnant_mass: f64,
    /// Characteristic values kept for explain text and tables.
    pub l_zams: f64,
    pub r_zams: f64,
    pub l_tms: f64,
    pub r_tms: f64,
    pub l_bgb: f64,
    pub l_tip: f64,
}

/// Sketch: giant phases (red giant branch, helium burning, asymptotic giant
/// branch) last this fraction of the main-sequence life (HPT00 figure 4 gives
/// about 0.06 for 1 solar mass to about 0.1 for massive stars).
pub const GIANT_FRACTION: f64 = 0.1;
/// Sketch: luminosity at the tip of the giant phases for stars with a
/// degenerate helium core (red giant branch tip, about 2500 L_sun).
pub const L_TIP_LOW_MASS: f64 = 2500.0;
/// Upper initial mass for white dwarfs (HPT00 `Mup` is about 6.1 + 1.0 log Z/0.02
/// for carbon ignition; 8 solar masses is the usual observed limit).
pub const M_UP_WD: f64 = 8.0;
/// Sketch: initial mass above which the remnant is a black hole.
pub const M_BH: f64 = 25.0;

/// Cummings et al. 2018 (ApJ 866, 21) initial-final mass relation, MIST-based
/// fit, three pieces; extrapolated below 0.87 solar masses (those stars have
/// not died yet anywhere in the universe).
pub fn white_dwarf_mass(mi: f64) -> f64 {
    if mi < 2.8 {
        0.080 * mi + 0.489
    } else if mi < 3.65 {
        0.187 * mi + 0.184
    } else {
        0.107 * mi + 0.471
    }
}

/// Error for masses outside the fits.
#[derive(Clone, PartialEq, Debug)]
pub struct OutOfRange(pub String);

impl LifeLine {
    pub fn new(fits: &Fits, mass: f64) -> Result<Self, OutOfRange> {
        if (BROWN_DWARF_MIN..MASS_RANGE.0).contains(&mass) {
            return Ok(Self::brown_dwarf(fits, mass));
        }
        if !(MASS_RANGE.0..=MASS_RANGE.1).contains(&mass) {
            return Err(OutOfRange(format!(
                "{mass} solar masses is outside the stars and brown dwarfs (0.0124 to 100); a planetary-mass object is placed by later stages"
            )));
        }
        let (l_zams, r_zams) = (fits.l_zams(mass), fits.r_zams(mass));
        // Kelvin-Helmholtz time G M^2 / (R L) = 31.4 Myr M^2 / (R L) in solar units.
        let t_zams = 31.4 * mass * mass / (r_zams * l_zams);
        let t_ms = fits.t_ms(mass);
        let t_bgb_ms = fits.t_bgb(mass).max(t_ms);
        let t_tms = t_zams + t_ms;
        let t_bgb = t_zams + t_bgb_ms;
        let t_end = t_bgb + GIANT_FRACTION * t_ms;
        let low = mass <= fits.zpars[3];
        let l_tms = fits.l_tms(mass);
        let l_bgb = if low {
            fits.l_bgb(mass).max(l_tms)
        } else {
            l_tms
        };
        let l_tip = if mass <= fits.zpars[2] {
            L_TIP_LOW_MASS.max(l_bgb)
        } else {
            10.0 * l_bgb
        };
        let (remnant, remnant_mass) = if mass < M_UP_WD {
            (Phase::WhiteDwarf, white_dwarf_mass(mass).min(mass))
        } else if mass < M_BH {
            (Phase::NeutronStar, 1.4)
        } else {
            (Phase::BlackHole, (0.25 * mass).max(3.0))
        };
        Ok(LifeLine {
            initial_mass: mass,
            z: fits.z,
            t_zams,
            t_tms,
            t_bgb,
            t_end,
            remnant,
            remnant_mass,
            l_zams,
            r_zams,
            l_tms,
            r_tms: fits.r_tms(mass),
            l_bgb,
            l_tip,
        })
    }

    fn brown_dwarf(fits: &Fits, mass: f64) -> Self {
        let never = f64::MAX;
        LifeLine {
            initial_mass: mass,
            z: fits.z,
            t_zams: never,
            t_tms: never,
            t_bgb: never,
            t_end: never,
            remnant: Phase::BrownDwarf,
            remnant_mass: mass,
            l_zams: 0.0,
            r_zams: 0.1,
            l_tms: 0.0,
            r_tms: 0.1,
            l_bgb: 0.0,
            l_tip: 0.0,
        }
    }

    pub fn is_substellar(&self) -> bool {
        self.remnant == Phase::BrownDwarf
    }

    /// Phase boundaries from birth, Myr: where things change fast.
    pub fn boundaries(&self) -> [f64; 4] {
        [self.t_zams, self.t_tms, self.t_bgb, self.t_end]
    }

    pub fn phase_at(&self, t: f64) -> Phase {
        if self.is_substellar() {
            Phase::BrownDwarf
        } else if t < self.t_zams {
            Phase::PreMainSequence
        } else if t < self.t_tms {
            Phase::MainSequence
        } else if t < self.t_bgb {
            Phase::HertzsprungGap
        } else if t < self.t_end {
            Phase::Giant
        } else {
            self.remnant
        }
    }

    /// State at age `t` Myr from birth. Pure: the same call gives the same
    /// answer on every device, at any time, in any order.
    pub fn state_at(&self, fits: &Fits, t: f64) -> StarState {
        let mass = self.initial_mass;
        let phase = self.phase_at(t);
        let (l, r, level, mm) = match phase {
            Phase::PreMainSequence => (self.l_zams, self.r_zams, Level::Sketch, mass),
            Phase::MainSequence => {
                let (l, r) = fits
                    .ms_state(mass, t - self.t_zams)
                    .unwrap_or((self.l_tms, self.r_tms));
                (l, r, Level::Standard, mass)
            }
            Phase::HertzsprungGap => {
                // HPT00 eqs. 26-27: geometric from TMS to BGB.
                let tau = (t - self.t_tms) / (self.t_bgb - self.t_tms);
                let l = self.l_tms * m::pow(self.l_bgb / self.l_tms, tau);
                let rg = fits.r_gb(mass, self.l_bgb).max(self.r_tms);
                let r = self.r_tms * m::pow(rg / self.r_tms, tau);
                (l, r, Level::Sketch, mass)
            }
            Phase::Giant => {
                let tau = (t - self.t_bgb) / (self.t_end - self.t_bgb);
                let l = self.l_bgb * m::pow(self.l_tip / self.l_bgb, tau);
                let r = fits.r_gb(mass, l).max(self.r_tms);
                (l, r, Level::Sketch, mass)
            }
            Phase::BrownDwarf => {
                // Burrows et al. 2001 (Rev. Mod. Phys. 73, 719) eq. 3: cooling
                // with age; radius about 0.1 R_sun (sketch).
                let gyr = (t / 1000.0).max(0.001);
                let l = 4.0e-5 * m::pow(gyr, -1.3) * m::pow(mass / 0.05, 2.64);
                (l, 0.1, Level::Sketch, mass)
            }
            Phase::WhiteDwarf => {
                // HPT00 eqs. 90-91: Mestel cooling, A = 4 for carbon-oxygen;
                // radius from the Nauenberg-type law, floored at a neutron star.
                let age = (t - self.t_end).max(0.0);
                let mr = self.remnant_mass;
                let l = 635.0 * mr * m::pow(self.z, 0.4) / m::pow(4.0 * (age + 0.1), 1.4);
                let x = m::pow(M_CH / mr, 2.0 / 3.0) - m::pow(mr / M_CH, 2.0 / 3.0);
                let r = (0.0115 * m::sqrt(x.max(0.0))).max(1.4e-5);
                (l, r, Level::Standard, mr)
            }
            Phase::NeutronStar => {
                // HPT00 eq. 93, radius 10 km.
                let age = (t - self.t_end).max(0.1);
                let mr = self.remnant_mass;
                (
                    0.02 * m::pow(mr, 2.0 / 3.0) / (age * age),
                    1.4e-5,
                    Level::Sketch,
                    mr,
                )
            }
            Phase::BlackHole => {
                let mr = self.remnant_mass;
                // Schwarzschild radius 2GM/c^2 = 4.24e-6 R_sun per solar mass.
                (1.0e-10, 4.24e-6 * mr, Level::Sketch, mr)
            }
        };
        StarState {
            phase,
            luminosity: l,
            radius: r,
            teff: teff(l, r),
            mass: mm,
            level,
        }
    }
}

/// Effective temperature from luminosity and radius (solar units).
pub fn teff(l: f64, r: f64) -> f64 {
    TEFF_SUN * m::pow(l / (r * r), 0.25)
}
