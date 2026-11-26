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
- `tests/ui/party.e2e.mjs`: the card opens from the header and closes again. While the table has only heard of Beowulf, the card draws the ship and its tanks read "not known". After the GM shows Beowulf to the table (Known to on Build), the tanks read their levels and the players' screen stays live. Screens: project files `sse4/handover-screens/MC8-screens/`.

## Known

- Fixed 26 Nov by travel's PR 452 (record writes go through the logged intents that players replay) and the kernel's PR 546 (stacked packs merge their data views, which ended the `data view "rumour" is not defined` fault). The reveal is now checked end to end in party.e2e. The card's fallback to the ship's system stays, for screens whose root scene cannot be read.
