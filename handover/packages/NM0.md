# NM0: naming sources, licences, V3 and clash odds

Track NM, design `handover/design/SSE4-naming-design.md`, section 7 (NM0) and section 8. Alex asked for it to be
built and tweaked later (22 Nov 2026, 16:26). This package belongs to the master data sheet thread.

## Sources and licences

All eleven sources were checked. They are in `docs/citations/naming.md`. What the later packages need to know:

- **Planet letters go by discovery order** (IAU). The design already has it this way: Roman numerals by orbit
  are the default, and letters from b are the real-practice form. Circumbinary planets are written **(AB)b**
  (Hessman et al. 2010), and NM3a follows that.
- **Hessman et al. 2010 is an arXiv preprint** (arXiv:1012.0707), rejected by A&A, with no DOI. It is cited as a
  preprint.
- **The IAU star-name list has no culture column** (451 names in the 2021 file). A Many-nations or night-sky
  pack that wants a name's origin has to add it by hand, from the WGSN bulletins.
- **The English place-name element sources are copyrighted** (Smith 1956, and the Nottingham Key). NM3b writes
  its own glosses and cites Blackie 1887 (Project Gutenberg #69018, public domain).
- **The licences allow shipping with a credit**:
  - IAU material: CC BY 4.0.
  - USGS gazetteer and USNO catalogues: US Government works.
  - Unicode data: Unicode License V3.
  - GeoNames: CC BY 4.0.
  - Natural Earth: public domain.
  - Wiktionary: CC BY-SA 4.0, kept as a separately licensed data file.
  
  `packs/base-names` carries a NOTICE section per list.
- The exclusion rules for generated and suggested names come from the IAU and from Gazetteer rules 9 and 10.
  They bar living people, people dead less than three years, political, military or religious figures
  (political figures before the 19th century are allowed), commercial names, and pets. They go into the
  Many-nations and English-sounding lists as a check on every row.

## V3

V3 is only on Alex's machine. The trailer thread, which can reach it, is reading how V3 makes names (read only).
Until its summary arrives, the `docs/v3-parity.md` row says "not yet read".

## Clash odds

A scheme with N distinct names gives about n²/2N clashing pairs among n names drawn at random. Once n names are
fixed, a share n/N of the next draws clash and have to draw again (design 4.4). "Runs out" means the scheme has
fewer names than are asked of it.

| Scheme (sizes as planned) | N | At 10^4 names: pairs, share of next draws | At 10^6 names |
|---|---|---|---|
| Catalogue (counters, numerals, letters, coordinates) | unbounded | none: unique by construction | none |
| Today's surface syllables (`base-surface`: 30 syllables, 2 or 3 joined) | 2.8e4 | 1,790 pairs, 36% | runs out |
| English-sounding (about 120 first elements × 60 endings × 4 forms) | 2.9e4 | 1,740 pairs, 35% | runs out |
| Many nations, words alone (10 languages × about 500 words) | 5e3 | runs out | runs out |
| Many nations, word plus a feature term (about 20 terms) | 1e5 | 500 pairs, 10% | runs out |
| Alien Sounds (75 consonant-vowel syllables, 2 to 4 joined) | 3.2e7 | 2 pairs, 0.03% | 15,600 pairs, 3% |

What this means for the build:

1. **Names are needed per scope, not per universe.** The clash check is among siblings and within a campaign
   scope (design 4.4), and a scope holds tens to thousands of names. Ten to the six is a whole explored
   galaxy, where repeats are realistic (Earth has many Newports), and two can be told apart by their
   designation.
2. **List schemes must not be the only maker for a large scope.** Many nations needs its compound forms and a
   fallback to the Catalogue name when a scope's list runs out. NM3c's acceptance test checks that a
   500-world campaign gets no repeat in any one system.
3. **Today's syllable scheme is too small for whole campaigns.** Its replacement, English-sounding, should
   reach at least 10^5 names, which needs a third slot or the feature forms. NM3b checks this.
4. **Sounds schemes have room enough.** Retries stay rare up to a million names.
