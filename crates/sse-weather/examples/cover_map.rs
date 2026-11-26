//! A day's cloud as a picture (W-synoptic): an equirectangular map, one pixel a degree, of the
//! light the clouds send back. `cargo run -p sse-weather --example cover_map -- Earth 200.5 out.pgm`
use sse_core::math as m;
use sse_weather::scenario::{block, fixture};
use sse_weather::Date;
use std::io::Write;
fn main() {
    let label = std::env::args().nth(1).unwrap_or("Earth-like".into());
    let day: f64 = std::env::args()
        .nth(2)
        .map(|s| s.parse().unwrap())
        .unwrap_or(200.5);
    let b = block();
    let c = fixture(&label);
    let date = Date { day };
    let p = b.prepare(&c, date);
    let sys = b.systems_at(&c, &p, date, &[]);
    for s in &sys {
        eprintln!(
            "{} {} lat {:.0} lon {:.0} {:.0} km",
            s.id, s.kind, s.lat_deg, s.lon_deg, s.size_km
        );
    }
    let (w, h) = (360usize, 180usize);
    let mut out = Vec::new();
    writeln!(out, "P5 {w} {h} 255").unwrap();
    for j in 0..h {
        for i in 0..w {
            let lat = 89.5 - j as f64;
            let lon = -179.5 + i as f64;
            let cell = b.cell_at(&c, &p, date, &sys, None, lat, lon);
            // Adding method, top down, non-absorbing slabs: R = (1-g)tau/(2+(1-g)tau), g 0.85, ground 0.06.
            let mut r = 0.06f64;
            for l in cell.layers.iter() {
                let rs = 0.15 * l.optical_depth / (2.0 + 0.15 * l.optical_depth) * l.cloud_cover;
                let t = 1.0 - rs;
                r = rs + t * t * r / (1.0 - rs * r);
            }
            out.push((255.0 * m::pow(r, 1.0 / 2.2).min(1.0)) as u8);
        }
    }
    std::fs::write(std::env::args().nth(3).unwrap_or("cover.pgm".into()), out).unwrap();
}
