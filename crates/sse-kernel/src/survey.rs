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
}

/// A world surveyed (or found in the cache): where it came from, the body,
/// the time read, and the survey.
pub(crate) fn surveyed<'c>(
    from: From,
    options: Option<&serde_json::Value>,
    quick: bool,
    cache: &'c mut Cache,
) -> Result<(String, Body, Option<i64>, &'c Survey), Response> {
    let opts: SurveyOptions = match options {
        Some(v) => serde_json::from_value(v.clone())
            .map_err(|e| err("bad_request", format!("options: {e}")))?,
        None => SurveyOptions::default(),
    };
    let mut s = sse_surface::scenario::settings();
    if quick {
        s.plates.cells_per_face = 16;
        s.atlas.tile_samples = 32;
    }
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
                .into_iter()
                .find(|b| b.address.to_string() == body)
                .ok_or_else(|| {
                    err(
                        "no_surface_body",
                        format!("{body} is not a surface body of {system}"),
                    )
                })?;
            let age = (b.age_gyr + w.shift_gyr).max(0.0);
            (
                w.source.to_string(),
                b,
                *seed,
                age,
                (w.at_s != i64::MAX).then_some(w.at_s),
            )
        }
    };
    let key = serde_json::to_string(&(&seed, &body, epoch_gyr, quick, &opts)).expect("plain data");
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
    Ok((source, body, at_s, sv))
}

pub(crate) fn survey_surface(r: Request, cache: &mut Cache) -> Result<serde_json::Value, Response> {
    if r.tiles.len() > MAX_TILES {
        return Err(err(
            "too_many_tiles",
            format!("at most {MAX_TILES} tiles per request"),
        ));
    }
    let (source, body, at_s, sv) = surveyed(r.from, r.options, r.quick, cache)?;
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
        o.insert("climate".into(), v(&l.climate));
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
