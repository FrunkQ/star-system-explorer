//! MP6: where a body's elements are (matter pack design section 5, steps 3
//! and 4). The blocks are Equations in `packs/base-physics/formulas/
//! composition.json`; these are the same sums for the composition chain,
//! which runs them once per element (a test holds the two together):
//!
//! - `sse.composition.share_in_material`: an element's share of the formation
//!   material it rides in (CI chondrite share times the star's ratio to the
//!   Sun, normalised over the material), with the oxygen its oxide binds;
//! - `sse.composition.separation_strength`: how far a world separated into
//!   metal and silicate, from the heat it had to melt (accretion plus Al-26),
//!   a process strength, never a class;
//! - `sse.composition.core_share`: the share of an element that sank with the
//!   metal, by mass balance with its metal-silicate partition;
//! - `sse.composition.crust_share`: the share of the silicate's element that
//!   the crust holds, by its enrichment over the mantle;
//! - `sse.composition.crust_fraction`: the crust's share of the silicate.
//!
//! Partitions and enrichments are read from Earth's measured core, mantle
//! and crust (the sheet's `earth_mass_fraction`): single typical values, not
//! functions of pressure and oxygen (fudge register).

use sse_core::math as m;

pub const MS95: &str = "McDonough, W. F. and Sun, S.-s. 1995, Chemical Geology 120, 223: the composition of the Earth (CI chondrite and pyrolite). doi:10.1016/0009-2541(94)00140-4";
pub const MD03: &str = "McDonough, W. F. 2003, Treatise on Geochemistry 2, 547: compositional model for the Earth's core (table 4). doi:10.1016/B0-08-043751-6/02015-6";
pub const RG03: &str = "Rudnick, R. L. and Gao, S. 2003, Treatise on Geochemistry 3, 1: composition of the continental crust. doi:10.1016/B0-08-043751-6/03016-4";
pub const L03: &str = "Lodders, K. 2003, ApJ 591, 1220: solar system abundances and condensation temperatures of the elements (host phases, table 8). doi:10.1086/375492";
pub const ET12: &str = "Elkins-Tanton, L. T. 2012, Annual Review of Earth and Planetary Sciences 40, 113: magma oceans in the inner solar system (accretion and Al-26 energy against the energy to melt). doi:10.1146/annurev-earth-042711-105503";
pub const PO14: &str = "Palme, H. and O'Neill, H. St. C. 2014, Treatise on Geochemistry (2nd ed.) 3, 1: cosmochemical estimates of mantle composition (CI H and O). doi:10.1016/B978-0-08-095975-7.00201-1";

/// An element's mass share of its material, and the oxygen its oxide binds
/// with it, from the material's summed weight `total` (the sum of
/// `ci · ratio · (1 + oxygen)` over the material's riders).
pub fn share_in_material(ci: f64, ratio: f64, oxygen_per_mass: f64, total: f64) -> (f64, f64) {
    if total <= 0.0 || ci <= 0.0 || ratio <= 0.0 {
        return (0.0, 0.0);
    }
    let w = ci * ratio;
    (w / total, w * oxygen_per_mass.max(0.0) / total)
}

/// How far a world separated, 0 to 1: the heat per kilogram it had to melt
/// with (three fifths of GM/R from accretion, plus Al-26 left after it
/// formed) over the energy to melt rock.
pub fn separation_strength(
    g: f64,
    mass_kg: f64,
    radius_m: f64,
    al26_j_kg: f64,
    e_melt: f64,
) -> f64 {
    if radius_m <= 0.0 || e_melt <= 0.0 {
        return 0.0;
    }
    let e = 0.6 * g * mass_kg / radius_m + al26_j_kg.max(0.0);
    (e / e_melt).clamp(0.0, 1.0)
}

/// The share of an element in the core: mass balance between metal (share
/// `metal` of the separated body) and silicate with partition `d` (core
/// concentration over silicate), times how far the world separated.
pub fn core_share(d: f64, metal: f64, strength: f64) -> f64 {
    let fm = metal.clamp(0.0, 1.0);
    if d <= 0.0 || fm <= 0.0 {
        return 0.0;
    }
    let s = strength.clamp(0.0, 1.0);
    s * fm * d / (fm * d + (1.0 - fm))
}

/// The share of the silicate's element held in the crust: crust share `fc`
/// of the silicate with enrichment `e` (crust concentration over mantle),
/// times how far the world separated.
pub fn crust_share(e: f64, fc: f64, strength: f64) -> f64 {
    let fc = fc.clamp(0.0, 1.0);
    if e <= 0.0 || fc <= 0.0 {
        return 0.0;
    }
    let s = strength.clamp(0.0, 1.0);
    s * fc * e / (fc * e + (1.0 - fc))
}

/// The crust's share of the silicate: a shell of `thickness` and `density`
/// at the surface over the silicate's mass.
pub fn crust_fraction(radius_m: f64, silicate_kg: f64, thickness_m: f64, density: f64) -> f64 {
    if silicate_kg <= 0.0 || radius_m <= 0.0 {
        return 0.0;
    }
    let h = thickness_m.clamp(0.0, radius_m);
    let inner = radius_m - h;
    let shell = 4.0 / 3.0 * m::PI * (radius_m * radius_m * radius_m - inner * inner * inner);
    (shell * density / silicate_kg).clamp(0.0, 0.5)
}
