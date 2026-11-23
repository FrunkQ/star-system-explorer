# V3 gap 23 part 1: Describe on the card, kept as a description

Owner: Traveller pack track. Stage 5.

## What it does

- The body card shows **Describe ›** on the GM's chart room wherever Minerva
  can run (`ViewContext.describe`, given by the shell only when the Minerva
  host exists). It opens the Agents panel on Minerva with that subject.
- Minerva is handed only that subject, what it orbits, what orbits it and the
  scene's star (`aboutOne`), from the chosen audience's own scene, as before:
  nothing they have not been shown is sent, and every number in her text is
  checked against those facts. System-wide story hits are left out of a
  description.
- A passed draft can be kept: **Keep as its description** writes a narrative
  note in the record (`session.addNote`, kernel `add_note`, tags
  `["description"]`, `narrative: true`) for the audience it was written for
  ("everyone", or that audience). Notes never change the universe's state hash.

## Checks

- `packages/agents/test/minerva.test.ts`: `aboutOne` picks the right subjects
  and refuses one the audience does not know.
- `tests/ui/minerva.e2e.mjs`: Describe on Tarn, facts hold Tarn and its
  neighbours only, Write, Keep; the state hash does not move.

## Next

- Players seeing a kept description: notes reach seats through the record
  channel ("send the event to ordinary seats of these audiences only"), which
  the p2p replicas do not carry yet (they replay intents). Needs a lane for
  record events to seats (chart room thread with the kernel thread).
- Showing the kept description on the card itself (read from `record` notes
  tagged description, newest first), with Withdraw (`supersedes`, empty text).
