# SSE4 performance readout and minimum spec (track PF)

Status: PF1 built 22 Nov 2026 (thread "Measure performance for minimum specs"). PF2 and PF3 proposed here. Questions for Alex in section 7.

Alex, 22 Nov 2026: "maybe add in some options performance measurement stuff that can be switched on in settings that provides extra debug info and gives us a chance to start building a 'minimum spec' for each big chunk."

Why now: ND3b (moving Rust atoms into Equation blocks, `SSE4-equation-block-design.md` 8B) requires each moved family to stay within 5 % of its previous time on the desktop and the phone tier. That needs the same workload timed the same way on known machines, before and after.

## 1. One mechanism, three uses

One recorder, one chunk list and one report format serve:

1. **The readout** (PF1): a switch in More, "Performance readout", off by default. While on, a small card shows each big chunk's calls, median and 95th percentile against its budget, the frame time, start-up, memory and long tasks. Any row opens to its ops.
2. **The bench** (PF2): the same chunks timed on fixed workloads on the device check page `/harness/`, so two machines run exactly the same work. Run in a browser by anyone, or by script in Node, Chromium and Chromium with its CPU slowed four times (the phone stand-in GH0 used).
3. **The minimum spec table** (PF3): `docs/minimum-spec.md`, one row per chunk, built by a script from reports kept in `docs/perf-reports/`.

It builds on what exists, with no second mechanism:
- The kernel's device measure and tiers (K6: basic, standard, capable by work units a second) name the device classes.
- The renderer's frame-time governor already measures the globe; the readout records the same frame work.
- The view styles design filters by machine capability; it can later read the minimum spec table instead of guessing.
- `/harness/` already runs every engine scenario in the browser for the hash check; PF2 times the same scenarios.

## 2. The big chunks

The list is data (`packages/ui-contract/src/perf-chunks.json`, schema `sse.perf-chunks/1`), so it can be edited without code. A test fails when a new kernel op has no chunk; anything unlisted still shows, under Other, by its op name.

| Chunk | What it covers | Budget (95th percentile) |
|---|---|---|
| Opening the app | starting the kernel, loading packs, opening the universe, the device measure | 5 s |
| Working out a system | resolve: derive, explain, scene, resolved system, edits, search | 100 ms |
| Making and ageing a system | V3 import, formation run, look-ahead | 5 s (shows progress) |
| Climate and weather | `weather` | 300 ms |
| Surfaces, caves and descents | surface frames and surveys, caves, descents | 2 s |
| Light, spectra and the sky | light, spectra, activity, air optics, nebulae, planet finding, the sky from a world | 300 ms |
| Matter and fields | data sheet look-ups, phases, magnetic fields | 100 ms |
| Life | `life_view` | 300 ms |
| Galaxy and cosmic web | galaxy chart, star and halo cells, web slices, the Window, sectors | 1 s |
| Orbits, routes and voyages | orbits at a time, calendar, routes, voyages | 300 ms |
| Seats, rules and programs | the game layer | 100 ms |
| Drawing | the globe's own frame work, the page's frame interval | 33 ms (30 frames a second) |
| Clicks and keys | click or key to the next paint (event timing) | 200 ms (the web's line for "good") |

Budgets follow the standing rule "no heavy work for a click": a click-time read stays under 100 ms; a deliberate act (a survey, a galaxy chart) may take longer; a run with a progress bar has a whole-run budget. A chunk's verdict is within, near (over three quarters of its budget) or over.

Detail levels: a request that names its `detail` or `level` (Orbit, Survey) is recorded per level, so Survey is never averaged with Orbit.

## 3. What is measured, and how

- **Kernel calls.** The kernel client has one observer for every call and job on the page (`setCallObserver`), set only while the readout is on. Each call gives its op, detail, the round trip, the kernel's own time (read in the worker around `handle`), bytes in and out, and the kernel's WebAssembly memory afterwards. A job is timed whole as `job:<kind>`.
- **Frames.** The globe runtime records its own drawing time per tier (`render.globe`). The page's frame interval comes from one `requestAnimationFrame` callback a frame that draws nothing (`page.frame`).
- **Clicks and keys.** The browser's event timing, one entry per interaction (`page.interaction`). Long tasks (over 50 ms on the main thread) are counted.
- **Start-up.** Stamps since the page began: packs fetched, kernel ready, universe open, device measured, first chart. Kept even while the readout is off (five numbers a visit), so switching it on later still shows them.
- **Memory.** The kernel's WebAssembly memory; the page's JavaScript heap where the browser says (Chromium only).
- **The device.** User agent, cores, `deviceMemory`, screen and pixel ratio, the kernel's tier and units a second, the GPU backend and adapter name, reduced motion.

Determinism: every time is read on the page or in the worker, never inside the kernel, and never enters a hash. `clippy.toml` still forbids the wall clock in Rust.

Cost: off, a kernel call pays one null check. On, a call pays two clock reads and a short regex on the first 256 characters of the request, plus one empty animation-frame callback a frame.

## 4. Reports

`sse.perf-report/1`: label (what the person calls the device), source (`app` or `bench`), the day (no time), kernel version and site, the device description, start-up stamps, chunk summaries with their ops, memory and long tasks.

Nothing leaves the device. A report is made only when the person presses Copy report or Save. It holds timings and what the device says about itself (including the GPU adapter name), never a universe, a seed, a body's name or a campaign.

## 5. PF2: the bench (proposed, next)

- `sse-wasm` gains `engine_names()` and `engine_run(name)`, so each harness scenario (resolve, formation pipeline, surface atlas, galaxy, life history, nav and the rest, 38 in all) runs alone.
- `/harness/` times each one (three runs, the median) after its hash check and maps it to a chunk with the same chunk list, then offers Copy report and Save with `source: "bench"`.
- `tests/harness/bench.mjs` runs the same in Node, headless Chromium and Chromium with CPU slowed four times, writes a report per run, and with `--against <report>` lists every scenario more than 5 % slower. That is ND3b's timing check, run on one machine before and after a move.
- Not in CI: cloud runners vary by more than 5 % from run to run, so a 5 % gate would fail at random (question 4).

## 6. PF3: the minimum spec table (proposed)

`scripts/min-spec.mjs` reads every report in `docs/perf-reports/` and writes `docs/minimum-spec.md`: for each chunk, each device's 95th percentile against the budget, and the slowest device class that stays within it. Device classes are the kernel's tiers (basic, standard, capable) crossed with the GPU backend (WebGPU, WebGL2, none), with named reference machines where Alex sends reports (AC-ProArt, his phone). The table is regenerated, never typed.

## 7. Questions for Alex

1. **Players too.** Give the players' page the same switch (in its settings) so phone timings come from real players' devices? Recommended: yes, as PF1b; players' phones are the likeliest minimum spec.
2. **Where the switch lives.** It is in More now, as "Performance readout". When the chart room gets one settings place, it moves there. Recommended: yes.
3. **The budgets** in section 2 as the starting line, edited as data when they prove wrong? Recommended: yes.
4. **No timing gate in CI for now.** PF2's script checks 5 % on one machine before and after; a CI job could later run the bench and attach the table without ever failing a build (this would edit a workflow, so it needs your yes then). Recommended: script only now.
5. **Reference devices.** Run the bench on AC-ProArt and your phone once PF2 lands and paste the reports here, so the first table has real machines beside the cloud runner? Recommended: yes; it takes a minute on each.
