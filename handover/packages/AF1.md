# AF1: Start fresh, and new features you can find (alpha aid)

27 Nov 2026. Asked by Alex: "when i look online i am not seeing a lot of the new functionality - is that because i am by
default loading a legacy data set that does not enable things? Do I need a local 'clear all' easy access button to
start a fresh view of the app?" and then "just during alpha - at beta we can calm down".

**Revisit at beta.** The Start fresh door on the start sheet and at the top of More is an alpha aid. At beta, move it
back to Campaign › Your data (where Clear all data lives) or tone it down.

## What was found

- The live site was current (code from PRs merged up to about 10:40 UTC was in the deployed bundle). No service worker,
  no Cache Storage; `index.html` is `max-age=0, must-revalidate` and assets are content-hashed. Not a stale build.
- **A kept Solar System is frozen at the sheet it was entered from.** "Open the Solar System" enters `packs/base-sol`
  as catalogued edits only when Sol is not yet adopted; after that its values live in the campaign's chronicle. A
  campaign that opened Sol before 26 Nov 20:31 (RW7a, Saturn's small moons and J2) never gets them.
- **Carry on opened the base system.** With a kept campaign the start sheet's primary door was "Carry on: Kestrel",
  and it opened Kestrel whatever the campaign was last looking at. A GM working in the Solar System came back to the
  fictional sample, where none of the Sol pictures can show.
- **Most new pictures need True.** The default view style is Chart (`DEFAULT_VIEW`); prominences, ring gaps, shadows,
  glint and the picture clock's motion are True only. Asked Alex whether a fresh visit should open on True (decision
  card in the thread); not changed here.
- Per-device choices (view style, desk, depth, lenses, start sheet skip) persist by design (TM6). Clear all data
  existed but four presses deep (More › Campaign… › Your data › Clear all data…).
- Not in the app at all yet, so no reset can show them: GZ12's "Make this home" (kernel op only, the button waits on
  the galaxy charts thread); CO1 Trojans (only in grown systems, nothing for Sol as measured); Sol today (CM0) needs
  the pack file built and opened, it is not published.

## What changed

- **Start fresh** (`App.svelte`, `StartSheet.svelte`, `your-data.ts`): one press from the start sheet's alpha box or the
  top of More opens a dialog with two choices, each saying what it clears and needing a second press.
  - Reset settings: forgets every `sse4.*` entry except the campaign, its set-aside copy, the opening marker, recent
    places (so Carry on still goes back) and agent keys (`resetSettings()`).
  - Clear everything: the existing `clearAll()` (campaign, set-aside copy, kept packs, media, settings, keys), with
    Download a backup beside it and "It cannot be undone" in the words.
  - Both reload without the place in the address bar, so the visit opens at the start sheet.
- **Old Sol named**: the campaign save keeps `workspace.sol_sheet`, an FNV-1a fingerprint of the sheet Sol was entered
  from (`solFingerprint`, `ui-contract/src/sol.ts`). A kept campaign holding Sol from another sheet (or from before
  the fingerprint) shows a note: entered from an older catalogue, with Start fresh beside it.
  Fudge (alpha): the right fix is a top-up that enters only the changed catalogue values and keeps the GM's own edits.
  It needs stable addresses for the sheet's rows; `solBatches` numbers bodies in sheet order (`body:N`), so a moon
  added mid-sheet shifts every later address. Note to improve: give sheet rows stable ids in their addresses, then
  top up on reopen.
- **Carry on** names where the campaign was last looked at ("Carry on: The Solar System") and goes back there once the
  campaign has opened (the newest remembered place).
- **What's new**: an entry for this week saying where each new thing is (True view first). The dot on More lights up
  for every device that has not read it.

## Checks

- `npm run ui:check` clean; `npm run ui:test` 1180 pass.
- New `tests/ui/fresh.e2e.mjs` (CI line added): start sheet offers it; opening Sol keeps the fingerprint and no note;
  a changed fingerprint shows the note after the reopen note; Reset settings forgets depth and view, keeps campaign
  and keys; Carry on reads and goes to The Solar System; Clear everything leaves no campaign to carry on and no keys.
  Passes in Chromium here (Firefox not installed in this container; CI runs both). Goes red without the change (no
  `[data-start-fresh]`; Carry on read "Kestrel").
- `campaign.e2e.mjs`, `start.e2e.mjs`, `desk-memory.e2e.mjs`, `walk.e2e.mjs` pass; press budgets unchanged.
- No Rust changed, so cargo checks were not rerun.

Screens: project files `sse4/handover-screens/AF1-screens/`.
