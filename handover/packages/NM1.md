# NM1: the naming engine

Track NM (`design/SSE4-naming-design.md`, accepted for building 22 Nov 2026: "just implement, we can tweak later"), built 22 Nov 2026 by the kernel thread. New crate `crates/sse-names`; the kernel loads schemes with the stack and answers `name_sample`.

## Where schemes live
A pack carries its schemes in `provides.other.names = {"schemes": [...], "avoid": [...]}`. The design wrote `provides.names`; the pack format keeps everything that is not physics under `provides.other`, so names sit there like every other pack-data kind. Schemes from every pack in the stack are gathered in stack order; a later pack's scheme with the same id replaces the earlier one. Any problem refuses the stack (`stack_rejected`) with the scheme id and a readable reason.

## The makers (`scheme.rs`, `make.rs`)
- **Forms** per kind (`planet`, `moon`, `peak`, ...) with weights, plus `default`. A form is text with parts in braces.
- **List** `{list:id}`: a row from the scheme's list, each row text with optional meaning, script, romanised form and source.
- **Sounds** `{word}`: classes are one capital letter with weighted sounds; shapes are class strings ("CVE"); length in syllables (at most 8); `never` is a list of sound sequences that may not touch; stress, say-it and IPA per sound. A drawn word comes with its say-it ("KHAZ-rak") and IPA ("/ˈxaz.ɾak/") and the hints for unusual sounds.
- **Number**: `{roman}`, `{letter}` (b onward, as exoplanets), `{Letter}`, `{n}`, `{serial}` and `{serial:N}` (zero padded), `{coord}` (Jhhmm±ddmm from the `ra_deg` and `dec_deg` facts).
- **Meanings** `{means:id}` and `{means:fact?}`: the scheme's word for a meaning, or for the meaning a fact names ("colour" = "red"). A `?` part drops away when the fact is missing. Meaning words joined into one word stay one word ("Khazrak"), and each part keeps its meaning for the card.
- **Mix** `{"mix": {"of": [[scheme, weight]...], "clump": "sys", "strays_permille": 80}}`: one member chosen per clump (all bodies of a system share a tradition), with a set share of strays.
- **Following** `{parent}`, `{parent.short}`, `{system}` are kept in the stored form, not baked in, so `Named::show(parent, system)` gives "Mystar II" today and "Haven II" after the star is renamed. The scheme's `follow` list says which kinds follow.

## Draws
- Keyed, never sequential: `Draws::new(seed, address, "names.<layer>.v1")` with an index base from the first 8 bytes of the scheme's content hash plus `attempt << 32`. The mix uses `names.<layer>.mix.v1`. So a name depends only on the seed, the address, the layer and the scheme's own content: changing any other pack changes no name, and no physical stream is touched (names leave the physical state hash alone; tested).
- Up to 64 tries per name. A drawn part is capitalised when it starts a word and lower case when it joins one.
- **Siblings** (`siblings`) are named in address order. An exact clash, or a near clash (the folded names are one edit apart, at least 3 letters), moves that sibling to its next draw. Numbered names ("Mystar II") skip the near check, since "II" and "III" are meant to be one apart.
- **Avoid list**: a folded whole word, or an entry of 4 or more letters found inside the folded name, is redrawn. The kernel reports how many avoid entries a scheme's own fixed words hit (`avoid_hits`), for the pack author.

## Text safety (`text.rs`, design 4.10)
- Refused: controls, direction marks and overrides, zero-width and other invisible characters, private use, tag characters, ASCII other than letters, digits and ` -'.,()`, more than two combining marks on one letter, more than 64 characters, and spaces at the ends or doubled.
- Composition (NFC) for Latin, Greek, Cyrillic and Hangul, from a generated pair table (`compose_table.rs`, 781 pairs, Unicode 15.1.0, written by `scripts/build-compose-table.py`). A full NFC table is not carried: those scripts are where precomposed letters exist in names. Other scripts pass unchanged.
- The fold for clashes and the avoid list: decompose, drop marks, lower case, map look-alikes (Cyrillic "а" and Latin "a", "0" and "o", ...), drop apostrophes, hyphens and dots.
- Limits: 1 MB per scheme, 50,000 rows per list, 256 items in any one list of classes, shapes or forms, 8 syllables.

## Old settings
`Scheme::from_naming_settings` reads the old base-surface `NamingSettings` (syllable list) as a Sounds maker with one class `S`, so packs and saves made before NM1 still load. NM4 moves the surface feature call sites onto the engine.

## The kernel
- `Observe.names` holds the stack's `sse_names::Library`.
- New op `name_sample {scheme, kind, layer?, count? (1 to 10,000), start?, seed?, parent?, facts?}` gives the digest (SHA-256 of the names joined by newlines), the first 12 names with their parts, faces and text, and the avoid hits. Cost class Light. Sample addresses are `u:sample/sys:{i/10}/body:{i%10}`. It is for the Names page and pack authors; NM2 adds name records on real addresses.

## Choices made while building (open to tweak)
1. The `never` rule works on sound sequences, not spellings. So in the Vesh draft `k` then `kh` gives "kkh" and `aa` then `aa` is allowed ("aaaa"). The Vesh author is told.
2. Near clashes apply to worded names only, not numbered ones.
3. A numbered name never redraws (its number is its identity).
4. The mix clumps by an address segment kind (`sys` by default), the simplest reading of "a system's names come from one tradition".

## Checks
- `crates/sse-names/src/tests.rs` (9): determinism and a 10,000-name digest with an unrelated scheme stacked; Vesh faces and meanings; parent following and renames; siblings; mix clumps and strays; the avoid list; every hostile text of 4.10; hostile schemes (sizes, bad forms, cycles); old settings.
- `crates/sse-kernel/src/names_tests.rs` (4): the sample digest is the same with an unrelated pack beside it; the resolved system is identical with and without the names pack; a hostile scheme refuses the stack with its reason; unknown scheme or count is a plain error.
- Fixture pack `tests/fixtures/packs/names-toy.json` (the Vesh draft and a catalogue scheme).

## Goldens
The kernel contract golden is re-blessed on purpose: five requests are appended at the end of `tests/contract/requests.json` (load the names toy stack, a 10,000-name Vesh sample, a peak named from its colour, catalogue planets under a parent, an unknown scheme). The only earlier lines that change are `hello` (its op list gains `name_sample`) and the unknown-op error (its list of ops gains it too). Because the same golden is checked by the JS runner against the wasm kernel, the 10,000 names are proven identical natively and in the browser.
