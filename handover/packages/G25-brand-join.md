# V3 gap 25 part 1: branding and join words on player views

Owner: Traveller pack track. Stage 5.

## What it does

- A seat may carry a brand (`Seat.brand {name, logo?}`): a faction's or
  ship's name and a small logo, shown in its screens' header. Logo is a PNG,
  JPEG or WebP data URI, about 64 KB at most; no SVG (it can carry script) and
  no outside address. `checkBrand` words each problem; `checkSeats` and the
  Seats panel form refuse a bad one. Presentation only.
- The words a player screen says while it joins, waits for the scene and when
  the GM is away come from the stacked packs' vocabulary layer
  (`provides.other.vocabulary.join {joining, waiting, away}`, beside the
  `terms` of `words.ts`): line by line, the topmost pack that words a line
  wins, else the app's plain words. `{seat}` and `{brand}` are filled in.
- The Traveller pack words them in its own (original) voice: "Calling the
  ship's computer for {brand}…", "Link open. Receiving the charts…", "Signal
  lost…".

## Golden

`tests/golden/traveller-codes.json`: the Traveller pack's hash only, because
the pack gained `vocabulary.join`. No code reading changed.

## Checks

- `tests/ui/brand.test.ts`: brand checks, seat check, join words from the
  stack, filling.
- `tests/ui/embed.e2e.mjs`: a branded seat's own page shows the brand.

## Part 2: quick overrides

- `Seat.live` holds the GM's live switches for a seat's screens, as player view
  parameters by name. The Seats panel lists them per seat (`LIVE_OVERRIDES`,
  for now "Hide labels", which sets `labels: false`). A switch goes to the
  seat's screens at once through the host's state message. The seat plan is
  not saved with the campaign, so neither are these.
- `ScreenRuntime` takes `live` and lays it over each mount's parameters, only
  where the view declares that parameter (`withLive`), remounting when it
  changes.
- Checks: `tests/ui/brand.test.ts` (`withLive`), `tests/ui/embed.e2e.mjs` (the
  crew's chart drops its names when the GM ticks Hide labels, and gets them
  back).

## Next

- Hide orbit lines once a player view draws orbit lines (add it to
  `LIVE_OVERRIDES`); the switches for the single shared table (the panel lists
  seats only when there are several); sealed seats (their screens take the seat
  from the sealed host, which does not resend it yet).
- V3's "size coming" on the join screen.
