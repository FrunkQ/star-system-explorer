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
The core golden did not move. Two moved on purpose, each by its stack or pack hash only:
- `tests/golden/areas/time.jsonl`, both `stack_loaded` lines: TP4's base-time carries the Hubble example as `456056396909749800` (written as a browser writes it), which is now read as the double it stands for, 456056396909749824.
- `tests/golden/time.json`, the base-time `manifest` hash, for the same number.
Every ladder rung, event, window, chronicle head and run digest is unchanged.

## What must stay exact, checked (27 Nov)
The rule only touches free-form JSON numbers: `cbor::Value` read from JSON (authored values, program and sifter values) and the untyped parts of a pack. It rounds nothing that a browser had not already rounded, because a browser holds every JSON number as a double before the kernel sees it.
- Typed fields are read by their type and untouched: every `at_s`, `occurred_at_s`, `now_s` is an `i64` field. `typed_seconds_past_2_53_stay_exact` sets the campaign's now to 435494880000000001 s natively and reads it back exact.
- Seeds are 32-byte hashes, sent as seed text (`open_universe.seed`, postcards); hashes and ids are hex strings.
- Saves (`export_chronicle`) carry every event as hex canonical CBOR, which is binary and keeps its integers as encoded; this rule never reads them.
- Shipped packs and fixtures: 211 whole numbers above 2^53, every one a double exactly (the only exceptions were digits inside DOI strings, and a test golden the kernel never reads back). So no pack number changes value today.

Not this package's fault, but found here, for TP2: a browser sends a time in seconds as a JavaScript number, so past 2^53 s (285 million years after the Big Bang) the web cannot name every second. At today's date a double is 64 s apart. Natively the kernel keeps every second; from the web, times "to the second" need to travel as text or as a BigInt-safe form. TP2 owns that choice.
