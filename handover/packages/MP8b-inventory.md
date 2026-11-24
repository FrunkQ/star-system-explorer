# MP8b: TG11's resource inventory over the composition (proven core thread, 24 Nov)

This is the second part of MP8 (matter pack design, section 5, step 6, and build table row MP8), and it is TG11's inventory half (tags design, section 5.5). Alex accepted it 23 Nov ("build them as stated, use blocks").

## What it does

**Resource kinds are pack data.** They live in `provides.other.resources.kinds`, one entry per kind: `{id, name?, matter?, reach?, about?}`.
- `matter` is an element id on the sheet (lower case: `fe`, `ir`) or `h2o`. A kind with no matter is a made good (stock only).
- `reach` is `crust`, `sea`, `air`, `surface` (the default: crust, sea and air together) or `whole`.
- Ids are lower case letters, digits and dashes, and the stack refuses any other.
- Where two packs declare the same id, the later pack wins.
- Any element id or `h2o` is a kind by itself. So a resource nobody declared, or one whose element nobody wrote a block for, still has an inventory (MP8's check). No block is written per resource.

**`read_inventory {system, body?, kinds?, at_s?}`** (Trivial) returns `inventory_read {bodies}`.
- Each body gives its mass, its source, and per kind: kg, per_kg, by_reservoir and note.
- Amounts are read off `read_composition` at Survey, using the same `bulk_of`, and summed over the kind's reach.
- Water is the body's surface water for sea and surface, and all its water for whole.
- Left without `kinds`, the read covers every declared kind. An unknown kind is refused (`unknown_resource`); naming no kinds when the stack declares none is refused (`no_kinds`).

**Fact-built worlds use their makeup.** `sse_surface::Body` gains `makeup` (from the `makeup.metal`, `rock`, `carbon`, `ice` and `gas` facts placement writes). `Bulk::of_body` then takes each material from its share, with ice as water.
- Without makeup facts nothing changes, and the field is not serialised. So no survey, surface or golden hash moved apart from the contract golden below.

**No dice anywhere.** The same world always gives the same amounts.

What has been mined out (`extracted`) and what has been brought in (`stock`), with held = natural − extracted + stock, are kept values owned by the Traveller pack thread (TG11's other half, with TG8's `resource.*` labels). Deposits are part of the crust: `survey_surface`'s `holdings` (MP8a) says how much of it is ore.

## Checks

`sol_tests::inventories_follow_the_makeup` adds two asteroids beside Sol from makeup facts, and a fixture pack declaring iron (reach whole), iridium, water ice and packaged food:
- the metal-rich rock holds over three times the iron per kg of the carbonaceous one;
- iridium (no block anywhere) has an inventory;
- the carbonaceous rock holds more water;
- packaged food holds nothing natural, with a note;
- an undeclared `ni` works on its own;
- the source reads "the body's makeup facts";
- the same read twice is identical;
- with no kinds named, all four come back;
- `unobtainium` is refused.

## Goldens changed on purpose

`tests/golden/contract.jsonl` changes as follows:
- the op list in `hello` gains `read_inventory`;
- the unknown-op error lists it;
- two new cases are added. The contract stack has no sheet, so both read as `unknown_resource`.

## Stand-ins (in each answer)

- Natural amounts only.
- Deposits are inside the crust.
- A gas envelope's hydrogen and helium are not counted yet (MP8c: giant envelopes, with helium-3 once the sheet carries isotope shares).

## Next

MP8c is the giants:
- the heavy-element share from the mass–metallicity relation (Thorngren et al. 2016);
- the envelope's H and He at protosolar shares (Lodders 2021, on the sheet);
- helium-3 from an isotope share row the data sheet thread would add.
