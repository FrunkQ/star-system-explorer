# RW3: the catalogue format and its reader

Thread "Build the catalogue format and reader", 26 Nov 2026. Track RW (`design/SSE4-real-catalogues-design.md`, accepted 26 Nov).

## Built
- Crate `crates/sse-catalogue`: the `sse.catalogue/1` header and its checks, the text form (JSON), the packed form (byte-ranged blocks, SHA-256 each, blocks end at every named step), `read_packed` over any `ByteSource`, `Packed::plan` for a cut by size or rank, `read_block` for one piece on its own, the preview, the price rule, a base64 codec. Format spec: `design/SSE4-catalogue-format.md`.
- Kernel op `read_catalogue` (import preview; read-only, no universe). New contract area `catalogues` (`tests/contract/areas/catalogues.json`, golden `tests/golden/areas/catalogues.jsonl`). Core golden unchanged.
- `sse catalogue pack|show` in the CLI, for build scripts on Alex's PC.
- Fixture `tests/fixtures/catalogues/test-belt.catalogue.json` (invented values; re-written by `SSE_BLESS=1 cargo test -p sse-catalogue`).

## Gates, each seen red with the fix removed
- `a_prefix_read_never_touches_the_rest_of_the_file`: with the plan widened to every block it fails (the source refuses any byte past the plan).
- `a_cut_at_a_step_reads_exactly_to_it`: fails with the same change.
- `no_credit_no_reading`: fails with the credit check removed.

## Golden
New area `catalogues` only (blessed 26 Nov): the reason is the new op. No other golden moved.

## Not done (later packages)
Estimating flows (RW3b), populations and the table read (RW4), the Solar System files (RW7), store byte ranges (RW8), motion models (RW17). Nothing here estimates or fills a value.
