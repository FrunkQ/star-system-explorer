//! The `weather` op (weather track, W5): one op for every reader of the
//! rule-based weather block. `want` picks what comes back.

use crate::err;
use crate::protocol::Response;
use sse_surface::atlas::TileKey;
use sse_weather::{Conditions, Date, Pin, WeatherBlock};

type Out = Result<Response, Response>;

/// What `want` may be, as the refusal names them.
const WANTS: &str = "climate, systems, field, patch, glint, report, column or sound";

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
    pub half_km: Option<f64>,
    pub span_days: f64,
    pub pins: Vec<serde_json::Value>,
    pub sources: Option<serde_json::Value>,
    pub block: Option<serde_json::Value>,
    /// The epoch the climate is kept under (a body's age, Myr; 0 otherwise).
    pub epoch_myr: f64,
    /// The body and its moons, when asked by system and body (tides).
    pub raisers: Option<(sse_surface::Body, Vec<sse_climate::forcing::MoonPull>)>,
    /// The data sheet (the sea's optics).
    pub sheet: std::sync::Arc<sse_reference::Reference>,
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
            let mut out = serde_json::json!({
                "climate": to_json(&averages),
                "circulation": to_json(&core.circulation),
                "core": to_json(core),
            });
            // A giant's jets by latitude, for the curls and drift (K-a).
            if core.circulation.regime == "banded-giant" {
                let gs = &block.giants;
                let l = sse_weather::giants::layout(gs, &c);
                out["jets"] = to_json(&sse_weather::giants::jet_profile(gs, &c, &l));
            }
            out
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
        "flow" => {
            rule_only("flow")?;
            let t = block.flow(&c, date, a.span_days).ok_or_else(|| {
                err(
                    "not_a_giant",
                    "the flow table is a giant's: its jets, storms and eddies",
                )
            })?;
            to_json(&t)
        }
        "carry" => {
            rule_only("carry")?;
            let t = block.carry(&c, date).ok_or_else(|| {
                err(
                    "not_a_giant",
                    "a carry table is a giant's whose cloud is renewed: its jets, storms and eddies over two renewals",
                )
            })?;
            to_json(&t)
        }
        "stirred" => {
            rule_only("stirred")?;
            let half = a.half_km.filter(|h| *h > 0.0).ok_or_else(|| {
                err(
                    "bad_patch",
                    "a stirred patch needs `half_km` > 0 round `lat`, `lon`",
                )
            })?;
            // Each cell follows its cloud back up to two renewals through every eddy (seconds a cell on
            // Jupiter): a reference for a small patch, kept to 16 across.
            let n = a.n.unwrap_or(8).clamp(1, 16);
            let t = block
                .stirred(&c, date, a.lat, a.lon, half, n)
                .ok_or_else(|| {
                    err(
                        "not_a_giant",
                        "stirred cloud is a giant's: carried by its jets, storms and eddies",
                    )
                })?;
            to_json(&t)
        }
        "patch" => {
            rule_only("patch")?;
            let half = a.half_km.filter(|h| *h > 0.0).ok_or_else(|| {
                err(
                    "bad_patch",
                    "a patch needs `half_km` > 0 round `lat`, `lon`",
                )
            })?;
            let n = a.n.unwrap_or(16).clamp(1, 64);
            to_json(&block.patch(&c, date, a.lat, a.lon, half, n, &pins))
        }
        "report" => {
            rule_only("report")?;
            let r = block.report(&c, date, a.lat, a.lon, &pins, None);
            let mut out = to_json(&r);
            // The sea at the site: waves under this wind, and the tides.
            let cs = &block.settings.climate;
            let core = climates.orbit(cs, &c, a.epoch_myr);
            let body = a.raisers.as_ref().map(|(b, m)| (b, m.as_slice()));
            if let Some(sea) = crate::sea::site(
                &a.sheet,
                cs,
                core,
                a.lat,
                r.wind_km_h,
                r.wind_from_deg,
                body,
            ) {
                out["sea"] = sea;
            }
            out
        }
        "glint" => {
            // The sun's glint off the seas, for the view's drawing (EC2).
            let cs = &block.settings.climate;
            let core = climates.orbit(cs, &c, a.epoch_myr);
            let g = crate::sea::glint(&a.sheet, cs, core);
            g.unwrap_or(serde_json::Value::Null)
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
        other => return Err(err("bad_want", format!("weather want {other}: {WANTS}"))),
    };
    Ok(Response::WeatherRead {
        want: a.want,
        world: c.label.clone(),
        provenance: to_json(&chosen.provenance()),
        data,
    })
}
