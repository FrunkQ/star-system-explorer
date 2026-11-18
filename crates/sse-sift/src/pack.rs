//! Rules, atlases and sifters as pack content (`provides.other.rules`,
//! `.classes`, `.axes`, `.sifters`), stacked bottom to top.

use crate::classify::{Atlas, Axis, Class};
use crate::rules::{Rule, RuleSet};
use crate::sifter::Sifter;
use sse_core::cbor::Value;
use sse_pack::PackManifest;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SiftContent {
    pub rules: RuleSet,
    pub atlas: Atlas,
    pub sifters: Vec<Sifter>,
}

fn take<T: serde::de::DeserializeOwned>(p: &PackManifest, key: &str) -> Result<Vec<T>, String> {
    match p.provides.other.get(key) {
        None => Ok(Vec::new()),
        // Through JSON, so integers written in a pack read as numbers where a number is wanted.
        Some(v @ Value::Array(_)) => serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("{}: {key}: {e}", p.id)),
        Some(_) => Err(format!("{}: {key} must be a list", p.id)),
    }
}

/// Merge the sift content of a stack (bottom to top). An id may be defined
/// once across the stack: a second definition is a validation error, as
/// with the one-publisher rule for quantities, never a silent override.
pub fn load(stack: &[PackManifest]) -> Result<SiftContent, Vec<String>> {
    let mut out = SiftContent::default();
    out.atlas.sticky_margin = 0.1;
    let mut errs = Vec::new();
    let mut owner: BTreeMap<String, String> = BTreeMap::new();
    let mut claim = |kind: &str, id: &str, pack: &str, errs: &mut Vec<String>| {
        if let Some(prev) = owner.insert(format!("{kind}:{id}"), pack.to_string()) {
            errs.push(format!("{pack}: {kind} {id} is already defined by {prev}"));
        }
    };
    for p in stack {
        match take::<Rule>(p, "rules") {
            Ok(rs) => rs.into_iter().for_each(|r| {
                claim("rule", &r.id, &p.id, &mut errs);
                out.rules.rules.push(r)
            }),
            Err(e) => errs.push(e),
        }
        match take::<(String, Axis)>(p, "axes") {
            Ok(xs) => xs.into_iter().for_each(|(id, a)| {
                claim("axis", &id, &p.id, &mut errs);
                out.atlas.axes.insert(id, a);
            }),
            Err(e) => errs.push(e),
        }
        match take::<Class>(p, "classes") {
            Ok(cs) => cs.into_iter().for_each(|c| {
                claim("class", &c.id, &p.id, &mut errs);
                out.atlas.classes.push(c)
            }),
            Err(e) => errs.push(e),
        }
        match take::<Sifter>(p, "sifters") {
            Ok(ss) => ss.into_iter().for_each(|s| {
                claim("sifter", &s.id, &p.id, &mut errs);
                out.sifters.push(s)
            }),
            Err(e) => errs.push(e),
        }
    }
    if let Err(e) = out.rules.validate() {
        errs.push(e.0);
    }
    if let Err(e) = out.atlas.validate() {
        errs.push(e.0);
    }
    if errs.is_empty() {
        Ok(out)
    } else {
        Err(errs)
    }
}
