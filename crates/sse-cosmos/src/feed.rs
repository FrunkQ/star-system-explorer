//! GH3: progenitors feed the host (galaxy histories design section 7.5).
//!
//! Every member of a host's assembly tree (GH1) brings its own galaxy (G3 on
//! its own halo) and follows its own orbit (GH2). What reaches the host, and
//! when, is read off that orbit:
//! - stars it loses along the way join the **stellar halo** at the slice
//!   they leave;
//! - gas pushed out by the host's hot gas joins the host's **hot reservoir**
//!   when its star formation stops, and cools back in through G3's return;
//! - what is left when it reaches the centre (stars, gas and its black hole)
//!   arrives in one **merger**, which the host answers by its ratios:
//!   bulge growth, a burst, fresh gas, disc heating;
//! - every closest approach and every pass through the disc is a **pass**,
//!   which can heat the disc, start a burst and leave a mark.
//!
//! Each response writes a dated [`Mark`] (design 9.4) for the renderer.
//! Progenitors' own galaxies are fed by nothing (recursion one level deep);
//! that and the response shapes are fudge F47.

use crate::assembly::{Progenitor, Tree, FILL_STREAM};
use crate::cosmology::Cosmology;
use crate::galaxy::{Galaxy, GalaxyParams, GalaxyState};
use crate::haloes::{Halo, Haloes};
use crate::infall::{self, Fate, Grip, Host, OrbitParams, Pass};
use serde::{Deserialize, Serialize};
use sse_core::{Address, Draws, Stream};
use std::collections::BTreeMap;

/// How strongly the host answers, and how long its marks last.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct ResponseParams {
    /// Disc heating: the share of thin-disc stars moved to the thick disc
    /// is `heat_k` x (bound mass over disc mass) at a merger, and the same
    /// times (v_c / v_z) at a pass through the disc, at most all of them
    /// (after Toth and Ostriker 1992). Stand-in, F47: 0.02 leaves a
    /// Milky-Way-mass host with no great merger a thick disc of about a
    /// tenth to a fifth of its disc (the Milky Way's is about a sixth;
    /// Bland-Hawthorn and Gerhard 2016, doi:10.1146/annurev-astro-081915-023441).
    pub heat_k: f64,
    /// The same at a merger, which heats the disc there and then most
    /// (the Gaia Sausage heated most of the disc it found into the "Splash";
    /// Belokurov et al. 2020, doi:10.1093/mnras/staa876). Stand-in, F47.
    pub heat_merge_k: f64,
    /// A closest approach squeezes the gas: a burst of `burst_fraction`
    /// (G3's) times the bound mass over the host's mass inside the pass.
    /// Off by default: G3's burst share is set for whole mergers, and at
    /// every pass it makes small hosts three times too many stars.
    pub pass_bursts: bool,
    /// Marks weaker than this are not listed.
    pub mark_floor: f64,
    /// Per kind: the driving number at which a mark has half its strength,
    /// and how long it takes to fade by a factor e (0 = for good).
    pub marks: BTreeMap<String, MarkKind>,
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct MarkKind {
    pub half: f64,
    pub fades_gyr: f64,
}

impl Default for ResponseParams {
    fn default() -> Self {
        let k = |half: f64, fades_gyr: f64| MarkKind { half, fades_gyr };
        let marks = [
            // Design 9.4's table; lasting times from its "Lasts" column.
            ("warp", k(0.05, 1.0)),
            ("ripples", k(0.02, 1.0)),
            ("tilt", k(0.1, 4.0)),
            ("thick_disc", k(0.1, 0.0)),
            ("driven_arms", k(0.05, 0.5)),
            ("ring", k(0.05, 0.3)),
            ("burst_knots", k(0.05, 0.05)),
            ("lopsided", k(0.05, 1.0)),
            ("gas_trail", k(0.01, 0.5)),
            ("warm_side", k(0.1, 0.1)),
            ("fresh_gas", k(0.1, 2.0)),
        ]
        .into_iter()
        .map(|(n, v)| (n.to_string(), v))
        .collect();
        ResponseParams {
            heat_k: 0.02,
            heat_merge_k: 0.1,
            pass_bursts: false,
            mark_floor: 1.0e-2,
            marks,
        }
    }
}

impl ResponseParams {
    /// A mark's strength from its driving number: x / (x + half).
    fn strength(&self, kind: &str, x: f64) -> (f64, f64) {
        let k = self.marks.get(kind).copied().unwrap_or(MarkKind {
            half: 1.0,
            fades_gyr: 1.0,
        });
        let x = x.max(0.0);
        (x / (x + k.half.max(1.0e-12)), k.fades_gyr)
    }
}

/// One scar on a galaxy (design 9.4): the renderer draws these and nothing else.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Mark {
    pub kind: String,
    /// The progenitor that made it.
    pub caused_by: String,
    pub began_gyr: f64,
    /// 0 to 1 when it began.
    pub strength: f64,
    /// It falls by a factor e over this; `None` lasts for good.
    pub fades_gyr: Option<f64>,
    /// A few numbers for its shape (radius, sense of turning, speed).
    pub shape: BTreeMap<String, f64>,
}

impl Mark {
    /// Its strength at a time: nothing before it began.
    pub fn strength_at(&self, t_gyr: f64) -> f64 {
        if t_gyr < self.began_gyr {
            return 0.0;
        }
        match self.fades_gyr {
            Some(f) if f > 0.0 => {
                self.strength * sse_core::math::exp(-(t_gyr - self.began_gyr) / f)
            }
            _ => self.strength,
        }
    }
}

/// One member of the tree with its galaxy and its orbit.
#[derive(Clone, PartialEq, Debug)]
pub struct Member {
    pub progenitor: Progenitor,
    pub halo: Halo,
    pub galaxy: Galaxy,
    pub fate: Fate,
}

/// The galaxy's state at a time (the history point at or before it).
pub fn state_at(g: &Galaxy, t_gyr: f64) -> &GalaxyState {
    g.history
        .iter()
        .take_while(|s| s.t_gyr <= t_gyr)
        .last()
        .unwrap_or(&g.history[0])
}

/// Every member's galaxy and orbit in the host's potential, followed to
/// `t_to` (the host's own galaxy gives the potential: one pass, F47).
#[allow(clippy::too_many_arguments)]
pub fn members(
    c: &Cosmology,
    hs: &Haloes,
    gp: &GalaxyParams,
    op: &OrbitParams,
    host: &Halo,
    host_galaxy: &Galaxy,
    tree: &Tree,
    t_to: f64,
) -> Result<(Host, Vec<Member>), String> {
    let host_addr: Address = host.address.parse().map_err(|e| format!("{e:?}"))?;
    let d = Draws::new(
        hs.seed,
        host_addr,
        Stream::new(FILL_STREAM).map_err(|e| e.to_string())?,
    );
    let first = tree
        .progenitors
        .iter()
        .map(|p| p.t_infall_gyr)
        .fold(t_to, f64::min);
    let pot = Host::new(c, gp, host, host_galaxy, first, t_to, op);
    let mut out = vec![];
    for p in &tree.progenitors {
        let k0 = p.draw_base;
        let u = [
            d.unit(k0 + 2),
            d.unit(k0 + 3),
            d.unit(k0 + 4),
            d.unit(k0 + 5),
            d.unit(k0 + 6),
        ];
        let m0 = crate::assembly::mass_today_for(c, p.mass_at_infall_msun, p.z_infall);
        let paddr: Address = p.address.parse().map_err(|e| format!("{e:?}"))?;
        let halo = hs.median_at(m0, &paddr);
        let g = crate::galaxy::evolve(c, gp, &halo);
        let at = state_at(&g, p.t_infall_gyr);
        let grip = Grip {
            stars_msun: at.stars,
            gas_msun: at.gas,
            disc_kpc: at.disc_kpc,
            gas_scale: gp.structure.gas_scale_factor,
        };
        let body = infall::infall(
            c,
            gp,
            op,
            host.mass_at(p.z_infall),
            &p.address,
            p.mass_at_infall_msun,
            p.z_infall,
            p.t_infall_gyr,
            u,
            Some(grip),
        );
        let fate = infall::follow(&pot, &body, t_to, op);
        out.push(Member {
            progenitor: p.clone(),
            halo,
            galaxy: g,
            fate,
        });
    }
    Ok((pot, out))
}

/// What reaches the host, in time order.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Arrival {
    /// Stars torn off join the stellar halo.
    Halo {
        t_gyr: f64,
        from: String,
        stars: f64,
        z: f64,
    },
    /// Gas pushed out joins the host's hot reservoir.
    Hot {
        t_gyr: f64,
        from: String,
        gas: f64,
        z: f64,
    },
    /// What is left reaches the centre.
    Merge {
        t_gyr: f64,
        from: String,
        bound_msun: f64,
        stars: f64,
        z_stars: f64,
        gas: f64,
        z_gas: f64,
        black_hole: f64,
        prograde: f64,
    },
    /// A closest approach or a pass through the disc.
    Pass { from: String, pass: Pass },
    /// It falls in: the halo's growth already counted its baryons as smooth
    /// inflow, so the part already in its galaxy (which arrives later, as
    /// above) is taken back from the host's inflow (budget closure).
    Infall {
        t_gyr: f64,
        from: String,
        baryons: f64,
    },
}

impl Arrival {
    pub fn t_gyr(&self) -> f64 {
        match self {
            Arrival::Halo { t_gyr, .. }
            | Arrival::Hot { t_gyr, .. }
            | Arrival::Merge { t_gyr, .. }
            | Arrival::Infall { t_gyr, .. } => *t_gyr,
            Arrival::Pass { pass, .. } => pass.t_gyr,
        }
    }
}

/// The mean metal fraction of a galaxy's stars formed by a time (each
/// history step's new stars at that step's gas metals).
fn star_metals(g: &Galaxy, t_gyr: f64) -> f64 {
    let (mut m, mut z) = (0.0_f64, 0.0_f64);
    let mut prev = 0.0;
    for s in g.history.iter().take_while(|s| s.t_gyr <= t_gyr) {
        let new = (s.stars - prev).max(0.0);
        m += new;
        z += new * s.z_gas;
        prev = s.stars;
    }
    if m > 0.0 {
        z / m
    } else {
        0.0
    }
}

/// What each member brings, read off its orbit.
pub fn arrivals(members: &[Member]) -> Vec<Arrival> {
    let mut out = vec![];
    for mb in members {
        let f = &mb.fate;
        let from = mb.progenitor.address.clone();
        let g = &mb.galaxy;
        // Its own star formation runs until its gas is pushed out.
        let stop = f.quench_gyr.unwrap_or(f.t_gyr).min(f.t_gyr);
        let at = state_at(g, stop);
        let stars0 = at.stars;
        let z_stars = star_metals(g, stop);
        let fell = state_at(g, mb.progenitor.t_infall_gyr);
        out.push(Arrival::Infall {
            t_gyr: mb.progenitor.t_infall_gyr,
            from: from.clone(),
            baryons: fell.stars + fell.gas,
        });
        // Stars leave at each slice by what the orbit took.
        for i in 1..f.kept.len().min(f.track.len()) {
            let lost = (f.kept[i - 1] - f.kept[i]).max(0.0) * stars0;
            if lost > 0.0 {
                out.push(Arrival::Halo {
                    t_gyr: f.track[i].0,
                    from: from.clone(),
                    stars: lost,
                    z: z_stars,
                });
            }
        }
        // Gas pushed out when its star formation stopped (if before the end).
        let mut gas_left = at.gas;
        if let Some(q) = f.quench_gyr {
            if q < f.t_gyr || f.merged_gyr.is_none() {
                out.push(Arrival::Hot {
                    t_gyr: q,
                    from: from.clone(),
                    gas: at.gas,
                    z: at.z_gas,
                });
                gas_left = 0.0;
            }
        }
        for p in &f.passes {
            out.push(Arrival::Pass {
                from: from.clone(),
                pass: p.clone(),
            });
        }
        if let Some(tm) = f.merged_gyr {
            let kept = f.kept.last().copied().unwrap_or(1.0);
            let prograde = f.passes.last().map_or(0.0, |p| p.prograde);
            out.push(Arrival::Merge {
                t_gyr: tm,
                from: from.clone(),
                bound_msun: f.dark_kept * mb.progenitor.mass_at_infall_msun,
                stars: kept * stars0,
                z_stars,
                gas: gas_left,
                z_gas: at.z_gas,
                black_hole: at.black_hole,
                prograde,
            });
        }
    }
    out.sort_by(|a, b| a.t_gyr().total_cmp(&b.t_gyr()));
    out
}

/// What the feed left in a galaxy, read now.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Assembled {
    /// Stars not born here (stellar halo plus those that merged in).
    pub exsitu_msun: f64,
    pub stellar_halo_msun: f64,
    /// Mean metal fraction of the stellar halo, and of all stars not born here.
    pub stellar_halo_z: f64,
    pub exsitu_z: f64,
    pub thick_disc_msun: f64,
    /// The member that gave most of the stars not born here, and its share.
    pub biggest: String,
    pub biggest_share: f64,
    pub merged: u32,
    pub torn_apart: u32,
    pub members: u32,
    pub marks: Vec<Mark>,
}

/// The host's running state the feed acts on (galaxy.rs steps it).
pub(crate) struct Feeding<'a> {
    pub rp: &'a ResponseParams,
    pub arrivals: &'a [Arrival],
    pub next: usize,
    pub out: Assembled,
    pub halo_metals: f64,
    pub exsitu_metals: f64,
    pub by_member: BTreeMap<String, f64>,
    /// Baryons owed back from the inflow (Infall), carried until paid.
    pub owed: f64,
    pub next_infall: usize,
}

/// The host's reservoirs one feed event can touch.
pub(crate) struct Pools<'a> {
    pub gas: &'a mut f64,
    pub stars: &'a mut f64,
    pub bulge: &'a mut f64,
    pub metals: &'a mut f64,
    pub bh: &'a mut f64,
    pub ejected: &'a mut f64,
    pub ejected_metals: &'a mut f64,
    pub inflow: &'a mut f64,
    pub agn: &'a mut Vec<(f64, f64)>,
}

impl Feeding<'_> {
    fn mark(&mut self, kind: &str, from: &str, t: f64, x: f64, shape: &[(&str, f64)]) {
        let (s, fades) = self.rp.strength(kind, x);
        if s < self.rp.mark_floor {
            return;
        }
        self.out.marks.push(Mark {
            kind: kind.into(),
            caused_by: from.into(),
            began_gyr: t,
            strength: s,
            fades_gyr: (fades > 0.0).then_some(fades),
            shape: shape.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        });
    }

    fn heat(&mut self, pl: &Pools, f: f64) -> f64 {
        let thin = (*pl.stars - *pl.bulge - self.out.thick_disc_msun).max(0.0);
        let moved = thin * f.clamp(0.0, 1.0);
        self.out.thick_disc_msun += moved;
        moved
    }

    /// The inflow this step after paying what infalls owe up to `t_end`.
    pub fn pay(&mut self, t_end: f64, inflow: f64) -> f64 {
        while self.next_infall < self.arrivals.len()
            && self.arrivals[self.next_infall].t_gyr() < t_end
        {
            if let Arrival::Infall { baryons, .. } = self.arrivals[self.next_infall] {
                self.owed += baryons;
            }
            self.next_infall += 1;
        }
        let paid = self.owed.min(inflow);
        self.owed -= paid;
        inflow - paid
    }

    /// Apply every event before `t_end`.
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        t_end: f64,
        mh: f64,
        gp: &GalaxyParams,
        quiet: &dyn Fn(f64) -> bool,
        pl: &mut Pools,
    ) {
        while self.next < self.arrivals.len() && self.arrivals[self.next].t_gyr() < t_end {
            let a = self.arrivals[self.next].clone();
            self.next += 1;
            match a {
                Arrival::Halo { from, stars, z, .. } => {
                    *pl.stars += stars;
                    self.out.stellar_halo_msun += stars;
                    self.out.exsitu_msun += stars;
                    self.halo_metals += stars * z;
                    self.exsitu_metals += stars * z;
                    *self.by_member.entry(from).or_default() += stars;
                }
                Arrival::Infall { .. } => {}
                Arrival::Hot {
                    t_gyr,
                    from,
                    gas,
                    z,
                } => {
                    *pl.ejected += gas;
                    *pl.ejected_metals += gas * z;
                    self.mark("gas_trail", &from, t_gyr, gas / pl.gas.max(1.0), &[]);
                }
                Arrival::Merge {
                    t_gyr,
                    from,
                    bound_msun,
                    stars,
                    z_stars,
                    gas,
                    z_gas,
                    black_hole,
                    prograde,
                } => {
                    self.out.merged += 1;
                    let mu = (bound_msun + stars + gas) / mh.max(1.0);
                    let disc = (*pl.stars - *pl.bulge).max(0.0) + *pl.gas;
                    let f_gas = (*pl.gas + gas) / (*pl.gas + gas + *pl.stars + stars).max(1.0);
                    // Bulge growth by the mass ratio, less when gas-rich (Hopkins et al. 2009).
                    let thin_before = (*pl.stars - *pl.bulge).max(0.0);
                    let moved = thin_before * mu.min(1.0) * (1.0 - f_gas);
                    *pl.bulge += moved;
                    if thin_before > 0.0 {
                        self.out.thick_disc_msun *= 1.0 - moved / thin_before;
                    }
                    // Its stars settle at the centre; its gas and metals join.
                    *pl.stars += stars;
                    *pl.bulge += stars;
                    self.out.exsitu_msun += stars;
                    self.exsitu_metals += stars * z_stars;
                    *self.by_member.entry(from.clone()).or_default() += stars;
                    *pl.gas += gas;
                    *pl.metals += gas * z_gas;
                    *pl.inflow += gas;
                    // Heating.
                    let fh = self.rp.heat_merge_k * (bound_msun + stars + gas) / disc.max(1.0);
                    let heated = self.heat(pl, fh);
                    // The burst, and the holes merge.
                    let before = *pl.gas;
                    let burst = (gp.burst_fraction * mu.min(1.0) * *pl.gas).min(*pl.gas);
                    if *pl.gas > 0.0 {
                        *pl.metals -= *pl.metals * burst / *pl.gas;
                    }
                    *pl.gas -= burst;
                    let born = (1.0 - gp.return_fraction) * burst;
                    *pl.stars += born;
                    *pl.bulge += born;
                    *pl.bh += black_hole;
                    if !quiet(t_gyr) {
                        let feed = gp.black_hole_feed * burst;
                        *pl.bh += feed;
                        pl.agn.push((t_gyr, feed));
                    }
                    let thin = (*pl.stars - *pl.bulge).max(1.0);
                    self.mark("thick_disc", &from, t_gyr, heated / thin, &[]);
                    self.mark("tilt", &from, t_gyr, mu, &[("prograde", prograde)]);
                    self.mark("lopsided", &from, t_gyr, mu, &[]);
                    self.mark("burst_knots", &from, t_gyr, burst / before.max(1.0), &[]);
                    if gas > 0.0 {
                        // Fresh gas dilutes the disc's metals (two families of disc stars).
                        let x = gas / (*pl.gas).max(1.0);
                        self.mark("fresh_gas", &from, t_gyr, x, &[("z_in", z_gas)]);
                    }
                }
                Arrival::Pass { from, pass: p } => {
                    let shape = [
                        ("r_kpc", p.big_r_kpc),
                        ("prograde", p.prograde),
                        ("speed_kms", p.speed_kms),
                    ];
                    if p.disc {
                        let x = p.bound_msun / p.disc_msun.max(1.0)
                            * (p.v_c_kms / p.vz_kms.max(1.0)).min(10.0);
                        self.heat(pl, self.rp.heat_k * x);
                        self.mark("ripples", &from, p.t_gyr, x, &shape);
                        // A fast, straight pass near the centre punches a ring.
                        let straight = {
                            let q = p.vz_kms / p.speed_kms.max(1.0);
                            q * q
                        };
                        let inner = sse_core::math::exp(-p.big_r_kpc / p.disc_kpc.max(1.0e-3));
                        self.mark("ring", &from, p.t_gyr, x * straight * inner, &shape);
                    } else {
                        let mu = p.bound_msun / p.host_inside_msun.max(1.0);
                        self.mark("warp", &from, p.t_gyr, mu, &shape);
                        let slow = (p.v_c_kms / p.speed_kms.max(1.0)).min(1.0);
                        let fwd = 0.5 * (1.0 + p.prograde);
                        self.mark("driven_arms", &from, p.t_gyr, mu * slow * fwd, &shape);
                        self.mark("warm_side", &from, p.t_gyr, mu, &shape);
                        if self.rp.pass_bursts {
                            let before = *pl.gas;
                            let burst = (gp.burst_fraction * mu.min(1.0) * *pl.gas).min(*pl.gas);
                            if *pl.gas > 0.0 {
                                *pl.metals -= *pl.metals * burst / *pl.gas;
                            }
                            *pl.gas -= burst;
                            *pl.stars += (1.0 - gp.return_fraction) * burst;
                            self.mark(
                                "burst_knots",
                                &from,
                                p.t_gyr,
                                burst / before.max(1.0),
                                &shape,
                            );
                        }
                    }
                }
            }
        }
    }

    pub fn finish(mut self) -> Assembled {
        let h = self.out.stellar_halo_msun;
        self.out.stellar_halo_z = if h > 0.0 { self.halo_metals / h } else { 0.0 };
        let x = self.out.exsitu_msun;
        self.out.exsitu_z = if x > 0.0 { self.exsitu_metals / x } else { 0.0 };
        if let Some((k, v)) = self
            .by_member
            .iter()
            .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(a.0)))
        {
            self.out.biggest = k.clone();
            self.out.biggest_share = if x > 0.0 { v / x } else { 0.0 };
        }
        self.out
    }
}

/// The tree, its members and what reached the host, for a host halo.
pub struct Fed {
    pub tree: Tree,
    pub members: Vec<Member>,
    pub arrivals: Vec<Arrival>,
    pub galaxy: Galaxy,
}

/// A host galaxy fed by its tree (Chart level), with authored edits.
#[allow(clippy::too_many_arguments)]
pub fn evolve_fed(
    c: &Cosmology,
    hs: &Haloes,
    gp: &GalaxyParams,
    op: &OrbitParams,
    rp: &ResponseParams,
    host: &Halo,
    edits: &[crate::galaxy::Edit],
) -> Result<Fed, String> {
    let unfed = crate::galaxy::evolve(c, gp, host);
    let now = unfed.now.t_gyr;
    let tree = crate::assembly::tree(
        c,
        hs.seed,
        host,
        unfed.now.z,
        &hs.params.assembly,
        crate::assembly::Level::Chart,
    )?;
    let (_, mut members) = members(c, hs, gp, op, host, &unfed, &tree, now)?;
    // A removed G2 merger takes its pin out of the tree.
    let removed: Vec<usize> = edits
        .iter()
        .filter_map(|e| match e {
            crate::galaxy::Edit::RemoveMerger { index } => Some(*index),
            _ => None,
        })
        .collect();
    members.retain(|mb| {
        !removed
            .iter()
            .any(|i| mb.progenitor.address.ends_with(&format!("/prog:m{i}")))
    });
    let arr = arrivals(&members);
    let mut galaxy = crate::galaxy::evolve_feeding(c, gp, host, edits, Some((rp, &arr)));
    if let Some(a) = galaxy.assembled.as_mut() {
        a.members = members.len() as u32;
        a.torn_apart = members
            .iter()
            .filter(|mb| mb.fate.disrupted_gyr.is_some())
            .count() as u32;
    }
    Ok(Fed {
        tree,
        members,
        arrivals: arr,
        galaxy,
    })
}
