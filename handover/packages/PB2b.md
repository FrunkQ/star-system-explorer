# PB2b: Bookmarks

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Design section 4A ("bookmarks define what").

## What
- `packages/ui-contract/src/bookmark.ts`: `Bookmark` (`sse.bookmark/1`: id, name, scope, at_yr, instrument, scale, view, selected; nothing about players), `makeBookmark`, `bookmarkName` (no repeats), `nextBookmarkId`, add, rename, remove, `moveBookmark` (drag reorder), `updateBookmark` ("Update to where I am"), `bookmarkReachable` (another campaign's place cannot be opened), `bookmarksFrom` (reads a save, drops anything malformed).
- Kept with the campaign: `CampaignSave.workspace.bookmarks`, `Workspace.bookmarks`; autosave watches them.
- `packages/shell/src/run/Bookmarks.svelte`: the button after Redo (every desk; through `chart/Stage.svelte`'s new `extraTools` snippet), key B (listed in Help, Keyboard). Its list opens from the button: "+ Bookmark where I am" (then the name is ready to type), rows that go back on a press, rename (pencil or double-click), update, remove, drag to reorder.
- App's `goBookmark` puts back the selection, rail scale, instrument, ruler date and view controls (re-framing a fixed "Seen from").
- `run/OnAir.svelte`: the on-air chip, in the top bar right of the desk switch on the Run desk; on the status line on other desks; not in a phone's bar (the Switchboard is on the rail).

## Checks
- `packages/ui-contract/test/bookmark.test.ts` (4). `npm run ui:check`, `npm run ui:test`.
- New `tests/ui/bookmarks.e2e.mjs` (in `ui:e2e`): bookmark Tarn, rename, go elsewhere, come back, kept after reload, drag reorder, remove, the chip in the top bar on Run. `switchboard`, `table`, `landing-table`, `shell` pass.
- Screenshots: project files `sse4/handover-screens/PB2b-screens/`.

## Later
- A pack can carry bookmarks (an adventure's key places): read them with `bookmarksFrom` when packs gain a slot.
- Tours (top-level design) are ordered bookmarks with captions.
- The bookmark button is hidden with the tools on a phone.
