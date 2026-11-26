//! How aurorae shine (MF9 part 2b, visual polish VP-L2): the power each
//! driver brings down, how deep its electrons reach in this air, which of
//! the light table's lines its gases give off, how much of each gets out
//! before collisions put it out, at what height, and the colour of the mix.
//!
//! Nothing here asks what kind of world it is: the lines come from the
//! air's own gases (`UpperAir::gases`, ids on the master data sheet), the
//! heights from its own density profile. The physics is the `sse.glow.*`
//! Equations in `packs/base-physics/formulas/glow.json`.

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_flow::Atom;
use sse_light::colour::{colour_of_lines, Colour};
use sse_light::table::LightTable;
use std::sync::OnceLock;

fn block(id: &str) -> &'static dyn Atom {
    sse_atoms::formulas::loaded()
        .iter()
        .find(|a| a.decl().id == id)
        .map(|a| a as &dyn Atom)
        .unwrap_or_else(|| panic!("base formula block {id} is missing"))
}

pub(crate) fn run(id: &str, inputs: &[f64]) -> Result<Vec<f64>, String> {
    block(id)
        .eval(inputs, &[])
        .map_err(|e| format!("{id}: {e}"))
}

fn table() -> &'static LightTable {
    static T: OnceLock<LightTable> = OnceLock::new();
    T.get_or_init(LightTable::base)
}

fn sheet() -> &'static sse_reference::Reference {
    static S: OnceLock<sse_reference::Reference> = OnceLock::new();
    S.get_or_init(sse_reference::Reference::base)
}

/// Atomic mass unit, kg, from the master data sheet.
fn amu() -> f64 {
    sheet()
        .constant("m_u")
        .expect("the master data sheet has the atomic mass unit m_u")
}

/// One gas's share of the air by number where aurorae glow.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GasShare {
    /// Element or substance id on the master data sheet ("o", "n2", "co2").
    pub matter: String,
    pub share: f64,
}

/// The air at one height.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AirLevel {
    /// Above the ground (for a giant, above the 1 bar level), m.
    pub height_m: f64,
    pub number_density_m3: f64,
    /// Mean mass of one particle, in atomic mass units.
    pub mean_mass_u: f64,
}

/// The high air where aurorae glow. With no profile the air falls off
/// with one scale height from its base (`Body::air`, or `base_pressure_pa`
/// and `scale_height_m` here for a giant with no ground).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UpperAir {
    pub gases: Vec<GasShare>,
    /// Measured or modelled levels, rising.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub profile: Vec<AirLevel>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_pressure_pa: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scale_height_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gravity_m_s2: Option<f64>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub source: String,
}

/// One line of an aurora.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Emission {
    pub line: String,
    /// The light table's name for the line ("[O I] 557.7 nm (green aurora, airglow)").
    pub name: String,
    pub matter: String,
    pub wavelength_m: f64,
    /// Both hemispheres together, W.
    pub power_w: f64,
    /// Share of the power coming down that leaves in this line.
    pub share: f64,
    /// Middle of the glowing layer (half the light above, half below), m.
    pub peak_height_m: f64,
    /// From the height with a tenth of the light below to the one with a
    /// tenth above, m.
    pub thickness_m: f64,
    /// The line's own colour, linear sRGB with the brightest channel at one
    /// (black for a line the eye cannot see).
    pub colour: [f64; 3],
}

/// What an oval or a diffuse glow gives off.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Glow {
    /// Power coming down, both hemispheres, W.
    pub power_w: f64,
    /// North and south, W (equal halves until seasons are modelled).
    pub power_by_hemisphere: [f64; 2],
    /// The falling electrons' energy, eV.
    pub electron_ev: f64,
    /// Where they stop, m above the base.
    pub stops_at_m: f64,
    pub emissions: Vec<Emission>,
    /// The mix to the eye; absent when no line is in the eye's range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub colour: Option<Colour>,
}

/// The air sampled on an even grid of heights: (height, number density,
/// mass density).
fn levels(air: &UpperAir, base: Option<(f64, f64, f64)>) -> Result<Vec<(f64, f64, f64)>, String> {
    let u = amu();
    if air.profile.len() >= 2 {
        let p = &air.profile;
        if p.windows(2).any(|w| w[1].height_m <= w[0].height_m)
            || p.iter()
                .any(|l| !(l.number_density_m3 > 0.0 && l.mean_mass_u > 0.0))
        {
            return Err(
                "the upper air's profile must rise in height with positive densities".into(),
            );
        }
        let (z0, z1) = (p[0].height_m, p[p.len() - 1].height_m);
        let n = 400;
        let mut out = Vec::with_capacity(n + 1);
        let mut j = 0;
        for i in 0..=n {
            let z = z0 + (z1 - z0) * i as f64 / n as f64;
            while j + 2 < p.len() && z > p[j + 1].height_m {
                j += 1;
            }
            let (a, b) = (&p[j], &p[j + 1]);
            let f = ((z - a.height_m) / (b.height_m - a.height_m)).clamp(0.0, 1.0);
            let nd =
                a.number_density_m3 * m::exp(f * m::ln(b.number_density_m3 / a.number_density_m3));
            let mm = a.mean_mass_u + f * (b.mean_mass_u - a.mean_mass_u);
            out.push((z, nd, nd * mm * u));
        }
        return Ok(out);
    }
    // One scale height from the base: column p/g, density p/(g H).
    let (p0, h, g) =
        base.ok_or("the upper air has no profile and no base pressure and scale height")?;
    if !(p0 > 0.0 && h > 0.0 && g > 0.0) {
        return Err(
            "the upper air needs a positive base pressure, scale height and gravity".into(),
        );
    }
    let mean_u = mean_mass_u(&air.gases)?;
    let n = 400;
    let top = 40.0 * h;
    Ok((0..=n)
        .map(|i| {
            let z = top * i as f64 / n as f64;
            let rho = p0 * m::exp(-z / h) / (g * h);
            (z, rho / (mean_u * u), rho)
        })
        .collect())
}

/// Mean particle mass of the gases, u, from their sheet molar masses.
fn mean_mass_u(gases: &[GasShare]) -> Result<f64, String> {
    let (mut s, mut w) = (0.0, 0.0);
    for g in gases {
        let mm = sheet()
            .gas_molar_mass_kg_mol(&g.matter)
            .ok_or_else(|| format!("upper air gas {}: no molar mass on the sheet", g.matter))?;
        s += g.share * mm * 1000.0;
        w += g.share;
    }
    if w <= 0.0 {
        return Err("the upper air's gases have no share".into());
    }
    Ok(s / w)
}

/// The aurora from `power_w` coming down as electrons of `electron_ev` into
/// this air: each line its gases give off, its share, height and colour.
pub fn glow(
    power_w: f64,
    electron_ev: f64,
    air: &UpperAir,
    base: Option<(f64, f64, f64)>,
) -> Result<Glow, String> {
    let lv = levels(air, base)?;
    // Mass column above each level: exact for an exponential between
    // levels, the top level's own scale height above it.
    let k = lv.len();
    let mut col = vec![0.0; k];
    let (za, zb) = (lv[k - 2].0, lv[k - 1].0);
    let top_h = (zb - za) / m::ln(lv[k - 2].2 / lv[k - 1].2).max(1e-9);
    col[k - 1] = lv[k - 1].2 * top_h;
    for i in (0..k - 1).rev() {
        let (a, b) = (lv[i].2, lv[i + 1].2);
        let dz = lv[i + 1].0 - lv[i].0;
        let seg = if (a - b).abs() > 1e-12 * a {
            (a - b) * dz / m::ln(a / b)
        } else {
            a * dz
        };
        col[i] = col[i + 1] + seg;
    }
    let charge = sheet()
        .constant("e")
        .expect("the master data sheet has the elementary charge e");
    let range = run("sse.glow.electron_range", &[electron_ev * charge])?[0];
    // The electrons stop where the column above equals their range; the
    // energy they leave falls as the air thins above that.
    let stop = (0..k).find(|&i| col[i] <= range).unwrap_or(0);
    let q: Vec<f64> = (0..k)
        .map(|i| if i >= stop { lv[i].2 } else { 0.0 })
        .collect();
    let qsum: f64 = q.iter().sum();
    let share_of = |id: &str| {
        air.gases
            .iter()
            .filter(|g| g.matter == id)
            .map(|g| g.share)
            .sum::<f64>()
    };
    let eye = table().eye.as_ref();
    let mut emissions = vec![];
    for line in &table().lines {
        let Some(a) = &line.aurora else { continue };
        let made: f64 = a
            .from
            .iter()
            .map(|p| p.energy_share * share_of(&p.matter))
            .sum();
        if made <= 0.0 || qsum <= 0.0 {
            continue;
        }
        let mut e = Vec::with_capacity(k);
        for i in 0..k {
            let s = if q[i] > 0.0 {
                run(
                    "sse.glow.line_survival",
                    &[a.lifetime_s, a.quench_m3_s, lv[i].1],
                )?[0]
            } else {
                0.0
            };
            e.push(q[i] * s);
        }
        let esum: f64 = e.iter().sum();
        if esum <= 0.0 {
            continue;
        }
        let at = |frac: f64| {
            let mut acc = 0.0;
            for i in 0..k {
                acc += e[i];
                if acc >= frac * esum {
                    return lv[i].0;
                }
            }
            lv[k - 1].0
        };
        let share = made * esum / qsum;
        let colour = match eye {
            Some(eye) => colour_of_lines(eye, &[(line.at_m, 1.0)])?.linear_srgb,
            None => [0.0; 3],
        };
        let matter = a
            .from
            .iter()
            .filter(|p| share_of(&p.matter) > 0.0)
            .map(|p| p.matter.clone())
            .collect::<Vec<_>>()
            .join("+");
        emissions.push(Emission {
            line: line.id.clone(),
            name: line.name.clone(),
            matter,
            wavelength_m: line.at_m,
            power_w: power_w * share,
            share,
            peak_height_m: at(0.5),
            thickness_m: at(0.9) - at(0.1),
            colour,
        });
    }
    let colour = match eye {
        Some(eye) if !emissions.is_empty() => {
            let lines: Vec<(f64, f64)> = emissions
                .iter()
                .map(|e| (e.wavelength_m, e.power_w))
                .collect();
            let c = colour_of_lines(eye, &lines)?;
            (c.xyz.iter().sum::<f64>() > 0.0).then_some(c)
        }
        _ => None,
    };
    Ok(Glow {
        power_w,
        power_by_hemisphere: [power_w / 2.0, power_w / 2.0],
        electron_ev,
        stops_at_m: lv[stop].0,
        emissions,
        colour,
    })
}
