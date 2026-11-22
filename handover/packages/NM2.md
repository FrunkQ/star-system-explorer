# NM2: names on real worlds

Track NM (`design/SSE4-naming-design.md`, sections 4.2 to 4.5), built 22 Nov 2026 by the kernel thread on NM1's engine. Code: `crates/sse-kernel/src/names.rs`, tests in `names_tests.rs`.

## Names are facts
Names live in the chronicle as ordinary facts, so they are dated, replayed and disclosed like any other fact. No new record type was needed.

| Field | What it is | Written by |
|---|---|---|
| `naming.<layer>` or `naming.<layer>.<kind>` | The scheme that names this layer here and beneath (for one kind only, when given). An empty value means "no names here". | `set_naming`, an authored event |
| `name.<layer>` | A typed name: plain text that always wins and never follows. The older `name` field still counts as the typed common name. | `rename`, an authored event |
| `names.fixed.<layer>` | A drawn name made permanent: the stored form with its parts and faces (NM1's `Named`). It keeps `{parent}` in its form, so it still follows a renamed parent. | `fix_names`, or `promote_star` |

Anything else is drawn when read: a keyed draw, the same on every machine, so an unvisited world's name costs nothing to keep.

## Layers and choosing
- A layer is any id of lower-case letters, digits and `_`. `designation` and `common` are the usual pair, `local_<culture>` for a culture's own.
- **The naming tree.** A thing hangs from its `parent` fact when that names an address (a moon from its planet, a planet from its star, the star from its system), else from the address above. Settings inherit down this tree, and `{parent}` reads the parent's name in it. A loop of parent facts stops after 64 steps.
- **Choosing a scheme**: the nearest setting on the thing or above it (a per-kind setting before a plain one), else the stack's default. Packs give defaults in `provides.other.names.defaults = {layer: scheme}`; a later pack wins.
- **The kind** a form is chosen by is `body.kind`, else `kind`, else the address segment's kind.
- **Index** (`{roman}`, `{letter}`): the position among siblings of the same kind under the same parent, in address order. **Serial**: the number of the system the thing is in.
- **Siblings** are drawn as a group in address order with NM1's clash rule, every sibling whether typed or fixed, so fixing or typing one never moves another.
- **Parent text**: the parent's name in the same layer, else its common name, else its designation (design 4.5). So under a Vesh-named star the catalogue gives "Zakqu III" for the common layer and "KS 0005 III" for the designation.

## Ops
- `read_names {address, at_s?, audience?}`: every layer's shown name with its source (`typed`, `fixed`, `drawn`), the scheme in force, the drawn parts and faces, and under a typed name the one it would have otherwise (`original`, for "follow the parent again"). With `audience`, only the layers that audience's data view shows there, and without `original`.
- `set_naming {address, layer, scheme?, kind?, at_s, rename_fixed?}`: sets or takes away a setting. Unfixed names beneath change at once; fixed ones stay. With `rename_fixed`, every fixed name beneath that the new setting governs is drawn afresh from the new scheme in the same event, dated `at_s`, so the history reads "renamed in 2291" and the old names show before that date.
- `rename {address, layer?, text?, at_s}`: types a name (checked and composed by NM1's text rules), or with no text takes it away.
- `fix_names {address, at_s}`: fixes every drawn name at and beneath the address. The campaign clash rule applies here (design 4.4): a name that folds the same as one already fixed or typed in the same system, or fixed earlier in the same call, moves to its next draw. Fixing is an event, so replay gives the same result.
- `promote_star` now fixes the star's names in the same event, as "looking promotes" says.

## Seats
A scene carries each subject's shown names as `name.<layer>` facts, so a name is disclosed exactly like any other field (the pack's disclosure tiers and data views). `naming.*` and `names.fixed.*` are bookkeeping and never reach a scene. When no scheme is in play (no defaults, no settings, no fixed names), scenes are left exactly as before, which keeps every existing scene and golden unchanged.

## Engine change
- `{letter:a}` letters from a (moons after a numeral, "Tarn II a"), asked for by the data sheet thread for NM3a. `{letter}` still starts at b.
- `Named` and `Part` deserialise (fixed records are read back).
- `Library.defaults`.
- The kernel holds the library behind an `Arc`, so reading names does not copy it.

## Checks (`names_tests.rs`)
- A grown system gets designations ("KS 0005", "KS 0005 III", "KS 0005 III a") and distinct common names.
- **Three seats**: a far seat sees only the designation, a near seat adds the common name, a lander adds the local name; the scene carries the same and no bookkeeping.
- **Replay**: the same requests in a fresh kernel give the same names and state hash.
- **Typed names survive** the world being grown again (new physical facts), refuse unsafe text, and taking one away shows the drawn name again.
- **A setting** on a world changes only unfixed names beneath it (its moon included); a fixed one stays until `rename_fixed`, and reads its old name before that date.
- **Renaming a star** renames its planets and their moons in both layers with no new events on them; a typed planet name stays and its moon follows it; a fixed name still follows.
- A promoted star's names are fixed.
- `sse-names`: moons lettered from a.

## Not here
- Names on resolve for systems that are not adopted (NM4's call sites), and per-campaign clash checks across systems.
- "Which layer a chart shows" is a view setting for the chart room (design 4.2).
- Find by any known name.

## Goldens
The contract golden is re-blessed on purpose: ten requests are appended (a names campaign with a small authored system: read, fix, a setting with renames, a star rename, a far seat's names and a GM scene). The `hello` op list and the unknown-op error gain the four new ops. Nothing else changes, because no earlier stack has names in play.
