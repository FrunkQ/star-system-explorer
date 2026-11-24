---
name: sse4-exotic-objects
description: SSE4 exotic objects design (track XO, 24 Nov 2026): black holes with real light bending, pulsars, stripped stars; where it lives, status, what is waiting on Alex
metadata:
  type: project
  modified: 2026-11-24T16:56:19.378Z
---
Thread "Design exotic stars and black holes" (started 24 Nov 2026 after Alex's "yes to all" at 16:41).

Design: /mnt/project-files/sse4/SSE4-exotic-objects-design.md (repo copy handover/design/, PR 420). Track XO ("exotic objects", a package code only; Alex queried the name), packages XO1 to XO18; status file handover/status/exotic-objects.md (no builder thread owns it yet). **ACCEPTED by Alex 24 Nov 2026 17:03** ("Yes to all" to 11 questions). Builders start with XO1 (bend table), XO17 and XO18.

Added by Alex the same day:
- **Composite** at the end of the band switch with a drop-down to tick bands (design 6A, XO17): one mechanism for every view, presets as pack data, each band scaled to its own brightest, legend lists layers, seats only their sensors' bands, phones two bands.
- **Hazard map** (design 3.8, XO18, PR 433; was "approach diagram"; Alex 17:19: wanted for most hazardous objects, Jupiter's belts named): one instrument for any hazardous body, rail entry plus card line plus a system-scale Hazards overlay. Two readings: Map (a cut, drawn with the Mag tracer) and Profile along a stated line (equator's plane, over the poles, a ship's course). Hazards: heat, radiation (belts from read_field `belts` with edges, moon-swept gaps, per-shield dose, latitude fall-off; light; storms), beams and jets, debris, tide, field, clocks, air, climbing out. Two ship marks: Park (closest orbit it can still climb out of) and Dive (closest fall-through that comes back; free to leave, so much deeper); inside Dive is "too late". Op `hazard_map`. GM figure, shown to players through the eye. Open points 12 to 15 in section 13 with recommendations builders follow. Builder thread "Build exotic stars and black holes" told 24 Nov.

Mockup (working WebGL picture traced from the real bending equation, band switch with Composite, straight-light comparison, approach diagram): https://claude.ai/artifact/RHxpiUGRM3QVemo4EveD1n ; file and screenshots in /mnt/project-files/sse4/handover-screens/XO-screens/.

Design core (three general mechanisms, no object-type list):
- `light-path` stage in True with benders: strong gravity (kernel "bend table", exact for no spin, same on every device, phone-cheap; Luminet 1979 method), weak gravity (4GM/c²b; microlensing O15, cluster arcs), motion (fast-ship Window). Spin bending is a second pass for GPU-high and stills (XO7); phones stay on the no-spin table with a badge (fudge).
- The feed: matter onto any body through a disc; inner edge = largest of surface, last stable orbit, magnetospheric radius; thin, hot and thick flows blended by Eddington share.
- Shape and outflow: shape from spin and pull, gravity darkening, wind optical depth (soft-edged Wolf-Rayet stars), colliding-wind dust spirals.
- Time stretch chip on moving True views; hard cap of 3 flashes a second.

Honest looks recommended: a fed hole's disc is blue-white in Visible, the orange ring is Radio in mapped colour; pulsar beams are cones only in diagrams (HUD, Chart, Mag), True shows the pulse only when the beam crosses the viewer.

Alex 24 Nov 16:53: pulsar glitches (neutron superfluid catching the crust up) are in, as a small cited block `sse.pulsar.glitch`; he likes cheap "cool physics wins" that a clever user can verify.

Existing state checked at main 67a5957: `sse_light` Part::Disc (Shakura-Sunyaev) and shifts (O7) exist; remnants carry mass only; renderer R5b (discs, jets) not started and becomes XO4 to XO6.

Sources in the note are from memory; the citations thread should check them before blocks rely on them.

Related: [[sse4-renderer-track]], [[sse4-light-track]], [[sse4-magnetic-fields]], [[star-types-and-one-body]]
