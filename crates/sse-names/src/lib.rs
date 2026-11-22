//! SSE4 names (track NM, `handover/design/SSE4-naming-design.md`).
//!
//! - [`scheme`]: a name scheme as pack data (`provides.names`), its load
//!   checks and the [`scheme::Library`] a stack holds, with the avoid list.
//! - [`make`]: names as keyed draws on their own stream, the stored form
//!   that follows a renamed parent, the faces (spelling, say it, IPA,
//!   native script), and the sibling clash rule.
//! - [`text`]: text safety, composition and the fold.
//!
//! Names are labels: nothing here reads or writes a physical fact, and no
//! flow can take a name as an input.
#![forbid(unsafe_code)]

mod compose_table;
pub mod make;
pub mod scheme;
pub mod text;

pub use make::{name, siblings, Named, Request};
pub use scheme::{Library, Scheme};

#[cfg(test)]
mod tests;
