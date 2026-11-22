//! HEALPix, the equal-area sphere grid, in the RING scheme. Written fresh
//! from the published description (Górski, K. M. et al. 2005, HEALPix: a
//! framework for high-resolution discretization and fast analysis of data
//! distributed on the sphere, ApJ 622, 759, section 4,
//! doi:10.1086/427976), not from the reference library, which is GPL.
//!
//! The sphere is cut into 12 base pixels, each split into `nside²` pixels of
//! equal area 4π / (12 nside²). Pixels lie on 4 nside − 1 rings of constant
//! latitude, numbered from the north pole; in the polar caps (|z| > 2/3) ring
//! i holds 4i pixels, in the equatorial belt every ring holds 4 nside.
//!
//! Angles: θ is colatitude (0 at the north pole), φ longitude in [0, 2π).
//! Everything is integer arithmetic plus `sqrt`, `floor` and the pinned
//! `acos`/`atan2`, so every device gives the same pixel.

use sse_core::math as m;

/// A HEALPix grid of resolution `nside` (a power of two, 1 to 8192).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grid {
    pub nside: u64,
}

impl Grid {
    pub fn new(nside: u64) -> Result<Self, String> {
        if nside == 0 || nside > 8192 || !nside.is_power_of_two() {
            return Err(format!(
                "nside {nside} must be a power of two from 1 to 8192"
            ));
        }
        Ok(Grid { nside })
    }

    pub fn npix(&self) -> u64 {
        12 * self.nside * self.nside
    }

    /// Pixels in the north polar cap: 2 nside (nside − 1).
    fn ncap(&self) -> u64 {
        2 * self.nside * (self.nside - 1)
    }

    /// Solid angle of every pixel, sr.
    pub fn pixel_area(&self) -> f64 {
        4.0 * m::PI / self.npix() as f64
    }

    /// Centre of pixel `p` as (z = cos θ, φ).
    pub fn centre_z_phi(&self, p: u64) -> (f64, f64) {
        let ns = self.nside;
        let nsf = ns as f64;
        let ncap = self.ncap();
        let npix = self.npix();
        if p < ncap {
            // North cap: ring i (1-based) starts at 2 i (i − 1).
            let i = isqrt(1 + 2 * p).div_ceil(2);
            let j = p - 2 * i * (i - 1) + 1;
            let fi = i as f64;
            let z = 1.0 - fi * fi / (3.0 * nsf * nsf);
            let phi = m::PI / (2.0 * fi) * (j as f64 - 0.5);
            (z, phi)
        } else if p < npix - ncap {
            let q = p - ncap;
            let i = q / (4 * ns) + ns;
            let j = q % (4 * ns) + 1;
            let s = ((i - ns + 1) % 2) as f64;
            let z = 4.0 / 3.0 - 2.0 * i as f64 / (3.0 * nsf);
            // Rings alternate between centres on and between the cap's meridians.
            let phi = m::PI / (2.0 * nsf) * (j as f64 - 1.0 + 0.5 * s);
            (z, phi)
        } else {
            let q = npix - p;
            let i = isqrt(2 * q - 1).div_ceil(2);
            let j = 4 * i + 1 - (q - 2 * i * (i - 1));
            let fi = i as f64;
            let z = -1.0 + fi * fi / (3.0 * nsf * nsf);
            let phi = m::PI / (2.0 * fi) * (j as f64 - 0.5);
            (z, phi)
        }
    }

    /// Centre of pixel `p` as (θ, φ).
    pub fn centre(&self, p: u64) -> (f64, f64) {
        let (z, phi) = self.centre_z_phi(p);
        (m::acos(z.clamp(-1.0, 1.0)), phi)
    }

    /// Centre of pixel `p` as a unit vector (x, y, z).
    pub fn centre_vec(&self, p: u64) -> [f64; 3] {
        let (z, phi) = self.centre_z_phi(p);
        let s = m::sqrt((1.0 - z * z).max(0.0));
        [s * m::cos(phi), s * m::sin(phi), z]
    }

    /// The pixel holding direction (z = cos θ, φ).
    pub fn pixel_z_phi(&self, z: f64, phi: f64) -> u64 {
        let ns = self.nside as i64;
        let nsf = self.nside as f64;
        let za = z.abs();
        let mut tt = phi / (m::PI / 2.0);
        tt -= 4.0 * m::floor(tt / 4.0);
        if tt >= 4.0 {
            tt = 0.0;
        }
        if za <= 2.0 / 3.0 {
            // Equatorial belt: the two edge lines through the point.
            let t1 = nsf * (0.5 + tt);
            let t2 = nsf * z * 0.75;
            let jp = m::floor(t1 - t2) as i64;
            let jm = m::floor(t1 + t2) as i64;
            let ir = ns + 1 + jp - jm; // 1 .. 2 nside + 1
            let kshift = 1 - (ir & 1);
            let ip = (jp + jm - ns + kshift + 1).div_euclid(2).rem_euclid(4 * ns);
            self.ncap() + ((ir - 1) * 4 * ns + ip) as u64
        } else {
            let tp = tt - m::floor(tt);
            let tmp = nsf * m::sqrt(3.0 * (1.0 - za));
            let jp = m::floor(tp * tmp) as i64;
            let jm = m::floor((1.0 - tp) * tmp) as i64;
            let ir = (jp + jm + 1).max(1); // ring from the nearer pole
            let ip = (m::floor(tt * ir as f64) as i64).rem_euclid(4 * ir);
            if z > 0.0 {
                (2 * ir * (ir - 1) + ip) as u64
            } else {
                self.npix() - (2 * ir * (ir + 1)) as u64 + ip as u64
            }
        }
    }

    /// The pixel holding direction (θ, φ).
    pub fn pixel(&self, theta: f64, phi: f64) -> u64 {
        self.pixel_z_phi(m::cos(theta), phi)
    }

    /// The pixel holding a direction vector (need not be unit length).
    pub fn pixel_vec(&self, v: [f64; 3]) -> u64 {
        let r = m::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
        if r <= 0.0 {
            return 0;
        }
        let z = v[2] / r;
        let mut phi = m::atan2(v[1], v[0]);
        if phi < 0.0 {
            phi += 2.0 * m::PI;
        }
        self.pixel_z_phi(z, phi)
    }
}

/// Integer square root (floor), exact for all u64.
fn isqrt(n: u64) -> u64 {
    if n < 2 {
        return n;
    }
    let mut x = m::sqrt(n as f64) as u64;
    while x * x > n {
        x -= 1;
    }
    while (x + 1) * (x + 1) <= n {
        x += 1;
    }
    x
}
