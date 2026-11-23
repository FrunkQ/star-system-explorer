# V3 gap 26: a player view inside a virtual tabletop

Owner: Traveller pack track. Re-specified on the V4 view contract: no special
embed screen, the seat's own join link with `&embed=1`.

## What it does

- A seat may list the tabletops it can sit inside (`Seat.embed_origins`):
  exact web origins, `https://` only (plain `http://` only for localhost or
  127.0.0.1, for testing). No wildcards, no paths, no user names.
  `embedOrigin` normalises one; `checkSeats` refuses a malformed or
  un-normalised entry; the Seats panel refuses one at the form with the reason.
- The Seats panel shows a **tabletop link** beside the join link of a seat that
  lists any. It is the join link plus `&embed=1`.
- `players.html` with `embed=1`:
  - not framed: the ordinary page (a link opened by mistake still works);
  - framed by a listed tabletop: the view alone, with no header, ask box,
    sound bar or status line (the footer stays in the DOM, hidden, for tests);
  - framed by anything else, or the seat lists none, or the browser gives no
    framing origin: refused, with the reason in words (`embedMode`).
  The framing origin is `location.ancestorOrigins[0]` where the browser has it
  (Chromium, WebKit), else the referrer's origin (Firefox).
- Bridge (`EmbedBridge`, v1): posts only to `window.parent` with the exact
  origin as target. Out: `{source:"sse4", v:1, type:"ready", seat, audience}`
  and `{type:"focus", address, name}` when focus changes. In: only
  `{source:"vtt", type:"focus", address}`, only from that origin and from
  `window.parent`, and only for a subject the seat's scene already holds.
  Nothing else is heard; no intent travels through the bridge.

## Files

- `packages/ui-contract/src/embed.ts`; `Seat.embed_origins` in `seat.ts`;
  `addSeat(..., {embedOrigins})` and the check in `seats.ts`.
- `packages/shell/src/Players.svelte` (embedded mode, refusal, bridge);
  `packages/shell/src/SeatsPanel.svelte` (the field and the tabletop link).
- Tests: `tests/ui/embed.test.ts` (origins, modes, bridge, seat check) and
  `tests/ui/embed.e2e.mjs` (a page on 127.0.0.1 frames the app on localhost:
  embedded, ready heard, focus followed; an unlisted seat refused).

## Limits and next

- The site sends no `frame-ancestors` header: the list lives in the GM's
  session, not on the server, so the refusal is in the page. A seat's view only
  ever carries what its scene allows, framed or not.
- The e2e turns off Chromium's third-party storage partitioning so the framed
  player can reach the GM's room through the in-browser channel; through the
  site's relay (other devices) no such switch is needed.
- Next: a "subject clicked" message for tabletops that place tokens, once a
  tabletop asks for one.
