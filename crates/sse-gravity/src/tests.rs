//! Checks for XO1 (exotic objects design section 11, rows 1 to 4, 7 and 8).

use crate::bend::{self, b_crit, b_of_row, row_of_b, BendTable, CAPTURED_ROWS, ROWS};
use crate::scenario::table;
use sse_core::math as m;

#[test]
fn rows_cover_both_sides_of_the_critical_ray() {
    assert_eq!(b_of_row(0), 0.0);
    assert!(b_of_row(CAPTURED_ROWS - 1) < b_crit());
    assert!(b_of_row(CAPTURED_ROWS) > b_crit());
    assert_eq!(b_of_row(ROWS - 1), bend::B_MAX);
    for i in 1..ROWS {
        assert!(b_of_row(i) > b_of_row(i - 1), "row {i}");
        let back = row_of_b(b_of_row(i)).unwrap();
        assert!((back - i as f64).abs() < 1e-9, "row {i}: {back}");
    }
    assert_eq!(row_of_b(bend::B_MAX * 1.01), None);
}

#[test]
fn rays_inside_root_27_are_swallowed_and_outside_escape() {
    let t = table();
    for (i, r) in t.rows.iter().enumerate() {
        assert_eq!(r.captured, r.b < b_crit(), "row {i}");
        if r.captured && r.end_phi < bend::PHI_MAX {
            assert_eq!(*r.u.last().unwrap(), bend::U_HORIZON, "row {i}");
        } else if r.captured {
            // still winding at the table's last angle
        } else {
            if r.end_phi < bend::PHI_MAX {
                assert_eq!(*r.u.last().unwrap(), 0.0, "row {i}");
            }
            assert!(r.end_phi > m::PI, "row {i}: light is always bent inwards");
            // the table's highest u is near its turning point
            let top = r.u.iter().cloned().fold(0.0, f64::max);
            assert!(top <= r.closest_u * (1.0 + 1e-9), "row {i}");
        }
    }
    // Rays near b_c wind round the photon orbit: the closest escaping row
    // sweeps more than one and a half turns, past the table's last angle.
    assert!(t.rows[CAPTURED_ROWS].end_phi > bend::PHI_MAX);
    assert!(t.rows[CAPTURED_ROWS - 1].end_phi > bend::PHI_MAX);
}

/// The total sweep of an escaping ray by quadrature, as an independent
/// check of the table's step-by-step trace:
/// Δφ = 2 ∫₀^{u₀} du / √(1/b² − u² + 2u³), with u = u₀(1 − t²).
fn sweep_by_quadrature(b: f64) -> f64 {
    let u0 = bend::turning_point(b);
    let c = 1.0 / (b * b);
    let g = |t: f64| {
        if t <= 0.0 {
            // limit at the turning point: 2 / √(2 − 6u₀)
            return 2.0 / m::sqrt(2.0 - 6.0 * u0);
        }
        let u = u0 * (1.0 - t * t);
        2.0 * u0 * t / m::sqrt(c - u * u + 2.0 * u * u * u)
    };
    let n = 20_000;
    let h = 1.0 / n as f64;
    let mut s = g(0.0) + g(1.0);
    for k in 1..n {
        s += g(k as f64 * h) * if k % 2 == 1 { 4.0 } else { 2.0 };
    }
    2.0 * s * h / 3.0
}

#[test]
fn escaping_sweeps_match_an_independent_quadrature() {
    let t = table();
    for i in [CAPTURED_ROWS + 8, CAPTURED_ROWS + 40, 200, ROWS - 1] {
        let r = &t.rows[i];
        let q = sweep_by_quadrature(r.b);
        assert!(
            (r.end_phi - q).abs() < 1e-6,
            "row {i} b {}: table {} quadrature {q}",
            r.b,
            r.end_phi
        );
    }
}

#[test]
fn far_rays_turn_as_the_weak_field_series_says() {
    // α = 4/b + 15π/4b² + 128/3b³ + 3465π/64b⁴ (Keeton and Petters 2005);
    // the first term alone is the weak bender.
    let r = &table().rows[ROWS - 1];
    let b = r.b;
    let series = 4.0 / b
        + 15.0 * m::PI / (4.0 * b * b)
        + 128.0 / (3.0 * b * b * b)
        + 3465.0 * m::PI / (64.0 * b * b * b * b);
    assert!(
        ((r.turn - series) / series).abs() < 2e-5,
        "{} vs {series}",
        r.turn
    );
    let weak = 4.0 / b;
    // the first term alone (the weak bender) is within 6% at the table's edge
    assert!(((r.turn - weak) / r.turn).abs() < 0.06);
}

#[test]
fn the_table_reads_back_its_own_rows() {
    let t = table();
    let r = &t.rows[200];
    for k in [0usize, 10, 100, 255] {
        let got = t.u_at(r.b, bend::phi_of_col(k)).unwrap();
        assert!((got - r.u[k]).abs() < 1e-12, "col {k}");
    }
    assert_eq!(t.u_at(r.b, bend::PHI_MAX * 1.1), None);
    assert_eq!(t.flat().len(), ROWS * (bend::COLS + 4));
}

#[test]
fn the_table_is_the_same_bits_every_time() {
    let a = BendTable::compute();
    assert_eq!(a.hash(), table().hash());
    crate::golden::check("gravity.json", &crate::scenario::golden());
}
