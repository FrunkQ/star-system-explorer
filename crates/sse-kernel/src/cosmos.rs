//! `galaxy_chart` and `web_slice`: the cosmos suite (G1 to G4) read through
//! the kernel for the galaxy chart (U6) and the foam and cluster maps (V9).
//! Read only: nothing is written to the chronicle. Every number comes from
//! sse-cosmos's own functions with the open universe's seed; this file only
//! picks what a chart needs and shapes it.

use crate::err;
use crate::protocol::Response;
use sse_core::{math as m, Address, Seed};
use sse_cosmos::cosmology::Cosmology;
use sse_cosmos::envelope::{GalaxyEnvelope, Place};
use sse_cosmos::haloes::Haloes;
use sse_cosmos::rings::{self, GROUPS};
use sse_cosmos::scenario::{base_cosmology_pack, CosmologyPack};
use sse_cosmos::web::{Web, WebParams};

/// The named haloes a GM can chart before a universe has its own galaxy list:
/// the median halo of each mass, as G3's golden uses them.
pub const HALOES: [(&str, f64); 3] = [
    ("dwarf", 1.0e11),
    ("milky-way-like", 1.0e12),
    ("group", 1.0e13),
];

/// The cosmology's present, Gyr after the origin (DM6's dated calls).
pub(crate) fn now_gyr() -> Result<f64, Response> {
    Ok(settings()?.params.now_gyr)
}

fn settings() -> Result<CosmologyPack, Response> {
    CosmologyPack::from_packs(&[base_cosmology_pack()]).map_err(|e| err("bad_pack", e))
}

/// The cosmology and its tables (the sigma(M) table is 281 integrals, about
/// a quarter of a second in a browser). Every galaxy, web and star request
/// builds one from the same parameters, so the last one is kept: the tables
/// are a pure function of the parameters, so keeping them changes no answer.
pub(crate) fn cosmology(p: &sse_cosmos::cosmology::Params) -> Cosmology {
    thread_local! {
        static LAST: std::cell::RefCell<Option<Cosmology>> = const { std::cell::RefCell::new(None) };
    }
    LAST.with(|l| {
        let mut l = l.borrow_mut();
        match l.as_ref() {
            Some(c) if &c.p == p => c.clone(),
            _ => {
                let c = Cosmology::new(p.clone());
                *l = Some(c.clone());
                c
            }
        }
    })
}

thread_local! {
    /// The last few fed galaxies (GH3): a fed galaxy follows its whole tree,
    /// a tenth of a second or more, and every look at the home galaxy needs
    /// it. Keyed by everything it is made from.
    static FED: std::cell::RefCell<Vec<(String, std::rc::Rc<sse_cosmos::galaxy::Galaxy>)>> =
        const { std::cell::RefCell::new(Vec::new()) };
}

/// A galaxy fed by its assembly tree (GH3), with authored edits.
pub(crate) fn fed_galaxy(
    c: &Cosmology,
    cp: &CosmologyPack,
    hs: &Haloes,
    h: &sse_cosmos::haloes::Halo,
    edits: &[sse_cosmos::galaxy::Edit],
) -> Result<std::rc::Rc<sse_cosmos::galaxy::Galaxy>, Response> {
    let key = serde_json::to_string(&(
        format!("{:?}", hs.seed),
        hs.universe.to_string(),
        h,
        cp,
        edits,
    ))
    .map_err(|e| err("internal", e.to_string()))?;
    if let Some(g) = FED.with(|f| {
        f.borrow()
            .iter()
            .find(|(k, _)| *k == key)
            .map(|e| e.1.clone())
    }) {
        return Ok(g);
    }
    let fed = sse_cosmos::feed::evolve_fed(
        c,
        hs,
        &cp.galaxy,
        &cp.sky.satellites.orbits,
        &cp.sky.satellites.responses,
        h,
        edits,
    )
    .map_err(|e| err("bad_address", e))?;
    let mut fed = fed;
    // GH4: what it tore apart stays as places in its envelope.
    sse_cosmos::feed::with_remains(
        &mut fed,
        &cp.galaxy,
        &cp.sky.satellites.orbits,
        &cp.rings,
        &cp.sky.satellites.remains,
        hs.seed,
    );
    let g = std::rc::Rc::new(fed.galaxy);
    FED.with(|f| {
        let mut f = f.borrow_mut();
        f.insert(0, (key, g.clone()));
        f.truncate(4);
    });
    Ok(g)
}

/// The sky draw's settings (Y2).
pub(crate) fn sky_params() -> Result<sse_cosmos::sky::SkyParams, Response> {
    Ok(settings()?.sky)
}

fn web(c: &Cosmology, cp: &CosmologyPack, seed: &Seed, root: &Address) -> Result<Web, Response> {
    Web::new(
        c,
        WebParams {
            octaves: cp.web.octaves.min(4),
            ..cp.web.clone()
        },
        seed,
        root,
    )
    .map_err(|e| err("bad_request", e))
}

pub(crate) struct GalaxyAsk<'a> {
    pub seed: &'a Seed,
    pub root: &'a Address,
    pub halo: Option<&'a str>,
    pub mass_msun: Option<f64>,
    pub at_gyr: Option<f64>,
    pub place_kpc: Option<f64>,
    pub place_phi: Option<f64>,
    /// A halo cell (V9's cluster map): `halo` is then one of its catalogued haloes.
    pub cell: Option<[i64; 3]>,
    /// The GM's history edits, inked in this view (G6); empty for the galaxy as grown.
    pub edits: &'a [sse_cosmos::galaxy::Edit],
}

/// One galaxy: its history curves, its rings now (abundances, deaths, rates),
/// its green band through time, the cosmic epochs, and the conditions at one
/// place when asked.
pub(crate) fn galaxy_chart(a: GalaxyAsk) -> Result<serde_json::Value, Response> {
    let cp = settings()?;
    let c = cosmology(&cp.params);
    let w = web(&c, &cp, a.seed, a.root)?;
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: cp.haloes.clone(),
        seed: *a.seed,
        universe: a.root.clone(),
    };
    let catalogued = match (a.cell, a.halo) {
        (Some(idx), Some(k)) => {
            let cell = hs.cell(idx);
            Some(
                cell.haloes
                    .into_iter()
                    .find(|h| h.address == k || h.address.ends_with(&format!("/{k}")))
                    .ok_or_else(|| {
                        err("unknown_halo", format!("no halo {k} in {}", cell.address))
                    })?,
            )
        }
        (Some(_), None) => return Err(err("bad_request", "with a cell, name one of its haloes")),
        _ => None,
    };
    let h = match catalogued {
        Some(h) => h,
        None => named_halo(&hs, a.halo, a.mass_msun)?,
    };
    galaxy_chart_of(&c, &cp, &hs, a, h)
}

fn named_halo(
    hs: &Haloes,
    halo: Option<&str>,
    mass_msun: Option<f64>,
) -> Result<sse_cosmos::haloes::Halo, Response> {
    let (key, mass) = match (halo, mass_msun) {
        (Some(k), None) => HALOES
            .iter()
            .find(|(n, _)| *n == k)
            .map(|(n, m)| (n.to_string(), *m))
            .ok_or_else(|| {
                err(
                    "unknown_halo",
                    format!(
                        "no halo {k}; named haloes: {}",
                        HALOES.iter().map(|h| h.0).collect::<Vec<_>>().join(", ")
                    ),
                )
            })?,
        (None, Some(m)) if m > 1.0e8 && m < 1.0e16 => (format!("m{m:e}"), m),
        (None, Some(_)) => return Err(err("bad_request", "mass_msun must be 1e8 to 1e16")),
        (None, None) => ("milky-way-like".to_string(), 1.0e12),
        _ => return Err(err("bad_request", "give a halo name or a mass, not both")),
    };
    Ok(hs.median(mass, &key))
}

fn galaxy_chart_of(
    c: &Cosmology,
    cp: &CosmologyPack,
    hs: &Haloes,
    a: GalaxyAsk,
    h: sse_cosmos::haloes::Halo,
) -> Result<serde_json::Value, Response> {
    sse_cosmos::galaxy::check_edits(&h, a.edits).map_err(|e| err("bad_edit", e))?;
    let g = fed_galaxy(c, cp, hs, &h, a.edits)?;
    let g = &*g;
    // Before and after, when the GM has inked history: the ghost the chart draws.
    let compare = if a.edits.is_empty() {
        None
    } else {
        Some(sse_cosmos::events::compare(
            &*fed_galaxy(c, cp, hs, &h, &[])?,
            g,
        ))
    };
    let addr: Address = h
        .address
        .parse()
        .map_err(|e| err("bad_address", format!("{e:?}")))?;
    let d = rings::evolve(g, &cp.galaxy, &cp.rings, *a.seed, &addr);
    let env = GalaxyEnvelope {
        galaxy: g,
        disc: &d,
        params: cp.envelope.clone(),
        seed: *a.seed,
        address: addr.clone(),
        remains: Default::default(),
    };
    let now = g.now.t_gyr;
    // Galaxy-wide events over the whole history (mergers, inked starbursts, nucleus episodes).
    let events = sse_cosmos::events::Events {
        env: &env,
        halo: &h,
        edits: a.edits,
        green: cp.green.clone(),
        stars: cp.stars.clone(),
        params: cp.events.clone(),
        seed: *a.seed,
    }
    .galaxy_events(g, 0.0, now * 1000.0 + 1.0);
    let removed = |i: usize| {
        a.edits
            .iter()
            .any(|e| matches!(e, sse_cosmos::galaxy::Edit::RemoveMerger { index } if *index == i))
    };
    let mergers: Vec<serde_json::Value> = h
        .mergers
        .iter()
        .enumerate()
        .map(|(i, mg)| {
            serde_json::json!({ "index": i, "t_gyr": mg.t_gyr, "ratio": mg.ratio, "removed": removed(i) })
        })
        .collect();
    let at = a.at_gyr.unwrap_or(now).clamp(g.t_start_gyr, now);
    let sun_r = env.sun_radius_kpc();
    let ring_rows: Vec<serde_json::Value> = d
        .rings
        .iter()
        .map(|r| {
            let ab: serde_json::Map<String, serde_json::Value> = GROUPS
                .iter()
                .enumerate()
                .map(|(i, n)| (n.to_string(), serde_json::json!(r.abundance(i))))
                .collect();
            serde_json::json!({
                "inner_kpc": r.inner_kpc,
                "outer_kpc": r.outer_kpc,
                "radius_kpc": r.radius_kpc,
                "gas": r.gas,
                "stars": r.stars(),
                "abundances": ab,
                "deaths": r.deaths,
                "cc_per_century": r.cc_rate * 100.0,
                "ia_per_century": r.ia_rate * 100.0,
            })
        })
        .collect();
    // The green band every half Gyr from the disc's first stars to now.
    let mut bands = Vec::new();
    let first = (g.t_start_gyr * 2.0) as i64 + 1;
    let last = (now * 2.0) as i64;
    for k in first..=last {
        let t = k as f64 * 0.5;
        let band = |complex| {
            env.green_band(t, &cp.green, complex, 0.25)
                .map(|(lo, hi)| serde_json::json!([lo, hi]))
        };
        bands.push(
            serde_json::json!({ "t_gyr": t, "habitable": band(false), "complex": band(true) }),
        );
    }
    // Arm crests at `at` (where the envelope's arm term peaks), as (r, phi) polylines.
    // Trailing: the crest falls back against the disc's turning (+phi) as r grows.
    let e = &cp.envelope;
    let rd = g.now.disc_kpc.max(0.1);
    let outer = d.rings.last().map(|r| r.outer_kpc).unwrap_or(rd * 8.0);
    let tan_pitch = m::tan(e.pitch_deg * m::PI / 180.0);
    let omega_p = e.pattern_kms_kpc * 1.0227;
    let arms: Vec<Vec<[f64; 2]>> = (0..e.arms)
        .map(|k| {
            (0..=96)
                .map(|i| {
                    // Log-spaced radii, so the tight inner windings stay smooth.
                    let r = rd * 0.5 * m::pow(outer / (rd * 0.5), f64::from(i) / 96.0);
                    let phi = omega_p * at - m::ln(r / rd) / tan_pitch
                        + e.arm_phase0
                        + 2.0 * m::PI * f64::from(k) / f64::from(e.arms.max(1));
                    [r, phi]
                })
                .collect()
        })
        .collect();
    let place = a.place_kpc.map(|r| {
        let l = env.local(
            Place {
                r_kpc: r,
                phi: a.place_phi.unwrap_or(0.0),
                z_pc: 0.0,
                t_gyr: at,
            },
            &cp.green,
        );
        let gr = env.green(&l, &cp.green);
        serde_json::json!({ "local": l, "green": gr, "ring": d.ring_at(r).radius_kpc })
    });
    let rotation = rotation_curve(c, &cp.galaxy.structure, &h, &g.now, sun_r);
    Ok(serde_json::json!({
        "halo": h.address,
        "key": h.address.rsplit_once("/halo:").map_or(h.address.as_str(), |x| x.1),
        "halo_mass_msun": h.mass,
        "morphology": g.morphology,
        "barred": g.barred,
        "t_start_gyr": g.t_start_gyr,
        "now_gyr": now,
        "at_gyr": at,
        "now": g.now,
        "history": g.history,
        "active_nuclei": g.active_nuclei,
        "sun_radius_kpc": sun_r,
        "disc_kpc": rd,
        "arms": arms,
        "arm_contrast": e.arm_contrast,
        "rings": ring_rows,
        "core_collapse_per_century": d.core_collapse_rate() * 100.0,
        "ia_per_century": d.ia_rate() * 100.0,
        "green_bands": bands,
        "epochs": sse_cosmos::epochs::epochs(c, &cp.epochs),
        "place": place,
        "mergers": mergers,
        "events": events,
        "edits": a.edits,
        "compare": compare,
        "rotation_curve": rotation,
        "assembly": g.assembled,
        "model": [
            "galaxy: a halo's median growth, gas and star formation by published relations (G3), not a simulation",
            "rings: chemical evolution ring by ring with delayed type Ia and neutron-star mergers",
            "green band: the G4 criteria (metals, supernovae, passages, bursts, a quiet nucleus, time)",
            "inked history (G6): each edit acts from its own time on; everything before the earliest edit is unchanged",
            "rotation curve (DM4): dark matter on a profile set by the halo's own growth and reshaped by its galaxy, plus discs, bulge, hot gas and black hole; the galaxy now, at the halo's mass now",
            "assembly (GH3): every progenitor above a thousandth of the halo brings its own galaxy on its own orbit; stars torn off join the stellar halo, gas pushed out joins the hot halo, the rest reaches the centre; each merger and close pass answers by its ratios and leaves a dated mark",
        ],
    }))
}

/// DM4's rotation curve card: the halo's structure now and the circular
/// speed by part on log-spaced radii out to r200 (at most 100 kpc).
fn rotation_curve(
    c: &Cosmology,
    sp: &sse_cosmos::structure::StructureParams,
    h: &sse_cosmos::haloes::Halo,
    now: &sse_cosmos::galaxy::GalaxyState,
    sun_kpc: f64,
) -> serde_json::Value {
    let a = c.scale_of_time(now.t_gyr).min(1.0);
    let s = sse_cosmos::structure::Structure::new(c, sp, &|z| h.mass_at(z), now, a);
    let outer = s.r200_kpc.min(100.0);
    let points: Vec<_> = (0..=48)
        .map(|k| s.speeds(0.1 * m::pow(outer / 0.1, f64::from(k) / 48.0)))
        .collect();
    let (v_max, r_max) = s.v_max();
    serde_json::json!({
        "structure": s,
        "points": points,
        "at_sun": s.speeds(sun_kpc),
        "dark_density_at_sun_msun_pc3": s.dark_density(sun_kpc) * 1.0e-9,
        "v_max_kms": v_max,
        "r_v_max_kpc": r_max,
        "units": "radii kpc, speeds km/s, masses solar masses",
    })
}

pub(crate) struct SliceAsk<'a> {
    pub seed: &'a Seed,
    pub root: &'a Address,
    pub z: Option<f64>,
    pub t_gyr: Option<f64>,
    pub centre: [f64; 3],
    pub extent_mpc: f64,
    pub n: u32,
    pub cube: bool,
    /// With a cube: its galaxies instead (V9w light from haloes).
    pub haloes: bool,
    /// The smallest halo drawn, if above the pack's floor.
    pub galaxy_floor_msun: Option<f64>,
}

/// A square foam slice of the cosmic web (G1) at one epoch, for V9's maps.
pub(crate) fn web_slice(a: SliceAsk) -> Result<serde_json::Value, Response> {
    if a.n < 4 || a.n > 256 {
        return Err(err("bad_request", "n must be 4 to 256 samples a side"));
    }
    if a.cube && a.n > 48 {
        return Err(err("bad_request", "a cube is 4 to 48 parcels a side"));
    }
    if !(a.extent_mpc > 0.0 && a.extent_mpc <= 2000.0) {
        return Err(err(
            "bad_request",
            "extent_mpc must be above 0 and at most 2000",
        ));
    }
    let cp = settings()?;
    let c = cosmology(&cp.params);
    let w = web(&c, &cp, a.seed, a.root)?;
    let scale = match (a.z, a.t_gyr) {
        (Some(z), None) if z > -0.99 => 1.0 / (1.0 + z),
        (None, Some(t)) if t > 0.0 => c.scale_of_time(t).min(1.0),
        (None, None) => 1.0,
        _ => {
            return Err(err(
                "bad_request",
                "give a redshift z or a time t_gyr, not both",
            ))
        }
    };
    let octaves = w.params.octaves;
    if a.haloes && !a.cube {
        return Err(err("bad_request", "haloes come with a cube"));
    }
    if a.cube && a.haloes {
        // The cube's galaxies alone (its parcels are the plain cube ask), so
        // a view can show the parcels first and light them after.
        let hs = Haloes {
            cosmo: &c,
            web: &w,
            params: cp.haloes.clone(),
            seed: *a.seed,
            universe: a.root.clone(),
        };
        if let Some(f) = a.galaxy_floor_msun {
            if !(1.0e8..=1.0e15).contains(&f) {
                return Err(err("bad_request", "galaxy_floor_msun must be 1e8 to 1e15"));
            }
        }
        let lp = &sse_cosmos::web_light::WebLightParams {
            floor_msun: a
                .galaxy_floor_msun
                .map_or(cp.web_light.floor_msun, |f| f.max(cp.web_light.floor_msun)),
            ..cp.web_light.clone()
        };
        let median = web_light_table(&hs, &cp)?;
        let galaxies = sse_cosmos::web_light::cube_galaxies(
            &hs,
            &median,
            lp,
            scale,
            a.centre,
            a.extent_mpc,
            octaves - 1,
        );
        return Ok(serde_json::json!({
            "t_gyr": c.time_of_scale(scale),
            "z": 1.0 / scale - 1.0,
            "now_gyr": c.age_gyr(),
            "centre": a.centre,
            "extent": a.extent_mpc,
            "galaxies": galaxies,
            "galaxy_floor_msun": lp.floor_msun,
            "model": ["galaxies: G2's haloes from the web light floor up whose matter started inside the cube, each moved from the peak it collapsed from by the cube's own truncated Zel'dovich step, at its mass then (its own growth history); stars and star formation from G3 on the median halo of its mass, per half dex, as shares of its halo then"],
        }));
    }
    if a.cube {
        let v = w.volume(&c, scale, a.centre, a.extent_mpc, a.n, octaves - 1);
        return Ok(serde_json::json!({
            "t_gyr": c.time_of_scale(scale),
            "z": 1.0 / scale - 1.0,
            "now_gyr": c.age_gyr(),
            "volume": v,
            "model": ["the cosmic web from seeded Gaussian fields grown linearly; each parcel of matter moved by the truncated Zel'dovich approximation (Coles, Melott and Shandarin 1993) and given the room that step leaves it; voids, sheets, filaments and knots by the T-web, its threshold set so voids fill the pack's share of space (G1); a picture of the large-scale pattern, not an N-body simulation"],
        }));
    }
    let s = w.slice(&c, scale, a.centre, a.extent_mpc, a.n, octaves);
    Ok(serde_json::json!({
        "t_gyr": c.time_of_scale(scale),
        "z": 1.0 / scale - 1.0,
        "now_gyr": c.age_gyr(),
        "slice": s,
        "model": ["the cosmic web from seeded Gaussian fields, grown linearly and moved by the Zel'dovich approximation (G1); a picture of the large-scale pattern, not an N-body simulation"],
    }))
}

thread_local! {
    /// G3 on median haloes for the web's light: about twelve galaxies run
    /// through all of time, the same for every cube of one cosmology, so the
    /// last table is kept (keyed by what it is made from; changes no answer).
    static WEB_LIGHT: std::cell::RefCell<Option<(String, std::rc::Rc<sse_cosmos::web_light::MedianGalaxies>)>> =
        const { std::cell::RefCell::new(None) };
}

fn web_light_table(
    hs: &Haloes,
    cp: &CosmologyPack,
) -> Result<std::rc::Rc<sse_cosmos::web_light::MedianGalaxies>, Response> {
    let key = serde_json::to_string(&(
        format!("{:?}", hs.seed),
        hs.universe.to_string(),
        &cp.params,
        &cp.haloes,
        &cp.galaxy,
        &cp.web_light,
    ))
    .map_err(|e| err("internal", e.to_string()))?;
    if let Some(t) = WEB_LIGHT.with(|w| {
        w.borrow()
            .as_ref()
            .filter(|(k, _)| *k == key)
            .map(|(_, t)| t.clone())
    }) {
        return Ok(t);
    }
    let t = std::rc::Rc::new(sse_cosmos::web_light::MedianGalaxies::new(
        hs,
        &cp.galaxy,
        cp.web_light.floor_msun,
        cp.haloes.max_mass,
    ));
    WEB_LIGHT.with(|w| *w.borrow_mut() = Some((key, t.clone())));
    Ok(t)
}

/// DM3: one cell's matter ledger today.
pub(crate) fn matter_budget(
    seed: &Seed,
    root: &Address,
    cell: [i64; 3],
) -> Result<serde_json::Value, Response> {
    let cp = settings()?;
    let c = cosmology(&cp.params);
    let w = web(&c, &cp, seed, root)?;
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: cp.haloes.clone(),
        seed: *seed,
        universe: root.clone(),
    };
    let lp = &cp.ledger;
    let res = sse_cosmos::ledger::Reservoirs::new(
        &hs,
        &cp.galaxy,
        lp.galaxy_low_msun,
        cp.haloes.max_mass,
    );
    let l = sse_cosmos::ledger::cell(&hs, &res, lp, cell);
    let mut v = serde_json::to_value(&l).map_err(|e| err("internal", e.to_string()))?;
    v["model"] = serde_json::json!([
        "matter: the cell's mass from the web's density; dark and ordinary in the universe's shares",
        "haloes: the catalogued ones (G2) and, below the floor, the same mass function down to floor_low_msun; the rest of the dark matter never collapsed into those",
        "reservoirs: G3 on a median halo once per half dex of mass, read off by each halo's mass; hot halo gas is a halo's share of ordinary matter less what its galaxy holds and threw out; gas between galaxies is the rest of the cell's",
        "environments: the T-web at the truncated Zel'dovich smoothing (voids, sheets, filaments, knots) on a grid of equal-mass parcels, each given the room the Zel'dovich step leaves it today",
    ]);
    Ok(v)
}

pub(crate) struct CellAsk<'a> {
    pub seed: &'a Seed,
    pub root: &'a Address,
    pub cell: [i64; 3],
    pub min_mass: f64,
    pub limit: usize,
}

/// One halo cell (G2) for V9's cluster map: its galaxies' haloes, biggest
/// first, down to `min_mass` and at most `limit` of them, with the mass
/// function they were drawn from. Small on purpose: no merger trees.
pub(crate) fn halo_cell(a: CellAsk) -> Result<serde_json::Value, Response> {
    if !(a.min_mass >= 1.0e8 && a.min_mass < 1.0e16) {
        return Err(err("bad_request", "min_mass_msun must be 1e8 to 1e16"));
    }
    if a.limit == 0 || a.limit > 1000 {
        return Err(err("bad_request", "limit must be 1 to 1000"));
    }
    let cp = settings()?;
    let c = cosmology(&cp.params);
    let w = web(&c, &cp, a.seed, a.root)?;
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: cp.haloes.clone(),
        seed: *a.seed,
        universe: a.root.clone(),
    };
    let cell = hs.cell(a.cell);
    let side = cp.haloes.cell_mpc;
    let total = cell.haloes.len();
    let mut list: Vec<&sse_cosmos::haloes::Halo> = cell
        .haloes
        .iter()
        .filter(|h| h.mass >= a.min_mass)
        .collect();
    // Biggest first; ties by address so every device agrees.
    list.sort_by(|x, y| {
        y.mass
            .total_cmp(&x.mass)
            .then_with(|| x.address.cmp(&y.address))
    });
    let above = list.len();
    list.truncate(a.limit);
    let t_of_z = |z: f64| c.time_of_scale(1.0 / (1.0 + z));
    let haloes: Vec<serde_json::Value> = list
        .iter()
        .map(|h| {
            let major: Vec<&sse_cosmos::haloes::Merger> =
                h.mergers.iter().filter(|m| m.ratio >= cp.haloes.major_ratio).collect();
            serde_json::json!({
                "address": h.address,
                "key": h.address.rsplit_once("/halo:").map_or(h.address.as_str(), |x| x.1).to_string(),
                "mass_msun": h.mass,
                "position_mpc": h.position,
                "half_mass_gyr": t_of_z(h.z_half),
                "spin": h.spin,
                "mergers": h.mergers.len(),
                "major_mergers": major.len(),
                "last_major_gyr": major.iter().map(|m| m.t_gyr).fold(None, |a: Option<f64>, t| Some(a.map_or(t, |x| x.max(t)))),
                "neighbour": h.neighbour,
            })
        })
        .collect();
    let bins: Vec<serde_json::Value> = cell
        .bins
        .iter()
        .map(|(lm, e, n)| serde_json::json!({ "log10_mass": lm, "expected": e, "drawn": n }))
        .collect();
    Ok(serde_json::json!({
        "address": cell.address,
        "index": cell.index,
        "side_mpc": side,
        "origin_mpc": [a.cell[0] as f64 * side, a.cell[1] as f64 * side, a.cell[2] as f64 * side],
        "delta_l": cell.delta_l,
        "mass_msun": cell.mass,
        "halo_mass_msun": cell.halo_mass,
        "total": total,
        "above_min": above,
        "min_mass_msun": a.min_mass,
        "haloes": haloes,
        "bins": bins,
        "now_gyr": c.age_gyr(),
        "model": [
            "haloes: counts from the Sheth and Tormen 1999 mass function, raised or lowered by the cell's overdensity; each placed on the web's highest nearby peak (G2)",
            "growth and mergers: the Correa et al. 2015 median history with scatter, mergers from the Fakhouri, Ma and Boylan-Kolchin 2010 rate",
            "positions are comoving Mpc today; a halo is the galaxy's dark matter, the galaxy itself is grown when charted",
        ],
    }))
}

/// What `set_system_age` pins for a reconstructed origin (17.1).
pub(crate) struct OriginAt {
    pub age_gyr: f64,
    /// Campaign time elapsed since the clock was first set, Gyr.
    pub elapsed_gyr: f64,
    pub r_now_kpc: Option<f64>,
    pub phi_now_deg: Option<f64>,
    pub keep_feh: Option<f64>,
}

/// One plausible past for a star made older or younger, in the campaign's
/// galaxy. Until G7 gives a campaign its own galaxy, that is the median
/// Milky-Way-like galaxy grown from the universe's seed (as the galaxy
/// chart's default), and a star with no galactic place is put at the Sun's
/// radius (fudge register).
pub(crate) fn reconstruct_origin(
    seed: &Seed,
    root: &Address,
    a: OriginAt,
) -> Result<sse_cosmos::origin::Origin, Response> {
    let cp = settings()?;
    let c = cosmology(&cp.params);
    let w = web(&c, &cp, seed, root)?;
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: cp.haloes.clone(),
        seed: *seed,
        universe: root.clone(),
    };
    let h = named_halo(&hs, None, None)?;
    let g = fed_galaxy(&c, &cp, &hs, &h, &[])?;
    let g = &*g;
    let addr: Address = h
        .address
        .parse()
        .map_err(|e| err("bad_address", format!("{e:?}")))?;
    let d = rings::evolve(g, &cp.galaxy, &cp.rings, *seed, &addr);
    let env = GalaxyEnvelope {
        galaxy: g,
        disc: &d,
        params: cp.envelope.clone(),
        seed: *seed,
        address: addr,
        remains: Default::default(),
    };
    let ask = sse_cosmos::origin::OriginAsk {
        now_gyr: g.now.t_gyr + a.elapsed_gyr,
        age_gyr: a.age_gyr,
        r_now_kpc: a.r_now_kpc.unwrap_or_else(|| env.sun_radius_kpc()),
        phi_now_deg: a.phi_now_deg.unwrap_or(0.0),
        keep_feh: a.keep_feh,
    };
    Ok(sse_cosmos::origin::reconstruct(&env, &cp.origin, &ask))
}

/// Event kind and fact of a promoted star (night sky design, Y7a): the
/// whole drawn description is stored, so later fill rules never move it.
pub const STAR_PIN_KIND: &str = "star.pin";
pub const STAR_DESCRIPTION: &str = "star.description";

/// Every promoted star in a chronicle state: the stored descriptions, read
/// back exactly (address order). This is the loader rule for saves: a pin
/// is never redrawn.
pub(crate) fn star_pins(
    state: &sse_chronicle::State,
) -> Result<Vec<sse_cosmos::stars::StarSystem>, Response> {
    let mut out = vec![];
    for f in state.facts.values() {
        if let Some(sse_core::cbor::Value::Text(t)) = f.get(STAR_DESCRIPTION) {
            out.push(
                serde_json::from_str(t)
                    .map_err(|e| err("bad_record", format!("stored star: {e}")))?,
            );
        }
    }
    Ok(out)
}

/// The campaign's galaxy (as `reconstruct_origin`: the median Milky-Way-like
/// galaxy until G7) with G5's stars over it, and the galaxy's now in Myr.
pub(crate) fn with_stars<R>(
    seed: &Seed,
    root: &Address,
    params: Option<sse_cosmos::stars::StarParams>,
    f: impl FnOnce(&sse_cosmos::stars::Stars, f64) -> Result<R, Response>,
) -> Result<R, Response> {
    with_world(seed, root, params, |st, _, now| f(st, now))
}

/// The cosmos around the campaign's galaxy: the cosmology, the web's
/// haloes and the pack's settings (Y4 reads other galaxies through it).
pub(crate) struct World<'a> {
    pub c: &'a Cosmology,
    pub hs: &'a Haloes<'a>,
    pub cp: &'a CosmologyPack,
    /// Our galaxy's own halo.
    pub host: &'a sse_cosmos::haloes::Halo,
}

/// As `with_stars`, with the world around the galaxy too.
pub(crate) fn with_world<R>(
    seed: &Seed,
    root: &Address,
    params: Option<sse_cosmos::stars::StarParams>,
    f: impl FnOnce(&sse_cosmos::stars::Stars, &World, f64) -> Result<R, Response>,
) -> Result<R, Response> {
    let cp = settings()?;
    let c = cosmology(&cp.params);
    let w = web(&c, &cp, seed, root)?;
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: cp.haloes.clone(),
        seed: *seed,
        universe: root.clone(),
    };
    let h = named_halo(&hs, None, None)?;
    let g = fed_galaxy(&c, &cp, &hs, &h, &[])?;
    let g = &*g;
    let addr: Address = h
        .address
        .parse()
        .map_err(|e| err("bad_address", format!("{e:?}")))?;
    let d = rings::evolve(g, &cp.galaxy, &cp.rings, *seed, &addr);
    let env = GalaxyEnvelope {
        galaxy: g,
        disc: &d,
        params: cp.envelope.clone(),
        seed: *seed,
        address: addr.clone(),
        remains: Default::default(),
    };
    let st = sse_cosmos::stars::Stars {
        env: &env,
        green: cp.green.clone(),
        params: params.unwrap_or_else(|| cp.stars.clone()),
        seed: *seed,
        galaxy: addr,
    };
    let world = World {
        c: &c,
        hs: &hs,
        cp: &cp,
        host: &h,
    };
    f(&st, &world, g.now.t_gyr * 1000.0)
}

/// One 10 pc cell's systems at a time: the fill as today's rules draw it,
/// with the campaign's pins claiming their share first. With no cell, the
/// cell at the Sun's radius.
pub(crate) fn star_cell(
    st: &sse_cosmos::stars::Stars,
    cell: Option<[i64; 3]>,
    t_myr: f64,
    pins: &[sse_cosmos::stars::StarSystem],
) -> Result<serde_json::Value, Response> {
    if !t_myr.is_finite() || t_myr < 0.0 {
        return Err(err("bad_request", "t_myr must be 0 or more"));
    }
    let frame = st.frame_of(t_myr);
    let cell = cell.unwrap_or_else(|| {
        st.cell_at(
            st.from_grid([st.env.sun_radius_kpc() * 1000.0, 0.0, 0.0], t_myr),
            t_myr,
        )
    });
    let systems = st
        .cell(cell, frame, pins)
        .map_err(|e| err("bad_request", e))?;
    let rows: Vec<serde_json::Value> = systems
        .iter()
        .map(|s| {
            let at = st.at(s, t_myr);
            serde_json::json!({
                "address": s.address.to_string(),
                "pinned": s.pinned,
                "drawn_in": {"cell": s.cell, "frame": s.frame, "slot": s.slot},
                "mass_msun": s.mass_msun,
                "companions": s.companions,
                "birth_gyr": s.birth_gyr,
                "feh": s.feh,
                "xyz_pc": at.xyz_pc,
                "state": at.state,
            })
        })
        .collect();
    Ok(serde_json::json!({
        "galaxy": st.galaxy.to_string(),
        "cell": cell,
        "frame": frame,
        "frame_start_myr": frame as f64 * st.params.frame_myr,
        "t_myr": t_myr,
        "cell_pc": st.params.cell_pc,
        "expected": st.expected(cell, frame),
        "pins": systems.iter().filter(|s| s.pinned).count(),
        "systems": rows,
        "model": [
            "fill: a Poisson count from the G4 envelope at the cell's centre, then per system its birth, Kroupa IMF mass, companions, abundances and an epicyclic orbit, keyed by cell and frame (G5)",
            "pins: promoted systems are stored whole in the record and read back as stored; they claim their share of the cell's count first, so nothing is counted twice",
        ],
    }))
}

#[cfg(test)]
mod tests {
    use super::{settings, star_cell, star_pins, with_stars};
    use crate::Kernel;

    fn open() -> Kernel {
        let mut k = Kernel::new();
        let r = k.handle_json(r#"{"op":"open_universe","id":"g3","seed":"g3"}"#);
        assert!(r.contains("universe_opened"), "{r}");
        k
    }

    fn ask(k: &mut Kernel, req: &str) -> serde_json::Value {
        serde_json::from_str(&k.handle_json(req)).unwrap()
    }

    #[test]
    fn galaxy_chart_matches_the_g3_golden_and_is_repeatable() {
        let mut k = open();
        let r = ask(
            &mut k,
            r#"{"op":"galaxy_chart","halo":"milky-way-like","place_kpc":8.0}"#,
        );
        assert_eq!(r["type"], "galaxy_charted", "{r}");
        let c = &r["chart"];
        // The same galaxy as G3's golden (seed g3, universe g3).
        assert_eq!(c["morphology"], "barred");
        assert_eq!(c["halo"], "u:g3/halo:milky-way-like");
        let golden: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/golden/galaxy.json"),
            )
            .unwrap(),
        )
        .unwrap();
        // G3's golden is the galaxy unfed; the chart's is fed by its
        // tree (GH3), with about the same stars, since what arrives was
        // taken back from its smooth inflow.
        let g = &golden["galaxies"][1];
        let r_stars = c["now"]["stars"].as_f64().unwrap() / g["stars"].as_f64().unwrap();
        assert!((0.7..1.3).contains(&r_stars), "{r_stars}");
        let a = &c["assembly"];
        assert!(a["members"].as_u64().unwrap() > 20, "{a}");
        assert!(!a["marks"].as_array().unwrap().is_empty());
        assert!(a["exsitu_msun"].as_f64().unwrap() > 0.0);
        assert_eq!(c["rings"].as_array().unwrap().len(), 24);
        assert!(c["history"].as_array().unwrap().len() > 10);
        assert!(c["green_bands"]
            .as_array()
            .unwrap()
            .iter()
            .any(|b| !b["habitable"].is_null()));
        assert!(c["place"]["local"]["feh"].is_number());
        assert!(c["epochs"].as_array().unwrap().len() > 3);
        let again = ask(
            &mut k,
            r#"{"op":"galaxy_chart","halo":"milky-way-like","place_kpc":8.0}"#,
        );
        assert_eq!(r, again);
        let bad = ask(&mut k, r#"{"op":"galaxy_chart","halo":"andromeda"}"#);
        assert_eq!(bad["code"], "unknown_halo");
    }

    #[test]
    fn inked_history_changes_only_what_comes_after() {
        let mut k = open();
        let base = ask(&mut k, r#"{"op":"galaxy_chart","halo":"milky-way-like"}"#);
        let c0 = &base["chart"];
        assert!(c0["compare"].is_null());
        let mergers = c0["mergers"].as_array().unwrap();
        let events = c0["events"].as_array().unwrap();
        let own = events.iter().filter(|e| e["kind"] == "merger").count();
        assert_eq!(own, mergers.len(), "every halo merger is an event");
        // Ink a major merger at 8 Gyr: the galaxy changes from 8 Gyr on, not before.
        let r = ask(
            &mut k,
            r#"{"op":"galaxy_chart","halo":"milky-way-like","edits":[{"kind":"merger","t_gyr":8.0,"ratio":0.5}]}"#,
        );
        let c = &r["chart"];
        assert_eq!(r["type"], "galaxy_charted", "{r}");
        let first = c["compare"]["first_change_gyr"].as_f64().unwrap();
        assert!((7.9..8.2).contains(&first), "{first}");
        assert!(c["events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["id"] == "edit/0"));
        assert_ne!(c["now"]["bulge"], c0["now"]["bulge"]);
        for (a, b) in c["history"]
            .as_array()
            .unwrap()
            .iter()
            .zip(c0["history"].as_array().unwrap())
        {
            if a["t_gyr"].as_f64().unwrap() < 7.9 {
                assert_eq!(a, b, "history before the edit is untouched");
            }
        }
        // A bad edit is refused.
        let bad = ask(
            &mut k,
            r#"{"op":"galaxy_chart","halo":"dwarf","edits":[{"kind":"remove_merger","index":999}]}"#,
        );
        assert_eq!(bad["code"], "bad_edit", "{bad}");
    }

    /// DM3 through the kernel: the ledger closes and is repeatable.
    #[test]
    fn matter_budget_closes_and_is_repeatable() {
        let mut k = open();
        let r = ask(&mut k, r#"{"op":"matter_budget","cell":[0,0,0]}"#);
        assert_eq!(r["type"], "matter_budgeted", "{r}");
        let b = &r["budget"];
        let f = |p: &str, q: &str| b[p][q].as_f64().unwrap();
        let dark = f("dark", "catalogued_haloes")
            + f("dark", "haloes_below_floor")
            + f("dark", "never_collapsed");
        let ord: f64 = [
            "stars",
            "cold_gas",
            "hot_gas",
            "thrown_out",
            "black_holes",
            "between_galaxies",
        ]
        .iter()
        .map(|q| f("ordinary", q))
        .sum();
        let m = b["matter_msun"].as_f64().unwrap();
        assert!(((dark + ord) / m - 1.0).abs() < 1e-9, "{b}");
        assert_eq!(b["environments"].as_array().unwrap().len(), 4);
        let again = ask(&mut k, r#"{"op":"matter_budget","cell":[0,0,0]}"#);
        assert_eq!(r, again);
    }

    #[test]
    fn web_slice_grows_contrast_with_time() {
        let mut k = open();
        let early = ask(
            &mut k,
            r#"{"op":"web_slice","z":3.0,"extent_mpc":100,"n":16}"#,
        );
        let now = ask(&mut k, r#"{"op":"web_slice","extent_mpc":100,"n":16}"#);
        assert_eq!(now["type"], "web_sliced", "{now}");
        let spread = |v: &serde_json::Value| {
            let d: Vec<f64> = v["web"]["slice"]["density"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_f64().unwrap())
                .collect();
            assert_eq!(d.len(), 256);
            d.iter().fold(0.0_f64, |a, x| a.max(*x)) - d.iter().fold(f64::MAX, |a, x| a.min(*x))
        };
        assert!(spread(&now) > spread(&early), "the foam grows its contrast");
        assert!(now["web"]["t_gyr"].as_f64().unwrap() > early["web"]["t_gyr"].as_f64().unwrap());
        let bad = ask(&mut k, r#"{"op":"web_slice","extent_mpc":100,"n":1000}"#);
        assert_eq!(bad["code"], "bad_request");
        let mut fresh = Kernel::new();
        let none = ask(&mut fresh, r#"{"op":"web_slice","extent_mpc":100,"n":16}"#);
        assert_eq!(none["code"], "no_universe");
    }
    #[test]
    fn a_web_cube_gives_parcels_for_the_3d_view() {
        let mut k = open();
        let r = ask(
            &mut k,
            r#"{"op":"web_slice","cube":true,"extent_mpc":128,"n":12}"#,
        );
        assert_eq!(r["type"], "web_sliced", "{r}");
        let v = &r["web"]["volume"];
        assert_eq!(v["position"].as_array().unwrap().len(), 12 * 12 * 12 * 3);
        let env = v["environment"].as_array().unwrap();
        let room: Vec<f64> = v["room"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        let total: f64 = room.iter().sum();
        let void_room: f64 = env
            .iter()
            .zip(&room)
            .filter(|(e, _)| *e == "void")
            .map(|(_, r)| r)
            .sum();
        let void_mass = env.iter().filter(|e| *e == "void").count() as f64 / env.len() as f64;
        assert!(
            void_mass < void_room / total,
            "voids hold less matter than space"
        );
        let big = ask(
            &mut k,
            r#"{"op":"web_slice","cube":true,"extent_mpc":128,"n":64}"#,
        );
        assert_eq!(big["code"], "bad_request");
    }
    #[test]
    fn a_web_cube_with_haloes_gives_its_galaxies() {
        let mut k = open();
        let r = ask(
            &mut k,
            r#"{"op":"web_slice","cube":true,"haloes":true,"extent_mpc":40,"n":8}"#,
        );
        assert_eq!(r["type"], "web_sliced", "{r}");
        let w = &r["web"];
        let floor = w["galaxy_floor_msun"].as_f64().unwrap();
        let gals = w["galaxies"].as_array().unwrap();
        assert!(!gals.is_empty(), "{w}");
        for g in gals {
            assert!(g["halo_msun"].as_f64().unwrap() >= floor);
            assert!(g["stars_msun"].as_f64().unwrap() > 0.0);
        }
        // The galaxies come alone; the parcels are the plain cube.
        assert!(w.get("volume").is_none());
        let plain = ask(
            &mut k,
            r#"{"op":"web_slice","cube":true,"extent_mpc":40,"n":8}"#,
        );
        assert!(plain["web"].get("galaxies").is_none());
        let flat = ask(
            &mut k,
            r#"{"op":"web_slice","haloes":true,"extent_mpc":40,"n":8}"#,
        );
        assert_eq!(flat["code"], "bad_request");
        // A higher floor keeps to the bigger haloes.
        let few = ask(
            &mut k,
            r#"{"op":"web_slice","cube":true,"haloes":true,"galaxy_floor_msun":1e12,"extent_mpc":40,"n":8}"#,
        );
        assert_eq!(few["web"]["galaxy_floor_msun"], 1.0e12);
        assert!(few["web"]["galaxies"].as_array().unwrap().len() < gals.len());
    }
    #[test]
    fn a_cell_lists_its_haloes_and_one_can_be_charted() {
        let mut k = open();
        let c = ask(&mut k, r#"{"op":"halo_cell","cell":[0,0,0]}"#);
        let c = &c["cell"];
        let hs = c["haloes"].as_array().unwrap();
        assert!(!hs.is_empty(), "{c}");
        let m: Vec<f64> = hs
            .iter()
            .map(|h| h["mass_msun"].as_f64().unwrap())
            .collect();
        assert!(m.windows(2).all(|w| w[0] >= w[1]), "biggest first");
        assert!(c["halo_mass_msun"].as_f64().unwrap() <= c["mass_msun"].as_f64().unwrap());
        let again = ask(&mut k, r#"{"op":"halo_cell","cell":[0,0,0]}"#);
        assert_eq!(c, &again["cell"]);
        let key = hs[0]["key"].as_str().unwrap();
        let g = ask(
            &mut k,
            &format!(r#"{{"op":"galaxy_chart","cell":[0,0,0],"halo":"halo:{key}"}}"#),
        );
        assert_eq!(g["chart"]["halo"], hs[0]["address"]);
        assert!((g["chart"]["halo_mass_msun"].as_f64().unwrap() - m[0]).abs() < 1.0);
        let bad = k.handle_json(r#"{"op":"galaxy_chart","cell":[0,0,0],"halo":"halo:99999"}"#);
        assert!(bad.contains("unknown_halo"), "{bad}");
    }

    /// Stars brighter than V = 3, 4 and 5 over the whole sky: about 171, 513
    /// and 1,602 (the standard cumulative counts, as tabulated at
    /// https://en.wikipedia.org/wiki/Apparent_magnitude and
    /// https://oeis.org/A053406).
    const REAL_SKY: [(f64, f64); 3] = [(3.0, 171.0), (4.0, 513.0), (5.0, 1602.0)];

    fn n_brighter(sky: &serde_json::Value, mag: f64) -> f64 {
        sky["counts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["mag"].as_f64() == Some(mag))
            .map_or(0.0, |c| c["n"].as_f64().unwrap())
    }

    #[test]
    fn the_sky_from_the_suns_place_counts_like_the_real_one() {
        let mut k = open();
        let r = ask(
            &mut k,
            r#"{"op":"sky_from_here","limit_mag":5,"bands":["B","V"],"galaxies":false}"#,
        );
        assert_eq!(r["type"], "sky", "{r}");
        let sky = &r["sky"];
        // A fill sky is statistical, and this galaxy is only like the Milky
        // Way: within a factor of 1.5 either way of the real counts.
        for (mag, real) in REAL_SKY {
            let n = n_brighter(sky, mag);
            assert!(
                n > real / 1.5 && n < real * 1.5,
                "V < {mag}: {n} against {real}"
            );
        }
        let stars = sky["stars"].as_array().unwrap();
        assert!(!stars.is_empty());
        // Brightest first, each at or above the limit.
        for w in stars.windows(2) {
            assert!(w[0]["mag"].as_f64() <= w[1]["mag"].as_f64());
        }
        assert!(stars.iter().all(|s| s["mag"].as_f64().unwrap() <= 5.0));
        // Dust reddens: B-V excess is positive wherever A_V is.
        let s = stars
            .iter()
            .find(|s| s["a_v"].as_f64().unwrap() > 0.05)
            .unwrap();
        assert!(s["colours"][0]["excess"].as_f64().unwrap() > 0.0, "{s}");

        // A point's magnitude is its system's: a single star's photosphere
        // read through the same sight line gives the same V.
        let one = stars
            .iter()
            .find(|s| s["double"] == false && s["mag"].as_f64().unwrap() > 3.0)
            .unwrap();
        let c = &one["components"][0];
        let pc = sse_core::Units::standard().get("pc").unwrap().scale;
        let p = ask(
            &mut k,
            &serde_json::json!({
                "op": "read_photometry",
                "bands": ["V"],
                "stars": [{
                    "temperature_k": c["teff_k"],
                    "radius_m": c["radius_m"],
                    "distance_m": one["distance_pc"].as_f64().unwrap() * pc,
                    "a_v": one["a_v"],
                }],
            })
            .to_string(),
        );
        let v = p["stars"][0]["magnitudes"][0]["m_vega"].as_f64().unwrap();
        assert!((v - one["mag"].as_f64().unwrap()).abs() < 1e-9, "{v} {one}");

        // The star the sky finds is the one its cell draws.
        let addr = one["address"].as_str().unwrap();
        let cell: Vec<i64> = addr
            .split("cell:")
            .nth(1)
            .unwrap()
            .split('/')
            .next()
            .unwrap()
            .split('_')
            .map(|x| x.parse().unwrap())
            .collect();
        let drawn = ask(
            &mut k,
            &format!(
                r#"{{"op":"star_cell","cell":[{},{},{}],"t_myr":{}}}"#,
                cell[0], cell[1], cell[2], sky["t_myr"]
            ),
        );
        let row = drawn["cell"]["systems"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["address"] == addr)
            .unwrap_or_else(|| panic!("{addr} not drawn in its cell"));
        assert_eq!(row["companions"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn the_sky_is_the_same_twice_and_a_promoted_star_appears_once() {
        use sse_core::math as m;
        let mut k = open();
        let q = r#"{"op":"sky_from_here","limit_mag":3,"galaxies":false,"cone":{"l_deg":90,"b_deg":0,"half_angle_deg":60}}"#;
        let a = ask(&mut k, q);
        assert_eq!(a["type"], "sky", "{a}");
        let b = ask(&mut k, q);
        assert_eq!(a, b);
        // Every star is inside the cone.
        let stars = a["sky"]["stars"].as_array().unwrap();
        assert!(stars.len() > 5, "{}", a["sky"]["counts"]);
        for s in stars {
            let (l, bb) = (s["l_deg"].as_f64().unwrap(), s["b_deg"].as_f64().unwrap());
            let (l, bb) = (l.to_radians(), bb.to_radians());
            let cos = m::cos(bb) * m::sin(l); // cos of the angle to (l 90, b 0)
            assert!(cos >= m::cos(60f64.to_radians()) - 1e-9, "{s}");
        }
        // Promote one: it is still there once, now pinned, as bright.
        let pick = &stars[stars.len() / 2];
        let addr = pick["address"].as_str().unwrap();
        let p = ask(
            &mut k,
            &format!(r#"{{"op":"promote_star","address":"{addr}"}}"#),
        );
        assert_eq!(p["type"], "star_promoted", "{p}");
        let c = ask(&mut k, q);
        let after = c["sky"]["stars"].as_array().unwrap();
        let hits: Vec<_> = after.iter().filter(|s| s["address"] == addr).collect();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0]["pinned"], true);
        assert_eq!(hits[0]["mag"], pick["mag"]);
        assert_eq!(after.len(), stars.len());
    }

    #[test]
    fn other_galaxies_shine_as_their_mass_and_distance_say() {
        let mut k = open();
        let q = r#"{"op":"sky_from_here","limit_mag":9,"bands":["B","V"],"stars":false}"#;
        let r = ask(&mut k, q);
        assert_eq!(r["type"], "sky", "{r}");
        let sky = &r["sky"];
        assert!(sky["stars"].as_array().unwrap().is_empty());
        let gs = sky["galaxies"].as_array().unwrap();
        assert!(gs.len() >= 3, "{}", sky["galaxy_walk"]);
        for w in gs.windows(2) {
            assert!(w[0]["mag"].as_f64() <= w[1]["mag"].as_f64());
        }
        // The Sun's absolute V, from its own photosphere at 10 pc.
        let pc = sse_core::Units::standard().get("pc").unwrap().scale;
        let sun = ask(
            &mut k,
            &serde_json::json!({"op": "read_photometry", "bands": ["V"], "stars": [{
                "temperature_k": 5772.0, "luminosity_w": 3.828e26, "distance_m": 10.0 * pc}]})
            .to_string(),
        );
        let sun_v = sun["stars"][0]["magnitudes"][0]["m_vega"].as_f64().unwrap();
        // The brightest galaxy beyond our own halo (satellites are tested
        // apart).
        let g = gs.iter().find(|g| g["satellite"] != true).unwrap();
        let d = g["distance_mpc"].as_f64().unwrap();
        let z = g["redshift"].as_f64().unwrap();
        let v = g["magnitudes"][1]["m_vega"].as_f64().unwrap();
        let abs_v = g["absolute"][1]["m_vega"].as_f64().unwrap();
        let ext = g["magnitudes"][1]["extinction"].as_f64().unwrap();
        // Distance modulus with the luminosity distance; the K-correction
        // is tiny this close.
        let dm = 5.0 * sse_core::math::log10((1.0 + z) * d * 1.0e6 / 10.0);
        assert!((v - ext - abs_v - dm).abs() < 0.05, "{g}");
        // Mass-to-light in V of a galaxy with stars of every age: 0.3 to 5.
        let l_v = sse_core::math::pow(10.0, (sun_v - abs_v) / 2.5);
        let ml = g["stellar_mass_msun"].as_f64().unwrap() / l_v;
        assert!(ml > 0.3 && ml < 5.0, "M/L_V {ml}: {g}");
        // Seen as it was: the look-back is the light time, 3.26 Myr per Mpc
        // this close.
        let lb = g["lookback_myr"].as_f64().unwrap();
        assert!((lb / (d * 3.2616) - 1.0).abs() < 0.03, "{lb} for {d} Mpc");
        assert!(z > 0.0 && z < 0.01);

        // The same halo through the galaxy chart has the same stellar mass.
        let addr = g["address"].as_str().unwrap();
        let cell: Vec<i64> = addr
            .split("cell:")
            .nth(1)
            .unwrap()
            .split('/')
            .next()
            .unwrap()
            .split('.')
            .map(|x| x.parse().unwrap())
            .collect();
        let halo = addr.rsplit('/').next().unwrap();
        let chart = ask(
            &mut k,
            &format!(
                r#"{{"op":"galaxy_chart","cell":[{},{},{}],"halo":"{halo}"}}"#,
                cell[0], cell[1], cell[2]
            ),
        );
        assert_eq!(chart["chart"]["halo"], g["address"], "{chart}");
        // The sky shows its unfed envelope ("about"); the chart follows its
        // whole tree (GH3), so the two agree to a few tenths.
        assert_eq!(g["about"], true);
        let r = chart["chart"]["now"]["stars"].as_f64().unwrap()
            / g["stellar_mass_now_msun"].as_f64().unwrap();
        assert!((0.7..1.3).contains(&r), "{r}");

        // Order independence: a narrow field around it finds it unchanged.
        let (l, b) = (g["l_deg"].as_f64().unwrap(), g["b_deg"].as_f64().unwrap());
        let narrow = ask(
            &mut k,
            &format!(
                r#"{{"op":"sky_from_here","limit_mag":9,"bands":["B","V"],"stars":false,"cone":{{"l_deg":{l},"b_deg":{b},"half_angle_deg":5}}}}"#
            ),
        );
        let found: Vec<_> = narrow["sky"]["galaxies"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x["address"] == g["address"])
            .collect();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0], g);
    }

    #[test]
    fn a_sky_ask_out_of_range_is_refused() {
        let mut k = open();
        for q in [
            r#"{"op":"sky_from_here","t_myr":-1}"#,
            r#"{"op":"sky_from_here","limit_mag":31}"#,
            r#"{"op":"sky_from_here","band":"Q"}"#,
            r#"{"op":"sky_from_here","observer":"u:g3/halo:nowhere"}"#,
        ] {
            let r = ask(&mut k, q);
            assert_eq!(r["type"], "error", "{q}: {r}");
        }
    }

    #[test]
    fn a_promoted_star_is_stored_whole_and_survives_new_fill_rules() {
        let mut k = open();
        let c = ask(&mut k, r#"{"op":"star_cell"}"#);
        assert_eq!(c["type"], "star_celled", "{c}");
        let cell = &c["cell"];
        let systems = cell["systems"].as_array().unwrap();
        assert!(systems.len() > 3, "{cell}");
        assert_eq!(cell["pins"], 0);
        let pick = systems[systems.len() / 2]["address"]
            .as_str()
            .unwrap()
            .to_string();
        let p = ask(
            &mut k,
            &format!(r#"{{"op":"promote_star","address":"{pick}"}}"#),
        );
        assert_eq!(p["type"], "star_promoted", "{p}");
        assert_eq!(p["already"], false);
        assert!(p["event_id"].is_string());
        // Read again: the same systems, one now pinned, the same numbers.
        let again = ask(&mut k, r#"{"op":"star_cell"}"#);
        let rows = again["cell"]["systems"].as_array().unwrap();
        assert_eq!(rows.len(), systems.len());
        for (a, b) in rows.iter().zip(systems) {
            assert_eq!(a["address"], b["address"]);
            assert_eq!(a["mass_msun"], b["mass_msun"]);
            assert_eq!(a["xyz_pc"], b["xyz_pc"]);
            assert_eq!(a["pinned"], a["address"] == pick.as_str());
        }
        // Promoting it again writes nothing.
        let twice = ask(
            &mut k,
            &format!(r#"{{"op":"promote_star","address":"{pick}"}}"#),
        );
        assert_eq!(twice["already"], true);
        assert_eq!(twice["state_hash"], p["state_hash"]);
        assert_eq!(twice["description"], p["description"]);
        // A save with the pin loads it unchanged.
        let ex = ask(&mut k, r#"{"op":"export_chronicle"}"#);
        let mut k2 = Kernel::new();
        let r = k2.handle_json(
            &serde_json::json!({"op": "import_chronicle", "export": ex["export"]}).to_string(),
        );
        assert!(!r.contains("\"error\""), "{r}");
        let loaded = ask(&mut k2, r#"{"op":"star_cell"}"#);
        assert_eq!(loaded["cell"], again["cell"]);
        // New fill rules (new draw keys) redraw the fill, never the pin.
        let u = k.universe().unwrap();
        let pins = star_pins(
            &u.chronicle
                .state_at(sse_chronicle::UniverseTime::seconds(i64::MAX)),
        )
        .unwrap();
        assert_eq!(pins.len(), 1);
        let stored: sse_cosmos::stars::StarSystem =
            serde_json::from_value(p["description"].clone()).unwrap();
        assert_eq!(pins[0], stored);
        let t = again["cell"]["t_myr"].as_f64().unwrap();
        let cell_ix: [i64; 3] = serde_json::from_value(again["cell"]["cell"].clone()).unwrap();
        let base = settings().unwrap().stars;
        let v2 = sse_cosmos::stars::StarParams {
            key_version: base.key_version + 1,
            ..base
        };
        let redrawn = with_stars(&u.seed, &u.root, Some(v2), |st, _| {
            star_cell(st, Some(cell_ix), t, &pins)
        })
        .unwrap();
        let r2 = redrawn["systems"].as_array().unwrap();
        let mine: Vec<&serde_json::Value> = r2
            .iter()
            .filter(|s| s["address"] == pick.as_str())
            .collect();
        assert_eq!(mine.len(), 1);
        let before = rows.iter().find(|s| s["address"] == pick.as_str()).unwrap();
        assert_eq!(mine[0], before, "the pin reads bit-identical");
        let others = |v: &[serde_json::Value]| -> Vec<serde_json::Value> {
            v.iter()
                .filter(|s| s["address"] != pick.as_str())
                .map(|s| s["mass_msun"].clone())
                .collect()
        };
        assert_ne!(others(r2), others(rows), "the fill was redrawn");
        // Bad addresses are refused.
        let bad = ask(&mut k, r#"{"op":"promote_star","address":"u:g3/sys:1"}"#);
        assert_eq!(bad["type"], "error", "{bad}");
    }

    #[test]
    fn satellite_paths_are_asked_for_and_end_where_the_sky_puts_each_survivor() {
        let mut k = open();
        let plain = ask(
            &mut k,
            r#"{"op":"sky_from_here","limit_mag":30,"bands":["V"],"stars":false}"#,
        );
        // Off by default: the sky's answer does not change.
        assert!(plain["sky"]["galaxy_walk"]["satellites"]
            .get("paths")
            .is_none());
        let r = ask(
            &mut k,
            r#"{"op":"sky_from_here","limit_mag":30,"bands":["V"],"stars":false,"satellite_paths":true}"#,
        );
        assert_eq!(r["sky"]["galaxies"], plain["sky"]["galaxies"]);
        let walk = &r["sky"]["galaxy_walk"]["satellites"];
        let paths = walk["paths"].as_array().unwrap();
        // One path for every member followed, whatever became of it.
        assert_eq!(paths.len() as f64, walk["subhaloes"].as_f64().unwrap());
        let merged = paths.iter().filter(|p| p["fate"] == "merged").count();
        assert_eq!(merged as f64, walk["merged"].as_f64().unwrap());
        assert!(paths.iter().any(|p| p["fate"] == "torn apart"), "{walk}");
        let now = r["sky"]["t_myr"].as_f64().unwrap() / 1000.0;
        let r200 = walk["r200_kpc"].as_f64().unwrap();
        let c = walk["centre_kpc"].as_array().unwrap();
        let c: Vec<f64> = c.iter().map(|v| v.as_f64().unwrap()).collect();
        for p in paths {
            let pts = p["path"].as_array().unwrap();
            let t0 = pts[0][0].as_f64().unwrap();
            assert!(
                (t0 - p["infall_gyr"].as_f64().unwrap()).abs() < 1.0e-9,
                "{p}"
            );
            // Time runs forward along it, and it starts at the host's edge
            // then (smaller than now for an early infall), never past
            // today's.
            let ts: Vec<f64> = pts.iter().map(|x| x[0].as_f64().unwrap()).collect();
            assert!(ts.windows(2).all(|w| w[1] > w[0]), "{p}");
            let r0 = sse_core::math::sqrt(
                (0..3)
                    .map(|i| {
                        let d = pts[0][i + 1].as_f64().unwrap() - c[i];
                        d * d
                    })
                    .sum::<f64>(),
            );
            assert!(r0 > 1.0 && r0 < 1.5 * r200, "{r0} {p}");
            if p["fate"] == "merged" {
                assert!(ts.last().unwrap() <= &(now + 1.0e-9));
            } else {
                assert!((ts.last().unwrap() - now).abs() < 1.0e-6, "{p}");
            }
        }
        // A survivor's path ends where the sky puts it: its direction times
        // its distance from the observer.
        let mut checked = 0;
        for g in r["sky"]["galaxies"].as_array().unwrap() {
            if g["satellite"] != true {
                continue;
            }
            let p = paths.iter().find(|p| p["address"] == g["address"]).unwrap();
            let last = p["path"].as_array().unwrap().last().unwrap();
            let d = g["distance_kpc"].as_f64().unwrap();
            for i in 0..3 {
                let want = g["dir"][i].as_f64().unwrap() * d;
                let got = last[i + 1].as_f64().unwrap();
                assert!((want - got).abs() < 1.0e-6 * d.max(1.0), "{want} {got}");
            }
            checked += 1;
        }
        assert!(checked > 0);
    }

    #[test]
    fn the_group_s_edge_is_the_zero_velocity_sphere_of_the_mass_inside_it() {
        // The edge reads the Sun's mass from the sheet in force: a stack.
        let mut k = Kernel::new();
        let pack = |p: &str| -> serde_json::Value {
            serde_json::from_str(
                &std::fs::read_to_string(format!("{}/../../{p}", env!("CARGO_MANIFEST_DIR")))
                    .unwrap(),
            )
            .unwrap()
        };
        let r = k.handle_json(&serde_json::json!({"op": "load_stack", "packs": [pack("packs/base-reference/pack.json"), pack("packs/base-physics/pack.json")]}).to_string());
        assert!(r.contains("stack_loaded"), "{r}");
        let r = k.handle_json(r#"{"op":"open_universe","id":"g3","seed":"g3"}"#);
        assert!(r.contains("universe_opened"), "{r}");
        let plain = ask(
            &mut k,
            r#"{"op":"sky_from_here","limit_mag":30,"bands":["V"],"stars":false}"#,
        );
        assert!(plain["sky"]["galaxy_walk"].get("group_edge").is_none());
        let r = ask(
            &mut k,
            r#"{"op":"sky_from_here","limit_mag":30,"bands":["V"],"stars":false,"satellite_paths":true}"#,
        );
        let walk = &r["sky"]["galaxy_walk"];
        let e = &walk["group_edge"];
        assert_eq!(e["block"], "sse.gravity.zero_velocity_radius", "{r}");
        let radius = e["radius_mpc"].as_f64().unwrap();
        let host = walk["satellites"]["host_mass_msun"].as_f64().unwrap();
        let mass = e["mass_msun"].as_f64().unwrap();
        // Sandage 1986: 3e12 suns at 13.8 Gyr reach about 1.28 Mpc; scaled
        // as the cube root of mass times age squared.
        let t = e["t_gyr"].as_f64().unwrap();
        let want = 1.277_16 * sse_core::math::cbrt(mass / 3.0e12 * (t / 13.8) * (t / 13.8));
        assert!((radius - want).abs() < 2.0e-3 * want, "{radius} {want}");
        assert!(mass >= host && radius > 0.5 && radius < 3.0, "{e}");
        // Settled: exactly the galaxies inside it are counted, satellites not.
        let inside = r["sky"]["galaxies"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|g| g["satellite"] != true && g["distance_mpc"].as_f64().unwrap() <= radius)
            .count();
        assert_eq!(inside as f64, e["members"].as_f64().unwrap(), "{e}");
    }

    #[test]
    fn our_galaxy_s_satellites_are_its_surviving_subhaloes() {
        let mut k = open();
        let r = ask(
            &mut k,
            r#"{"op":"sky_from_here","limit_mag":13,"bands":["V"],"stars":false}"#,
        );
        let sky = &r["sky"];
        let walk = &sky["galaxy_walk"]["satellites"];
        let r200 = walk["r200_kpc"].as_f64().unwrap();
        // A 1e12 halo's r_200 is about 200 kpc.
        assert!(r200 > 180.0 && r200 < 240.0, "{walk}");
        // Giocoli 2008 above psi 1e-3: about 54.6 subhaloes ever accreted.
        let drawn = walk["subhaloes"].as_f64().unwrap();
        assert!((drawn - 54.6).abs() < 4.0 * 54.6_f64.sqrt(), "{walk}");
        let merged = walk["merged"].as_f64().unwrap();
        assert!(merged > 0.0 && merged < drawn, "{walk}");
        let sats: Vec<&serde_json::Value> = sky["galaxies"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|g| g["satellite"] == true)
            .collect();
        let now = r["sky"]["t_myr"].as_f64().unwrap() / 1000.0;
        let mut classical = 0;
        for g in &sats {
            let rc = g["from_centre_kpc"].as_f64().unwrap();
            // On orbits now (GH2): a satellite can swing out past r200 to
            // its farthest point (the splashback radius, about 1.5 r200).
            assert!(rc <= r200 * 1.5, "{g}");
            // Quenched: no star formation once ram pressure has taken its gas.
            let infall = g["infall_gyr"].as_f64().unwrap();
            if let Some(q) = g["orbit"]["quenched_gyr"].as_f64() {
                assert!(q >= infall, "{g}");
                if q < now {
                    assert_eq!(g["sfr_msun_yr"].as_f64(), Some(0.0), "{g}");
                }
            }
            // Light time inside our halo is under a Myr.
            assert!(g["lookback_myr"].as_f64().unwrap() < 1.0, "{g}");
            if g["absolute"][0]["m_vega"].as_f64().unwrap() < -8.0 {
                classical += 1;
            }
        }
        // The Milky Way has about a dozen satellites brighter than M_V -8
        // (the classical dwarfs and the Magellanic Clouds); hosts like it
        // in simulations scatter by a factor of a few.
        assert!(
            (3..=40).contains(&classical),
            "{classical} of {}",
            sats.len()
        );
        // The same satellite through a 3 degree cone around it.
        let s0 = sats[0];
        let q = serde_json::json!({"op":"sky_from_here","limit_mag":13,"bands":["V"],"stars":false,
            "cone":{"l_deg":s0["l_deg"],"b_deg":s0["b_deg"],"half_angle_deg":3.0}});
        let c = ask(&mut k, &q.to_string());
        let again = c["sky"]["galaxies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|g| g["address"] == s0["address"])
            .cloned();
        assert_eq!(again.as_ref(), Some(s0));
    }
}
