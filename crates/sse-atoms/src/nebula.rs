//! Nebulae from causes (track O3; nebulae-and-light design section 3): how
//! far a hot star's light ionises the gas round it, how a dying star's shed
//! shell and a supernova's blast wave grow with time, and how bright the
//! hydrogen they light shines. Each is a published relation; `sse-nebula`
//! builds nebula objects from these blocks.

use crate::common::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::Dimension as D;
use sse_flow::{Atom, AtomError, CostClass};

const OF06: &str = "Osterbrock, D. E. and Ferland, G. J. 2006, Astrophysics of Gaseous Nebulae and Active Galactic Nuclei, 2nd edn, University Science Books: chapter 2 (Strömgren sphere; case B recombination coefficient 2.59e-13 cm³/s at 10⁴ K) and chapter 4 (Balmer emissivities; Hα/Hβ = 2.86) https://uscibooks.aip.org/books/astrophysics-of-gaseous-nebulae-and-active-galactic-nuclei-second-edition/";
const STROMGREN39: &str = "Strömgren, B. 1939, The physical state of interstellar hydrogen, ApJ 89, 526. doi:10.1086/144074";
const RL79: &str = "Rybicki, G. B. and Lightman, A. P. 1979, Radiative Processes in Astrophysics, Wiley, chapter 1: photon numbers from Planck's law. doi:10.1002/9783527618170";
const SEDOV: &str = "Sedov, L. I. 1959, Similarity and Dimensional Methods in Mechanics, Academic Press; Taylor, G. I. 1950, The formation of a blast wave by a very intense explosion, Proc. R. Soc. A 201, 159 (ξ = 1.1517 for γ = 5/3). doi:10.1098/rspa.1950.0049";
const CMB88: &str = "Cioffi, D. F., McKee, C. F. and Bertschinger, E. 1988, Dynamics of radiative supernova remnants, ApJ 334, 252: t_PDS = 1.33×10⁴ E₅₁^(3/14) n₀^(-4/7) yr, R_PDS = 14.0 E₅₁^(2/7) n₀^(-3/7) pc, R = R_PDS (4t/3t_PDS - 1/3)^(3/10). doi:10.1086/166834";
const KPF78: &str = "Kwok, S., Purton, C. R. and Fitzgerald, P. M. 1978, On the origin of planetary nebulae, ApJ 219, L125: a fast late wind sweeps the slow giant wind into a shell. doi:10.1086/182621";

/// Case B recombination coefficient at 10⁴ K, m³/s.
pub const ALPHA_B: f64 = 2.59e-19;
/// Hα energy, J (656.3 nm).
pub const H_ALPHA_J: f64 = 6.626_070_15e-34 * 299_792_458.0 / 656.28e-9;
/// Hα photons per case B recombination at 10⁴ K: α_eff(Hβ)/α_B × Hα/Hβ in energy
/// over the photon-energy ratio = 0.45 (Osterbrock and Ferland 2006, tables 4.1, 4.2).
pub const H_ALPHA_PER_RECOMBINATION: f64 = 0.450;
/// Hydrogen mass, kg, and the mass per hydrogen nucleus with helium (μ_H = 1.4).
pub const M_H: f64 = 1.673_5e-27;
pub const MU_H: f64 = 1.4;
/// Sedov-Taylor constant for γ = 5/3.
pub const XI_SEDOV: f64 = 1.1517;
const PC: f64 = 648_000.0 / m::PI * AU;
const YR: f64 = JULIAN_YEAR;
/// Ionisation energy of hydrogen, J (13.598 eV).
pub const E_ION_H: f64 = 13.598 * 1.602_176_634e-19;

/// Photons per second above `e_min` from a black body of radius `r` at `t`:
/// 4πR² · (2π/c²)(kT/h)³ ∫ x²/(eˣ-1) dx from x₀, by the series
/// Σ e^(-kx₀)(x₀²/k + 2x₀/k² + 2/k³), 400 terms (fixed).
pub fn ionising_photons(t_k: f64, radius_m: f64, e_min_j: f64) -> f64 {
    if t_k <= 0.0 || radius_m <= 0.0 {
        return 0.0;
    }
    let (h, c, k) = (6.626_070_15e-34, C, 1.380_649e-23);
    let x0 = e_min_j / (k * t_k);
    let mut s = 0.0;
    for n in 1..=400 {
        let kf = n as f64;
        let e = m::exp(-kf * x0);
        if e == 0.0 {
            break;
        }
        s += e * (x0 * x0 / kf + 2.0 * x0 / (kf * kf) + 2.0 / (kf * kf * kf));
    }
    let kt_h = k * t_k / h;
    4.0 * m::PI * radius_m * radius_m * (2.0 * m::PI / (c * c)) * kt_h * kt_h * kt_h * s
}

/// Strömgren radius, m, for `q` ionising photons/s in hydrogen of number density `n` (m⁻³).
pub fn stromgren_radius(q: f64, n: f64) -> f64 {
    if q <= 0.0 || n <= 0.0 {
        return 0.0;
    }
    m::cbrt(3.0 * q / (4.0 * m::PI * n * n * ALPHA_B))
}

/// Supernova remnant shock radius, m, at `t` s after an explosion of energy
/// `e` J with ejecta mass `m_ej` kg into hydrogen density `n0` (m⁻³): free
/// expansion, then Sedov-Taylor, then the pressure-driven snowplough.
pub fn remnant_radius(e: f64, m_ej: f64, n0: f64, t: f64) -> (f64, &'static str) {
    let rho = MU_H * M_H * n0;
    let v_ej = m::sqrt(2.0 * e / m_ej);
    let r_free = v_ej * t;
    // Free expansion ends when the swept-up mass equals the ejecta mass.
    let r_sweep = m::cbrt(3.0 * m_ej / (4.0 * m::PI * rho));
    if r_free < r_sweep {
        return (r_free, "free expansion");
    }
    let e51 = e / 1e44;
    let n_cc = n0 * 1e-6;
    let t_pds = 1.33e4 * YR * m::pow(e51, 3.0 / 14.0) * m::pow(n_cc, -4.0 / 7.0);
    if t < t_pds {
        let r = XI_SEDOV * m::pow(e * t * t / rho, 0.2);
        return (r.max(r_sweep), "Sedov-Taylor");
    }
    let r_pds = 14.0 * PC * m::pow(e51, 2.0 / 7.0) * m::pow(n_cc, -3.0 / 7.0);
    (
        r_pds * m::pow(4.0 * t / (3.0 * t_pds) - 1.0 / 3.0, 0.3),
        "pressure-driven snowplough",
    )
}

fn nonneg(x: f64, what: &str) -> Result<f64, AtomError> {
    if x >= 0.0 && x.is_finite() {
        Ok(x)
    } else {
        Err(AtomError(format!("{what} must be zero or more, got {x}")))
    }
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    let number_density = D([-3, 0, 0, 0, 0, 0, 0]);
    let rate = crate::common::dims::RATE;
    vec![
        Def {
            id: "sse.nebula.ionising_photon_rate",
            version: "1.0.0",
            inputs: vec![
                port("teff", "star.effective_temperature", D::TEMPERATURE, None),
                port("radius", "star.radius", D::LENGTH, None),
            ],
            outputs: vec![port("q", "light.ionising_photon_rate", rate, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[RL79, OF06],
            curious: "Only the hottest stars give out much light energetic enough to tear electrons off hydrogen. A star twice as hot as the Sun gives thousands of times more.",
            science: "Photons above 13.6 eV from a black body photosphere, counted exactly with the series form of the Planck photon integral. Real hot-star atmospheres differ from a black body by up to a factor of a few above the Lyman edge; a model-atmosphere table can replace this block.",
            formula: "Q = 4πR² (2π/c²)(kT/h)³ ∫ x² / (eˣ − 1) dx,  x > 13.6 eV / kT",
            f: |i, _| Ok(vec![ionising_photons(nonneg(i[0], "temperature")?, nonneg(i[1], "radius")?, E_ION_H)]),
        }
        .build(),
        Def {
            id: "sse.nebula.stromgren_radius",
            version: "1.0.0",
            inputs: vec![
                port("q", "light.ionising_photon_rate", rate, None),
                port("density", "gas.hydrogen_density", number_density, None),
            ],
            outputs: vec![port("radius", "nebula.ionised_radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[STROMGREN39, OF06],
            curious: "A hot star lights a glowing bubble in the gas round it, out to where every ionising photon has been used up keeping the gas ionised. Thinner gas, bigger bubble.",
            science: "Ionisation balance in a uniform medium: photons emitted equal recombinations to excited levels (case B) inside the sphere.",
            formula: "R_S = (3Q / (4π n² α_B))^⅓",
            f: |i, _| Ok(vec![stromgren_radius(nonneg(i[0], "photon rate")?, nonneg(i[1], "density")?)]),
        }
        .build(),
        Def {
            id: "sse.nebula.balmer_alpha_power",
            version: "1.0.0",
            inputs: vec![port("recombinations", "nebula.recombination_rate", rate, None)],
            outputs: vec![port("power", "nebula.h_alpha_power", D::POWER, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[OF06],
            curious: "Each time an electron falls back onto a proton it cascades down the energy ladder, and nearly half the time one step gives out the red light of hydrogen alpha. That is why glowing nebulae are red.",
            science: "Case B at 10⁴ K: 0.45 Hα photons per recombination, so L(Hα) = 1.37×10⁻¹⁹ W per recombination per second. For an ionisation-bounded nebula the recombination rate equals the ionising photon rate.",
            formula: "L(Hα) = 0.45 hν(Hα) × recombinations per second",
            f: |i, _| Ok(vec![H_ALPHA_PER_RECOMBINATION * H_ALPHA_J * nonneg(i[0], "recombination rate")?]),
        }
        .build(),
        Def {
            id: "sse.nebula.shell_radius",
            version: "1.0.0",
            inputs: vec![
                port("speed", "nebula.expansion_speed", D::VELOCITY, None),
                port("age", "nebula.age", D::TIME, None),
            ],
            outputs: vec![port("radius", "nebula.shell_radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[KPF78],
            curious: "The gas a dying star throws off coasts outwards at a steady speed, so a ring nebula's size is a clock: measure it and its speed and you know its age.",
            science: "In the interacting-winds picture the swept-up shell of a planetary nebula expands at a nearly constant speed of tens of km/s over its visible life.",
            formula: "R = v t",
            f: |i, _| Ok(vec![nonneg(i[0], "speed")? * nonneg(i[1], "age")?]),
        }
        .build(),
        Def {
            id: "sse.nebula.remnant_radius",
            version: "1.0.0",
            inputs: vec![
                port("energy", "event.explosion_energy", D::POWER.mul(D::TIME), None),
                port("ejecta", "event.ejecta_mass", D::MASS, None),
                port("density", "gas.hydrogen_density", number_density, None),
                port("age", "nebula.age", D::TIME, None),
            ],
            outputs: vec![port("radius", "nebula.shell_radius", D::LENGTH, None)],
            params: vec![],
            cost: CostClass::Trivial,
            references: &[SEDOV, CMB88],
            curious: "A supernova's blast coasts at first, then slows as it sweeps up gas, then cools and ploughs on as a thin shell until it is no faster than the gas around it.",
            science: "Free expansion at √(2E/M) until the swept mass equals the ejecta; the Sedov-Taylor blast wave R = 1.15 (E t²/ρ)^⅕; after shell formation Cioffi, McKee and Bertschinger's pressure-driven snowplough. Solar metallicity; uniform surroundings.",
            formula: "R = v t;  1.15 (E t²/ρ)^⅕;  R_PDS (4t/3t_PDS − ⅓)^0.3",
            f: |i, _| {
                let e = positive(i[0], "explosion energy")?;
                let mej = positive(i[1], "ejecta mass")?;
                let n = positive(i[2], "density")?;
                Ok(vec![remnant_radius(e, mej, n, nonneg(i[3], "age")?).0])
            },
        }
        .build(),
    ]
}
