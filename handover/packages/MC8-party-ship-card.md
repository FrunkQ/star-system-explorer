# MC8: the party's ship on a players' screen

Track MC. Owner thread "Build ship manifest and crew roster". Open question 2 (players see a ship's pictures once it is shown to them) was built on the recommendation, yes. The players' side thread agreed the mount on 24 Nov.

## What it does

- `PartyShipCard` (views/manifest) opens from the "Aboard Beowulf, at Mars" line in a players' screen header. That line is now a button, and `data-party-ship` / `data-party-at` are unchanged. It shows:
  - the ship's picture, on the view the GM chose (the same Pictures panel, read only);
  - the crew and the manifest the table knows of;
  - the fuel tanks.
  
  It changes nothing.
- It reads from the seat's own scene, so the kernel has already left out what the table does not know:
  - Known to, and each fact's tier;
  - a tank whose level is still to be learned reads "not known", never full.
- People and cargo live at the universe's root, so the card asks for the root scene. If that can't be read, it falls back to the ship's system, and says who is aboard cannot be read on this screen.
- A seat's scene carries no actor kinds, so `seenAboard(subjects, ship)` (ui-contract manifest.ts) tells them apart by their facts:
  - a person: role, kind of being, state, age;
  - a lot: what, amount, unit;
  - neither: listed as "aboard".
- A sealed screen has no kernel, and the card says so.
- Nothing secret travels in host state `party`; the card reads only what the scene discloses.

## Checks

- `tests/ui/manifest.test.ts`: `seenAboard` sorts people, lots and the rest; leaves out what is elsewhere or not disclosed; and shows a state.
- `tests/ui/party.e2e.mjs`: the card opens from the header and closes again. With Beowulf not shown to the table, it says the table knows nothing more and shows nothing of it. Screens: project files `sse4/handover-screens/MC8-screens/`.

## Known

- **Blocked, reported 24 Nov (coordinator):** a players' screen cannot follow a campaign once the GM has used a Known to eye. Replaying that `set_known` gives a different state on the replica, so the screen asks to reload. The kernel alone is deterministic with or without the record, which points at the observe setup the replica builds. Until that is fixed, a known ship and its shown crew can't be checked end to end on a players' screen.
- A replica's root-scope scene can fail with `data view "rumour" is not defined`. The card then falls back to the ship's system.
