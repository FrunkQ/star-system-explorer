//! `formation_record` (the Formation screen, formation design section 7, package F10): how a system in the open
//! universe was made, read back. A grown system keeps its formation request on its `adopt.formation` event; this op
//! re-runs that request through the same pure pipeline as `run_formation` and hands back the run, with the request
//! and the manifest hash the adoption recorded, so the screen can say when a newer pipeline would grow it
//! differently. An imported or authored system has no formation record, and the answer says so and why.
//!
//! Read-only: nothing is written, and nothing stored on a system is changed.

use crate::adopt::{ADOPT_FORMATION, ADOPT_V3};
use crate::err;
use crate::protocol::{FormationRequest, Response};
use crate::systems;
use sse_chronicle::{Chronicle, UniverseTime};
use sse_core::Address;

pub(crate) fn formation_record(
    ch: &Chronicle,
    root: &Address,
    system: &str,
) -> Result<Response, Response> {
    let sys: Address = system
        .parse()
        .map_err(|e| err("bad_address", format!("{e}")))?;
    if !root.contains(&sys) || sys == *root {
        return Err(err(
            "bad_address",
            format!("{sys} is not a system in universe {root}"),
        ));
    }
    let adopted = ch.events().iter().rev().find(|e| {
        (e.body.kind == ADOPT_FORMATION || e.body.kind == ADOPT_V3) && e.body.subject.contains(&sys)
    });
    let none = |source: &str, why: &str| Response::FormationRecord {
        system: sys.to_string(),
        source: source.to_string(),
        request: None,
        adopted_manifest: None,
        record: None,
        why: Some(why.to_string()),
    };
    let Some(e) = adopted else {
        let st = ch.state_at(UniverseTime::seconds(i64::MAX));
        if !st.facts.keys().any(|a| sys.contains(a)) {
            return Err(err(
                "unknown_system",
                format!("{sys} holds nothing in this universe"),
            ));
        }
        return Ok(none(
            "facts",
            "this system was written fact by fact, not grown, so it has no formation history",
        ));
    };
    if e.body.kind == ADOPT_V3 {
        return Ok(none(
            "v3",
            "this system came from an SSE 3 save, which carries no formation history",
        ));
    }
    let data = e
        .body
        .payload
        .data
        .as_ref()
        .and_then(|d| serde_json::to_value(d).ok())
        .unwrap_or(serde_json::Value::Null);
    let req: FormationRequest = serde_json::from_value(data["request"].clone()).map_err(|e| {
        err(
            "bad_record",
            format!("the adoption's formation request: {e}"),
        )
    })?;
    let adopted_manifest = data["manifest"].as_str().map(str::to_string);
    let ran = systems::run_formation(req.clone().into())?;
    let record = serde_json::to_value(&ran).expect("plain data");
    Ok(Response::FormationRecord {
        system: sys.to_string(),
        source: "formation".to_string(),
        request: Some(serde_json::to_value(&req).expect("plain data")),
        adopted_manifest,
        record: Some(record),
        why: None,
    })
}
