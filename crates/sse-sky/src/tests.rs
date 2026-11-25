use crate::healpix::*;
use sse_core::math as m;

#[test]
fn bad_nside_is_refused() {
    assert!(Grid::new(0).is_err());
    assert!(Grid::new(3).is_err());
    assert!(Grid::new(16384).is_err());
    assert_eq!(Grid::new(32).unwrap().npix(), 12_288);
}

/// Known centres (Górski et al. 2005, the base-pixel layout; figures 4 and 5 in
/// the preprint, the ApJ figure number unchecked): at nside 1 the
/// first pixel sits at z = 2/3, φ = π/4; the last at z = −2/3, φ = 7π/4;
/// pixel 4 is on the equator at φ = 0.
#[test]
fn known_centres() {
    let g = Grid::new(1).unwrap();
    let (z, phi) = g.centre_z_phi(0);
    assert!((z - 2.0 / 3.0).abs() < 1e-15 && (phi - m::PI / 4.0).abs() < 1e-15);
    let (z, phi) = g.centre_z_phi(11);
    assert!((z + 2.0 / 3.0).abs() < 1e-15 && (phi - 7.0 * m::PI / 4.0).abs() < 1e-15);
    let (z, phi) = g.centre_z_phi(4);
    assert!(z.abs() < 1e-15 && phi.abs() < 1e-15);
}

/// Every pixel's centre lies in that pixel, at several resolutions.
#[test]
fn centres_round_trip() {
    for nside in [1u64, 2, 4, 8, 32, 64] {
        let g = Grid::new(nside).unwrap();
        for p in 0..g.npix() {
            let (z, phi) = g.centre_z_phi(p);
            assert_eq!(g.pixel_z_phi(z, phi), p, "nside {nside} pixel {p}");
            assert_eq!(
                g.pixel_vec(g.centre_vec(p)),
                p,
                "nside {nside} pixel {p} by vector"
            );
        }
    }
}

/// Rings: 4 nside − 1 of them, 4i pixels on cap ring i, 4 nside in the belt,
/// each ring at one latitude.
#[test]
fn rings_have_the_published_counts() {
    let g = Grid::new(16).unwrap();
    let mut rings: Vec<(f64, u64)> = vec![];
    for p in 0..g.npix() {
        let (z, _) = g.centre_z_phi(p);
        match rings.last_mut() {
            Some((rz, n)) if (*rz - z).abs() < 1e-12 => *n += 1,
            _ => rings.push((z, 1)),
        }
    }
    assert_eq!(rings.len() as u64, 4 * 16 - 1);
    for (i, (_, n)) in rings.iter().enumerate() {
        let ring = i as u64 + 1;
        let want = if ring < 16 {
            4 * ring
        } else if ring > 3 * 16 {
            4 * (4 * 16 - ring)
        } else {
            4 * 16
        };
        assert_eq!(*n, want, "ring {ring}");
    }
}

/// Equal area: evenly spread directions (a Fibonacci lattice) fill every
/// pixel about equally; and edge cases (poles, φ = 2π) land somewhere valid.
#[test]
fn pixels_have_equal_area() {
    let g = Grid::new(8).unwrap();
    let n = 768_000u64;
    let mut count = vec![0u64; g.npix() as usize];
    let golden = m::PI * (3.0 - m::sqrt(5.0));
    for k in 0..n {
        let z = 1.0 - (2.0 * k as f64 + 1.0) / n as f64;
        let phi = (golden * k as f64) % (2.0 * m::PI);
        count[g.pixel_z_phi(z, phi) as usize] += 1;
    }
    let mean = n as f64 / g.npix() as f64;
    for (p, c) in count.iter().enumerate() {
        assert!(
            (*c as f64 - mean).abs() < 0.03 * mean,
            "pixel {p}: {c} against {mean}"
        );
    }
    for (z, phi) in [
        (1.0, 0.0),
        (-1.0, 3.0),
        (0.0, 2.0 * m::PI),
        (2.0 / 3.0, 2.0 * m::PI - 1e-9),
    ] {
        assert!(g.pixel_z_phi(z, phi) < g.npix());
    }
}
