# MC6: the ship's log says what each Build change changed

Track MC. Owner thread "Build ship manifest and crew roster". Alex, 24 Nov 17:54: "The log record all flights as well as cargo transfers and refits?" Transfers were already in the Log (MC5). Refits showed only "Facts edited". Flights are MC7, read from the travel track's flight log.

## What it does

- Kernel: the `history` op takes an optional `changes: true`. With it, each event summary carries `changes`, the facts it set or unset at exactly the asked address (`FactChange {field, value, unset?, large?}`). A value over 4 KB of JSON, such as a picture, is named but not sent (`large`), so a history stays small. Without the flag the answer is unchanged, so the contract golden is unchanged.
- `session.history(address, { changes: true })` passes it through, and `HistoryEvent.changes` holds the result.
- `shipLog` walks the events oldest first and keeps each fact's value so far, so an edit reads like this: "Changed: Jump drive 2 → 3; Name Kestrel → Kestrel II; Design changed". The rules:
  - Numbers, short words and yes/no are shown.
  - Designs, pictures and long text are only "changed".
  - A cleared fact reads "cleared".
  - An event without changes keeps the old "Facts edited".
- `factEditWords` (constructs.ts) can word any other log's edits (a body's, later).
- `MANIFEST_WORDS` names the picture fields: Pictures, Opens on, Design.
- The Ships and stations card asks for changes when it loads the Log.

## Checks

- Kernel test `a_history_can_name_what_each_edit_changed_at_one_address` (withdraw_tests.rs): only the address's own facts are given, a picture-sized value is marked large, and plain history has no changes.
- `tests/ui/manifest.test.ts`: from → to across a refit, with the design and pictures "changed" and the old words kept without changes.
- `tests/ui/manifest.e2e.mjs` (by hand): Kittiwake's log names the design, picture and opens-on changes made on Build.
- No golden change.
