//! RW3: `read_catalogue`, the import preview over `sse-catalogue`. Reading
//! a file is not physics, so it is code; everything estimated from what a
//! catalogue holds is a block flow (RW3b), not here.

use crate::err;
use crate::protocol::Response;
use sse_catalogue::{b64, preview, read_bytes, Cut, Refusal};

/// Rows a preview shows by default, and at most.
const LIMIT: u32 = 20;
const MAX_LIMIT: u32 = 200;

fn refused(r: Refusal) -> Response {
    err(&format!("catalogue_{}", r.code), r.reason)
}

pub(crate) fn read(
    text: Option<&str>,
    packed_b64: Option<&str>,
    min_size: Option<f64>,
    rank: Option<u64>,
    limit: Option<u32>,
) -> Result<Response, Response> {
    let cut = match (min_size, rank) {
        (Some(_), Some(_)) => return Err(err("bad_request", "give min_size or rank, not both")),
        (Some(s), None) if !s.is_finite() => {
            return Err(err("bad_request", "min_size is not finite"))
        }
        (Some(s), None) => Cut::Size(s),
        (None, Some(k)) => Cut::Rank(k),
        (None, None) => Cut::All,
    };
    let bytes = match (text, packed_b64) {
        (Some(t), None) => t.as_bytes().to_vec(),
        (None, Some(b)) => b64::decode(b).map_err(|e| err("bad_request", e))?,
        _ => return Err(err("bad_request", "give text or packed_b64, one of them")),
    };
    let (h, r, p) = read_bytes(&bytes, cut).map_err(refused)?;
    let limit = limit.unwrap_or(LIMIT).min(MAX_LIMIT) as usize;
    Ok(Response::CatalogueRead {
        preview: preview(&h, &r, p.as_ref(), limit),
    })
}
