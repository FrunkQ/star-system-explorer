# Evolution Lab: what SSE4 reuses, and how it grows

Status: K0 draft for Alex's review. Source read: Evolution Lab `ARCHITECTURE.md` (sections 0 to 7),
`docs/IMPLEMENTATION_DIRECTION.md`, `package.json`, and Evolution Lab Design `docs/CONTRACTS.md` (contract index).
Design authority on the SSE4 side: `handover/design/SSE4-life-design.md` (stages L1 and L2) and the design handoff
section 11A (interfaces and stages L0 to L5). This note checks those against the real Evolution Lab and records what
lines up and what does not.

## 1. The fit, in one paragraph

Evolution Lab already insists on what SSE4's kernel now provides: determinism (`INV-DET`), named order-independent
draws (`CTR-RANDOM-DRAW`), content-hashed checkpoints with exact resume and fork (`CTR-CHECKPOINT`,
`CTR-FORK-MANIFEST`), a run manifest (`CTR-RUN-MANIFEST`), an immutable causal chronicle with occurrence, activation
and recognition times (`CTR-CHRONICLE`, `CTR-EVENT-TIME`), evidence labels (`CTR-EVIDENCE-LABEL`) and three-lens help
(Curious, Biology, Engine). Its own boundary rule `BOUND-SSE` says it must never import SSE stores or types and meets
SSE only through adapters. So reuse is by contract, not by copying a runtime.

## 2. Contract by contract

| Evolution Lab contract | SSE4 counterpart | State |
|---|---|---|
| `CTR-RANDOM-DRAW` order-independent named randomness | Keyed draws `sse.draw.sha256.v1` (K2, ADR 0004) | Same idea. Evolution Lab's draws are TypeScript and were not checked against our algorithm in K0, so assume no byte match; the suite socket (K10) runs mechanisms on SSE4 draws. |
| `CTR-EVENT-TIME` occurrence, activation, recognition | Chronicle fold order (activated_at, recognised_at, id) (K3) | Matches. |
| `CTR-CHRONICLE`, `CTR-CHECKPOINT`, `CTR-FORK-MANIFEST` | Chronicle, checkpoints, branches (K3) | Matches. A fork is a branch. |
| `CTR-RUN-MANIFEST`, `CTR-RUN` | RunManifest (K3) | Matches; the suite adds its mechanism versions to it. |
| `CTR-EVIDENCE-LABEL` | Validity labels and event `evidence` field (K4, K3) | Map: Evolution Lab's statuses become SSE4 evidence values. Exact list to fix at L0. |
| `CTR-EXPLANATION-BUNDLE`, `CTR-HELP-VIEW` (Curious, Biology, Engine) | Explanation bundles with curious, science and engine lenses (K4) | Matches. Biology is the life suite's name for the science lens. |
| `CTR-ENV` environment provider (habitat facts: medium, phase, temperature and pressure ranges, cycles, resources, energy gradients, radiation, aerosols) | SSE4 publishes these as quantities from physics atoms (P1, P4) through the pack hooks (K5) | **The main seam.** Every `CTR-ENV` fact must have exactly one SSE4 publisher. L0 lists them; missing ones become atom issues (P5). |
| Engine returns fluxes and deposits; tags are derived outputs, never the only output | Life outputs are authored-by-suite events with quantities; tags derived by rules (K8) | Matches the V3 lesson in `docs/v3-parity.md` section 1. |
| `CTR-SIGNATURE` active effects, preservation, detection, interpretation | Disclosure tiers and data views (K9) | Matches the life design's "true state, surviving evidence, what an observer detects". |
| `CTR-HANDOFF` biological to technosphere seam | Civilisation tags and later technosphere pack | Deferred in both projects. |
| `CTR-MECHANISM-PACK`, `CTR-DOMAIN-RULEPACK`, `CTR-PRESENTATION-PACK`, `CTR-SCENARIO-PACK` | SSE4 packs (`sse.pack/1`, K5) | Evolution Lab's packs become kinds of SSE4 pack. One-publisher and explicit `replaces` apply. |
| `CTR-ABSTRACTION`, `CTR-BACKMAP-MANIFEST` adaptive hierarchy and fine-state reconstruction | Envelopes and partitioning (K7) | Same "probabilities to deterministics" shape Alex asked for: an aggregate resolves to detail only when looked at. |
| `CTR-FEASIBILITY-QUERY` bidirectional constraints and inverse generation | Feasibility queries (K7) | Matches; used by "give me a world with X". |
| `CTR-MODEL-DEVELOPMENT`, `CTR-TUNING`, `CTR-EVALUATION-*`, `CTR-FITNESS-VECTOR` | Not in SSE4 | Stays in Evolution Lab, the workbench for proving mechanisms. SSE4 only receives promoted mechanisms with their fidelity evidence. |
| `CTR-VIEW`, `CTR-TEMPORAL-VIEW` reusable Svelte view models | View contract (V1) and biosphere view (U7) | Evolution Lab's Svelte components are Svelte 5, like SSE4's UI; the life timeline view can be ported against the V1 contract. |
| `CTR-FLUX-ACCOUNTING` unit-bearing flows | SI quantities with dimension checks (K2) | Matches; fluxes carry SSE4 dimensions. |

## 3. What is lifted, and when

- **L0 (interfaces):** no code lifted. Write the `CTR-ENV` publisher list, the evidence-status mapping and the suite
  socket schema (K10) with Evolution Lab's contract IDs cited.
- **L1 and L2 (procedural biosphere, SSE4-native):** no Evolution Lab code. The guild resolve, roles and four lenses
  (ancestry, resource, habitat, capability) follow its vocabulary so L3 boxes slot in.
- **L3 onward (mechanisms box by box):** a mechanism enters only after Evolution Lab has qualified it
  (`SESSION-01`, `SESSION-02` and later evidence packets), and with a fidelity contract against the procedural box it
  replaces, as the life design's stage table says. It is ported to Rust under `sse_core::math` (TypeScript maths is
  not bit-stable across engines) and checked against Evolution Lab's own fixtures within a stated tolerance.
- **Views:** `LevelsThroughTime.svelte` and the temporal projection are candidates for U7, ported against V1.

Evolution Lab is relicensed to Apache-2.0 with a notice crediting Alex (Alex, 17 Nov), so lifting is clear once that
licence is in its repo.

## 4. Differences to settle at L0

1. **Runtime language.** Evolution Lab is TypeScript (Svelte 5.56, Vite 8, Vitest 4, TypeScript 5.9.3). SSE4's kernel
   is Rust. Mechanisms port to Rust; Evolution Lab stays TypeScript as the workbench. Recommend that split.
2. **Draw algorithm.** As section 2: no byte parity between the two projects' randomness, only statistical and
   fidelity parity. Recommend accepting that rather than porting SSE4's draws into Evolution Lab now.
3. **Galactic mode.** Evolution Lab `OBJ-004` wants a domain-neutral engine reusable for galaxy evolution. SSE4's
   galaxy design uses the same K7 envelope machinery, so both suites plug into one socket (K10). No extra work in M1.
