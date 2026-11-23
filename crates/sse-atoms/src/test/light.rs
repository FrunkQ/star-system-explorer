//! `light` blocks as they were in Rust before ND3b moved them into formula
//! files; kept for the test that the formulas give the same faces and bits.

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, AtomError, CostClass};

const PC91: &str = "Pearl, J. C. and Conrath, B. J. 1991, The albedo, effective temperature, and energy balance of Neptune, as determined from Voyager data, J. Geophys. Res. 96, 18921 https://ui.adsabs.harvard.edu/abs/1991JGR....9618921P";
const LI18: &str = "Li, L. et al. 2018, Less absorbed solar energy and more internal heat for Jupiter, Nature Communications 9, 3709. doi:10.1038/s41467-018-06107-2";
const NSSDC: &str = "Williams, D. R., NASA Planetary Fact Sheets (Bond albedo, black-body temperature, dipole field strength), NSSDCA https://nssdc.gsfc.nasa.gov/planetary/factsheet/";
const PARKER58: &str = "Parker, E. N. 1958, Dynamics of the interplanetary gas and magnetic fields, ApJ 128, 664: the spiral interplanetary field. doi:10.1086/146579";
const KR95: &str = "Kivelson, M. G. and Russell, C. T. (eds) 1995, Introduction to Space Physics, Cambridge University Press, chapter 9 (magnetopause stand-off from pressure balance, Chapman-Ferraro compression factor 2) https://www.cambridge.org/9780521457149";
const ZARKA07: &str = "Zarka, P. 2007, Plasma interactions of exoplanets with their parent star and associated radio emissions, Planet. Space Sci. 55, 598: the radio-magnetic Bode's law, P_radio ≈ 2×10⁻³ of the incident magnetic power https://ui.adsabs.harvard.edu/abs/2007P%26SS...55..598Z";
const ZARKA01: &str = "Zarka, P., Treumann, R. A., Ryabov, B. P. and Ryabov, V. B. 2001, Magnetically-driven planetary radio emissions and application to extrasolar planets, Astrophys. Space Sci. 277, 293. doi:10.1023/A:1012221527425";
const CYCLOTRON: &str = "Treumann, R. A. 2006, The electron-cyclotron maser for astrophysical application, Astron. Astrophys. Rev. 13, 229: auroral radio is emitted near the local electron gyrofrequency f = eB/(2π m_e). doi:10.1007/s00159-006-0001-y";

/// Vacuum permeability, N A⁻² (CODATA 2018).
pub const MU0: f64 = sse_core::sheet::MU0;
/// Electron charge over electron mass, C/kg (CODATA 2018).
pub const E_OVER_ME: f64 = sse_core::sheet::E / sse_core::sheet::M_E;
/// Proton mass, kg (CODATA 2018).
pub const M_PROTON: f64 = sse_core::sheet::M_P;
/// The Sun's sidereal rotation rate (Carrington, 25.38 d), rad/s.
/// Dimensions: magnetic moment (A m²), magnetic field (tesla = kg s⁻² A⁻¹), area.
pub const MOMENT: D = D([2, 0, 0, 1, 0, 0, 0]);
pub const TESLA: D = D([0, 1, -2, -1, 0, 0, 0]);
pub const AREA: D = D([2, 0, 0, 0, 0, 0, 0]);
pub const OMEGA_SUN: f64 = 2.0 * m::PI / (25.38 * DAY);

fn nonneg(x: f64, what: &str) -> Result<f64, AtomError> {
    if x >= 0.0 && x.is_finite() {
        Ok(x)
    } else {
        Err(AtomError(format!("{what} must be zero or more, got {x}")))
    }
}

/// T⁴ = (1 − A) S / (4σ) + q / σ: absorbed light shared round the sphere plus
/// the body's own heat flowing out through its surface.
pub fn effective_temperature(insolation: f64, albedo: f64, internal_flux: f64) -> f64 {
    let t4 =
        (1.0 - albedo) * insolation / (4.0 * STEFAN_BOLTZMANN) + internal_flux / STEFAN_BOLTZMANN;
    m::sqrt(m::sqrt(t4.max(0.0)))
}

/// Magnetopause stand-off where the compressed dipole's pressure meets the
/// wind's ram pressure: (2 B_eq(r))² / 2μ₀ = ρv², B_eq(r) = μ₀M / (4π r³).
pub fn magnetopause_radius(moment: f64, density: f64, speed: f64) -> f64 {
    let ram = density * speed * speed;
    if moment <= 0.0 || ram <= 0.0 {
        return 0.0;
    }
    m::pow(
        MU0 * moment * moment / (8.0 * m::PI * m::PI * ram),
        1.0 / 6.0,
    )
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    vec![
        Def {
            id: "sse.light.effective_temperature",
            version: "1.0.0",
            inputs: vec![
                port("flux", "planet.insolation", D::FLUX, None),
                port("albedo", "planet.bond_albedo", D::NONE, Some((0.0, 1.0))),
                port("internal_flux", "body.internal_heat_flux", D::FLUX, None),
            ],
            outputs: vec![port("teff", "body.effective_temperature", D::TEMPERATURE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[PC91, LI18, NSSDC],
            curious: "Everything glows by its own warmth. A world warmed only by its star glows at the bare-ball temperature; Jupiter, Saturn and Neptune glow warmer because they are still losing the heat they were born with.",
            science: "Global energy balance: absorbed starlight spread over the whole sphere plus internal heat flux, radiated as a black body. This is the effective temperature Voyager and Cassini measured for the giants (Neptune 59.3 K, Pearl and Conrath 1991). The same block gives a star its effective temperature when the internal flux is its own luminosity over its surface.",
            formula: "T_eff = ((1 − A) S / 4σ + q / σ)^¼",
            f: |i, _| {
                nonneg(i[0], "insolation")?;
                nonneg(i[2], "internal heat flux")?;
                Ok(vec![effective_temperature(i[0], i[1], i[2])])
            },
        }
        .build(),
        Def {
            id: "sse.light.reflected_area",
            version: "1.0.0",
            inputs: vec![
                port("radius", "body.radius", D::LENGTH, None),
                port("albedo", "planet.bond_albedo", D::NONE, Some((0.0, 1.0))),
                port("source_radius", "star.radius", D::LENGTH, None),
                port("distance", "orbit.semi_major_axis", D::LENGTH, None),
            ],
            outputs: vec![port("area", "light.reflected_area", AREA, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[NSSDC, PARKER58],
            curious: "Planets shine mostly by borrowed light. How much comes back depends on how big the world is, how pale it is, and how much of its star's light reaches it.",
            science: "A sphere of radius R at distance d intercepts πR² of a flux that, for a star of radius R* glowing at T*, is π B_λ(T*) (R*/d)². A fraction A (the Bond albedo) is sent back. Written as an effective area so the reflected spectrum is that area times the star's surface brightness. Grey albedo: real albedos vary with wavelength.",
            formula: "A_eff = A π R² (R* / d)²",
            f: |i, _| {
                let r = positive(i[0], "radius")?;
                let d = positive(i[3], "distance")?;
                let rs = nonneg(i[2], "source radius")?;
                Ok(vec![i[1] * m::PI * r * r * (rs / d) * (rs / d)])
            },
        }
        .build(),
        Def {
            id: "sse.light.stellar_wind",
            version: "1.0.0",
            inputs: vec![port("distance", "orbit.semi_major_axis", D::LENGTH, None)],
            outputs: vec![
                port("density", "wind.density", D::DENSITY, None),
                port("speed", "wind.speed", D::VELOCITY, None),
                port("field", "wind.transverse_field", TESLA, None),
            ],
            params: vec![
                param("n_1au", D::NONE, 7.0, (0.0, 1e6), "proton number density at 1 au, per cm³ (Sun: about 7)"),
                param("speed_km_s", D::NONE, 440.0, (10.0, 5000.0), "wind speed, km/s (Sun: about 440)"),
                param("radial_field_nt", D::NONE, 3.5, (0.0, 1e6), "radial field at 1 au, nT (Sun: about 3.5)"),
                param("spin_days", D::NONE, 25.38, (0.01, 1e5), "the star's rotation period, days (Sun: 25.38)"),
            ],
            cost: CostClass::Trivial,
            references: &[PARKER58, KR95],
            curious: "Stars blow a thin wind of charged gas that carries their magnetic field out past the planets, wound into a spiral by the star's spin like water from a garden sprinkler.",
            science: "Parker's wind at constant speed: density falls as 1/d², the radial field as 1/d², and the azimuthal field B_φ = B_r Ω d / v as 1/d. B_φ is the part of the field across the flow that a magnetosphere intercepts. Defaults are solar values at 1 au; a star's own wind comes with stellar activity (O10).",
            formula: "ρ = n₁ m_p (1 au / d)², B_⊥ = B_r1 (1 au / d)² Ω d / v",
            f: |i, p| {
                let d = positive(i[0], "distance")?;
                let x = AU / d;
                let rho = p[0] * 1e6 * M_PROTON * x * x;
                let v = p[1] * 1e3;
                let omega = 2.0 * m::PI / (p[3] * DAY);
                let b = p[2] * 1e-9 * x * x * omega * d / v;
                Ok(vec![rho, v, b])
            },
        }
        .build(),
        Def {
            id: "sse.light.magnetopause_radius",
            version: "1.0.0",
            inputs: vec![
                port("moment", "body.magnetic_moment", MOMENT, None),
                port("density", "wind.density", D::DENSITY, None),
                port("speed", "wind.speed", D::VELOCITY, None),
            ],
            outputs: vec![port("radius", "body.magnetopause_radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[KR95],
            curious: "A magnetic field holds the wind off like a bubble. Earth's bubble reaches about ten Earth radii towards the Sun; a world with no field has none.",
            science: "Pressure balance at the nose: the dipole field there, doubled by the currents on the boundary (Chapman-Ferraro), against the wind's ram pressure. Ignores plasma inside the bubble, which inflates Jupiter's to about twice this.",
            formula: "R_mp = (μ₀ M² / (8π² ρ v²))^⅙",
            f: |i, _| {
                nonneg(i[0], "magnetic moment")?;
                Ok(vec![magnetopause_radius(i[0], i[1], i[2])])
            },
        }
        .build(),
        Def {
            id: "sse.light.auroral_radio_power",
            version: "1.0.0",
            inputs: vec![
                port("field", "wind.transverse_field", TESLA, None),
                port("speed", "wind.speed", D::VELOCITY, None),
                port("magnetopause", "body.magnetopause_radius", D::LENGTH, None),
                port("radius", "body.radius", D::LENGTH, None),
            ],
            outputs: vec![port("power", "radio.auroral_power", D::POWER, None)],
            params: vec![param(
                "efficiency",
                D::NONE,
                2e-3,
                (0.0, 1.0),
                "share of the intercepted magnetic power that comes out as radio (Zarka: about 2×10⁻³)",
            )],
            cost: CostClass::Trivial,
            references: &[ZARKA07, ZARKA01],
            curious: "Magnetised planets are radio beacons. Jupiter is the loudest radio source in the sky after the Sun at long wavelengths; a world with no field is silent.",
            science: "The radio-magnetic Bode's law: every magnetised planet in the Solar System puts about the same small share of the wind's magnetic energy flux through its magnetosphere's cross-section into auroral radio. No magnetosphere (stand-off inside the body) gives no radio.",
            formula: "P = η (B_⊥² / μ₀) v π R_mp²  if R_mp > R, else 0",
            f: |i, p| {
                let (b, v, rmp, r) = (i[0], i[1], i[2], i[3]);
                if rmp <= r {
                    return Ok(vec![0.0]);
                }
                Ok(vec![p[0] * b * b / MU0 * v * m::PI * rmp * rmp])
            },
        }
        .build(),
        Def {
            id: "sse.light.cyclotron_frequency",
            version: "1.0.0",
            inputs: vec![
                port("moment", "body.magnetic_moment", MOMENT, None),
                port("radius", "body.radius", D::LENGTH, None),
            ],
            outputs: vec![port("frequency", "radio.max_frequency", crate::common::dims::RATE, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[CYCLOTRON],
            curious: "Auroral radio comes out at the rate electrons circle the field lines, so a stronger field means a higher top note. Jupiter's reaches tens of megahertz; Earth's only a few.",
            science: "Electron-cyclotron maser emission sits near the local gyrofrequency; its highest frequency is set by the strongest field, at the magnetic pole on the surface: B_pole = μ₀M / (2π R³) for a dipole. Real surface fields with multipoles go higher (Jupiter's 40 MHz cutoff).",
            formula: "f_max = (e / 2π mₑ) μ₀ M / (2π R³)",
            f: |i, _| {
                let r = positive(i[1], "radius")?;
                let b = MU0 * nonneg(i[0], "magnetic moment")? / (2.0 * m::PI * r * r * r);
                Ok(vec![E_OVER_ME * b / (2.0 * m::PI)])
            },
        }
        .build(),
    ]
}
