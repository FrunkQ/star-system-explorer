# ADR 0003: canonical encoding profile

Status: accepted (K1).

Kernel state, chronicle events and network frames use canonical CBOR, implemented in `sse_core::cbor` (our own encoder and strict decoder, so no dependency can change bytes). Profile, a subset of RFC 8949 section 4.2:

- shortest integer heads, range -2^64 to 2^64-1; definite lengths only;
- map keys are text only, sorted by encoded bytes (length, then bytewise), no duplicates;
- floats always 64-bit (no shortest-float rule: simpler and unambiguous); NaN rejected; -0.0 kept distinct;
- no tags, no undefined.

The decoder rejects anything non-canonical, so decode then encode returns the same bytes. Typed Rust values cross via serde (`to_canonical` / `from_canonical`).

Human-edited documents (flows, packs, screens) are JSON on disk; their content hash is the SHA-256 of the canonical CBOR of the parsed value, so JSON whitespace and key order never change a hash and no JSON number formatting rule is needed.

The worker boundary (K6) carries JSON text; serde_json and JavaScript both round-trip f64 exactly through shortest decimal form.
