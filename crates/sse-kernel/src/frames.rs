//! `surface_frames`: a world's surface as time-lapse keyframes on the S2
//! plate grid, from its first plates to the epoch, with dated events
//! (`sse_surface::frames`). Read only; not cached (one call per film).

use crate::err;
use crate::protocol::Response;
use crate::survey::{resolve, From};

pub(crate) fn surface_frames(
    from: From,
    options: Option<&serde_json::Value>,
    detail: Option<&str>,
    frames_max: Option<u32>,
) -> Result<serde_json::Value, Response> {
    let survey = match detail.unwrap_or("orbit") {
        "orbit" => false,
        "survey" => true,
        other => {
            return Err(err(
                "bad_request",
                format!("detail {other}: orbit or survey"),
            ))
        }
    };
    let w = resolve(from, options, false)?;
    let frames = sse_surface::frames::surface_frames(
        &w.settings,
        &w.body,
        &w.seed,
        w.epoch_gyr,
        &w.opts,
        survey,
        frames_max.map(|x| x as usize),
        w.track_before.as_ref(),
    )
    .map_err(|e| err("no_ground", e))?;
    let mut o = serde_json::json!({
        "body": w.body.address.to_string(),
        "label": w.body.label,
        "source": w.source,
        "frames": frames,
    });
    if let Some(t) = w.at_s {
        o["at_s"] = t.into();
    }
    Ok(o)
}
