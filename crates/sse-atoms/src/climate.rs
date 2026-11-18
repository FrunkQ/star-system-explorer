//! Light arriving, and the temperatures it sets; atmosphere basics (P1 first wave).

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const CK: &str = "Catling, D. C. and Kasting, J. F. 2017, Atmospheric Evolution on Inhabited and Lifeless Worlds, Cambridge University Press, chapters 2 and 4. doi:10.1017/9781139020558";
const SEAGER: &str = "Seager, S. 2010, Exoplanet Atmospheres, Princeton University Press, section 3.1 (heat redistribution factor)";
const PIERRE: &str = "Pierrehumbert, R. T. 2010, Principles of Planetary Climate, Cambridge University Press, sections 2.7 and 8.7. doi:10.1017/CBO9780511780783";

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let temp = |name: &str, kind: &str| port(name, kind, D::TEMPERATURE, None);
    vec![
        Def {
            id: "sse.climate.insolation",
            version: "1.0.0",
            inputs: vec![
                port("luminosity", "star.luminosity", D::POWER, None),
                port("distance", "orbit.semi_major_axis", D::LENGTH, None),
            ],
            outputs: vec![port("flux", "planet.insolation", D::FLUX, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[CK],
            curious: "Starlight spreads out as it travels: twice as far away, a quarter of the light.",
            science: "Inverse-square law for an isotropic source. At the semi-major axis this is close to the orbit-averaged flux for low eccentricity (the exact average is larger by 1/√(1 - e²)).",
            formula: "S = L / (4π d²)",
            f: |i, _| {
                let d = positive(i[1], "distance")?;
                Ok(vec![positive(i[0], "luminosity")? / (4.0 * m::PI * d * d)])
            },
        }
        .build(),
        Def {
            id: "sse.climate.equilibrium_temperature",
            version: "1.0.0",
            inputs: vec![
                port("flux", "planet.insolation", D::FLUX, None),
                port("albedo", "planet.bond_albedo", D::NONE, Some((0.0, 1.0))),
            ],
            outputs: vec![temp("teq", "planet.equilibrium_temperature")],
            params: vec![param(
                "redistribution",
                D::NONE,
                0.25,
                (0.25, 2.0 / 3.0),
                "1/4 when heat spreads round the whole world, 1/2 for the day side only, 2/3 with no spreading at the substellar point",
            )],
            cost: CostClass::Trivial,
            references: &[CK, SEAGER],
            curious: "How warm a world would be if it were a bare ball that only absorbs and glows. Earth's comes out at a frozen −18 °C; the air makes up the rest.",
            science: "Balance of absorbed starlight against black-body emission. The redistribution factor sets how much of the sphere shares the absorbed heat.",
            formula: "T_eq = (f S (1 - A) / σ)^¼",
            f: |i, p| {
                let s = positive(i[0], "flux")?;
                let absorbed = p[0] * s * (1.0 - i[1]);
                if absorbed < 0.0 {
                    return Err(sse_flow::AtomError("albedo above 1".into()));
                }
                Ok(vec![m::pow(absorbed / STEFAN_BOLTZMANN, 0.25)])
            },
        }
        .build(),
        Def {
            id: "sse.climate.grey_surface_temperature",
            version: "1.0.0",
            inputs: vec![
                temp("teq", "planet.equilibrium_temperature"),
                port("optical_depth", "atmosphere.infrared_optical_depth", D::NONE, Some((0.0, 100.0))),
            ],
            outputs: vec![temp("surface", "planet.surface_temperature")],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[CK, PIERRE],
            curious: "Air that lets sunlight in but holds heat back keeps the ground warmer than the bare-ball figure. The thicker that blanket, the warmer.",
            science: "Grey radiative-equilibrium atmosphere in the Eddington approximation, with all sunlight absorbed at the ground and one infrared optical depth τ. Earth needs τ ≈ 0.83; Venus about 125. It ignores convection, which in reality caps the lapse rate, so it overstates hot thick atmospheres.",
            formula: "T_s = T_eq (1 + ¾ τ)^¼",
            f: |i, _| {
                let t = positive(i[0], "equilibrium temperature")?;
                if i[1] < 0.0 {
                    return Err(sse_flow::AtomError("optical depth is negative".into()));
                }
                Ok(vec![t * m::pow(1.0 + 0.75 * i[1], 0.25)])
            },
        }
        .build(),
        Def {
            id: "sse.atmosphere.scale_height",
            version: "1.0.0",
            inputs: vec![
                temp("temperature", "atmosphere.temperature"),
                port("mean_molecular_weight", "atmosphere.mean_molecular_weight", D::NONE, Some((1.0, 100.0))),
                port("g", "body.surface_gravity", D::ACCELERATION, None),
            ],
            outputs: vec![port("scale_height", "atmosphere.scale_height", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[CK],
            curious: "How quickly the air thins as you climb. Every scale height up, the pressure drops to about a third.",
            science: "Isothermal hydrostatic atmosphere of an ideal gas. Mean molecular weight in atomic mass units (Earth air 28.97).",
            formula: "H = k T / (μ m_u g)",
            f: |i, _| {
                let t = positive(i[0], "temperature")?;
                let mu = positive(i[1], "mean molecular weight")?;
                let g = positive(i[2], "gravity")?;
                Ok(vec![k::BOLTZMANN * t / (mu * k::AMU * g)])
            },
        }
        .build(),
        Def {
            id: "sse.atmosphere.surface_pressure",
            version: "1.0.0",
            inputs: vec![
                port("atmosphere_mass", "atmosphere.mass", D::MASS, None),
                port("g", "body.surface_gravity", D::ACCELERATION, None),
                port("radius", "body.radius", D::LENGTH, None),
            ],
            outputs: vec![port("pressure", "atmosphere.surface_pressure", D::PRESSURE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[CK],
            curious: "Air pressure is the weight of all the air above you, spread over the ground.",
            science: "Hydrostatic surface pressure of a thin atmosphere (thickness much less than the radius), with gravity taken as constant with height.",
            formula: "P = M_atm g / (4π R²)",
            f: |i, _| {
                let r = positive(i[2], "radius")?;
                Ok(vec![positive(i[0], "atmosphere mass")? * positive(i[1], "gravity")? / (4.0 * m::PI * r * r)])
            },
        }
        .build(),
        Def {
            id: "sse.atmosphere.jeans_parameter",
            version: "1.0.0",
            inputs: vec![
                port("mass", "body.mass", D::MASS, None),
                port("exobase_radius", "atmosphere.exobase_radius", D::LENGTH, None),
                temp("exobase_temperature", "atmosphere.exobase_temperature"),
                port("particle_mass", "atmosphere.particle_mass_amu", D::NONE, Some((1.0, 100.0))),
            ],
            outputs: vec![port("lambda", "atmosphere.jeans_parameter", D::NONE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[CK, "Chamberlain, J. W. and Hunten, D. M. 1987, Theory of Planetary Atmospheres, 2nd edn, Academic Press"],
            curious: "Compares how tightly a world holds a gas molecule with how fast heat makes it move. Below about 1.5 the gas pours away; above about 30 it stays for billions of years.",
            science: "Escape parameter at the exobase: gravitational energy over thermal energy for a particle of the given mass (atomic mass units). Jeans escape flux falls roughly as λ e^(-λ).",
            formula: "λ = G M m / (k T r)",
            f: |i, _| {
                let mm = positive(i[0], "mass")?;
                let r = positive(i[1], "exobase radius")?;
                let t = positive(i[2], "exobase temperature")?;
                let mp = positive(i[3], "particle mass")? * k::AMU;
                Ok(vec![G * mm * mp / (k::BOLTZMANN * t * r)])
            },
        }
        .build(),
    ]
}
