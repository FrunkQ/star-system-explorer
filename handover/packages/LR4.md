# LR4: "In orbit" for the table

Track LR (status/landing.md). Between arrival and the site, the players saw only the header. Now the GM can send a
world's own screen and the table sees the world turning below.

- `playerBody` (views/body `PlayerOrbit.svelte`, in `PLAYER_STACK`): the world the send names (the screen selects it
  from `focus`), drawn from this seat's own kernel through `ctx.ground.surfaceTiles` on Body's globe path
  (`mountGlobe`, true-colour recipe, spin on, no cutaway, no controls). Players watch.
- `players.orbit.screen.json` ("In orbit"): the globe, the players' card and text. Copied by `scripts/build-shell.sh`.
- `here()` (App.svelte): on the world rail on any screen but Land, with a planet or moon selected, Send this view sends
  the orbit screen with the world as `focus`, titled by `orbitTitle` ("Beowulf in orbit of Mars" when the party's ship
  is there, else "In orbit of Mars").
- Players.svelte: the tile grant (`Session.grantLanding`) now comes from `groundGrant`: a landing's world, or an orbit
  send's world when the table already knows it (it is in the seat's scene). Nothing else is opened. Run desk told.
- TableStatus (Run desk file, a one-prop seam): `orbitUnsent` shows Send this view when the world on screen is not the one the table is shown in orbit.
- `run-through.e2e` step 3b sends Mars from its world screen after the voyage and checks the player's globe.

Not yet: the world's air on the player globe (a player seat does not read resolved systems; the GM would need to send
the air facts, as a landing sends `world`), and a ship marker. No golden change.
- surface.e2e: the morning hour is -5.5 local h (was -5.25), dark enough for the stars check after the sea drawing merged.

## LR4b: the world's air on the orbit globe (25 Nov)
- The GM's kernel reads the focus world's air (resolved facts, then `read_air_optics`) while players are connected,
  and `here()` sends it with the orbit as `Shown.air` (seat.ts); a send made before it is read goes again with it.
- `PlayerContext.shown()` (ui-contract view.ts, through ScreenRuntime from Players.svelte) gives a player view what
  the GM sent; `PlayerOrbit` passes the air to `mountGlobe`, so the table's globe has the same atmosphere pass as the
  GM's. `data-orbit-air` marks it; `run-through.e2e` waits for it.
- Also carried: the renderer's review of LR6, the hull and the ground now share one sun and sky term
  (`sunLightOn`, `skyLightOn` in gpu-surface.ts).

## LR4c: the party's ship in orbit on the table's globe (25 Nov)
- `here()` names the party's ship with the orbit (`Shown.ship`) when it is at the focus world; `PlayerOrbit` draws it
  as a named marker going round on a tilted ring, passing behind the world on the far side (SVG over the globe, theme
  tokens only). A presentation drift (fudge register, "The ship in orbit on the table"). `run-through.e2e` checks it.
