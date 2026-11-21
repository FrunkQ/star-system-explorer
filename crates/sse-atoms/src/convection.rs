//! Surface convection of stars and other bodies with a churning outer layer
//! (light track, stellar surfaces): granules, supergranules, how fast they
//! churn and how long each lives. One process for any body: the sizes come
//! from the photosphere's pressure scale height (the same block as an
//! atmosphere's, `sse.atmosphere.scale_height`), so a red supergiant gets a
//! few huge cells and a red dwarf many small ones, with no type gates.
//! `sse-activity` reads these for the star's surface and its magnetic
//! network; the renderer animates cells from the sizes and lifetimes.

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const TRAMP13: &str = "Trampedach, R., Asplund, M., Collet, R., Nordlund, Å. and Stein, R. F. 2013, A grid of three-dimensional stellar atmosphere models of solar metallicity. I. General properties, granulation, and atmospheric expansion, ApJ 769, 18: granule size scales with the surface pressure scale height across the grid. doi:10.1088/0004-637X/769/1/18";
const FREY02: &str = "Freytag, B., Steffen, M. and Dorch, B. 2002, Spots on the surface of Betelgeuse: results from new 3D stellar convection models, Astron. Nachr. 323, 213: red supergiants have only a few giant convection cells across their faces. doi:10.1002/1521-3994(200208)323:3/4<213::AID-ASNA213>3.0.CO;2-H";
const SCHW75: &str = "Schwarzschild, M. 1975, On the scale of photospheric convection in red giants and supergiants, ApJ 195, 137: cell size follows the pressure scale height, so giants' cells are huge. doi:10.1086/153313";
const NSA09: &str = "Nordlund, Å., Stein, R. F. and Asplund, M. 2009, Solar surface convection, Living Rev. Solar Phys. 6, 2: granules about 1 to 2 Mm across living minutes, with upflows of a few km/s near the surface. doi:10.12942/lrsp-2009-2";
const RR10: &str = "Rieutord, M. and Rincon, F. 2010, The Sun's supergranulation, Living Rev. Solar Phys. 7, 2: cells about 30 Mm across, lifetimes one to two days, horizontal flows about 300 to 400 m/s, outlined by the magnetic network. doi:10.12942/lrsp-2010-2";
const GN89: &str = "Gray, D. F. and Nagel, T. 1989, The granulation boundary in the H-R diagram, ApJ 341, 421: no granulation in stars hotter than about early F. doi:10.1086/167505";

const T_SUN: f64 = 5772.0;
const G_SUN: f64 = GM_SUN / (R_SUN * R_SUN);
const K_B: f64 = 1.380_649e-23;
const AMU: f64 = 1.660_539_066_60e-27;

/// Granule size, m: a multiple of the pressure scale height.
pub fn granule_size(scale_height_m: f64, factor: f64) -> f64 {
    factor * scale_height_m
}

/// Speed of the churning gas near the surface, m/s. Convective flux
/// F = σT⁴ carried as ρv³ with photospheric density ρ ∝ g μ / T (constant
/// opacity) gives v ∝ T^(5/3) g^(-1/3); normalised to the Sun and never
/// faster than sound.
pub fn convective_speed(teff_k: f64, g: f64, v_sun: f64, mu: f64) -> f64 {
    let v = v_sun * m::pow(teff_k / T_SUN, 5.0 / 3.0) * m::cbrt(G_SUN / g);
    v.min(sound_speed(teff_k, mu))
}

/// Adiabatic sound speed of a monatomic ideal gas, m/s.
pub fn sound_speed(t_k: f64, mu: f64) -> f64 {
    m::sqrt(5.0 / 3.0 * K_B * t_k / (mu * AMU))
}

/// A cell's life: `k` times its size over the speed that drives it.
pub fn cell_lifetime(size_m: f64, speed_m_s: f64, k: f64) -> f64 {
    k * size_m / speed_m_s
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    vec![
        Def {
            id: "sse.convection.granule_size",
            version: "1.0.0",
            inputs: vec![port("h", "atmosphere.scale_height", D::LENGTH, None)],
            outputs: vec![port("size", "convection.cell_size", D::LENGTH, None)],
            params: vec![param("factor", D::NONE, 10.0, (3.0, 30.0), "granule diameter in pressure scale heights (Sun: about 1.4 Mm over 0.14 Mm)")],
            cost: CostClass::Trivial,
            references: &[TRAMP13, SCHW75, NSA09],
            curious: "The surface of a cool star is a boiling pan of bright cells. On the Sun each is about the size of France; on a red supergiant a few cells cover the whole star.",
            science: "Hot gas rises, spreads and sinks over about ten pressure scale heights: thin hot surfaces under strong gravity make small cells, cool distended ones make huge cells. The scale height is the same block an atmosphere uses, read at the photosphere.",
            formula: "d = f H,  H = k T / (μ m_u g)",
            f: |i, p| Ok(vec![granule_size(positive(i[0], "scale height")?, p[0])]),
        }
        .build(),
        Def {
            id: "sse.convection.speed",
            version: "1.0.0",
            inputs: vec![
                port("teff", "star.effective_temperature", D::TEMPERATURE, None),
                port("g", "body.surface_gravity", D::ACCELERATION, None),
            ],
            outputs: vec![port("speed", "convection.speed", D::VELOCITY, None)],
            params: vec![
                param("v_sun", D::VELOCITY, 2000.0, (500.0, 5000.0), "the Sun's near-surface convective speed, m/s (a few km/s; Nordlund et al. 2009)"),
                param("mu", D::NONE, 1.3, (0.5, 3.0), "mean molecular weight of the photosphere (neutral solar mix about 1.3)"),
            ],
            cost: CostClass::Trivial,
            references: &[NSA09, FREY02],
            curious: "The hotter the surface and the weaker its gravity, the faster it boils. On giant stars the gas churns nearly as fast as sound.",
            science: "The flux the surface must carry, σT⁴, is moved by rising gas as about ρv³. With photospheric density set by gravity over a fixed opacity, ρ ∝ gμ/T, so v ∝ T^(5/3) g^(−1/3), normalised to the Sun and capped at the sound speed.",
            formula: "v = v☉ (T/T☉)^(5/3) (g☉/g)^(1/3),  v ≤ √(5kT/(3μ m_u))",
            f: |i, p| Ok(vec![convective_speed(positive(i[0], "temperature")?, positive(i[1], "gravity")?, p[0], p[1])]),
        }
        .build(),
        Def {
            id: "sse.convection.cell_lifetime",
            version: "1.0.0",
            inputs: vec![
                port("size", "convection.cell_size", D::LENGTH, None),
                port("speed", "convection.speed", D::VELOCITY, None),
            ],
            outputs: vec![port("life", "convection.cell_lifetime", D::TIME, None)],
            params: vec![param("k", D::NONE, 0.5, (0.1, 3.0), "life over the time to cross the cell (fixed so the Sun's granules live about 6 minutes)")],
            cost: CostClass::Trivial,
            references: &[NSA09],
            curious: "Each solar granule lives only minutes before it fades and a new one takes its place. Giant cells on a supergiant last months to years.",
            science: "A cell lasts about the time its gas takes to cross it.",
            formula: "t = k d / v",
            f: |i, p| Ok(vec![cell_lifetime(positive(i[0], "cell size")?, positive(i[1], "speed")?, p[0])]),
        }
        .build(),
        Def {
            id: "sse.convection.supergranule_size",
            version: "1.0.0",
            inputs: vec![port("granule", "convection.cell_size", D::LENGTH, None)],
            outputs: vec![port("size", "convection.supergranule_size", D::LENGTH, None)],
            params: vec![param("ratio", D::NONE, 25.0, (5.0, 60.0), "supergranule over granule diameter (Sun: about 30 Mm over 1.2 Mm)")],
            cost: CostClass::Trivial,
            references: &[RR10, GN89],
            curious: "Granules sit inside much bigger slow cells whose edges sweep the star's magnetic field into a glowing net. Spots and flares rise through that net.",
            science: "Supergranulation's origin is still debated; this block scales it with the granules, as the Sun shows, until better theory or observation of other stars replaces it.",
            formula: "D = r d",
            f: |i, p| Ok(vec![positive(i[0], "granule size")? * p[0]]),
        }
        .build(),
        Def {
            id: "sse.convection.supergranule_lifetime",
            version: "1.0.0",
            inputs: vec![
                port("size", "convection.supergranule_size", D::LENGTH, None),
                port("speed", "convection.speed", D::VELOCITY, None),
            ],
            outputs: vec![port("life", "convection.supergranule_lifetime", D::TIME, None)],
            params: vec![param("share", D::NONE, 0.2, (0.05, 1.0), "supergranule flow over the granule speed (Sun: about 0.4 over 2 km/s)")],
            cost: CostClass::Trivial,
            references: &[RR10],
            curious: "The Sun's big network cells last a day or two.",
            science: "The slow horizontal flow crosses a supergranule once in its life.",
            formula: "T = D / (s v)",
            f: |i, p| Ok(vec![cell_lifetime(positive(i[0], "supergranule size")?, positive(i[1], "speed")? * p[0], 1.0)]),
        }
        .build(),
    ]
}
