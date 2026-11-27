//! MF2 part 2: a giant's own field. A giant has no iron core to stir; its
//! dynamo is the deep hydrogen that conducts once squeezed hard enough. The
//! same ledger runs on it as on a rocky core (Make: flow outruns the leak;
//! strength from the power through the dynamo's top), with the layer found
//! by walking down the giant's pressure until its fluid conducts like a
//! metal:
//!
//! - pressure inside: an n = 1 polytrope (Hubbard 1984), P = K ρ², with
//!   ρ = ρc sin ξ / ξ, ξ = π r / R, K = 2 G R² / π, ρc = π M / (4 R³);
//! - conductivity at that pressure: the sheet's `giant-envelope`
//!   `sigma_elec_profile` (French et al. 2012, along Jupiter's adiabat),
//!   interpolated in log σ against log P;
//! - the dynamo's top: where σ first reaches the sheet's `metallic_from`;
//! - power: the giant's own cooling light (formation's `self_light`), the
//!   whole of it passing through the dynamo's top;
//! - field: Christensen (2010) as for a core, onset from Rm, spin from
//!   the Rossby factor, the surface dipole as for a core.
//!
//! An ice giant runs the same sums; that the conducting fluid there is
//! ionic water, not hydrogen, is in the fudge register.

use crate::{Body, Climate};
use sse_atoms::ledger as lg;
use sse_core::math as m;
use sse_reference::Reference;
use std::sync::OnceLock;

/// The shipped sheet, read once.
fn sheet() -> &'static Reference {
    static S: OnceLock<Reference> = OnceLock::new();
    S.get_or_init(Reference::base)
}

fn num(v: Option<&serde_json::Value>) -> Option<f64> {
    let v = v?;
    v.as_f64().or_else(|| v.get("v").and_then(|x| x.as_f64()))
}

/// Liquid core alloy's conductivity at the core-mantle boundary, S/m (the
/// sheet's `earth-core-liquid`, Pozzo et al. 2012).
pub fn core_sigma() -> Option<f64> {
    num(sheet()
        .row("mixtures", "earth-core-liquid")?
        .get("sigma_elec"))
}

/// The deep hydrogen's conductivity against pressure, (P Pa, σ S/m) rising
/// in pressure, and the conductivity from which it counts as a metal.
fn profile() -> Option<(Vec<(f64, f64)>, f64)> {
    let row = sheet().row("mixtures", "giant-envelope")?;
    let p = row.get("sigma_elec_profile")?;
    let cols: Vec<&str> = p
        .get("columns")?
        .as_array()?
        .iter()
        .filter_map(|c| c.as_str())
        .collect();
    let ip = cols.iter().position(|c| *c == "p_pa")?;
    let is = cols.iter().position(|c| *c == "sigma_elec")?;
    let mut rows: Vec<(f64, f64)> = p
        .get("rows")?
        .as_array()?
        .iter()
        .filter_map(|r| Some((r.get(ip)?.as_f64()?, r.get(is)?.as_f64()?)))
        .filter(|(p, s)| *p > 0.0 && *s > 0.0)
        .collect();
    rows.sort_by(|a, b| a.0.total_cmp(&b.0));
    let metal = num(row.get("metallic_from").and_then(|x| x.get("sigma_elec")))?;
    (rows.len() >= 2).then_some((rows, metal))
}

/// σ at pressure `p`: log-log between the profile's rows, held at the ends.
fn sigma_at(rows: &[(f64, f64)], p: f64) -> f64 {
    if p <= rows[0].0 {
        // Above the profile's top the fluid is molecular: σ falls on with
        // the first segment's slope.
        let (p0, s0) = rows[0];
        let (p1, s1) = rows[1];
        let k = (m::ln(s1) - m::ln(s0)) / (m::ln(p1) - m::ln(p0));
        return if p <= 0.0 {
            0.0
        } else {
            s0 * m::exp(k * (m::ln(p) - m::ln(p0)))
        };
    }
    for w in rows.windows(2) {
        let ((p0, s0), (p1, s1)) = (w[0], w[1]);
        if p <= p1 {
            let f = (m::ln(p) - m::ln(p0)) / (m::ln(p1) - m::ln(p0));
            return m::exp(m::ln(s0) + f * (m::ln(s1) - m::ln(s0)));
        }
    }
    rows[rows.len() - 1].1
}

/// Settings for a giant's dynamo.
#[derive(Clone, PartialEq, Debug)]
pub struct GiantDynamo {
    /// Thermal expansivity and specific heat of the deep fluid (French et
    /// al. 2012's adiabat near a megabar: about 1e-5 per K and 1.4e4 J/kg/K).
    pub alpha: f64,
    pub cp: f64,
    /// Steps of the walk down from the cloud tops.
    pub steps: u32,
    /// The host star's metal share where the world gives none (for its
    /// cooling track).
    pub z_default: f64,
}

impl Default for GiantDynamo {
    fn default() -> Self {
        GiantDynamo {
            alpha: 1.0e-5,
            cp: 1.4e4,
            steps: 400,
            z_default: sse_formation::hurley::Z_SUN,
        }
    }
}

/// What the walk and the ledger found.
#[derive(Clone, PartialEq, Debug)]
pub struct GiantField {
    pub dynamo_top_m: f64,
    pub density: f64,
    pub flux_w_m2: f64,
    pub sigma_mid: f64,
    pub flow_m_s: f64,
    pub rm: f64,
    pub onset: f64,
    pub spin_factor: f64,
    /// rms field inside the dynamo, T.
    pub field_t: f64,
}

/// A giant's dynamo, or None when the body is not a giant or the sheet has
/// no profile.
pub fn giant_field(b: &Body, s: &crate::SurfaceSettings, g: &GiantDynamo) -> Option<GiantField> {
    if !b.deep_envelope {
        return None;
    }
    let (rows, metal) = profile()?;
    let gn = sse_core::constants::g();
    let mass = b.mass_earth * sse_core::sheet::M_EARTH;
    let r = b.radius_earth * sse_core::sheet::R_EARTH_EQUATORIAL;
    if mass <= 0.0 || r <= 0.0 {
        return None;
    }
    let rho_c = m::PI * mass / (4.0 * r * r * r);
    let k = 2.0 * gn * r * r / m::PI;
    let rho_at = |x: f64| {
        let xi = m::PI * x;
        if xi <= 0.0 {
            rho_c
        } else {
            rho_c * m::sin(xi) / xi
        }
    };
    let mut top = 0.0;
    for i in 0..=g.steps {
        let x = 1.0 - f64::from(i) / f64::from(g.steps);
        let rho = rho_at(x).max(0.0);
        if sigma_at(&rows, k * rho * rho) >= metal {
            top = x;
            break;
        }
    }
    let empty = GiantField {
        dynamo_top_m: 0.0,
        density: 0.0,
        flux_w_m2: 0.0,
        sigma_mid: 0.0,
        flow_m_s: 0.0,
        rm: 0.0,
        onset: 0.0,
        spin_factor: 0.0,
        field_t: 0.0,
    };
    if top <= 0.0 {
        return Some(empty);
    }
    let rt = top * r;
    // Mass inside the dynamo's top: 4π ρc (R/π)³ (sin ξ − ξ cos ξ).
    let xi = m::PI * top;
    let rp = r / m::PI;
    let m_in = 4.0 * m::PI * rho_c * rp * rp * rp * (m::sin(xi) - xi * m::cos(xi));
    let density = m_in / (4.0 / 3.0 * m::PI * rt * rt * rt);
    let z = match &b.climate {
        Climate::Star { z, .. } => *z,
        Climate::Fixed { then: Some(a), .. } => a.z,
        Climate::Fixed { then: None, .. } => g.z_default,
    };
    let fits = sse_formation::hurley::Fits::new(z);
    let msun = mass / sse_core::sheet::M_SUN;
    let light = sse_formation::body::self_light(
        &fits,
        msun,
        b.age_gyr * 1000.0,
        Some(r / sse_core::sheet::R_SUN),
    );
    let flux = light.luminosity_lsun * sse_core::sheet::L_SUN / (4.0 * m::PI * rt * rt);
    let g_top = gn * m_in / (rt * rt);
    let spin_s = b
        .spin_hours
        .map(|h| h.abs() * 3600.0)
        .filter(|x| *x > 0.0)
        .unwrap_or(s.core.spin_hours_default * 3600.0);
    let (u, _) = lg::flow_speed(
        flux,
        g_top,
        g.alpha,
        density,
        g.cp,
        spin_s,
        rt,
        lg::FLOW_C,
        lg::FLOW_B,
    );
    let rho_mid = rho_at(top / 2.0).max(0.0);
    let sigma_mid = sigma_at(&rows, k * rho_mid * rho_mid);
    let (rm, _) = lg::reynolds(u, rt, sigma_mid, sse_core::sheet::MU0);
    let onset = lg::onset(rm, lg::ONSET_LO, lg::ONSET_HI);
    let (_, spin_factor) = lg::rossby(spin_s, rt, u, lg::ROSSBY_SAT, lg::ROSSBY_SLOPE);
    let field_t =
        sse_atoms::magnetic::dynamo_field_rms(density, flux, s.core.field_c, s.core.field_f_ohm)
            * onset
            * spin_factor;
    Some(GiantField {
        dynamo_top_m: rt,
        density,
        flux_w_m2: flux,
        sigma_mid,
        flow_m_s: u,
        rm,
        onset,
        spin_factor,
        field_t,
    })
}

/// The giant's field facts, as `core::field_facts` gives a rocky world's.
pub fn field_facts(b: &Body) -> Option<Vec<(&'static str, f64)>> {
    let s = crate::scenario::settings();
    let gf = giant_field(b, &s, &GiantDynamo::default())?;
    let r = b.radius_earth * sse_core::sheet::R_EARTH_EQUATORIAL;
    let (b_eq, moment) = if gf.field_t > 0.0 && r > 0.0 {
        sse_atoms::magnetic::surface_dipole(
            gf.field_t,
            gf.dynamo_top_m,
            r,
            sse_atoms::magnetic::DIPOLE_RATIO,
        )
    } else {
        (0.0, 0.0)
    };
    Some(vec![
        ("magnetic.dynamo_field", gf.field_t),
        ("body.surface_magnetic_field", b_eq),
        ("body.magnetic_moment", moment),
        ("interior.dynamo_radius", gf.dynamo_top_m),
        ("interior.dynamo_shell_thickness", gf.dynamo_top_m),
        ("interior.dynamo_flow_speed", gf.flow_m_s),
        ("interior.magnetic_reynolds", gf.rm),
        ("interior.dynamo_onset", gf.onset),
    ])
}
