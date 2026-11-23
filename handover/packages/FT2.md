# FT2: technology on a civilisation, and the availability reading (kernel)

Track FT (`design/SSE4-future-tech-design.md`, sections 5 to 8). Built 23 Nov 2026 by the kernel thread on FT1's `packs/base-tech`.

This is the lighter FT2 Alex asked for on 23 Nov: in 4.1, alien life carries culture and technology, so 4.0 gets a lighter touch. Level, exceptions and advances are plain dated data, set by hand or by a pack and inherited down the tree. "Develop it" is one dated event. There is no research mechanic, no automated tech tree and no hook for one.

## The record (`sse-record/src/tech.rs`)
- **What is written.** `record.tech` is one event, either on an actor or on the universe root (the setting). It sets any of:
  - a `level`;
  - `gain`: advances held early ("Develop it");
  - `lose`: advances never taken up.
- **How it folds.** `tech_of(holder, at)` folds each holder's events in time order. A later gain undoes a loss of the same advance, and the other way round. It then walks from the holder up its parents to the root, and the nearest holder that says a thing wins.
- **What comes back.** Each part of the answer carries `by` (who said it), `since_s` and `reason`. So a fleet shows "inherited from the Vesh", and a setting-wide level shows as the root's.

## The kernel (`sse-kernel/src/tech.rs`, ops)
- **Loading.** `provides.other.tech` is read at load:
  - figures, with `better` and their marks; a ceiling must name its law;
  - the top pack's ladder;
  - advances, with what they set, their home level, their part and their mark.
  - An advance that sets an unknown figure is refused.
- **Who holds an advance.** A civilisation holds an advance at or below its level unless it lost it, and any advance it gained.
- **The size rule.**
  - An advance `n` levels above the builder: `size_per_level_short`ⁿ.
  - An advance `m` levels below the builder: `shrink_per_level_beyond`^min(m, `shrink_levels`).
  - The powers are written out as repeated products.
- **The reading.** It reads a design's `parts: [{part, needs: {figure: value}}]` through a civilisation. Each need reads one of:
  - `can build`, by the lowest-level held advance that meets it, with its shrink;
  - `short`, naming the lowest-level advance on the ladder that meets it, `levels_short` and the prototype's `size_factor`, in words such as "2 levels short: Woven carbon cable (a prototype 100 times the size)";
  - `past known`, with `times_their_best` and words such as "needs 31905 times their best strength for weight".
- **Fictional by ceiling.** A need past a figure's ceiling carries `fictional`, naming the law, for example "breaks chemical bond energy", and the advance's chip turns fictional.
- **Overall.** The design reads as its worst need. The reading also gives `part_sizes` (each part's largest factor) and `leans_on` (one chip per advance, inked built, not yet built or fictional).
- **Ops.**
  - `set_tech {holder, level?, gain?, lose?, writer?, reason?, at_s}` returns `tech_set`. It refuses an unknown advance, a level off the ladder, and a holder that is neither an actor nor the root.
  - `tech_of {holder, at_s?}` returns `tech_read`, which includes `holds`.
  - `tech_reading {holder, parts, at_s?}` returns `tech_availability`.

## Checks (`sse-kernel/src/tech_tests.rs`)
- **Inheritance.** A level set on the setting reaches a fleet two actors down. A loss set on the Vesh is inherited, with who said it. Bad advances, bad levels and bad holders are refused.
- **The reading never says no.**
  - PBO fibre at level 4: can build.
  - An Earth beanstalk's 3 × 10⁷ J/kg: 2 levels short, 100 times the size, not yet built.
  - "Develop it" dated 2310: the reading is `can build` after that date and `short` before it.
  - A ringworld's 1.5 × 10¹² J/kg: past known, fictional by the bond ceiling, and about 32,000 times their best.
- **Size beyond the home level.** At level 6, a chemical rocket (home level 3) shrinks to 0.25: half a level, for at most two levels. An unknown figure is refused.

## Golden
- `tests/golden/contract.jsonl`: the same "unknown op" line as OV3, which now also lists `set_tech`, `tech_of` and `tech_reading`.
- Nothing else moves. Nothing in physics reads tech.

## Next
- Design section 5.2, proposed on 23 Nov and still waiting on Alex's questions 15 to 18, replaces the flat size rule for advances that have a part family. Being short of the home level would worsen the advance's own figure by the family's maturity default, floored at the best older advance held, and the sizing block would size the part. The flat multiplier built here stays as the second path, for fictional and unknown devices.
- Ship needs from the ship record's own figures, so the travel thread's ship generator passes its parts. This is a small follow-up when that thread asks.
- FT7 with the Traveller pack thread: the Traveller TL ladder as a setting ladder.
- "Show it without" belongs to the renderer and ship generator. A setting's own fictional advances in the fudge register ("setting fiction") are FT3's.
