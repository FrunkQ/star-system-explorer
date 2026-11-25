//! `reference_tiles`: a catalogued Solar System body's surface without a
//! campaign. The size comparison and any other reference view show Mars,
//! Venus or the Moon as their own faces from `packs/base-sol`'s measured
//! values. The Solar System is entered as the start sheet's door enters it,
//! into a private reference universe kept apart from the campaign (never
//! saved, never in its chronicle), and read with `resolved_system`. The
//! reference universe is kept, so its worlds' orbit runs are reused, until
//! the stack is loaded again.

use crate::protocol::{Request, Response};
use crate::{err, Kernel, Universe};
use sse_core::math as m;

const SYSTEM: &str = "u:reference/sys:sol";

/// The base-sol sheet, as built in (the catalogued Solar System is a base
/// pack, the same one `frame.rs` reads the Sun's frame from).
fn sol_sheet() -> serde_json::Value {
    let p: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-sol/pack.json"))
            .expect("base-sol pack");
    p["provides"]["other"]["sol"].clone()
}

/// The sheet's bodies at their addresses (Sun star:0, bodies body:N in
/// sheet order, moons under their host), and the batch of catalogued
/// edits, as `packages/ui-contract/src/sol.ts` builds them.
pub(crate) fn sol_batch(
    sheet: &serde_json::Value,
    system: &str,
) -> (Vec<(String, String)>, Vec<serde_json::Value>) {
    let bodies = sheet["bodies"].as_array().cloned().unwrap_or_default();
    let mut addr: Vec<(String, String)> = vec![];
    let mut count: Vec<(String, usize)> = vec![];
    let mut next = |under: &str, seg: &str| {
        let key = format!("{under}|{seg}");
        let i = match count.iter_mut().find(|(k, _)| *k == key) {
            Some((_, n)) => {
                *n += 1;
                *n - 1
            }
            None => {
                count.push((key, 1));
                0
            }
        };
        format!("{under}/{seg}:{i}")
    };
    let mut left: Vec<&serde_json::Value> = bodies.iter().collect();
    while !left.is_empty() {
        let Some(i) = left.iter().position(|b| match b["host"].as_str() {
            None => true,
            Some(h) => addr.iter().any(|(id, _)| id == h),
        }) else {
            break;
        };
        let b = left.remove(i);
        let id = b["id"].as_str().unwrap_or_default().to_string();
        let a = match b["host"].as_str() {
            None => next(system, "star"),
            Some(h) => {
                let ha = addr
                    .iter()
                    .find(|(x, _)| x == h)
                    .map(|x| x.1.clone())
                    .unwrap_or_default();
                if ha.contains("/star:") {
                    next(system, "body")
                } else {
                    next(&ha, "body")
                }
            }
        };
        addr.push((id, a));
    }
    let at = |id: &str| addr.iter().find(|(x, _)| x == id).map(|x| x.1.clone());
    let mut edits = vec![];
    let mut edit = |a: &str, f: &str, v: serde_json::Value| {
        edits.push(serde_json::json!({"address": a, "field": f, "value": v}));
    };
    for b in &bodies {
        let Some(a) = b["id"].as_str().and_then(at) else {
            continue;
        };
        edit(&a, "name", b["name"].clone());
        edit(&a, "body.kind", b["kind"].clone());
        match b["host"].as_str().and_then(at) {
            Some(h) => edit(&a, "orbit.host", serde_json::Value::String(h)),
            None => edit(
                system,
                "name",
                serde_json::Value::String("Solar System".into()),
            ),
        }
        let Some(values) = b["values"].as_object() else {
            continue;
        };
        for (f, x) in values {
            let unit = x["unit"].as_str().unwrap_or("1");
            let v = &x["v"];
            if f == "system.age_gyr" {
                if let Some(g) = v.as_f64() {
                    edit(
                        system,
                        "system.age",
                        serde_json::json!(g * 1e9 * sse_core::sheet::YR_JULIAN),
                    );
                }
                continue;
            }
            let value = match (unit, v.as_f64()) {
                ("deg", Some(d)) => serde_json::json!(d * m::PI / 180.0),
                _ => v.clone(),
            };
            edit(&a, f, value);
        }
    }
    (addr, edits)
}

impl Kernel {
    /// The reference universe, opened and filled on first use.
    fn reference_universe(&mut self) -> Result<Universe, Response> {
        if let Some(u) = self.reference.take() {
            return Ok(u);
        }
        let campaign = self.universe.take();
        let sheet = sol_sheet();
        let (_, edits) = sol_batch(&sheet, SYSTEM);
        let opened = self.dispatch(Request::OpenUniverse {
            id: "reference".into(),
            seed: "reference".into(),
        });
        let out = match opened {
            Ok(Response::UniverseOpened { .. }) => {
                let batch: Request = serde_json::from_value(serde_json::json!({
                    "op": "set_authored_batch", "occurred_at_s": 0,
                    "evidence": "catalogued", "source": "packs/base-sol", "edits": edits,
                }))
                .map_err(|e| err("bad_reference", e.to_string()))?;
                match self.dispatch(batch) {
                    Ok(Response::Authored { .. }) => self
                        .universe
                        .take()
                        .ok_or_else(|| err("bad_reference", "the reference universe did not open")),
                    Ok(r) | Err(r) => Err(r),
                }
            }
            Ok(r) | Err(r) => Err(r),
        };
        self.universe = campaign;
        out
    }

    /// `reference_tiles`: tiles of one base-sol body, by its sheet id.
    pub(crate) fn reference_tiles(
        &mut self,
        body: &str,
        tiles: &[crate::protocol::ReferenceTile],
    ) -> Result<Response, Response> {
        let sheet = sol_sheet();
        let (addr, _) = sol_batch(&sheet, SYSTEM);
        let Some(address) = addr.iter().find(|(id, _)| id == body).map(|x| x.1.clone()) else {
            return Err(err(
                "no_reference_body",
                format!(
                    "no {body} in base-sol; it has {}",
                    addr.iter()
                        .map(|x| x.0.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        };
        let u = self.reference_universe()?;
        let campaign = self.universe.replace(u);
        let r = self.dispatch(Request::ResolvedSystem {
            system: SYSTEM.into(),
            at_s: None,
            branch: None,
            tiles: Some(
                tiles
                    .iter()
                    .map(|t| crate::protocol::TileRequest {
                        body: address.clone(),
                        face: t.face,
                        level: t.level,
                        x: t.x,
                        y: t.y,
                    })
                    .collect(),
            ),
        });
        self.reference = std::mem::replace(&mut self.universe, campaign);
        match r? {
            Response::SystemResolved {
                tiles,
                refused,
                surfaces,
                ..
            } => Ok(Response::ReferenceTiles {
                body: body.into(),
                regime_label: surfaces
                    .iter()
                    .find(|s| s["address"] == address.as_str())
                    .and_then(|s| s["regime_label"].as_str())
                    .map(str::to_string),
                address,
                tiles,
                refused,
            }),
            other => Err(other),
        }
    }
}
