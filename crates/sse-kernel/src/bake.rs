//! ER11a: the `bake` op (edits-reach design section 11). The kernel bakes
//! tables for the renderer from the physics in force, in the one format
//! `sse.table/1` (`sse_physics::table`), so the renderer keeps only
//! geometry and lookups and an edit to a law changes the picture.
//!
//! Adding a table: write its producer in the crate that owns the law (a
//! function from the subject and size to a `sse_physics::table::Baked`,
//! reading blocks from the physics in force), add one row to [`TABLES`] and
//! one arm to [`produce`], and give it a test that its sampling is within
//! its bound (`sse_physics::table::check_sampling`, run by `Table::seal`).

use crate::err;
use crate::protocol::Response;
use sse_physics::table::{Size, Table};
use std::sync::Arc;

/// The tables the kernel bakes: id, then what it is, for the error that
/// lists them.
pub const TABLES: &[(&str, &str)] = &[(
    sse_light::disc_table::ID,
    "a thin disc round a compact mass: shift and the ring's T⁴ against radius and the ray's angular momentum over radius (subject: mass_kg or gravitational_radius_m, and rate_kg_s, inner_m and outer_m, or read_light_paths' feed)",
)];

/// Tables kept, newest last; the oldest is dropped past this many.
pub const KEPT: usize = 24;

type Out = Result<Response, Response>;

/// What a table is kept by: id, subject (canonical JSON), size, physics hash.
type Key = (String, String, Size, String);

#[derive(Default)]
pub struct Cache {
    kept: Vec<(Key, Arc<Table>)>,
}

#[cfg(test)]
impl Cache {
    pub fn len(&self) -> usize {
        self.kept.len()
    }
}

fn produce(
    id: &str,
    subject: &serde_json::Value,
    size: Size,
    physics: &sse_physics::Physics,
) -> Result<sse_physics::table::Baked, String> {
    match id {
        sse_light::disc_table::ID => {
            let blocks = physics.blocks().clone();
            let get = move |id: &str| blocks.get(id).cloned();
            sse_light::disc_table::bake(subject, size, &get)
        }
        _ => Err(format!(
            "no table {id}; the kernel bakes: {}",
            TABLES
                .iter()
                .map(|(i, what)| format!("{i} ({what})"))
                .collect::<Vec<_>>()
                .join("; ")
        )),
    }
}

/// Bake (or hand back the kept copy of) one table.
pub fn bake(
    cache: &mut Cache,
    physics: &Arc<sse_physics::Physics>,
    id: &str,
    subject: Option<serde_json::Value>,
    tier: Option<String>,
    with_values: bool,
) -> Out {
    let out = |t: &Table, kept: bool, tier: &Option<String>| {
        let mut v = serde_json::to_value(t).expect("plain data");
        v["tier"] = tier.clone().map_or(serde_json::Value::Null, Into::into);
        if !with_values {
            v["values"] = serde_json::Value::Null;
        }
        Response::Baked { table: v, kept }
    };
    // Numbers as doubles, so 281, 281.0 and 2.81e2 are one subject: the same
    // key, and the same echo whichever way the request was spelled (a
    // browser's JSON writes 1476600000.0 as 1476600000).
    let subject = as_doubles(subject.unwrap_or(serde_json::Value::Null));
    let size = Size::of_tier(tier.as_deref());
    let hash = physics.hash().hex();
    // serde_json keeps object keys sorted (no preserve_order), so this is
    // canonical.
    let key: Key = (id.to_string(), subject.to_string(), size, hash.clone());
    if let Some((_, t)) = cache.kept.iter().find(|(k, _)| *k == key) {
        return Ok(out(t, true, &tier));
    }
    let baked = produce(id, &subject, size, physics).map_err(|e| err("bad_table", e))?;
    let table = Table::seal(id, subject, size, tier.clone(), hash, baked)
        .map_err(|e| err("bad_table", e))?;
    let table = Arc::new(table);
    cache.kept.push((key, table.clone()));
    if cache.kept.len() > KEPT {
        cache.kept.remove(0);
    }
    Ok(out(&table, false, &tier))
}

/// Every number in `v` as a double.
fn as_doubles(v: serde_json::Value) -> serde_json::Value {
    use serde_json::Value;
    match v {
        Value::Number(n) => n
            .as_f64()
            .and_then(serde_json::Number::from_f64)
            .map_or(Value::Number(n), Value::Number),
        Value::Array(a) => Value::Array(a.into_iter().map(as_doubles).collect()),
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, v)| (k, as_doubles(v))).collect()),
        other => other,
    }
}
