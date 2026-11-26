//! Y3 part 2 (night sky design section 5): `sky_glow`, the light of every
//! star the sky draw does not show, on a HEALPix map in galactic (l, b) as
//! seen from a place. The physics is `sse_sky::glow`; this is the op.

use crate::{err, Response};
use sse_core::math as m;
use sse_cosmos::sky::SkyParams;
use sse_cosmos::stars::Stars;
use sse_sky::glow::{glow, mag_per_arcsec2_ab, GlowSettings};
use sse_sky::healpix::Grid;

type Out = Result<serde_json::Value, Response>;

pub(crate) struct Ask<'a> {
    pub position_pc: Option<[f64; 3]>,
    pub t_myr: f64,
    pub limit_mag: f64,
    pub band: &'a str,
    pub bands: Vec<String>,
    pub nside: u64,
}

pub(crate) fn sky_glow(st: &Stars, sky: &SkyParams, ask: Ask) -> Out {
    if !ask.t_myr.is_finite() || ask.t_myr < 0.0 {
        return Err(err("bad_request", "t_myr must be 0 or more"));
    }
    if !ask.limit_mag.is_finite() || ask.limit_mag > 30.0 {
        return Err(err("bad_request", "limit_mag must be a number up to 30"));
    }
    if ask.nside > 64 {
        return Err(err("bad_request", "nside is at most 64 for a glow map"));
    }
    let table = sse_light::LightTable::base();
    let mut want = ask.bands.clone();
    if want.is_empty() {
        want = vec!["B".into(), "V".into(), "Ks".into()];
    }
    if !want.iter().any(|b| b == ask.band) {
        want.push(ask.band.to_string());
    }
    let bands =
        sse_light::photometry::bands_of(&table, &want).map_err(|e| err("bad_request", e))?;
    let limit = bands
        .iter()
        .find(|b| b.id == ask.band)
        .copied()
        .ok_or_else(|| err("bad_request", format!("unknown passband {}", ask.band)))?;
    let pc_m = sse_core::Units::standard()
        .get("pc")
        .map_err(|e| err("bad_request", format!("{e:?}")))?
        .scale;
    let obs = ask
        .position_pc
        .unwrap_or_else(|| st.from_grid([st.env.sun_radius_kpc() * 1000.0, 0.0, 0.0], ask.t_myr));
    let set = GlowSettings {
        nside: ask.nside,
        ..GlowSettings::default()
    };
    let g = glow(
        st,
        sky,
        &set,
        obs,
        ask.t_myr,
        ask.limit_mag,
        limit,
        &bands,
        pc_m,
    )
    .map_err(|e| err("bad_request", e))?;
    let grid = Grid::new(g.nside).map_err(|e| err("bad_request", e))?;
    let out_bands: Vec<_> = bands
        .iter()
        .zip(&g.intensity)
        .map(|(b, i)| {
            let mags: Vec<f64> = i
                .iter()
                .map(|x| mag_per_arcsec2_ab(*x, b.effective_m) - b.ab_minus_vega)
                .collect();
            // Whole-sky light as a single magnitude (Vega), for a glance.
            let total: f64 = i.iter().sum::<f64>() * grid.pixel_area();
            let lam = b.effective_m;
            let f_nu = total * lam * lam / sse_core::quantity::consts::C;
            let whole_ab = -2.5 * m::log10(f_nu / sse_light::photometry::AB_ZERO);
            serde_json::json!({
                "band": b.id,
                "intensity": i,
                "mag_arcsec2": mags,
                "whole_sky_ab": whole_ab,
                "whole_sky_mag": whole_ab - b.ab_minus_vega,
            })
        })
        .collect();
    Ok(serde_json::json!({
        "nside": g.nside,
        "order": "ring",
        "frame": "galactic l, b from the observer: l = 0 toward the galaxy's centre, b = 90 toward its north",
        "limit_mag": ask.limit_mag,
        "limit_band": ask.band,
        "observer_pc": obs,
        "t_myr": ask.t_myr,
        "radii_pc": g.radii_pc,
        "bands": out_bands,
        "a_v": g.a_v_total,
        "units": "intensity: I_λ at the band's effective wavelength, W m⁻² m⁻¹ sr⁻¹; mag_arcsec2: Vega magnitudes per square arcsecond",
    }))
}

#[cfg(test)]
mod tests {
    use crate::Kernel;
    use sse_core::math as m;
    use sse_sky::glow::pixel_of_lb;
    use sse_sky::healpix::Grid;

    fn open() -> Kernel {
        let mut k = Kernel::new();
        let r = k.handle_json(r#"{"op":"open_universe","id":"g3","seed":"g3"}"#);
        assert!(r.contains("universe_opened"), "{r}");
        k
    }

    fn ask(k: &mut Kernel, req: &str) -> serde_json::Value {
        serde_json::from_str(&k.handle_json(req)).unwrap()
    }

    fn band<'a>(g: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
        g["bands"]
            .as_array()
            .unwrap()
            .iter()
            .find(|b| b["band"] == id)
            .unwrap()
    }

    fn mag_at(g: &serde_json::Value, id: &str, l: f64, b: f64) -> f64 {
        let grid = Grid::new(g["nside"].as_u64().unwrap()).unwrap();
        band(g, id)["mag_arcsec2"][pixel_of_lb(&grid, l, b) as usize]
            .as_f64()
            .unwrap()
    }

    /// The Milky Way band: brighter along the plane than at the poles, and
    /// toward the centre than the anticentre; dust hides the inner galaxy
    /// in V and much less in Ks; the pole near the measured integrated starlight (about
    /// 23 to 24 V magnitudes per square arcsecond at the galactic poles,
    /// Leinert et al. 1998, doi:10.1051/aas:1998105, table 32, 30 to 60 S10).
    #[test]
    fn the_glow_follows_the_galaxy_and_its_dust() {
        // A Milky Way calibration check: on the median reference galaxy
        // (GZ12; the web's real homes scatter, fudge register).
        crate::cosmos::use_reference_home();
        let mut k = open();
        let g = ask(
            &mut k,
            r#"{"op":"sky_glow","limit_mag":6,"nside":8,"bands":["V","Ks"]}"#,
        );
        assert_eq!(g["type"], "sky_glowed", "{g}");
        let g = &g["glow"];
        let pole = mag_at(g, "V", 0.0, 90.0);
        let south = mag_at(g, "V", 0.0, -90.0);
        assert!((22.0..=25.5).contains(&pole), "north pole V {pole}");
        assert!((pole - south).abs() < 0.5, "poles {pole} {south}");
        // The plane is brighter than the poles, toward the centre most.
        let plane_out = mag_at(g, "V", 90.0, 3.0);
        let centre = mag_at(g, "V", 0.0, -6.0);
        let anti = mag_at(g, "V", 180.0, -6.0);
        // With the arms trailing (24 Nov) the Sun sits between the two arms
        // of the fixed pattern, so the near disc's young stars are thin and
        // the plane reads about 0.8 magnitudes brighter than the pole; the
        // measured sky has 2 to 3 (Leinert et al. 1998), helped by the Local
        // Arm the two-arm pattern lacks (docs/fudges.md, envelope.rs arms).
        // Loosened from 1.0 on 24 Nov (PR 463, log packages/GH3-leading-arm.md):
        // restore 1.0 when the Local Arm lands (recommended to Alex).
        assert!(plane_out < pole - 0.5, "plane {plane_out} pole {pole}");
        assert!(centre < anti, "centre {centre} anticentre {anti}");
        // Dust: along the midplane V sees only the near disc, so the centre
        // is barely brighter than the anticentre; Ks sees through to the
        // inner galaxy. And the midplane lane is deeper in V than in Ks.
        let v_gain = mag_at(g, "V", 180.0, 0.0) - mag_at(g, "V", 0.0, 0.0);
        let k_gain = mag_at(g, "Ks", 180.0, 0.0) - mag_at(g, "Ks", 0.0, 0.0);
        assert!(
            k_gain > v_gain + 0.5,
            "centre over anticentre V {v_gain} Ks {k_gain}"
        );
        let v_lane = mag_at(g, "V", 0.0, 0.0) - mag_at(g, "V", 0.0, -8.0);
        let k_lane = mag_at(g, "Ks", 0.0, 0.0) - mag_at(g, "Ks", 0.0, -8.0);
        assert!(v_lane > k_lane, "lane V {v_lane} Ks {k_lane}");
        let av = g["a_v"].as_array().unwrap();
        let grid = Grid::new(8).unwrap();
        assert!(av[pixel_of_lb(&grid, 0.0, 0.0) as usize].as_f64().unwrap() > 5.0);
    }

    /// No double counting: the whole sky's V light, drawn stars plus glow,
    /// is the same for a draw to V 2 and to V 4 (the glow gives back exactly
    /// what the deeper draw takes), within a few per cent; and the glow's
    /// walk radii are the draw's.
    #[test]
    fn drawn_stars_plus_glow_is_the_same_at_any_limit() {
        let mut k = open();
        let total = |k: &mut Kernel, lim: f64| -> (f64, Vec<f64>, Vec<f64>) {
            let sky = ask(
                k,
                &format!(r#"{{"op":"sky_from_here","limit_mag":{lim},"max_stars":100000}}"#),
            );
            let stars = sky["sky"]["stars"].as_array().unwrap();
            let mut drawn = 0.0;
            for s in stars {
                let v = s["magnitudes"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|x| x["band"] == "V")
                    .unwrap();
                let ab = v["m_ab"].as_f64().unwrap();
                drawn += m::pow(10.0, -0.4 * ab);
            }
            let g = ask(
                k,
                &format!(r#"{{"op":"sky_glow","limit_mag":{lim},"nside":8,"bands":["V"]}}"#),
            );
            let gb = band(&g["glow"], "V");
            // The glow as a total flux in the same AB units.
            let glow_ab = m::pow(10.0, -0.4 * gb["whole_sky_ab"].as_f64().unwrap());
            let radii_draw: Vec<f64> = sky["sky"]["walked"]["classes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c["radius_pc"].as_f64().unwrap())
                .collect();
            let radii_glow: Vec<f64> = g["glow"]["radii_pc"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| r.as_f64().unwrap())
                .collect();
            (drawn + glow_ab, radii_draw, radii_glow)
        };
        let (t2, rd2, rg2) = total(&mut k, 2.0);
        let (t4, rd4, rg4) = total(&mut k, 4.0);
        assert_eq!(rd2, rg2, "radii at V 2");
        assert_eq!(rd4, rg4, "radii at V 4");
        let diff = (t2 - t4).abs() / t4;
        assert!(
            diff < 0.05,
            "whole-sky light {t2} at V 2, {t4} at V 4: {diff}"
        );
    }

    #[test]
    fn a_glow_ask_out_of_range_is_refused() {
        let mut k = open();
        for q in [
            r#"{"op":"sky_glow","t_myr":-1}"#,
            r#"{"op":"sky_glow","limit_mag":31}"#,
            r#"{"op":"sky_glow","band":"Q"}"#,
            r#"{"op":"sky_glow","nside":3}"#,
            r#"{"op":"sky_glow","nside":128}"#,
        ] {
            let r = ask(&mut k, q);
            assert_eq!(r["type"], "error", "{q}: {r}");
        }
    }
}
