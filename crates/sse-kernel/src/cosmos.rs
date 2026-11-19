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

fn settings() -> Result<CosmologyPack, Response> {
    CosmologyPack::from_packs(&[base_cosmology_pack()]).map_err(|e| err("bad_pack", e))
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
}

/// One galaxy: its history curves, its rings now (abundances, deaths, rates),
/// its green band through time, the cosmic epochs, and the conditions at one
/// place when asked.
pub(crate) fn galaxy_chart(a: GalaxyAsk) -> Result<serde_json::Value, Response> {
    let cp = settings()?;
    let c = Cosmology::new(cp.params.clone());
    let w = web(&c, &cp, a.seed, a.root)?;
    let hs = Haloes {
        cosmo: &c,
        web: &w,
        params: cp.haloes.clone(),
        seed: a.seed.clone(),
        universe: a.root.clone(),
    };
    let (key, mass) = match (a.halo, a.mass_msun) {
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
    let h = hs.median(mass, &key);
    let g = sse_cosmos::galaxy::evolve(&c, &cp.galaxy, &h);
    let addr: Address = h
        .address
        .parse()
        .map_err(|e| err("bad_address", format!("{e:?}")))?;
    let d = rings::evolve(&g, &cp.galaxy, &cp.rings, a.seed.clone(), &addr);
    let env = GalaxyEnvelope {
        galaxy: &g,
        disc: &d,
        params: cp.envelope.clone(),
        seed: a.seed.clone(),
        address: addr.clone(),
    };
    let now = g.now.t_gyr;
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
        bands.push(serde_json::json!({ "t_gyr": t, "habitable": band(false), "complex": band(true) }));
    }
    // Arm crests at `at` (where the envelope's arm term peaks), as (r, phi) polylines.
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
                    let phi = omega_p * at
                        + m::ln(r / rd) / tan_pitch
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
    Ok(serde_json::json!({
        "halo": h.address,
        "key": key,
        "halo_mass_msun": mass,
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
        "epochs": sse_cosmos::epochs::epochs(&c, &cp.epochs),
        "place": place,
        "model": [
            "galaxy: a halo's median growth, gas and star formation by published relations (G3), not a simulation",
            "rings: chemical evolution ring by ring with delayed type Ia and neutron-star mergers",
            "green band: the G4 criteria (metals, supernovae, passages, bursts, a quiet nucleus, time)",
        ],
    }))
}

pub(crate) struct SliceAsk<'a> {
    pub seed: &'a Seed,
    pub root: &'a Address,
    pub z: Option<f64>,
    pub t_gyr: Option<f64>,
    pub centre: [f64; 3],
    pub extent_mpc: f64,
    pub n: u32,
}

/// A square foam slice of the cosmic web (G1) at one epoch, for V9's maps.
pub(crate) fn web_slice(a: SliceAsk) -> Result<serde_json::Value, Response> {
    if a.n < 4 || a.n > 256 {
        return Err(err("bad_request", "n must be 4 to 256 samples a side"));
    }
    if !(a.extent_mpc > 0.0 && a.extent_mpc <= 2000.0) {
        return Err(err("bad_request", "extent_mpc must be above 0 and at most 2000"));
    }
    let cp = settings()?;
    let c = Cosmology::new(cp.params.clone());
    let w = web(&c, &cp, a.seed, a.root)?;
    let scale = match (a.z, a.t_gyr) {
        (Some(z), None) if z > -0.99 => 1.0 / (1.0 + z),
        (None, Some(t)) if t > 0.0 => c.scale_of_time(t).min(1.0),
        (None, None) => 1.0,
        _ => return Err(err("bad_request", "give a redshift z or a time t_gyr, not both")),
    };
    let octaves = w.params.octaves;
    let s = w.slice(&c, scale, a.centre, a.extent_mpc, a.n, octaves);
    Ok(serde_json::json!({
        "t_gyr": c.time_of_scale(scale),
        "z": 1.0 / scale - 1.0,
        "now_gyr": c.age_gyr(),
        "slice": s,
        "model": ["the cosmic web from seeded Gaussian fields, grown linearly and moved by the Zel'dovich approximation (G1); a picture of the large-scale pattern, not an N-body simulation"],
    }))
}

#[cfg(test)]
mod tests {
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
        let r = ask(&mut k, r#"{"op":"galaxy_chart","halo":"milky-way-like","place_kpc":8.0}"#);
        assert_eq!(r["type"], "galaxy_charted", "{r}");
        let c = &r["chart"];
        // The same galaxy as G3's golden (seed g3, universe g3).
        assert_eq!(c["morphology"], "barred");
        assert_eq!(c["halo"], "u:g3/halo:milky-way-like");
        let golden: serde_json::Value = serde_json::from_str(
            &std::fs::read_to_string(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/galaxy.json"),
            )
            .unwrap(),
        )
        .unwrap();
        let g = &golden["galaxies"][1];
        assert_eq!(c["now"]["stars"], g["stars"]);
        assert_eq!(c["now"]["sfr"], g["sfr"]);
        assert_eq!(c["core_collapse_per_century"], g["core_collapse_per_century"]);
        assert_eq!(c["rings"].as_array().unwrap().len(), 24);
        assert!(c["history"].as_array().unwrap().len() > 10);
        assert!(c["green_bands"].as_array().unwrap().iter().any(|b| !b["habitable"].is_null()));
        assert!(c["place"]["local"]["feh"].is_number());
        assert!(c["epochs"].as_array().unwrap().len() > 3);
        let again = ask(&mut k, r#"{"op":"galaxy_chart","halo":"milky-way-like","place_kpc":8.0}"#);
        assert_eq!(r, again);
        let bad = ask(&mut k, r#"{"op":"galaxy_chart","halo":"andromeda"}"#);
        assert_eq!(bad["code"], "unknown_halo");
    }

    #[test]
    fn web_slice_grows_contrast_with_time() {
        let mut k = open();
        let early = ask(&mut k, r#"{"op":"web_slice","z":3.0,"extent_mpc":100,"n":16}"#);
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
}
