# NM3c: Many nations in base-names

Track NM, design `handover/design/SSE4-naming-design.md` 4.8 C and decisions 7 and 8. Built by the master
data sheet thread on 22 Nov 2026. Alex said "just implement ... we can tweak later", so every choice here can
change.

## What was built

- **Ten schemes, `nations-en` to `nations-sw`**, one for each of the design's first languages: English,
  Mandarin, Hindi, Spanish, Arabic, French, Portuguese, Russian, Japanese and Swahili.
  - Each is a List of real words, 96 to 193 per language. They cover nature, sky, land, water, weather,
    plants, animals, colours, seasons and a few warm abstract words (joy, story, gift, friend).
  - Every row carries its meaning, its language, its own script (where that is not Latin), its IPA and its
    source.
  - No real language is made by joining syllables.
- **Three mixes.** `many-nations` weights the languages evenly and is the default.
  `many-nations-speakers` weights them by Ethnologue 2023's total speakers, rounded.
  `many-nations-agencies` uses the languages of agencies that had sent a craft to the Moon or beyond by 2025,
  and is authored.
  - Each mix clumps by system, with 100 strays per thousand. A system is named in one language, with about
    one name in ten from another.
  - The weights are data, so a GM can edit them.
- **Features** put an English term round the word ("Mount Nisr", "Lake Mizuumi", "Bīng Sea"), the way
  international maps do. Design 4.8 C rule 4 keeps a language's own place-name grammar out until it has a
  cited grammar or a reviewer who speaks the language.

## Where the words come from

`scripts/fetch-name-sources.py` downloads two CC BY 4.0 datasets, each pinned to a release tag. It writes
the chosen concepts to `packs/base-names/sources/many-nations.tsv`, a file of 1,413 rows that is kept in the
repo. `scripts/build-base-names.py` reads that file with no network and romanises each word.

- **NorthEuraLex 0.9** (Dellert et al. 2020), CLDF v4.1: en, es, fr, pt, ru, hi, ar.
- **WOLD** (Haspelmath and Tadmor 2009), CLDF v4.2: sw, ja (already in Hepburn), zh (numbered pinyin, which
  becomes tone marks).

The romanisation is ours, and each scheme's `about` names its rule:
- Russian: BGN/PCGN, without the soft and hard signs.
- Hindi: Hunterian from the IPA, with long vowels marked.
- Arabic: close to ALA-LC from the IPA.
- Mandarin: pinyin with tone marks, and apostrophes before syllables that open with a vowel.

## Left out on review (`DROP` in the fetch script, each with its reason)

- **Wrong senses in the source.** The Hindi "wind" (पाद) means flatulence. The Arabic "birch" means rods.
- **Unfortunate words.** The Arabic "grass" (حشيش) also means hashish. "Egg" is crude slang in Russian,
  Spanish and Arabic, so that concept is gone everywhere. "Pleasure" is dropped in es, fr and pt.
- **English loans, phrases and rare Sanskrit forms**, such as Hindi बेरी "berry" and Japanese 初めて "for the
  first time".
- **Hits on the avoid list.** Three romanised forms hold an English swear word: Arabic "winter", Japanese
  "wetland" and Mandarin "stone". English "apple" is a brand.
- **Fixed by rule.** Phrases, hyphenated compounds and Swahili adjective stems ("-pya") are left out. Where a
  row gives several spellings, the first stands.

## Not here, or known

- No language has been reviewed by a speaker. Every scheme says "first pass, corrections welcome". The
  reviewed mark waits for a reviewer and the Explorers report route (NM9).
- Russian lost ё in the source (лед, not лёд), so it romanises as "Led". Arabic case endings and tanwin are
  handled by rule, not by a dictionary.
- English words can repeat their feature term ("Lake Lake"). This is harmless but worth tidying when forms
  learn to skip a word equal to their term.
- IAU NameExoWorlds names (CC BY 4.0) would add proposed names by country. Their meanings are only in the
  downloadable sheets, so they wait.

## Checks

- `base_names_many_nations_rows_are_sourced`: every row has a meaning, a language and a source. Non-Latin
  languages have their own script and IPA. No row is on the avoid list, and each scheme says it is a first
  pass.
- `base_names_many_nations_clump_by_system`: across 50 systems of 10 planets made as siblings, a 500-world
  campaign, no system repeats a name. Strays are 3 to 20% of names, set at 10%. The agencies preset never
  draws a language weighted 0.

Goldens: none change.
