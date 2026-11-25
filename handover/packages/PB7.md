# PB7: the role card

Thread "Build Run desk playbook and players' side", 24 Nov 2026. Design section 7.3, question 4 (one layout per role), and the "what exists" table in section 8.

## What
- **The role card** (`run/RoleCard.svelte`): a role's name on the Switchboard opens it, growing from the heading. Its fields, in order:
  - Name.
  - Knows: its own discoveries (fog of war), its own knowledge, what everyone has been shown, or what another role knows.
  - Layout: one screen document.
  - Links: the join link, and the tabletop link when it has tabletops.
  - Players' requests from that role, with Approve and Decline.
  - **More:**
    - may suggest names, and whether they apply without asking;
    - the brand, with logo, motto and accent;
    - tabletops;
    - the quick switches ("Hide labels");
    - tonight's redirect ("its sends go to Bridge's screens");
    - discoveries, for a fog-of-war role;
    - Remove.
- **"+ Add a role"** opens the same card empty. Sealed is chosen there, because a sealed role gets its own room when it is made.
- **Look and sensors** are named on the card as coming with looks (PB10) and with bands on players' screens.
- **The Seats dialog and `SeatsPanel.svelte` are deleted.** The status line keeps:
  - the on-air chip (off the Run desk);
  - Send this view;
  - the players' link;
  - a "Roles" link that opens the Switchboard.
- **Requests** are no longer on the status line. The chip says "N asking", and the role heading carries "N asking" too.
- **ui-contract:**
  - `seats.ts` gains `updateSeat` (a role's own audience is renamed with it), `addSeat({ audience })` to share another role's knowledge, and `oneLayoutPerRole` (a seat with two screen documents becomes two roles with one audience; `share.addSeat` applies it).
  - `board.ts` gains `TablePort.seats` (`SeatPort`), made in `run/table-port.ts` from the share and session.
  - `brand.ts` gains `SeatBrand.accent` (#rrggbb) and `motto` (60 characters at most), checked, for the Atlas letterhead (AT6, asked by "Build star atlas and editions").
- Tars's shelf passage on sharing now describes roles instead of "More seats".

## Checks
- `seats.test.ts`: one new test (5 pass). `ui:check` and `ui:test` (605).
- table, embed and p2p e2e moved from the Seats dialog to the role card. They add a role, hold it while the table carries on, take its links, refuse a wildcard tabletop, check the brand and live Hide labels, approve a request from the role card, run fog-of-war discoveries for Vesh and a sealed Orun, and check the GM going offline.
- switchboard, steer, players, playbook, bookmarks, landing-table, party and known pass.
- Screenshots: project files `sse4/handover-screens/PB7-screens/`.

## Later
- Roles saved with the campaign (design 5.6). The plan is still tonight's only, as it was with the Seats dialog.
- A request's badge on the screen that asked, not only the role. The relay does not yet say which screen sent it.
