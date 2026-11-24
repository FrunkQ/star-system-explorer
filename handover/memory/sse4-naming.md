---
name: sse4-naming
description: SSE4 naming design (track NM, accepted for building by Alex 22 Nov 2026): name schemes, name layers, names follow parent, packs on Explorers
metadata:
  type: project
  modified: 2026-11-22T16:31:22.300Z
---
Design file /mnt/project-files/sse4/SSE4-naming-design.md, repo copy handover/design/, PR 196, track NM in the build plan. Thread "Design naming and language packs" (Fable, 22 Nov 2026).

Alex's own words, 22 Nov:
- Names today are "very English, not alien, and not scientific"; wants naming packs under Reference: scientific designations, English-sounding, "international English" (many Earth nations), and alien languages with pronunciation; GM default for the universe, overridden per system or planet; simple text files, on Explorers.
- 16:25: the user can always rename anything by hand; "HD102 I, HD102 II" also become "Mystar I, Mystar II" when the star is renamed, recursively where appropriate.
- 16:26: "just implement - not much to review - we can tweak later", as long as packs are "available from store" and can be included in world/universe packs. So section 9 is decisions made by Claude, not rulings; all changeable.

Shape: crate `sse-names`, pack data `provides.names`, built-in pack `base-names`. Five makers (list, sounds, number, pattern, mix), later learned and meanings. Three layers per object (designation, common, local), each a dated fact per audience. Settings inherit down the address tree. A name whose form mentions its parent is stored as the form, so it follows a rename; a typed name is plain text and stays. Names are keyed draws on their own stream, fixed on promote, never read by physics. Real languages use checked word lists only, never joined syllables. Plain-text "name sheet" is a second way in beside JSON. Default planet form is Roman numerals by orbit.

Packages NM0 to NM11 (design section 7); order NM0, NM1, then NM2 and NM3a, then NM4. NM9 (Explorers Names shelf) needs Alex's own yes if it touches the site database. NM6a is a mockup by a Fable thread that does not block the build.

State (22 Nov 2026 evening): NM1 engine done (PR 207, kernel thread; schemes live in provides.other.names; op name_sample). Data sheet thread built NM3a (PR 208: catalogue, catalogue-letters, avoid list), NM3b `english` and NM3d vesh/oluen (PR 209), NM3c Many nations (ten languages from NorthEuraLex and WOLD, CC BY 4.0; mixes many-nations, -speakers, -agencies; clump sys, 10% strays; no speaker review yet). The V3 parity row is written from the trailer thread's read; gaps for NM4: common-name chance by rarity and habitability, belt, ring and barycentre forms, and a Bayer-style scheme. Next is NM2 (kernel).
Gotchas: scheme labels may not hold ":"; the avoid list matches substrings ("shit" in romanised words); wiktionary is blocked by the proxy, raw.githubusercontent works; scripts/fetch-name-sources.py needs the network, and its DROP lists rows left out with reasons.

Related: [[sse4-matter-pack]], [[sse4-tags-design]], [[explorers-paid-packs]]

