//! K17: elements typed in another convention, converted on the way in.

use sse_core::math as m;

fn send(k: &mut crate::Kernel, req: serde_json::Value) -> serde_json::Value {
    serde_json::from_str(&k.handle_json(&req.to_string())).unwrap()
}

fn deg(x: f64) -> f64 {
    x * m::PI / 180.0
}

/// An orbit in Earth's equator plane (i = 0 there) is tilted by the
/// obliquity to the ecliptic, with its node at the equinox's far side; an
/// ecliptic orbit typed in equatorial terms comes back flat.
#[test]
fn earths_equator_to_the_ecliptic() {
    let eps = deg(23.439_279_4); // only the test's own number: any tilt works
    let f = crate::conventions::plane(deg(90.0), deg(90.0) - eps, 0.0).unwrap();
    let (i, node, _) = crate::conventions::to_system(f, 0.0, 0.0, 0.0);
    assert!((i - eps).abs() < 1e-12, "{i}");
    // The ecliptic seen from the equator: i = eps, its ascending node at
    // the equinox (RA 0); brought home it is flat.
    let (i2, _, _) = crate::conventions::to_system(f, eps, 0.0, 0.0);
    assert!(i2.abs() < 1e-9 || (i2 - deg(180.0)).abs() < 1e-9, "{i2}");
    let _ = node;
    // Measured from the system's own plane, nothing changes.
    let own = crate::conventions::plane(0.0, deg(90.0), 0.0).unwrap();
    let (a, b, c) = crate::conventions::to_system(own, 0.3, 1.2, 2.1);
    assert!(
        (a - 0.3).abs() < 1e-12 && (b - 1.2).abs() < 1e-12 && (c - 2.1).abs() < 1e-12,
        "{a} {b} {c}"
    );
}

#[test]
fn the_op_converts_centre_and_date_and_gives_the_edits() {
    let mut k = crate::Kernel::new();
    send(
        &mut k,
        serde_json::json!({"op":"open_universe","id":"cv","seed":"cv"}),
    );
    send(
        &mut k,
        serde_json::json!({"op":"set_authored_batch","occurred_at_s":0,"edits":[
            {"address":"u:cv/sys:0/body:0","field":"body.mass","value":1.303e22},
            {"address":"u:cv/sys:0/body:0/body:0","field":"body.mass","value":1.586e21},
            {"address":"u:cv/sys:0/body:0/body:0","field":"orbit.host","value":"u:cv/sys:0/body:0"}
        ]}),
    );
    // A moon's ellipse about the pair's shared centre is the relative
    // one shrunk by M / (M + m): the relative orbit is (M + m) / M larger.
    let r = send(
        &mut k,
        serde_json::json!({"op":"elements_in","address":"u:cv/sys:0/body:0/body:0",
            "semi_major_axis_m":1.7536e7,"eccentricity":0.0,"inclination_rad":0.0,
            "ascending_node_rad":0.0,"argument_of_periapsis_rad":0.0,
            "centred_on":"shared","true_on_s":86400}),
    );
    assert_eq!(r["type"], "elements_converted", "{r}");
    let a = r["elements"]["semi_major_axis_m"].as_f64().unwrap();
    let want = 1.7536e7 * (1.303e22 + 1.586e21) / 1.303e22;
    assert!((a / want - 1.0).abs() < 1e-12, "{a}");
    let edits = r["elements"]["edits"].as_array().unwrap();
    assert!(edits
        .iter()
        .any(|e| e["field"] == "orbit.epoch_s" && e["value"] == 86400));
    let bad = send(
        &mut k,
        serde_json::json!({"op":"elements_in","address":"u:cv/sys:0/body:0/body:0",
            "semi_major_axis_m":1.0,"eccentricity":0.0,"inclination_rad":0.0,
            "ascending_node_rad":0.0,"argument_of_periapsis_rad":0.0,"centred_on":"middle"}),
    );
    assert_eq!(bad["code"], "bad_request", "{bad}");
}
