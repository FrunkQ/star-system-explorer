use std::collections::BTreeMap;

use crate::fog::*;
use sse_light::emit::{Model, Source};

fn sun_at_1au() -> Vec<Source> {
    let s = sse_reference::Reference::base();
    vec![Source {
        address: "sun".into(),
        luminosity_w: s.constant("L_sun").unwrap(),
        t_k: s.constant("T_sun").unwrap(),
        distance_m: s.constant("au").unwrap(),
    }]
}

/// A 50 m hull in deep space, lit by no star (a sunlit hull is seen much
/// further: at 1 au a 50 m hull is about 11th magnitude from a million km).
fn hull(heat_w: f64, plume: Option<Plume>) -> Craft {
    let mut heat = BTreeMap::new();
    if heat_w > 0.0 {
        heat.insert(400, heat_w);
    }
    Craft {
        address: "kestrel".into(),
        radius_m: 25.0,
        bond_albedo: 0.3,
        lit_by: vec![],
        heat_by_k: heat,
        plumes: plume.into_iter().collect(),
        transmitters: vec![],
    }
}

fn contact(c: Craft, d: f64) -> Contact {
    Contact {
        craft: c,
        distance_m: d,
        extinction_av: 0.0,
        lost_for_s: None,
        speed_m_s: 0.0,
        accel_m_s2: 0.0,
    }
}

#[test]
fn a_sunlit_hull_is_seen_further_than_a_dark_one() {
    let m = Model::base();
    let eyes = default_fit(&base_fits().unwrap(), "today", 50.0).unwrap();
    let mut lit = hull(1e5, None);
    lit.lit_by = sun_at_1au();
    let dark = hull(1e5, None);
    assert!(noticed_to(&lit, &eyes, m).unwrap() > 3.0 * noticed_to(&dark, &eyes, m).unwrap());
}

#[test]
fn the_default_fit_grows_with_the_hull_and_stops_at_its_cap() {
    let fits = base_fits().unwrap();
    let small = default_fit(&fits, "today", 10.0).unwrap();
    let big = default_fit(&fits, "today", 1000.0).unwrap();
    assert_eq!(small.len(), 4);
    assert!((small[0].aperture_m - 0.1).abs() < 1e-12, "floor");
    assert!((big[0].aperture_m - 1.0).abs() < 1e-12, "cap");
    assert!(default_fit(&fits, "far", 10.0)
        .unwrap_err()
        .contains("today"));
}

#[test]
fn a_fusion_torch_is_seen_far_off_and_a_coasting_hull_only_close() {
    let m = Model::base();
    let eyes = default_fit(&base_fits().unwrap(), "today", 50.0).unwrap();
    // A 1 GW-jet fusion torch at 3000 km/s: its plasma is hundreds of
    // millions of kelvin, X-ray hot.
    let torch = Plume {
        jet_power_w: 1e9,
        exhaust_m_s: 3e6,
        particle_kg: None,
    };
    let lit = hull(1e6, Some(torch.clone()));
    let d = signature(&lit, m).unwrap();
    let plume_t = d
        .parts
        .iter()
        .find_map(|p| match p {
            sse_light::Part::Thermal { t_k, cause, .. } if cause == "drive plume" => Some(*t_k),
            _ => None,
        })
        .unwrap();
    assert!(plume_t > 1e8 && plume_t < 1e9, "{plume_t}");
    let cold = hull(1e5, None);
    // There is no hiding a warm hull: 100 kW of waste heat at 400 K shows
    // in a cooled 0.4 m infrared telescope out to about 1e10 m against the
    // zodiacal glow. The torch's own heat carries it further.
    let far = 2e10;
    let a = mark(&contact(lit.clone(), far), &eyes, m).unwrap();
    let b = mark(&contact(cold.clone(), far), &eyes, m).unwrap();
    assert_eq!(a.mark, "dotted", "{:?}", a.why);
    assert_eq!(b.mark, "unseen", "{:?}", b.why);
    assert!(a.noticed_to_m > 2.0 * b.noticed_to_m);
    assert!((a.seen_as_of_s - 66.71).abs() < 1e-2, "{}", a.seen_as_of_s);
    // Within about 2000 km its 50 m shape shows in the infrared.
    let near = mark(&contact(cold.clone(), 1e6), &eyes, m).unwrap();
    assert_eq!(near.mark, "ink", "{:?}", near.why);
    assert!(near.resolved_by.contains(&"infrared telescope".to_string()));
    // The crew's own ring swells when the drive is lit.
    assert!(noticed_to(&lit, &eyes, m).unwrap() > 2.0 * noticed_to(&cold, &eyes, m).unwrap());
}

#[test]
fn a_lost_contact_leaves_a_growing_ring() {
    let m = Model::base();
    let eyes = default_fit(&base_fits().unwrap(), "today", 50.0).unwrap();
    let mut c = contact(hull(1e5, None), 1e11);
    c.lost_for_s = Some(3600.0);
    c.speed_m_s = 1e3;
    c.accel_m_s2 = 1.0;
    let k = mark(&c, &eyes, m).unwrap();
    assert_eq!(k.mark, "unseen");
    let r = k.ring_m.unwrap();
    assert!((r - (3.6e6 + 0.5 * 3600.0 * 3600.0)).abs() < 1e-6, "{r}");
    assert!(k.why.iter().any(|w| w.contains("Last seen")));
}
