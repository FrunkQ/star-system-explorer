//! `survey_caves`: S14a's Site-level cave survey. A world is surveyed
//! (through the same cache as `survey_surface`), and the lava-tube passages
//! within `radius_km` of a site are placed from its tube fields' envelopes.

use crate::err;
use crate::protocol::Response;
use crate::survey::{surveyed, Cache, From};
use sse_core::draw::{Draws, Stream};

/// Default window around the site, km.
pub const DEFAULT_RADIUS_KM: f64 = 5.0;

pub(crate) struct Ask<'a> {
    pub from: From<'a>,
    pub options: Option<&'a serde_json::Value>,
    pub quick: bool,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub radius_km: Option<f64>,
}

pub(crate) fn survey_caves(a: Ask, cache: &mut Cache) -> Result<serde_json::Value, Response> {
    if !(-90.0..=90.0).contains(&a.lat_deg) || !a.lon_deg.is_finite() {
        return Err(err(
            "bad_request",
            "site: lat_deg in -90..90 and a finite lon_deg",
        ));
    }
    let radius = a.radius_km.unwrap_or(DEFAULT_RADIUS_KM);
    if !(radius.is_finite() && radius > 0.0) {
        return Err(err("bad_request", "radius_km must be above 0"));
    }
    let (source, body, at_s, sv) = surveyed(a.from, a.options, a.quick, cache)?;
    if sv.atlas.layers().is_none() {
        return Err(err(
            "no_ground",
            format!("{} has no ground to hold caves", body.label),
        ));
    }
    let d = Draws::new(
        sv.seed,
        sv.envelope.address.clone(),
        Stream::new("surface.caves.v1").expect("valid stream"),
    );
    let caves = sse_surface::caves::passages(
        &sse_surface::scenario::settings().caves,
        &sv.atlas,
        &sv.tubes,
        &d,
        a.lat_deg,
        a.lon_deg,
        radius,
    );
    let mut o = serde_json::json!({
        "body": body.address.to_string(),
        "label": body.label,
        "source": source,
        "caves": caves,
    });
    if let Some(t) = at_s {
        o["at_s"] = t.into();
    }
    Ok(o)
}
