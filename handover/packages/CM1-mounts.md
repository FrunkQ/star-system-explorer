# CM1: the mount (compound ships)

Track CM. Design: `handover/design/SSE4-compound-ships-design.md` sections 2 to 5 and 13, accepted by Alex 27 Nov 2026 00:43 UTC, with his rulings in section 14A winning. Built 27 Nov 2026 by the thread "Build mounts for compound ships".

## What it is

One construct carried by another, with two rules: **capture** (`mount_construct`, record event `construct.mount`) and **release** (`release_construct`, event `construct.release`). One mechanism for stages, landers in adapters, fighters in hangars, strap-on boosters, a Soyuz at a station port, a rover in a surface base's garage (ruling 14A: a land structure is a construct like any other) and a sky crane's tethered rover.

- `crates/sse-trigger/src/mount.rs`: the facts, the five places (`front`, `behind`, `inside`, `hull`, `outside`), bonds (`fixed`, `latch`, `rest`), lines (`propellant`, `power`, `air`, `people`), ports and bays, `mount_of`, `carried_by`, `carriers_of`, `compound`, `mount`, `release`.
- Ops in `crates/sse-kernel/src/protocol.rs` and `lib.rs`; replies `construct_mounted` and `construct_released` carry the mount, the compound the carrier is in after the event, and notes.
- RS4 (`split.rs`): a dock refuses a mounted construct, a merge refuses while the merged construct still carries anything and unsets its mount facts. `docked.to` reads as a latch on the hull, and `release_construct` releases a dock, so saves and pack logs keep working with no migration.

## The facts

On the carried construct: `mount.on`, `mount.place`, `mount.bond`, `mount.lines` (always written); `mount.port`, `mount.own_port`, `mount.bay`, `mount.station.along_m`, `mount.station.clock_deg`, `mount.station.stand_off_m`, `mount.push_m_s` (when given). A release unsets exactly these.

On a carrier: ports `port.<id>.place`, `.size_m`, `.lines`; a bay is a part with any of `part.<id>.bay_length_m`, `bay_radius_m`, `bay_door_m`, `bay_holds_kg`. On a carried construct, when known: `construct.length_m`, `construct.radius_m`.

## What is checked, and what is only noted

Refused, each with the reason in words: an unknown place, bond or line; a construct carrying itself or what carries it (no loops); one already carried or docked; a later mount or release of the same construct (mounts are written in date order); station numbers on a place that has none (along and clock are for hull and outside, stand-off for outside only); a bay named on a place other than inside; a missing, occupied or misplaced port, a line the port does not pass, ports of different sizes; a second construct at an unported front or behind; a carrier with several bays and none named; a carried construct too long, too wide or too broad for the door of its bay, or a bay load over `bay_holds_kg` (everything already in the bay with what it carries, plus the newcomer).

Noted and made (steer, never stop): inside a carrier with no bay part; a size or a mass not known, so not checked.

Nothing moves at a mount or a release: every construct keeps its own mass, parts and kept values. The compound (root, members, total mass, members with no mass) is read at a date and never stored. The test `release_and_capture_move_nothing` checks the sum of the constructs' masses at six dates across a release, a release and a capture. CM4 is the absolute conservation check with propellant.

## Left for the next packages

- The separation push has no default here: the mount keeps `push_m_s` when given, and the release reply notes when none was. The pack value (design 4.1, 0.5 m/s, unverified) is read by CM2 when it seeds the released construct's flight log.
- Capture here is authored (the GM says it happened). Reach and speed (`capture.reach_m`, `capture.speed_m_s`) against both flight logs, and the rendezvous offer, are CM2.
- Lines are declared, not used: transfers across them are CM3.
- Compound templates (making a stack and its mounts in one event) are CM11.
- Bond defaults per place (fixed on the axis, rest in a bay, latch on the hull and outside) are a reading of the design's words, not physics. A nose docking port gives `fixed` unless `bond: "latch"` is passed.

## Goldens and gates

- New area `compound` (`tests/contract/areas/compound.json`, `tests/golden/areas/compound.jsonl`): the Apollo stack (CSM root, S-IVB behind, Eagle in its adapter), the refused loop, transposition and docking at the nose port, and an RS4 split, dock and release of a dock. Only this area's golden is new. The core `contract.jsonl` does not move: its op lists are masked and checked against `ops()`.
- Reach ledger (`handover/packages/ER-reach-ledger.txt`): `mount_construct` and `release_construct` are `quiet` (they read no physics); `split_construct` and `join_constructs` now have an area case and move from `untested` to `quiet`.
- `packages/ui-contract/src/perf-chunks.json`: the two ops join the chunk with split and join.
