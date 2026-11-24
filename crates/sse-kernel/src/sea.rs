//! Tides and wind waves for a world's card (the survey's climate summary)
//! and a landing site (the weather op's report). The numbers come from the
//! formula blocks in `packs/base-physics/formulas`: `sse.tides.ocean_range`
//! and `sse.tides.period` for each body raising a tide, and
//! `sse.ocean.wind_waves` from the wind, the fetch and gravity, so the card,
//! the site report and the renderer's drawn waves read one set of figures.

use sse_climate::forcing::MoonPull;
use sse_climate::{Climate, ClimateSettings, Conditions};
use sse_core::math as m;
use sse_core::sheet;
use sse_flow::Atom;

/// Run one formula block with its default settings.
fn run(id: &str, inputs: &[f64]) -> Option<Vec<f64>> {
    let a = sse_atoms::formulas::loaded()
        .iter()
        .find(|a| a.decl().id == id)?;
    let p: Vec<f64> = a.decl().params.iter().map(|p| p.default).collect();
    a.eval(inputs, &p).ok()
}

/// One body raising a tide on the world.
pub(crate) struct Raiser {
    pub name: String,
    pub mass_kg: f64,
    pub distance_m: f64,
    /// Once round the world's sky: its orbit, or the world's year for the
    /// star. None: the world keeps one face to it (a fixed bulge).
    pub period_s: Option<f64>,
}

/// The bodies raising tides on a world: what it orbits, and its moons.
pub(crate) fn raisers(body: &sse_surface::Body, c: &Conditions, moons: &[MoonPull]) -> Vec<Raiser> {
    let mut out = vec![];
    let world_kg = body.mass_earth * sheet::M_EARTH;
    if let Some(o) = &body.orbit {
        if o.host_msun > 0.0 && o.a_au > 0.0 {
            out.push(Raiser {
                name: "host".into(),
                mass_kg: o.host_msun * sheet::M_SUN,
                distance_m: o.a_au * sheet::AU,
                period_s: (!c.star_locked).then_some(c.year_days * sheet::DAY),
            });
        }
    }
    for (i, mo) in moons.iter().enumerate() {
        if mo.mass_kg <= 0.0 || mo.a_m <= 0.0 {
            continue;
        }
        let total = world_kg + mo.mass_kg;
        out.push(Raiser {
            name: format!("moon {}", i + 1),
            mass_kg: mo.mass_kg,
            distance_m: mo.a_m,
            period_s: run("sse.orbit.period", &[mo.a_m, total]).map(|v| v[0]),
        });
    }
    out
}

/// Wind waves at one wind and fetch: (significant height m, period s).
pub(crate) fn waves(wind_m_s: f64, fetch_m: f64, gravity: f64) -> Option<(f64, f64)> {
    let v = run(
        "sse.ocean.wind_waves",
        &[wind_m_s.abs().max(1e-3), fetch_m.max(1.0), gravity],
    )?;
    Some((v[0], v[1]))
}

/// The open water a wind crosses in the band at `lat_deg`: the band's sea
/// between coasts, held to a storm's size.
pub(crate) fn fetch_m(cs: &ClimateSettings, c: &Conditions, cl: &Climate, lat_deg: f64) -> f64 {
    let n = cl.lat_deg.len();
    let land = if cl.land.len() == n {
        cl.lerp(&cl.land, lat_deg)
    } else {
        c.land_fraction()
    };
    let k = c.coasts_by_lat.len();
    let coasts = if k > 0 {
        let i = (((lat_deg + 90.0) / 180.0 * k as f64) as usize).min(k - 1);
        c.coasts_by_lat[i]
    } else {
        cs.land_sea.coasts_per_band
    };
    let round = 2.0 * m::PI * c.radius_m * m::cos(lat_deg * m::PI / 180.0).max(1e-6);
    let open = (1.0 - land).clamp(0.0, 1.0) * round / (0.5 * coasts).max(1.0);
    open.min(cs.sea.storm_fetch_m)
}

fn tide_json(c: &Conditions, body: &sse_surface::Body, rs: &[Raiser]) -> serde_json::Value {
    let world_kg = body.mass_earth * sheet::M_EARTH;
    let spin_s = c.rotation_hours * 3600.0;
    let mut list: Vec<(String, f64, Option<f64>)> = rs
        .iter()
        .filter_map(|r| {
            let h = run(
                "sse.tides.ocean_range",
                &[r.mass_kg, world_kg, c.radius_m, r.distance_m],
            )?[0];
            let t = r
                .period_s
                .and_then(|p| run("sse.tides.period", &[spin_s, p]))
                .map(|v| v[0] / 3600.0);
            Some((r.name.clone(), h, t))
        })
        .collect();
    list.sort_by(|a, b| b.1.total_cmp(&a.1));
    // Spring: every tide together; neap: the largest against the rest.
    let moving: Vec<f64> = list.iter().filter(|x| x.2.is_some()).map(|x| x.1).collect();
    let spring: f64 = moving.iter().sum();
    let neap = match moving.split_first() {
        Some((a, rest)) => (a - rest.iter().sum::<f64>()).max(0.0),
        None => 0.0,
    };
    serde_json::json!({
        "by": list.iter().map(|(n, h, t)| serde_json::json!({
            "raiser": n, "range_m": h, "period_h": t, "fixed_bulge": t.is_none(),
        })).collect::<Vec<_>>(),
        "spring_range_m": spring,
        "neap_range_m": neap,
        "block": "sse.tides.ocean_range",
        "note": "open-sea equilibrium tide; coasts and bays can reach several to tens of times it",
    })
}

/// The sea's tides and waves for a world's card, or None without a sea.
pub(crate) fn summary(
    cs: &ClimateSettings,
    body: &sse_surface::Body,
    cl: &Climate,
    moons: &[MoonPull],
) -> Option<serde_json::Value> {
    let c = &cl.conditions;
    if !c.solid_surface || c.liquid.is_none() || c.ocean_fraction <= 0.0 {
        return None;
    }
    let rs = raisers(body, c, moons);
    let n = cl.lat_deg.len();
    // Waves in each band with sea, from the band's year-round wind.
    let mut bands = vec![];
    let (mut wsum, mut hsum, mut tsum) = (0.0, 0.0, 0.0);
    for j in 0..n {
        let lat = cl.lat_deg[j];
        let sea = if cl.land.len() == n {
            1.0 - cl.land[j]
        } else {
            c.ocean_fraction
        };
        if sea <= 0.0 {
            bands.push(serde_json::Value::Null);
            continue;
        }
        let fetch = fetch_m(cs, c, cl, lat);
        let Some((h, t)) = waves(cl.wind_m_s[j], fetch, c.gravity_m_s2) else {
            bands.push(serde_json::Value::Null);
            continue;
        };
        let w = sea * m::cos(lat * m::PI / 180.0);
        wsum += w;
        hsum += w * h;
        tsum += w * t;
        bands.push(serde_json::json!({ "wave_m": h, "period_s": t, "fetch_km": fetch / 1000.0 }));
    }
    let (rough_j, rough) = bands
        .iter()
        .enumerate()
        .filter_map(|(j, b)| b.get("wave_m").and_then(|x| x.as_f64()).map(|h| (j, h)))
        .fold((0, 0.0), |a, b| if b.1 > a.1 { b } else { a });
    Some(serde_json::json!({
        "liquid": c.liquid,
        "tides": tide_json(c, body, &rs),
        "waves": {
            "typical_m": if wsum > 0.0 { hsum / wsum } else { 0.0 },
            "typical_period_s": if wsum > 0.0 { tsum / wsum } else { 0.0 },
            "roughest_m": rough,
            "roughest_lat_deg": cl.lat_deg[rough_j],
            "bands": bands,
            "block": "sse.ocean.wind_waves",
            "note": "from each band's year-round wind near the surface; Earth's fitted relations, with this world's gravity",
        },
    }))
}

/// A landing site's sea state: waves under the wind blowing now, with the
/// fetch of the site's band, and the tides when the body is known.
pub(crate) fn site(
    cs: &ClimateSettings,
    cl: &Climate,
    lat_deg: f64,
    wind_km_h: f64,
    body: Option<(&sse_surface::Body, &[MoonPull])>,
) -> Option<serde_json::Value> {
    let c = &cl.conditions;
    if !c.solid_surface || c.liquid.is_none() || c.ocean_fraction <= 0.0 {
        return None;
    }
    let u = wind_km_h / 3.6;
    let fetch = fetch_m(cs, c, cl, lat_deg);
    let (h, t) = waves(u, fetch, c.gravity_m_s2)?;
    let mut o = serde_json::json!({
        "liquid": c.liquid,
        "wave_m": h,
        "period_s": t,
        "wind_m_s": u,
        "fetch_km": fetch / 1000.0,
        "block": "sse.ocean.wind_waves",
        "line": format!("Waves about {h:.1} m every {t:.0} s offshore, under this wind over {:.0} km of open {}", fetch / 1000.0, c.liquid.as_deref().unwrap_or("sea")),
    });
    if let Some((b, moons)) = body {
        o["tides"] = tide_json(c, b, &raisers(b, c, moons));
    }
    Some(o)
}
