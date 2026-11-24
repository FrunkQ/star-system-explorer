# RS4 (construct half): split and join at a dated event, and NM4 names for V3 worlds

Kernel track, 24 Nov 2026. This is a landing run-through piece. The travel thread owns the flight log half (`record.flight`, `log_flight`, `flight_log`). It writes its own Split and Join entries, using the address returned here.

## What was built

**Parts.** A construct's parts are its facts named `part.<id>.<field>`, with each part's mass in `part.<id>.mass_kg`. A part might be a stage, a lander, a service module or a cargo pod. The whole construct's mass is `construct.mass_kg`, when it has one. This is one general shape for any construct, from any pack. Nothing in it is specific to a game.

**Split.** `split_construct {address, key?, name?, parts?, mass_kg?, at_s}` (in `sse_trigger::split`):
- It makes a new construct of the same kind, under the same parent. Its key is `key`, else the next free number.
- It moves the named parts' facts to the new construct, together with their mass plus `mass_kg` more (for example, propellant left in a stage).
- It writes `split.from`, a `name` (the one given, else the old name and the first part) and `tag.construct` on the new construct.
- It lowers the old construct's `construct.mass_kg` by what moved.
- One authored `construct.split` event writes both constructs and unsets the moved facts on the old one. Before that date nothing has moved.
- **Refused by name:**
  - an unknown part (the message lists the parts there are);
  - more mass than the construct has;
  - a key that is taken;
  - nothing to move;
  - a construct that is retired or not there.
- Reply: `construct_split {address, event_id, state_hash}`.

**Join.** `join_constructs {address, with, how?, at_s}` writes one `construct.join` event. `how` is one of:
- `dock`, the default: both constructs stay, and `with` carries `docked.to`.
- `undock`: removes `docked.to`.
- `merge`: `with`'s parts and mass move across, and `with` is retired with `joined.into`. If both constructs have a part with the same id, the merge is refused.

Reply: `constructs_joined {how, event_id, state_hash}`.

A construct made by a split can be retired like one made from a template. `retire` now accepts `split.from` as well as `template.id`.

**NM4: names for V3 worlds.**
- A V3 adoption has no kind facts. A body now takes its catalogue kind from V3's `roleHint` (star, planet or moon). Failing that, the kind is read off the facts: a body with a temperature is a star, a body round a star is a planet, and a body round a planet is a moon. A planet gets its Roman numeral ("Sun I") instead of "{parent} {n}".
- The save's own name, `v3.name`, is the common name.
- `spec_from_facts` names NAV bodies by the typed or authored common name, so `ship_at_world` and the voyage reports say "Earth", not an address. Lookups by address still work, because NAV matches a body by address or name.

## Checks
- `cargo test -p sse-kernel split_tests`:
  - `a_split_moves_the_part_and_its_mass_at_its_date`;
  - `dock_undock_and_merge`.
- `cargo test -p sse-kernel an_adopted_v3_world_keeps_its_name_and_its_kind`.
- Contract cases at the end of `tests/contract/requests.json`: a freighter "Columbia" with an "eagle" part is split, docked and merged.

## Golden (on purpose)
`tests/golden/contract.jsonl`:
- line 1: the op list gains `split_construct` and `join_constructs`;
- line 41: the unknown-op message names them;
- the seven new cases at the end;
- nothing else: the V3 adoption in the contract cases reads no names or NAV bodies.
