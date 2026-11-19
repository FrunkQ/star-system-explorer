//! Prints each fixture world's regime and climate: `cargo run -p sse-weather --example regimes`.
use sse_weather::circulation::circulation;
use sse_weather::scenario::{block, fixtures, settings};
use sse_weather::WeatherBlock;

fn main() {
    let s = settings();
    let b = block();
    for f in fixtures() {
        let c = &f.conditions;
        let ci = circulation(&s, c);
        let cl = b.climate(c);
        println!(
            "{:24} {:15} cells {} hadley {:4.0}° storms {:?} cloud {:.2} rain {:.3} m/yr {:16} sun@ground {:.2}",
            c.label, ci.regime, ci.cells, ci.hadley_edge_deg, ci.storm_tracks_deg.iter().map(|x| x.round()).collect::<Vec<_>>(),
            cl.cloud_cover_mean, cl.precip_mean_m_yr, cl.condensate, cl.surface_sun_fraction
        );
    }
}
