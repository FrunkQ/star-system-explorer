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
| R6c: the time-lapse reads PR 394's frames: a dry world (sea_level_m null) has no sea and its basins are bare rock; the kernel's ice-age onsets, which all fall in the last few million years, get a strip of their own under the main scrubber with a fine range over the orbit-history window, a line each ("Ice age 3 of 9 in the last 200 thousand years, begun 20 thousand years ago: ...") and their why on the card; times inside the window read in thousands of years; the frame's own kernel line on pick; a Solar System Earth test (31 ice ages dated in 500 thousand years, each frame's climate on its own land) | done (PR 436) | packages/R6c-timelapse.md |
| R6c part 2: each ice age's spread on the globe, from the cells the kernel names at its onset. Proven core's PR 417 sends them: the Solar System's Earth has 28 ice ages in 500 thousand years, every one with its cells, and its first frame is 265 K against 283 K today (the faint young Sun) | done (25 Nov, checked on main after PR 417) | packages/R6c-timelapse.md |
| R6d: natural outlines in the time-lapse (Alex 25 Nov, "reuse on the timelapse"): `recolour` blends its cube cells bilinearly and thresholds coast and ice with the one shared edge noise and threshold from ui-contract globe.ts (owned by "Build detail ladder and world textures", beside `sampleNatural`; no second copy), keyed on direction and world seed only so coasts and ice edges stay still between frames unless they move; a frame-to-frame stability test | waiting on that thread's function | |
| W-sea2 drawing: sea colour from the liquid's own absorption: `sea.ts` reads the agreed keys (missing ones named per step, said in the Land note), the Land screen's water body is the kernel's deep colour with the floor (tile albedo) seen through exp(−2az) of liquid, and the sky splits by exact Fresnel at the liquid's index; the landing's sea carries the numbers to watching seats. Until the kernel sends them the look is as before | done (drawing); kernel numbers pending from the climate thread | packages/W-sea2-drawing.md |
| Parked by Alex (24 Nov 20:41, "we will come back to this"): the time-lapse world's air and oceans evolving through the frames (outgassing, oxygen built up by life); today the frames hold today's oceans and air throughout, so early Earth does not look right. Kernel side first (proven core, climate) | parked | |
| W-sea3 drawing: the Land screen sums up to six wave trains, the kernel's own when sent (so a methane sea moves as methane does), the V10k stand-in otherwise; the note says which. Ripple detail to the kernel's cut-off still to come | done (drawing); kernel trains pending | packages/W-sea2-drawing.md |
| W-sea4 drawing: froth over the kernel's whitecap share as soft streaks along the wind (or the kernel's streak heading), at the froth's own reflectance; nothing drawn until sent | done (drawing); kernel numbers pending | packages/W-sea2-drawing.md |

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
