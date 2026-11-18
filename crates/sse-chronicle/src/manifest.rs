//! Run manifest (H5.6): what identifies a shareable universe.

use serde::{Deserialize, Serialize};
use sse_core::{ContentHash, Seed};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PackRef {
    pub id: String,
    pub version: String,
    pub content_hash: ContentHash,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct RunManifest {
    /// Schema id of this document.
    pub format: String,
    pub universe_seed: Seed,
    pub seed_algorithm: String,
    pub kernel_version: String,
    /// The pack stack, bottom to top.
    pub packs: Vec<PackRef>,
    pub authored_chronicle_hash: ContentHash,
    pub ordering_version: String,
    #[serde(default)]
    pub checkpoints: Vec<ContentHash>,
}

pub const MANIFEST_FORMAT: &str = "sse.run-manifest/1";
pub const ORDERING_VERSION: &str = "sse.order.v1"; // (activated_at, recognised_at, id)

impl RunManifest {
    pub fn new(
        seed: Seed,
        kernel_version: &str,
        packs: Vec<PackRef>,
        authored: ContentHash,
    ) -> Self {
        RunManifest {
            format: MANIFEST_FORMAT.into(),
            universe_seed: seed,
            seed_algorithm: sse_core::draw::DRAW_ALGORITHM.into(),
            kernel_version: kernel_version.into(),
            packs,
            authored_chronicle_hash: authored,
            ordering_version: ORDERING_VERSION.into(),
            checkpoints: Vec::new(),
        }
    }
    pub fn id(&self) -> ContentHash {
        ContentHash::of(self).expect("manifest is canonical")
    }
}
