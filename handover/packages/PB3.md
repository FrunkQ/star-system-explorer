# PB3: the Playbook (first part)

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Design sections 4 (the Playbook), 4.4 (adding), 4.5 (folding), 4.6 (edit, reorder), 5.4 (press again to go back) and 11 (data shapes).

## What
- `packages/ui-contract/src/playbook.ts`: `Playbook` (`sse.playbook/1`) of chapters, pages, sections and presets. A `Preset` has a title, `to` (a role id or "everyone", never a screen) and steps: `show` (a bookmark id), `hold` (a card), `say` (words to read aloud; nothing sent). Functions to add, rename and remove parts; add, update, move (drag) and remove presets; `presetsInView` for keys 1 to 9; `fire` (every step at once; pressed while up it takes itself back: lift, back or release) and `isUp`; `showOf` (a bookmark's place under the preset's name, with `focus` = the bookmark's selection); `presetWords`; `playbookFrom` (reads a save, drops unknown steps).
- Kept with the campaign: `CampaignSave.workspace.playbook`, `Workspace.playbook`; `Workspace.markHere` (the shell bookmarks where the GM is, for Keep on Live and Update from live).
- `Shown.focus` (seat.ts): the thing a bookmark looks at; `Players.svelte` selects it once per send when its scene has it. Older players ignore it.
- `packages/shell/src/run/Playbook.svelte`: the open page under Live on the Switchboard (picker, sections, rows, + Add, + Section, drag reorder). `run/PresetMonitor.svelte`: a selected preset in the monitor, Edit, Done. `run/fired.svelte.ts`: fired tonight, session only.
- `run/Switchboard.svelte`: Keep beside Send; a preset's dot is a source like Live (drag or press-then-press at any role or screen); lines start at the preset that sent them; keys 1 to 9.

## Checks
- `packages/ui-contract/test/playbook.test.ts` (6). `npm run ui:check`, `npm run ui:test` (545).
- New `tests/ui/playbook.e2e.mjs` (in `ui:e2e`): add from a bookmark, edit in the monitor (nothing sent), fire turns the players' screen to Vault, press again takes it back, Read aloud shows its words and sends nothing, Keep on Live, key 1, fold, a new page, kept after reload. switchboard, bookmarks, table, players, landing-table, party and campaign pass.
- Screenshots: project files `sse4/handover-screens/PB3-screens/`.

## The "just now" strip (second commit)
- `run/fired.svelte.ts` `noteSent` and `justSent`: the last six live sends tonight (the Switchboard's Send and dots, the status line's Send this view, Again). Each row has Keep (a bookmark of that place and thing, then a preset for the role it went to, on the open page, open in the monitor) and Again. playbook.e2e covers Keep and Again.

## Not yet (the rest of PB3)
- Reveal steps with the eye; A landing as an add kind.
- On players' screens a Show selects the bookmark's thing but does not yet change their instrument or view (needs layouts, PB13).
- A pack's playbook as extra chapters (`playbookFrom` reads it; packs need a slot).
