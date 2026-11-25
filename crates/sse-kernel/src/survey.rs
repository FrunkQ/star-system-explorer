//! `survey_surface`: one world's surveyed surface, every stage in order
//! (S2 plates, S3 wear, S4 impacts, S6 regime landforms, S7 cover, deposits,
//! built layer, GM edits and the feature register), with the aerostat band
//! and any tiles asked for. A fixture world needs no universe; a body of an
//! adopted system is read at the system's date like `resolved_system`.

use crate::adopt::{worlds_at, MAX_TILES};
use crate::err;
use crate::protocol::{Response, SurfaceTile};
use sse_chronicle::Chronicle;
use sse_core::{Address, Seed};
use sse_formation::pipeline::SystemRun;
use sse_surface::atlas::TileKey;
use sse_surface::survey::{survey_world, Survey, SurveyOptions};
use sse_surface::{Body, SurveyState};

/// Surveys kept so that more tiles or another listing limit of the same
/// world come back without re-running the stages.
pub const CACHE_SIZE: usize = 4;

/// Recent surveys by key (seed, body, epoch, quick, options), newest last.
pub(crate) type Cache = Vec<(String, Survey)>;

/// The seed fixture worlds are surveyed under (no universe needed).
pub const FIXTURE_SEED: &str = "sse.survey-fixture";

/// Where the world comes from.
pub(crate) enum From<'a> {
    Fixture(&'a str),
    System {
        ch: &'a Chronicle,
        seed: &'a Seed,
        system: &'a Address,
        body: &'a str,
        at_s: Option<i64>,
        run: Option<&'a SystemRun>,
    },
}

pub(crate) struct Request<'a> {
    pub from: From<'a>,
    pub options: Option<&'a serde_json::Value>,
    pub tiles: &'a [SurfaceTile],
    pub quick: bool,
    /// Most features and deposits listed (0: all).
    pub limit: usize,
    /// The master data sheet, for what the deposits hold (MP8).
    pub reference: &'a sse_reference::Reference,
}

/// A world surveyed (or found in the cache): where it came from, the body,
/// the time read, and the survey.
/// A survey with what it was read from: the source, the body, the time, the
/// survey, and the moons raising tides and steering the spin axis.
pub(crate) type Surveyed<'c> = (
    String,
    Body,
    Option<i64>,
    &'c Survey,
    Vec<sse_climate::forcing::MoonPull>,
);

pub(crate) fn surveyed<'c>(
    from: From,
    options: Option<&serde_json::Value>,
    quick: bool,
    cache: &'c mut Cache,
) -> Result<Surveyed<'c>, Response> {
    let w = resolve(from, options, quick)?;
    let (source, body, seed, epoch_gyr, at_s, opts, s) = (
        w.source,
        w.body,
        w.seed,
        w.epoch_gyr,
        w.at_s,
        w.opts,
        w.settings,
    );
    let key = serde_json::to_string(&(&seed, &body, epoch_gyr, quick, &opts)).expect("plain data");
    let moons = opts.moons.clone();
    let sv = match cache.iter().position(|(k, _)| *k == key) {
        Some(i) => {
            let hit = cache.remove(i);
            cache.push(hit);
            &cache[cache.len() - 1].1
        }
        None => {
            let sv = survey_world(&s, &body, &seed, epoch_gyr, &opts)
                .map_err(|e| err("survey_refused", e))?;
            if cache.len() >= CACHE_SIZE {
                cache.remove(0);
            }
            cache.push((key, sv));
            &cache[cache.len() - 1].1
        }
    };
    Ok((source, body, at_s, sv, moons))
}

/// A world to read: where it came from, the body, its seed and epoch, the
/// time read, the survey options and the settings (Orbit level if quick).
pub(crate) struct Resolved {
    pub source: String,
    pub body: Body,
    pub seed: Seed,
    pub epoch_gyr: f64,
    pub at_s: Option<i64>,
    pub opts: SurveyOptions,
    pub settings: sse_surface::SurfaceSettings,
    /// SF1: the orbit track over the window that ends at the epoch (the
    /// survey's own track starts there), for dating past ice ages.
    pub track_before: Option<sse_climate::forcing::OrbitTrack>,
}

pub(crate) fn resolve(
    from: From,
    options: Option<&serde_json::Value>,
    quick: bool,
) -> Result<Resolved, Response> {
    let mut opts: SurveyOptions = match options {
        Some(v) => serde_json::from_value(v.clone())
            .map_err(|e| err("bad_request", format!("options: {e}")))?,
        None => SurveyOptions::default(),
    };
    let mut s = sse_surface::scenario::settings();
    if quick {
        // Quick is the Orbit level (S16): the coarse run a full survey refines.
        s = sse_surface::detail::orbit_settings(&s);
        s.atlas.tile_samples = 32;
    }
    let mut track_before = None;
    let (source, body, seed, epoch_gyr, at_s): (String, Body, Seed, f64, Option<i64>) = match from {
        From::Fixture(name) => {
            let all = sse_surface::scenario::fixtures();
            let names: Vec<String> = all.iter().map(|f| f.body.label.clone()).collect();
            let b = all
                .into_iter()
                .map(|f| f.body)
                .find(|b| b.label.eq_ignore_ascii_case(name))
                .ok_or_else(|| {
                    err(
                        "unknown_fixture",
                        format!("no surface fixture {name}; fixtures: {}", names.join(", ")),
                    )
                })?;
            let age = b.age_gyr;
            (
                format!("fixture:{}", b.label),
                b,
                Seed::from_text(FIXTURE_SEED),
                age,
                None,
            )
        }
        From::System {
            ch,
            seed,
            system,
            body,
            at_s,
            run,
        } => {
            let w = worlds_at(ch, system, at_s, run, &s)?;
            let b = w
                .bodies
                .iter()
                .find(|b| b.address.to_string() == body)
                .cloned()
                .ok_or_else(|| {
                    err(
                        "no_surface_body",
                        format!("{body} is not a surface body of {system}"),
                    )
                })?;
            let age = (b.age_gyr + w.shift_gyr).max(0.0);
            // SOL5: the record's measured air, unless the caller gave a mix.
            if opts.gases.is_none() {
                opts.gases = crate::adopt::catalogued_gases(w.facts.get(&b.address));
            }
            if opts.star_distance_au.is_none() {
                opts.star_distance_au = crate::adopt::star_distance_au(&w.bodies, &b);
            }
            // W9 part 2: the orbit's wobble from its siblings (T1), unless
            // the caller gave one.
            if opts.orbit_track.is_none() {
                let (track, moons) = crate::ice_ages::orbit_inputs(&s.climate, seed, &w.bodies, &b);
                opts.orbit_track = track;
                if opts.moons.is_empty() {
                    opts.moons = moons;
                }
            }
            track_before = crate::ice_ages::orbit_inputs_from(
                &s.climate,
                seed,
                &w.bodies,
                &b,
                -s.climate.forcing.window_yr,
            )
            .0;
            (
                w.source.to_string(),
                b,
                *seed,
                age,
                (w.at_s != i64::MAX).then_some(w.at_s),
            )
        }
    };
    Ok(Resolved {
        source,
        body,
        seed,
        epoch_gyr,
        at_s,
        opts,
        settings: s,
        track_before,
    })
}

pub(crate) fn survey_surface(r: Request, cache: &mut Cache) -> Result<serde_json::Value, Response> {
    if r.tiles.len() > MAX_TILES {
        return Err(err(
            "too_many_tiles",
            format!("at most {MAX_TILES} tiles per request"),
        ));
    }
    let run = match &r.from {
        From::System { run, .. } => *run,
        From::Fixture(_) => None,
    };
    let (source, body, at_s, sv, moons) = surveyed(r.from, r.options, r.quick, cache)?;
    let mut tiles = vec![];
    let mut refused = vec![];
    for t in r.tiles {
        let why = match TileKey::new(t.face, t.level, t.x, t.y) {
            None => Some("bad tile key".to_string()),
            Some(key) => match sv.atlas.tile(key, SurveyState::Surveyed) {
                Ok(tile) => {
                    tiles.push(serde_json::to_value(&tile).expect("plain data"));
                    None
                }
                Err(e) => Some(format!("{e:?}")),
            },
        };
        if let Some(why) = why {
            refused.push(serde_json::json!({
                "face": t.face, "level": t.level, "x": t.x, "y": t.y, "why": why,
            }));
        }
    }
    let ly = sv.atlas.layers();
    let (aerostat, aerostat_refused) = match &sv.aerostat {
        Ok(b) => (Some(v(b)), None),
        Err(e) => (None, Some(e.clone())),
    };
    let mut out = serde_json::json!({
        "body": body.address.to_string(),
        "label": body.label,
        "source": source,
        "version": sse_surface::survey::SURVEY_VERSION,
        "epoch_myr": sv.atlas.epoch_myr,
        "stages": sv.stages,
        "envelope": v(&sv.envelope),
        "deposit_kinds": sse_surface::cover::DEPOSITS,
        "tubes": v(&sv.tubes),
        // SW1: how its bare ground weathers (tiles carry each sample's
        // maturity); absent where nothing from space reaches the ground.
        "weathering": v(&sv.weathering.as_ref().filter(|w| w.active)),
        "tiles": tiles,
        "refused": refused,
    });
    let o = out.as_object_mut().expect("object");
    if let Some(a) = at_s {
        o.insert("at_s".into(), a.into());
    }
    if let Some(a) = aerostat {
        o.insert("aerostat".into(), a);
    }
    if let Some(a) = aerostat_refused {
        o.insert("aerostat_refused".into(), a.into());
    }
    if let Some(l) = ly {
        // The one climate, summarised (the full climate comes from the
        // `weather` op's `climate` want): its key, cells and why.
        let k = &l.climate.world.climate;
        // W9: what each band receives at the datum, and its area label.
        let cs = &sse_surface::scenario::settings().climate;
        let ss = &cs.saltation;
        let places: Vec<serde_json::Value> = k
            .lat_deg
            .iter()
            .map(|&lat| {
                let w = l.climate.world.water_at(cs, lat, 0.0);
                let sa = sse_climate::saltation::at(ss, k, lat, sse_climate::saltation::grain_for(ss, w.t_k));
                let (area, because) = sse_climate::water::area(cs, &w);
                let ia = l
                    .climate
                    .world
                    .ice_ages
                    .as_ref()
                    .map(|m| m.at(cs, k, l.climate.world.lapse_k_km, lat, 0.0));
                serde_json::json!({
                    "lat_deg": lat,
                    "area": area,
                    "label": cs.areas.labels.get(area).cloned().unwrap_or_default(),
                    "because": because,
                    "rain_m_yr": w.rain_m_yr,
                    "snow_m_yr": w.snow_m_yr,
                    "runoff_m_yr": w.runoff_m_yr,
                    "ice_balance_m_yr": w.ice_balance_m_yr,
                    "sand_moving": sa.transport_share,
                    "dunes": sse_climate::saltation::dune_activity(ss, sa.transport_share, w.aridity),
                    "ice_share": ia.as_ref().map(|x| x.glaciated_share),
                    "glacial_advances_per_myr": ia.as_ref().map(|x| x.advances_per_myr),
                })
            })
            .collect();
        o.insert(
            "climate".into(),
            serde_json::json!({
                "key": k.key,
                "regime": k.circulation.regime,
                "cells": k.circulation.cells,
                "hadley_edge_deg": k.circulation.hadley_edge_deg,
                "lat_deg": k.lat_deg,
                "t_k": k.t_k,
                "precip_m_yr": k.precip_m_yr,
                "lapse_k_km": l.climate.world.lapse_k_km,
                "liquid": l.climate.liquid,
                "water": l.climate.water,
                "pressure_bar": l.climate.pressure_bar,
                "assumed": l.climate.world.assumed,
                "why": l.climate.why,
                "places": places,
                "ice_ages": l.climate.world.ice_ages.as_ref().map(|m| {
                    let (lo, hi) = m.obliquity_range();
                    serde_json::json!({
                        "source": m.source,
                        "obliquity_min_deg": lo,
                        "obliquity_max_deg": hi,
                        "window_yr": m.spin.last().map(|x| x.t_yr).unwrap_or(0.0),
                    })
                }),
                // Tides and wind waves, from the formula blocks.
                "sea": crate::sea::summary(r.reference, cs, &body, k, &moons),
                // W8: the main gas's ice moving between ground and air.
                "volatile": k.exchange.as_ref().map(|e| {
                    serde_json::json!({
                        "gas": e.gas,
                        "name": e.name,
                        "p_mean_pa": e.p_mean_pa,
                        "p_min_pa": e.p_min_pa,
                        "p_max_pa": e.p_max_pa,
                        "p_swing": e.p_swing,
                        "p_pa": e.seasons.iter().map(|x| x.p_pa).collect::<Vec<_>>(),
                        "frost_k": e.seasons.iter().map(|x| x.frost_k).collect::<Vec<_>>(),
                        "ice_kg_m2": e.ice_mean_kg_m2,
                        "perennial": e.perennial,
                        "buffered": e.buffered,
                    })
                }),
            }),
        );
        o.insert("air".into(), v(&l.air));
        let total = l.deposits.len();
        let mut order: Vec<usize> = (0..total).collect();
        order.sort_by(|&a, &b| {
            l.deposits[b]
                .radius_km
                .total_cmp(&l.deposits[a].radius_km)
                .then(a.cmp(&b))
        });
        let kept = keep(&order, r.limit);
        let d: Vec<_> = kept.iter().map(|&i| &l.deposits[i]).collect();
        o.insert("deposits".into(), v(&d));
        o.insert("deposits_total".into(), total.into());
        // MP8: what the deposits hold, drawn from the crust.
        let mut h = crate::composition::holdings(&body, run, l, r.reference);
        let kept_ids: Vec<&str> = d.iter().map(|x| x.id.as_str()).collect();
        h.places.retain(|p| kept_ids.contains(&p.id.as_str()));
        o.insert("holdings".into(), v(&h));
        o.insert("built".into(), v(&l.built));
        o.insert("built_kinds".into(), v(&l.built_kinds));
        o.insert("applied".into(), v(&l.applied));
    }
    if let Some(reg) = &sv.register {
        // The remarkable first, then the named landscape, then craters by size.
        let f = &reg.features;
        let rank = |i: usize| -> (u8, f64) {
            let x = &f[i];
            let tier = if !x.remarkable.is_empty() {
                0
            } else if x.kind != "crater" {
                1
            } else {
                2
            };
            (tier, -x.extent_km)
        };
        let mut order: Vec<usize> = (0..f.len()).collect();
        order.sort_by(|&a, &b| {
            let (ta, ea) = rank(a);
            let (tb, eb) = rank(b);
            ta.cmp(&tb).then(ea.total_cmp(&eb)).then(a.cmp(&b))
        });
        let kept = keep(&order, r.limit);
        let mut reg_json = v(reg);
        reg_json["features"] = v(&kept.iter().map(|&i| &f[i]).collect::<Vec<_>>());
        reg_json["features_total"] = f.len().into();
        o.insert("register".into(), reg_json);
    }
    Ok(out)
}

/// The first `limit` of `order` (all when 0), back in their original order.
fn keep(order: &[usize], limit: usize) -> Vec<usize> {
    let n = if limit == 0 {
        order.len()
    } else {
        limit.min(order.len())
    };
    let mut k = order[..n].to_vec();
    k.sort();
    k
}

fn v<T: serde::Serialize>(x: &T) -> serde_json::Value {
    serde_json::to_value(x).expect("plain data")
}
