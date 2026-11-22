//! Air optics (track O, for the renderer's sky and the spectrum chart): how
//! strongly a planet's air scatters light by wavelength (Rayleigh), how
//! thick it is, and its optical depth. Gas data (refractivity and King
//! factor per gas) is pack data in `base-light`; these blocks are the
//! physics.

use crate::common::*;
use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::{Atom, CostClass};

const BOD99: &str = "Bodhaine, B. A., Wood, N. B., Dutton, E. G. and Slusser, J. R. 1999, On Rayleigh optical depth calculations, J. Atmos. Oceanic Technol. 16, 1854: σ = 24π³(n²−1)²/(λ⁴N_s²(n²+2)²)·F; N_s = 2.546899×10¹⁹ cm⁻³ at 288.15 K; dry air 4.5105×10⁻²⁷ cm² at 550 nm; τ = P A σ / (m_a g); King factors N₂ 1.034 + 3.17×10⁻⁴ λ⁻², O₂ 1.096 + 1.385×10⁻³ λ⁻² + 1.448×10⁻⁴ λ⁻⁴, Ar 1.00, CO₂ 1.15. doi:10.1175/1520-0426(1999)016<1854:ORODC>2.0.CO;2";
const PR72: &str = "Peck, E. R. and Reeder, K. 1972, Dispersion of air, J. Opt. Soc. Am. 62, 958: (n−1)×10⁸ = 8060.51 + 2480990/(132.274 − λ⁻²) + 17455.7/(39.32957 − λ⁻²), λ in µm, 288.15 K, 101325 Pa. doi:10.1364/JOSA.62.000958";
const BUCH95: &str = "Bucholtz, A. 1995, Rayleigh-scattering calculations for the terrestrial atmosphere, Applied Optics 34, 2765. doi:10.1364/AO.34.002765";

/// Boltzmann constant, J/K.
pub const K_B: f64 = 1.380_649e-23;
/// Atomic mass unit, kg.
pub const AMU: f64 = 1.660_539_066_60e-27;
/// Number density at which dry-air refractivity (Peck and Reeder) is given: 288.15 K, 101325 Pa, m⁻³.
pub const N_S_288: f64 = 2.546_899e25;
/// Loschmidt's number, 273.15 K and 101325 Pa, m⁻³ (CODATA).
pub const N_LOSCHMIDT: f64 = 2.686_780_111e25;

/// Dry-air refractivity n − 1 at `lambda_m` (Peck and Reeder 1972), at N_S_288.
pub fn air_refractivity(lambda_m: f64) -> f64 {
    let s2 = 1.0 / (lambda_m * 1e6 * lambda_m * 1e6);
    (8060.51 + 2_480_990.0 / (132.274 - s2) + 17455.7 / (39.329_57 - s2)) * 1e-8
}

/// Rayleigh cross-section per molecule, m², from the refractivity `n1` (n − 1)
/// measured at number density `n_ref`, and the King factor.
pub fn rayleigh_cross_section(lambda_m: f64, n1: f64, n_ref: f64, king: f64) -> f64 {
    let n = 1.0 + n1;
    let a = (n * n - 1.0) / (n * n + 2.0);
    let l2 = lambda_m * lambda_m;
    24.0 * m::PI * m::PI * m::PI * a * a / (l2 * l2 * n_ref * n_ref) * king
}

/// Optical depth of a whole air column: τ = P σ / (m̄ g).
pub fn column_optical_depth(pressure_pa: f64, sigma_m2: f64, mean_mass_kg: f64, g: f64) -> f64 {
    pressure_pa * sigma_m2 / (mean_mass_kg * g)
}

/// Pressure scale height, m: H = kT / (m̄ g).
pub fn scale_height(t_k: f64, mean_mass_kg: f64, g: f64) -> f64 {
    sse_core::constants::k_b() * t_k / (mean_mass_kg * g)
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let area = D([2, 0, 0, 0, 0, 0, 0]);
    let number_density = D([-3, 0, 0, 0, 0, 0, 0]);
    let inverse_length = D([-1, 0, 0, 0, 0, 0, 0]);
    vec![
        Def {
            id: "sse.air.rayleigh_cross_section",
            version: "1.0.0",
            inputs: vec![
                port("wavelength", "light.wavelength", D::LENGTH, Some((1e-8, 1e-3))),
                port("refractivity", "gas.refractivity", D::NONE, Some((0.0, 0.1))),
                port("n_ref", "gas.reference_density", number_density, None),
                port("king", "gas.king_factor", D::NONE, Some((1.0, 2.0))),
            ],
            outputs: vec![port("sigma", "gas.rayleigh_cross_section", area, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[BOD99, BUCH95],
            curious: "Air molecules scatter blue light far more than red, as the inverse fourth power of wavelength: that is why the sky is blue and sunsets red.",
            science: "Rayleigh scattering by molecules much smaller than the wavelength, from the gas's refractive index (Lorentz-Lorenz) with the King factor for non-spherical molecules.",
            formula: "σ = 24π³ / (λ⁴ N_ref²) · ((n² − 1)/(n² + 2))² · F_K",
            f: |i, _| Ok(vec![rayleigh_cross_section(positive(i[0], "wavelength")?, i[1].max(0.0), positive(i[2], "reference density")?, i[3])]),
        }
        .build(),
        Def {
            id: "sse.air.dry_air_refractivity",
            version: "1.0.0",
            inputs: vec![port("wavelength", "light.wavelength", D::LENGTH, Some((2.3e-7, 2e-6)))],
            outputs: vec![port("refractivity", "gas.refractivity", D::NONE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[PR72],
            curious: "Air bends light by about three parts in ten thousand, a little more for blue than red.",
            science: "Measured dispersion of dry air at 15 °C and one atmosphere, 230 nm to 1.7 µm.",
            formula: "(n−1)×10⁸ = 8060.51 + 2480990/(132.274 − λ⁻²) + 17455.7/(39.32957 − λ⁻²),  λ in µm",
            f: |i, _| Ok(vec![air_refractivity(positive(i[0], "wavelength")?)]),
        }
        .build(),
        Def {
            id: "sse.air.column_optical_depth",
            version: "1.0.0",
            inputs: vec![
                port("pressure", "atmosphere.surface_pressure", D::PRESSURE, None),
                port("sigma", "gas.rayleigh_cross_section", area, None),
                port("mean_mass", "atmosphere.mean_molecular_mass", D::MASS, None),
                port("gravity", "body.surface_gravity", D::ACCELERATION, None),
            ],
            outputs: vec![port("tau", "atmosphere.rayleigh_optical_depth", D::NONE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[BOD99],
            curious: "Looking straight up from Earth's surface, about one in ten blue-green photons is scattered on the way through; Venus's air scatters almost all of them.",
            science: "The column above the surface holds P / (m̄ g) molecules per unit area, each scattering with cross-section σ. Bodhaine et al. evaluate g at the column's mass-weighted height; this block takes g as given.",
            formula: "τ = P σ / (m̄ g)",
            f: |i, _| Ok(vec![column_optical_depth(i[0].max(0.0), i[1].max(0.0), positive(i[2], "mean molecular mass")?, positive(i[3], "gravity")?)]),
        }
        .build(),
        Def {
            id: "sse.air.scale_height",
            version: "1.0.0",
            inputs: vec![
                port("temperature", "atmosphere.temperature", D::TEMPERATURE, None),
                port("mean_mass", "atmosphere.mean_molecular_mass", D::MASS, None),
                port("gravity", "body.surface_gravity", D::ACCELERATION, None),
            ],
            outputs: vec![port("h", "atmosphere.scale_height", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[BOD99],
            curious: "Air thins by a factor of e every scale height: about 8.4 km on Earth, 11 km on Mars, 40 km on Titan, where gravity is weak and the air cold but heavy.",
            science: "Isothermal hydrostatic balance.",
            formula: "H = k T / (m̄ g)",
            f: |i, _| Ok(vec![scale_height(positive(i[0], "temperature")?, positive(i[1], "mean molecular mass")?, positive(i[2], "gravity")?)]),
        }
        .build(),
        Def {
            id: "sse.air.scattering_coefficient",
            version: "1.0.0",
            inputs: vec![
                port("density", "atmosphere.number_density", number_density, None),
                port("sigma", "gas.rayleigh_cross_section", area, None),
            ],
            outputs: vec![port("beta", "atmosphere.rayleigh_scattering_coefficient", inverse_length, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[BUCH95, BOD99],
            curious: "At sea level, blue-green light travels about 87 km before air scatters it, on average.",
            science: "Volume scattering coefficient: molecules per cubic metre times the cross-section each.",
            formula: "β = n σ",
            f: |i, _| Ok(vec![i[0].max(0.0) * i[1].max(0.0)]),
        }
        .build(),
    ]
}
