# LR2 and LR5: the hour of a landing, and a whole landing in its link

Thread "Build landing run-through", 24 Nov 2026. Gaps LR2 and LR5 (`handover/status/landing.md`).

## What
- LR2: the Land screen's "Hours from now" slider bound degrees (±180) under an hours label, and a landing went in at
  whatever hour the campaign's now put the site (03:42 at Mars in the walk: a black screen). Now the slider is in the
  world's own local hours (±12, a 24th of its solar day) and shows the local time, and four presses, Dawn, Midday,
  Dusk and Night, move the world's clock to the next such hour. They sit on the site card before landing and on the
  walk pad after. The table gets the hour as before (`LandingShow.hour_deg`).
- LR5: the site link carries the hour and the lander: `~surface?site=lat,lon&time=midday&lander=party` (ui-contract
  place.ts `Place.site.time`, `.lander`, `LANDING_TIMES`). The Land screen applies them when it takes the site. So a
  bookmark or a Playbook preset of a landing is just a place link; firing it opens Land on that site at that hour with
  that lander, and Send this view sends it. App's `fromHash` now always follows a link that carries a site, so a
  landing fired while Land is already open still goes.

## Files
`packages/views/surface/src/Surface.svelte`, `packages/ui-contract/src/place.ts`, `src/view.ts`,
`test/place.test.ts`, `packages/shell/src/App.svelte` (two lines), `tests/ui/surface.e2e.mjs` (slider value in hours),
`tests/ui/run-through.e2e.mjs`.

## Checks
`npm run ui:check`, `place.test.ts`, `tests/ui/run-through.e2e.mjs` (Midday pressed, the table lands at 11:57 local;
a link with `time=dusk&lander=capsule` comes back whole).

## LR5b: the Land screen's own link
On Land, App's `place` carries `site` from the landing being chosen (ui-contract landing.ts `landingLinkSite`: site,
quick hour within half an hour, lander "party" for a `{ship}` lander), kept as text so a pose never stirs it; the
address bar follows. `here()`'s title is `landingTitle` ("Mars, landing at Dribeque Planitia, midday"). Test in
`landing.test.ts` and `run-through.e2e` (the hash ends `&time=midday&lander=party`; the table line names the landing).

## For the Run desk (PB3 "A landing")
A landing preset stores the place link and fires it, then Send this view.
