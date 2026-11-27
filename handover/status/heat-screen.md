# Status: a world's energy flow (the heat flow screen)

Owner thread: "Show a planet's heat flows over time" (opened 26 Nov 2026 by the coordinator, from Alex's message of 21:48 UTC: heat flows "under extended details... kinda like the ship energy flows... a time slider to see the changes over time and probably getting smaller as it ages"). Covers: HV, the screen side of track HF: the melt design's C6 thermal-line chart, and later the tidal-orbit design's D4 moon-distance chart beside it.

Only the owner thread edits this file. One row per package; logs in `handover/packages/`.

| Package | State | Log |
|---|---|---|
| HV1 Energy flow on the Body screen: a world's heat books as the Flow view (RK13's), a thermal line (mantle and core temperature; heat lost and out of the core) with the history's marks, a slider along the history's own steps, words for why it is as hot as it is, and honesty marks chosen by conditions | done (PR 670) | packages/HV1.md |
| HV2a the survey publishes the climate's sunlight budget (arriving, reflected, absorbed, Bond albedo, internal, emitted), for the outside half | done (PR 678) | packages/HV2a.md |
| HV2b the outside half: the climate's sunlight books as their own flow at their own scale (sunlight and heat from inside in; reflected and given off as warmth out), the reflected share as a bar, words comparing sunlight with the heat from inside, and life's share from the LE2 ledger (LE-U1: what life catches, what staying alive costs, the pace of evolution). Plus HV2c, found on the way: a moon's sunlight was read at its orbit round the planet (our Moon got 150,000 times Earth's sunlight); now at the planet's distance from the star | in review | packages/HV2b.md |
| HV2 the full books: once HF4b.1 publishes the melt design's section 7 fields, check the view on Earth, Mars, the Moon, Io; add the lid and crust to the line (HF4b.1, HF4b.2) and retire each honesty mark as its package lands | waits on HF4b.1 ("Fix low heat flow in worlds", expected 27 Nov) | |
| HV3 moon distance and tidal heat over time (tidal-orbit design D4), beside the thermal line | waits on HF6 | |

## The agreed data shape (26 Nov 2026)

Agreed with "Design melting inside worlds" and "Fix low heat flow in worlds": the melt design's section 7 names, per `CoreStep`, in W/m² of surface (the view multiplies by the world's area for powers):

- `heat_lost = heat_conducted + heat_erupted`
- `heat_made = heat_radiogenic + heat_tidal`
- `heat_lost = heat_made + mantle_cooling + lid_cooling + core_heat` (a kernel test holds this to 1 part in 10⁶)
- `core_freezing + core_cooling = core_heat`; stores are negative while they warm; `lid_cooling` and `heat_erupted` are zero, never absent, until HF4b.1.
- Marks are `CoreEvent` kinds (melting-start, last-lavas, erupting, conducting, crust-recycling, tidal-runaway, freeze-out, dynamo-start, inner-core-birth, dynamo-death); the view draws any kind with its `why`.

`core.rs` is the heat flow thread's: screen threads ask, never edit it.

## Open points for Alex (not blocking)

1. The slider is a second time control inside one panel (interface style rule 4 says one ruler for all of time). It looks back along the world's own history without moving the campaign, the way the Power card's "Starlight as at" tries a distance. Recommend keeping it: dragging the campaign back billions of years would re-resolve every system to show one world's past.
2. (Superseded 26 Nov 23:18: Alex wants sunlight in, as the outside half; built in HV2b as two linked halves at labelled scales.) Sunlight is not in the flow. It is the surface's budget, not the inside's: for Earth about 2,500 times the inside heat, so it would turn every inside band into a hairline, and it never reaches the mantle. Recommend one sentence beside the flow ("sunlight brings N times more to the surface, and leaves again as warmth") once the climate publishes absorbed sunlight.
