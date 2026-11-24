---
name: sse4-exotic-objects
description: SSE4 exotic objects design (track XO, 24 Nov 2026): black holes with real light bending, pulsars, stripped stars; where it lives, status, what is waiting on Alex
metadata:
  type: project
  modified: 2026-11-24T16:56:19.378Z
---
Thread "Design exotic stars and black holes" (started 24 Nov 2026 after Alex's "yes to all" at 16:41).

Design: /mnt/project-files/sse4/SSE4-exotic-objects-design.md (repo copy handover/design/, PR 420). Track XO, packages XO1 to XO16. **Status: proposed**; 11 numbered questions in section 13 wait on Alex (asked 24 Nov about 17:10).

Mockup (working WebGL picture traced from the real bending equation, band switch, straight-light comparison): https://claude.ai/artifact/RHxpiUGRM3QVemo4EveD1n ; file and screenshots in /mnt/project-files/sse4/handover-screens/XO-screens/.

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
