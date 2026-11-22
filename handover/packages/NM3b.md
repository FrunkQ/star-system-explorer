# NM3b: the English-sounding scheme in base-names

Track NM, design `handover/design/SSE4-naming-design.md` 4.8 B. Built by the master data sheet thread,
22 Nov 2026. Alex: "just implement ... we can tweak later", so every choice here is changeable.

## What was built

- **Scheme `english`** in `packs/base-names` (written by `scripts/build-base-names.py`). Names are built the
  way English place names are: a first element and an ending, each a real Old English or Norse place-name
  element (Ash + ton, Thorn + ley, Brock + hurst). Sometimes Great, Little, North, Upper and the like go
  before it.
- **Every part carries its meaning** on the card ("Brockhurst": badger + wooded hill). The list rows have
  `means` and `source`, and the engine puts both on each `Part`.
- **Sources.** The glosses are our own short wording of well-known meanings, citing Blackie 1887
  (*Geographical Etymology*, public domain, Project Gutenberg #69018). NM0 found that Smith 1956 and the
  Nottingham place-name survey are under copyright, so nothing is copied from them.
- **Size.** There are 136 first elements, 61 endings and 14 words that go before, which gives about 8,300
  plain words and 1.2 × 10^5 names with a word before. No pairing makes a triple letter: Kers (as in Kersey),
  Mil (Milton), Wel and Nes are the real spellings that avoid it.
- **Forms by kind.** Peaks are "Mount …", "… Pike" and "… Fell". Ranges are "… Hills", "… Fells" and
  "… Range". There are also "Lake …" or "… Water", "Sea of …", "… Island" or "… Isle", "Port …",
  "… Edge" or "… Scar", "… Gorge", "… Deep" and "… Ice".
- **Craters** may take the surname of one of 36 long-dead British astronomers and explorers, with their
  dates on the card (Halley, Herschel, Somerville, Flinders, Kingsley and others). This follows the IAU rule:
  dead for many years, and known for science or travel rather than politics, war or religion.

## Not here

- Settlement names that follow a feature next to them ("Brockhurst Bridge") need the parent's feature, not
  its body, which the kernel's request does not carry yet.
- The list is a first pass and is marked so in the scheme's `about`. Corrections are welcome.

## Checks

`base_names_english_means_what_it_says` (sse-names) runs 1,000 draws for each of six kinds:
- none is on the avoid list;
- every listed part has a meaning and a source;
- planets give more than 950 unique names in 1,000.

Goldens: none change. The names never feed physics.
