# ADR 0004: keyed draw algorithm

Status: accepted (K2).

H5.3 asks for a counter-based generator, "Philox or a SHA-256-derived stream; pick one, pin it, version it". Chosen: `sse.draw.sha256.v1`, defined in `crates/sse-core/src/draw.rs`: the first 8 bytes of a domain-separated SHA-256 over the seed, the address text, the stream name and the counter, each length-prefixed.

Why SHA-256 over Philox: it is already a dependency (content hashes), it has no tuning constants to get wrong, length prefixes make collisions between (address, stream) pairs impossible by construction, and it is trivially identical on every target. Cost is about a quarter of a microsecond per draw natively, acceptable for resolve-time work. If profiling later shows draws dominate, a faster algorithm comes in as `.v2`, selected by the run manifest, so old universes keep `v1`.

Rules carried by the code: a universe seed is 32 bytes (`Seed::from_text` hashes whatever the GM types); stream names must end in `.v<N>`; draws are indexed, never stateful; bounded integers use Lemire's method with retries keyed on the same index; normals use Box-Muller on pinned maths.
