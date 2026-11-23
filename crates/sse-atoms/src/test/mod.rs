//! ND3b: the blocks that moved from Rust into formula files, as their Rust
//! was before the move. Test-only: `moved_blocks_keep_their_faces_and_bits`
//! checks each formula block against its old self, face and bits.
#![allow(dead_code, unused_imports, unused_variables, clippy::all)]

pub mod activity;
pub mod air;
pub mod climate;
pub mod convection;
pub mod heat;
pub mod layers;
pub mod light;
pub mod magnetic;
pub mod nebula;
pub mod tides;

pub fn atoms() -> Vec<Box<dyn sse_flow::Atom>> {
    let mut all = vec![];
    all.extend(activity::atoms());
    all.extend(air::atoms());
    all.extend(climate::atoms());
    all.extend(convection::atoms());
    all.extend(heat::atoms());
    all.extend(layers::atoms());
    all.extend(light::atoms());
    all.extend(magnetic::atoms());
    all.extend(nebula::atoms());
    all.extend(tides::atoms());
    all
}
