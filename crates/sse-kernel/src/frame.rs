//! A system's frame in the galaxy (Y5's ask): how its own orbit-plane
//! longitude and latitude, the frame the Window reports bodies in, sit in
//! galactic l and b, the frame the stars and the glow come in.
//!
//! Sol's is measured: the J2000 ecliptic pole and equinox, the frame the
//! Sol fixture's elements are given in, read from `packs/base-sol`. Any
//! other system's is drawn, keyed by the universe seed and the system's
//! address: the pole is isotropic (planetary systems show no strong
//! alignment with the galactic plane) and the longitude zero is uniform
//! about it. Fudge F38.

use crate::{err, Response};
use sse_core::math as m;
use sse_core::{Address, Draws, Seed, Stream};

const STREAM: &str = "system.frame.v1";

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Frame {
    /// The system's north pole (+z of its frame) in galactic l, b.
    pub pole: (f64, f64),
    /// Its longitude zero (+x) in galactic l, b.
    pub zero: (f64, f64),
    pub evidence: &'static str,
    pub source: String,
}

fn unit_lb(l_deg: f64, b_deg: f64) -> [f64; 3] {
    let (l, b) = (l_deg.to_radians(), b_deg.to_radians());
    [m::cos(b) * m::cos(l), m::cos(b) * m::sin(l), m::sin(b)]
}

fn lb_of(v: [f64; 3]) -> (f64, f64) {
    let mut l = m::atan2(v[1], v[0]).to_degrees();
    if l < 0.0 {
        l += 360.0;
    }
    (l, m::asin(v[2].clamp(-1.0, 1.0)).to_degrees())
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn norm(a: [f64; 3]) -> [f64; 3] {
    let r = m::sqrt(a[0] * a[0] + a[1] * a[1] + a[2] * a[2]);
    [a[0] / r, a[1] / r, a[2] / r]
}

impl Frame {
    /// The system's axes in galactic Cartesian (x toward l 0, b 0; z toward
    /// the north galactic pole): x the longitude zero made exactly
    /// perpendicular to the pole, z the pole, y completing a right-handed set.
    pub fn axes(&self) -> [[f64; 3]; 3] {
        let z = unit_lb(self.pole.0, self.pole.1);
        let x0 = unit_lb(self.zero.0, self.zero.1);
        let d = x0[0] * z[0] + x0[1] * z[1] + x0[2] * z[2];
        let x = norm([x0[0] - d * z[0], x0[1] - d * z[1], x0[2] - d * z[2]]);
        [x, cross(z, x), z]
    }

    /// Rows of the matrix taking a vector in the system's frame to galactic
    /// Cartesian: g = M s.
    pub fn to_galactic(&self) -> [[f64; 3]; 3] {
        let [x, y, z] = self.axes();
        [[x[0], y[0], z[0]], [x[1], y[1], z[1]], [x[2], y[2], z[2]]]
    }

    /// The galactic l, b of a direction given as the system's own longitude
    /// and latitude.
    #[cfg(test)]
    pub fn galactic_of(&self, lon_deg: f64, lat_deg: f64) -> (f64, f64) {
        let s = unit_lb(lon_deg, lat_deg);
        let mm = self.to_galactic();
        let g = [0, 1, 2].map(|i| mm[i][0] * s[0] + mm[i][1] * s[1] + mm[i][2] * s[2]);
        lb_of(g)
    }

    pub fn json(&self) -> serde_json::Value {
        let [x, _, z] = self.axes();
        // Where the system's plane rises through the galactic plane.
        let node = cross([0.0, 0.0, 1.0], z);
        let node_l = if node[0] * node[0] + node[1] * node[1] > 1e-24 {
            serde_json::json!(lb_of(norm(node)).0)
        } else {
            serde_json::Value::Null
        };
        let zero = lb_of(x);
        serde_json::json!({
            "pole_l_deg": self.pole.0, "pole_b_deg": self.pole.1,
            "zero_l_deg": zero.0, "zero_b_deg": zero.1,
            "node_l_deg": node_l,
            "to_galactic": self.to_galactic(),
            "evidence": self.evidence,
            "source": self.source,
            "model": "to_galactic takes a unit vector in the system's frame (x the longitude zero, z its north pole) to galactic Cartesian (x toward l 0, b 0; z the north galactic pole); node_l_deg is where the system's plane rises through the galactic plane",
        })
    }
}

/// Sol: the J2000 ecliptic pole and equinox from `packs/base-sol`.
pub(crate) fn sol() -> Frame {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../../../packs/base-sol/pack.json"))
            .expect("base-sol pack");
    let sun = &pack["provides"]["other"]["sol"]["bodies"][0]["values"];
    let v = |k: &str| {
        sun[format!("system.frame.{k}")]["v"]
            .as_f64()
            .expect("base-sol carries the Sun's frame")
    };
    Frame {
        pole: (v("pole_l_deg"), v("pole_b_deg")),
        zero: (v("zero_l_deg"), v("zero_b_deg")),
        evidence: "measured",
        source: "J2000 ecliptic pole and equinox (base-sol: Hipparcos A_G, IAU 2006 obliquity)"
            .into(),
    }
}

/// Any other system: an isotropic pole and a uniform longitude zero, keyed.
pub(crate) fn drawn(seed: &Seed, system: &Address) -> Result<Frame, Response> {
    let d = Draws::new(
        *seed,
        system.clone(),
        Stream::new(STREAM).map_err(|e| err("bad_request", e.to_string()))?,
    );
    let sin_b = 2.0 * d.unit(0) - 1.0;
    let pole = (360.0 * d.unit(1), m::asin(sin_b).to_degrees());
    let z = unit_lb(pole.0, pole.1);
    // The ascending node on the galactic plane, then turned about the pole.
    let n = cross([0.0, 0.0, 1.0], z);
    let x0 = if n[0] * n[0] + n[1] * n[1] > 1e-24 {
        norm(n)
    } else {
        [1.0, 0.0, 0.0]
    };
    let y0 = cross(z, x0);
    let psi = 2.0 * core::f64::consts::PI * d.unit(2);
    let (c, s) = (m::cos(psi), m::sin(psi));
    let x = [0, 1, 2].map(|i| c * x0[i] + s * y0[i]);
    Ok(Frame {
        pole,
        zero: lb_of(x),
        evidence: "drawn",
        source: format!("isotropic, keyed ({STREAM})"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sol_s_frame_puts_the_ecliptic_where_it_is_measured() {
        let f = sol();
        // The ecliptic north pole is at l 96.38, b 29.81 (J2000).
        assert!((f.pole.0 - 96.38).abs() < 0.01 && (f.pole.1 - 29.81).abs() < 0.01);
        // Ecliptic longitude 90 at latitude 0 is the summer solstice point,
        // RA 90 Dec 23.44, which is galactic l 186.37, b -0.02: the ecliptic
        // crosses the Milky Way there (and opposite, by the galactic centre).
        let (l, b) = f.galactic_of(90.0, 0.0);
        assert!(
            (l - 186.37).abs() < 0.01 && (b + 0.02).abs() < 0.01,
            "{l} {b}"
        );
        // The matrix is a rotation.
        let mm = f.to_galactic();
        for i in 0..3 {
            for j in 0..3 {
                let d: f64 = (0..3).map(|k| mm[k][i] * mm[k][j]).sum();
                assert!((d - if i == j { 1.0 } else { 0.0 }).abs() < 1e-12);
            }
        }
    }

    #[test]
    fn drawn_frames_are_keyed_and_isotropic() {
        let seed = Seed::from_text("frame-test");
        let mut north = 0usize;
        let mut sum_sin_b = 0.0;
        let n = 2000;
        for i in 0..n {
            let a: Address = format!("u:t/cell:0_0_0/sys:f{i}").parse().unwrap();
            let f = drawn(&seed, &a).unwrap();
            assert_eq!(f, drawn(&seed, &a).unwrap());
            let sb = m::sin(f.pole.1.to_radians());
            sum_sin_b += sb;
            if sb > 0.0 {
                north += 1;
            }
            // The zero lies in the system's plane.
            let z = unit_lb(f.pole.0, f.pole.1);
            let x = unit_lb(f.zero.0, f.zero.1);
            assert!((x[0] * z[0] + x[1] * z[1] + x[2] * z[2]).abs() < 1e-9);
        }
        // Isotropic: sin b uniform on [-1, 1], mean 0 with sd 1/sqrt(3n).
        assert!((sum_sin_b / n as f64).abs() < 4.0 / m::sqrt(3.0 * n as f64));
        assert!((north as f64 / n as f64 - 0.5).abs() < 0.05);
    }
}
