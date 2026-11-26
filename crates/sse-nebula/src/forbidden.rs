//! Forbidden lines in glowing gas (O4 part 1): oxygen and sulphur ions
//! knocked into slow-to-shine levels by the nebula's free electrons, each
//! line from the layer of gas where its ion lives. The atomic data are the
//! light table's `nebular` rows; the physics is the `sse.nebular.*` blocks
//! (`packs/base-physics/formulas/nebular.json`) and `sse.glow.line_survival`
//! for the knocks that take the energy away before it shines.

use sse_atoms::nebula as nb;
use sse_core::math as m;
use sse_flow::Atom;
use sse_light::table::LightTable;
use sse_light::{LightDescription, LineEmission, Reference, Zone};
use std::sync::OnceLock;

/// Temperature of the free electrons in every glowing nebula, K: held at the
/// value photoionised gas settles near rather than worked from its heating
/// and cooling (fudge register F64).
pub const ELECTRON_T_K: f64 = 1.0e4;

/// How the glowing gas is laid out round the lighting stars.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Layout {
    /// A filled sphere round the stars (a star-forming glow).
    Sphere,
    /// A thin shell at this radius, m (a planetary nebula).
    Shell { radius_m: f64 },
}

/// Radial cells a filled sphere is cut into for an ion in local balance.
const CELLS: usize = 32;
/// Steps across a cross-section fit's range when the stripping rate is
/// summed over the starlight's photons.
const ENERGY_STEPS: usize = 160;

fn run(id: &str, inputs: &[f64]) -> Result<Vec<f64>, String> {
    sse_atoms::formulas::loaded()
        .iter()
        .find(|a| a.decl().id == id)
        .ok_or_else(|| format!("base formula block {id} is missing"))?
        .eval(inputs, &[])
        .map_err(|e| format!("{id}: {e}"))
}

pub(crate) fn light_table() -> &'static LightTable {
    table()
}

fn table() -> &'static LightTable {
    static T: OnceLock<LightTable> = OnceLock::new();
    T.get_or_init(LightTable::base)
}

fn sheet() -> &'static sse_reference::Reference {
    static S: OnceLock<sse_reference::Reference> = OnceLock::new();
    S.get_or_init(sse_reference::Reference::base)
}

/// Atoms of an element per hydrogen atom, from the master data sheet's
/// protosolar abundances.
pub fn per_hydrogen(element: &str) -> Result<f64, String> {
    let a = |id: &str| -> Result<f64, String> {
        sheet()
            .row("elements", id)
            .and_then(|r| r.get("abundance_protosolar"))
            .and_then(|v| v.get("v"))
            .and_then(|v| v.as_f64())
            .ok_or_else(|| format!("the sheet has no protosolar abundance for {id}"))
    };
    Ok(a(element)? / a("h")?)
}

/// Adds every forbidden line the light table knows to `light`, for gas of
/// `n_h` hydrogen atoms per m³ that balances `recombinations` a second
/// against the ionising light of `stars` (effective temperature K, radius m).
/// Returns a sentence for the Why.
pub fn add_forbidden_lines(
    light: &mut LightDescription,
    stars: &[(f64, f64)],
    n_h: f64,
    recombinations: f64,
    layout: Layout,
) -> Result<Option<String>, String> {
    if !(n_h > 0.0 && recombinations > 0.0) {
        return Ok(None);
    }
    let ev = sse_core::sheet::E;
    let (h, c) = (sse_core::sheet::H, sse_core::sheet::C);
    let photons = |e_ev: f64| -> f64 {
        stars
            .iter()
            .map(|(t, r)| nb::ionising_photons(*t, *r, e_ev * ev))
            .sum()
    };
    // Electrons from hydrogen and once-ionised helium.
    let n_e = n_h * (1.0 + per_hydrogen("he")?);
    let volume = recombinations / (nb::ALPHA_B * n_h * n_h);
    let h_edge_ev = nb::E_ION_H / ev;
    let depth = |f: f64| match layout {
        Layout::Sphere => m::cbrt(f),
        Layout::Shell { .. } => f,
    };
    // Cells of glowing gas: (depth at the middle, radius, volume).
    let cells: Vec<(f64, f64, f64)> = match layout {
        Layout::Sphere => {
            let big_r = m::cbrt(3.0 * volume / (4.0 * m::PI));
            (0..CELLS)
                .map(|i| {
                    let (a, b) = (i as f64 / CELLS as f64, (i + 1) as f64 / CELLS as f64);
                    let mid = 0.5 * (a + b);
                    let v = 4.0 / 3.0 * m::PI * big_r * big_r * big_r * (b * b * b - a * a * a);
                    (mid, mid * big_r, v)
                })
                .collect()
        }
        Layout::Shell { radius_m } => vec![(0.5, radius_m, volume)],
    };
    // The gas that takes photons of this energy: the one whose edge lies
    // highest below it.
    let takers = table()
        .photon_takers
        .as_ref()
        .ok_or("the light table has no photon_takers")?;
    let taker_share = |e: f64| -> Result<f64, String> {
        let t = takers
            .list
            .iter()
            .filter(|t| t.from_ev <= e)
            .max_by(|a, b| a.from_ev.total_cmp(&b.from_ev))
            .ok_or_else(|| format!("no gas takes photons of {e} eV"))?;
        per_hydrogen(&t.matter)
    };
    let mut added = Vec::new();
    for def in &table().lines {
        let Some(nl) = &def.nebular else { continue };
        let Some(matter) = &def.matter else { continue };
        let share = |e: f64| -> Result<f64, String> {
            if e <= h_edge_ev {
                return Ok(1.0);
            }
            Ok(run(
                "sse.nebular.ion_zone_share",
                &[photons(e), recombinations, taker_share(e)?],
            )?[0])
        };
        // Each cell's share of the element in this ion; the zone follows.
        let (fills, zone): (Vec<f64>, Zone) = if let Some(b) = &nl.balance {
            if nl.makes_ev > h_edge_ev {
                return Err(format!(
                    "{}: a balanced ion must exist wherever hydrogen is ionised",
                    def.id
                ));
            }
            // Photons a second able to strip it, each by its cross-section.
            let f = &b.fit;
            let (lo, hi) = (nl.strips_ev, f.max_ev);
            let mut strip = 0.0;
            for i in 0..ENERGY_STEPS {
                let e0 = lo * m::pow(hi / lo, i as f64 / ENERGY_STEPS as f64);
                let e1 = lo * m::pow(hi / lo, (i + 1) as f64 / ENERGY_STEPS as f64);
                let sigma = run(
                    "sse.nebular.photoionisation_cross_section",
                    &[
                        m::sqrt(e0 * e1) * ev,
                        f.e0_ev * ev,
                        f.sigma0_m2,
                        f.ya,
                        f.p,
                        f.yw,
                        f.y0,
                        f.y1,
                    ],
                )?[0];
                strip += (photons(e0) - photons(e1)) * sigma;
            }
            let mut fills = Vec::with_capacity(cells.len());
            for (_, r, _) in &cells {
                let rate = strip / (4.0 * m::PI * r * r);
                fills.push(
                    run(
                        "sse.nebular.ion_balance",
                        &[rate, b.recombination_m3_s, n_e],
                    )?[0],
                );
            }
            // The zone starts where a tenth of the ion's gas lies nearer the star.
            let total: f64 = fills.iter().zip(&cells).map(|(x, c)| x * c.2).sum();
            let mut seen = 0.0;
            let mut inner = 0.0;
            for (x, c) in fills.iter().zip(&cells) {
                seen += x * c.2;
                if seen >= 0.1 * total {
                    inner = (c.0 - 0.5 / cells.len() as f64).max(0.0);
                    break;
                }
            }
            if matches!(layout, Layout::Shell { .. }) {
                inner = 0.0;
            }
            (fills, Zone { inner, outer: 1.0 })
        } else {
            let (made, stripped) = (share(nl.makes_ev)?, share(nl.strips_ev)?);
            let fill = (made - stripped).max(0.0);
            let z = Zone {
                inner: depth(stripped),
                outer: depth(made),
            };
            // Hard-edged layer: the cells' volumes add to the layer's share.
            (vec![fill], z)
        };
        let gas: f64 = if nl.balance.is_some() {
            fills.iter().zip(&cells).map(|(x, c)| x * c.2).sum()
        } else {
            fills[0] * volume
        };
        if gas <= 0.0 {
            continue;
        }
        let q = run(
            "sse.nebular.collisional_excitation",
            &[
                ELECTRON_T_K,
                if nl.upper_cm > 0.0 {
                    0.01 / nl.upper_cm
                } else {
                    def.at_m
                },
                nl.upsilon,
                nl.lower_weight,
                nl.upper_weight,
            ],
        )?;
        let shines = if nl.rate_s > 0.0 {
            run("sse.glow.line_survival", &[1.0 / nl.rate_s, q[1], n_e])?[0]
        } else {
            1.0
        };
        let n_ion = per_hydrogen(matter)? * n_h;
        let power = n_ion * n_e * q[0] * nl.share * shines * h * c / def.at_m * gas;
        light.lines.push(LineEmission {
            line: def.id.clone(),
            power_w: power,
            cause: format!("electron knocks on {} ions", def.species),
            zone: Some(zone),
        });
        added.push(def.name.clone());
        if let Some(b) = &nl.balance {
            let r = Reference::parse(&b.source);
            if !light.references.contains(&r) {
                light.references.push(r);
            }
        }
        let r = Reference::parse(&nl.source);
        let t = Reference::parse(&takers.source);
        if !light.references.contains(&t) {
            light.references.push(t);
        }
        if !light.references.contains(&r) {
            light.references.push(r);
        }
    }
    if added.is_empty() {
        return Ok(None);
    }
    for id in [
        "sse.nebular.collisional_excitation",
        "sse.nebular.ion_zone_share",
        "sse.nebular.photoionisation_cross_section",
        "sse.nebular.ion_balance",
    ] {
        if let Some(a) = sse_atoms::formulas::loaded()
            .iter()
            .find(|a| a.decl().id == id)
        {
            light.references.extend(a.decl().references.iter().cloned());
        }
    }
    Ok(Some(format!(
        "Electrons at about {:.0} K knock oxygen and sulphur ions into levels that shine only slowly: {}. Each comes from the layer where the star's light keeps its ion, the harder-to-make ions nearest the star.",
        ELECTRON_T_K,
        added.join(", ")
    )))
}
