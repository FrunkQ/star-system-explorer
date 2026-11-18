# Numbers: one value, one hash, on every client (cross-track change)

Status: done (18 Nov 2026), kernel track, branch `claude/project-thread-dzkz4p`. Asked for by the coordinator after the worker contract found two traps (see `K6-additions.md`). **Every track must merge this and re-baseline** (below).

## What changed
1. **Exact float parsing.** Root `Cargo.toml`: `serde_json = { version = "1", features = ["float_roundtrip"] }`. serde_json's default parse is not correctly rounded (`1.2e27` came out one ulp off and re-serialised as `1.2000000000000001e+27`), so text written by JavaScript and by Rust could decode to different floats. With the feature, every float parses exactly and the shortest text round-trips.
2. **JSON numbers decode by value** (`sse_core::cbor`, `Deserialize for Value`, a K1 interface). JSON has one number type and the kernel's clients are JavaScript, where `JSON.stringify(281.0)` is `"281"`. So in any human-readable format a number that is integral and smaller than 2^63 in size (the i64 range) becomes `Value::Int`; everything else is `Value::Float`. (First pushed with a 1e21 limit; C1 found integers between 2^63 and 1e21 could not be written back as JSON, so the limit became 2^63 in C1, with no golden moved.) `281`, `281.0`, `2.81e2` are one value with one hash. Binary CBOR (stores, `from_value`) keeps the types as encoded, so stored events and their ids are untouched. NaN is still refused; `-0.0` from JSON becomes 0, as JavaScript writes it.
   - Why by value and not by declared quantity type: authored fields, pack `provides.other` content and flow parameters are free-form, with no declared type to consult, and the K1 encoding hashes `Int` and `Float` differently. One rule at the decode point covers every path at once and is exactly JavaScript's own behaviour, so nothing a JS client can send is ambiguous.
   - Readers: anything reading a number from a `Value` must accept `Int` and `Float` (every crate already does via `as_num`/`as_f64` helpers; a typed `f64` field deserialised from a `Value` accepts integers). Code that matched only `Value::Float(_)` on JSON-sourced data was checked: `sse-cosmos::birth` and `sse-suite::scenario` already take both. Code that builds `Value::Float` in Rust (flows, resolve) is unaffected.

## Evidence
- `sse-core` `json_numbers_decode_by_value_as_javascript_writes_them`: the spellings, the 2^63 edge, `-0.0`, exact `1.2e27`, and binary CBOR keeping `Float(281.0)`.
- `sse-kernel` `number_spellings_hash_identically`: `281`, `281.0`, `2.81e2`, `2.810E+2` give one event id and state hash (pinned `c0b8445b…`); `1.2e27`, `1.20e+27` and the 28-digit integer spelling agree.
- `packages/kernel-client` test "number spellings give one hash through the worker": raw text through the Node worker gives the same pinned hash.
- Full suite, worker contract (98 of 98), Node and Chromium harness: pass.

## Goldens moved (on purpose)
Only one file, after merging the physics branch at 638f241:
- `tests/golden/observe.json`: `gm` and `tarn_after` digests. Cause: `tests/fixtures/observe/tarn-world.json` writes integral floats (`130000.0`, `18489600.0`, `94246658541.0`, `281.0`), which now decode as integers, exactly as the worker would send them. `tarn_before` is unchanged (those facts are hidden there).

Unchanged: `harness.json`, `contract.jsonl` (its fixtures were already written as JS writes them), `resolve`, `sift`, `suite`, `cosmos`, `formation`, `formation-early`, every pack content hash.

## For the other tracks
- Merge `claude/project-thread-dzkz4p` (or cherry-pick this commit), run your goldens, and re-bless only what moves, naming this change as the reason. A move means a fixture spelled an integral number as a float.
- Write fixture numbers as JavaScript would (`281`, not `281.0`) so the files read the same as what the UI sends.
