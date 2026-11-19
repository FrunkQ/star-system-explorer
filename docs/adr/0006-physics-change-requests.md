# ADR 0006: physics changes are change requests carried in a campaign pack

Status: accepted (U3, 18 Nov 2026)

## Context
The flow editor lets a GM change the physics (H13: Orac's change requests; build plan U3: "open a real flow, change a parameter, see diff and badge"). The base packs are shared, hashed data and must never be edited in place; every value must keep saying which pack published it (K5), and the U0 rule says every change goes through the session as an intent.

## Decision
- A physics change is a `ChangeRequest` document (`packages/ui-contract/src/physics.ts`): flow id, parameter edits (node, parameter, value in force, proposed value), author (GM or Orac), rationale, status (draft, adopted, declined, superseded). Orac (A2) will file the same documents.
- Adopted requests are compiled into one campaign pack, `campaign.physics`, stacked on top of the base packs. It carries an edited copy of each touched flow and an explicit `replaces: {flow, pack, range}` for each. The base pack is never changed.
- Validation and preview use the kernel itself: `Session.trial(pack, refs)` loads the candidate stack, derives the refs and restores the stack in force, inside the session's request queue so no other call sees the temporary stack. A request the kernel rejects (for example a parameter outside an atom's range) never reaches the Adopt button.
- Adoption is one intent, `adopt_physics {pack, requests}`, translated by the one `intentToRequest` into `load_stack [...base, pack]`. Only a GM seat applies it; any other seat's goes to the GM. Withdrawing is the same intent with the request left out (or `pack: null` for the base physics as shipped).
- Every derived value records its publisher; values published by a non-base pack are badged "changed physics" (card field mark, subject stamp, explain stamp, flow editor header).

## Consequences
- Physics changes are not chronicle events yet: the kernel has no op to record a stack change, and the run manifest still lists the packs loaded when the universe was opened. The state hash does not move when physics changes. Reported to the kernel track; until then, the change-request list in the workspace is the record.
- Flow documents carry the base flow's `tests`; the kernel does not run flow tests at `load_stack` today. When it does, an edited flow will need re-baselined tests (or an exemption rule), decided then.
