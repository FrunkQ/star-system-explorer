# TP4b: one reading of a JSON number, so web and native hash every pack alike

Kernel, 27 Nov 2026. Found by TP4 (handover/packages/TP4.md, "A trap found on the way"); a determinism fault, fixed at its cause.

## The fault
A browser holds every JSON number as a double and writes it back by value: `JSON.stringify` gives the shortest digits that read back as the same double, plain from 1e-7 up to 1e21. So `4.560563969097498e17` goes out as `456056396909749800`, and `3.0856775814913674e16` (base-reference's parsec row) as `30856775814913670`. The kernel read the file's spelling as a float and the browser's as an integer (and not even the double's exact value, which is 456056396909749824), so a pack with such a number had one hash natively and another on the web. On main that day it was base-reference; TP4's time area hit it first.

## The rule (one, in `sse_core::cbor::json_number`)
A JSON number is first the nearest double, as in a browser; then, integral and below 2^63 in size, the integer that double is exactly; otherwise that double. It was already the rule for spellings like `281.0` (the kernel's `Value` decoder, NV2); two gaps closed:
- the decoder took integers past 2^53 as written, not as the double a browser holds; now they round through the double first (`9007199254740993` reads as 9007199254740992, as in a browser);
- packs are typed documents holding `serde_json::Value` (settings, `provides.other`), which never passed through that decoder. `sse_pack::json_numbers` applies the same rule to a pack's numbers; `PackManifest::from_json` and the new `PackManifest::from_value` (every op that takes packs: `load_stack` and the three others) read through it.

## Tests, seen red
- `crates/sse-pack/tests/web_numbers.rs`: every `packs/*/pack.json` read as written and as a browser sends it (JSON.parse then JSON.stringify, emulated) has one content hash; spellings of one number read as one. Red without the fix: base-reference.
- `packages/kernel-client/test/contract.test.ts`, "every shipped pack gives one stack hash as written and as a browser sends it": the real worker, each shipped pack on base-physics, the file's own text against `JSON.stringify(JSON.parse(text))`. Red without the fix: base-reference.
- `cbor::tests::json_numbers_decode_by_value_as_javascript_writes_them`: now says the browser digits of a double past 2^53 read as that double's integer, and that 2^63 - 1 (which a browser cannot hold) is a float.

## Goldens
None moved, core included: no fixture pack holds such a number in a free-form field (TP4 had already written its two as a browser writes them).
