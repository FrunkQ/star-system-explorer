# SB2: small bodies through the kernel

Track SB, thread "Build asteroids and rings up close", 24 Nov 2026.

## What was built
- `crates/sse-kernel/src/small.rs`, dispatch arms in `lib.rs`, requests and response in `protocol.rs` (cost rows Trivial), perf-chunks sources in the "matter" chunk.
  - `small_body {system, member?}` or `{system, cloud, rank?}` resolves one member with `sse_small::resolve` (host star luminosity at the system's age, primary mass, the run's upheaval); with neither, `notables` lists the bodies formation made (a shattered world's core), each resolved, with a `why`. Response `small_body_is {body | notables}`.
  - `make_small_body {system, address, kind, diameter_km | mass_kg, a_au, e?}`: kinds asteroid, comet-nucleus, contact-binary, binary-pair. The body is a one-member cloud copied from the system's nearest cloud (`sse_small::made_field`, fudge SB-made-field; comet nuclei prefer oort, scattered-disc and outer-belt clouds) and resolved by the same method with pins (`resolve_pinned`, `Pins {pair, e}`): a pinned pair kind picks its mechanism from that kind's odds. The answer carries `made_from`.
  - The run is formed on first ask if the system has a formation request but no run yet, as `read_inventory` does.
- `sse-small`: `resolve_pinned`, `Pins`, `made_field`, `mass_for_diameter`, `nearest_cloud`.
- Settings are parsed once per process (`OnceLock`): parsing the sheet for grain densities was the slow part.

## Evidence
- `cargo test -p sse-kernel --lib small_tests`: a belt member resolves the same every time and from either address form; rank 0 refused; notables list; made contact binary and comet nucleus follow their pins; unknown kind refused.
- Golden: `tests/golden/contract.jsonl` gained 8 appended lines from 8 appended cases in `tests/contract/requests.json` (load base-physics, open universe "small", adopt kernel-form-9, notables, a member by cloud and rank, rank 0 refused, a made contact binary, an unknown kind refused). No existing line changed. Reason: new ops (PR 454 split means new ops only append).
- TypeScript types regenerated (`Request.ts`, `Response.ts`).

## Not done
- Members are not yet listed at their own address in `resolved_system`, so the resource books do not see them until a reader asks `small_body` (Traveller seam).
- `sse.shape.spin_figure` formula block: listed under candidates in docs/fudges.md.
