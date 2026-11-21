# Converter browser module: sse-convert-wasm (21 Nov 2026)

Alex, 21 Nov (design section 15, accepted): all import and export code lives in the converter
(FrunkQ/starsystem-convert, convert.starsystemx.com), and V3 to V4 reuses SSE4's own importer, not a second copy.

## What was built
- `crates/sse-convert-wasm`: a cdylib depending only on `sse-import-v3` (without formation). Exports:
  - `import_v3(save_text, universe_id) -> string`: JSON `{universe, facts, report}`, exactly the kernel's `import_v3`
    response body without `type` and `reconstructed`, or `{error}`;
  - `importer_version() -> string`: `{importer, report_format}` for the converter's pin check.
- `sse-import-v3` gains a `reconstruct` feature (default in the crate, off in the workspace entry); only F9's
  `reconstruct` module needs `sse-formation`. The kernel and the CLI ask for the feature.
- `Import::to_value()` builds the `{universe, facts, report}` JSON once; the kernel's op and the module both use it,
  so the two cannot drift.
- `scripts/build-convert-wasm.sh` builds to `target/convert-wasm/` (wasm, JS glue and a `PIN` file with the SSE4
  commit and sha256 of each file). The converter vendors those files pinned by that commit and hash.
- `scripts/check.sh` also builds `sse-convert-wasm` natively, so the importer keeps building without formation.

## Checks
- `cargo test -p sse-convert-wasm`: the module's JSON equals the kernel op's for `tests/fixtures/v3/synthetic-sol.json`;
  a bad save is an error, not a panic.
- `cargo test -p sse-import-v3 --no-default-features` passes (the reconstruct test is behind the feature).
- Size: 703 KB wasm, 224 KB gzipped (the full kernel is about 13 MB). Dropping formation saves build time more than
  bytes: the linker already discarded most of it.
- No golden changed.
