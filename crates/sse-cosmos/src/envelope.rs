//! P from a galaxy: patch envelopes and green regions (galaxy design
//! section 7, stage G4).
//!
//! G0 gave a fixed Milky-Way-like envelope. Here any galaxy from G3 (its
//! reservoir history and its rings) gives the envelope at any place and
//! time, computed and never stored:
//!
//! ```text
//! from the rings it overlaps:  gas, stars, abundances by group, star formation, death rates
//!                              (linear in radius between ring centres, and in time between steps,
//!                              so neighbouring patches agree at their shared face)
//! x spiral arm phase:          a log-spiral density wave with a pattern speed; in an arm the gas
//!                              and star formation are higher (Lin and Shu 1964; Roberts 1969)
//! x vertical position:         thin disc, thick disc and young stars, each with its scale height
//! x local texture:             a lognormal factor from keyed value noise on a lattice, smooth in space
//! ```
//!
//! Derived hazards: stellar density, encounter rate, supernovae within a
//! sterilising distance, long gamma-ray bursts within their lethal distance
//! (favoured at low metallicity), active-nucleus dose, and the time since the
//! gas passed the rocky-planet metallicity floor.
//!
//! The green regions are a classification over those (Lineweaver, Fenner and
//! Gibson 2004 made editable): enough metals for rocky planets, not so many
//! that migrating giant planets dominate, few sterilising events, few close
//! passages, and (for complex life) enough quiet time. Each criterion is a
//! pack setting; `green_band` answers "where, at this time" for a disc.

use crate::galaxy::Galaxy;
use crate::rings::{Disc, RingState};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, Seed};

/// Pack settings (`provides.other.cosmology.envelope`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EnvelopeParams {
    /// Scale heights, pc: gas and young stars, thin disc, thick disc.
    pub gas_height_pc: f64,
    pub thin_height_pc: f64,
    pub thick_height_pc: f64,
    /// Share of a ring's stars in the thick disc.
    pub thick_share: f64,
    /// Mean stellar mass, solar masses (turns mass density into systems).
    pub mean_star_msun: f64,
    /// Spiral: number of arms, pitch angle (degrees), pattern speed (km/s/kpc),
    /// arm contrast (gas and star formation x 1 + A cos(phase)), and the arm
    /// phase at the reference radius at t = 0 (radians).
    pub arms: u32,
    pub pitch_deg: f64,
    pub pattern_kms_kpc: f64,
    pub arm_contrast: f64,
    pub arm_phase0: f64,
    /// Flat rotation speed (km/s), for the arms' winding and encounter speeds.
    pub rotation_kms: f64,
    /// Local texture: lognormal sigma (dex) on gas and star formation, on a lattice of this spacing (pc).
    pub texture_sigma_dex: f64,
    pub texture_cell_pc: f64,
    /// Velocity dispersion of disc stars at the Sun's radius (km/s), rising inward as exp(-R / 2 R_d).
    pub sigma_kms: f64,
    /// Hazard distances: sterilising supernova (pc), close passage (pc), lethal gamma-ray burst (kpc).
    pub sterilising_pc: f64,
    pub encounter_pc: f64,
    pub grb_kpc: f64,
    /// Long gamma-ray bursts per solar mass formed at low metallicity, suppressed as
    /// 1 / (1 + (Z / Z_c)^2) (Graham and Fruchter 2013 find few above a third solar).
    pub grb_per_msun: f64,
    pub grb_z_crit: f64,
    /// Active-nucleus dose: black hole mass gained in the last `agn_window_gyr`, per 1e6
    /// solar masses, times (1 kpc / R)^2.
    pub agn_window_gyr: f64,
}

impl Default for EnvelopeParams {
    fn default() -> Self {
        EnvelopeParams {
            gas_height_pc: 100.0,
            thin_height_pc: 300.0,
            thick_height_pc: 900.0,
            thick_share: 0.12,
            mean_star_msun: 0.5,
            arms: 2,
            pitch_deg: 12.0,
            pattern_kms_kpc: 25.0,
            arm_contrast: 0.6,
            arm_phase0: 0.0,
            rotation_kms: 220.0,
            texture_sigma_dex: 0.2,
            texture_cell_pc: 50.0,
            sigma_kms: 35.0,
            sterilising_pc: 10.0,
            encounter_pc: 0.1,
            grb_kpc: 2.0,
            grb_per_msun: 1.6e-7,
            grb_z_crit: 0.3,
            agn_window_gyr: 1.0,
        }
    }
}

/// What makes a place green (`provides.other.cosmology.green`). A "Force" pack may change any of them.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GreenParams {
    /// Rocky planets need [Fe/H] at least this (and the time since it was reached is the quiet clock).
    pub feh_min: f64,
    /// Above this, migrating giant planets dominate (Fischer and Valenti 2005; Lineweaver 2001).
    pub feh_max: f64,
    /// Sterilising supernovae per Gyr within `sterilising_pc`, at most.
    pub sterilising_max_per_gyr: f64,
    /// Close passages per Myr within `encounter_pc`, at most.
    pub encounters_max_per_myr: f64,
    /// Lethal gamma-ray bursts per Gyr, at most.
    pub grb_max_per_gyr: f64,
    pub agn_dose_max: f64,
    /// Complex life needs this long since the rocky floor was passed (Lineweaver et al. 2004: 4 +- 1 Gyr).
    pub quiet_gyr: f64,
}

impl Default for GreenParams {
    fn default() -> Self {
        GreenParams {
            feh_min: -0.5,
            feh_max: 0.3,
            sterilising_max_per_gyr: 4.0,
            encounters_max_per_myr: 1.0,
            grb_max_per_gyr: 3.0,
            agn_dose_max: 1.0,
            quiet_gyr: 4.0,
        }
    }
}

/// Where a patch is in a galaxy: radius (kpc), azimuth (radians), height (pc), universe time (Gyr).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Place {
    pub r_kpc: f64,
    pub phi: f64,
    pub z_pc: f64,
    pub t_gyr: f64,
}

/// The envelope's numbers at a place.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Local {
    pub place: Place,
    /// Gas and stars, solar masses per pc^3; systems per pc^3.
    pub gas_pc3: f64,
    pub stars_pc3: f64,
    pub systems_pc3: f64,
    /// Gas abundances, [X/H] in dex.
    pub feh: f64,
    pub alpha_fe: f64,
    /// C/O by number, from the C+N and alpha budgets (solar 0.55, Asplund et al. 2009).
    pub c_o: f64,
    /// r-process and K-40 relative to iron, dex.
    pub r_fe: f64,
    pub k40_fe: f64,
    /// Star formation, solar masses per Gyr per pc^3.
    pub sfr_gyr_pc3: f64,
    /// Supernova rates per Gyr per pc^3.
    pub cc_gyr_pc3: f64,
    pub ia_gyr_pc3: f64,
    /// 1 + A cos(arm phase): above 1 in an arm.
    pub arm: f64,
    pub texture: f64,
    pub sterilising_per_gyr: f64,
    pub encounters_per_myr: f64,
    pub grb_per_gyr: f64,
    pub agn_dose: f64,
    /// Gyr since the gas here passed `GreenParams::feh_min`, if it has.
    pub rocky_for_gyr: Option<f64>,
}

/// Each green criterion, pass or fail, and the verdict.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Green {
    pub metals_enough: bool,
    pub metals_not_too_many: bool,
    pub few_supernovae: bool,
    pub few_passages: bool,
    pub few_bursts: bool,
    pub nucleus_quiet: bool,
    pub quiet_long_enough: bool,
    /// Every criterion but the quiet clock: simple life could start.
    pub habitable: bool,
    /// All of them: complex life has had time.
    pub complex: bool,
}

pub struct GalaxyEnvelope<'a> {
    pub galaxy: &'a Galaxy,
    pub disc: &'a Disc,
    pub params: EnvelopeParams,
    pub seed: Seed,
    pub address: Address,
}

fn lerp(a: f64, b: f64, w: f64) -> f64 {
    a + (b - a) * w
}

/// Interpolate two ring states field by field.
fn mix(a: &RingState, b: &RingState, w: f64) -> RingState {
    RingState {
        gas: lerp(a.gas, b.gas, w),
        stars: lerp(a.stars, b.stars, w),
        feh: lerp(a.feh, b.feh, w),
        alpha_h: lerp(a.alpha_h, b.alpha_h, w),
        cn_h: lerp(a.cn_h, b.cn_h, w),
        s_h: lerp(a.s_h, b.s_h, w),
        r_h: lerp(a.r_h, b.r_h, w),
        k40_h: lerp(a.k40_h, b.k40_h, w),
        sfr: lerp(a.sfr, b.sfr, w),
        cc_rate: lerp(a.cc_rate, b.cc_rate, w),
        ia_rate: lerp(a.ia_rate, b.ia_rate, w),
        nsm_so_far: if w < 0.5 { a.nsm_so_far } else { b.nsm_so_far },
    }
}

impl GalaxyEnvelope<'_> {
    fn area_pc2(&self, i: usize) -> f64 {
        let r = &self.disc.rings[i];
        m::PI * (r.outer_kpc * r.outer_kpc - r.inner_kpc * r.inner_kpc) * 1.0e6
    }

    /// A ring's state per unit area at time t (linear between steps).
    fn ring_at_time(&self, i: usize, t: f64) -> RingState {
        let st = &self.disc.states[i];
        let x = ((t - self.disc.t_start_gyr) / self.disc.step_gyr - 1.0)
            .clamp(0.0, (st.len() - 1) as f64);
        let k = (m::floor(x) as usize).min(st.len() - 1);
        let k1 = (k + 1).min(st.len() - 1);
        let mut s = mix(&st[k], &st[k1], x - k as f64);
        let a = self.area_pc2(i);
        s.gas /= a;
        s.stars /= a;
        s.sfr /= a;
        s.cc_rate /= a;
        s.ia_rate /= a;
        s
    }

    /// Surface densities and abundances at radius r and time t, linear in r between ring centres.
    pub fn surface(&self, r: f64, t: f64) -> RingState {
        let rings = &self.disc.rings;
        let i = rings.partition_point(|g| g.radius_kpc < r);
        if i == 0 {
            return self.ring_at_time(0, t);
        }
        if i >= rings.len() {
            return self.ring_at_time(rings.len() - 1, t);
        }
        let (a, b) = (rings[i - 1].radius_kpc, rings[i].radius_kpc);
        mix(
            &self.ring_at_time(i - 1, t),
            &self.ring_at_time(i, t),
            (r - a) / (b - a),
        )
    }

    /// 1 + A cos(m (phi - phi_arm)): a trailing log spiral, rotating at the pattern speed.
    pub fn arm(&self, p: &Place) -> f64 {
        let e = &self.params;
        let rd = self.galaxy.now.disc_kpc.max(0.1);
        let tan_pitch = m::tan(e.pitch_deg * m::PI / 180.0);
        let omega_p = e.pattern_kms_kpc * 1.0227; // radians per Gyr
        let phase = f64::from(e.arms)
            * (p.phi
                - omega_p * p.t_gyr
                - m::ln(p.r_kpc.max(0.01) / rd) / tan_pitch
                - e.arm_phase0);
        1.0 + e.arm_contrast * m::cos(phase)
    }

    /// Lognormal texture from keyed value noise on a lattice (trilinear, so smooth across patches).
    pub fn texture(&self, p: &Place) -> f64 {
        let e = &self.params;
        if e.texture_sigma_dex <= 0.0 {
            return 1.0;
        }
        let d = Draws::new(
            self.seed,
            self.address.clone(),
            Stream::new("cosmos.envelope.texture.v1").expect("stream"),
        );
        let pc = p.r_kpc * 1000.0;
        let (x, y, z) = (
            pc * m::cos(p.phi) / e.texture_cell_pc,
            pc * m::sin(p.phi) / e.texture_cell_pc,
            p.z_pc / e.texture_cell_pc,
        );
        let (fx, fy, fz) = (m::floor(x), m::floor(y), m::floor(z));
        let (wx, wy, wz) = (x - fx, y - fy, z - fz);
        let key = |i: f64, j: f64, k: f64| -> f64 {
            // Lattice index to a draw index: offset to positive and pack into 20 bits per axis
            // (normal(i) reads draws 2i and 2i + 1, so the index stays below 2^62).
            let w = |v: f64| ((v as i64 + (1 << 19)) as u64) & 0xf_ffff;
            d.normal(w(i) | (w(j) << 20) | (w(k) << 40))
        };
        let mut v = 0.0;
        for (di, wi) in [(0.0, 1.0 - wx), (1.0, wx)] {
            for (dj, wj) in [(0.0, 1.0 - wy), (1.0, wy)] {
                for (dk, wk) in [(0.0, 1.0 - wz), (1.0, wz)] {
                    v += wi * wj * wk * key(fx + di, fy + dj, fz + dk);
                }
            }
        }
        m::pow(10.0, e.texture_sigma_dex * v)
    }

    /// The envelope at a place.
    pub fn local(&self, p: Place, g: &GreenParams) -> Local {
        let e = &self.params;
        let s = self.surface(p.r_kpc, p.t_gyr);
        let arm = self.arm(&p);
        let tex = self.texture(&p);
        let vz = |h: f64| m::exp(-p.z_pc.abs() / h) / (2.0 * h);
        let gas = s.gas * vz(e.gas_height_pc) * arm * tex;
        let stars = s.stars
            * ((1.0 - e.thick_share) * vz(e.thin_height_pc)
                + e.thick_share * vz(e.thick_height_pc));
        let sfr = s.sfr * 1.0e9 * vz(e.gas_height_pc) * arm * tex;
        let cc = s.cc_rate * 1.0e9 * vz(e.gas_height_pc) * arm * tex;
        let ia = s.ia_rate
            * 1.0e9
            * ((1.0 - e.thick_share) * vz(e.thin_height_pc)
                + e.thick_share * vz(e.thick_height_pc));
        let d = e.sterilising_pc;
        let sterilising = (cc + ia) * 4.0 / 3.0 * m::PI * d * d * d;
        let rd = self.galaxy.now.disc_kpc.max(0.1);
        let sigma = e.sigma_kms * m::exp(-(p.r_kpc - 3.15 * rd) / (2.0 * rd));
        let n = stars / e.mean_star_msun;
        let v_pc_myr = m::sqrt(2.0) * sigma * 1.022_712;
        let encounters = n * m::PI * e.encounter_pc * e.encounter_pc * v_pc_myr;
        // Lethal bursts: within grb_kpc (wider than the disc is thick, so a surface integral).
        let zrel = m::pow(10.0, s.feh);
        let grb = m::PI * e.grb_kpc * e.grb_kpc * 1.0e6 * s.sfr * 1.0e9 * e.grb_per_msun
            / (1.0 + (zrel / e.grb_z_crit) * (zrel / e.grb_z_crit));
        let agn: f64 = self
            .galaxy
            .active_nuclei
            .iter()
            .filter(|(t, _)| *t <= p.t_gyr && *t > p.t_gyr - e.agn_window_gyr)
            .map(|(_, gain)| gain / 1.0e6)
            .sum::<f64>()
            / (p.r_kpc * p.r_kpc).max(0.01);
        Local {
            place: p,
            gas_pc3: gas,
            stars_pc3: stars,
            systems_pc3: n,
            feh: s.feh,
            alpha_fe: s.alpha_h - s.feh,
            c_o: 0.55 * m::pow(10.0, s.cn_h - s.alpha_h),
            r_fe: s.r_h - s.feh,
            k40_fe: s.k40_h - s.feh,
            sfr_gyr_pc3: sfr,
            cc_gyr_pc3: cc,
            ia_gyr_pc3: ia,
            arm,
            texture: tex,
            sterilising_per_gyr: sterilising,
            encounters_per_myr: encounters,
            grb_per_gyr: grb,
            agn_dose: agn,
            rocky_for_gyr: self
                .reached(p.r_kpc, g.feh_min)
                .and_then(|at| (at <= p.t_gyr).then_some(p.t_gyr - at)),
        }
    }

    /// When the gas at radius r first reached [Fe/H] = feh, if ever (step resolution, interpolated).
    pub fn reached(&self, r: f64, feh: f64) -> Option<f64> {
        let n = self.disc.states.first().map_or(0, Vec::len);
        let mut prev: Option<(f64, f64)> = None;
        for k in 0..n {
            let t = self.disc.time(k + 1);
            let f = self.surface(r, t).feh;
            if f >= feh {
                return Some(match prev {
                    Some((tp, fp)) if f > fp => tp + (t - tp) * (feh - fp) / (f - fp),
                    _ => t,
                });
            }
            prev = Some((t, f));
        }
        None
    }

    /// The green classification at a place.
    pub fn green(&self, l: &Local, g: &GreenParams) -> Green {
        let metals_enough = l.feh >= g.feh_min;
        let metals_not_too_many = l.feh <= g.feh_max;
        let few_supernovae = l.sterilising_per_gyr <= g.sterilising_max_per_gyr;
        let few_passages = l.encounters_per_myr <= g.encounters_max_per_myr;
        let few_bursts = l.grb_per_gyr <= g.grb_max_per_gyr;
        let nucleus_quiet = l.agn_dose <= g.agn_dose_max;
        let quiet_long_enough = l.rocky_for_gyr.is_some_and(|q| q >= g.quiet_gyr);
        let habitable = metals_enough
            && metals_not_too_many
            && few_supernovae
            && few_passages
            && few_bursts
            && nucleus_quiet;
        Green {
            metals_enough,
            metals_not_too_many,
            few_supernovae,
            few_passages,
            few_bursts,
            nucleus_quiet,
            quiet_long_enough,
            habitable,
            complex: habitable && quiet_long_enough,
        }
    }

    /// The green band at time t, on the midplane and azimuthally averaged (arms and texture
    /// off): the inner and outer radius of the widest run of `complex` (or `habitable`) radii,
    /// scanned at `step_kpc`.
    pub fn green_band(
        &self,
        t: f64,
        g: &GreenParams,
        complex: bool,
        step_kpc: f64,
    ) -> Option<(f64, f64)> {
        let smooth = GalaxyEnvelope {
            galaxy: self.galaxy,
            disc: self.disc,
            params: EnvelopeParams {
                arm_contrast: 0.0,
                texture_sigma_dex: 0.0,
                ..self.params.clone()
            },
            seed: self.seed,
            address: self.address.clone(),
        };
        let outer = self.disc.rings.last().map_or(0.0, |r| r.outer_kpc);
        let mut best: Option<(f64, f64)> = None;
        let mut run: Option<f64> = None;
        let mut r = step_kpc;
        while r <= outer + 1.0e-9 {
            let l = smooth.local(
                Place {
                    r_kpc: r,
                    phi: 0.0,
                    z_pc: 0.0,
                    t_gyr: t,
                },
                g,
            );
            let gr = smooth.green(&l, g);
            let ok = if complex { gr.complex } else { gr.habitable };
            match (ok, run) {
                (true, None) => run = Some(r),
                (false, Some(a)) => {
                    let cand = (a, r - step_kpc);
                    if best.is_none_or(|b| cand.1 - cand.0 > b.1 - b.0) {
                        best = Some(cand);
                    }
                    run = None;
                }
                _ => {}
            }
            r += step_kpc;
        }
        if let Some(a) = run {
            let cand = (a, r - step_kpc);
            if best.is_none_or(|b| cand.1 - cand.0 > b.1 - b.0) {
                best = Some(cand);
            }
        }
        best
    }

    /// Outcome first at galaxy scale: up to `n` places at time t where complex life has had
    /// time, spread round the band and lifted off the midplane by a keyed draw, best (lowest
    /// sterilising rate) first.
    pub fn find_green(&self, t: f64, g: &GreenParams, n: u32) -> Vec<(Local, Green)> {
        let Some((a, b)) = self.green_band(t, g, true, 0.25) else {
            return vec![];
        };
        let d = Draws::new(
            self.seed,
            self.address.clone(),
            Stream::new("cosmos.envelope.find.v1").expect("stream"),
        );
        let mut out = vec![];
        for i in 0..u64::from(n) * 4 {
            let p = Place {
                r_kpc: a + (b - a) * d.unit(3 * i),
                phi: m::TAU * d.unit(3 * i + 1),
                z_pc: (d.unit(3 * i + 2) - 0.5) * 200.0,
                t_gyr: t,
            };
            let l = self.local(p, g);
            let gr = self.green(&l, g);
            if gr.complex {
                out.push((l, gr));
            }
        }
        out.sort_by(|x, y| x.0.sterilising_per_gyr.total_cmp(&y.0.sterilising_per_gyr));
        out.truncate(n as usize);
        out
    }
}

impl GalaxyEnvelope<'_> {
    /// Radius scale against the Milky Way (R_d = 2.6 kpc), so "the Sun's radius" means
    /// the same place in any disc: 8.2 kpc x R_d / 2.6.
    pub fn sun_radius_kpc(&self) -> f64 {
        8.2 * self.galaxy.now.disc_kpc / 2.6
    }

    /// Least-squares slope of present gas [Fe/H] with radius from 0.5 to 1.7 Sun radii, dex per kpc.
    pub fn gradient(&self, t: f64) -> f64 {
        let r0 = self.sun_radius_kpc();
        let pts: Vec<(f64, f64)> = (0..=24)
            .map(|i| {
                let r = r0 * (0.5 + 1.2 * f64::from(i) / 24.0);
                (r, self.surface(r, t).feh)
            })
            .collect();
        let n = pts.len() as f64;
        let (sx, sy) = (
            pts.iter().map(|p| p.0).sum::<f64>() / n,
            pts.iter().map(|p| p.1).sum::<f64>() / n,
        );
        pts.iter().map(|p| (p.0 - sx) * (p.1 - sy)).sum::<f64>()
            / pts.iter().map(|p| (p.0 - sx) * (p.0 - sx)).sum::<f64>()
    }

    /// G0's patch model with its enrichment, alpha and star formation taken from this galaxy,
    /// so the existing patch, cohort and system machinery forms systems from a G3 galaxy.
    /// The enrichment clock tau is fitted so the Sun's ring matches at its birth (4.57 Gyr ago) and now.
    pub fn patch_model(&self, base: &crate::PatchModel) -> crate::PatchModel {
        let mut pm = base.clone();
        let now = self.galaxy.now.t_gyr;
        let r0 = self.sun_radius_kpc();
        let s_now = self.surface(r0, now);
        let then = self.surface(r0, now - 4.57).feh;
        let start = self.disc.t_start_gyr;
        let want = m::pow(10.0, then - s_now.feh);
        let ratio = |tau: f64| {
            (1.0 - m::exp(-(now - 4.57 - start) / tau)) / (1.0 - m::exp(-(now - start) / tau))
        };
        // ratio falls as tau grows: bisect on log tau over 0.1 to 100 Gyr.
        let (mut lo, mut hi) = (m::ln(0.1), m::ln(100.0));
        for _ in 0..60 {
            let mid = 0.5 * (lo + hi);
            if ratio(m::exp(mid)) > want {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        pm.id = format!("{}+galaxy", base.id);
        pm.now_gyr = crate::patch::tidy(now);
        pm.galaxy_start_gyr = crate::patch::tidy(start);
        pm.sun_radius_kpc = crate::patch::tidy(r0);
        pm.enrichment.feh_now_at_sun = crate::patch::tidy(s_now.feh);
        pm.enrichment.gradient_dex_per_kpc = crate::patch::tidy(self.gradient(now));
        pm.enrichment.tau_gyr = crate::patch::tidy(m::exp(0.5 * (lo + hi)));
        pm.alpha_fe.now = crate::patch::tidy(s_now.alpha_h - s_now.feh);
        pm.sfr.galaxy_msun_per_yr = crate::patch::tidy(self.galaxy.now.sfr);
        pm.sfr.scale_length_kpc = crate::patch::tidy(self.galaxy.now.disc_kpc);
        pm
    }
}
