//! Ten toy atoms: test fixtures for the flow engine (K4), NOT the physics
//! library. Real atoms arrive in P1 through the issue, spec, tests, code rule,
//! each with checked citations. Some formulas here are textbook-correct
//! (insolation, equilibrium temperature, Kepler's third law, surface gravity,
//! escape velocity, density, Stefan-Boltzmann); the main-sequence power laws
//! are deliberately crude.

use crate::decl::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;

pub struct FnAtom {
    decl: AtomDecl,
    f: fn(&[f64], &[f64]) -> Vec<f64>,
}

impl Atom for FnAtom {
    fn decl(&self) -> &AtomDecl {
        &self.decl
    }
    fn eval(&self, i: &[f64], p: &[f64]) -> Result<Vec<f64>, AtomError> {
        Ok((self.f)(i, p))
    }
}

fn port(name: &str, kind: &str, dim: D, valid: Option<(f64, f64)>) -> Port {
    Port {
        name: name.into(),
        kind: kind.into(),
        dim,
        valid,
        shape: None,
        any_dim: false,
    }
}

fn atom(
    id: &str,
    inputs: Vec<Port>,
    outputs: Vec<Port>,
    params: Vec<Param>,
    formula: &str,
    curious: &str,
    f: fn(&[f64], &[f64]) -> Vec<f64>,
) -> Box<dyn Atom> {
    Box::new(FnAtom {
        decl: AtomDecl {
            id: id.into(),
            version: "0.1.0".into(),
            inputs,
            outputs,
            params,
            cost_class: CostClass::Trivial,
            references: vec!["toy fixture: not for physics use".into()],
            explain: ExplainText {
                curious: curious.into(),
                science: String::new(),
                formula: formula.into(),
            },
            class: None,
            dims: None,
        },
        f,
    })
}

const M_SUN: f64 = GM_SUN / G;
const FOUR_PI: f64 = 4.0 * m::PI;

pub fn registry() -> AtomRegistry {
    let mut r = AtomRegistry::default();
    let ms = Some((0.08 * M_SUN, 20.0 * M_SUN));
    r.insert(atom(
        "toy.luminosity_from_mass",
        vec![port("mass", "star.mass", D::MASS, ms)],
        vec![port("luminosity", "star.luminosity", D::POWER, None)],
        vec![Param {
            name: "exponent".into(),
            dim: D::NONE,
            default: 3.5,
            range: Some((1.0, 6.0)),
            description: "power-law index".into(),
        }],
        "L = L☉ (M/M☉)^k",
        "Heavier stars burn far brighter.",
        |i, p| vec![L_SUN * m::pow(i[0] / M_SUN, p[0])],
    ));
    r.insert(atom(
        "toy.radius_from_mass",
        vec![port("mass", "star.mass", D::MASS, ms)],
        vec![port("radius", "star.radius", D::LENGTH, None)],
        vec![],
        "R = R☉ (M/M☉)^0.8",
        "Heavier stars are bigger.",
        |i, _| vec![R_SUN * m::pow(i[0] / M_SUN, 0.8)],
    ));
    r.insert(atom(
        "toy.effective_temperature",
        vec![
            port("luminosity", "star.luminosity", D::POWER, None),
            port("radius", "star.radius", D::LENGTH, None),
        ],
        vec![port(
            "teff",
            "star.effective_temperature",
            D::TEMPERATURE,
            None,
        )],
        vec![],
        "T = (L / (4π R² σ))^¼",
        "A star's surface temperature follows from how much light leaves how much surface.",
        |i, _| {
            vec![m::pow(
                i[0] / (FOUR_PI * i[1] * i[1] * STEFAN_BOLTZMANN),
                0.25,
            )]
        },
    ));
    r.insert(atom(
        "toy.insolation",
        vec![
            port("luminosity", "star.luminosity", D::POWER, None),
            port(
                "distance",
                "orbit.semi_major_axis",
                D::LENGTH,
                Some((1e7, 1e16)),
            ),
        ],
        vec![port("flux", "planet.insolation", D::FLUX, None)],
        vec![],
        "S = L / (4π a²)",
        "Starlight spreads out over a growing sphere, so it weakens with the square of distance.",
        |i, _| vec![i[0] / (FOUR_PI * i[1] * i[1])],
    ));
    r.insert(atom(
        "toy.equilibrium_temperature",
        vec![
            port("flux", "planet.insolation", D::FLUX, None),
            port("albedo", "planet.bond_albedo", D::NONE, Some((0.0, 1.0))),
        ],
        vec![port(
            "teq",
            "planet.equilibrium_temperature",
            D::TEMPERATURE,
            None,
        )],
        vec![],
        "T_eq = (S (1 − A) / 4σ)^¼",
        "How warm a planet would be with no air, balancing sunlight in against heat out.",
        |i, _| vec![m::pow(i[0] * (1.0 - i[1]) / (4.0 * STEFAN_BOLTZMANN), 0.25)],
    ));
    r.insert(atom(
        "toy.orbital_period",
        vec![
            port("distance", "orbit.semi_major_axis", D::LENGTH, None),
            port("mass", "star.mass", D::MASS, None),
        ],
        vec![port("period", "orbit.period", D::TIME, None)],
        vec![],
        "P = 2π √(a³ / G M)",
        "Farther planets take longer to go round.",
        |i, _| vec![m::TAU * m::sqrt(i[0] * i[0] * i[0] / (G * i[1]))],
    ));
    r.insert(atom(
        "toy.surface_gravity",
        vec![
            port("mass", "planet.mass", D::MASS, None),
            port("radius", "planet.radius", D::LENGTH, None),
        ],
        vec![port("g", "planet.surface_gravity", D::ACCELERATION, None)],
        vec![],
        "g = G m / r²",
        "How hard the ground pulls you down.",
        |i, _| vec![G * i[0] / (i[1] * i[1])],
    ));
    r.insert(atom(
        "toy.escape_velocity",
        vec![
            port("mass", "planet.mass", D::MASS, None),
            port("radius", "planet.radius", D::LENGTH, None),
        ],
        vec![port("v", "planet.escape_velocity", D::VELOCITY, None)],
        vec![],
        "v = √(2 G m / r)",
        "How fast you must go to leave and never fall back.",
        |i, _| vec![m::sqrt(2.0 * G * i[0] / i[1])],
    ));
    r.insert(atom(
        "toy.bulk_density",
        vec![
            port("mass", "planet.mass", D::MASS, None),
            port("radius", "planet.radius", D::LENGTH, None),
        ],
        vec![port("density", "planet.bulk_density", D::DENSITY, None)],
        vec![],
        "ρ = m / (4/3 π r³)",
        "Mass packed per volume: rock and iron are dense, gas and ice are not.",
        |i, _| vec![i[0] / (FOUR_PI / 3.0 * i[1] * i[1] * i[1])],
    ));
    r.insert(atom(
        "toy.greenhouse_residual",
        vec![
            port(
                "t_surface",
                "planet.surface_temperature",
                D::TEMPERATURE,
                None,
            ),
            port("flux", "planet.insolation", D::FLUX, None),
            port("albedo", "planet.bond_albedo", D::NONE, Some((0.0, 1.0))),
        ],
        vec![port("residual", "residual.flux", D::FLUX, None)],
        vec![Param {
            name: "tau".into(),
            dim: D::NONE,
            default: 0.8,
            range: Some((0.0, 100.0)),
            description: "grey infrared optical depth".into(),
        }],
        "σT⁴ − (S(1 − A)/4)(1 + ¾τ) = 0",
        "Air traps some heat, so the ground warms until it sheds as much as it gets.",
        |i, p| {
            vec![
                STEFAN_BOLTZMANN * m::pow(i[0], 4.0)
                    - i[1] * (1.0 - i[2]) / 4.0 * (1.0 + 0.75 * p[0]),
            ]
        },
    ));
    r
}

/// Quantity kinds used by the toy atoms.
pub fn kinds() -> std::collections::BTreeMap<String, sse_core::QuantityKind> {
    let k = |id: &str, dim: D| {
        (
            id.to_string(),
            sse_core::QuantityKind {
                id: id.into(),
                dim,
                range: None,
                description: String::new(),
            },
        )
    };
    [
        k("star.mass", D::MASS),
        k("star.luminosity", D::POWER),
        k("star.radius", D::LENGTH),
        k("star.effective_temperature", D::TEMPERATURE),
        k("orbit.semi_major_axis", D::LENGTH),
        k("orbit.period", D::TIME),
        k("planet.insolation", D::FLUX),
        k("planet.bond_albedo", D::NONE),
        k("planet.equilibrium_temperature", D::TEMPERATURE),
        k("planet.surface_temperature", D::TEMPERATURE),
        k("planet.mass", D::MASS),
        k("planet.radius", D::LENGTH),
        k("planet.surface_gravity", D::ACCELERATION),
        k("planet.escape_velocity", D::VELOCITY),
        k("planet.bulk_density", D::DENSITY),
    ]
    .into_iter()
    .collect()
}
