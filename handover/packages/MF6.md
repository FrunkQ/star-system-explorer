# MF6 part 1 (light track): what stops the wind, the bubble, moons inside, oceans answering

Magnetic fields design (`handover/design/SSE4-magnetic-fields-design.md` once synced; project files copy now)
sections 6.1 and 6.6, stage MF6, accepted by Alex 22 Nov ("agree to all").

## What was built
- Atoms (sse-atoms `field.rs`, count 69 → 73): `sse.field.ionopause_altitude` (air pressure meets the outside
  pressure), `sse.field.shue_flaring` (Shue et al. 1998 exponent), `sse.field.standoff_in_field` (a moon's dipole
  against its host's field and plasma), `sse.induction.response` (thin conducting shell, thickness capped at the
  skin depth). The own-field stand-off is the existing `sse.light.magnetopause_radius`.
- Crate `sse-field`: `read(body, settings)` computes all three obstacles for every body (own field, air,
  bare surface) and takes the outermost; works out whether a moon sits inside, outside or crosses its host's bubble
  (host stand-off over the wind's gust range); inside, the pressure is the host's field and plasma; each conducting
  layer's induced amplitude and dipole moment from the host field's swing; why lines, stand-ins, references.
- Kernel op `read_field { bodies }` → `field_read { bodies }`, error `bad_field`. Conditions are handed in
  (radius, moment, air, wind, host, layers) until MF2 publishes the field as facts; then `system`/`body`
  addressing joins, as the design's kernel surface says. `read_field_history` waits on MF3.

## Evidence (`cargo test -p sse-field`, 4 tests)
- Earth nose 9.9 radii (about 10); Shue α 0.59. Mercury 1.33 radii (about 1.5, Anderson et al. 2011).
- Jupiter 41 radii from its dipole alone; the measured 60 to 90 needs its plasma (MF7's sources). Not yet met.
- Venus air boundary 566 km (measured ionopause 250 to 400 km); Mars 305 km (300 to 800); the Moon struck at its surface.
- Ganymede inside Jupiter's bubble, its own bubble 2.3 of its radii (about 2, Kivelson et al. 1998).
- Titan at 20.3 Saturn radii crosses Saturn's edge (17 to 30 radii as the wind gusts).
- Europa's 100 km salty ocean (2.75 S/m) answers Jupiter's 230 nT, 11.1-hour swing at 0.998 of a perfect conductor
  (Kivelson et al. 2000: about 0.97, within 30 %); fresh water barely answers.

## Goldens
`tests/golden/contract.jsonl` gains the `read_field` cases on purpose; `base-physics` pack and fixture re-blessed.

## Next
MF7 (cut-off, dose, belts and the plasma sources that also inflate Jupiter's bubble), MF9 aurorae, MF4 stars when
MF2 lands, MF5 wind torque with the kernel thread.
