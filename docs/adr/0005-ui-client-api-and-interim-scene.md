# ADR 0005: one client API for every interface; interim scene and marks until K9 and V12

Status: accepted (U0, U1).

## Decision
- `packages/ui-contract` is "the same client API the player app uses" (H8.8). It holds the view contract (H8.1), screen documents (H8.3), intents, seats and the approval rule (H8.7), and the `Session`, which is the only code outside `kernel-client` that talks to the kernel. Shells and views see a `ViewSession` interface; the kernel client sits in a true private field.
- Every edit is an intent. `intentToRequest` is the single translation from an approved intent to a kernel `set_authored`, so a GM edit and the same intent from an auto-approved player seat produce byte-identical events. The GM seat is `authority: "gm"` with audience `all`; nothing else distinguishes it.
- `tests/ui/boundary.test.ts` enforces the rule: no file under `packages/shell`, `packages/views`, `packages/chart-grammar`, `packages/skins` may import anything outside those folders except `ui-contract/src/index.ts` and `ui-contract/src/host-browser.ts`, or any package but `svelte` and fonts, or name a kernel internal.
- Universe time on the ruler is whole Julian years (365.25 d) since the universe origin; the session converts to kernel seconds exactly with bigint and writes them as digits (JSON.stringify refuses bigint).

## Interim parts (to be replaced, not extended)
- `SystemScene` (`ui-contract/src/scene.ts`) stands in for kernel-built scenes until K9 (H6.7). The shell loads a fixture in this shape and labels it "Fixture data" on the chart. Values shown after an edit come from the intent result, since the kernel has no read of authored field values yet (`subscribe` arrives with K9).
- `markFor` (`ui-contract/src/mark.ts`) is a minimal table evaluator of a symbology document so no view hard-codes an encoding (H8.4A). V12 replaces it with the full symbology engine; the shell's `diagrammatic.stub.json` is placeholder data, not V3's class table.
- View packages live under `packages/views/*` now (H15 layout). V1 formalises the contract further (scene subscriptions, tiers probe); the mount signature here passes the scene directly.

- U2: derived values come from the kernel through `derivation.ts` and a derivation document (which flow serves which object, which facts feed which inputs). It is interim scene building, replaced by K9; it never computes physics, only scales units for display.

## Consequences
Player apps (V-track) reuse the session and views unchanged. A co-GM or a sensors seat with `auto_approve` scopes works with no new path. Anything a GM tool needs from the kernel must be added to the session, where the player app gets it too.
