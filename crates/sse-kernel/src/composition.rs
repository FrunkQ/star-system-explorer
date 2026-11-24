//! `read_composition` (MP6): a body's elements on a click. The worlds come
//! from `worlds_at` (formation's run or the facts, as `resolved_system`
//! builds them, with no surface stage); formation's ledger gives each
//! world's materials where there is a run. The sums are the
//! `sse.composition.*` blocks (`sse_surface::composition`).

use crate::adopt::worlds_at;
use crate::err;
use crate::protocol::Response;
use sse_chronicle::Chronicle;
use sse_core::Address;
use sse_formation::pipeline::SystemRun;
use sse_surface::composition::{compose, Bulk, Level, Settings, Sheet};
use std::collections::BTreeMap;

/// Al-26 heat per kg left after a body formed, from the Al-26 block (the
/// Sun's share of Al-26 until MP7 hands the star's own).
fn al26_left(formed_myr: f64) -> f64 {
    let atoms = sse_atoms::atoms();
    let Some(a) = atoms
        .iter()
        .find(|a| a.decl().id == "sse.heat.al26_planetesimal_heating")
    else {
        return 0.0;
    };
    let p: Vec<f64> = a.decl().params.iter().map(|p| p.default).collect();
    let t = formed_myr.max(0.0) * 1.0e6 * sse_core::sheet::YR_JULIAN;
    a.eval(&[1.0, t], &p).map(|o| o[0]).unwrap_or(0.0)
}

/// A world's bulk: its facts, with formation's ledger and Al-26 where it
/// was formed by a run.
pub(crate) fn bulk_of(b: &sse_surface::Body, run: Option<&SystemRun>) -> Bulk {
    let mut bulk = Bulk::of_body(b);
    if let Some(p) = run.and_then(|r| r.protoplanets.iter().find(|p| p.address == b.address)) {
        bulk = bulk.with_ledger(&p.by_group);
        bulk.al26_j_kg = al26_left(p.formed_myr);
    }
    bulk
}

/// MP7: the star's own elements, from what it inherited; a system with no
/// formation run reads as the Sun's (compose notes the stand-in). Every
/// composition read (read_composition, read_inventory, holdings) uses it.
pub(crate) fn star_ratio(
    run: Option<&SystemRun>,
    sheet: &sse_reference::Reference,
) -> Result<BTreeMap<String, f64>, Response> {
    match run {
        Some(r) => crate::star_elements::star_element_ratios(&r.history.inheritance, sheet)
            .map_err(|e| err("bad_pack", e)),
        None => Ok(BTreeMap::new()),
    }
}

/// MP8: what a surveyed world's deposits hold, from its Survey composition.
pub(crate) fn holdings(
    b: &sse_surface::Body,
    run: Option<&SystemRun>,
    layers: &sse_surface::cover::Layers,
    sheet: &sse_reference::Reference,
) -> sse_surface::composition::Holdings {
    let settings = Settings::default();
    let c = compose(
        &Sheet::read(sheet),
        &bulk_of(b, run),
        &star_ratio(run, sheet).unwrap_or_default(),
        Level::Survey,
        &settings,
    );
    let ores = &sse_surface::scenario::settings().cover.ores;
    sse_surface::composition::holdings(&c, layers, ores, settings.crust_density)
}

pub(crate) fn read(
    ch: &Chronicle,
    system: &Address,
    body: Option<&str>,
    level: Option<&str>,
    at_s: Option<i64>,
    run: Option<&SystemRun>,
    sheet: &sse_reference::Reference,
) -> Result<Response, Response> {
    let level = match level.unwrap_or("orbit") {
        "orbit" => Level::Orbit,
        "survey" => Level::Survey,
        x => {
            return Err(err(
                "bad_level",
                format!("level {x:?}: use \"orbit\" or \"survey\""),
            ))
        }
    };
    let want: Option<Address> = match body {
        Some(b) => Some(b.parse().map_err(|e| err("bad_address", format!("{e}")))?),
        None => None,
    };
    let ss = sse_surface::scenario::settings();
    let w = worlds_at(ch, system, at_s, run, &ss)?;
    let rows = Sheet::read(sheet);
    let settings = Settings::default();
    let ratio = star_ratio(run, sheet)?;
    let mut out = vec![];
    for b in &w.bodies {
        if want.as_ref().is_some_and(|a| *a != b.address) {
            continue;
        }
        let mut c = compose(&rows, &bulk_of(b, run), &ratio, level, &settings);
        if b.deep_envelope {
            c.stand_ins.push(
                "a deep hydrogen and helium envelope: only the solid core is counted here".into(),
            );
        }
        out.push(serde_json::to_value(&c).expect("serialisable"));
    }
    if let (Some(a), true) = (&want, out.is_empty()) {
        return Err(err(
            "unknown_body",
            format!("{a} is not a world of {system} with a mass and a radius"),
        ));
    }
    Ok(Response::CompositionRead { bodies: out })
}
