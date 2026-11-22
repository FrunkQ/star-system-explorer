//! The `weather` op (weather track, W5): one op for every reader of the
//! rule-based weather block. `want` picks what comes back.

use crate::err;
use crate::protocol::Response;
use sse_surface::atlas::TileKey;
use sse_weather::{Conditions, Date, Pin, WeatherBlock};

type Out = Result<Response, Response>;

fn to_json<T: serde::Serialize>(t: &T) -> serde_json::Value {
    serde_json::to_value(t).expect("plain data")
}

pub(crate) struct Ask {
    pub want: String,
    pub world: Option<String>,
    pub conditions: Option<serde_json::Value>,
    pub day: f64,
    pub tile: Option<[u32; 4]>,
    pub n: Option<u32>,
    pub lat: f64,
    pub lon: f64,
    pub pins: Vec<serde_json::Value>,
    pub sources: Option<serde_json::Value>,
    pub block: Option<serde_json::Value>,
    /// The epoch the climate is kept under (a body's age, Myr; 0 otherwise).
    pub epoch_myr: f64,
}

pub(crate) fn weather(a: Ask, climates: &mut sse_climate::Cache) -> Out {
    let sources: sse_weather::aerosols::Sources = match a.sources {
        Some(v) => serde_json::from_value(v)
            .map_err(|e| err("bad_sources", format!("not weather sources: {e}")))?,
        None => Default::default(),
    };
    let block = sse_weather::scenario::block().with_sources(sources);
    let other: Option<Box<dyn WeatherBlock>> = match a.block {
        None => None,
        Some(v) => {
            let id: sse_weather::BlockId = serde_json::from_value(v)
                .map_err(|e| err("bad_block", format!("not a weather block id: {e}")))?;
            if id == block.id() {
                None
            } else {
                let stack = [sse_weather::scenario::base_weather_pack()];
                Some(
                    sse_weather::swap::block_for(&stack, &id)
                        .map_err(|e| err("unknown_block", e))?,
                )
            }
        }
    };
    let c: Conditions = match (a.conditions, a.world.as_deref()) {
        (Some(v), _) => serde_json::from_value(v)
            .map_err(|e| err("bad_conditions", format!("not weather conditions: {e}")))?,
        (None, Some(w)) => sse_weather::scenario::fixtures()
            .into_iter()
            .find(|f| f.conditions.label.eq_ignore_ascii_case(w))
            .map(|f| f.conditions)
            .ok_or_else(|| {
                let names: Vec<String> = sse_weather::scenario::fixtures()
                    .into_iter()
                    .map(|f| f.conditions.label)
                    .collect();
                err(
                    "no_weather_world",
                    format!("no weather fixture {w}; try one of: {}", names.join(", ")),
                )
            })?,
        (None, None) => {
            return Err(err(
                "no_weather_world",
                "give `world` (a fixture label), `conditions`, or a `system` and `body`",
            ))
        }
    };
    let pins: Vec<Pin> = a
        .pins
        .into_iter()
        .map(serde_json::from_value)
        .collect::<Result<_, _>>()
        .map_err(|e| err("bad_pins", format!("not weather pins: {e}")))?;
    let date = Date { day: a.day };
    let chosen: &dyn WeatherBlock = match &other {
        Some(b) => b.as_ref(),
        None => &block,
    };
    let rule_only = |what: &str| -> Result<(), Response> {
        if other.is_some() {
            Err(err(
                "not_provided",
                format!("the chosen weather block does not provide `{what}`; the rule block does"),
            ))
        } else {
            Ok(())
        }
    };
    let data = match a.want.as_str() {
        "climate" => {
            // The one climate (W7a), kept by key so a re-read reuses it.
            let core = climates.orbit(&block.settings.climate, &c, a.epoch_myr);
            let averages = match &other {
                Some(b) => b.climate(&c),
                None => sse_weather::circulation::averages(&block.settings, &c, core),
            };
            serde_json::json!({
                "climate": to_json(&averages),
                "circulation": to_json(&core.circulation),
                "core": to_json(core),
            })
        }
        "systems" => to_json(&chosen.systems(&c, date, &pins)),
        "field" => {
            let [face, level, x, y] = a.tile.unwrap_or([0, 0, 0, 0]);
            let key = TileKey::new(face as u8, level.min(30) as u8, x, y).ok_or_else(|| {
                err(
                    "bad_tile",
                    "tile is [face 0-5, level, x, y] inside the face",
                )
            })?;
            let n = a.n.unwrap_or(16).clamp(1, 64);
            to_json(&chosen.field(&c, date, key, n, &pins))
        }
        "report" => {
            rule_only("report")?;
            to_json(&block.report(&c, date, a.lat, a.lon, &pins, None))
        }
        "column" => {
            rule_only("column")?;
            to_json(&sse_weather::hooks::column(
                &block, &c, date, a.lat, a.lon, &pins, None,
            ))
        }
        "sound" => {
            rule_only("sound")?;
            to_json(&sse_weather::hooks::sound(
                &block, &c, date, a.lat, a.lon, &pins, None,
            ))
        }
        other => {
            return Err(err(
                "bad_want",
                format!("weather want {other}: climate, systems, field, report, column or sound"),
            ))
        }
    };
    Ok(Response::WeatherRead {
        want: a.want,
        world: c.label.clone(),
        provenance: to_json(&chosen.provenance()),
        data,
    })
}
