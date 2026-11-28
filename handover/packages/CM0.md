# CM0: make Sol today installable

Track CM (compound ships, `handover/design/SSE4-compound-ships-design.md` section 13, accepted by Alex 27 Nov 2026
00:43 UTC; his question 9: "Make the pack installable first"). Built 27 Nov 2026.

## What was wrong

The design's audit (Part A.3, item 1) found that nothing loaded the pack into a campaign: the only way in was Roll
call's `?spaceflight=<folder URL>` address, which fetched files straight from a folder. Checking it found two more
things:

1. **The kernel refused the pack.** Its calendar carries an `about` line for people (UTC days counted on TT), and the
   kernel's calendar shape refused any field it did not know, so `load_stack` returned `stack_rejected` for the whole
   pack. Seen red in `crates/sse-kernel/tests/sol_today_pack.rs` before the fix.
2. **The pack is a folder, and every pack comes in as one document.** The Open door (a file, a drop, an Explorers
   link) takes one `sse.pack/1` JSON file; Sol today is `pack.json` plus 1,285 data files.
3. **It is too big for local storage.** As one file it is 4.16 MB; the browser holds about 5.2 million characters
   for the whole site (measured in Chromium), and an added pack is part of the campaign's autosave, so autosave would
   have failed with "storage is full" as soon as the campaign grew a little.

## What CM0 does

- **Kernel:** a pack calendar may carry an `about` line, carried and never read (`orbits.rs`, `Calendar.about`;
  skipped when empty, so no existing reply changes; no golden moves).
- **One general bundler, not a special door:** `packages/ui-contract/src/pack-bundle.ts` turns any folder pack into
  one document by holding every file its `provides.other.files` names under `provides.other.contents`, by path;
  `heldFiles(pack)(path)` reads one back. `scripts/bundle-pack.mjs <folder> <out>` runs the same rule on disk. The
  file is what goes on Explorers; it is built fresh from the folder and never committed (no second copy of 4 MB).
- **Roll call reads the installed pack:** `rollcall/feed.ts` reads a pack through one `PackRead`, from a folder
  (`folderRead`) or from the stack (`heldRoll`: the top stacked pack whose crews file it holds). Roll call takes the
  `?spaceflight=` folder first (kept as a shortcut), then a stacked pack, then the campaign's own people and logs, and
  opens again when the stack changes.
- **Big packs kept beside the save:** `campaign-store.ts` keeps any added pack over 256 K characters once in
  IndexedDB (`sse-packs`), by the SHA-256 of its text, and the save in local storage names it with an
  `sse.pack.kept/1` stub. Reopening puts it back (checking the hash) before the kernel sees the save; a missing pack
  sets the save aside with the reason, as any unreadable save is. Save campaign (the download) always carries every
  pack in full. Packs neither the save nor its set-aside copy names are let go after each autosave; Your data's
  "clear everything" deletes the store. Without IndexedDB the pack stays in the save, as before.

## Checks

- `crates/sse-kernel/tests/sol_today_pack.rs`: the pack stacks, the campaign's calendar is the pack's, the `about`
  line is carried; a calendar without one reads as before.
- `packages/ui-contract/test/pack-bundle.test.ts`: every named file is held and reads back equal; the pack's own words
  are untouched; a missing file or a path out of the folder is refused; the script and the rule agree byte for byte;
  the Open door's sniffer calls it a pack, the pack check passes, and the browser kernel stacks it with its calendar.
- `tests/ui/rollcall.e2e.mjs` (second half): the bundled file opened through Campaign…, added, Roll call shows 20 July
  1969 with people aboard from the stack with no folder fetched, the kept save holds a stub and is under a million
  characters, and after a reload the campaign reopens and Roll call reads the pack again.

## Not done in CM0, and why

- **Publishing on Explorers.** Section 13 lists it under CM0; the coordinator's brief for this package said not to
  publish to Explorers. The file is ready to publish (`node scripts/bundle-pack.mjs store-packs/sol-today
  sol-today-0.1.0.json`); the pack's credits and licence (`CC-BY-4.0 AND Apache-2.0`, catalogue data, so free only)
  travel inside it.
- **Constructs for the showcase craft in the campaign.** The pack's logs name bodies by the navigation fixture's
  addresses (`u:sol/sys:sun/body:earth`), while a campaign's Sol is `u:<universe>/sys:sol`, and a construct in a
  campaign is made from a template under a place, then its log written with `log_flight`. Bringing Apollo 11 in as a
  construct needs a rule for which campaign system a pack's Sol is, and a template that can carry its log. That is the
  join CM1, CM2 and CM9 make; making a stand-in now would be a special. Roll call already reads every log from the
  installed pack.
- **Players' screens** receive the stack when they join, so a 4 MB pack goes to each player once; not measured here.

## Part 2 (27 Nov 2026): on Explorers, and fetched into every fresh campaign

Alex, 27 Nov 11:40 UTC: "have it on exploreres BUT also - for now - autoload in SSE - to keep the local executable
smnaller and still have the decent data set for testing. We will test this setup - it may survive into production -
this route gives us more options... i.e. we may ship a smaller version of sol and have a "expansion" on the store".

- `packages/shell/src/documents/stack.json` gains `from_explorers`: Explorers V4 listings fetched into every fresh
  campaign (now `https://explorers.starsystemx.com/api/v4/sse.sol-today-0.1.0`, the hub's slug for id and version).
  Each is checked against its listing's sha256 (`fetchExplorers`, as the Open in SSE4 button does), passes the same
  pack check and goes on the stack like a file opened by hand. A reopened campaign keeps its own stack, so a GM who
  takes the pack off keeps it off. It runs after the chart is up, so a campaign never waits for it; each request has a
  time limit (15 s for the listing, 90 s for the file), and when Explorers is slow, down or has no such pack, a line
  under the chart says the pack was not fetched and the campaign opened without it. The file is not in
  the app's build, so the download stays small.
- An automated browser (`navigator.webdriver`, every browser test) fetches nothing by itself, so tests never reach the
  live Explorers; `tests/ui/rollcall.e2e.mjs` says it is not automated for its own last part, serves Explorers from
  the bundle, and checks one fetch, Roll call reading it, and no second fetch on reopening.
- While Sol today is on the stack, the campaign's dates read in its Gregorian calendar (the top pack's calendar is the
  campaign's).
- Publishing: the hub's own reader (`starsystemx-creator-hub` `src/lib/v4/read.ts`) accepts the bundled file (pack,
  universe, 1,285 held files, well under the 50 MB gate). It is uploaded from Alex's Explorers account, as the hub
  gives a pack id to whoever publishes it first; the hub takes the listing's title from the upload form ("Sol today").

## Part 3 (28 Nov 2026): optional again

Alex, 28 Nov 13:29 UTC: "Loading the history of Spaceflight pack, I think, was a mistake. I didn't realize the basic
version came with a fully worked out solar system. We should keep that and only optionally load the Spaceflight pack
with the history of Spaceflight." This withdraws part 2's autoload.

- `stack.json` loses `from_explorers` and the shell loses the fetch into fresh campaigns: a new campaign opens on the
  built-in Solar System alone and asks Explorers for nothing.
- Sol today stays on Explorers (`sse.sol-today-0.1.0`) and is added only when someone asks: the item page's "Add to
  my campaign in SSE4" link, or the file through Campaign… (Open, or a drop). The Explorers link now has a time limit
  on each request (15 s listing, 90 s file), so a slow Explorers says why instead of waiting.
- Campaigns that already took it while autoload was live keep it: nothing is taken off a campaign silently. It is
  one pack in that campaign's stack, removable with Remove… on the Packs screen.
- `tests/ui/rollcall.e2e.mjs` last part: a fresh campaign in a browser that says it is not automated asks Explorers
  for nothing and holds no Sol today; the Explorers link adds it on the press and Roll call reads it.
