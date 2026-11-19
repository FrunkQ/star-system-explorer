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
