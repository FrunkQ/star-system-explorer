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

## Next (part 2)

- Quick overrides: an unsaved toggle on the Seats panel that turns a seat's
  screens' HUD (labels, orbit lines) off live, e.g. for a wall screen. Rides on
  the shared view controls' HUD; travels with the seat plan but is never put
  into the campaign file.
- V3's "size coming" on the join screen: show the manifest's size while a
  player joins.
