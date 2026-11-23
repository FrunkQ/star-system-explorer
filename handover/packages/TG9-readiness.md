# TG9: ship discipline and readiness, without numbers on tags

Track: Traveller pack. Design: `handover/design/SSE4-record-logic-programs-design.md` 8.1; acceptance row TG9 in `SSE4-tags-design.md`. Date 23 Nov 2026.

## What it does

- Kept kinds `ship.readiness` and `ship.discipline`, numbers from 0 to 1, tier `scanned`, in the Traveller pack's record block. Every ship and station template gives defaults: readiness 1, discipline by role (patrol 0.9, naval depot 0.95, courier and scout base 0.8, trader 0.6, far trader 0.5, corsair base 0.4...). These are game defaults written for this pack, not physics. Sites carry neither.
- Status tags `status.derelict` and `status.damaged`, declared in a `status` group (applies to constructs, authored, tier scanned).
- Two visible triggers in the pack:
  - `traveller.status.derelict`: when the tag appears, readiness is set to 0, once. The GM can then set readiness to anything with the tag left in place, as in "a derelict someone has quietly made spaceworthy".
  - `traveller.status.damaged`: while the tag is on, readiness is held at 0.5 or below. A GM value under 0.5 stays. Raising it above 0.5 is pulled back to 0.5 until the tag comes off.
  - Each write shows its rule as the writer in the value's history (`effect.traveller.status.*`), so "Why?" names the cause.
- Owner tags set nothing. "In transit" and "adrift" are not tags. Drive ability stays in the hull's facts (`traveller.jump` until MP16 moves it to the declared rating).

## Checks

- Kernel test `ship_readiness_follows_status_tags_through_visible_triggers`:
  - Template defaults.
  - Derelict goes to 0, and the history names the rule.
  - The GM sets 0.7 and it stays, with the tag still on.
  - Damaged goes to 0.5. A lower GM value stays.
  - A scan of the pack's triggers and programs finds no condition that reads a `tag.*` field as a value.
- `traveller-codes` golden: only the pack hash moved.

## Next

- Programs that read the values (a patrol's reaction time from discipline, a `depart` refused below a readiness threshold) come with the first ship programs (J6 roles).
- The Ships and stations card shows facts from the scene. Kept values (crew, readiness, discipline) should show there too, from the record, at the depth in force.
- The status group could move to a base pack once a second game pack needs it.
