//! XO2 and XO3 checks (exotic objects design 3.4, 3.5 and section 11).

use crate::disc_image::{self, landing};
use crate::feed::{self, run_block, FeedAsk, FeedSource, FlowKind, InnerSetBy};
use crate::paths::{self, Bender};
use crate::*;

const MSUN: f64 = sse_core::sheet::M_SUN;

/// Run `f` with the shipped physics in force (base reference and base
/// physics packs), as a kernel call would.
fn in_force<R>(f: impl FnOnce() -> R) -> R {
    use std::sync::{Arc, OnceLock};
    static P: OnceLock<Arc<sse_physics::Physics>> = OnceLock::new();
    let p = P.get_or_init(|| {
        let packs: Vec<sse_pack::PackManifest> = ["base-reference", "base-physics"]
            .iter()
            .map(|n| {
                let path = format!("{}/../../packs/{n}/pack.json", env!("CARGO_MANIFEST_DIR"));
                let text = std::fs::read_to_string(&path).expect("pack file");
                sse_pack::PackManifest::from_json(&text).expect("pack")
            })
            .collect();
        let sheet = Arc::new(sse_reference::Reference::from_packs(&packs).expect("sheet"));
        Arc::new(sse_physics::Physics::build(
            &packs,
            None,
            sheet,
            Arc::new(Default::default()),
        ))
    });
    sse_physics::with(p, sse_core::constants::Constants::REAL, f)
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs()
}

fn table() -> LightTable {
    LightTable::base()
}

fn band(l: &paths::SeenLight, id: &str) -> f64 {
    l.bands.iter().find(|b| b.band == id).unwrap().power_w
}

fn strength(f: &feed::Feed, k: FlowKind) -> f64 {
    f.flows.iter().find(|x| x.kind == k).unwrap().strength
}

/// Eddington luminosity 1.26e31 W per Sun (textbook figure, Frank, King
/// and Raine 2002 eq. 1.2: 1.3e31 W for M in Suns), efficiency 5.7% at
/// a non-spinning hole's last stable orbit and about 32% at a = 0.998
/// (Thorne 1974's limit; unverified from memory).
#[test]
fn eddington_and_efficiency() {
    in_force(|| {
        let e = run_block("sse.feed.eddington", &[MSUN, 0.1]).unwrap();
        assert!(rel(e[0], 1.257e31) < 1e-3, "{}", e[0]);
        let rg = 1.0;
        assert!(
            rel(
                run_block("sse.feed.efficiency", &[rg, 6.0]).unwrap()[0],
                0.0572
            ) < 1e-3
        );
        let isco = run_block("sse.gravity.last_stable_orbit", &[rg, 0.998]).unwrap()[0];
        let eta = run_block("sse.feed.efficiency", &[rg, isco]).unwrap()[0];
        assert!((0.30..0.33).contains(&eta), "{eta}");
        // Far out it is the Newtonian GM / 2r.
        let far = run_block("sse.feed.efficiency", &[rg, 1e5]).unwrap()[0];
        assert!(rel(far, 0.5e-5) < 1e-3, "{far}");
    })
}

/// The inner edge rule gives a hole's last stable orbit, a white dwarf's
/// surface and a magnetic neutron star's field edge, with no kind asked.
#[test]
fn the_inner_edge_is_whichever_is_largest() {
    in_force(|| {
        let given = |rate| FeedSource::Given {
            rate_kg_s: rate,
            outer_m: 1e9,
            why: "test".into(),
        };
        let hole = feed::describe(&FeedAsk {
            mass_kg: 10.0 * MSUN,
            radius_m: 0.0,
            spin: 0.0,
            magnetic_moment_a_m2: 0.0,
            source: given(1e15),
        })
        .unwrap();
        assert_eq!(hole.disc.inner_set_by, InnerSetBy::LastStableOrbit);
        let rg = 10.0 * 1476.6;
        assert!(rel(hole.disc.inner_m / rg, 6.0) < 1e-3);
        let dwarf = feed::describe(&FeedAsk {
            mass_kg: 0.6 * MSUN,
            radius_m: 8.7e6,
            spin: 0.0,
            magnetic_moment_a_m2: 0.0,
            source: given(1e14),
        })
        .unwrap();
        assert_eq!(dwarf.disc.inner_set_by, InnerSetBy::Surface);
        assert!(dwarf.efficiency < 2e-4, "{}", dwarf.efficiency);
        // An X-ray pulsar: 1e30 G cm³ = 1e27 A m², fed 1e14 kg/s: the disc
        // stops near 1,500 km (Frank, King and Raine's r_M of order 10⁸ cm).
        let pulsar = feed::describe(&FeedAsk {
            mass_kg: 1.4 * MSUN,
            radius_m: 1.2e4,
            spin: 0.0,
            magnetic_moment_a_m2: 1e27,
            source: given(1e14),
        })
        .unwrap();
        assert_eq!(pulsar.disc.inner_set_by, InnerSetBy::MagneticField);
        assert!(
            (1.0e6..2.5e6).contains(&pulsar.disc.inner_m),
            "{}",
            pulsar.disc.inner_m
        );
    })
}

/// A disc far from the hole, where light hardly bends and the gas moves
/// slowly: face on it sends twice its average light towards the viewer,
/// and the share falls as the cosine of the tilt (a flat glowing surface).
/// The traced total is then the plain Shakura and Sunyaev luminosity with
/// these factors: an absolute check of the image-plane sum.
#[test]
fn far_out_the_traced_disc_is_a_flat_glowing_surface() {
    in_force(|| {
        let t = table();
        let mass = 10.0 * MSUN;
        let rg = run_block("sse.gravity.gravitational_radius", &[mass]).unwrap()[0];
        let (r_in, r_out, mdot) = (2000.0 * rg, 6e4 * rg, 1e15);
        let mut d = LightDescription::new("flat");
        d.parts.push(Part::Disc {
            mass_kg: mass,
            accretion_kg_s: mdot,
            r_inner_m: r_in,
            r_outer_m: r_out,
            cause: "test".into(),
        });
        let avg = publish(&d, &t).total_w;
        let x = r_in / r_out;
        let ss73 = G * mass * mdot / (2.0 * r_in) * (1.0 - 3.0 * x + 2.0 * x * m::sqrt(x));
        let table = sse_gravity::scenario::table();
        let face = disc_image::trace(mass, mdot, r_in, r_out, 0.0, table, &t).unwrap();
        assert!(
            rel(face.luminosity_w, 2.0 * ss73) < 0.02,
            "{} vs {}",
            face.luminosity_w,
            2.0 * ss73
        );
        assert!(rel(face.luminosity_w, 2.0 * avg) < 0.03);
        let tilted = disc_image::trace(mass, mdot, r_in, r_out, m::PI / 3.0, table, &t).unwrap();
        assert!(rel(tilted.luminosity_w / face.luminosity_w, 0.5) < 0.02);
        // Seen face on, no side turns towards the viewer.
        assert!(rel(face.side_ratio, 1.0) < 1e-9, "{}", face.side_ratio);
        // Binning keeps every piece's σT⁴: the pieces carry the total exactly.
        let sum: f64 = face
            .pieces
            .iter()
            .map(|[t, a]| a * SIGMA * t * t * t * t)
            .sum();
        assert!(rel(sum, face.luminosity_w) < 1e-12);
    })
}

/// Luminet's view (10° above the disc): the far side's light is bent over
/// the top as the arch, a thin ring of rays that went round once shows,
/// and the side turning towards the viewer is the brighter, by the shift
/// block's own factor at the brightest point.
#[test]
fn luminet_view_shows_the_arch_the_ring_and_the_bright_side() {
    in_force(|| {
        let table = sse_gravity::scenario::table();
        let tilt = 80.0 * m::PI / 180.0;
        // Straight up the picture, rays at b = 8 to 20 land on the far side
        // after sweeping almost half a turn (the arch). In flat space they
        // would reach the disc's plane at b / sin 170°, almost six times
        // further out; the bending brings the far side up over the top.
        let phi0 = m::PI / 2.0 + tilt;
        for b in [8.0, 12.0, 20.0] {
            let (order, r) = landing(table, b, 0.0, tilt, 6.0, 1e3).expect("lands");
            assert_eq!(order, 0);
            assert!(r > b && r < 0.5 * b / m::sin(phi0), "b {b}: r {r}");
        }
        // Inside the shadow's edge straight down, a ray falls in before it
        // reaches the far crossing: the dark middle.
        assert!(landing(table, 3.0, 0.0, 0.0, 6.0, 1e3).is_none());
        let lp = paths::read(&crate::scenario::fixtures()[0], &table_light()).unwrap();
        let img = lp.image.as_ref().unwrap();
        assert!(
            img.order_shares[1] > 0.0 && img.order_shares[1] < 0.1,
            "{:?}",
            img.order_shares
        );
        assert!(img.side_ratio > 3.0, "{}", img.side_ratio);
        assert!(
            img.brightest.image_angle_rad > 0.0,
            "brightest on the approaching (left) side"
        );
        assert!(
            img.brightest.shift > 1.0,
            "blueshifted: {}",
            img.brightest.shift
        );
        // The brightness ratio is g⁴ T⁴ between the two brightest points.
        let (a, r) = (&img.approaching, &img.receding);
        let want = m::pow(a.shift * a.t_emitted_k / (r.shift * r.t_emitted_k), 4.0);
        assert!(rel(img.side_ratio, want) < 1e-9);
    })
}

fn table_light() -> LightTable {
    table()
}

/// Design 3.4 as band totals: the 10 Sun hole peaks in X-rays, the quasar
/// in the ultraviolet and outshines a galaxy in visible light, and the
/// starved centre shows almost nothing in visible light.
#[test]
fn the_three_fixtures_reproduce_the_columns_of_design_3_4() {
    in_force(|| {
        let t = table();
        let all: Vec<_> = crate::scenario::fixtures()
            .iter()
            .map(|a| paths::read(a, &t).unwrap())
            .collect();
        for lp in &all {
            assert_eq!(lp.bender, Bender::Strong);
            assert!(rel(lp.last_stable_orbit_m / lp.gravitational_radius_m, 6.0) < 1e-9);
        }
        let (hole, quasar, starved) = (&all[0], &all[1], &all[2]);
        let hl = hole.light.as_ref().unwrap();
        assert_eq!(hl.peak_band, "xray", "{:?}", hl.bands);
        let hf = hole.feed.as_ref().unwrap();
        assert!(
            (0.05..0.2).contains(&hf.eddington_share),
            "{}",
            hf.eddington_share
        );
        assert!(strength(hf, FlowKind::Thin) > 0.95);
        // The disc reaches 0.6 a / (1 + q) out (Paczyński 1977).
        assert!(rel(hf.disc.outer_m, 0.6 * 4e9 / 1.1) < 1e-9);
        let ql = quasar.light.as_ref().unwrap();
        assert_eq!(ql.peak_band, "uv", "{:?}", ql.bands);
        // The Milky Way gives off about 1e37 W (some 2.5e10 Suns, unverified);
        // the quasar's disc outshines a galaxy like it in visible light alone.
        assert!(band(ql, "visible") > 1e37, "{}", band(ql, "visible"));
        let sf = starved.feed.as_ref().unwrap();
        assert!(sf.eddington_share < 1e-3, "{}", sf.eddington_share);
        assert!(strength(sf, FlowKind::Hot) > 0.99);
        let sl = starved.light.as_ref().unwrap();
        assert!(band(sl, "visible") < 1e-6 * sf.eddington_luminosity_w);
        assert!(band(sl, "visible") < 1e-3 * band(ql, "visible") * 1e-3);
        // The Milky Way's centre seen from 8.2 kpc: shadow about 50 µas across
        // (design section 11; the EHT measured 51.8 ± 2.3, unverified).
        let across_uas = 2.0 * starved.shadow_angle_rad * 180.0 / m::PI * 3600.0 * 1e6;
        assert!((48.0..54.0).contains(&across_uas), "{across_uas}");
        // The well-fed discs are far too bright to look at.
        assert!(
            hole.image
                .as_ref()
                .unwrap()
                .exposure
                .times_sun_surface
                .unwrap()
                > 1e3
        );
    })
}

/// A bare hole gives no light: it shows only by bending.
#[test]
fn a_bare_hole_is_dark() {
    in_force(|| {
        let mut a = crate::scenario::fixtures()[0].clone();
        a.feed = None;
        let lp = paths::read(&a, &table()).unwrap();
        assert!(lp.light.is_none() && lp.image.is_none() && lp.feed.is_none());
        assert_eq!(lp.bender, Bender::Strong);
    })
}

/// The chart and the picture agree: `read_light` with a tilt gives the
/// same band totals as `read_light_paths` for the same disc.
#[test]
fn read_light_with_a_tilt_matches_the_traced_picture() {
    in_force(|| {
        let t = table();
        let lp = paths::read(&crate::scenario::fixtures()[0], &t).unwrap();
        let f = lp.feed.as_ref().unwrap();
        let model = emit::Model::base();
        let c = emit::Conditions {
            address: "fixture/stellar-hole".into(),
            radius_m: lp.horizon_m,
            accretion: Some(emit::Accretion {
                mass_kg: 10.0 * MSUN,
                rate_kg_s: f.disc_rate_kg_s(),
                r_inner_m: f.disc.inner_m,
                r_outer_m: f.disc.outer_m,
                tilt_deg: Some(80.0),
            }),
            ..Default::default()
        };
        let (d, _) = emit::describe(&c, model).unwrap();
        let chart = publish(&d, &model.table);
        // The picture's disc alone (its light also holds the hot flow's glow).
        let mut disc = LightDescription::new("disc");
        disc.parts = lp
            .light
            .as_ref()
            .unwrap()
            .description
            .parts
            .iter()
            .filter(|p| matches!(p, Part::Thermals { .. }))
            .cloned()
            .collect();
        let pic = band_powers(&disc, &model.table.bands, &model.table);
        let total: f64 = pic.iter().map(|b| b.power_w).sum();
        for b in &pic {
            let c = chart.bands.iter().find(|x| x.band == b.band).unwrap();
            if b.power_w > 1e-9 * total {
                assert!(
                    rel(c.power_w, b.power_w) < 1e-12,
                    "{}: {} vs {}",
                    b.band,
                    c.power_w,
                    b.power_w
                );
            }
        }
    })
}
