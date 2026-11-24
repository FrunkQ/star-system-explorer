# MP8c: giants' envelopes in the composition (proven core thread, 24 Nov)

This is the third part of MP8 (matter pack design, section 5; tags design, section 5.5). Alex accepted it 23 Nov ("build them as stated, use blocks"). Before it, a giant's whole mass was counted as solids (an MP6 stand-in). Now a giant is mostly hydrogen and helium, so TG8's "gas skimming on giants only" and "helium-3 on all four giants" have amounts to read.

## Block

`sse.composition.heavy_element_share`: Z_planet = clamp(Z_star · a · (M_Jup / M)^b, 0, 1).
- This is Thorngren et al. 2016's mass–metallicity relation for measured warm giants. Its parameters are a = 9.7 and b = 0.45, and M_Jup comes from the sheet.
- The matching sum is in `sse_atoms::matter`, held to the block by `composition_blocks_match_the_chain`. The atom count goes up by one.

## Chain

**Where a giant's gas comes from.** There are three cases:
- **Run bodies:** formation's captured gas (`Protoplanet.gas`) shares the body's mass with its solid ledger (`Bulk::with_ledger_and_gas`).
- **Fact-built worlds with `makeup.gas`:** the gas is that share of the mass.
- **Fact-built deep envelopes with no makeup** (catalogued Jupiter to Neptune): the solids are the relation's share of the mass, keeping their proportions, and the rest is envelope (`Bulk::with_envelope`). The star's metal share is the system's `star.metallicity_z`, or the kernel's default.

**The envelope's elements.** They are hydrogen and helium at their protosolar shares, from `abundance_protosolar` times `atomic_weight` on the sheet (Lodders 2021). They sit in a new reservoir, `envelope_kg`, which is not serialised when zero, so rocky worlds' answers are unchanged.

**Rocky elements** separate as before, using solids only.

## Inventory

- `read_inventory` gains reach `envelope`.
- A kind's matter may be an isotope id such as `he3`. The isotope is the element's reservoirs times its mass share: the sheet's protosolar atom fraction (`isotopes[].abundance_protosolar`, Lodders 2021, PR 365) times the isotope's mass, over the element's weight.
- An isotope without that row reads 0 with a note.

## Checks

`sol_tests::giants_are_mostly_envelope`:
- Jupiter's heavy share is between 8% and 25%;
- Neptune's is more than twice Jupiter's;
- the H/He mass ratio is between 2.2 and 3.2;
- a `hydrogen` kind with reach envelope gives Jupiter over 1e26 kg and Earth none;
- `helium-3` on Jupiter is the envelope's helium at 1.66e-4 of its atoms, by mass, within 2%.

Jupiter's hydrogen reads 62% of its mass.

## Goldens changed on purpose

- The `base-physics` pack and its fixture gain the block.
- `tests/golden/contract.jsonl` changes as follows:
  - stack hashes follow the pack;
  - formation-run giants in the matter case now carry an envelope.

## Fudge register

The relation's validity range is about 0.03 to 10 Jupiter masses; outside it the fit is extrapolated. The envelope also carries no heavy elements of its own: they are all counted in the solids.
