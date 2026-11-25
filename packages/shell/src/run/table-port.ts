// The table as the Switchboard sees it (PB2, ui-contract `TablePort`), made from the shell's share. PB7 adds the
// roles themselves for the role card (`seats`), which replaced the Seats dialog.
import { fogViews, observeOf, type SceneSubject, type SeatPort, type Shown, type TablePort, type ViewSession } from "../../../ui-contract/src/index.ts";
import type { Share } from "../share.svelte.ts";

export interface SeatContext {
  session: ViewSession;
  /** The bodies a fog-of-war discovery can be marked on: the star and what orbits it. */
  bodies: () => readonly SceneSubject[];
  /** The scope the scene is read in for an audience's discoveries, and the date a discovery is marked at. */
  scope: () => string;
  at_yr: () => number;
  /** A change the table made (an approved request, a discovery): the physics moved. */
  onchange: () => void;
}

export function tablePort(share: Share, live: () => Shown, seatCtx?: SeatContext): TablePort {
  return {
    get state() {
      const st = share.state;
      return {
        board: st.board,
        roles: st.plan.seats.filter((x) => x.authority !== "gm").map((x) => ({ id: x.id, label: x.id === "table" ? "Table" : x.label, sealed: x.sealed, audience: x.audience })),
        players: st.players,
        link: st.link,
        can_undo: st.can_undo,
        can_redo: st.can_redo,
        waiting: st.waiting.length,
        cards: share.cards,
      };
    },
    act: (f) => share.act(f),
    undo: () => share.undo(),
    redo: () => share.redo(),
    start: () => share.start(live()),
    links: (role) => share.links(role),
    live,
    ...(seatCtx ? { seats: seatPort(share, seatCtx) } : {}),
  };
}

function seatPort(share: Share, c: SeatContext): SeatPort {
  const words = (r: Share["state"]["waiting"][number]) => {
    const i = r.intent as { kind: string; field?: string; address?: string; value?: unknown };
    if (i.kind !== "set_value") return i.kind;
    const who = c.bodies().find((b) => b.address === i.address)?.label ?? i.address ?? "";
    return `${i.field} of ${who} → ${JSON.stringify(i.value)}`;
  };
  return {
    plan: () => share.state.plan,
    fog: () => fogViews(observeOf(c.session.stack().base)?.setup),
    add: async (o) => {
      const bad = await share.addSeat(o);
      if (!bad) c.onchange();
      return bad;
    },
    update: (id, change) => share.updateSeat(id, change),
    remove: (id) => share.removeSeat(id),
    setLive: (id, l) => share.setLive(id, l),
    discover: async (audience, address, known) => {
      const a = share.state.plan.audiences.find((x) => x.id === audience);
      if (!a) return "no such audience";
      const bad = await share.discover(a, address, known, c.at_yr());
      c.onchange();
      return bad;
    },
    discovered: async (audience) => {
      const r = await c.session.scene(c.scope(), { audience });
      return r.ok ? r.scene.subjects.map((s) => s.address) : [];
    },
    bodies: () => c.bodies().map((b) => ({ address: b.address, label: b.label })),
    requests: () => share.state.waiting.map((r) => ({ id: r.id, from: r.from, seat: r.seat, words: words(r) })),
    answer: async (id, from, approve) => {
      if (approve) {
        await share.host.approve(id, from);
        c.onchange();
      } else share.host.decline(id, from);
    },
  };
}
