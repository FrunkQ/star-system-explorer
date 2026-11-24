# Status: Surface time-lapse and sea drawing

Owner thread: "Build surface time-lapse" (handed R6 part 2 by "Build photoreal renderer", 24 Nov 2026). Covers the
drawing of the kernel's `surface_frames` on the Body globe (`packages/renderer/src/timelapse.ts` and its test, the
time-lapse parts of `packages/views/body`: Body.svelte's `lapse` API and BodyTimeline.svelte, and the `frames()`
plumbing in `packages/renderer/src/runtime.ts`), and the drawing side of the sea items W-sea2, W-sea3 and W-sea4
(a sea shading file of its own). The renderer thread keeps the rest of `packages/renderer`; the kernel side of the
sea is "Build shared climate and weather" (`status/weather.md`), the frames op is proven core's (SF).

Only the owner thread edits this file. Logs are in `handover/packages/`.

| Package | State | Log |
|---|---|---|
| R6c: the time-lapse reads PR 394's frames: a dry world (sea_level_m null) has no sea and its basins are bare rock; the kernel's ice-age onsets, which all fall in the last few million years, get a strip of their own under the main scrubber with a fine range over the orbit-history window, a line each ("Ice age 3 of 9 in the last 200 thousand years, begun 20 thousand years ago: ...") and their why on the card; times inside the window read in thousands of years; the frame's own kernel line on pick; a Solar System Earth test (31 ice ages dated in 500 thousand years, each frame's climate on its own land) | done (this PR) | packages/R6c-timelapse.md |
| R6c part 2: each ice age's spread on the globe, from the cells the kernel names at its onset (reader done; waits on proven core sending `cells` on "ice-age" events) | waiting on the kernel | packages/R6c-timelapse.md |
| W-sea2 drawing: sea colour from the liquid's own absorption (deep colour, shallow fade over the floor, Fresnel with the liquid's index) | next | |
| W-sea3 drawing: waves as the kernel's wave trains (Gerstner sum), ripple detail to the kernel's cut-off | after W-sea2 | |
| W-sea4 drawing: whitecaps and streaks of froth from the kernel's whitecap fraction | after W-sea3 | |

## The sea contract (agreed with the climate thread, 24 Nov 2026)
Same keys at the site (weather `report`, `data.sea`) and on the globe (`survey.climate.sea.bands[]`, one entry per
climate band, null over land). Every key is additive; a missing key means "not sent yet" and the drawing says so.
- W-sea2: `bands_nm` [650, 550, 450]; `absorb_per_m` [R,G,B]; `backscatter_per_m` [R,G,B]; `refractive_index`;
  `deep_rgb` (R∞ ≈ 0.33 b_b/(a+b_b), Gordon and Morel, computed by the kernel). The drawing does the shallow fade
  exp(−2az) over the floor and the Fresnel with that index.
- W-sea3: `trains`, up to 6 of {wavelength_m, amplitude_m, period_s, dir_deg} from the kernel's spectrum and its
  capillary-gravity dispersion; `ripple_cutoff_m`; `superfluid` and `liquid_note` for the card; `wave_m` and
  `period_s` kept as before.
- W-sea4: `whitecap_fraction`, `foam_albedo`, `streak_dir_deg`.
