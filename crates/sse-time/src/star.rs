//! The ageing star: F0's life line (luminosity, radius, phase, remnant) read
//! at any age, plus what the life line leaves out and ageing needs: mass
//! lost in winds, the core mass, and the colour of the light.
//!
//! The life line is reused, not duplicated. Mass loss is a layer on top:
//! Reimers' wind integrated over the life line's giant phase with fixed
//! steps, the rest of the envelope thrown off at the end of the giant phase
//! (the planetary nebula). F7 replaces the giant phase with tables; this
//! layer then reads them unchanged.

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_formation::hurley::Fits;
use sse_formation::lifeline::{teff, LifeLine, Phase, StarState};

/// Giant-branch core mass-luminosity constant, L_sun per Msun^6.
pub const CORE_LUM_K: f64 = 2.3e5;

/// Pack settings for the ageing star (`provides.other.time.star`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct StarSettings {
    /// Reimers' efficiency.
    pub reimers_eta: f64,
    /// Fixed steps of the wind integration across the giant phase.
    pub wind_steps: u32,
    /// Core mass at the base of the giant branch, as a fraction of the remnant.
    pub core_start_fraction: f64,
}

/// One row of the mass table.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct MassRow {
    /// Age from birth, Myr.
    pub t: f64,
    pub mass: f64,
}

/// A star through its whole life.
#[derive(Clone, Debug)]
pub struct AgeingStar {
    pub fits: Fits,
    pub line: LifeLine,
    pub settings: StarSettings,
    /// Mass against age across the giant phase (fixed steps).
    pub masses: Vec<MassRow>,
}

/// The state ageing reads at one age.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AgedState {
    pub star: StarState,
    /// Current mass after winds, solar masses.
    pub mass: f64,
    /// Helium or carbon-oxygen core, solar masses.
    pub core_mass: f64,
}

impl AgeingStar {
    pub fn new(mass: f64, z: f64, settings: StarSettings) -> Result<Self, String> {
        let fits = Fits::new(z);
        let line = LifeLine::new(&fits, mass).map_err(|e| e.0)?;
        let mut s = AgeingStar {
            fits,
            line,
            settings,
            masses: Vec::new(),
        };
        s.masses = s.integrate_wind();
        Ok(s)
    }

    /// Does the core clock drive this star's giant phase? Low-mass stars
    /// with a degenerate helium core (tip at 2500 L_sun) that end as white
    /// dwarfs.
    pub fn clocked(&self) -> bool {
        let l = &self.line;
        l.remnant == Phase::WhiteDwarf && l.l_tip > l.l_bgb && l.t_end > l.t_bgb && l.l_bgb > 0.0
    }

    /// Core mass for a giant luminosity: L = 2.3e5 Mc^6 (Boothroyd and
    /// Sackmann 1988; Refsdal and Weigert 1970).
    fn core_for(l: f64) -> f64 {
        m::pow(l / CORE_LUM_K, 1.0 / 6.0)
    }

    /// Fraction `s` of the giant phase (by age) reached when the core has
    /// grown to `mc`. The core grows by burning hydrogen at the luminosity it
    /// sets, dMc/dt proportional to L proportional to Mc^6, so Mc^-5 falls
    /// linearly with age: most of the time is spent faint, the climb to the
    /// tip is fast (the real shape of a red giant branch).
    fn clock_fraction(&self, mc: f64) -> f64 {
        let l = &self.line;
        let (c0, c1) = (Self::core_for(l.l_bgb), Self::core_for(l.l_tip));
        let (x0, x1) = (m::pow(c0, -5.0), m::pow(c1, -5.0));
        ((x0 - m::pow(mc, -5.0)) / (x0 - x1)).clamp(0.0, 1.0)
    }

    /// Core mass at giant-phase fraction `s` (by age).
    fn clock_core(&self, s: f64) -> f64 {
        let l = &self.line;
        let (c0, c1) = (Self::core_for(l.l_bgb), Self::core_for(l.l_tip));
        let (x0, x1) = (m::pow(c0, -5.0), m::pow(c1, -5.0));
        m::pow(x0 - s * (x0 - x1), -0.2)
    }

    /// Age at which the giant's luminosity has climbed a fraction `u` of the
    /// way from base to tip in log L (used to place steps evenly in brightness).
    pub fn age_at_giant_fraction(&self, u: f64) -> f64 {
        let l = &self.line;
        let lum = l.l_bgb * m::pow(l.l_tip / l.l_bgb, u);
        let s = if self.clocked() {
            self.clock_fraction(Self::core_for(lum))
        } else {
            u
        };
        l.t_bgb + s * (l.t_end - l.t_bgb)
    }

    /// The star at age `t` with current mass `mass`. On the main sequence
    /// and in the remnant this is the life line. On a clocked giant branch,
    /// the luminosity comes from the core clock, the life line is read at the
    /// age where it has that luminosity, and the radius comes from the HPT00
    /// giant-branch law at the current (wind-reduced) mass, as HPT00 does.
    fn star_with(&self, t: f64, mass: f64) -> StarState {
        let l = &self.line;
        if !(self.clocked() && l.phase_at(t) == Phase::Giant) {
            return l.state_at(&self.fits, t);
        }
        let s = (t - l.t_bgb) / (l.t_end - l.t_bgb);
        let lum = CORE_LUM_K * m::pow(self.clock_core(s), 6.0);
        let u = m::ln(lum / l.l_bgb) / m::ln(l.l_tip / l.l_bgb);
        let mut st = l.state_at(
            &self.fits,
            l.t_bgb + u.clamp(0.0, 1.0) * (l.t_end - l.t_bgb),
        );
        st.luminosity = lum;
        st.radius = self.fits.r_gb(mass, lum).max(l.r_tms);
        st.teff = teff(lum, st.radius);
        st.mass = mass;
        st
    }

    /// Reimers 1975: dM/dt = -4e-13 eta L R / M solar masses per year (L, R,
    /// M solar), integrated from the end of the main sequence to the end of
    /// the giant phase in fixed midpoint steps. The steps are even in the
    /// core clock's luminosity, so the fast climb to the tip gets most of them.
    fn integrate_wind(&self) -> Vec<MassRow> {
        let l = &self.line;
        let n = self.settings.wind_steps.max(1);
        let mut out = vec![MassRow {
            t: l.t_tms,
            mass: l.initial_mass,
        }];
        if l.t_end <= l.t_tms || l.remnant != Phase::WhiteDwarf {
            return out;
        }
        let mut ts = vec![l.t_tms, l.t_bgb];
        for k in 1..n {
            ts.push(self.age_at_giant_fraction(f64::from(k) / f64::from(n)));
        }
        ts.push(l.t_end);
        let mut mass = l.initial_mass;
        for w in ts.windows(2) {
            let (a, b) = (w[0], w[1]);
            if b <= a {
                continue;
            }
            let mid = 0.5 * (a + b);
            let st = self.star_with(mid, mass);
            let rate = 4.0e-13 * self.settings.reimers_eta * st.luminosity * st.radius / mass;
            let floor = self.core_at(b).max(l.remnant_mass);
            mass = (mass - rate * (b - a) * 1.0e6).max(floor);
            out.push(MassRow { t: b, mass });
        }
        out
    }

    /// Core mass: on a clocked giant branch from the core clock; otherwise
    /// a fraction of the remnant at the base of the giant branch growing
    /// linearly to the remnant (sketch). The remnant at the end.
    pub fn core_at(&self, t: f64) -> f64 {
        let l = &self.line;
        if t >= l.t_end {
            return l.remnant_mass;
        }
        if self.clocked() {
            let s = ((t - l.t_bgb) / (l.t_end - l.t_bgb)).clamp(0.0, 1.0);
            return self.clock_core(s).min(l.remnant_mass);
        }
        let start = self.settings.core_start_fraction * l.remnant_mass;
        if t <= l.t_bgb {
            start.min(l.remnant_mass)
        } else {
            start + (l.remnant_mass - start) * (t - l.t_bgb) / (l.t_end - l.t_bgb)
        }
    }

    /// Mass after winds at age `t` Myr.
    pub fn mass_at(&self, t: f64) -> f64 {
        let l = &self.line;
        if t >= l.t_end {
            return l.remnant_mass;
        }
        if t <= l.t_tms || self.masses.len() < 2 {
            return l.initial_mass;
        }
        let i = self
            .masses
            .partition_point(|r| r.t <= t)
            .clamp(1, self.masses.len() - 1);
        let (a, b) = (self.masses[i - 1], self.masses[i]);
        a.mass + (b.mass - a.mass) * (t - a.t) / (b.t - a.t)
    }

    pub fn state_at(&self, t: f64) -> AgedState {
        let mass = self.mass_at(t);
        AgedState {
            star: self.star_with(t, mass),
            mass,
            core_mass: self.core_at(t),
        }
    }
}

/// Colour bands by effective temperature: (lower bound K, name). Pack data.
pub fn colour_name(bands: &[(f64, String)], teff: f64) -> String {
    bands
        .iter()
        .find(|(lo, _)| teff >= *lo)
        .map_or("red".into(), |(_, n)| n.clone())
}

/// sRGB of a black body at `teff`, from Helland's fit to the CIE blackbody
/// locus (1000 to 40000 K), 0 to 255 per channel.
pub fn blackbody_rgb(teff: f64) -> [u8; 3] {
    let t = teff.clamp(1000.0, 40000.0) / 100.0;
    let r = if t <= 66.0 {
        255.0
    } else {
        329.698_727_446 * m::pow(t - 60.0, -0.133_204_759_2)
    };
    let g = if t <= 66.0 {
        99.470_802_586_1 * m::ln(t) - 161.119_568_166_1
    } else {
        288.122_169_528_3 * m::pow(t - 60.0, -0.075_514_849_2)
    };
    let b = if t >= 66.0 {
        255.0
    } else if t <= 19.0 {
        0.0
    } else {
        138.517_731_223_1 * m::ln(t - 10.0) - 305.044_792_730_7
    };
    let c = |x: f64| m::round(x.clamp(0.0, 255.0)) as u8;
    [c(r), c(g), c(b)]
}
