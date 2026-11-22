//! MF7 part 2: electron radiation belts (magnetic fields design 6.4).
//!
//! A belt exists where field lines close and electrons can bounce without
//! striking air, rings or the bubble's edge: from the top of the air (or the
//! rings' outer edge) to the last closed drift shell, a share of the
//! stand-off. Its intensity is capped by the Kennel-Petschek limit
//! (`sse.belts.kp_limit`), filled to the share that measured belts reach.
//! Moons on a shell sweep it. Doses are given behind three shields.

use super::*;
use sse_atoms::belts::{self as bl, Electron, Shape};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BeltSettings {
    /// Spectrum at the limit for a belt fed by the wind (Earth, Uranus) and
    /// for one fed by a spinning, moon-loaded magnetosphere (Jupiter).
    pub index: f64,
    pub break_kev: f64,
    pub break_index: f64,
    pub spun_break_kev: f64,
    pub spun_break_index: f64,
    pub anisotropy: f64,
    pub gain_ln: f64,
    pub wpe_over_w: f64,
    /// Share of the limit the belt is filled to.
    pub fill: f64,
    /// Last closed drift shell as a share of the stand-off.
    pub closed_share: f64,
    /// Where a wind-driven belt is fed by injections, as a share of the
    /// stand-off (Earth's most intense belt sits near L 5 of 10).
    pub source_share: f64,
    /// The air stops trapped electrons this many scale heights up.
    pub air_scale_heights: f64,
    /// Wave path, planetary radii, where moons' plasma is held near the
    /// equator by spin (instead of D = L).
    pub confined_path_radii: f64,
    /// Shields, kg/m² (0.3, 1 and 3 g/cm²).
    pub shields_kg_m2: [f64; 3],
    /// Stopping power of tissue for the electrons that get through, MeV cm²/g.
    pub stopping_mev_cm2_g: f64,
    pub shells: usize,
}

impl Default for BeltSettings {
    fn default() -> Self {
        BeltSettings {
            index: 1.0,
            break_kev: 1500.0,
            break_index: 6.0,
            spun_break_kev: 2500.0,
            spun_break_index: 1.75,
            anisotropy: 0.3,
            gain_ln: 3.0,
            wpe_over_w: 300.0,
            fill: 0.6,
            closed_share: 0.7,
            source_share: 0.5,
            air_scale_heights: 100.0,
            confined_path_radii: 3.0,
            shields_kg_m2: [3.0, 10.0, 30.0],
            stopping_mev_cm2_g: 1.85,
            shells: 16,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BeltShell {
    pub distance_m: f64,
    pub l: f64,
    pub field_t: f64,
    /// The limit's normalisation C_K and the belt's (per cm² s sr keV at 1 keV).
    pub limit_c: f64,
    pub belt_c: f64,
    /// Equatorial intensity at 1 MeV, per cm² s sr keV.
    pub intensity_1mev: f64,
    /// Electrons above 1 MeV crossing a square centimetre each second, from all directions.
    pub flux_above_1mev_cm2_s: f64,
    /// Dose rate, Sv an hour, behind each shield.
    pub dose_sv_h: [f64; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swept_by: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BeltGap {
    pub by: String,
    pub from_m: f64,
    pub to_m: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BeltDose {
    pub distance_m: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub moon: Option<String>,
    pub flux_above_1mev_cm2_s: f64,
    pub dose_sv_h: [f64; 3],
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Belts {
    pub inner_m: f64,
    pub outer_m: f64,
    /// What sets each edge: "air", "rings", "surface"; "bubble".
    pub inner_set_by: String,
    pub outer_set_by: String,
    /// Wave path rule: "L" or "confined".
    pub wave_path: String,
    /// The shell injections feed; inside it the belt keeps that shell's
    /// intensity (it is carried inward, not filled to the higher limit).
    pub source_m: f64,
    pub source_set_by: String,
    pub break_kev: f64,
    pub break_index: f64,
    pub shields_kg_m2: [f64; 3],
    /// Electron energy, MeV, that crosses each shield.
    pub shield_cut_mev: [f64; 3],
    pub shells: Vec<BeltShell>,
    pub gaps: Vec<BeltGap>,
    pub orbits: Vec<BeltDose>,
}

/// Belts inside a body's own field. `reach` is the field's stand-off.
#[allow(clippy::too_many_arguments)]
pub(crate) fn belts_of(
    b: &Body,
    set: &Settings,
    reach: f64,
    q: Option<&Electron>,
    why: &mut Vec<String>,
    stand_ins: &mut Vec<String>,
    used: &mut Vec<&'static str>,
) -> Option<Belts> {
    let bs = &set.belts;
    let r = b.radius_m;
    if b.moment_a_m2 <= 0.0 || reach <= r {
        return None;
    }
    let q = match q {
        Some(q) => q,
        None => {
            why.push("Belts: not computed, as the electron's mass and charge were not supplied from the data sheet.".into());
            return None;
        }
    };
    // Inner edge: air, rings or surface.
    let air_top = b
        .air
        .as_ref()
        .map(|a| r + bs.air_scale_heights * a.scale_height_m)
        .unwrap_or(r);
    let (mut inner, mut inner_by) = (r, "surface");
    if air_top > inner {
        inner = air_top;
        inner_by = "air";
    }
    if b.rings_outer_m > inner {
        inner = b.rings_outer_m;
        inner_by = "rings";
    }
    let outer = bs.closed_share * reach;
    if outer <= inner * 1.05 {
        why.push(format!("Belts: none. Its bubble closes {} out, too near the {} for electrons to drift round without striking it.", radii(outer, r), inner_by));
        return None;
    }
    used.extend(["sse.belts.kp_limit", "sse.belts.electron_range"]);
    let spun = b.plasma_source_kg_s > 0.0;
    let shape = Shape {
        index: bs.index,
        break_kev: if spun {
            bs.spun_break_kev
        } else {
            bs.break_kev
        },
        break_index: if spun {
            bs.spun_break_index
        } else {
            bs.break_index
        },
        anisotropy: bs.anisotropy,
    };
    let b0 = MU0 * b.moment_a_m2 / (4.0 * m::PI * r * r * r);
    let field = |d: f64| b0 * (r / d) * (r / d) * (r / d);
    let path = |d: f64| {
        let l = d / r;
        if spun {
            l.min(bs.confined_path_radii) * r
        } else {
            l * r
        }
    };
    // The cap at one shell, scaled exactly as B / (D R_p) elsewhere.
    let c_ref = bl::kp_cap(
        field(inner),
        path(inner),
        bs.wpe_over_w,
        bs.gain_ln,
        &shape,
        q,
    );
    let limit = |d: f64| c_ref * (field(d) / field(inner)) * (path(inner) / path(d));
    // The source shell: a spun belt is fed outward from the innermost moon
    // in it (a stand-in for the moon that loads the plasma); a wind-driven
    // one from injections near half the stand-off.
    let (source, source_by) = match b
        .moons
        .iter()
        .filter(|mo| spun && mo.orbit_m > inner && mo.orbit_m < outer)
        .min_by(|a, c| a.orbit_m.total_cmp(&c.orbit_m))
    {
        Some(mo) => (mo.orbit_m, mo.address.clone()),
        None => (
            (bs.source_share * reach).clamp(inner, outer),
            "injections".to_string(),
        ),
    };
    let cap = |d: f64| limit(d.max(source));
    // Omnidirectional factor: ∫ sin^(2S) α dΩ.
    let omni = {
        let n = 180;
        let mut s = 0.0;
        for i in 0..n {
            let a = (i as f64 + 0.5) / n as f64 * m::PI;
            s += m::pow(m::sin(a), 2.0 * bs.anisotropy) * m::sin(a);
        }
        2.0 * m::PI * s * m::PI / n as f64
    };
    // Omnidirectional flux above e_kev for C = 1, per cm² s.
    let flux_above = |e_kev: f64| {
        let n = 120;
        let (lo, hi) = (e_kev.max(1.0), 1e5);
        let ln = m::ln(hi / lo);
        let mut s = 0.0;
        for i in 0..n {
            let e = lo * m::exp(ln * (i as f64 + 0.5) / n as f64);
            s += shape.at(e) * e * ln / n as f64;
        }
        omni * s
    };
    let cut: Vec<f64> = bs
        .shields_kg_m2
        .iter()
        .map(|x| bl::penetration_energy(*x))
        .collect();
    let per_c: Vec<f64> = cut.iter().map(|c| flux_above(c * 1e3)).collect();
    let above_1mev = flux_above(1e3);
    // MeV cm²/g to J m²/kg, and per second to per hour.
    let s_si = bs.stopping_mev_cm2_g * 1e6 * q.charge_c * 1e-4 / 1e-3;
    let dose = |c: f64| -> [f64; 3] {
        let mut d = [0.0; 3];
        for (k, v) in d.iter_mut().enumerate() {
            *v = c * per_c[k] * 1e4 * s_si * 3600.0;
        }
        d
    };
    // Gaps swept by moons on a shell inside the belt.
    let mut gaps = vec![];
    for mo in &b.moons {
        if mo.orbit_m > inner && mo.orbit_m < outer {
            let w = mo.radius_m.max(0.005 * mo.orbit_m);
            gaps.push(BeltGap {
                by: mo.address.clone(),
                from_m: mo.orbit_m - w,
                to_m: mo.orbit_m + w,
            });
        }
    }
    let n = bs.shells.max(2);
    let mut shells = vec![];
    for i in 0..n {
        let d = inner * m::pow(outer / inner, (i as f64 + 0.5) / n as f64);
        let swept = gaps
            .iter()
            .find(|g| d >= g.from_m && d <= g.to_m)
            .map(|g| g.by.clone());
        let c_k = limit(d);
        let c = if swept.is_some() {
            0.0
        } else {
            bs.fill * cap(d)
        };
        shells.push(BeltShell {
            distance_m: d,
            l: d / r,
            field_t: field(d),
            limit_c: c_k,
            belt_c: c,
            intensity_1mev: c * shape.at(1e3),
            flux_above_1mev_cm2_s: c * above_1mev,
            dose_sv_h: dose(c),
            swept_by: swept,
        });
    }
    // Doses at chosen orbits and at each moon (a moon meets the electrons
    // drifting onto its shell, so its own gap does not shelter it).
    let mut orbits = vec![];
    let mut targets: Vec<(f64, Option<String>)> = b.orbits_m.iter().map(|d| (*d, None)).collect();
    targets.extend(
        b.moons
            .iter()
            .map(|mo| (mo.orbit_m, Some(mo.address.clone()))),
    );
    for (d, moon) in targets {
        let c = if d > inner && d < outer {
            bs.fill * cap(d)
        } else {
            0.0
        };
        orbits.push(BeltDose {
            distance_m: d,
            moon,
            flux_above_1mev_cm2_s: c * above_1mev,
            dose_sv_h: dose(c),
        });
    }
    let peak = shells
        .iter()
        .filter(|s| s.swept_by.is_none())
        .fold(None::<&BeltShell>, |a, s| match a {
            Some(x) if x.dose_sv_h[1] >= s.dose_sv_h[1] => Some(x),
            _ => Some(s),
        });
    why.push(format!(
        "Belts: electrons are trapped from {} ({}) out to {} (where drift paths still close inside the bubble). Fed at {} ({}), they are pumped up until the waves they drive scatter them away, so the belt sits near that limit there and outward, and keeps that intensity as it spreads inward{}{}",
        radii(inner, r),
        match inner_by { "rings" => "the rings' outer edge", "air" => "the top of the air", _ => "the surface" },
        radii(outer, r),
        radii(source, r),
        if source_by == "injections" { "by injections from the night side".to_string() } else { format!("outward from {source_by}'s plasma") },
        if spun { "; with moons loading the spinning field, the spectrum stays hard to tens of MeV." } else { "." },
        match peak {
            Some(p) => format!(" Behind 1 g/cm² of shielding the dose peaks at about {:.3} Sv an hour near {:.1} radii.", p.dose_sv_h[1], p.l),
            None => String::new(),
        }
    ));
    for g in &gaps {
        why.push(format!(
            "{} sweeps a gap at {:.1} radii: it absorbs the electrons that drift onto its path.",
            g.by,
            (g.from_m + g.to_m) / 2.0 / r
        ));
    }
    stand_ins.push("belts: electrons only; filled to 0.6 of the Kennel-Petschek limit in a dense plasma at and beyond the source shell (half the stand-off, or the innermost moon of a spun belt) and kept flat inward of it; this is the belt after strong injections, often ten times above quiet times (a quiet magnetosphere like Neptune's, and gas and dust like Saturn's E ring, keep real belts lower; Jupiter's falls faster outward than the limit); the last closed drift shell is 0.7 of the stand-off; moons sweep a gap of their own width; no slot from wave losses inside a plasmasphere yet, no proton belt; dose is the electrons that pass the shield times a constant stopping power, without bremsstrahlung".into());
    Some(Belts {
        inner_m: inner,
        outer_m: outer,
        inner_set_by: inner_by.into(),
        outer_set_by: "bubble".into(),
        wave_path: if spun { "confined".into() } else { "L".into() },
        source_m: source,
        source_set_by: source_by,
        break_kev: shape.break_kev,
        break_index: shape.break_index,
        shields_kg_m2: bs.shields_kg_m2,
        shield_cut_mev: [cut[0], cut[1], cut[2]],
        shells,
        gaps,
        orbits,
    })
}
