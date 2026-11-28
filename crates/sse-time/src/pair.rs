//! BX1: a pair of stars aged together, in one window, as one dated line.
//!
//! Track BX (status `handover/status/binaries.md`), Alex's rulings of 26 Nov:
//! any number of stars as nested pairs of mass centres (ruling 3), what one
//! star gives the other taken from its own budget. F7's rule table
//! (`binary.rs`) told the story of a contact but carried nothing forward;
//! here both stars keep the masses they really have, step by step:
//!
//! - each star reads its own life line (`star.rs`) at its own effective age,
//!   with its own winds; the orbit widens as winds carry mass off (a·M kept,
//!   Jeans 1924);
//! - each step the donor's radius is set against its Roche lobe
//!   (`sse.orbit.roche_lobe`, Eggleton 1983) at periastron;
//! - at first contact the critical mass ratio (Hurley, Tout and Pols 2002,
//!   `binary::q_crit`) decides: stable transfer, or a common envelope
//!   (Webbink 1984) or a merger as F7 does;
//! - stable transfer runs at the overflow rate (`sse.orbit.overflow_rate`),
//!   no faster than the donor's thermal timescale lets its envelope go
//!   (`sse.star.thermal_timescale`); the accretor takes what its own thermal
//!   timescale allows and the rest leaves the pair from beside it, carrying
//!   its orbital angular momentum; the orbit follows the angular momentum
//!   (circularised at first contact, as tides do);
//! - a main-sequence star that gains or loses mass is rebuilt at its new mass
//!   keeping its share of the main sequence (an accretor's share falls as its
//!   core mixes in fresh fuel: Hurley, Tout and Pols 2002 section 7.1);
//!   a giant keeps its core and its line, its envelope the mass it has; a
//!   donor stripped to its core becomes a white dwarf of that core;
//! - a planet round either star is swallowed when the star swells past it;
//! - BX3: gravitational waves (`sse.orbit.gw_decay`, Peters 1964) and, for a
//!   tidally locked main-sequence star with a convective envelope over a
//!   radiative core, magnetic braking (`sse.star.magnetic_braking`) drain the
//!   orbit; a red or brown dwarf so brought into touch with a white dwarf
//!   gives just what keeps it at its lobe (a cataclysmic variable), and a
//!   main-sequence donor onto a compact star has its own critical ratio.
//!
//! A node is a star or a pair of nodes; a pair inside a pair acts one level
//! up as one body at its centre of mass, its mass the sum it has at each
//! moment (computed, never stored). Only two stars side by side trade gas in
//! this step; deaths as jumps are BX2.
//!
//! Times are millions of years of the pair's age, like F7's events (TP1
//! converts both together); masses solar, lengths au.

use crate::binary::{common_envelope_separation, q_crit, BinaryRules};
use crate::epoch::TimeEvent;
use crate::star::{AgeingStar, StarSettings};
use serde::{Deserialize, Serialize};
use sse_atoms::binary as bx;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_flow::Reference;
use sse_formation::lifeline::{LifeLine, Phase};
use std::collections::BTreeMap;

/// Settings for the pair stepper (stand-ins in the fudge register).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PairSettings {
    /// Largest change in a star's radius in one step (share).
    pub radius_step: f64,
    /// Largest share of the donor's envelope given in one step.
    pub transfer_step: f64,
    /// Steps across a main sequence at the least.
    pub main_sequence_steps: u32,
    /// The accretor takes at most this many times its own mass over its thermal timescale.
    pub accretor_thermal_factor: f64,
    /// `sse.orbit.overflow_rate`: the mass above which the rate stops growing, solar masses.
    pub overflow_cap_msun: f64,
    /// `sse.star.thermal_timescale`: the share of the binding energy.
    pub thermal_share: f64,
    /// A main-sequence star is rebuilt at its new mass after this share changes.
    pub rebuild_share: f64,
    /// A donor's envelope below this share of its core is gone: it is stripped.
    pub strip_share: f64,
    pub max_steps: u32,
    /// Rows kept: a new row when a mass or the separation moves by this share, or a phase changes.
    pub row_share: f64,
    /// BX3 `sse.orbit.gw_decay`: Peters' factor (64/5).
    pub gw_factor: f64,
    /// BX3 `sse.star.magnetic_braking`: the braking strength in solar units.
    pub braking_scale: f64,
    /// BX3: main-sequence stars between these masses (solar) have a convective
    /// envelope over a radiative core and brake; below the lower one a star is
    /// convective through and its field no longer brakes it (the period gap).
    pub braking_mass_msun: [f64; 2],
    /// BX3: tides hold a star's spin to the orbit (so braking drains the orbit)
    /// when the period is shorter than this many days.
    pub lock_period_days: f64,
    /// BX3: a main-sequence star feeding a compact one may give up to this many
    /// times its own thermal rate when the orbit's loss drives it.
    pub driven_thermal_factor: f64,
    /// BX3: a low-mass dwarf's radius goes as its mass to this power while it
    /// gives gas away (it is assumed to stay in balance).
    pub donor_radius_exponent: f64,
    /// BX3: a brown dwarf donor's radius goes as its mass to this power (−1/3,
    /// a degenerate body): the orbit then widens again (the period bounce).
    pub degenerate_radius_exponent: f64,
}

impl Default for PairSettings {
    fn default() -> Self {
        PairSettings {
            radius_step: 0.02,
            transfer_step: 0.01,
            main_sequence_steps: 200,
            accretor_thermal_factor: 10.0,
            overflow_cap_msun: 5.0,
            thermal_share: 0.5,
            rebuild_share: 0.01,
            strip_share: 0.01,
            max_steps: 400_000,
            row_share: 0.02,
            gw_factor: 12.8,
            braking_scale: 5.83e-16,
            braking_mass_msun: [0.35, 1.25],
            lock_period_days: 10.0,
            driven_thermal_factor: 100.0,
            donor_radius_exponent: 0.8,
            degenerate_radius_exponent: -1.0 / 3.0,
        }
    }
}

/// A star or a pair of nodes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "node", rename_all = "snake_case")]
pub enum Node {
    Star {
        address: String,
        mass_msun: f64,
    },
    Pair {
        a: Box<Node>,
        b: Box<Node>,
        separation_au: f64,
        #[serde(default)]
        eccentricity: f64,
    },
}

/// A planet circling one star of a pair.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanetIn {
    pub address: String,
    pub host: String,
    pub semi_major_axis_au: f64,
}

/// One star at one row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SideRow {
    pub address: String,
    pub phase: String,
    pub mass_msun: f64,
    pub core_msun: f64,
    pub radius_rsun: f64,
    pub luminosity_lsun: f64,
    pub teff_k: f64,
    /// Its radius over its Roche lobe's at periastron (0 for a pair acting as one body).
    pub lobe_fill: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PairRow {
    pub age_myr: f64,
    pub a: SideRow,
    /// None once the two have merged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub b: Option<SideRow>,
    pub separation_au: f64,
    pub eccentricity: f64,
    pub period_days: f64,
    /// Gas leaving the donor now, solar masses a year.
    pub transfer_msun_yr: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub donor: Option<String>,
    /// Mass that has left the pair altogether so far (beyond winds), solar masses.
    pub lost_msun: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlanetFate {
    pub address: String,
    pub host: String,
    pub fate: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at_myr: Option<f64>,
    pub semi_major_axis_au: f64,
}

/// A pair's life as one dated line.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PairLine {
    pub to_myr: f64,
    pub rows: Vec<PairRow>,
    pub events: Vec<TimeEvent>,
    pub planets: Vec<PlanetFate>,
    /// The pairs inside this one, each with its own line.
    pub inner: Vec<PairLine>,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

const G_AU: f64 = 4.0 * m::PI * m::PI; // au³ per solar mass per year², by Kepler's law in these units

fn msun_kg() -> f64 {
    GM_SUN / G
}
fn rsun_au() -> f64 {
    R_SUN / AU
}

/// One star of a pair, with the mass it really has.
#[derive(Clone)]
struct Star {
    address: String,
    z: f64,
    settings: StarSettings,
    star: AgeingStar,
    /// Effective age minus the pair's age, Myr.
    offset: f64,
    mass: f64,
    /// The life line's own wind mass at the last step, to apply its winds.
    wind_ref: f64,
    /// Mass at the last rebuild.
    built_at: f64,
    /// A stripped core's own line (a white dwarf from the moment it was bared).
    bare: Option<LifeLine>,
}

struct Now {
    phase: Phase,
    radius_rsun: f64,
    lum: f64,
    teff: f64,
    core: f64,
}

impl Star {
    fn new(address: &str, mass: f64, z: f64, settings: &StarSettings) -> Result<Star, String> {
        let star = AgeingStar::new(mass, z, settings.clone())?;
        Ok(Star {
            address: address.into(),
            z,
            settings: settings.clone(),
            wind_ref: star.mass_at(0.0),
            star,
            offset: 0.0,
            mass,
            built_at: mass,
            bare: None,
        })
    }

    fn te(&self, t: f64) -> f64 {
        t + self.offset
    }

    fn now(&self, t: f64) -> Now {
        if let Some(l) = &self.bare {
            let st = l.state_at(&self.star.fits, self.te(t).max(l.t_end));
            return Now {
                phase: st.phase,
                radius_rsun: st.radius,
                lum: st.luminosity,
                teff: st.teff,
                core: self.mass,
            };
        }
        let te = self.te(t);
        let st = self.star.state_with_mass(te, self.mass);
        let core = if st.phase.is_remnant() {
            self.mass
        } else {
            self.star.core_at(te).min(self.mass)
        };
        Now {
            phase: st.phase,
            radius_rsun: st.radius,
            lum: st.luminosity,
            teff: st.teff,
            core,
        }
    }

    fn remnant(&self, t: f64) -> bool {
        self.bare.is_some() || self.now(t).phase.is_remnant()
    }

    /// Apply the life line's own winds since the last step.
    fn winds(&mut self, t: f64) -> f64 {
        if self.bare.is_some() {
            return 0.0;
        }
        let w = self.star.mass_at(self.te(t));
        let d = w - self.wind_ref;
        self.wind_ref = w;
        // A remnant has the life line's remnant mass, whatever the pair gave it.
        if self.now(t).phase.is_remnant() && self.mass > w && d < 0.0 {
            let lost = self.mass - self.mass.min(w.max(self.star.line.remnant_mass));
            self.mass -= lost;
            return -lost;
        }
        self.mass = (self.mass + d).max(1e-3);
        d
    }

    /// A main-sequence star whose mass has moved is rebuilt at its new mass,
    /// keeping its share of the main sequence (`fresh` below 1 for an
    /// accretor whose core mixes in new fuel).
    fn rebuild(&mut self, t: f64, accreting: bool) -> Result<(), String> {
        if self.bare.is_some() || (self.mass - self.built_at).abs() < 0.01 * self.built_at {
            return Ok(());
        }
        let te = self.te(t);
        let ph = self.star.line.phase_at(te);
        if !matches!(ph, Phase::MainSequence | Phase::PreMainSequence) {
            return Ok(());
        }
        let share = (te / self.star.line.t_tms).clamp(0.0, 1.0);
        let fresh = if accreting {
            self.built_at / self.mass
        } else {
            1.0
        };
        let star = AgeingStar::new(self.mass, self.z, self.settings.clone())?;
        let te_new = (share * fresh).min(0.999) * star.line.t_tms;
        self.offset = te_new - t;
        self.wind_ref = star.mass_at(te_new);
        self.star = star;
        self.built_at = self.mass;
        Ok(())
    }

    /// Bare the core: the rest of the envelope is gone and it is a white dwarf from now.
    fn strip(&mut self, t: f64, core: f64) {
        let mut l = self.star.line.clone();
        l.remnant = Phase::WhiteDwarf;
        l.remnant_mass = core;
        l.t_end = self.te(t);
        self.bare = Some(l);
        self.mass = core;
    }

    fn row(&self, t: f64, fill: f64) -> SideRow {
        let n = self.now(t);
        SideRow {
            address: self.address.clone(),
            phase: n.phase.label().into(),
            mass_msun: self.mass,
            core_msun: n.core,
            radius_rsun: n.radius_rsun,
            luminosity_lsun: n.lum,
            teff_k: n.teff,
            lobe_fill: fill,
        }
    }

    /// Mass over its thermal timescale, solar masses a year.
    fn thermal_rate(&self, t: f64, envelope_only: bool, share: f64) -> f64 {
        let n = self.now(t);
        let core = if envelope_only { n.core } else { 0.0 };
        let tau = bx::thermal_timescale(
            self.mass * msun_kg(),
            core * msun_kg(),
            n.radius_rsun * R_SUN,
            n.lum.max(1e-6) * L_SUN,
            share,
        ) / JULIAN_YEAR;
        let m = if envelope_only {
            self.mass - n.core
        } else {
            self.mass
        };
        m.max(0.0) / tau.max(1.0)
    }
}

/// BX3: a main-sequence star's convective envelope, solar masses: all of it
/// below the lower braking mass, a share falling to none at the upper one
/// (after Hurley, Pols and Tout 2000, `M_env,0`), none otherwise.
fn convective_envelope(mass: f64, phase: Phase, set: &PairSettings) -> f64 {
    let [lo, hi] = set.braking_mass_msun;
    if !matches!(phase, Phase::MainSequence | Phase::PreMainSequence) || mass >= hi {
        return 0.0;
    }
    if mass < lo {
        return mass;
    }
    let x = (hi - mass) / (hi - lo);
    lo * x * x
}

/// BX3: the orbit's angular momentum lost each year (solar masses au² a year,
/// as J = m1 m2 sqrt(G a / M) in those units), and the separation's own
/// rate from gravitational waves (au a year), both negative.
fn orbit_losses(
    s: &[Star; 2],
    t: f64,
    a: f64,
    set: &PairSettings,
    braking: &mut bool,
) -> (f64, f64) {
    let (m1, m2) = (s[0].mass, s[1].mass);
    let ms = msun_kg();
    let adot = bx::gw_decay(m1 * ms, m2 * ms, a * AU, set.gw_factor) / AU * JULIAN_YEAR;
    let mut jdot = 0.0;
    let p_days = kepler_days(a, m1 + m2);
    *braking = false;
    if p_days < set.lock_period_days {
        let omega = 2.0 * m::PI / (p_days * DAY);
        for k in 0..2 {
            let n = s[k].now(t);
            if s[k].bare.is_some() {
                continue;
            }
            let env = convective_envelope(s[k].mass, n.phase, set);
            // A star convective through has no braking field (disrupted braking).
            if env <= 0.0 || env >= s[k].mass {
                continue;
            }
            let si = bx::magnetic_braking(
                s[k].mass * ms,
                env * ms,
                n.radius_rsun * R_SUN,
                omega,
                set.braking_scale,
            );
            // kg m2 s-2 to solar masses au2 per year, per year.
            jdot += si / (ms * AU * AU) * JULIAN_YEAR * JULIAN_YEAR;
            *braking = true;
        }
    }
    (jdot, adot)
}

fn kepler_days(a_au: f64, m: f64) -> f64 {
    m::sqrt(a_au * a_au * a_au / m.max(1e-9)) * JULIAN_YEAR / DAY
}

/// Lobe fill of `d` against `acc` at periastron.
fn fill(d: &Star, acc: &Star, t: f64, peri: f64) -> f64 {
    let n = d.now(t);
    if n.phase.is_remnant() || d.bare.is_some() {
        return 0.0;
    }
    n.radius_rsun * rsun_au() / bx::roche_lobe(d.mass / acc.mass, peri)
}

const REF_JEANS: &str = "Jeans, J. H. 1924, Cosmogonic problems associated with a secular decrease of mass, MNRAS 85, 2: an orbit widens as a·M stays constant while the pair slowly loses mass. doi:10.1093/mnras/85.1.2";

/// Age a pair of two stars together from birth to `to_myr`.
#[allow(clippy::too_many_arguments)]
fn age_two(
    a_addr: &str,
    a_mass: f64,
    b_addr: &str,
    b_mass: f64,
    sep_au: f64,
    e0: f64,
    z: f64,
    to_myr: f64,
    planets: &[PlanetIn],
    star_set: &StarSettings,
    rules: &BinaryRules,
    set: &PairSettings,
) -> Result<PairLine, String> {
    let mut s = [
        Star::new(a_addr, a_mass, z, star_set)?,
        Star::new(b_addr, b_mass, z, star_set)?,
    ];
    let mut out = PairLine {
        to_myr,
        rows: vec![],
        events: vec![],
        planets: vec![],
        inner: vec![],
        why: vec![],
        stand_ins: stand_ins(set),
        references: references(),
    };
    let (mut a, mut e) = (sep_au, e0);
    let mut merged = false;
    let mut lost = 0.0;
    let mut donor: Option<usize> = None;
    // The donor of the last stable episode (a pause in the flow is not a new contact).
    let mut episode: Option<usize> = None;
    let mut worlds: Vec<(PlanetIn, Option<f64>)> = planets
        .iter()
        .filter(|p| p.host == a_addr || p.host == b_addr)
        .map(|p| (p.clone(), None))
        .collect();
    let mut t = 0.0;
    let mut dt = (s[0].star.line.t_tms.min(s[1].star.line.t_tms)
        / f64::from(set.main_sequence_steps))
    .min(to_myr.max(1e-6));
    let mut last_r = [s[0].now(0.0).radius_rsun, s[1].now(0.0).radius_rsun];
    let mut steps = 0u32;
    let mut rate_now = 0.0;
    let mut last_row: Option<(f64, f64, f64, String, String, bool)> = None;
    // BX3: how far angular momentum loss has drawn the orbit in since the last
    // episode, whether a star is braking, and the share of the orbit lost a year.
    let mut drawn_in = 0.0;
    let mut braking = false;
    let mut loss_rate = 0.0;
    let mut a_episode = sep_au;
    loop {
        // A row when something has moved enough.
        let fills = if merged {
            [0.0, 0.0]
        } else {
            let peri = a * (1.0 - e);
            [fill(&s[0], &s[1], t, peri), fill(&s[1], &s[0], t, peri)]
        };
        let key = (
            s[0].mass,
            if merged { 0.0 } else { s[1].mass },
            a,
            s[0].now(t).phase.label().to_string(),
            if merged {
                String::new()
            } else {
                s[1].now(t).phase.label().to_string()
            },
            donor.is_some(),
        );
        let moved = match &last_row {
            None => true,
            Some(k) => {
                (k.0 - key.0).abs() > set.row_share * k.0
                    || (k.1 - key.1).abs() > set.row_share * k.1.max(1e-3)
                    || (k.2 - key.2).abs() > set.row_share * k.2.max(1e-9)
                    || k.3 != key.3
                    || k.4 != key.4
                    || k.5 != key.5
            }
        };
        if moved || t >= to_myr {
            out.rows.push(PairRow {
                age_myr: t,
                a: s[0].row(t, fills[0]),
                b: (!merged).then(|| s[1].row(t, fills[1])),
                separation_au: if merged { 0.0 } else { a },
                eccentricity: if merged { 0.0 } else { e },
                period_days: if merged {
                    0.0
                } else {
                    kepler_days(a, s[0].mass + s[1].mass)
                },
                transfer_msun_yr: rate_now,
                donor: donor.map(|d| s[d].address.clone()),
                lost_msun: lost,
            });
            last_row = Some(key);
        }
        if t >= to_myr {
            break;
        }
        steps += 1;
        if steps > set.max_steps {
            out.why.push(format!(
                "The line stops at {t:.1} million years: more than {} steps were needed.",
                set.max_steps
            ));
            break;
        }
        let t1 = (t + dt).min(to_myr);
        let step = t1 - t;
        // Winds: the orbit widens as a·M stays constant.
        let m_before = s[0].mass + if merged { 0.0 } else { s[1].mass };
        let hosts_before = [s[0].mass, s[1].mass];
        s[0].winds(t1);
        if !merged {
            s[1].winds(t1);
        }
        let m_after = s[0].mass + if merged { 0.0 } else { s[1].mass };
        if !merged {
            a *= m_before / m_after;
            // BX3: gravitational waves and magnetic braking draw the orbit in.
            let (jdot, adot) = orbit_losses(&s, t1, a, set, &mut braking);
            let mt = s[0].mass + s[1].mass;
            let j = s[0].mass * s[1].mass * m::sqrt(G_AU * a / mt);
            let a0 = a;
            let jr = (1.0 + jdot * step * 1e6 / j).max(0.5);
            a = (a * jr * jr + adot * step * 1e6).max(1e-9);
            drawn_in += (a0 - a).max(0.0);
            loss_rate = ((a0 - a) / a0).max(0.0) / (step * 1e6).max(1e-30);
        }
        rate_now = 0.0;
        if !merged {
            let peri = a * (1.0 - e);
            let f = [fill(&s[0], &s[1], t1, peri), fill(&s[1], &s[0], t1, peri)];
            let d = if f[0] >= 1.0 && f[0] >= f[1] {
                Some(0)
            } else if f[1] >= 1.0 {
                Some(1)
            } else {
                None
            };
            match d {
                None => donor = None,
                Some(di) => {
                    let ai = 1 - di;
                    let nd = s[di].now(t1);
                    let (md, ma) = (s[di].mass, s[ai].mass);
                    // A new contact (or the other star's turn): the critical ratio decides again.
                    if donor.is_none() && episode != Some(di) {
                        let q = md / ma;
                        let main_seq =
                            matches!(nd.phase, Phase::MainSequence | Phase::PreMainSequence);
                        // BX3: a dwarf feeding a compact star has its own limit.
                        let onto_compact = s[ai].remnant(t1);
                        let qc = if main_seq && onto_compact {
                            rules.q_crit_main_sequence_onto_degenerate
                        } else {
                            q_crit(rules, nd.phase, md, nd.core)
                        };
                        let mut numbers = BTreeMap::new();
                        numbers.insert("donor_mass".into(), md);
                        numbers.insert("accretor_mass".into(), ma);
                        numbers.insert("mass_ratio".into(), q);
                        numbers.insert("critical_ratio".into(), qc);
                        numbers.insert("separation_au".into(), a);
                        let phase = nd.phase.label();
                        if q >= qc && main_seq && !onto_compact {
                            // Two main-sequence stars run away into one.
                            let total = md + ma;
                            let keep = if md >= ma { di } else { ai };
                            out.events.push(event(
                                t1,
                                "merger",
                                &s[di].address,
                                &s[ai].address,
                                format!("The {phase} star filled its Roche lobe with a mass ratio of {q:.2}, above the critical {qc:.2}: the transfer ran away and the two stars merged into one of about {total:.2} solar masses."),
                                numbers,
                            ));
                            s[keep].mass = total;
                            s[keep].rebuild(t1, true)?;
                            if keep == 1 {
                                s.swap(0, 1);
                            }
                            merged = true;
                            donor = None;
                            t = t1;
                            continue;
                        }
                        if q >= qc {
                            // A common envelope, or a merger inside it.
                            let r_d = nd.radius_rsun * rsun_au();
                            let a_f = common_envelope_separation(md, nd.core, ma, a, r_d, rules);
                            let r_acc = s[ai].now(t1).radius_rsun * rsun_au();
                            let fits = r_acc < bx::roche_lobe(ma / nd.core.max(1e-3), a_f);
                            numbers.insert("separation_after_au".into(), a_f);
                            if fits {
                                let menv = md - nd.core;
                                out.events.push(event(
                                    t1,
                                    "common-envelope",
                                    &s[di].address,
                                    &s[ai].address,
                                    format!("The {phase} star overflowed its Roche lobe faster than its {ma:.2} solar mass companion could take the gas (mass ratio {q:.2}, critical {qc:.2}). The companion was engulfed; drag pulled the orbit in from {a:.3} to {a_f:.4} au and the released energy blew the {menv:.2} solar mass envelope away, leaving the bare {:.2} solar mass core.", nd.core),
                                    numbers,
                                ));
                                lost += menv;
                                s[di].strip(t1, nd.core);
                                a = a_f;
                                e = 0.0;
                                drawn_in = 0.0;
                                a_episode = a;
                                donor = None;
                                t = t1;
                                dt = dt.min(1e-3);
                                continue;
                            }
                            let total = md + ma;
                            out.events.push(event(
                                t1,
                                "merger",
                                &s[di].address,
                                &s[ai].address,
                                format!("The {phase} star's envelope swallowed its companion; the orbit's energy could not blow it away before the companion reached the core, and the two merged into one of about {total:.2} solar masses."),
                                numbers,
                            ));
                            s[di].mass = total;
                            if di == 1 {
                                s.swap(0, 1);
                            }
                            merged = true;
                            donor = None;
                            t = t1;
                            continue;
                        }
                        // Stable: tides round the orbit off first (angular momentum kept).
                        a *= 1.0 - e * e;
                        e = 0.0;
                        episode = Some(di);
                        // BX3: brought into touch by the orbit shrinking, not the star swelling.
                        let pulled = drawn_in > 0.1 * a;
                        let how = if pulled {
                            let why = if braking {
                                "magnetic braking and gravitational waves"
                            } else {
                                "gravitational waves"
                            };
                            format!("{why} drew the orbit in from {a_episode:.4} to {a:.4} au until the {phase} star filled its Roche lobe")
                        } else {
                            format!("The {phase} star swelled to fill its Roche lobe")
                        };
                        let how = {
                            let mut c = how.chars();
                            c.next().map_or(String::new(), |f| {
                                f.to_uppercase().collect::<String>() + c.as_str()
                            })
                        };
                        let feeds_dwarf = s[ai].remnant(t1)
                            && s[ai].now(t1).phase == Phase::WhiteDwarf
                            && main_seq;
                        numbers.insert("period_days".into(), kepler_days(a, md + ma));
                        out.events.push(if feeds_dwarf {
                            event(
                                t1,
                                "cataclysmic-variable",
                                &s[di].address,
                                &s[ai].address,
                                format!("{how}, and began feeding its {ma:.2} solar mass white dwarf companion: a cataclysmic variable, with a period of {:.2} hours. The mass ratio {q:.2} is below the critical {qc:.2}, so the flow is stable; what the white dwarf does with the gas (novae) is not worked out yet.", kepler_days(a, md + ma) * 24.0),
                                numbers,
                            )
                        } else {
                            event(
                                t1,
                                "stable-mass-transfer",
                                &s[di].address,
                                &s[ai].address,
                                format!("{how} and began pouring gas onto its {ma:.2} solar mass companion. The mass ratio {q:.2} is below the critical {qc:.2}, so the flow is stable."),
                                numbers,
                            )
                        });
                        drawn_in = 0.0;
                        a_episode = a;
                    }
                    donor = Some(di);
                    // The overflow rate, no faster than the envelope can adjust.
                    let fl = f[di];
                    let over = bx::overflow_rate(md * msun_kg(), fl, set.overflow_cap_msun)
                        / msun_kg()
                        * JULIAN_YEAR;
                    let thermal = s[di].thermal_rate(t1, true, set.thermal_share);
                    let dwarf = nd.phase == Phase::BrownDwarf;
                    let giant =
                        !dwarf && !matches!(nd.phase, Phase::MainSequence | Phase::PreMainSequence);
                    let rate = if giant {
                        over.min(thermal)
                    } else {
                        over.min(s[di].thermal_rate(t1, false, set.thermal_share))
                    };
                    let envelope = (md - nd.core).max(0.0);
                    let mut dm = (rate * step * 1e6).min(envelope);
                    // BX3: a dwarf fed to a compact star by the orbit's loss gives just
                    // what keeps it at its lobe: the gas lost from beside the accretor
                    // and the dwarf shrinking with its mass widen the lobe back to it.
                    if !giant && s[ai].remnant(t1) {
                        let r0 = nd.radius_rsun * rsun_au();
                        let fill_after = |dm: f64| {
                            let (md2, ma2) = (md - dm, ma);
                            let mt = md + ma;
                            let j = md * ma * m::sqrt(G_AU * a / mt);
                            let jl = dm * (md / mt) * (md / mt) * m::sqrt(G_AU * mt * a);
                            let mt2 = md2 + ma2;
                            let x = (j - jl).max(1e-12) * mt2 / (md2 * ma2);
                            let a2 = x * x / (G_AU * mt2);
                            // A brown dwarf is held up by degeneracy: it swells as it shrinks.
                            let zeta = if dwarf {
                                set.degenerate_radius_exponent
                            } else {
                                set.donor_radius_exponent
                            };
                            let r2 = r0 * m::pow(md2 / md, zeta);
                            r2 / bx::roche_lobe(md2 / ma2, a2)
                        };
                        // A brown dwarf has no thermal brake on how fast it can give.
                        let thermal_cap = if dwarf {
                            f64::INFINITY
                        } else {
                            s[di].thermal_rate(t1, false, set.thermal_share)
                                * set.driven_thermal_factor
                                * step
                                * 1e6
                        };
                        let cap = thermal_cap.min(set.transfer_step * envelope.max(1e-6));
                        dm = if fill_after(cap) > 1.0 {
                            cap
                        } else {
                            let (mut lo, mut hi) = (0.0, cap);
                            for _ in 0..40 {
                                let mid = 0.5 * (lo + hi);
                                if fill_after(mid) > 1.0 {
                                    lo = mid;
                                } else {
                                    hi = mid;
                                }
                            }
                            hi
                        };
                    }
                    // Keep each step small: take what the step allows, and shorten the next.
                    let most = set.transfer_step * envelope.max(1e-6);
                    if dm > most {
                        dm = most;
                    }
                    rate_now = dm / (step * 1e6);
                    let take = if s[ai].remnant(t1) {
                        0.0
                    } else {
                        s[ai].thermal_rate(t1, false, set.thermal_share)
                            * set.accretor_thermal_factor
                            * step
                            * 1e6
                    };
                    let kept = dm.min(take);
                    let gone = dm - kept;
                    // Angular momentum: the lost gas leaves from beside the accretor.
                    let mt = md + ma;
                    let j = md * ma * m::sqrt(G_AU * a / mt);
                    let j_lost = gone * (md / mt) * (md / mt) * m::sqrt(G_AU * mt * a);
                    let (md2, ma2) = (md - dm, ma + kept);
                    let mt2 = md2 + ma2;
                    let j2 = (j - j_lost).max(1e-12);
                    let x = j2 * mt2 / (md2 * ma2);
                    a = x * x / (G_AU * mt2);
                    s[di].mass = md2;
                    s[ai].mass = ma2;
                    lost += gone;
                    s[di].rebuild(t1, false)?;
                    s[ai].rebuild(t1, true)?;
                    let core_now = s[di].now(t1).core;
                    if giant && s[di].mass - core_now < set.strip_share * core_now {
                        let core = core_now;
                        s[di].strip(t1, core);
                        out.events.push(event(
                            t1,
                            "stripped",
                            &s[di].address,
                            &s[ai].address,
                            format!("The donor has given away its whole envelope: its bare {core:.2} solar mass core is left, a white dwarf from now."),
                            BTreeMap::new(),
                        ));
                        donor = None;
                    }
                }
            }
        }
        // Planets: orbits widen as their star loses mass; swallowed when it swells past them.
        for (p, gone) in worlds.iter_mut() {
            if gone.is_some() {
                continue;
            }
            // After a merger every planet circles the one star left.
            let hi = if merged || p.host == s[0].address {
                0
            } else {
                1
            };
            let before = hosts_before[hi];
            p.semi_major_axis_au *= before / s[hi].mass.max(1e-6);
            let r = s[hi].now(t1).radius_rsun * rsun_au();
            if r >= p.semi_major_axis_au {
                *gone = Some(t1);
                out.events.push(event(
                    t1,
                    "engulfed",
                    &p.address,
                    &s[hi].address,
                    format!("The star swelled to {:.2} au and swallowed the planet orbiting at {:.2} au.", r, p.semi_major_axis_au),
                    BTreeMap::new(),
                ));
            }
        }
        // The next step: no star's radius moves too much at once.
        let r_now = [s[0].now(t1).radius_rsun, s[1].now(t1).radius_rsun];
        let mut grow: f64 = 2.0;
        for k in 0..2 {
            if merged && k == 1 {
                continue;
            }
            let ch = m::ln((r_now[k] / last_r[k].max(1e-9)).max(1e-12)).abs();
            if ch > 0.0 {
                grow = grow.min(set.radius_step / ch);
            }
        }
        let ms_step = s[0].star.line.t_tms.min(if merged {
            f64::INFINITY
        } else {
            s[1].star.line.t_tms
        }) / f64::from(set.main_sequence_steps);
        dt = (step * grow.clamp(0.25, 2.0)).min(ms_step.max(1e-6));
        if let (Some(d), true) = (donor, rate_now > 0.0) {
            let envelope = (s[d].mass - s[d].now(t1).core).max(1e-6);
            dt = dt.min(set.transfer_step * envelope / rate_now / 1e6);
        }
        if !merged && loss_rate > 0.0 {
            dt = dt.min(set.radius_step / loss_rate / 1e6);
        }
        dt = dt.max(1e-9);
        last_r = r_now;
        t = t1;
    }
    for (p, gone) in worlds {
        out.planets.push(PlanetFate {
            address: p.address,
            host: p.host,
            fate: if gone.is_some() { "engulfed" } else { "kept" }.into(),
            at_myr: gone,
            semi_major_axis_au: p.semi_major_axis_au,
        });
    }
    out.why.push(if out.events.is_empty() {
        "The two stars never came close enough to touch: each aged as if alone, the orbit widening as winds carried mass away.".into()
    } else {
        format!(
            "Their story: {}.",
            out.events
                .iter()
                .map(|e| e.kind.as_str())
                .collect::<Vec<_>>()
                .join(", then ")
        )
    });
    Ok(out)
}

fn event(
    t: f64,
    kind: &str,
    a: &str,
    b: &str,
    why: String,
    numbers: BTreeMap<String, f64>,
) -> TimeEvent {
    TimeEvent {
        age_myr: t,
        kind: kind.into(),
        subjects: vec![a.into(), b.into()],
        why,
        level: "standard".into(),
        plausible_future: false,
        numbers,
    }
}

fn stand_ins(set: &PairSettings) -> Vec<String> {
    vec![
        format!("steps keep each star's radius within {} and each transfer step within {} of the envelope; rows every {} change", set.radius_step, set.transfer_step, set.row_share),
        format!("the accretor keeps at most {} times its own thermal rate, the rest leaving from beside it with its orbital angular momentum", set.accretor_thermal_factor),
        "a white dwarf, neutron star or black hole keeps none of the gas it is given (novae and X-ray binaries are BX4)".into(),
        "a donor stripped to its core is a white dwarf of that core from then (a helium star stage is left out)".into(),
        "a main-sequence star rebuilt at its new mass keeps its share of the main sequence; an accretor's share falls as the mass ratio (fresh fuel mixed into its core)".into(),
        "a pair acting as one body one level up trades no gas; a death is still the life line's slow mass loss (jumps and kicks are BX2)".into(),
        format!("orbits shrink by gravitational waves (Peters 1964) and, below a {}-day period where tides lock spin to orbit, by magnetic braking of main-sequence stars between {} and {} solar masses (Hurley, Tout and Pols 2002 eq. 50); a star convective through brakes no more", set.lock_period_days, set.braking_mass_msun[0], set.braking_mass_msun[1]),
        format!("a dwarf feeding a compact star gives just what keeps it at its lobe, its radius going as its mass to the {} (a brown dwarf's to the {:.2}), up to {} times its thermal rate; it never swells out of balance, so it does not detach at the period gap's top", set.donor_radius_exponent, set.degenerate_radius_exponent, set.driven_thermal_factor),
        "ages in millions of years like F7's events, until the time scales design (TP1) converts both".into(),
    ]
}

fn references() -> Vec<Reference> {
    let mut out = vec![
        Reference::parse(crate::binary::REF_HTP02),
        Reference::parse(crate::binary::REF_WEBBINK),
        Reference::parse(REF_JEANS),
    ];
    for d in sse_atoms::decls_of(&[
        "sse.orbit.roche_lobe",
        "sse.orbit.overflow_rate",
        "sse.star.thermal_timescale",
    ]) {
        for r in d.references {
            if !out.contains(&r) {
                out.push(r);
            }
        }
    }
    out
}

/// The mass of a node at an age, from its line (a star alone keeps its own).
fn mass_on(line: &PairLine, t: f64) -> f64 {
    let rows = &line.rows;
    let i = rows
        .partition_point(|r| r.age_myr <= t)
        .clamp(1, rows.len().max(1))
        - 1;
    let r = &rows[i.min(rows.len() - 1)];
    r.a.mass_msun + r.b.as_ref().map_or(0.0, |b| b.mass_msun)
}

/// Age any node from birth to `to_myr`: a star alone, two stars side by
/// side trading gas, or a pair whose sides are nodes acting as one body each.
pub fn age_node(
    node: &Node,
    z: f64,
    to_myr: f64,
    planets: &[PlanetIn],
    star_set: &StarSettings,
    rules: &BinaryRules,
    set: &PairSettings,
) -> Result<PairLine, String> {
    if to_myr.is_nan() || to_myr < 0.0 {
        return Err("the age must not be negative".into());
    }
    match node {
        Node::Star { address, mass_msun } => {
            // A star alone: a pair with nothing to touch.
            let s = Star::new(address, *mass_msun, z, star_set)?;
            let mut out = PairLine {
                to_myr,
                rows: vec![],
                events: vec![],
                planets: vec![],
                inner: vec![],
                why: vec!["A star alone.".into()],
                stand_ins: vec![],
                references: vec![],
            };
            let mut st = s;
            let n = 200;
            for k in 0..=n {
                let t = to_myr * f64::from(k) / f64::from(n);
                st.winds(t);
                out.rows.push(PairRow {
                    age_myr: t,
                    a: st.row(t, 0.0),
                    b: None,
                    separation_au: 0.0,
                    eccentricity: 0.0,
                    period_days: 0.0,
                    transfer_msun_yr: 0.0,
                    donor: None,
                    lost_msun: 0.0,
                });
            }
            Ok(out)
        }
        Node::Pair {
            a,
            b,
            separation_au,
            eccentricity,
        } => match (a.as_ref(), b.as_ref()) {
            (
                Node::Star {
                    address: aa,
                    mass_msun: ma,
                },
                Node::Star {
                    address: ba,
                    mass_msun: mb,
                },
            ) => age_two(
                aa,
                *ma,
                ba,
                *mb,
                *separation_au,
                *eccentricity,
                z,
                to_myr,
                planets,
                star_set,
                rules,
                set,
            ),
            _ => {
                // Each side acts as one body at its centre of mass.
                let la = age_node(a, z, to_myr, planets, star_set, rules, set)?;
                let lb = age_node(b, z, to_myr, planets, star_set, rules, set)?;
                let m0 = mass_on(&la, 0.0) + mass_on(&lb, 0.0);
                let side = |l: &PairLine, n: &Node, t: f64| -> SideRow {
                    let mass = mass_on(l, t);
                    SideRow {
                        address: match n {
                            Node::Star { address, .. } => address.clone(),
                            Node::Pair { .. } => "pair".into(),
                        },
                        phase: if matches!(n, Node::Pair { .. }) {
                            "pair".into()
                        } else {
                            l.rows
                                .iter()
                                .rev()
                                .find(|r| r.age_myr <= t)
                                .map_or(String::new(), |r| r.a.phase.clone())
                        },
                        mass_msun: mass,
                        core_msun: 0.0,
                        radius_rsun: 0.0,
                        luminosity_lsun: 0.0,
                        teff_k: 0.0,
                        lobe_fill: 0.0,
                    }
                };
                let mut out = PairLine {
                    to_myr,
                    rows: vec![],
                    events: vec![],
                    planets: vec![],
                    inner: vec![],
                    why: vec!["Each side acts as one body at its centre of mass; the orbit between them widens as they lose mass (a·M kept).".into()],
                    stand_ins: stand_ins(set),
                    references: references(),
                };
                let n = 200;
                for k in 0..=n {
                    let t = to_myr * f64::from(k) / f64::from(n);
                    let (sa, sb) = (side(&la, a, t), side(&lb, b, t));
                    let mt = sa.mass_msun + sb.mass_msun;
                    let sep = separation_au * m0 / mt.max(1e-9);
                    out.rows.push(PairRow {
                        age_myr: t,
                        a: sa,
                        b: Some(sb),
                        separation_au: sep,
                        eccentricity: *eccentricity,
                        period_days: kepler_days(sep, mt),
                        transfer_msun_yr: 0.0,
                        donor: None,
                        lost_msun: 0.0,
                    });
                }
                for l in [la, lb] {
                    if l.rows.first().is_some_and(|r| r.b.is_some()) || !l.inner.is_empty() {
                        out.inner.push(l);
                    }
                }
                Ok(out)
            }
        },
    }
}
