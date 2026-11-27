//! Every shipped pack hashes the same natively and as a browser sends it.
//!
//! A browser holds every JSON number as a double and writes it back with
//! `JSON.stringify`: shortest round-trip digits, plain from 1e-7 up to 1e21
//! (so `4.560563969097498e17` goes out as `456056396909749800` and `281.0`
//! as `281`), exponent form outside that. The kernel must read both
//! spellings as one number, or the web and native stack hashes part.

use serde_json::Value;

/// A number as JavaScript's `Number.prototype.toString` writes it. Rust's
/// `Display` for f64 is the same shortest round-trip digits, always plain;
/// outside JavaScript's plain range the exponent form is used instead (its
/// spelling differs, `1e21` for `1e+21`, but it reads back alike).
fn js_number(x: f64) -> String {
    let a = x.abs();
    if x == 0.0 {
        "0".into()
    } else if (1e-7..1e21).contains(&a) {
        format!("{x}")
    } else {
        format!("{x:e}")
    }
}

/// The value a browser hands the kernel: parsed by `JSON.parse` (every
/// number a double) and written back by `JSON.stringify`.
fn as_browser_sends(v: &Value) -> Value {
    match v {
        Value::Number(n) => serde_json::from_str(&js_number(n.as_f64().expect("a finite number")))
            .expect("a JSON number"),
        Value::Array(a) => Value::Array(a.iter().map(as_browser_sends).collect()),
        Value::Object(m) => Value::Object(
            m.iter()
                .map(|(k, v)| (k.clone(), as_browser_sends(v)))
                .collect(),
        ),
        x => x.clone(),
    }
}

fn shipped() -> Vec<(String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../packs");
    let mut out = Vec::new();
    let mut dirs: Vec<_> = std::fs::read_dir(&root)
        .expect("packs/")
        .flatten()
        .map(|d| d.path())
        .collect();
    dirs.sort();
    for d in dirs {
        let f = d.join("pack.json");
        if let Ok(t) = std::fs::read_to_string(&f) {
            out.push((d.file_name().unwrap().to_string_lossy().into_owned(), t));
        }
    }
    out
}

#[test]
fn the_browser_spelling_of_a_number_is_one_number() {
    for (text, browser) in [
        ("4.560563969097498e17", "456056396909749800"),
        ("281.0", "281"),
        ("2.81e2", "281"),
        ("1e10", "10000000000"),
        ("-0.0", "0"),
        ("3.0856775814913674e16", "30856775814913670"),
        ("1e21", "1e21"),
        ("435494880000000000", "435494880000000000"),
    ] {
        let read = |t: &str| {
            let mut v: Value = serde_json::from_str(t).unwrap();
            sse_pack::json_numbers(&mut v);
            v
        };
        let (a, b) = (read(text), read(browser));
        assert_eq!(a, b, "{text} and {browser} are one number");
        assert_eq!(
            serde_json::from_str::<Value>(&js_number(a.as_f64().unwrap()))
                .unwrap()
                .as_f64(),
            a.as_f64()
        );
    }
    assert_eq!(js_number(4.560563969097498e17), "456056396909749800");
    assert_eq!(js_number(281.0), "281");
}

#[test]
fn every_shipped_pack_hashes_the_same_in_a_browser() {
    let mut differ = Vec::new();
    let packs = shipped();
    assert!(packs.len() > 20, "the shipped packs are found");
    for (name, text) in &packs {
        let native =
            sse_pack::PackManifest::from_json(text).unwrap_or_else(|e| panic!("{name}: {e}"));
        let v: Value = serde_json::from_str(text).unwrap();
        let web = sse_pack::PackManifest::from_value(&as_browser_sends(&v))
            .unwrap_or_else(|e| panic!("{name} (browser): {e}"));
        if native.content_hash() != web.content_hash() {
            differ.push(name.clone());
        }
    }
    assert!(
        differ.is_empty(),
        "these packs hash differently as a browser sends them: {differ:?}"
    );
}
