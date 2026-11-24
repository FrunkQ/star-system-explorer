# What exists on main for the Playbook and Switchboard (24 Nov, from "Build Run desk playbook and players' side")

1. **Seats.** `packages/ui-contract/src/seat.ts`: `Seat {id, label, audience, authority gm|player, may_send, auto_approve?, sealed, screens?, lens?, embed_origins?, brand?, live?, shown?}`; a plan holds the seats (V10).
   - A screen joins by link: `/players.html?join=<room>&seat=<id>` (its own kernel replays the GM's intents, checked by state hash) or `?sealed=<room>` (no kernel; the GM sends finished scenes).
   - Transport: BroadcastChannel in the same browser, else the site Worker's relay room.
   - There is no "role" separate from a seat. A seat is in effect a role: any number of tabs or devices can open the same seat link. A tab on the GM's own machine can be a seat. `screens` lists screen documents, each meant as its own window or device.
2. **What can be sent.** GmHost host-state keys: `shown` = `{scope, title?, screen?, landing?}` or `{off: true}` (Send this view, Screens off, for the whole table); `seats` (each with its own `shown` for a per-seat send, and `live` quick switches); `party` (PartyShip `{ship, name, nav, at, voyage?}`); `clock_yr`; `data_views`.
   - Reveal is the `set_known` intent through the intent log (dated; `undo_known`).
   - Land rides inside `shown.landing` (the renderer's LandingShow: system, body, site, lander, phase, flight_s, pose…); seats replay it on their own kernel and follow the GM's pose.
   - Nothing else is sendable. Read-aloud does not exist.
3. **Looks.** Only the mockup (artifact "SSE4 Look screen"). Built: view styles Chart, 3D, True and Wireframe; V2 player themes with filter chains (skins/player, FilterChain in chart-grammar). Not built: signal dial, signal faults, screen delivery transitions, pinned preview, in-universe looks as data.
4. **GM-side preview of a seat.** None today. The cheap path: the GM's kernel already builds any audience's scene (`session.scene(scope, {audience})`, which sealed seats get), so a preview can mount the same player view stack on it, at one scene build per seat per change. Live True or 3D thumbnails for every seat are the heavy part. I recommend Chart previews by default, with True only for the selected screen.
5. **Beats.** Nothing built beyond the shape agreed with Alex. There is only `ui-contract/src/arrival.ts` (uncommitted): it offers "Beowulf is at Mars. Tell the table?" and sends one set_known.
   - Proposed: a preset is plain data, an existing action (a Shown, a set_known, a landing, a read-aloud text, a look) plus a target role, so that firing a preset and doing it live go down one path.
6. **Controls to absorb.** All in `packages/shell/src`, moving to `src/run/` in the chart room's split PR:
   - the table part of App.svelte's status line: share link, "Table sees: …", Send this view, Screens off, request count;
   - `SeatsPanel.svelte`, the seats dialog: add seat, fog of war, screens, per-seat Send, Screens off and Back with the table, live switches, embed origins, brand.

   The party line in the players' header stays on the players' page.

**Awkward bits**
- Per-seat Screens off is not yet honoured for sealed seats.
- Role versus device needs a new join step (a device picks, or is given, a role), because links are per seat today.
- Read-aloud needs a new Shown kind.
- Pause with a joke is easy as a Shown variant. A paused screen must stop receiving the scene rather than cover a live one, so a pause has to be a send.
- OBS out is easy: a clean output page per role. OBS in stays GM-side only, as agreed.
