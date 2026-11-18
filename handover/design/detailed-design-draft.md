> Superseded on 17 Nov 2026 by SSE4-design-handoff.md and its companions in this folder.

# SSE4 detailed design (draft 1, 13 November 2026)

Working draft while the docs service is down. It moves into a Claude Doc beside the high-level design when the service is back. Mockups follow as separate pages.

## 0. Decisions carried in (Alex, 13 November)

| Decision | Value |
| --- | --- |
| Engine | SSE's own, fit for purpose. Ovid Next is inspiration for the shape only (node editing, AI agents). No Ovid code or runtime. |
| Licence | GPL |
| Repo | https://github.com/FrunkQ/star-system-explorer (new, clean start) |
| Hosting | Cloudflare, on a v4 subdomain of starsystemx.com |
| Language and framework | Open to change |
| Agents | Orac (builder), Zen (support), Minerva (storyteller) |
| New base blocks | Specced by Orac and raised as GitHub change requests; most physics is built from existing blocks |
| Base blocks are general | e.g. one rotation block serves oblateness, black hole spin limits, binary infall (captured or flung away), weather |
| UI | Clean, fit for every device, not generic. Main UI simple, as now. Renderers add look and feel to player views. |
| Routing and transits | Total rethink: interactive on the map, not side-panel menus; built to create stories (running out of fuel); hazards highlighted |

## 1. Engine: language and runtime

**Recommendation: a Rust physics kernel compiled to WebAssembly, with a TypeScript interface layer.**

| Option | For | Against |
| --- | --- | --- |
| Rust kernel in WebAssembly | Identical floating-point results in every browser (WebAssembly arithmetic is IEEE-exact; we ship our own sin, exp, pow), so "same seed, same bytes" is enforceable; 3 to 10 times faster on numeric loops; runs headless natively for game engines; strong types for units | New language for the project; slower iteration for physics authors; build tooling |
| TypeScript throughout | One language; fastest to start; V3 physics ports almost line by line | JavaScript `Math` functions may differ between engines, so determinism needs our own maths library anyway; slower on Gyr history computations |

Either way: the kernel runs in a Web Worker, never on the page thread, and exposes one message API (load pack, set authored model, derive, compute history, plan route, query trace).

Decision needed from Alex: Rust/WebAssembly or TypeScript.

## 2. The node model in detail

- **Atom**: `{ id, version, ports: { in: Port[], out: Port[] }, cost, citation, tests, impl }`. Implementation is kernel code. Pure: output depends only on inputs.
- **Port**: `{ name, type, unit?, default? }`. Types: `Scalar<Dimension>`, `Vec3<Dimension>`, `Spectrum`, `Composition` (element and material masses), `Body`, `Region`, `Shell`, `Layer`, `TagSet`, `Rng`, `Time`, `Table`.
- **Dimension checking** at wire time: SI base exponents (mass, length, time, temperature, amount, current). Display units are a view setting, never stored.
- **Box**: a flow file `{ id, version, inputs, outputs, nodes, wires, settings, contributes? }`. `contributes` declares hooks (e.g. `star.surface.spectrum`) and an applicability test.
- **Solve**: `{ state ports, body, damping, maxIterations, tolerance }`; reports iterations and residual in the trace; a non-converging solve publishes the last state plus a `physics/unconverged` tag, never throws.
- **Pipeline compile**: topological sort of nodes plus hook expansion from installed packs, cached by (pipeline version, installed pack versions).
- **Trace**: every node run records inputs, outputs, time taken and memo hit. Kept for the selected body; summarised for others.

## 3. Data model

- **Authored store** and **derived store**, separate. Undo, save and diff operate on the authored store only.
- **Body**: identity, formation time, orbit (with explicit reference frame), shells[], regions[], layers[], features[], manual tags, overrides (categorical and numeric), plausibility label.
- **History**: per system, a list of dated events `{ t, kind, bodies, payload }` computed once from seed plus authored model; scrub reads it.
- **Hierarchy**: body, system, cluster, galaxy, group, filament; one node type with a scale tier and unit.

## 4. Routing and transits, rethought

Goal: plotting a journey is a direct, playful act on the map, and the physics produces the drama.

**Interaction**
1. Select a ship; its reachable region shows as a soft halo for the fuel it has (Δv budget against the current positions).
2. Drag from the ship to a target. A live trajectory follows the pointer, recomputed as you move (Lambert for coasts, constant-thrust for torch burns, chosen by the ship's drive).
3. The route bar under the map shows departure, arrival, Δv used, fuel left, peak G and dose, with one slider: faster versus cheaper.
4. Drop waypoints on the line to add a gravity assist, a refuel stop or a flyby. The line re-solves around them.
5. Commit, and the journey is scheduled against campaign time.

**Hazards drawn on the route**, not in a panel: radiation belts as tinted stretches with the dose, close passes as markers, G-limit breaches in red, eclipses and comms shadows, debris and ring crossings.

**Story mechanics**
- **Bingo fuel**: a route that leaves less than a set reserve turns amber; one that cannot stop at the target shows where the ship ends up instead.
- **Running dry**: a ship out of fuel shows its drift path through deep time, the nearest bodies that can rescue it in time, and any gravity assist that could save it.
- **What changed**: because routes are planned against the time model, a route crossing a long span re-reads positions along the way.

**Engine side**: route solvers are blocks and packs like the physics (Hohmann, Lambert, brachistochrone torch, relativistic, jump), so a setting can add its own drive.

## 5. Mockups to produce next

1. Edit mode on desktop and phone: a system with one body selected.
2. Routing: drag-to-plot with hazards and the fuel bar; and the out-of-fuel drift view.
3. The physics flow view for one body (the Newton panel as the live trace), opened from edit mode.
4. Deep-time slider across a group of systems.

## 6. Questions still open for Alex

1. Rust/WebAssembly or TypeScript for the kernel.
2. February release scope (suggested: kernel, V3 physics ported, edit mode, 2D and 3D views, deep time for stars and planets, routing).
3. Old saves: fine if a world classifies differently, with a note of what changed?
4. Classification as user-extendable rules in packs?
5. Galaxies in the first release or later, and the end of "again we will bring in the".
6. Oldest device to support.
7. Selling on the explorers site, or all free?
