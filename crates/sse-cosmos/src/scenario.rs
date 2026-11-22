//! The G0 golden scenario: the Sun's neighbourhood now, a 16 pc leaf,
//! resolved to its systems. Used by the tests and the browser harness.

use crate::{form_patch, PatchModel, Site, SystemBirth};
use sse_chronicle::Chronicle;
use sse_core::{Address, ContentHash, Seed};

pub fn seed() -> Seed {
    Seed::from_text("g0")
}
pub fn address() -> Address {
    "u:g0/patch:9:solar".parse().expect("valid address")
}

/// The envelope fixture formation stages read: the solar patch envelope and
/// its cohort envelopes, as JSON.
pub fn fixture() -> serde_json::Value {
    let model = PatchModel::milky_way_like();
    let mut c = Chronicle::new();
    let f = form_patch(&mut c, &seed(), &model, address(), &Site::solar(&model)).expect("forms");
    serde_json::json!({
        "about": "G0 envelope fixture: the default Milky-Way-like patch at the Sun's radius now (16 pc leaf), its conditions, and its cohort envelopes. Regenerate with SSE_BLESS=1 cargo test -p sse-cosmos.",
        "seed": "g0",
        "conditions": f.conditions,
        "patch": f.envelope,
        "cohorts": f.cohorts.iter().map(|(_, e)| e).collect::<Vec<_>>(),
    })
}

pub fn golden() -> serde_json::Value {
    let model = PatchModel::milky_way_like();
    let mut c = Chronicle::new();
    let f = form_patch(&mut c, &seed(), &model, address(), &Site::solar(&model)).expect("forms");
    let births: Vec<SystemBirth> = f
        .systems
        .iter()
        .map(|(a, r)| SystemBirth::from_facts(a, &r.facts).expect("typed"))
        .collect();
    let cond = serde_json::to_string(&f.conditions).expect("json");
    serde_json::json!({
        "systems": births.len(),
        "events": c.len(),
        "state": c.state_at(f.envelope.epoch).hash().hex(),
        "births_digest": ContentHash::of(&serde_json::to_string(&births).expect("json")).expect("hash").hex(),
        "conditions_digest": ContentHash::of(&cond).expect("hash").hex(),
    })
}

// ---- G1: cosmology, epochs and the web ----

use crate::cosmology::{Cosmology, Params};
use crate::epochs::{self, EpochParams};
use crate::web::{Cache, Web, WebParams};
use serde::{Deserialize, Serialize};
use sse_pack::PackManifest;

/// The cosmology pack's settings (`provides.other.cosmology`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct CosmologyPack {
    pub params: Params,
    pub web: WebParams,
    pub epochs: EpochParams,
    #[serde(default)]
    pub haloes: crate::haloes::HaloParams,
    #[serde(default)]
    pub galaxy: crate::galaxy::GalaxyParams,
    #[serde(default)]
    pub rings: crate::rings::RingParams,
    #[serde(default)]
    pub envelope: crate::envelope::EnvelopeParams,
    #[serde(default)]
    pub green: crate::envelope::GreenParams,
    #[serde(default)]
    pub stars: crate::stars::StarParams,
    #[serde(default)]
    pub events: crate::events::EventParams,
    #[serde(default)]
    pub nucleus: crate::nucleus::NucleusParams,
    /// Y2: the sky draw.
    #[serde(default)]
    pub sky: crate::sky::SkyParams,
    #[serde(default)]
    pub origin: crate::origin::OriginParams,
}

impl CosmologyPack {
    pub fn from_packs(stack: &[PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("cosmology"))
            .ok_or("no pack provides cosmology")?;
        serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("cosmology settings: {e}"))
    }
}

pub fn base_cosmology_pack() -> PackManifest {
    PackManifest::from_json(include_str!("../../../packs/base-cosmology/pack.json"))
        .expect("base cosmology pack")
}

/// G1 golden: the cosmology's key numbers, its epochs, a digest of the
/// sigma(M) table, a foam slice and T-web counts, identical on every device.
pub fn cosmology_golden() -> serde_json::Value {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).expect("settings");
    let c = Cosmology::new(cp.params.clone());
    let ep = epochs::epochs(&c, &cp.epochs);
    let u = Address::universe("g1").expect("address");
    let web = Web::new(
        &c,
        WebParams {
            octaves: 5,
            ..cp.web.clone()
        },
        &Seed::from_text("g1"),
        &u,
    )
    .expect("web");
    let slice = web.slice(&c, 1.0, [0.0; 3], 128.0, 12, 4);
    let mut cache = Cache::default();
    let mut counts = [0u32; 4];
    for i in 0..200 {
        let x = [
            f64::from(i) * 7.3,
            f64::from(i % 17) * 11.1,
            f64::from(i % 5) * 3.7,
        ];
        counts[web.environment(&mut cache, x, 3, 1.0) as usize] += 1;
    }
    let mut ch = Chronicle::new();
    epochs::record(&mut ch, &u, &ep).expect("epochs");
    let hex = |v: &dyn erased::Json| ContentHash::of(&v.json()).expect("hash").hex();
    serde_json::json!({
        "age_gyr": c.age_gyr(),
        "omega_r": c.omega_r,
        "epochs": ep.iter().map(|e| serde_json::json!([e.id, e.log10_yr, e.z])).collect::<Vec<_>>(),
        "sigma_table": hex(&c.sigma_table),
        "slice": hex(&slice),
        "t_web_counts": counts,
        "chronicle_head": ch.head().hex(),
    })
}

mod erased {
    pub trait Json {
        fn json(&self) -> String;
    }
    impl<T: serde::Serialize> Json for T {
        fn json(&self) -> String {
            serde_json::to_string(self).expect("json")
        }
    }
}

/// G2 golden: two cells' halo catalogues (digests, counts, masses) and their
/// merger events, identical on every device.
pub fn haloes_golden() -> serde_json::Value {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).expect("settings");
    let c = Cosmology::new(cp.params.clone());
    let u = Address::universe("g2").expect("address");
    let web = Web::new(&c, cp.web.clone(), &Seed::from_text("g2"), &u).expect("web");
    let hs = crate::haloes::Haloes {
        cosmo: &c,
        web: &web,
        params: crate::haloes::HaloParams {
            min_mass: 1.0e11,
            ..cp.haloes.clone()
        },
        seed: Seed::from_text("g2"),
        universe: u,
    };
    let mut ch = Chronicle::new();
    let cells: Vec<serde_json::Value> = [[0, 0, 0], [3, -2, 1]]
        .iter()
        .map(|i| {
            let cell = hs.cell(*i);
            hs.record(&mut ch, &cell).expect("record");
            serde_json::json!({
                "address": cell.address,
                "delta_l": cell.delta_l,
                "haloes": cell.haloes.len(),
                "halo_mass": cell.halo_mass,
                "mergers": cell.haloes.iter().map(|h| h.mergers.len()).sum::<usize>(),
                "digest": ContentHash::of(&serde_json::to_string(&cell).expect("json")).expect("hash").hex(),
            })
        })
        .collect();
    serde_json::json!({ "cells": cells, "chronicle_head": ch.head().hex() })
}

/// G3 golden: a Milky-Way-like galaxy in a median 1e12 halo and its rings,
/// plus a dwarf and a group galaxy: present numbers and digests of the full
/// histories, and the chronicle head after its active-nucleus events.
pub fn galaxy_golden() -> serde_json::Value {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).expect("settings");
    let c = Cosmology::new(cp.params.clone());
    let u = Address::universe("g3").expect("address");
    let web = Web::new(
        &c,
        WebParams {
            octaves: 4,
            ..cp.web.clone()
        },
        &Seed::from_text("g3"),
        &u,
    )
    .expect("web");
    let hs = crate::haloes::Haloes {
        cosmo: &c,
        web: &web,
        params: cp.haloes.clone(),
        seed: Seed::from_text("g3"),
        universe: u,
    };
    let mut ch = Chronicle::new();
    let gals: Vec<serde_json::Value> = [("dwarf", 1.0e11), ("milky-way-like", 1.0e12), ("group", 1.0e13)]
        .iter()
        .map(|(key, mass)| {
            let h = hs.median(*mass, key);
            let g = crate::galaxy::evolve(&c, &cp.galaxy, &h);
            crate::galaxy::record(&mut ch, &g).expect("record");
            let addr: Address = h.address.parse().expect("address");
            let d = crate::rings::evolve(&g, &cp.galaxy, &cp.rings, Seed::from_text("g3"), &addr);
            let r8 = d.ring_at(8.2 * g.now.disc_kpc / 2.6);
            serde_json::json!({
                "halo": h.address,
                "morphology": g.morphology,
                "stars": g.now.stars,
                "sfr": g.now.sfr,
                "z_gas": g.now.z_gas,
                "black_hole": g.now.black_hole,
                "disc_kpc": g.now.disc_kpc,
                "solar_ring_feh": r8.abundance(crate::rings::FE),
                "core_collapse_per_century": d.core_collapse_rate() * 100.0,
                "ia_per_century": d.ia_rate() * 100.0,
                "galaxy_digest": ContentHash::of(&serde_json::to_string(&g).expect("json")).expect("hash").hex(),
                "rings_digest": ContentHash::of(&serde_json::to_string(&d).expect("json")).expect("hash").hex(),
            })
        })
        .collect();
    serde_json::json!({ "galaxies": gals, "chronicle_head": ch.head().hex() })
}

/// G4 golden: the Milky-Way-like galaxy's envelope at a few places and times,
/// its green band through time, outcome-first finds, and the derived patch model.
pub fn envelope_golden() -> serde_json::Value {
    use crate::envelope::{GalaxyEnvelope, Place};
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).expect("settings");
    let c = Cosmology::new(cp.params.clone());
    let u = Address::universe("g3").expect("address");
    let web = Web::new(
        &c,
        WebParams {
            octaves: 4,
            ..cp.web.clone()
        },
        &Seed::from_text("g3"),
        &u,
    )
    .expect("web");
    let hs = crate::haloes::Haloes {
        cosmo: &c,
        web: &web,
        params: cp.haloes.clone(),
        seed: Seed::from_text("g3"),
        universe: u,
    };
    let h = hs.median(1.0e12, "milky-way-like");
    let g = crate::galaxy::evolve(&c, &cp.galaxy, &h);
    let addr: Address = h.address.parse().expect("address");
    let d = crate::rings::evolve(&g, &cp.galaxy, &cp.rings, Seed::from_text("g3"), &addr);
    let e = GalaxyEnvelope {
        galaxy: &g,
        disc: &d,
        params: cp.envelope.clone(),
        seed: Seed::from_text("g4"),
        address: addr,
    };
    let r0 = e.sun_radius_kpc();
    let places: Vec<serde_json::Value> = [
        (0.5, 0.0, 0.0, 13.8),
        (1.0, 1.0, 20.0, 13.8),
        (1.0, 2.5, 300.0, 9.0),
        (1.6, 4.0, -50.0, 13.8),
    ]
    .iter()
    .map(|(f, phi, z, t)| {
        let l = e.local(
            Place {
                r_kpc: r0 * f,
                phi: *phi,
                z_pc: *z,
                t_gyr: *t,
            },
            &cp.green,
        );
        let gr = e.green(&l, &cp.green);
        serde_json::json!({ "local": l, "green": gr })
    })
    .collect();
    let bands: Vec<serde_json::Value> = [4.0, 6.0, 8.0, 10.0, 12.0, 13.8]
        .iter()
        .map(|t| serde_json::json!({ "t_gyr": t, "habitable": e.green_band(*t, &cp.green, false, 0.25), "complex": e.green_band(*t, &cp.green, true, 0.25) }))
        .collect();
    let found: Vec<serde_json::Value> = e
        .find_green(13.8, &cp.green, 3)
        .iter()
        .map(|(l, _)| serde_json::json!(l.place))
        .collect();
    let pm = e.patch_model(&crate::PatchModel::milky_way_like());
    serde_json::json!({
        "places": places,
        "bands": bands,
        "found": found,
        "patch_model": { "feh_now_at_sun": pm.enrichment.feh_now_at_sun, "gradient": pm.enrichment.gradient_dex_per_kpc, "tau_gyr": pm.enrichment.tau_gyr, "sun_radius_kpc": pm.sun_radius_kpc },
    })
}

/// G5 golden: the cell holding the Sun's place now and in the frame before,
/// with one promoted system read at three times.
pub fn stars_golden() -> serde_json::Value {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).expect("settings");
    let c = Cosmology::new(cp.params.clone());
    let u = Address::universe("g3").expect("address");
    let web = Web::new(
        &c,
        WebParams {
            octaves: 4,
            ..cp.web.clone()
        },
        &Seed::from_text("g3"),
        &u,
    )
    .expect("web");
    let hs = crate::haloes::Haloes {
        cosmo: &c,
        web: &web,
        params: cp.haloes.clone(),
        seed: Seed::from_text("g3"),
        universe: u,
    };
    let h = hs.median(1.0e12, "milky-way-like");
    let g = crate::galaxy::evolve(&c, &cp.galaxy, &h);
    let addr: Address = h.address.parse().expect("address");
    let d = crate::rings::evolve(&g, &cp.galaxy, &cp.rings, Seed::from_text("g3"), &addr);
    let e = crate::envelope::GalaxyEnvelope {
        galaxy: &g,
        disc: &d,
        params: cp.envelope.clone(),
        seed: Seed::from_text("g4"),
        address: addr,
    };
    let st = crate::stars::Stars {
        env: &e,
        green: cp.green.clone(),
        params: cp.stars.clone(),
        seed: Seed::from_text("g5"),
        galaxy: Address::universe("g5").expect("address"),
    };
    let now = st.frame_of(13_800.0);
    let t = now as f64 * cp.stars.frame_myr;
    let sun = st.from_grid([e.sun_radius_kpc() * 1000.0, 0.0, 0.0], t);
    let cell = st.cell_at(sun, t);
    let a = st.cell(cell, now, &[]).expect("cell");
    let b = st.cell(cell, now - 1, &[]).expect("cell");
    let pin = st.promote(&a[0]);
    let reads: Vec<serde_json::Value> = [t - 2000.0, t, t + 500.0]
        .iter()
        .map(|x| serde_json::json!(st.at(&pin, *x)))
        .collect();
    serde_json::json!({
        "cell": cell,
        "now": { "count": a.len(), "digest": sse_core::ContentHash::of(&a).expect("hash").hex() },
        "before": { "count": b.len(), "digest": sse_core::ContentHash::of(&b).expect("hash").hex() },
        "pin": pin,
        "reads": reads,
        "birth": st.birth(&pin),
    })
}

/// The G6 "ink a merger" manifest (`tests/fixtures/cosmos/ink-a-merger.json`).
pub fn ink_a_merger() -> crate::events::Manifest {
    serde_json::from_str(include_str!(
        "../../../tests/fixtures/cosmos/ink-a-merger.json"
    ))
    .expect("manifest")
}

/// G6 golden: the "ink a merger" scenario's report.
pub fn galaxy_events_golden() -> serde_json::Value {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).expect("settings");
    crate::events::run_manifest(&cp, &ink_a_merger()).expect("scenario runs")
}

/// Nuclei golden: the black holes of the G3 dwarf, Milky-Way-like and group
/// galaxies, their light today and at their brightest, mergers, a window of
/// tidal disruptions and digests of the full nuclei.
pub fn nucleus_golden() -> serde_json::Value {
    let cp = CosmologyPack::from_packs(&[base_cosmology_pack()]).expect("settings");
    let c = Cosmology::new(cp.params.clone());
    let u = Address::universe("g3").expect("address");
    let web = Web::new(
        &c,
        WebParams {
            octaves: 4,
            ..cp.web.clone()
        },
        &Seed::from_text("g3"),
        &u,
    )
    .expect("web");
    let hs = crate::haloes::Haloes {
        cosmo: &c,
        web: &web,
        params: cp.haloes.clone(),
        seed: Seed::from_text("g3"),
        universe: u,
    };
    let np = &cp.nucleus;
    let nuclei: Vec<serde_json::Value> = [("dwarf", 1.0e11), ("milky-way-like", 1.0e12), ("group", 1.0e13)]
        .iter()
        .map(|(key, mass)| {
            let h = hs.median(*mass, key);
            let g = crate::galaxy::evolve(&c, &cp.galaxy, &h);
            let n = crate::nucleus::nucleus(&c, &cp.galaxy, np, &h, &g, &[], Seed::from_text("g3"));
            let brightest = n
                .history
                .iter()
                .fold(&n.now, |b, p| if p.luminosity_w > b.luminosity_w { p } else { b });
            let tdes = crate::nucleus::tidal_disruptions(&n, np, Seed::from_text("g3"), 13.0, 13.01);
            serde_json::json!({
                "address": n.address,
                "seed": n.seed.channel,
                "mass_msun": n.now.mass_msun,
                "class": n.now.class.name(),
                "eddington_ratio": n.now.eddington_ratio,
                "luminosity_w": n.now.luminosity_w,
                "brightest_gyr": brightest.t_gyr,
                "brightest_w": brightest.luminosity_w,
                "episodes": n.episodes.len(),
                "mergers": n.mergers.len(),
                "radio_loud": n.radio_loud,
                "tde_rate_per_yr": n.tde_rate_per_yr,
                "tdes_13_to_13_01_gyr": tdes.len(),
                "digest": ContentHash::of(&serde_json::to_string(&n).expect("json")).expect("hash").hex(),
            })
        })
        .collect();
    serde_json::json!({ "nuclei": nuclei })
}
