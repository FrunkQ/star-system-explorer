//! The colour of light inside a world's shadow (visual polish VP-L3): the
//! star's light bent through the limb of the world's air into its shadow,
//! reddened along the grazing path (Rayleigh scattering and the sheet's gas
//! absorption, `light_through_gas`) and spread thin by the bending. An
//! eclipsed moon turns copper round an Earth and stays black round an
//! airless world. The physics is the `sse.shadow.*` blocks
//! (`packs/base-physics/formulas/shadow.json`).

use crate::air::{self, Air};
use crate::colour::{self, Colour};
use crate::gas::{self, Amount, Layer, Passage};
use crate::table::LightTable;
use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_flow::{Atom, Reference};

/// The star lighting the world.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Star {
    pub teff_k: f64,
    pub radius_m: f64,
    /// From the world, m.
    pub distance_m: f64,
}

/// One question: what light reaches the middle of this world's shadow at
/// this distance behind it?
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShadowAsk {
    pub id: String,
    /// The world's air (gases, surface pressure and temperature, gravity).
    pub air: Air,
    pub radius_m: f64,
    /// Distance behind the world, along the shadow (a moon's orbit), m.
    pub distance_m: f64,
    pub star: Star,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShadowLight {
    pub id: String,
    /// Whether any bent light reaches the middle of the shadow here.
    pub lit: bool,
    /// The colour of that light to the eye; absent when none arrives.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour: Option<Colour>,
    /// The star's own colour, for comparison.
    pub star_colour: Colour,
    /// Lowest and highest heights above the surface the light skims, m.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heights_m: Option<[f64; 2]>,
    /// Share of green light (550 nm) that the grazing path lets through at
    /// the middle of those heights, before the spreading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transmission_550: Option<f64>,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

fn block(id: &str) -> Result<&'static dyn Atom, String> {
    sse_atoms::formulas::loaded()
        .iter()
        .find(|a| a.decl().id == id)
        .map(|a| a as &dyn Atom)
        .ok_or_else(|| format!("base formula block {id} is missing"))
}

fn run(id: &str, inputs: &[f64]) -> Result<f64, String> {
    Ok(block(id)?
        .eval(inputs, &[])
        .map_err(|e| format!("{id}: {e}"))?[0])
}

/// Heights tried per scale height, and how many scale heights up.
const STEPS_PER_H: usize = 8;
const SCALE_HEIGHTS: usize = 16;
/// Green, where the bend and the transmission are quoted.
const GREEN_M: f64 = 550e-9;

pub fn shadow_light(
    ask: &ShadowAsk,
    table: &LightTable,
    sheet: &sse_reference::Reference,
) -> Result<ShadowLight, String> {
    if !(ask.radius_m > 0.0 && ask.distance_m >= 0.0) {
        return Err("radius must be positive and distance zero or more".into());
    }
    let st = &ask.star;
    if !(st.teff_k > 0.0 && st.radius_m > 0.0 && st.distance_m > 0.0) {
        return Err("the star needs a temperature, radius and distance above zero".into());
    }
    let eye = table.eye.as_ref().ok_or("the light table has no eye")?;
    let wl = eye.wavelengths_m();
    let star_light: Vec<f64> = wl.iter().map(|l| crate::planck(*l, st.teff_k)).collect();
    let star_colour = colour::colour_of(eye, &star_light)?;
    let mut references: Vec<Reference> = Vec::new();
    for id in [
        "sse.shadow.grazing_column",
        "sse.shadow.refraction_bend",
        "sse.shadow.refraction_dilution",
    ] {
        for r in &block(id)?.decl().references {
            if !references.contains(r) {
                references.push(r.clone());
            }
        }
    }
    let mut stand_ins = vec![
        "clear air only: clouds, haze and dust along the limb (which dim and change an eclipse a great deal) are not counted yet".into(),
        "the air is one temperature all the way up (an exponential air with the surface's scale height)".into(),
        "the colour is the middle of the shadow; the shadow's edge, where the light skims higher, is not worked yet".into(),
    ];
    let unlit = |why: String, stand_ins: Vec<String>, references: Vec<Reference>| ShadowLight {
        id: ask.id.clone(),
        lit: false,
        colour: None,
        star_colour: star_colour.clone(),
        heights_m: None,
        transmission_550: None,
        why: vec![why],
        stand_ins,
        references,
    };
    if ask.air.gases.is_empty() || ask.air.surface_pressure_pa <= 0.0 {
        return Ok(unlit(
            "No air to bend the light: the shadow stays black.".into(),
            vec![],
            references,
        ));
    }
    let optics = air::optics(&ask.air, &[GREEN_M], table)?;
    let h_scale = optics.scale_height_m;
    let n0 = optics.surface_density_m3;
    let loschmidt = sse_atoms::air::N_LOSCHMIDT;
    let nu_green = air::refractivity_of(&ask.air, GREEN_M, table)?;
    let sigma: Vec<f64> = wl
        .iter()
        .map(|l| air::cross_section(&ask.air, *l, table))
        .collect::<Result<_, _>>()?;
    // Absorbing gases the sheet knows; the rest are read as clear.
    let mut optics_rows = Vec::new();
    for g in &ask.air.gases {
        match gas::GasOptics::from_sheet(sheet, &g.gas) {
            Ok(o) => optics_rows.push(o),
            Err(_) => stand_ins.push(format!(
                "{}: no substance row on the data sheet, so read as clear of absorption",
                g.gas
            )),
        }
    }
    let s_star = st.radius_m / st.distance_m;
    let d = ask.distance_m;
    let dh = h_scale / STEPS_PER_H as f64;
    let mut total = vec![0.0; wl.len()];
    let (mut lo, mut hi) = (f64::INFINITY, 0.0_f64);
    let mut green_at: Vec<f64> = Vec::new();
    for i in 0..STEPS_PER_H * SCALE_HEIGHTS {
        let h = (i as f64 + 0.5) * dh;
        let r = ask.radius_m + h;
        let n = n0 * m::exp(-h / h_scale);
        let nu = nu_green * n / loschmidt;
        let theta = run("sse.shadow.refraction_bend", &[nu, h_scale, r])?;
        // Light from this height reaches the shadow's middle at d when the
        // bend brings it to the axis, from some part of the star's disc.
        if (r - theta * d).abs() > s_star * d {
            continue;
        }
        let phi = run("sse.shadow.refraction_dilution", &[theta, h_scale, d])?;
        let column = run("sse.shadow.grazing_column", &[n, h_scale, r])?;
        let path = column / n;
        let passage = Passage {
            address: ask.air.address.clone(),
            layers: vec![Layer {
                t_k: ask.air.surface_temperature_k,
                path_m: path,
                gases: ask
                    .air
                    .gases
                    .iter()
                    .filter(|g| optics_rows.iter().any(|o| o.gas == g.gas))
                    .map(|g| Amount {
                        gas: g.gas.clone(),
                        number_density_m3: n * g.fraction,
                    })
                    .collect(),
            }],
            path_factor: 1.0,
        };
        let through: Vec<f64> = if passage.layers[0].gases.is_empty() {
            star_light.clone()
        } else {
            gas::pass(&passage, &optics_rows, &wl, &star_light, sheet)?.light_out
        };
        for (k, t) in total.iter_mut().enumerate() {
            *t += through[k] * m::exp(-sigma[k] * column) * phi * dh;
        }
        lo = lo.min(h);
        hi = hi.max(h);
        green_at.push(m::exp(
            -air::cross_section(&ask.air, GREEN_M, table)? * column,
        ));
    }
    if green_at.is_empty() {
        return Ok(unlit(
            format!(
                "The air bends light by at most {:.2}° here, too little to reach the middle of the shadow {:.0} km behind: it stays dark there.",
                run("sse.shadow.refraction_bend", &[nu_green * n0 / loschmidt, h_scale, ask.radius_m])? * 180.0 / m::PI,
                d / 1e3
            ),
            stand_ins,
            references,
        ));
    }
    let c = colour::colour_of(eye, &total)?;
    let t550 = green_at[green_at.len() / 2];
    let why = vec![
        format!(
            "Starlight skimming the air {:.1} to {:.1} km above the ground is bent by about {:.2}° and reaches the middle of the shadow {:.0} km behind.",
            lo / 1e3,
            hi / 1e3,
            run("sse.shadow.refraction_bend", &[nu_green * n0 * m::exp(-0.5 * (lo + hi) / h_scale) / loschmidt, h_scale, ask.radius_m])? * 180.0 / m::PI,
            d / 1e3
        ),
        format!(
            "Along that grazing path the air scatters blue light away far more than red: {:.0}% of green light gets through, so the shadow glows {}.",
            t550 * 100.0,
            c.hex
        ),
    ];
    references.extend(colour::references(eye));
    Ok(ShadowLight {
        id: ask.id.clone(),
        lit: true,
        colour: Some(c),
        star_colour,
        heights_m: Some([lo, hi]),
        transmission_550: Some(t550),
        why,
        stand_ins,
        references,
    })
}
