# L3-air: a chart world's early ocean follows the history's own air

Track LE (life on the chart's worlds, L3). Built 27 Nov 2026 by "Build life's energy baseline", on the coordinator's ruling after the correct-tilt Earth failed its life test (see `status/life-energy.md`, "The catalogued Earth on a correctly tilted Earth").

## The fault

`world_life::stack` built a chart world's habitats once, from today's air, and the history then held them for the world's whole life. Vent iron reaches the ocean only while the ocean is anoxic (`anoxic_below_o2`). Today's Earth air is 21% oxygen, so the early ocean never got its ferrous iron. It read oxygen that life only makes later, which breaks "nothing reads a value a later pass writes". The Archean mats ran on vent hydrogen alone, at 8e11 g C a year against the 1e14 to 1e15 of Canfield et al. 2006.

## The fix

- `history::Epoch` gains `graph_while`: habitat graphs used instead of `graph` while a world fact is below a threshold, first match wins. The fact is read at each step's start, from the physics boundary (the coupled air box) or else the epoch's own facts. The budget caches key on the variant too.
- `world_life` builds the anoxic-ocean graph (`stack_ocean(.., anoxic: true)`) when today's air holds oxygen. The today-held epoch uses it while the history's own `o2_fraction` is below `anoxic_below_o2`. There are no new numbers.
- `world_life::stack` keeps its behaviour: today's air, for the present-day biosphere, ledger and speed.

## Results

On main's test loader (Earth wrongly tilted) every life test still passes. On the correct loader (patch `open-sol-radians`) the catalogued Earth's mats reach 4.2e14 g C a year. Oxygen photosynthesis arrives at 4030 Myr and complex cells at 4415 (seed "home"), where before neither came. Land still does not arrive by today. That waits on the held-heat step and on Alex's answer on conditioning (status file, points 3 and 5).

## Checks

- `sse-life` `history_air_tests::an_anoxic_air_lets_the_ocean_keep_its_iron`: the anoxic variant's mats are over 1e14 and 100 times those without it. It went red with the variant selection removed.
- `history_air_tests::oxygen_in_the_air_closes_the_variant`: with oxygen in the air the variant changes nothing. It went red with the variant always chosen.
- `sse-kernel` `life_tests::earths_early_ocean_keeps_its_iron`: the catalogued Earth's mats are over 1e14, on both loaders. It went red without the anoxic graph.

## Goldens

None moved: the `sse-life` fixtures carry no variant, and the core contract's life cases are fixtures.
