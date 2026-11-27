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

/// Run one block with its settings in force (ER2).
fn run(id: &str, inputs: &[f64]) -> Option<Vec<f64>> {
    sse_physics::run(id, inputs).ok()
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
    // Below the block's range the sea is glassy; above it (hurricane
    // force) the fitted growth is held at its top.
    let w = wind_m_s.abs();
    if w < 0.5 {
        return Some((0.0, 0.0));
    }
    let v = run(
        "sse.ocean.wind_waves",
        &[w.min(30.0), fetch_m.max(1.0), gravity],
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

/// The sea liquid's own properties at a temperature (W-sea3), from the data
/// sheet's saturated-liquid table (held at its ends).
pub(crate) struct Liquid {
    pub rho: f64,
    pub mu: f64,
    pub sigma: f64,
    /// Below the sheet's lambda point (helium).
    pub superfluid: bool,
    pub lambda_k: Option<f64>,
}

fn liquid_id(cs: &ClimateSettings, liquid: &str) -> String {
    cs.substances
        .iter()
        .find(|s| s.name == liquid)
        .map(|s| s.gas.clone())
        .unwrap_or_else(|| liquid.to_string())
}

pub(crate) fn liquid(
    sheet: &sse_reference::Reference,
    cs: &ClimateSettings,
    name: &str,
    t_k: f64,
) -> Option<Liquid> {
    let id = liquid_id(cs, name);
    let ph = sheet.row("substances", &id)?.get("phases")?;
    let t = ph.get("saturated_liquid")?;
    let cols: Vec<&str> = t
        .get("columns")?
        .as_array()?
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    let rows: Vec<Vec<f64>> = t
        .get("rows")?
        .as_array()?
        .iter()
        .map(|r| {
            r.as_array()
                .map(|r| r.iter().filter_map(|x| x.as_f64()).collect())
                .unwrap_or_default()
        })
        .collect();
    let at = |name: &str| -> Option<f64> {
        let c = cols.iter().position(|x| *x == name)?;
        let k = cols.iter().position(|x| *x == "t_k")?;
        let first = rows.first()?;
        if t_k <= first[k] {
            return Some(first[c]);
        }
        for w in rows.windows(2) {
            if t_k <= w[1][k] {
                let f = (t_k - w[0][k]) / (w[1][k] - w[0][k]);
                return Some(w[0][c] + (w[1][c] - w[0][c]) * f);
            }
        }
        Some(rows.last()?[c])
    };
    let lambda_k = ph
        .get("lambda_point_k")
        .and_then(|v| v.get("v").unwrap_or(v).as_f64());
    Some(Liquid {
        rho: at("density")?,
        mu: at("viscosity")?,
        sigma: at("surface_tension")?,
        superfluid: lambda_k.is_some_and(|l| t_k < l),
        lambda_k,
    })
}

/// How much harder this air drives this liquid's waves than Earth's air
/// drives water: the wind is scaled by sqrt((ρ_air/ρ_liquid) / reference)
/// before Earth's fitted growth reads it (the waves' momentum comes from
/// the air, their restoring weight from the liquid).
pub(crate) fn drive(cs: &ClimateSettings, cl: &Climate, rho_l: f64) -> f64 {
    let r = cs.sea.reference_density_ratio;
    let rho_a = sse_climate::saltation::air_density(cl, cl.conditions.t_surface_k);
    if r > 0.0 && rho_l > 0.0 && rho_a > 0.0 {
        m::sqrt(rho_a / rho_l / r)
    } else {
        1.0
    }
}

/// Wavenumber of a capillary-gravity wave of angular frequency ω:
/// ω² = g k + (σ/ρ) k³, by Newton's method from the gravity wave.
fn wavenumber(omega: f64, g: f64, s: f64) -> f64 {
    let mut k = omega * omega / g;
    for _ in 0..30 {
        let f = g * k + s * k * k * k - omega * omega;
        let d = g + 3.0 * s * k * k;
        k -= f / d;
    }
    k
}

/// Wave trains to draw (at most five) for a sea of significant height `h`
/// and peak period `t`, travelling toward `dir_deg` (east 0, anticlockwise):
/// the Pierson-Moskowitz spectrum S(ω) = α g² ω⁻⁵ exp(−5/4 (ω_p/ω)⁴), α
/// set so that H_s = 4 sqrt(m₀), sampled at ω_p × 1, 1.4, 2, 2.8 and 4,
/// each with ω² = g k + (σ/ρ) k³ and fanned ±15° and ±30° about the wind.
/// `nu` is the liquid's kinematic viscosity (m²/s; 0 when unknown): each
/// train's amplitude decays by e in `damping_s` = 1 / (2 ν k²) once the wind
/// stops (Lamb 1932, art. 348).
pub(crate) fn trains(
    h: f64,
    t: f64,
    dir_deg: f64,
    g: f64,
    s: f64,
    nu: f64,
) -> Vec<serde_json::Value> {
    if h <= 0.0 || t <= 0.0 {
        return vec![];
    }
    let wp = 2.0 * m::PI / t;
    let alpha = 5.0 * m::pow(wp, 4.0) * (h / 4.0) * (h / 4.0) / (g * g);
    let step = m::sqrt(1.4);
    [
        (1.0, 0.0),
        (1.4, 15.0),
        (2.0, -15.0),
        (2.8, 30.0),
        (4.0, -30.0),
    ]
    .iter()
    .map(|(f, fan)| {
        let w = wp * f;
        let sw = alpha * g * g / m::pow(w, 5.0) * m::exp(-1.25 * m::pow(wp / w, 4.0));
        let dw = w * (step - 1.0 / step);
        let k = wavenumber(w, g, s);
        let d = m::fmod(dir_deg + fan + 360.0, 360.0);
        let mut o = serde_json::json!({
            "wavelength_m": 2.0 * m::PI / k,
            "amplitude_m": m::sqrt(2.0 * sw * dw),
            "period_s": 2.0 * m::PI / w,
            "dir_deg": d,
        });
        if nu > 0.0 {
            o["damping_s"] = serde_json::json!(1.0 / (2.0 * nu * k * k));
        }
        o
    })
    .collect()
}

/// The liquid's numbers for the card and the drawing: density, viscosity,
/// surface tension, the capillary cut-off 2π sqrt(σ/(ρ g)) below which
/// ripples are held by surface tension, and the superfluid line.
fn liquid_json(l: &Liquid, name: &str, g: f64) -> serde_json::Value {
    let mut o = serde_json::json!({
        "density_kg_m3": l.rho,
        "viscosity_pa_s": l.mu,
        "surface_tension_n_m": l.sigma,
        "ripple_cutoff_m": 2.0 * m::PI * m::sqrt(l.sigma / (l.rho * g)),
        "min_phase_speed_m_s": m::pow(4.0 * g * l.sigma / l.rho, 0.25),
        "superfluid": l.superfluid,
    });
    if l.superfluid {
        o["liquid_note"] = serde_json::json!(format!(
            "Liquid {name} below its lambda point ({:.2} K): a superfluid, flowing with no viscosity; its waves are ordinary capillary-gravity waves",
            l.lambda_k.unwrap_or(0.0)
        ));
    }
    o
}

/// Share of the sea under whitecaps at a 10 m wind (already scaled by the
/// liquid's drive), 0 to 1.
pub(crate) fn whitecap(cs: &ClimateSettings, u: f64) -> f64 {
    let Some(w) = &cs.sea.whitecaps else {
        return 0.0;
    };
    let x = u.abs() - w.onset_m_s;
    if x <= 0.0 {
        0.0
    } else {
        (w.coefficient * m::pow(x, w.exponent)).min(1.0)
    }
}

/// The bands the sea's colour is given in, nm (red, green, blue).
pub(crate) const BANDS_NM: [f64; 3] = [650.0, 550.0, 450.0];

/// A sheet table `{lambda_nm: [...], <key>: [...] or {v: [...]}}` read at
/// `nm`, linear between rows, held at the ends.
fn table_at(t: &serde_json::Value, key: &str, nm: f64) -> Option<f64> {
    let xs: Vec<f64> = t
        .get("lambda_nm")?
        .as_array()?
        .iter()
        .filter_map(|x| x.as_f64())
        .collect();
    let col = t.get(key)?;
    let col = col.get("v").unwrap_or(col);
    let ys: Vec<f64> = col.as_array()?.iter().filter_map(|x| x.as_f64()).collect();
    if xs.is_empty() || xs.len() != ys.len() {
        return None;
    }
    if nm <= xs[0] {
        return Some(ys[0]);
    }
    for i in 1..xs.len() {
        if nm <= xs[i] {
            let f = (nm - xs[i - 1]) / (xs[i] - xs[i - 1]);
            return Some(ys[i - 1] + (ys[i] - ys[i - 1]) * f);
        }
    }
    ys.last().copied()
}

/// The sea's optics from the data sheet (W-sea2): the liquid's absorption
/// and refractive index, its molecular backscatter where the sheet holds
/// one, and the deep-water colour R = 0.33 b_b / (a + b_b) in each band
/// (Morel and Prieur 1977 eq. 1, with a + b_b so that a clear, weakly
/// absorbing sea stays below 0.33). What the sheet lacks is named, not
/// guessed.
pub(crate) fn optics(
    sheet: &sse_reference::Reference,
    cs: &ClimateSettings,
    liquid: &str,
) -> serde_json::Value {
    let id = cs
        .substances
        .iter()
        .find(|s| s.name == liquid)
        .map(|s| s.gas.clone())
        .unwrap_or_else(|| liquid.to_string());
    let opt = sheet
        .row("substances", &id)
        .and_then(|r| r.get("optical"))
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    let read = |table: &str, key: &str| -> Option<Vec<f64>> {
        let t = opt.get(table)?;
        BANDS_NM.iter().map(|nm| table_at(t, key, *nm)).collect()
    };
    let absorb = read("absorption_liquid", "alpha");
    let n = opt
        .get("refractive_index_liquid")
        .and_then(|t| table_at(t, "n", 550.0));
    let share = opt
        .get("scattering_liquid")
        .and_then(|t| t.get("backscatter_share"))
        .and_then(|v| v.get("v").unwrap_or(v).as_f64());
    // Molecular scattering: the table inside its range, the sheet's power
    // law beyond it (from the nearest row).
    let scat = |nm: f64| -> Option<f64> {
        let t = opt.get("scattering_liquid")?;
        let xs: Vec<f64> = t
            .get("lambda_nm")?
            .as_array()?
            .iter()
            .filter_map(|x| x.as_f64())
            .collect();
        let (lo, hi) = (*xs.first()?, *xs.last()?);
        let ex = t
            .get("exponent")
            .and_then(|v| v.get("v").unwrap_or(v).as_f64());
        let end = if nm < lo {
            lo
        } else if nm > hi {
            hi
        } else {
            nm
        };
        let b = table_at(t, "b", end)?;
        Some(if end == nm {
            b
        } else {
            b * m::pow(nm / end, ex?)
        })
    };
    let back = BANDS_NM
        .iter()
        .map(|nm| scat(*nm))
        .collect::<Option<Vec<f64>>>()
        .zip(share)
        .map(|(b, s)| b.iter().map(|x| x * s).collect::<Vec<f64>>());
    let mut o = serde_json::json!({
        "bands_nm": BANDS_NM,
        "liquid_id": id,
        "block": "sse.reference look-up (substances.<liquid>.optical)",
    });
    let mut missing = vec![];
    match &absorb {
        Some(a) => o["absorb_per_m"] = serde_json::json!(a),
        None => missing.push("absorption"),
    }
    match &back {
        Some(b) => o["backscatter_per_m"] = serde_json::json!(b),
        None => missing.push("scattering"),
    }
    match n {
        Some(n) => o["refractive_index"] = serde_json::json!(n),
        None => missing.push("refractive index"),
    }
    if let (Some(a), Some(b)) = (&absorb, &back) {
        let rgb: Vec<f64> = a
            .iter()
            .zip(b)
            .map(|(a, b)| if a + b > 0.0 { 0.33 * b / (a + b) } else { 0.0 })
            .collect();
        o["deep_rgb"] = serde_json::json!(rgb);
    }
    if !missing.is_empty() {
        o["note"] = serde_json::json!(format!(
            "the data sheet holds no liquid {} for {liquid} yet, so its colour is not worked out",
            missing.join(", ")
        ));
    }
    o
}

/// The sea's tides and waves for a world's card, or None without a sea.
pub(crate) fn summary(
    sheet: &sse_reference::Reference,
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
    let name = c.liquid.as_deref().unwrap_or("");
    let liq = liquid(sheet, cs, name, c.t_surface_k);
    let rho_l = liq.as_ref().map_or(cl.liquid_kg_m3, |l| l.rho);
    let s_cap = liq.as_ref().map_or(0.0, |l| l.sigma / l.rho);
    let nu = liq.as_ref().map_or(0.0, |l| l.mu / l.rho);
    let drive = drive(cs, cl, rho_l);
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
        // Over the year's 10 m winds, storms and calms alike: the mean of
        // the waves each wind raises, not the waves of the mean wind.
        let (u, gust) = cl.wind10_at(lat);
        let g = c.gravity_m_s2;
        let wave = |w: f64, k: usize| {
            waves(w * drive, fetch, g).map_or(0.0, |v| if k == 0 { v.0 } else { v.1 })
        };
        let h = sse_climate::wind::expect(u, gust, |w| wave(w, 0));
        let t = sse_climate::wind::expect(u, gust, |w| wave(w, 1));
        let speed = sse_climate::wind::mean_speed(u, gust);
        let froth = sse_climate::wind::expect(u, gust, |w| whitecap(cs, w * drive));
        if h <= 0.0 {
            bands.push(serde_json::Value::Null);
            continue;
        }
        let w = sea * m::cos(lat * m::PI / 180.0);
        wsum += w;
        hsum += w * h;
        tsum += w * t;
        // Westerlies drive the waves east (0°), easterlies west (180°).
        let dir = if u >= 0.0 { 0.0 } else { 180.0 };
        bands.push(serde_json::json!({
            "wave_m": h, "period_s": t, "fetch_km": fetch / 1000.0, "wind_m_s": speed,
            "trains": trains(h, t, dir, g, s_cap, nu),
            "whitecap_fraction": froth,
            "streak_dir_deg": dir,
        }));
    }
    let (rough_j, rough) = bands
        .iter()
        .enumerate()
        .filter_map(|(j, b)| b.get("wave_m").and_then(|x| x.as_f64()).map(|h| (j, h)))
        .fold((0, 0.0), |a, b| if b.1 > a.1 { b } else { a });
    let mut o = serde_json::json!({
        "liquid": c.liquid,
        "tides": tide_json(c, body, &rs),
        "waves": {
            "typical_m": if wsum > 0.0 { hsum / wsum } else { 0.0 },
            "typical_period_s": if wsum > 0.0 { tsum / wsum } else { 0.0 },
            "roughest_m": rough,
            "roughest_lat_deg": cl.lat_deg[rough_j],
            "bands": bands,
            "block": "sse.ocean.wind_waves",
            "note": "the year's mean over each band's 10 m winds, storms and calms alike; wind sea only, no swell; Earth's fitted relations, with this world's gravity",
        },
    });
    merge(&mut o, optics(sheet, cs, name));
    if let Some(l) = &liq {
        merge(&mut o, liquid_json(l, name, c.gravity_m_s2));
    }
    o["drive"] = serde_json::json!(drive);
    if let Some(w) = &cs.sea.whitecaps {
        o["foam_albedo"] = serde_json::json!(w.foam_albedo);
    }
    Some(o)
}

/// The sun's glint off a world's seas seen from orbit (EC2): the waves'
/// mean square slope from the year's 10 m winds over each band of sea,
/// weighted by its share of sea and its area, and the liquid's reflectance
/// straight back, both from `sse.light.sea_glint` (Cox and Munk 1954;
/// Fresnel at normal incidence) with the liquid's refractive index looked
/// up on the data sheet. The wind is the one that raises the waves, scaled
/// by this world's air-to-liquid density as the waves are. None without a
/// sea; a liquid the sheet has no index for is named, not guessed.
pub(crate) fn glint(
    sheet: &sse_reference::Reference,
    cs: &ClimateSettings,
    cl: &Climate,
) -> Option<serde_json::Value> {
    let c = &cl.conditions;
    if !c.solid_surface || c.liquid.is_none() || c.ocean_fraction <= 0.0 {
        return None;
    }
    let name = c.liquid.as_deref().unwrap_or("");
    let liq = liquid(sheet, cs, name, c.t_surface_k);
    let rho_l = liq.as_ref().map_or(cl.liquid_kg_m3, |l| l.rho);
    let drive = drive(cs, cl, rho_l);
    // This universe's sheet first; the base sheet where it holds no row
    // for the liquid (as the bows read it).
    let mut o = optics(sheet, cs, name);
    if o.get("refractive_index").is_none() {
        o = optics(&sse_reference::Reference::base(), cs, name);
    }
    let index = o.get("refractive_index").and_then(|v| v.as_f64());
    let mut out = serde_json::json!({
        "liquid": c.liquid,
        "liquid_id": o.get("liquid_id"),
        "block": "sse.light.sea_glint",
        "index_from": "sse.reference look-up (substances.<liquid>.optical.refractive_index_liquid, 550 nm)",
    });
    let Some(n) = index else {
        out["missing"] = serde_json::json!(["refractive index"]);
        out["why"] = serde_json::json!([format!(
            "The data sheet has no refractive index for liquid {name}, so its glint is not worked out."
        )]);
        return Some(out);
    };
    let k = cl.lat_deg.len();
    let (mut wsum, mut ssum, mut usum) = (0.0, 0.0, 0.0);
    for j in 0..k {
        let lat = cl.lat_deg[j];
        let sea = if cl.land.len() == k {
            1.0 - cl.land[j]
        } else {
            c.ocean_fraction
        };
        if sea <= 0.0 {
            continue;
        }
        let (u, gust) = cl.wind10_at(lat);
        let s2 = sse_climate::wind::expect(u, gust, |w| {
            run("sse.light.sea_glint", &[(w * drive).abs(), n]).map_or(0.0, |v| v[0])
        });
        let w = sea * m::cos(lat * m::PI / 180.0);
        wsum += w;
        ssum += w * s2;
        usum += w * sse_climate::wind::mean_speed(u, gust);
    }
    if wsum <= 0.0 {
        return None;
    }
    let s2 = ssum / wsum;
    let wind = usum / wsum;
    let r0 = run("sse.light.sea_glint", &[0.0, n]).map(|v| v[1])?;
    out["slope_variance"] = serde_json::json!(s2);
    out["normal_reflectance"] = serde_json::json!(r0);
    out["refractive_index"] = serde_json::json!(n);
    out["wind_m_s"] = serde_json::json!(wind);
    out["drive"] = serde_json::json!(drive);
    out["why"] = serde_json::json!([
        format!("The waves' mean square slope is {s2:.4}, from the year's winds over the sea (about {wind:.1} m/s ten metres up), by Cox and Munk's clean-sea fit."),
        format!("The {name} reflects {:.1}% of the light straight back (refractive index {n:.3}, from the data sheet).", r0 * 100.0),
    ]);
    Some(out)
}

/// Copy `from`'s keys into `into` (both objects).
fn merge(into: &mut serde_json::Value, from: serde_json::Value) {
    if let (Some(a), serde_json::Value::Object(b)) = (into.as_object_mut(), from) {
        for (k, v) in b {
            a.entry(k).or_insert(v);
        }
    }
}

/// A landing site's sea state: waves under the wind blowing now, with the
/// fetch of the site's band, and the tides when the body is known.
pub(crate) fn site(
    sheet: &sse_reference::Reference,
    cs: &ClimateSettings,
    cl: &Climate,
    lat_deg: f64,
    wind_km_h: f64,
    wind_from_deg: f64,
    body: Option<(&sse_surface::Body, &[MoonPull])>,
) -> Option<serde_json::Value> {
    let c = &cl.conditions;
    if !c.solid_surface || c.liquid.is_none() || c.ocean_fraction <= 0.0 {
        return None;
    }
    let name = c.liquid.as_deref().unwrap_or("");
    let liq = liquid(sheet, cs, name, cl.lerp(&cl.t_k, lat_deg));
    let rho_l = liq.as_ref().map_or(cl.liquid_kg_m3, |l| l.rho);
    let drive = drive(cs, cl, rho_l);
    let u = wind_km_h / 3.6;
    let fetch = fetch_m(cs, c, cl, lat_deg);
    let (h, t) = waves(u * drive, fetch, c.gravity_m_s2)?;
    // The waves run downwind: from-direction clockwise from north to a
    // heading anticlockwise from east.
    let dir = m::fmod(90.0 - (wind_from_deg + 180.0) + 720.0, 360.0);
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
    merge(&mut o, optics(sheet, cs, name));
    if let Some(l) = &liq {
        merge(&mut o, liquid_json(l, name, c.gravity_m_s2));
        o["trains"] = serde_json::json!(trains(
            h,
            t,
            dir,
            c.gravity_m_s2,
            l.sigma / l.rho,
            l.mu / l.rho
        ));
    }
    o["drive"] = serde_json::json!(drive);
    o["whitecap_fraction"] = serde_json::json!(whitecap(cs, u * drive));
    o["streak_dir_deg"] = serde_json::json!(dir);
    if let Some(w) = &cs.sea.whitecaps {
        o["foam_albedo"] = serde_json::json!(w.foam_albedo);
    }
    Some(o)
}

#[cfg(test)]
mod optics_tests {
    use super::*;

    fn with_scattering() -> sse_reference::Reference {
        let mut r = sse_reference::Reference::base();
        let row = r
            .tables
            .get_mut("substances")
            .and_then(|t| t.get_mut("h2o"))
            .expect("h2o row");
        let opt = row
            .get_mut("optical")
            .and_then(|o| o.as_object_mut())
            .expect("optical");
        // The data sheet's coming row (Morel and Prieur 1977 table 1).
        opt.entry("scattering_liquid").or_insert(serde_json::json!({
            "kind": "table", "lambda_nm": [450.0, 550.0],
            "b": {"v": [0.0035, 0.0015], "unit": "m-1"},
            "exponent": {"v": -4.3}, "backscatter_share": {"v": 0.5}
        }));
        r
    }

    #[test]
    fn a_water_sea_is_blue_from_its_own_absorption() {
        let cs = &sse_surface::scenario::settings().climate;
        let o = optics(&with_scattering(), cs, "water");
        let rgb: Vec<f64> = o["deep_rgb"]
            .as_array()
            .expect("deep colour")
            .iter()
            .map(|x| x.as_f64().unwrap())
            .collect();
        // Red absorbed most, blue least: blue over green over red.
        assert!(rgb[2] > rgb[1] && rgb[1] > rgb[0], "{rgb:?}");
        // R = 0.33 b_b / (a + b_b) at 450 nm: about 0.06.
        assert!((0.04..0.08).contains(&rgb[2]), "{rgb:?}");
        let n = o["refractive_index"].as_f64().unwrap();
        assert!((1.33..1.34).contains(&n), "{n}");
    }

    #[test]
    fn a_liquid_the_sheet_cannot_colour_says_so() {
        let cs = &sse_surface::scenario::settings().climate;
        let o = optics(&with_scattering(), cs, "methane");
        assert!(o.get("deep_rgb").is_none());
        assert!(o["note"].as_str().unwrap().contains("absorption"));
    }
}

#[cfg(test)]
mod liquid_tests {
    use super::*;

    #[test]
    fn liquids_carry_their_own_properties_and_ripples() {
        let sheet = sse_reference::Reference::base();
        let cs = &sse_surface::scenario::settings().climate;
        let w = liquid(&sheet, cs, "water", 288.0).expect("water");
        let ch4 = liquid(&sheet, cs, "methane", 92.0).expect("methane");
        // Water near 1000 kg/m³ and 0.073 N/m; methane near 450 and 0.017.
        assert!((990.0..1001.0).contains(&w.rho) && (0.07..0.076).contains(&w.sigma));
        assert!((440.0..455.0).contains(&ch4.rho) && (0.015..0.018).contains(&ch4.sigma));
        // Capillary cut-off 2π sqrt(σ/(ρg)): water on Earth about 1.7 cm.
        let lw = 2.0 * m::PI * m::sqrt(w.sigma / (w.rho * 9.81));
        assert!((0.016..0.018).contains(&lw), "{lw}");
        // Methane under Titan's 1.35 m/s²: longer, about 3.3 cm.
        let lm = 2.0 * m::PI * m::sqrt(ch4.sigma / (ch4.rho * 1.35));
        assert!((0.03..0.036).contains(&lm), "{lm}");
        assert!(!w.superfluid && !ch4.superfluid);
    }

    #[test]
    fn trains_follow_the_dispersion_and_the_spectrum() {
        // Earth: 2 m sea, 7 s period, toward the east.
        let t = trains(2.0, 7.0, 0.0, 9.81, 0.072 / 1000.0, 1.0e-6);
        assert_eq!(t.len(), 5);
        let l0 = t[0]["wavelength_m"].as_f64().unwrap();
        // Deep water: λ = g T² / 2π ≈ 76 m.
        assert!((70.0..80.0).contains(&l0), "{l0}");
        let a: Vec<f64> = t
            .iter()
            .map(|x| x["amplitude_m"].as_f64().unwrap())
            .collect();
        assert!(a[0] > a[2] && a[2] > a[4], "{a:?}");
        // Sum of squares of amplitudes over the sampled bands stays below
        // m₀ × 2 = 2 (H_s/4)².
        let m0: f64 = a.iter().map(|x| x * x / 2.0).sum();
        assert!(m0 <= 0.25 * 1.05, "{m0}");
    }

    #[test]
    fn whitecaps_start_above_their_onset() {
        let cs = &sse_surface::scenario::settings().climate;
        assert_eq!(whitecap(cs, 4.0), 0.0);
        // 10 m/s: 5e-5 × 5.53³ ≈ 0.85 %.
        let w = whitecap(cs, 10.0);
        assert!((0.007..0.010).contains(&w), "{w}");
    }
}

#[cfg(test)]
mod helium_tests {
    use super::*;

    /// Alex's test world: a rogue planet in a void, no star, warmed only by
    /// its own faint heat to under 2 K, under a thin helium air. Nothing
    /// here is special to helium: the sheet's curve and properties do it.
    #[test]
    fn a_rogue_planet_holds_a_superfluid_helium_sea() {
        let sheet = sse_reference::Reference::base();
        let s = sse_surface::scenario::settings();
        let cs = &s.climate;
        let mut b = sse_surface::scenario::fixtures()
            .into_iter()
            .find(|f| f.body.label == "Earth")
            .unwrap()
            .body;
        b.label = "Rogue".into();
        b.orbit = None;
        b.oceans = 1.0;
        b.pressure_bar = 0.02;
        // σT⁴ at 1.8 K is about 6e-7 W/m².
        let t_k = 1.8;
        b.climate = sse_surface::Climate::Fixed {
            t_surface_k: t_k,
            then: None,
        };
        let env = sse_surface::envelope(&s, &b);
        let x = sse_climate::Extra {
            flux_w_m2: Some(0.0),
            internal_flux_w_m2: Some(5.670374419e-8 * t_k * t_k * t_k * t_k),
            gases: Some(std::collections::BTreeMap::from([("he".to_string(), 1.0)])),
            rotation_hours: Some(20.0),
            ..Default::default()
        };
        let (c, assumed) = sse_surface::climate::conditions(&s, &b, &env, &x);
        assert_eq!(c.liquid.as_deref(), Some("helium"), "{assumed:?}");
        let cl = sse_climate::climate::orbit(cs, &c, 0.0);
        let sea = summary(&sheet, cs, &b, &cl, &[]).expect("a sea card");
        let text = sea.to_string();
        assert_eq!(sea["liquid"], "helium", "{text:.800}");
        assert_eq!(sea["superfluid"], true, "{text:.800}");
        // Helium's ripples: σ/ρ about 2e-6 m³/s², a centimetre-scale cut-off.
        let cut = sea["ripple_cutoff_m"].as_f64().expect("cut-off");
        assert!(cut > 0.002 && cut < 0.03, "{cut}");
        // Waves under its own wind, band by band.
        let banded = sea["waves"]["bands"]
            .as_array()
            .expect("bands")
            .iter()
            .filter(|b| b["trains"].as_array().is_some_and(|t| !t.is_empty()))
            .count();
        assert!(banded > 0, "{text:.1500}");
        // Helium's tiny viscosity: a 1 m wave rings for days, about 40
        // times water's (ν 2.4e-8 against 1.0e-6 m²/s).
        let he = trains(0.1, 2.0, 0.0, 9.81, 2.86e-4 / 146.0, 3.576e-6 / 146.0);
        let h2o = trains(0.1, 2.0, 0.0, 9.81, 7.2e-5, 1.0e-6);
        let r = he[0]["damping_s"].as_f64().unwrap() / h2o[0]["damping_s"].as_f64().unwrap();
        assert!((30.0..50.0).contains(&r), "{r}");
    }
}
