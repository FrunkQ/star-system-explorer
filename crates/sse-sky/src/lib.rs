//! SSE4 night sky, light side (track Y, stage Y3). The equal-area sphere
//! grid the glow map is kept on; the glow itself follows once the sky draw
//! (Y2, `sky_from_here`) gives the drawn classes to subtract.
#![forbid(unsafe_code)]

pub mod glow;
pub mod healpix;

#[cfg(test)]
mod tests;
