//! TP4: the process declarations, the pace ladder, and the check that every
//! kernel op taking a date is named by a declaration (or is still on the
//! ledger `handover/packages/TP4-undeclared-ops.txt`, which only shrinks).

use serde_json::{json, Value};
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn pack(p: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(root().join(p)).unwrap()).unwrap()
}

fn send(k: &mut crate::Kernel, v: Value) -> Value {
    serde_json::from_str(&k.handle_json(&v.to_string())).unwrap()
}

const LEDGER: &str = "handover/packages/TP4-undeclared-ops.txt";

/// Whether a request field holds a date (or an age or a time on the axis):
/// a name ending `_s`, `_yr`, `_myr` or `_gyr` whose last word before it
/// says when (`at_s`, `occurred_at_s`, `t_myr`, `age_myr`, `true_on_s`).
/// Spans (`hours`, `wait_days`) are not dates and are not counted.
fn is_date_field(name: &str) -> bool {
    const UNITS: &[&str] = &["_s", "_yr", "_myr", "_gyr"];
    const WHEN: &[&str] = &[
        "at", "t", "now", "target", "depart", "from", "to", "since", "until", "age", "epoch", "on",
        "when", "date",
    ];
    UNITS.iter().any(|u| {
        name.strip_suffix(u)
            .and_then(|stem| stem.rsplit('_').next())
            .is_some_and(|w| WHEN.contains(&w))
    })
}

/// Every request op that takes a date, from the protocol's source: the
/// fields of each `Request` variant, and each variant's op name from
/// `Request::op`. Textual on purpose, like the other ledgers: no build of
/// a schema, and a new field is seen the moment it is written.
fn dated_ops(src: &str) -> Vec<String> {
    let start = src.find("pub enum Request {").expect("the Request enum");
    let body = &src[start..];
    let end = body.find("\n}\n").expect("the Request enum ends");
    let mut fields: Vec<(String, Vec<String>)> = Vec::new();
    for line in body[..end].lines().skip(1) {
        if let Some(rest) = line.strip_prefix("    ") {
            let first = rest.chars().next().unwrap_or(' ');
            if first.is_ascii_uppercase() {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric())
                    .collect();
                fields.push((name, Vec::new()));
                continue;
            }
        }
        if let Some(rest) = line.strip_prefix("        ") {
            let rest = rest.strip_prefix("pub ").unwrap_or(rest);
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '_')
                .collect();
            if !name.is_empty() && rest[name.len()..].trim_start().starts_with(':') {
                if let Some(v) = fields.last_mut() {
                    v.1.push(name);
                }
            }
        }
    }
    let ops_at = src.find("pub fn op(&self)").expect("Request::op");
    let ops_src = &src[ops_at..];
    let op_of = |variant: &str| -> String {
        for pat in [
            format!("Request::{variant} {{ .. }} => \""),
            format!("Request::{variant} => \""),
        ] {
            if let Some(i) = ops_src.find(&pat) {
                let s = &ops_src[i + pat.len()..];
                return s[..s.find('"').unwrap()].to_string();
            }
        }
        panic!("Request::{variant} has no op name in Request::op");
    };
    let mut out: Vec<String> = fields
        .iter()
        .filter(|(_, f)| f.iter().any(|n| is_date_field(n)))
        .map(|(v, _)| op_of(v))
        .collect();
    out.sort();
    out
}

fn ledger(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// The ops the live base-time pack's declarations name.
fn declared_ops() -> Vec<String> {
    let p = pack("packs/base-time/pack.json");
    let mut v: Vec<String> = p["provides"]["other"]["processes"]["list"]
        .as_array()
        .expect("base-time declares its processes")
        .iter()
        .flat_map(|d| d["ops"].as_array().unwrap().clone())
        .map(|o| o.as_str().unwrap().to_string())
        .collect();
    v.sort();
    v.dedup();
    v
}

/// What is wrong with the ledger against the source and the declarations:
/// a dated op that is neither declared nor on the ledger; a ledger line
/// that is declared now (lower the ledger); a ledger line that takes no
/// date or is no op (delete it).
fn faults(dated: &[String], declared: &[String], led: &[String]) -> Vec<String> {
    let mut f = Vec::new();
    for o in dated {
        if !declared.contains(o) && !led.contains(o) {
            f.push(format!(
                "op {o} takes a date and no process declares it: add it to a declaration's ops in \
                 packs/base-time (provides.other.processes), or give it one (time design section 10)"
            ));
        }
    }
    for o in led {
        if declared.contains(o) {
            f.push(format!(
                "op {o} is declared now: delete its line from {LEDGER} (the ledger only shrinks)"
            ));
        } else if !dated.contains(o) {
            f.push(format!(
                "{LEDGER} lists {o}, which is not an op that takes a date: delete its line"
            ));
        }
    }
    f
}

#[test]
fn every_dated_op_is_declared_or_on_the_ledger() {
    let src = std::fs::read_to_string(root().join("crates/sse-kernel/src/protocol.rs")).unwrap();
    let dated = dated_ops(&src);
    assert!(dated.len() > 80, "found only {} dated ops", dated.len());
    for o in &dated {
        assert!(crate::pace::is_op(o), "{o} is not a request op");
    }
    let declared = declared_ops();
    let led = ledger(&std::fs::read_to_string(root().join(LEDGER)).unwrap());
    let f = faults(&dated, &declared, &led);
    assert!(f.is_empty(), "{}", f.join("\n"));
}

/// The check goes red: a new op with a date and no declaration, and a
/// ledger line left behind after its op was declared.
#[test]
fn the_check_fails_an_undeclared_dated_op() {
    let src = std::fs::read_to_string(root().join("crates/sse-kernel/src/protocol.rs")).unwrap();
    let fake = src.replacen(
        "pub enum Request {\n",
        "pub enum Request {\n    TideAt {\n        body: String,\n        at_s: i64,\n    },\n",
        1,
    );
    let fake = fake.replacen(
        "Request::Hello => \"hello\",",
        "Request::Hello => \"hello\",\n            Request::TideAt { .. } => \"tide_at\",",
        1,
    );
    let dated = dated_ops(&fake);
    assert!(dated.contains(&"tide_at".to_string()));
    let declared = declared_ops();
    let led = ledger(&std::fs::read_to_string(root().join(LEDGER)).unwrap());
    let f = faults(&dated, &declared, &led);
    assert!(
        f.len() == 1 && f[0].contains("op tide_at takes a date"),
        "{f:?}"
    );
    let mut led2 = led.clone();
    led2.push("orbits_at".into());
    let f = faults(&dated_ops(&src), &declared, &led2);
    assert!(
        f.len() == 1 && f[0].contains("orbits_at is declared now"),
        "{f:?}"
    );
    // A span is not a date; a temperature is not one either.
    assert!(is_date_field("occurred_at_s") && is_date_field("t_myr"));
    assert!(is_date_field("age_myr") && is_date_field("true_on_s"));
    assert!(!is_date_field("t_k") && !is_date_field("duration_s"));
    assert!(!is_date_field("mean_anomaly_at_epoch_rad"));
}

fn live_kernel() -> crate::Kernel {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        json!({"op": "load_stack", "packs": [pack("packs/base-reference/pack.json"), pack("packs/base-time/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    k
}

/// The shipped declarations all work out, in order, at believable paces.
#[test]
fn the_live_ladder_works_out_every_rung() {
    let mut k = live_kernel();
    let r = send(&mut k, json!({"op": "pace_ladder"}));
    assert_eq!(r["type"], "pace_ladder", "{r}");
    let l = &r["ladder"];
    assert_eq!(l["refused"], json!([]), "{l}");
    let rungs = l["rungs"].as_array().unwrap();
    let day = 86_400.0;
    let yr = 365.25 * day;
    let tau = |id: &str, subject: &str| -> f64 {
        rungs
            .iter()
            .find(|g| g["id"] == id && g["subject"].as_str().is_some_and(|s| s.contains(subject)))
            .unwrap_or_else(|| panic!("no rung {id} for {subject}: {l}"))["tau_s"]
            .as_f64()
            .unwrap()
    };
    let near = |x: f64, want: f64, tol: f64| (x / want - 1.0).abs() < tol;
    let moon = tau("orbit.kepler", "Moon");
    assert!(near(moon, 27.32 * day, 0.01), "Moon {}", moon / day);
    let year = tau("orbit.kepler", "1 au");
    assert!(near(year, 365.256 * day, 1e-4), "year {}", year / day);
    let sun = tau("star.life", "Sun");
    assert!(near(sun, 1.0e10 * yr, 1e-9), "Sun {}", sun / yr);
    assert_eq!(
        tau("system.ageing", "Sun"),
        sun,
        "the march follows its driver"
    );
    let turn = tau("star.activity", "Sun");
    assert!(
        turn > 5.0 * day && turn < 40.0 * day,
        "turnover {}",
        turn / day
    );
    let hubble = tau("cosmos.expansion", "today");
    assert!(near(hubble, 14.45e9 * yr, 0.01), "Hubble {}", hubble / yr);
    // Fastest first; the campaign's record, paced by its marks, last.
    let ts: Vec<Option<f64>> = rungs.iter().map(|g| g["tau_s"].as_f64()).collect();
    let n = ts.iter().filter(|t| t.is_some()).count();
    assert!(ts[..n].windows(2).all(|w| w[0] <= w[1]), "{ts:?}");
    assert!(ts[n..].iter().all(Option::is_none));
    let last = rungs.last().unwrap();
    assert_eq!(last["id"], "campaign.record");
    assert!(last["why"].is_null(), "{last}");
}

/// Without the data sheet a rung says what it could not read, and the rest
/// of the ladder still comes back.
#[test]
fn a_rung_without_its_sheet_row_says_why() {
    let mut k = crate::Kernel::new();
    let r = send(
        &mut k,
        json!({"op": "load_stack", "packs": [pack("packs/base-time/pack.json")]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = send(&mut k, json!({"op": "pace_ladder"}));
    let rungs = r["ladder"]["rungs"].as_array().unwrap();
    let sun = rungs.iter().find(|g| g["id"] == "star.life").unwrap();
    assert!(sun["tau_s"].is_null());
    assert!(
        sun["why"].as_str().unwrap().contains("sheet/M_sun"),
        "{sun}"
    );
    let march = rungs.iter().find(|g| g["id"] == "system.ageing").unwrap();
    assert!(march["why"]
        .as_str()
        .unwrap()
        .contains("its driver star.life"));
}

/// A pack replaces a declaration by id and adds its own; a declaration of
/// the wrong form is refused by name, and the rest still come back.
#[test]
fn packs_add_replace_and_are_refused_by_form() {
    let mut k = crate::Kernel::new();
    let mut top = pack("packs/base-time/pack.json");
    top["id"] = json!("test.pace-top");
    top["provides"]["other"] = json!({"processes": {"list": [
        {"id": "star.activity", "says": "Flares, as this setting has them", "shape": "stream",
         "anchor": "system.birth_s", "tau": {"marks": "the setting's own flare tables"},
         "ops": ["read_activity"], "how": "replaced"},
        {"id": "toy.fourth", "says": "A fourth shape", "shape": "wave", "anchor": "system.birth_s",
         "tau": {"marks": "x"}, "ops": ["read_activity"], "how": "x"},
        {"id": "toy.march", "says": "A march with no grid", "shape": "march", "anchor": "system.birth_s",
         "tau": {"driver": "star.life"}, "ops": ["look_ahead"], "how": "x"},
        {"id": "toy.anchor", "says": "Its own zero", "shape": "line", "anchor": "my_epoch",
         "tau": {"marks": "x"}, "ops": ["look_ahead"], "how": "x"},
        {"id": "toy.driver", "says": "Driven by nothing", "shape": "line", "anchor": "system.birth_s",
         "tau": {"driver": "nobody"}, "ops": ["look_ahead"], "how": "x"},
        {"id": "toy.op", "says": "An op that is not", "shape": "line", "anchor": "system.birth_s",
         "tau": {"marks": "x"}, "ops": ["tide_at"], "how": "x"},
        {"id": "toy.typed", "says": "A typed number", "shape": "line", "anchor": "system.birth_s",
         "tau": {"seconds": 86400}, "ops": ["look_ahead"], "how": "x"}
    ]}});
    let r = send(
        &mut k,
        json!({"op": "load_stack", "packs": [pack("packs/base-reference/pack.json"), pack("packs/base-time/pack.json"), top]}),
    );
    assert_eq!(r["type"], "stack_loaded", "{r}");
    let r = send(&mut k, json!({"op": "pace_ladder"}));
    let l = &r["ladder"];
    let refused: Vec<(&str, &str)> = l["refused"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| (x["id"].as_str().unwrap(), x["why"].as_str().unwrap()))
        .collect();
    let why = |id: &str| {
        refused
            .iter()
            .find(|(i, _)| *i == id)
            .unwrap_or_else(|| panic!("{id} not refused: {l}"))
            .1
    };
    assert!(why("toy.fourth").contains("no fourth"));
    assert!(why("toy.march").contains("grid rule"));
    assert!(why("toy.anchor").contains("not a named anchor"));
    assert!(why("toy.driver").contains("not another declared process"));
    assert!(why("toy.op").contains("does not have"));
    assert!(why("toy.typed").contains("unknown field"));
    assert_eq!(refused.len(), 6, "{l}");
    let rungs = l["rungs"].as_array().unwrap();
    let flares: Vec<&Value> = rungs
        .iter()
        .filter(|g| g["id"] == "star.activity")
        .collect();
    assert_eq!(flares.len(), 1);
    assert_eq!(flares[0]["pack"], "test.pace-top");
    assert!(flares[0]["tau_s"].is_null());
    assert!(rungs.iter().any(|g| g["id"] == "orbit.kepler"));
}
