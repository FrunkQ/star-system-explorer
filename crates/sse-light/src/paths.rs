//! XO2: what the kernel hands the renderer for light near a mass, op
//! `read_light_paths` (exotic objects design 3.5). Facts only; nothing in
//! it is a picture.
//!
//! For a body and a far viewer: which bender applies, the sizes from the
//! gravity blocks (horizon, photon orbit, last stable orbit, the shadow and
//! Einstein ring as this viewer sees them), the bend table's hash and layout
//! (the table itself is fetched once, keyed by the hash), the feed (XO2),
//! the disc as this viewer sees it, traced point by point (XO3), its light
//! by band, and the viewer's clock rate and tide.
//!
//! What the renderer's `light-path` stage (XO4) needs to repeat on the GPU:
//! the table layout (`sse_gravity::bend::{b_of_row, row_of_b, phi_of_col}`),
//! the crossing angle and first-hit rule of [`crate::disc_image::landing`],
//! the ring temperature [`crate::disc_temperature`] and the formula block
//! `sse.light.disc_shift`. Given those, its picture and these numbers agree.

use crate::disc_image::{self, DiscImage};
use crate::feed::{self, run_block, Feed, FeedAsk, FeedSource};
use crate::*;
use serde::{Deserialize, Serialize};
use sse_gravity::bend;

/// The far viewer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Viewer {
    pub distance_m: f64,
    /// Angle between the line of sight and the body's spin axis (the
    /// disc's axis), degrees: 0 face on, 90 edge on.
    pub tilt_deg: f64,
}

/// One body to read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PathsAsk {
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub mass_kg: f64,
    /// Its surface; 0 for a black hole.
    #[serde(default)]
    pub radius_m: f64,
    #[serde(default)]
    pub spin: f64,
    #[serde(default)]
    pub magnetic_moment_a_m2: f64,
    /// Where matter comes from; none for a bare body.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feed: Option<FeedSource>,
    pub viewer: Viewer,
    /// What the fixture stands for, in words.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Bender {
    /// The bend table: the body is compact enough for rays that miss it to
    /// wind round it.
    Strong,
    /// 4GM/(c²b): rays bend by a small angle.
    Weak,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spin {
    pub a: f64,
    pub axis: String,
}

/// The bend table's layout and hash (the numbers themselves come with
/// `table: true`, once).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableRef {
    pub hash: String,
    pub rows: usize,
    pub captured_rows: usize,
    pub cols: usize,
    pub b_crit_rg: f64,
    pub b_max_rg: f64,
    pub phi_max_rad: f64,
}

pub fn table_ref() -> TableRef {
    TableRef {
        hash: sse_gravity::scenario::table().hash().hex(),
        rows: bend::ROWS,
        captured_rows: bend::CAPTURED_ROWS,
        cols: bend::COLS,
        b_crit_rg: bend::b_crit(),
        b_max_rg: bend::B_MAX,
        phi_max_rad: bend::PHI_MAX,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewerFacts {
    pub tilt_to_disc_rad: f64,
    pub distance_m: f64,
    /// A clock held still at the viewer's distance, against a far clock.
    pub clock_rate_hovering: f64,
    pub tide_per_metre_s2: f64,
}

/// The light the viewer receives, by band (isotropic-equivalent, W).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeenLight {
    pub bands: Vec<BandPower>,
    pub total_w: f64,
    /// The band carrying the most power.
    pub peak_band: String,
    pub description: LightDescription,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LightPaths {
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    pub bender: Bender,
    pub gravitational_radius_m: f64,
    pub spin: Spin,
    pub horizon_m: f64,
    pub photon_orbit_m: f64,
    pub last_stable_orbit_m: f64,
    pub shadow_angle_rad: f64,
    /// For a background far behind the body.
    pub einstein_angle_rad: f64,
    pub table: TableRef,
    pub feed: Option<Feed>,
    /// The disc as this viewer sees it (XO3); none without a disc.
    pub image: Option<DiscImage>,
    pub light: Option<SeenLight>,
    pub viewer: ViewerFacts,
    /// Shortcuts and argued items that touched this answer.
    pub marks: Vec<String>,
    pub blocks: Vec<String>,
    pub why: Vec<String>,
}

/// Read one body.
pub fn read(ask: &PathsAsk, light: &LightTable) -> Result<LightPaths, String> {
    let s = feed::sizes(ask.mass_kg, ask.spin)?;
    let rg = s.gravitational_radius_m;
    let photon = run_block("sse.gravity.photon_orbit", &[rg, ask.spin])?[0];
    let d = ask.viewer.distance_m;
    if d <= 0.0 {
        return Err("the viewer needs a distance above zero".into());
    }
    let shadow = run_block("sse.gravity.shadow_angle", &[rg, d])?[0];
    // A background far behind: D_s much larger than D_l.
    let einstein = run_block("sse.gravity.einstein_angle", &[rg, d, d * 1e12])?[0];
    let clocks = run_block("sse.gravity.clock_rate", &[rg, d])?;
    let tide = run_block("sse.gravity.tide", &[ask.mass_kg, d])?[0];
    let surface = if ask.radius_m > 0.0 {
        ask.radius_m
    } else {
        s.horizon_m
    };
    let bender = if surface < bend::B_MAX * rg {
        Bender::Strong
    } else {
        Bender::Weak
    };
    let tilt = ask.viewer.tilt_deg.clamp(0.0, 90.0) * m::PI / 180.0;
    let mut blocks: Vec<String> = [
        "sse.gravity.photon_orbit",
        "sse.gravity.shadow_angle",
        "sse.gravity.einstein_angle",
        "sse.gravity.clock_rate",
        "sse.gravity.tide",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let mut marks = vec![];
    let mut why = vec![format!(
        "{:.3e} kg: gravitational radius {rg:.4e} m; a viewer {d:.3e} m away sees its shadow {:.4e} rad across (radius {shadow:.4e} rad).",
        ask.mass_kg,
        2.0 * shadow,
    )];
    if ask.spin != 0.0 {
        marks.push("a spinning hole's light paths are drawn with the no-spin table until XO7 (fudge register: exotic objects)".into());
    }
    let (fed, image, seen) = match &ask.feed {
        None => {
            why.push("Nothing falls onto it, so it gives off no light of its own: it shows only by bending the light behind it.".into());
            (None, None, None)
        }
        Some(src) => {
            let f = feed::describe(&FeedAsk {
                mass_kg: ask.mass_kg,
                radius_m: ask.radius_m,
                spin: ask.spin,
                magnetic_moment_a_m2: ask.magnetic_moment_a_m2,
                source: src.clone(),
            })?;
            for b in &f.blocks {
                if !blocks.contains(b) {
                    blocks.push(b.clone());
                }
            }
            let mut desc = LightDescription::new(&ask.address);
            let mut image = None;
            let disc_rate = f.disc_rate_kg_s();
            if disc_rate > 0.0 && f.disc.outer_m > f.disc.inner_m {
                let img = disc_image::trace(
                    ask.mass_kg,
                    disc_rate,
                    f.disc.inner_m,
                    f.disc.outer_m,
                    tilt,
                    sse_gravity::scenario::table(),
                    light,
                )?;
                desc.parts.push(disc_image::part(&img));
                desc.why.extend(img.why.iter().cloned());
                desc.stand_ins.extend(img.stand_ins.iter().cloned());
                desc.references.push(disc_image::luminet_reference());
                desc.references.push(Reference {
                    text: R_SS73.into(),
                    doi: None,
                    url: Some("https://ui.adsabs.harvard.edu/abs/1973A&A....24..337S".into()),
                });
                blocks.push("sse.light.disc_shift".into());
                image = Some(img);
            }
            let hot = f.hot_power_w();
            if hot > 0.0 {
                desc.parts.push(Part::PowerLaw {
                    power_w: hot,
                    from_m: feed::HOT_FROM_M,
                    to_m: feed::HOT_TO_M,
                    alpha: feed::HOT_ALPHA,
                    cause: "hot thin flow".into(),
                });
            }
            desc.why.extend(f.why.iter().cloned());
            desc.stand_ins.extend(f.stand_ins.iter().cloned());
            let bands = band_powers(&desc, &light.bands, light);
            let total_w: f64 = bands.iter().map(|b| b.power_w).sum();
            let peak_band = bands
                .iter()
                .fold(None::<&BandPower>, |a, b| match a {
                    Some(x) if x.power_w >= b.power_w => Some(x),
                    _ => Some(b),
                })
                .map(|b| b.band.clone())
                .unwrap_or_default();
            why.push(format!(
                "Seen from {:.0}° off its axis it gives off {total_w:.3e} W (as if alike in every direction), most of it in {peak_band}.",
                ask.viewer.tilt_deg
            ));
            for st in &desc.stand_ins {
                if !marks.contains(st) {
                    marks.push(st.clone());
                }
            }
            let seen = SeenLight {
                bands,
                total_w,
                peak_band,
                description: desc,
            };
            (Some(f), image, Some(seen))
        }
    };
    Ok(LightPaths {
        address: ask.address.clone(),
        name: ask.name.clone(),
        about: ask.about.clone(),
        bender,
        gravitational_radius_m: rg,
        spin: Spin {
            a: ask.spin,
            axis: "the body's spin axis; the disc lies in its equator".into(),
        },
        horizon_m: s.horizon_m,
        photon_orbit_m: photon,
        last_stable_orbit_m: s.last_stable_orbit_m,
        shadow_angle_rad: shadow,
        einstein_angle_rad: einstein,
        table: table_ref(),
        feed: fed,
        image,
        light: seen,
        viewer: ViewerFacts {
            tilt_to_disc_rad: tilt,
            distance_m: d,
            clock_rate_hovering: clocks[0],
            tide_per_metre_s2: tide,
        },
        marks,
        blocks,
        why,
    })
}
