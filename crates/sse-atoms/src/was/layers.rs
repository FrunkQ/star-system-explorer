//! `layers` blocks as they were in Rust before ND3b moved them into formula
//! files; kept for the test that the formulas give the same faces and bits.

use crate::common::*;
use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::{Atom, AtomError, CostClass};

const ATKINS: &str = "Atkins, P. and de Paula, J. 2014, Atkins' Physical Chemistry, 10th edn, Oxford University Press, topic 5B (colligative properties; water's cryoscopic constant 1.86 K kg/mol)";
const RS: &str = "Robinson, R. A. and Stokes, R. H. 1959, Electrolyte Solutions, 2nd edn, Butterworths, appendix 8.10 (osmotic coefficient of NaCl about 0.93 near 2 mol/kg)";
const PW: &str = "Petrenko, V. F. and Whitworth, R. W. 1999, Physics of Ice, Oxford University Press (thermal conductivity of ice Ih, k = 651/T W/m). doi:10.1093/acprof:oso/9780198518945.001.0001";
const OS: &str = "Ojakangas, G. W. and Stevenson, D. J. 1989, Icarus 81, 220: thermal state of an ice shell on Europa (conductive shell with conductivity falling as 1/T) https://ui.adsabs.harvard.edu/abs/1989Icar...81..220O";

/// Molar mass of NaCl, kg/mol.
pub const NACL_MOLAR_MASS: f64 = 0.058_44;
/// Water's cryoscopic constant, K kg/mol.
pub const WATER_KF: f64 = 1.86;
/// Ice Ih conductivity coefficient: k = ICE_K / T, W/m.
pub const ICE_K: f64 = 651.0;

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let temp = |name: &str, kind: &str| port(name, kind, D::TEMPERATURE, None);
    vec![
        Def {
            id: "sse.ocean.brine_freezing_point",
            version: "1.0.0",
            inputs: vec![port(
                "salinity",
                "ocean.salinity_mass_fraction",
                D::NONE,
                Some((0.0, 0.2)),
            )],
            outputs: vec![temp("freezing_point", "ocean.freezing_point")],
            params: vec![
                param("van_t_hoff", D::NONE, 2.0, (1.0, 3.0), "ions per formula unit (2 for NaCl, 3 for MgCl2)"),
                param("osmotic_coefficient", D::NONE, 0.93, (0.5, 1.2), "non-ideality of the brine"),
                param("solute_molar_mass", dims::MOLAR_MASS, NACL_MOLAR_MASS, (0.01, 0.5), "molar mass of the salt"),
                param("pure_freezing_point", D::TEMPERATURE, sse_core::sheet::T0C, (200.0, 300.0), "freezing point of the pure solvent"),
            ],
            cost: CostClass::Trivial,
            references: &[ATKINS, RS],
            curious: "Salt water freezes colder than fresh water. A brine six times saltier than the sea stays liquid well below freezing, which is how oceans hide under ice.",
            science: "Freezing-point depression from the solute's molality, with the van 't Hoff factor and an osmotic coefficient for non-ideal brines. Good to about 10 percent up to 20 percent salt by mass; the NaCl eutectic (23 percent, 252 K) is the hard floor.",
            formula: "T_f = T₀ - i φ K_f b, b = w / (M_s (1 - w))",
            f: |i, p| {
                let w = i[0];
                if !(0.0..1.0).contains(&w) {
                    return Err(AtomError(format!("salinity {w} is not a mass fraction below 1")));
                }
                let molality = w / (p[2] * (1.0 - w));
                Ok(vec![p[3] - p[0] * p[1] * WATER_KF * molality])
            },
        }
        .build(),
        Def {
            id: "sse.heat.ice_shell_heat_flow",
            version: "1.0.0",
            inputs: vec![
                port("thickness", "layer.thickness", D::LENGTH, None),
                temp("surface_temperature", "layer.top_temperature"),
                temp("base_temperature", "layer.base_temperature"),
            ],
            outputs: vec![port("heat_flow", "layer.conductive_heat_flow", D::FLUX, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[PW, OS],
            curious: "Ice is a blanket. The heat leaking up from below sets how thick the ice can be before the water under it would freeze.",
            science: "Steady conduction through ice Ih whose conductivity falls as 1/T, integrated across the shell. The base sits at the freezing point of the water below. No convection: thick shells convect and carry more heat.",
            formula: "q = (651 / d) ln(T_base / T_surface)",
            f: |i, _| {
                let d = positive(i[0], "thickness")?;
                let ts = positive(i[1], "surface temperature")?;
                let tb = positive(i[2], "base temperature")?;
                Ok(vec![ICE_K / d * m::ln(tb / ts)])
            },
        }
        .build(),
        Def {
            id: "sse.heat.ice_shell_thickness",
            version: "1.0.0",
            inputs: vec![
                port("heat_flow", "layer.conductive_heat_flow", D::FLUX, None),
                temp("surface_temperature", "layer.top_temperature"),
                temp("base_temperature", "layer.base_temperature"),
            ],
            outputs: vec![port("thickness", "layer.thickness", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[PW, OS],
            curious: "Given how much heat comes up from inside, how thick the ice lid over an ocean settles.",
            science: "The conductive ice-shell relation solved for thickness. If the base is not warmer than the surface there is no ocean to cover.",
            formula: "d = (651 / q) ln(T_base / T_surface)",
            f: |i, _| {
                let q = positive(i[0], "heat flow")?;
                let ts = positive(i[1], "surface temperature")?;
                let tb = positive(i[2], "base temperature")?;
                if tb <= ts {
                    return Err(AtomError("base is not warmer than the surface: no conductive shell".into()));
                }
                Ok(vec![ICE_K / q * m::ln(tb / ts)])
            },
        }
        .build(),
        Def {
            id: "sse.interior.hydrostatic_pressure",
            version: "1.0.0",
            inputs: vec![
                port("density", "layer.density", D::DENSITY, None),
                port("g", "body.surface_gravity", D::ACCELERATION, None),
                port("depth", "layer.thickness", D::LENGTH, None),
            ],
            outputs: vec![port("pressure", "layer.pressure_increase", D::PRESSURE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &["Turcotte, D. L. and Schubert, G. 2014, Geodynamics, 3rd edn, Cambridge University Press, section 2.2. doi:10.1017/CBO9780511843877"],
            curious: "The deeper you go, the more weight sits on top of you. Ten kilometres of water presses about a thousand times harder than the air.",
            science: "Pressure added by a layer of constant density under constant gravity. Fine for oceans and crusts much thinner than the radius; deep interiors need gravity that falls with depth.",
            formula: "ΔP = ρ g h",
            f: |i, _| Ok(vec![positive(i[0], "density")? * positive(i[1], "gravity")? * i[2].max(0.0)]),
        }
        .build(),
    ]
}
