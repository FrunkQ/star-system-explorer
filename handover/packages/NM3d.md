# NM3d: the two invented languages in base-names

Track NM, design `handover/design/SSE4-naming-design.md` 4.8 D. The naming design thread wrote the
languages (22 Nov 2026) as name sheets. The master data sheet thread loads them on NM1's engine.

## What was built

- **The sheets** are `packs/base-names/sheets/vesh.names.txt` (guttural, one click, doubled stops, modifier
  first) and `oluen.names.txt` (soft, open syllables, long vowels, thing first). They are copied unchanged
  from `/mnt/project-files/sse4/names/`. Both are original, and neither comes from a published setting.
- **`scripts/build-base-names.py` reads them** into JSON schemes `vesh` and `oluen` in `packs/base-names`,
  covering:
  - the sound classes with their weights (the reader refuses a class whose sounds and weights differ);
  - shapes, length, never rules, stress ("second from last" becomes `penult`), say-it hints and IPA;
  - meanings and forms (a kind written twice gets two weighted forms);
  - `follow`, for kinds whose form names the parent (Oluen's moon, "Miel Selian").
- Vesh carries the never list the design thread extended for NM1 (`aa+aa`, `k+kh` and the rest).

## Not carried yet

- `replace` (Oluen) and `join` have no engine field. Oluen's never rules already stop the doubled vowels
  `replace` would merge, and both sheets' forms already put the parts in order. NM7 settles the text grammar,
  and the reader follows it then.

## Checks

`base_names_invented_languages_have_their_voices` (sse-names):
- every sound has an IPA symbol;
- each of 1,000 planet names per language comes with its say-it and IPA;
- no Vesh name has a soft sound (l, m, n, s, v, e, o), and no Oluen name a hard one (k, q, z, x, g), so they
  read as two different languages;
- each language gives over 900 distinct names in 1,000.

Goldens: none change.
