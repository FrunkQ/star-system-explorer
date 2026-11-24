//! TG13: the two universe packs written for SSE4 (`packs/near-future`,
//! `packs/space-opera`) dress the Sol and Kestrel fixtures through TG12
//! dressing rules, stack over the Traveller pack, and declare every tag.

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
pub(super) fn pack(p: &str) -> serde_json::Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
}
pub(super) fn send(k: &mut crate::Kernel, v: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}
fn load(k: &mut crate::Kernel, packs: Vec<serde_json::Value>) -> serde_json::Value {
    send(k, serde_json::json!({"op": "load_stack", "packs": packs}))
}
fn candidates(k: &mut crate::Kernel, scope: &str) -> Vec<serde_json::Value> {
    let r = send(k, serde_json::json!({"op": "dressing", "scope": scope}));
    assert_eq!(r["type"], "dressed", "{r}");
    r["candidates"].as_array().unwrap().clone()
}

const UNIVERSE: [&str; 2] = ["packs/near-future/pack.json", "packs/space-opera/pack.json"];

/// Sol from `packs/base-sol` as the start sheet enters it (`sol.ts`): the
/// Sun at `star:0`, planets at `body:N`, a moon under its planet, SI facts
/// (`body.mass` kg, `orbit.semi_major_axis` m).
fn sol(universe: &str) -> (crate::Kernel, String) {
    sol_with(vec![pack(universe)])
}

/// Sol under the stack `packs`.
pub(super) fn sol_with(packs: Vec<serde_json::Value>) -> (crate::Kernel, String) {
    let mut k = crate::Kernel::new();
    let r = load(&mut k, packs);
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "sol", "seed": "sol"}),
    );
    let sys = "u:sol/sys:0".to_string();
    let sheet = pack("packs/base-sol/pack.json")["provides"]["other"]["sol"].clone();
    let mut at: std::collections::BTreeMap<String, String> = Default::default();
    let mut n: std::collections::BTreeMap<String, u32> = Default::default();
    for b in sheet["bodies"].as_array().unwrap() {
        let id = b["id"].as_str().unwrap().to_string();
        let a = match b["host"].as_str() {
            None => format!("{sys}/star:0"),
            Some(h) => {
                let under = if at[h].contains("/star:") {
                    sys.clone()
                } else {
                    at[h].clone()
                };
                let i = n.entry(under.clone()).or_default();
                *i += 1;
                format!("{under}/body:{}", *i - 1)
            }
        };
        for (f, v) in b["values"].as_object().unwrap() {
            if f.starts_with("system.") {
                continue;
            }
            let r = send(
                &mut k,
                serde_json::json!({"op": "set_authored", "edit": {
                    "address": a, "field": f, "value": v["v"], "occurred_at_s": 0}}),
            );
            assert_eq!(r["type"], "authored", "{r}");
        }
        at.insert(id, a);
    }
    (k, sys)
}

/// The Kestrel Reach, authored from the chart room's scene fixture (its own
/// fact names: `mass_earth`, `a_au`, `surface_k`).
fn kestrel(universe: &str) -> crate::Kernel {
    kestrel_with(vec![pack(universe)])
}

/// The Kestrel Reach under the stack `packs`.
pub(super) fn kestrel_with(packs: Vec<serde_json::Value>) -> crate::Kernel {
    let mut k = crate::Kernel::new();
    let r = load(&mut k, packs);
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "kestrel", "seed": "kestrel reach campaign"}),
    );
    let scene = pack("packages/shell/src/documents/kestrel.scene.json");
    for s in scene["subjects"].as_array().unwrap() {
        for f in s["facts"].as_array().unwrap() {
            if f["value"].is_number() {
                let r = send(
                    &mut k,
                    serde_json::json!({"op": "set_authored", "edit": {
                        "address": s["address"], "field": f["field"], "value": f["value"], "occurred_at_s": 0}}),
                );
                assert_eq!(r["type"], "authored", "{r}");
            }
        }
    }
    k
}

/// At least one port, one base and one faction presence, all explained.
fn dressed_enough(c: &[serde_json::Value], universe: &str, fixture: &str) {
    let templates = pack(universe)["provides"]["other"]["templates"]["templates"].clone();
    let tags_of = |id: &str| -> Vec<String> {
        templates
            .as_array()
            .unwrap()
            .iter()
            .find(|t| t["id"] == id)
            .map(|t| {
                t["tags"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect()
            })
            .unwrap_or_default()
    };
    let port = c.iter().any(|x| {
        tags_of(x["template"].as_str().unwrap())
            .iter()
            .any(|t| t.contains(".port."))
    });
    let base = c.iter().any(|x| {
        let t = tags_of(x["template"].as_str().unwrap());
        !t.iter().any(|t| t.contains(".port.")) && x["where"] != "high-orbit"
    });
    let faction = c.iter().any(|x| x["faction"].is_string());
    assert!(port && base && faction, "{universe} on {fixture}: {c:#?}");
    for x in c {
        assert!(
            x["explain"].as_str().is_some_and(|e| e.len() > 20),
            "{universe} on {fixture}: unexplained {x}"
        );
        assert!(
            x["why"].as_array().is_some_and(|w| !w.is_empty()),
            "{universe} on {fixture}: no facts read for {x}"
        );
    }
}

#[test]
fn each_universe_pack_dresses_sol_and_kestrel() {
    for u in UNIVERSE {
        let (mut k, sys) = sol(u);
        let c = candidates(&mut k, &sys);
        dressed_enough(&c, u, "Sol");
        // Earth is the home-type world (a port and a base); the giants get
        // the giants' rules; the Sun is no world.
        let earth = c
            .iter()
            .filter(|x| x["world"] == "u:sol/sys:0/body:2")
            .count();
        assert!(earth >= 2, "{u}: Earth {c:#?}");
        assert!(
            c.iter().all(|x| x["world"] != "u:sol/sys:0/star:0"),
            "{u}: the Sun"
        );
        if std::env::var("SSE_SHOW").is_ok() {
            for x in &c {
                eprintln!("{u} Sol {} {} {}", x["world"], x["rule"], x["where"]);
            }
        }
        let mut k = kestrel(u);
        let c = candidates(&mut k, "u:kestrel/sys:1");
        dressed_enough(&c, u, "Kestrel");
        // Tarn is the home-type world; Vault the giant; Vault's moons, not Tarn's.
        let on = |w: &str| c.iter().filter(|x| x["world"] == w).count();
        assert!(on("u:kestrel/sys:1/body:3") >= 2, "{u}: Tarn {c:#?}");
        assert_eq!(on("u:kestrel/sys:1/body:0"), 0, "{u}: the star is no world");
        assert_eq!(
            on("u:kestrel/sys:1/body:3/body:0"),
            0,
            "{u}: Tarn a orbits no giant"
        );
        // The same campaign on another device: the same candidates.
        let mut again = kestrel(u);
        assert_eq!(candidates(&mut again, "u:kestrel/sys:1"), c);
    }
}

#[test]
fn each_universe_pack_stacks_over_the_traveller_pack() {
    for u in UNIVERSE {
        let mut k = crate::Kernel::new();
        let r = load(
            &mut k,
            vec![
                pack("packs/base-physics/pack.json"),
                pack("packs/base-time/pack.json"),
                pack("packs/base-reference/pack.json"),
                pack("packs/traveller/pack.json"),
                pack(u),
            ],
        );
        assert_eq!(r["type"], "stack_loaded", "{u}: {r}");
        // And straight over the app's opening packs, with no game system.
        let r = load(
            &mut k,
            vec![
                pack("packs/base-physics/pack.json"),
                pack("packs/base-time/pack.json"),
                pack("packs/base-reference/pack.json"),
                pack(u),
            ],
        );
        assert_eq!(r["type"], "stack_loaded", "{u}: {r}");
    }
    // Both universe packs together load too (their tags are namespaced).
    let mut k = crate::Kernel::new();
    let r = load(
        &mut k,
        vec![
            pack("packs/traveller/pack.json"),
            pack(UNIVERSE[0]),
            pack(UNIVERSE[1]),
        ],
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
}

#[test]
fn a_tag_the_pack_forgot_to_declare_is_refused() {
    for u in UNIVERSE {
        let mut p = pack(u);
        let tags = p["provides"]["other"]["tags"]["tags"]
            .as_array_mut()
            .unwrap();
        let i = tags
            .iter()
            .position(|t| t["id"].as_str().unwrap().contains(".faction."))
            .unwrap();
        tags.remove(i);
        let mut k = crate::Kernel::new();
        let r = load(&mut k, vec![p]);
        assert_eq!(r["type"], "stack_rejected", "{u}: {r}");
        assert!(
            r.to_string()
                .contains("which no pack in the stack declares"),
            "{u}: {r}"
        );
    }
}

#[test]
fn traveller_ports_and_bases_are_dressing_candidates_from_the_codes() {
    let mut k = crate::Kernel::new();
    let r = load(
        &mut k,
        vec![
            pack("packs/base-physics/pack.json"),
            pack("packs/base-atlas/pack.json"),
            pack("packs/base-time/pack.json"),
            pack("packs/traveller/pack.json"),
        ],
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "kestrel", "seed": "k"}),
    );
    let text =
        std::fs::read_to_string(root().join("tests/fixtures/sector/lanternfall.tab")).unwrap();
    let five: String = text.lines().take(5).map(|l| format!("{l}\n")).collect();
    let r = send(
        &mut k,
        serde_json::json!({"op": "import_sector", "text": five, "apply": true, "at_s": 0}),
    );
    assert_eq!(r["type"], "sector_import", "{r}");
    let c = candidates(&mut k, "u:kestrel");
    let placed = |w: &str| -> Vec<String> {
        let mut v: Vec<String> = c
            .iter()
            .filter(|x| x["world"] == w)
            .map(|x| x["template"].as_str().unwrap().to_string())
            .collect();
        v.sort();
        v
    };
    // Ashgrove A867975-9, bases N S: highport, downport, naval and scout bases.
    assert_eq!(
        placed("u:kestrel/sys:0912/body:@1"),
        ["downport", "highport_a", "naval_base", "scout_base"]
    );
    // Brannock's Hold B, S. Cinder Gate E, no bases. Dunmere C, N W.
    assert_eq!(
        placed("u:kestrel/sys:1013/body:@1"),
        ["downport", "highport_b", "scout_base"]
    );
    assert_eq!(placed("u:kestrel/sys:1114/body:@1"), ["beacon_e"]);
    assert_eq!(
        placed("u:kestrel/sys:1215/body:@1"),
        ["downport", "naval_base", "way_station"]
    );
    // Each reads the column it came from.
    assert!(c
        .iter()
        .all(|x| x["why"].as_array().unwrap().iter().any(|w| w["field"]
            .as_str()
            .unwrap()
            .starts_with("society.starport")
            || w["field"].as_str().unwrap().starts_with("traveller.bases."))));
}

/// Sector routes as links (X3l): a metadata file's routes between two
/// imported worlds become charted links, known to the GM until told; the
/// rest are listed with the reason. Importing again rewrites the same links.
#[test]
fn sector_routes_become_links_known_to_the_gm() {
    let mut k = crate::Kernel::new();
    let r = load(
        &mut k,
        vec![
            pack("packs/base-physics/pack.json"),
            pack("packs/base-atlas/pack.json"),
            pack("packs/base-time/pack.json"),
            pack("packs/traveller/pack.json"),
        ],
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    send(
        &mut k,
        serde_json::json!({"op": "open_universe", "id": "kestrel", "seed": "k"}),
    );
    let text =
        std::fs::read_to_string(root().join("tests/fixtures/sector/lanternfall.tab")).unwrap();
    let meta =
        std::fs::read_to_string(root().join("tests/fixtures/sector/kestrel-reach.xml")).unwrap();
    let ask = |apply: bool| {
        serde_json::json!({"op": "import_sector", "text": text, "metadata": meta,
            "apply": apply, "at_s": 0})
    };
    let links = |k: &mut crate::Kernel, audience: &str| -> Vec<serde_json::Value> {
        let r = send(
            k,
            serde_json::json!({"op": "links_in", "audience": audience}),
        );
        assert_eq!(r["type"], "links_listed", "{r}");
        r["links"].as_array().unwrap().clone()
    };

    let r = send(&mut k, ask(false));
    assert_eq!(r["type"], "sector_import", "{r}");
    let routes = r["routes"].as_array().unwrap();
    assert_eq!(routes.len(), 4, "{r}");
    assert_eq!(routes[0]["a"], "u:kestrel/sys:0912");
    assert_eq!(routes[0]["b"], "u:kestrel/sys:1013");
    assert!(routes[0].get("link").is_none(), "a preview writes nothing");
    assert!(routes[2]["left_out"]
        .as_str()
        .unwrap()
        .contains("neighbouring sector"));
    assert!(routes[3]["left_out"].as_str().unwrap().contains("2525"));
    assert!(r.get("record_hash").is_none());
    assert!(links(&mut k, "gm").is_empty());

    let r = send(&mut k, ask(true));
    assert_eq!(
        r["routes"][0]["link"],
        "u:kestrel/link:route-0912-1013-trade"
    );
    assert_eq!(
        r["routes"][1]["link"],
        "u:kestrel/link:route-1013-1114-courier"
    );
    assert!(r["record_hash"].is_string());
    let gm = links(&mut k, "gm");
    assert_eq!(gm.len(), 2, "{gm:?}");
    let trade = gm.iter().find(|l| l["kind"] == "Trade").unwrap();
    assert_eq!(trade["source"], "import_sector", "{trade}");
    assert_eq!(trade["name"], "Trade route 0912 to 1013");
    assert!(
        links(&mut k, "players").is_empty(),
        "the players do not know them yet"
    );

    // Again, told to everyone: the same two links, now known to the players.
    let mut again = ask(true);
    again["route_audience"] = serde_json::json!("everyone");
    let r = send(&mut k, again);
    assert_eq!(r["type"], "sector_import", "{r}");
    assert_eq!(links(&mut k, "gm").len(), 2);
    assert_eq!(links(&mut k, "players").len(), 2);

    let mut bad = ask(false);
    bad["route_audience"] = serde_json::json!(7);
    assert_eq!(send(&mut k, bad)["code"], "bad_link");
}
