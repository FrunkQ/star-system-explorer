# PB6: Read aloud on screens

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Design section 4.3, question 9 (accepted: words stay on the GM's monitor unless "also show on screens" is on).

## What
- `playbook.ts`: a Read aloud step may carry `on_screens: true`. `wordsOnly(p)` (nothing to any screen, so its button says Read and marks it read) and `sayOf(p)` (the words that go on screens). Firing puts the words over the picture: with a Show step they ride on the bookmark's picture; alone, they go over what the destination shows now, as a send from the preset, so pressing again takes them back (Back, or Lift for Everyone). Saved and read back.
- `seat.ts`: `Shown.say?: string` on the place variant. Older players ignore it and keep the picture.
- Players' page: a plain card of the words over the picture (`[data-say]`), with "Put away" for the player; new words bring it back. Reduced motion: no slide.
- Preset monitor: "Also show on screens" in the editor; the button says Fire instead of Mark read when it is on.
- A delivery for the card (Typewriter) comes with the look transitions work (AT15a / PB10).

## Checks
- `playbook.test.ts`: one new test (8 pass). `ui:check`.
- `playbook.e2e`: a plain Read aloud sends nothing; with the switch on the players' screen shows the words; fired again they go.
- Screenshot: project files `sse4/handover-screens/PB6-screens/read-aloud-on-screens.png`.
