//! Storm statistics for the fixture worlds: `cargo run -p sse-weather --example storms`.
use sse_weather::scenario::{block, fixtures};
use sse_weather::{Date, WeatherBlock};
use std::collections::BTreeMap;

fn main() {
    let b = block();
    for f in fixtures() {
        let c = &f.conditions;
        let mut kinds: BTreeMap<String, (usize, f64, f64)> = BTreeMap::new();
        let mut seen = std::collections::BTreeSet::new();
        let days = (c.year_days.min(400.0)) as i64;
        for d in 0..days {
            for s in b.systems(
                c,
                Date {
                    day: d as f64 + 0.5,
                },
                &[],
            ) {
                if seen.insert(s.id.clone()) {
                    let e = kinds.entry(s.kind.clone()).or_insert((0, 0.0, 0.0));
                    e.0 += 1;
                    e.1 += s.lat_deg.abs();
                    e.2 += s.size_km;
                }
            }
        }
        let summary: Vec<String> = kinds
            .iter()
            .map(|(k, (n, lat, size))| {
                format!(
                    "{k}: {n} (mean |lat| {:.0}°, {:.0} km)",
                    lat / *n as f64,
                    size / *n as f64
                )
            })
            .collect();
        println!("{:24} over {days} days: {}", c.label, summary.join("; "));
        let r = b.report(c, Date { day: 200.0 }, 45.0, 0.0, &[], None);
        println!("    report 45N 0E day 200: {}", r.lines.join(". "));
    }
}
