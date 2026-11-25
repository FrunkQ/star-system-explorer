//! K17 (editing design 10B, F4): orbits typed in another convention,
//! converted on the way in. Published elements are measured from one plane
//! (the system's own, a planet's equator, Earth's equator), centred on the
//! parent or on the pair's shared centre, and true on one date. The kernel
//! keeps every orbit in the system's own plane, about its host, with its own
//! epoch; this converts, and gives the edits to send.
//!
//! A plane is named by its north pole in the system's frame (longitude and
//! latitude) and the system-frame longitude its angles count from (the
//! direction on the system plane whose projection onto the plane is its
//! zero). Earth's equator in an ecliptic frame is the pole at longitude 90°,
//! latitude 90° - obliquity, counting from longitude 0 (the equinox).

use crate::err;
use crate::protocol::Response;
use sse_core::math as m;

type V = [f64; 3];

fn dot(a: V, b: V) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V, b: V) -> V {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn norm(a: V) -> Option<V> {
    let l = m::sqrt(dot(a, a));
    (l > 1e-15).then(|| [a[0] / l, a[1] / l, a[2] / l])
}
fn add3(a: V, b: V, c: V, x: f64, y: f64, z: f64) -> V {
    [
        a[0] * x + b[0] * y + c[0] * z,
        a[1] * x + b[1] * y + c[1] * z,
        a[2] * x + b[2] * y + c[2] * z,
    ]
}

/// Orbit normal and periapsis direction of (i, node, peri) in a frame.
fn axes(i: f64, node: f64, peri: f64) -> (V, V) {
    let (si, ci) = (m::sin(i), m::cos(i));
    let (sn, cn) = (m::sin(node), m::cos(node));
    let (sw, cw) = (m::sin(peri), m::cos(peri));
    let h = [si * sn, -si * cn, ci];
    let p = [cn * cw - sn * sw * ci, sn * cw + cn * sw * ci, sw * si];
    (h, p)
}

/// (i, node, peri) of an orbit from its normal and periapsis direction.
fn angles(h: V, p: V) -> (f64, f64, f64) {
    let i = m::acos(h[2].clamp(-1.0, 1.0));
    let n = [-h[1], h[0], 0.0];
    let tau = 2.0 * m::PI;
    match norm(n) {
        Some(n) => {
            let node = m::fmod(m::atan2(n[1], n[0]) + tau, tau);
            let peri = m::atan2(dot(cross(n, p), h), dot(n, p));
            (i, node, m::fmod(peri + tau, tau))
        }
        // In the plane: the node is undefined; count the periapsis from x.
        None => (
            i,
            0.0,
            m::fmod(m::atan2(p[1], p[0]) * h[2].signum() + tau, tau),
        ),
    }
}

/// A plane's frame in the system's: its x (zero direction), y and pole.
pub(crate) fn plane(pole_lon: f64, pole_lat: f64, zero_lon: f64) -> Result<(V, V, V), Response> {
    let z = [
        m::cos(pole_lat) * m::cos(pole_lon),
        m::cos(pole_lat) * m::sin(pole_lon),
        m::sin(pole_lat),
    ];
    let d = [m::cos(zero_lon), m::sin(zero_lon), 0.0];
    let k = dot(d, z);
    let x = norm([d[0] - k * z[0], d[1] - k * z[1], d[2] - k * z[2]]).ok_or_else(|| {
        err(
            "bad_request",
            "the zero direction lies along the plane's pole: give another zero_lon",
        )
    })?;
    Ok((x, cross(z, x), z))
}

/// (i, node, peri) measured from a plane, in the system's frame.
pub(crate) fn to_system(frame: (V, V, V), i: f64, node: f64, peri: f64) -> (f64, f64, f64) {
    let (x, y, z) = frame;
    let (h, p) = axes(i, node, peri);
    let h = add3(x, y, z, h[0], h[1], h[2]);
    let p = add3(x, y, z, p[0], p[1], p[2]);
    angles(h, p)
}

/// Request body of `elements_in`, after parsing.
pub(crate) struct Typed {
    pub a_m: f64,
    pub e: f64,
    pub i: f64,
    pub node: f64,
    pub peri: f64,
    pub mean_anomaly: Option<f64>,
}

/// The converted elements and the edits that set them on `address`.
pub(crate) fn convert(
    address: &str,
    t: Typed,
    frame: Option<(V, V, V)>,
    shared: Option<(f64, f64)>,
    true_on_s: Option<i64>,
) -> serde_json::Value {
    let (i, node, peri) = match frame {
        Some(f) => to_system(f, t.i, t.node, t.peri),
        None => (t.i, t.node, t.peri),
    };
    // About the shared centre, a body's ellipse has the same shape and
    // angles as the relative one, smaller by M / (M + m).
    let a = match shared {
        Some((host, own)) => t.a_m * (host + own) / host,
        None => t.a_m,
    };
    let mut edits = vec![
        serde_json::json!({"address": address, "field": "orbit.semi_major_axis", "value": a}),
        serde_json::json!({"address": address, "field": "orbit.eccentricity", "value": t.e}),
        serde_json::json!({"address": address, "field": "orbit.inclination", "value": i}),
        serde_json::json!({"address": address, "field": "orbit.ascending_node", "value": node}),
        serde_json::json!({"address": address, "field": "orbit.argument_of_periapsis", "value": peri}),
    ];
    if let Some(ma) = t.mean_anomaly {
        edits.push(serde_json::json!({"address": address, "field": "orbit.mean_anomaly_at_epoch", "value": ma}));
    }
    if let Some(ep) = true_on_s {
        edits.push(serde_json::json!({"address": address, "field": "orbit.epoch_s", "value": ep}));
    }
    serde_json::json!({
        "semi_major_axis_m": a,
        "eccentricity": t.e,
        "inclination_rad": i,
        "ascending_node_rad": node,
        "argument_of_periapsis_rad": peri,
        "mean_anomaly_at_epoch_rad": t.mean_anomaly,
        "epoch_s": true_on_s,
        "edits": edits,
    })
}
