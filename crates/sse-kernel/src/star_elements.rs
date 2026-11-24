//! MP7, a star's elements (matter pack design 8.1 and package MP7): any
//! element's abundance at a star, from what the star inherited.
//!
//! Each element row on the master data sheet says how the Sun's birth cloud
//! came by it (`origin.shares`: big bang, cosmic rays, low-mass stars,
//! massive stars, white dwarfs, neutron-star mergers, human-made). One
//! Equation block in `packs/base-cosmos` (`element_sources`) works out how
//! much each source made for this star against the Sun's, from its [Fe/H],
//! [alpha/Fe], C/O and r-process ratio. An element's ratio is its shares
//! weighted by those scales: a linear ratio by mass per hydrogen, 1 for
//! every element of a star that inherited exactly what the Sun did.

use sse_formation::inherit::Inheritance;
use sse_logic::equation::Parsed;
use std::collections::BTreeMap;
use std::sync::OnceLock;

struct Model {
    eq: Parsed,
    c_o_sun: f64,
}

fn model() -> &'static Result<Model, String> {
    static CELL: OnceLock<Result<Model, String>> = OnceLock::new();
    CELL.get_or_init(|| {
        let v: serde_json::Value = serde_json::from_str(sse_cosmos::model::BASE_COSMOS_JSON)
            .map_err(|e| format!("base-cosmos: {e}"))?;
        let other = &v["provides"]["other"];
        let eq = Parsed::new(Some(&other["element_sources"]["equation"]))?;
        let c_o_sun = other["patch_model"]["c_o"]["solar"]
            .as_f64()
            .ok_or("base-cosmos: patch_model.c_o.solar")?;
        Ok(Model { eq, c_o_sun })
    })
}

/// Each element's origin shares from the sheet, by element id.
pub fn origin_shares(sheet: &sse_reference::Reference) -> BTreeMap<String, BTreeMap<String, f64>> {
    let mut out = BTreeMap::new();
    for (id, row) in sheet.tables.get("elements").into_iter().flatten() {
        let shares: BTreeMap<String, f64> = row
            .get("origin")
            .and_then(|o| o.get("shares"))
            .and_then(|s| s.as_object())
            .into_iter()
            .flatten()
            .filter_map(|(k, v)| Some((k.clone(), v.as_f64()?)))
            .collect();
        if !shares.is_empty() {
            out.insert(id.clone(), shares);
        }
    }
    out
}

/// How much each source made for this star against the Sun, by the
/// sheet's source ids (the `element_sources` block; big bang 1, human-made 0).
pub fn source_scales(
    inh: &Inheritance,
    shares: &BTreeMap<String, BTreeMap<String, f64>>,
) -> Result<BTreeMap<String, f64>, String> {
    let m = model().as_ref().map_err(Clone::clone)?;
    let share = |el: &str, src: &str| {
        shares
            .get(el)
            .and_then(|s| s.get(src))
            .copied()
            .unwrap_or(0.0)
    };
    let x: Vec<f64> =
        m.eq.inputs
            .iter()
            .map(|n| match n.as_str() {
                "feh" => Ok(inh.feh),
                "alpha_fe" => Ok(inh.alpha_fe),
                "c_o" => Ok(inh.c_o),
                "c_o_sun" => Ok(m.c_o_sun),
                "r_process_rel" => Ok(inh.r_process_rel),
                "fe_massive" => Ok(share("fe", "massive_stars")),
                "c_massive" => Ok(share("c", "massive_stars")),
                other => Err(format!("element_sources: no value for input {other}")),
            })
            .collect::<Result<_, _>>()?;
    let out = m.eq.run_numbers(&x)?;
    let mut s = BTreeMap::new();
    s.insert("big_bang".to_string(), 1.0);
    s.insert("human_made".to_string(), 0.0);
    for (name, v) in m.eq.outputs.iter().zip(out) {
        let src = match name.as_str() {
            "s_massive" => "massive_stars",
            "s_white_dwarfs" => "white_dwarfs",
            "s_low_mass" => "low_mass_stars",
            "s_cosmic_rays" => "cosmic_rays",
            "s_mergers" => "neutron_star_mergers",
            other => other,
        };
        s.insert(src.to_string(), v);
    }
    Ok(s)
}

/// Every element's abundance at the star against the Sun's (by mass per
/// hydrogen): its origin shares weighted by the source scales. An element
/// with no shares, or a source the block does not scale, reads as the Sun's.
pub fn star_element_ratios(
    inh: &Inheritance,
    sheet: &sse_reference::Reference,
) -> Result<BTreeMap<String, f64>, String> {
    let shares = origin_shares(sheet);
    let scales = source_scales(inh, &shares)?;
    Ok(shares
        .iter()
        .map(|(el, sh)| {
            let r = sh
                .iter()
                .map(|(src, f)| f * scales.get(src).copied().unwrap_or(1.0))
                .fold(0.0, |a, b| a + b);
            (el.clone(), r)
        })
        .collect())
}
