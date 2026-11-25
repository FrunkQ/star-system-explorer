//! E: events (galaxy design section 9, stage G6).
//!
//! Events are how the galaxy becomes history rather than statistics. Here
//! they are drawn, never stored: the envelope's rates (G4, from the rings of
//! section 6) set a keyed Poisson count per cell of space and bin of time,
//! and each event's place and moment are keyed draws in that cell. So an
//! event belongs to its cell, not to whoever asks, and two neighbouring
//! patches see the same supernova at the same place: that is how an event
//! "crosses patch boundaries". What a site feels is worked out from those
//! events with the travel time respected: light (and radiation) arrives at
//! d / c, a remnant's shock at d / v.
//!
//! | Event | Drawn from | What a site feels |
//! |---|---|---|
//! | core-collapse supernova | the envelope's core-collapse rate per pc^3, in 100 pc cells and 10 Myr bins | harm within the sterilising distance; the remnant bubble passes within `remnant_pc` for `remnant_myr` (fresh metals, Al-26 for a cluster forming there) |
//! | type Ia supernova | the type Ia rate, as above | as above (iron, no Al-26) |
//! | neutron-star merger | `nsm_per_ia` x the type Ia rate (both follow a 1/t delay) | a kilonova: an r-process lump, harm as a supernova |
//! | long gamma-ray burst | the envelope's lethal-burst rate per pc^2, in 1 kpc columns and 100 Myr bins | harmful within `grb_kpc` |
//! | active nucleus episode | the galaxy's own episodes (G3) | a dose (G4's measure) along the line from the centre |
//! | close stellar encounter | the envelope's encounter rate at the site, 10 Myr bins | passage distance, speed and the passing star's mass; a comet shower inside `shower_pc` |
//! | arm crossing | the site's path against the spiral pattern | higher gas, star formation and hazard rates while inside |
//! | major merger, starburst | the halo's mergers and the GM's edits | galaxy-wide: bulge growth, a burst, a feeding black hole |
//!
//! History edits (`galaxy::Edit`) change the galaxy from their own time on;
//! the rates before the earliest edit are bit-identical, so every event that
//! happened before is the same event after.

use crate::envelope::{GalaxyEnvelope, GreenParams, Place};
use crate::galaxy::{self, Edit, Galaxy};
use crate::haloes::{poisson, Halo};
use crate::stars::{Orbit, StarParams};
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, Seed};
use std::collections::BTreeMap;

/// Light speed in parsecs per Myr.
pub const C_PC_MYR: f64 =
    sse_core::sheet::C * 1e6 * sse_core::sheet::YR_JULIAN / sse_core::sheet::PC;
const KMS_PC_MYR: f64 = 1e3 * 1e6 * sse_core::sheet::YR_JULIAN / sse_core::sheet::PC;

/// Pack settings (`provides.other.cosmology.events`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventParams {
    /// The event grid co-rotates with the disc at this rate (km/s/kpc), as the star grid does (G5).
    /// A frame convention like the star grid's, kept at its first value (DM4 log).
    pub grid_rotation_kms_kpc: f64,
    /// Supernova-scale cells (pc) and time bins (Myr).
    pub sn_cell_pc: f64,
    pub sn_bin_myr: f64,
    /// A remnant's bubble: radius (pc), shock speed (km/s) and how long it lasts (Myr).
    /// Tens of parsecs for about a million years (galaxy design section 9).
    pub remnant_pc: f64,
    pub remnant_kms: f64,
    pub remnant_myr: f64,
    /// Neutron-star mergers per type Ia supernova: about 3e-5 against 5e-3 per year in
    /// a Milky-Way-like galaxy (Abbott et al. 2023 for the merger rate; Li et al. 2011 for Ia).
    pub nsm_per_ia: f64,
    /// Gamma-ray burst columns (pc) and time bins (Myr).
    pub grb_cell_pc: f64,
    pub grb_bin_myr: f64,
    /// Close-encounter bins (Myr) and the passage distance inside which an Oort-like
    /// cloud is shaken into a comet shower (pc; about 10,000 au). Hills 1981's
    /// 1e4 au is the comets' semi-major axis; passages inside it shaking a
    /// shower is our reading, not a threshold the paper states.
    pub encounter_bin_myr: f64,
    pub shower_pc: f64,
    /// Arm crossings: the path is sampled every `arm_step_myr`, and a site is in an arm
    /// while the density wave's cos(phase) is above `arm_cos`.
    pub arm_step_myr: f64,
    pub arm_cos: f64,
}

impl EventParams {
    /// The start of the widest draw bin holding time t (Myr): every event drawn before
    /// it used rates from before t.
    pub fn before_bin(&self, t_myr: f64) -> f64 {
        let w = self
            .sn_bin_myr
            .max(self.grb_bin_myr)
            .max(self.encounter_bin_myr);
        m::floor(t_myr / w) * w
    }
}

impl Default for EventParams {
    fn default() -> Self {
        EventParams {
            grid_rotation_kms_kpc: 26.83,
            sn_cell_pc: 100.0,
            sn_bin_myr: 10.0,
            remnant_pc: 50.0,
            remnant_kms: 200.0,
            remnant_myr: 1.0,
            nsm_per_ia: 0.005,
            grb_cell_pc: 1000.0,
            grb_bin_myr: 100.0,
            encounter_bin_myr: 10.0,
            shower_pc: 0.05,
            arm_step_myr: 1.0,
            arm_cos: 0.5,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    CoreCollapse,
    TypeIa,
    NeutronStarMerger,
    GammaRayBurst,
    ActiveNucleus,
    Encounter,
    ArmEntry,
    ArmExit,
    Merger,
    Starburst,
}

impl Kind {
    /// The chronicle event kind.
    pub fn chronicle_kind(self) -> &'static str {
        match self {
            Kind::CoreCollapse => "galaxy.supernova.core_collapse",
            Kind::TypeIa => "galaxy.supernova.type_ia",
            Kind::NeutronStarMerger => "galaxy.kilonova",
            Kind::GammaRayBurst => "galaxy.gamma_ray_burst",
            Kind::ActiveNucleus => "galaxy.active_nucleus.exposure",
            Kind::Encounter => "galaxy.encounter",
            Kind::ArmEntry => "galaxy.arm.entry",
            Kind::ArmExit => "galaxy.arm.exit",
            Kind::Merger => "galaxy.merger",
            Kind::Starburst => "galaxy.starburst",
        }
    }
    fn slot(self) -> u64 {
        match self {
            Kind::CoreCollapse => 0,
            Kind::TypeIa => 1,
            Kind::NeutronStarMerger => 2,
            _ => 3,
        }
    }
}

/// One event, where and when it happened. Positions are in the co-rotating
/// event grid (pc); galaxy-wide events sit at the centre.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GalaxyEvent {
    pub kind: Kind,
    /// Stable within a universe: the cell or site, the bin and the draw.
    pub id: String,
    pub t_myr: f64,
    pub grid_pc: [f64; 3],
}

/// An event as one site feels it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Hit {
    pub kind: Kind,
    pub id: String,
    pub occurred_myr: f64,
    /// When it reaches the site (light, or the shock for a remnant-only hit).
    pub activated_myr: f64,
    /// When its effect ends at the site.
    pub until_myr: f64,
    pub distance_pc: f64,
    /// Radiation the design counts as harmful to a biosphere.
    pub harmful: bool,
    /// The remnant bubble passes over the site.
    pub remnant: bool,
    /// A close passage that shakes the comet cloud.
    pub shower: bool,
    /// Active nucleus: G4's dose. Encounter: (M / Msun) / ((b / pc) (v / km/s)), the
    /// impulse an outer comet cloud feels, in those units. Otherwise 0.
    pub dose: f64,
    /// Encounter only: passage distance (pc), relative speed (km/s), passing star's mass (Msun).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub passage: Option<[f64; 3]>,
    pub why: String,
}

/// Where a site is over time: fixed in the co-rotating grid, or on a star's orbit.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SitePath {
    Fixed { grid_pc: [f64; 3] },
    Orbit(Orbit),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EventSite {
    pub address: Address,
    pub path: SitePath,
}

/// Everything a site felt in a window, in order of arrival.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Exposure {
    pub site: String,
    pub from_myr: f64,
    pub to_myr: f64,
    pub hits: Vec<Hit>,
}

impl Exposure {
    pub fn count(&self, k: Kind) -> usize {
        self.hits.iter().filter(|h| h.kind == k).count()
    }
}

/// The event layer over one galaxy.
pub struct Events<'a> {
    pub env: &'a GalaxyEnvelope<'a>,
    pub halo: &'a Halo,
    pub edits: &'a [Edit],
    pub green: GreenParams,
    pub stars: StarParams,
    pub params: EventParams,
    pub seed: Seed,
}

fn stream(s: &str) -> Stream {
    Stream::new(s).expect("fixed stream names are valid")
}

impl Events<'_> {
    fn angle(&self, t_myr: f64) -> f64 {
        self.params.grid_rotation_kms_kpc * KMS_PC_MYR / 1000.0 * t_myr
    }
    /// Galactic position of a grid point at a time.
    pub fn from_grid(&self, g: [f64; 3], t_myr: f64) -> [f64; 3] {
        let a = self.angle(t_myr);
        [
            g[0] * m::cos(a) - g[1] * m::sin(a),
            g[0] * m::sin(a) + g[1] * m::cos(a),
            g[2],
        ]
    }
    /// Grid position of a galactic point at a time.
    pub fn to_grid(&self, x: [f64; 3], t_myr: f64) -> [f64; 3] {
        let a = -self.angle(t_myr);
        [
            x[0] * m::cos(a) - x[1] * m::sin(a),
            x[0] * m::sin(a) + x[1] * m::cos(a),
            x[2],
        ]
    }
    fn place(&self, g: [f64; 3], t_myr: f64) -> Place {
        let [x, y, z] = self.from_grid(g, t_myr);
        Place {
            r_kpc: m::hypot(x, y) / 1000.0,
            phi: m::atan2(y, x),
            z_pc: z,
            t_gyr: t_myr / 1000.0,
        }
    }
    /// A site's grid position at a time.
    pub fn site_at(&self, s: &SitePath, t_myr: f64) -> [f64; 3] {
        match s {
            SitePath::Fixed { grid_pc } => *grid_pc,
            SitePath::Orbit(o) => self.to_grid(o.xyz(t_myr), t_myr),
        }
    }
    fn key(&self, kind: &str, key: &str) -> Result<Address, String> {
        self.env
            .address
            .child_key(kind, key)
            .map_err(|e| format!("{e:?}"))
    }

    /// Expected core-collapse, type Ia and neutron-star-merger events in a cell and bin
    /// (the envelope at the cell centre and bin middle).
    pub fn expected_cell(&self, cell: [i64; 3], bin: i64) -> [f64; 3] {
        let (c, b) = (self.params.sn_cell_pc, self.params.sn_bin_myr);
        let mid = |i: i64| (i as f64 + 0.5) * c;
        let t = (bin as f64 + 0.5) * b;
        let l = self.env.local(
            self.place([mid(cell[0]), mid(cell[1]), mid(cell[2])], t),
            &self.green,
        );
        let vol_gyr = c * c * c * b / 1000.0;
        [
            l.cc_gyr_pc3 * vol_gyr,
            l.ia_gyr_pc3 * vol_gyr,
            self.params.nsm_per_ia * l.ia_gyr_pc3 * vol_gyr,
        ]
    }

    /// The supernovae and neutron-star mergers in one cell and bin: the same
    /// whoever asks, in whatever order.
    pub fn cell_events(&self, cell: [i64; 3], bin: i64) -> Result<Vec<GalaxyEvent>, String> {
        let name = format!("{}.{}.{}", cell[0], cell[1], cell[2]);
        let addr = self.key("event_cell", &name)?;
        let count = Draws::new(self.seed, addr.clone(), stream("cosmos.events.count.v1"));
        let d = Draws::new(self.seed, addr, stream("cosmos.events.v1"));
        let (c, b) = (self.params.sn_cell_pc, self.params.sn_bin_myr);
        let lam = self.expected_cell(cell, bin);
        let mut out = vec![];
        for kind in [Kind::CoreCollapse, Kind::TypeIa, Kind::NeutronStarMerger] {
            let s = kind.slot();
            let bi = bin.max(0) as u64;
            let n = poisson(&count, bi * 4 + s, lam[s as usize]).min(1 << 16);
            for j in 0..n {
                let i = |q: u64| (bi << 24) | (s << 20) | (j * 8 + q);
                out.push(GalaxyEvent {
                    kind,
                    id: format!("{name}/{}/{bin}/{j}", s),
                    t_myr: (bin as f64 + d.unit(i(0))) * b,
                    grid_pc: [
                        (cell[0] as f64 + d.unit(i(1))) * c,
                        (cell[1] as f64 + d.unit(i(2))) * c,
                        (cell[2] as f64 + d.unit(i(3))) * c,
                    ],
                });
            }
        }
        Ok(out)
    }

    /// Expected lethal gamma-ray bursts born in one column and bin.
    pub fn expected_column(&self, col: [i64; 2], bin: i64) -> f64 {
        let (c, b) = (self.params.grb_cell_pc, self.params.grb_bin_myr);
        let t = (bin as f64 + 0.5) * b;
        let g = [(col[0] as f64 + 0.5) * c, (col[1] as f64 + 0.5) * c, 0.0];
        let l = self.env.local(self.place(g, t), &self.green);
        let reach = self.env.params.grb_kpc * 1000.0;
        // The envelope gives bursts per Gyr within grb_kpc; per unit area that is / (pi R^2).
        l.grb_per_gyr / (m::PI * reach * reach) * c * c * b / 1000.0
    }

    /// The gamma-ray bursts in one column and bin, near the mid-plane (gas scale height).
    pub fn column_events(&self, col: [i64; 2], bin: i64) -> Result<Vec<GalaxyEvent>, String> {
        let name = format!("{}.{}", col[0], col[1]);
        let addr = self.key("grb_column", &name)?;
        let count = Draws::new(
            self.seed,
            addr.clone(),
            stream("cosmos.events.grb.count.v1"),
        );
        let d = Draws::new(self.seed, addr, stream("cosmos.events.grb.v1"));
        let (c, b) = (self.params.grb_cell_pc, self.params.grb_bin_myr);
        let bi = bin.max(0) as u64;
        let n = poisson(&count, bi, self.expected_column(col, bin)).min(1 << 16);
        let h = self.env.params.gas_height_pc;
        Ok((0..n)
            .map(|j| {
                let i = |q: u64| (bi << 24) | (j * 8 + q);
                let side = if d.unit(i(3)) < 0.5 { -1.0 } else { 1.0 };
                GalaxyEvent {
                    kind: Kind::GammaRayBurst,
                    id: format!("{name}/{bin}/{j}"),
                    t_myr: (bin as f64 + d.unit(i(0))) * b,
                    grid_pc: [
                        (col[0] as f64 + d.unit(i(1))) * c,
                        (col[1] as f64 + d.unit(i(2))) * c,
                        -side * h * m::ln(1.0 - d.unit(i(4))),
                    ],
                }
            })
            .collect())
    }

    /// Galaxy-wide events in a window: mergers, authored starbursts, active-nucleus episodes.
    pub fn galaxy_events(&self, g: &Galaxy, from_myr: f64, to_myr: f64) -> Vec<GalaxyEvent> {
        let inside = |t: f64| t >= from_myr && t < to_myr;
        let mut out = vec![];
        // Ids name where each came from, so inking one event never renames another.
        let removed = |i: usize| {
            self.edits
                .iter()
                .any(|e| matches!(e, Edit::RemoveMerger { index } if *index == i))
        };
        for (i, mg) in self.halo.mergers.iter().enumerate() {
            if !removed(i) && inside(mg.t_gyr * 1000.0) {
                out.push(GalaxyEvent {
                    kind: Kind::Merger,
                    id: format!("merger/{i}"),
                    t_myr: mg.t_gyr * 1000.0,
                    grid_pc: [0.0; 3],
                });
            }
        }
        for (i, e) in self.edits.iter().enumerate() {
            let (kind, t) = match e {
                Edit::Merger { t_gyr, .. } => (Kind::Merger, *t_gyr),
                Edit::Starburst { t_gyr, .. } => (Kind::Starburst, *t_gyr),
                _ => continue,
            };
            if inside(t * 1000.0) {
                out.push(GalaxyEvent {
                    kind,
                    id: format!("edit/{i}"),
                    t_myr: t * 1000.0,
                    grid_pc: [0.0; 3],
                });
            }
        }
        for (i, (t, _)) in g.active_nuclei.iter().enumerate() {
            if inside(t * 1000.0) {
                out.push(GalaxyEvent {
                    kind: Kind::ActiveNucleus,
                    id: format!("nucleus/{i}"),
                    t_myr: t * 1000.0,
                    grid_pc: [0.0; 3],
                });
            }
        }
        out
    }

    /// Everything a site feels from `from_myr` to `to_myr` (by arrival time), in arrival order.
    pub fn exposure(
        &self,
        site: &EventSite,
        from_myr: f64,
        to_myr: f64,
    ) -> Result<Exposure, String> {
        let p = &self.params;
        let e = &self.env.params;
        let harm = e.sterilising_pc;
        let reach = harm.max(p.remnant_pc);
        let shock = p.remnant_kms * KMS_PC_MYR;
        let lag = p.remnant_myr + reach / shock;
        let mut hits = vec![];
        let keep = |h: &Hit| h.activated_myr < to_myr && h.until_myr >= from_myr;

        // Supernova-scale events from every cell the site's path comes within reach of.
        let mut cache: BTreeMap<([i64; 3], i64), Vec<GalaxyEvent>> = BTreeMap::new();
        let b0 = m::floor((from_myr - lag) / p.sn_bin_myr) as i64;
        let b1 = m::floor(to_myr / p.sn_bin_myr) as i64;
        for bin in b0..=b1 {
            let (lo, hi) = self.sweep(&site.path, bin as f64 * p.sn_bin_myr, p.sn_bin_myr, reach);
            let c = p.sn_cell_pc;
            let cr = |k: usize| m::floor(lo[k] / c) as i64..=m::floor(hi[k] / c) as i64;
            for i in cr(0) {
                for j in cr(1) {
                    for k in cr(2) {
                        if let std::collections::btree_map::Entry::Vacant(v) =
                            cache.entry(([i, j, k], bin))
                        {
                            v.insert(self.cell_events([i, j, k], bin)?);
                        }
                    }
                }
            }
        }
        let mut seen = std::collections::BTreeSet::new();
        for ev in cache.values().flatten() {
            if !seen.insert(ev.id.clone()) {
                continue;
            }
            let s = self.site_at(&site.path, ev.t_myr);
            let d = dist(s, ev.grid_pc);
            if d > reach {
                continue;
            }
            let light = ev.t_myr + d / C_PC_MYR;
            let harmful = d <= harm;
            let remnant = d <= p.remnant_pc;
            let what = match ev.kind {
                Kind::CoreCollapse => "a massive star explodes",
                Kind::TypeIa => "a white dwarf explodes",
                _ => "two neutron stars merge in a kilonova, scattering r-process metals",
            };
            let h = Hit {
                kind: ev.kind,
                id: ev.id.clone(),
                occurred_myr: ev.t_myr,
                activated_myr: if harmful { light } else { ev.t_myr + d / shock },
                until_myr: if remnant {
                    ev.t_myr + d / shock + p.remnant_myr
                } else {
                    light
                },
                distance_pc: d,
                harmful,
                remnant,
                shower: false,
                dose: 0.0,
                passage: None,
                why: format!(
                    "{what} {d:.1} pc away{}{}",
                    if harmful {
                        "; its radiation is harmful this close"
                    } else {
                        ""
                    },
                    if remnant {
                        "; its remnant bubble sweeps over, bringing fresh metals"
                    } else {
                        ""
                    }
                ),
            };
            if keep(&h) {
                hits.push(h);
            }
        }

        // Gamma-ray bursts from columns within their lethal distance.
        let grb = e.grb_kpc * 1000.0;
        let g0 = m::floor((from_myr - grb / C_PC_MYR) / p.grb_bin_myr) as i64;
        let g1 = m::floor(to_myr / p.grb_bin_myr) as i64;
        let mut gseen = std::collections::BTreeSet::new();
        for bin in g0..=g1 {
            let (lo, hi) = self.sweep(&site.path, bin as f64 * p.grb_bin_myr, p.grb_bin_myr, grb);
            let c = p.grb_cell_pc;
            let cr = |k: usize| m::floor(lo[k] / c) as i64..=m::floor(hi[k] / c) as i64;
            for i in cr(0) {
                for j in cr(1) {
                    for ev in self.column_events([i, j], bin)? {
                        if !gseen.insert(ev.id.clone()) {
                            continue;
                        }
                        let d = dist(self.site_at(&site.path, ev.t_myr), ev.grid_pc);
                        if d > grb {
                            continue;
                        }
                        let at = ev.t_myr + d / C_PC_MYR;
                        let h = Hit {
                            kind: ev.kind,
                            id: ev.id,
                            occurred_myr: ev.t_myr,
                            activated_myr: at,
                            until_myr: at,
                            distance_pc: d,
                            harmful: true,
                            remnant: false,
                            shower: false,
                            dose: 0.0,
                            passage: None,
                            why: format!("a dying massive star beams a gamma-ray burst across the site from {:.2} kpc; it strips ozone and sterilises surfaces facing it", d / 1000.0),
                        };
                        if keep(&h) {
                            hits.push(h);
                        }
                    }
                }
            }
        }

        // Galaxy-wide: mergers, starbursts, the nucleus (light from the centre).
        let g = self.env.galaxy;
        let from_c = from_myr - 30.0;
        for ev in self.galaxy_events(g, from_c, to_myr) {
            let s = self.site_at(&site.path, ev.t_myr);
            let r = m::hypot(s[0], s[1]);
            let at = ev.t_myr + r / C_PC_MYR;
            let (dose, why) = match ev.kind {
                Kind::ActiveNucleus => {
                    let i: usize = ev.id[8..].parse().unwrap_or(0);
                    let gain = g.active_nuclei.get(i).map_or(0.0, |x| x.1);
                    let rk = (r / 1000.0).max(0.1);
                    let dose = gain / 1.0e6 / (rk * rk);
                    (dose, format!("the central black hole feeds and shines (gaining {gain:.1e} solar masses); dose {dose:.2} here, {:.1} kpc out", r / 1000.0))
                }
                Kind::Merger => (0.0, "another galaxy merges with this one: the bulge grows, gas bursts into stars and old orbits are heated".into()),
                _ => (0.0, "a starburst: gas turns to stars across the galaxy at once".into()),
            };
            let h = Hit {
                kind: ev.kind,
                id: ev.id,
                occurred_myr: ev.t_myr,
                activated_myr: at,
                until_myr: at,
                distance_pc: r,
                harmful: ev.kind == Kind::ActiveNucleus && dose > self.green.agn_dose_max,
                remnant: false,
                shower: false,
                dose,
                passage: None,
                why,
            };
            if keep(&h) {
                hits.push(h);
            }
        }

        hits.extend(self.encounters(site, from_myr, to_myr)?);
        hits.extend(self.arm_crossings(&site.path, from_myr, to_myr));
        hits.sort_by(|a, b| {
            a.activated_myr
                .total_cmp(&b.activated_myr)
                .then_with(|| a.id.cmp(&b.id))
        });
        Ok(Exposure {
            site: site.address.to_string(),
            from_myr,
            to_myr,
            hits,
        })
    }

    /// The box (grid pc) a site sweeps in a bin, grown by a reach.
    fn sweep(&self, s: &SitePath, t0: f64, dt: f64, reach: f64) -> ([f64; 3], [f64; 3]) {
        let mut lo = [f64::INFINITY; 3];
        let mut hi = [f64::NEG_INFINITY; 3];
        for q in 0..=8 {
            let x = self.site_at(s, t0 + dt * f64::from(q) / 8.0);
            for k in 0..3 {
                lo[k] = lo[k].min(x[k]);
                hi[k] = hi[k].max(x[k]);
            }
        }
        // Between samples an orbit drifts at most |v| dt / 16 off the polyline; pad by that.
        let pad = match s {
            SitePath::Fixed { .. } => 0.0,
            SitePath::Orbit(o) => (o.amp_r_pc * o.kappa + o.amp_z_pc * o.nu) * dt / 8.0,
        };
        (
            [
                lo[0] - reach - pad,
                lo[1] - reach - pad,
                lo[2] - reach - pad,
            ],
            [
                hi[0] + reach + pad,
                hi[1] + reach + pad,
                hi[2] + reach + pad,
            ],
        )
    }

    /// Close passages at a site (keyed on the site, by bin).
    pub fn encounters(
        &self,
        site: &EventSite,
        from_myr: f64,
        to_myr: f64,
    ) -> Result<Vec<Hit>, String> {
        let p = &self.params;
        let e = &self.env.params;
        let count = Draws::new(
            self.seed,
            site.address.clone(),
            stream("cosmos.events.encounter.count.v1"),
        );
        let d = Draws::new(
            self.seed,
            site.address.clone(),
            stream("cosmos.events.encounter.v1"),
        );
        let speed = Draws::new(
            self.seed,
            site.address.clone(),
            stream("cosmos.events.encounter.speed.v1"),
        );
        let b0 = m::floor(from_myr / p.encounter_bin_myr).max(0.0) as u64;
        let b1 = m::floor(to_myr / p.encounter_bin_myr).max(0.0) as u64;
        let mut out = vec![];
        for bin in b0..=b1 {
            let t = (bin as f64 + 0.5) * p.encounter_bin_myr;
            let l = self
                .env
                .local(self.place(self.site_at(&site.path, t), t), &self.green);
            let n = poisson(&count, bin, l.encounters_per_myr * p.encounter_bin_myr).min(1 << 16);
            // Mean relative speed back out of the rate: rate = n pi b^2 v.
            let area = m::PI * e.encounter_pc * e.encounter_pc;
            let v_mean = if l.systems_pc3 > 0.0 {
                l.encounters_per_myr / (l.systems_pc3 * area) / KMS_PC_MYR
            } else {
                0.0
            };
            // Maxwellian relative speeds with that mean: sigma_1d = v_mean / sqrt(8 / pi).
            let s1 = v_mean / m::sqrt(8.0 / m::PI);
            for j in 0..n {
                let i = |q: u64| (bin << 24) | (j * 8 + q);
                let tm = (bin as f64 + d.unit(i(0))) * p.encounter_bin_myr;
                if tm < from_myr || tm >= to_myr {
                    continue;
                }
                // Uniform over the target area: b = b_max sqrt(u).
                let b = e.encounter_pc * m::sqrt(d.unit(i(1)));
                let ns = |q: u64| speed.normal((bin << 24) | (j * 4 + q));
                let (x, y, z) = (ns(0), ns(1), ns(2));
                let v = (s1 * m::sqrt(x * x + y * y + z * z)).max(1.0);
                let mass = crate::stars::imf_quantile(&self.stars, d.unit(i(2)));
                let shower = b <= p.shower_pc;
                out.push(Hit {
                    kind: Kind::Encounter,
                    id: format!("{}/encounter/{bin}/{j}", site.address),
                    occurred_myr: tm,
                    activated_myr: tm,
                    until_myr: tm,
                    distance_pc: b,
                    harmful: false,
                    remnant: false,
                    shower,
                    dose: mass / (b.max(1.0e-4) * v),
                    passage: Some([b, v, mass]),
                    why: format!(
                        "a {mass:.2} solar-mass star passes {:.0} au away at {v:.0} km/s{}",
                        b * 206_264.8,
                        if shower { "; it shakes the outer comet cloud and a comet shower falls inward over the next few Myr" } else { "" }
                    ),
                });
            }
        }
        Ok(out)
    }

    /// When the site's path enters and leaves spiral arms.
    pub fn arm_crossings(&self, s: &SitePath, from_myr: f64, to_myr: f64) -> Vec<Hit> {
        let step = self.params.arm_step_myr;
        let e = &self.env.params;
        let inside = |t: f64| {
            let a = self.env.arm(&self.place(self.site_at(s, t), t));
            (a - 1.0) / e.arm_contrast.max(1.0e-9) > self.params.arm_cos
        };
        let mut out = vec![];
        let k0 = m::floor(from_myr / step) as i64;
        let k1 = m::ceil(to_myr / step) as i64;
        let mut was = inside(k0 as f64 * step);
        for k in k0 + 1..=k1 {
            let t = k as f64 * step;
            let now = inside(t);
            if now != was && t >= from_myr && t < to_myr {
                let kind = if now { Kind::ArmEntry } else { Kind::ArmExit };
                out.push(Hit {
                    kind,
                    id: format!("arm/{k}"),
                    occurred_myr: t,
                    activated_myr: t,
                    until_myr: t,
                    distance_pc: 0.0,
                    harmful: false,
                    remnant: false,
                    shower: false,
                    dose: 0.0,
                    passage: None,
                    why: if now {
                        "the site enters a spiral arm: more gas, more star formation, more supernovae and close passages".into()
                    } else {
                        "the site leaves the spiral arm for the quieter space between arms".into()
                    },
                });
            }
            was = now;
        }
        out
    }
}

fn dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let (x, y, z) = (a[0] - b[0], a[1] - b[1], a[2] - b[2]);
    m::sqrt(x * x + y * y + z * z)
}

/// One history curve before and after an edit.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Curve {
    pub name: String,
    pub t_gyr: Vec<f64>,
    pub before: Vec<f64>,
    pub after: Vec<f64>,
    /// Largest |after - before| / max(|before|, tiny), from the first change on.
    pub max_change: f64,
}

/// Before and after: the "ghost" the chart draws behind the redrawn curves.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Compare {
    /// The first history point that differs (None if nothing did).
    pub first_change_gyr: Option<f64>,
    pub morphology: [galaxy::Morphology; 2],
    pub curves: Vec<Curve>,
}

/// Compare two histories of the same halo point by point.
pub fn compare(before: &Galaxy, after: &Galaxy) -> Compare {
    type Read = fn(&galaxy::GalaxyState) -> f64;
    let pick: [(&str, Read); 6] = [
        ("stars", |s| s.stars),
        ("bulge_fraction", |s| {
            if s.stars > 0.0 {
                s.bulge / s.stars
            } else {
                0.0
            }
        }),
        ("gas", |s| s.gas),
        ("sfr", |s| s.sfr),
        ("z_gas", |s| s.z_gas),
        ("black_hole", |s| s.black_hole),
    ];
    let n = before.history.len().min(after.history.len());
    let first = (0..n).find(|&i| before.history[i] != after.history[i]);
    let curves = pick
        .iter()
        .map(|(name, f)| {
            let b: Vec<f64> = before.history[..n].iter().map(f).collect();
            let a: Vec<f64> = after.history[..n].iter().map(f).collect();
            let max_change = first.map_or(0.0, |i0| {
                (i0..n)
                    .map(|i| (a[i] - b[i]).abs() / b[i].abs().max(1.0e-30))
                    .fold(0.0, f64::max)
            });
            Curve {
                name: (*name).into(),
                t_gyr: before.history[..n].iter().map(|s| s.t_gyr).collect(),
                before: b,
                after: a,
                max_change,
            }
        })
        .collect();
    Compare {
        first_change_gyr: first.map(|i| before.history[i].t_gyr),
        morphology: [before.morphology, after.morphology],
        curves,
    }
}

/// Write the GM's edits (authored) and a site's hits (derived) to the chronicle.
pub fn record(
    ch: &mut sse_chronicle::Chronicle,
    galaxy_address: &str,
    edits: &[Edit],
    exposure: &Exposure,
) -> Result<Vec<sse_core::ContentHash>, String> {
    use sse_chronicle::{Event, EventBody, Evidence, Payload, Publisher, UniverseTime};
    use sse_core::cbor::Value;
    let at =
        |myr: f64| UniverseTime::seconds(m::round(myr * 1.0e6 * crate::cosmology::YEAR_S) as i64);
    let gal: Address = galaxy_address.parse().map_err(|e| format!("{e:?}"))?;
    let site: Address = exposure.site.parse().map_err(|e| format!("{e:?}"))?;
    let mut ids = vec![];
    for (i, e) in edits.iter().enumerate() {
        let t = match e {
            Edit::Merger { t_gyr, .. } | Edit::Starburst { t_gyr, .. } => *t_gyr,
            Edit::QuietNucleus { from_gyr, .. } => *from_gyr,
            Edit::RemoveMerger { .. } => 0.0,
        };
        let mut data = BTreeMap::new();
        data.insert("index".into(), Value::Int(i as i128));
        data.insert(
            "edit".into(),
            Value::Text(serde_json::to_string(e).map_err(|x| x.to_string())?),
        );
        let ev = Event::new(EventBody {
            kind: "galaxy.history_edit".into(),
            subject: vec![gal.clone()],
            occurred_at: at(t * 1000.0),
            activated_at: at(t * 1000.0),
            recognised_at: ch.next_stamp(),
            causes: vec![],
            payload: Payload {
                data: Some(Value::Map(data)),
                ..Default::default()
            },
            evidence: Evidence::Authored,
            publisher: Publisher::new("gm", "galaxy-edit", "1"),
        })
        .map_err(|e| format!("{e:?}"))?;
        ids.push(ch.append(ev).map_err(|e| format!("{e:?}"))?);
    }
    for h in &exposure.hits {
        let mut data = BTreeMap::new();
        data.insert("id".into(), Value::Text(h.id.clone()));
        data.insert("distance_pc".into(), Value::Float(h.distance_pc));
        data.insert("harmful".into(), Value::Bool(h.harmful));
        data.insert("remnant".into(), Value::Bool(h.remnant));
        data.insert("shower".into(), Value::Bool(h.shower));
        data.insert("dose".into(), Value::Float(h.dose));
        data.insert(
            "until_s".into(),
            Value::Float(h.until_myr * 1.0e6 * crate::cosmology::YEAR_S),
        );
        data.insert("why".into(), Value::Text(h.why.clone()));
        let ev = Event::new(EventBody {
            kind: h.kind.chronicle_kind().into(),
            subject: vec![site.clone()],
            occurred_at: at(h.occurred_myr),
            activated_at: at(h.activated_myr),
            recognised_at: ch.next_stamp(),
            causes: vec![],
            payload: Payload {
                data: Some(Value::Map(data)),
                ..Default::default()
            },
            evidence: Evidence::Derived,
            publisher: Publisher::new("sse.base-cosmos", "events", "1"),
        })
        .map_err(|e| format!("{e:?}"))?;
        ids.push(ch.append(ev).map_err(|e| format!("{e:?}"))?);
    }
    Ok(ids)
}

/// A scripted scenario ("ink a merger"): everything needed to reproduce it.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    /// `sse.galaxy-scenario/1`.
    pub format: String,
    pub about: String,
    /// Universe id; also the text of every seed.
    pub universe: String,
    /// A median halo of this mass (solar masses) under this key.
    pub halo_mass: f64,
    pub halo_key: String,
    pub edits: Vec<Edit>,
    /// The site: a fraction of the Sun's radius, an azimuth (radians) and height (pc), at
    /// the window's start; it then stays fixed in the co-rotating grid.
    pub site_name: String,
    pub site_r_sun: f64,
    pub site_phi: f64,
    pub site_z_pc: f64,
    pub window_myr: [f64; 2],
}

pub const MANIFEST_FORMAT: &str = "sse.galaxy-scenario/1";

/// Run a manifest: the galaxy before and after its edits, the before/after
/// comparison, the site's exposure both ways, and the chronicle head.
pub fn run_manifest(
    cp: &crate::scenario::CosmologyPack,
    man: &Manifest,
) -> Result<serde_json::Value, String> {
    use crate::cosmology::Cosmology;
    use crate::web::{Web, WebParams};
    if man.format != MANIFEST_FORMAT {
        return Err(format!("expected format {MANIFEST_FORMAT}"));
    }
    let [from, to] = man.window_myr;
    if from >= to || !from.is_finite() || to - from > 4000.0 {
        return Err("the window must run forwards, at most 4 Gyr".into());
    }
    let seed = Seed::from_text(&man.universe);
    let c = Cosmology::new(cp.params.clone());
    let u = Address::universe(&man.universe).map_err(|e| format!("{e:?}"))?;
    let web = Web::new(
        &c,
        WebParams {
            octaves: 4,
            ..cp.web.clone()
        },
        &seed,
        &u,
    )
    .map_err(|e| format!("{e:?}"))?;
    let hs = crate::haloes::Haloes {
        cosmo: &c,
        web: &web,
        params: cp.haloes.clone(),
        seed,
        universe: u,
    };
    let halo = hs.median(man.halo_mass, &man.halo_key);
    galaxy::check_edits(&halo, &man.edits)?;
    let addr: Address = halo.address.parse().map_err(|e| format!("{e:?}"))?;
    let site_addr = addr
        .child_key("site", &man.site_name)
        .map_err(|e| format!("{e:?}"))?;
    let mut ch = sse_chronicle::Chronicle::new();
    let mut out = vec![];
    let mut galaxies = vec![];
    let mut clock = None;
    for edits in [&[][..], &man.edits[..]] {
        let g = galaxy::evolve_with(&c, &cp.galaxy, &halo, edits);
        let clock_total = *clock.get_or_insert_with(|| crate::rings::inflow_clock(&g, &cp.rings));
        let d =
            crate::rings::evolve_clocked(&g, &cp.galaxy, &cp.rings, seed, &addr, Some(clock_total));
        let env = GalaxyEnvelope {
            galaxy: &g,
            disc: &d,
            params: cp.envelope.clone(),
            seed,
            address: addr.clone(),
            remains: Default::default(),
        };
        let ev = Events {
            env: &env,
            halo: &halo,
            edits,
            green: cp.green.clone(),
            stars: cp.stars.clone(),
            params: cp.events.clone(),
            seed,
        };
        let r = man.site_r_sun * env.sun_radius_kpc() * 1000.0;
        let x = [
            r * m::cos(man.site_phi),
            r * m::sin(man.site_phi),
            man.site_z_pc,
        ];
        let site = EventSite {
            address: site_addr.clone(),
            path: SitePath::Fixed {
                grid_pc: ev.to_grid(x, from),
            },
        };
        let x = ev.exposure(&site, from, to)?;
        if !edits.is_empty() {
            record(&mut ch, &halo.address, edits, &x)?;
        }
        let mut counts = BTreeMap::new();
        for h in &x.hits {
            *counts.entry(h.kind).or_insert(0usize) += 1;
        }
        let harmful = x.hits.iter().filter(|h| h.harmful).count();
        let showers = x.hits.iter().filter(|h| h.shower).count();
        let summary = serde_json::json!({
            "morphology": g.morphology,
            "stars": g.now.stars,
            "bulge_fraction": g.now.bulge / g.now.stars.max(1.0),
            "black_hole": g.now.black_hole,
            "sfr": g.now.sfr,
            "galaxy_digest": sse_core::ContentHash::of(&serde_json::to_string(&g).map_err(|e| e.to_string())?).map_err(|e| format!("{e:?}"))?.hex(),
            "hits": counts,
            "harmful": harmful,
            "showers": showers,
            "exposure_digest": sse_core::ContentHash::of(&x).map_err(|e| format!("{e:?}"))?.hex(),
        });
        out.push(summary);
        galaxies.push((g, x));
    }
    let cmp = compare(&galaxies[0].0, &galaxies[1].0);
    let first = cmp.first_change_gyr.unwrap_or(f64::INFINITY) * 1000.0;
    let earliest = man
        .edits
        .iter()
        .map(|e| e.from_gyr(&halo))
        .fold(f64::INFINITY, f64::min)
        * 1000.0;
    // Locality: every hit drawn from a bin that ends before the earliest edit is the
    // same hit (less one ring step: the envelope interpolates to the end of the step).
    let ring_step = cp.galaxy.step_myr * f64::from(cp.rings.every.max(1));
    let cut = cp.events.before_bin(earliest - ring_step);
    let early = |x: &Exposure| -> Vec<Hit> {
        x.hits
            .iter()
            .filter(|h| h.occurred_myr < cut)
            .cloned()
            .collect()
    };
    let changes: Vec<serde_json::Value> = cmp
        .curves
        .iter()
        .map(|cv| serde_json::json!({ "curve": cv.name, "max_change": cv.max_change }))
        .collect();
    Ok(serde_json::json!({
        "format": "sse.galaxy-scenario-report/1",
        "halo": halo.address,
        "site": site_addr.to_string(),
        "window_myr": man.window_myr,
        "edits": man.edits,
        "before": out[0],
        "after": out[1],
        "first_change_gyr": cmp.first_change_gyr,
        "edit_from_gyr": earliest / 1000.0,
        "history_unchanged_before_edit": first >= earliest,
        "events_unchanged_before_edit": early(&galaxies[0].1) == early(&galaxies[1].1),
        "morphology": cmp.morphology,
        "changes": changes,
        "chronicle_head": ch.head().hex(),
    }))
}
