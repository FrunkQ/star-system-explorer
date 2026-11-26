# MC9: life support read as air and supplies

Track: ship manifest and crew (MC). Date 26 Nov 2026. Asked by Alex, 26 Nov, answering the six crew questions: "life support did have a system in v3... just a 'days supply' - i guess we should have 'air' and 'supplies' as the 2 main ones."

## What a GM sees

- The Manifest's **Life support** now leads with two headings, **Air** and **Supplies**, each as days left for the people awake aboard. A heading lasts as long as the first of its stores to run out, and says which one when it holds more than one ("food runs out first").
- At Science and Engine depth each heading lists its stores as before (mass, days, daily use). At Curious only the two headings show.
- The sentence above them ("Oxygen runs out first, in 41 days, for 2 people awake") is unchanged.
- Cold sleep is as accepted on 23 Nov: people in cold sleep draw nothing, so a ship whose crew is all asleep shows "not used" under both headings.

## How it works (no kernel change, no golden change)

- The finer stores (oxygen, water, food) and their cited rates are kept: they are what the sums are made of, and recycling works per store (oxygen and water recycle, food does not). Air and supplies are a readout over them, not new amounts.
- The grouping is pack data: `provides.other.life_support.headings` in `sse.base-manifest`, `air: [oxygen]`, `supplies: [water, food]`. A higher pack (an alien kind that breathes something else, a setting's own words) replaces the headings whole. With no headings on the stack each store is its own heading, and a store no heading names gets one of its own, so grouping never hides a store.
- `lifeSupport(...)` in `ui-contract/manifest.ts` returns `headings` (`HeadingDays`: heading, its stores, total kg, days, first store out) beside the existing `stores` and `first`.
- Why not one "days supply" like V3: air and supplies fail on different clocks (air is recycled, food is not), and a GM wants to know which one bites.

## Checks

- `tests/ui/manifest.test.ts`, two new tests: the base pack's headings are air then supplies; air's days are absolute (82 kg of oxygen over two people's daily rate, the sleeper drawing nothing); supplies last as long as the shorter of water and food and name it; an all-asleep crew uses nothing; no headings, partial headings and a higher pack's headings each behave as above. They fail with the change removed (`headings` is missing).
- `tests/ui/manifest.e2e.mjs` unchanged and run against the built shell: passes. Screenshots (Curious depth, desk and phone): `/mnt/project-files/sse4/handover-screens/MC9-screens/`. The nested store rows at Science and Engine depth were not screenshotted.
