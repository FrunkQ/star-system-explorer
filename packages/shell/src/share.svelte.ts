// Sharing a session with players (V4): the GM shell records every applied
// intent from the moment the campaign opens, and serves players once the GM
// shares. A room is a random id; the players' link opens players.html with it.
// The GM serves both at once: BroadcastChannel for other windows and screens
// of this computer, and the site's relay (worker/index.js) for other devices.
import { BoardHistory, CardDeck, GROUND_TURN_MS, campaignGrounds, groundCandidates, groundTurns, holdGround, pickGround, stampFor, type Bookmark, type GroundSet, type HoldCard, boardWire, holdingCardsOn, driveTo, emptyBoard, hold, rejoin, release, screenHello, send as boardSend, type Board } from "../../ui-contract/src/index.ts";
import { GmHost, SealedHost, sealedLink, observeOf, addSeat, updateSeat, oneLayoutPerRole, type SeatChange, broadcastHub, defaultPlan, discoverIntent, joinLink, multiHub, relayHub, removeSeat, startIntents, type Audience, type HostState, type Hub, type Json, type PlayerRequest, type SeatPlan, type Session, type Shown, type PackDoc, type HoldingCard } from "../../ui-contract/src/index.ts";

declare const __SSE_APP__: string;
export const APP_VERSION: string = typeof __SSE_APP__ === "string" ? __SSE_APP__ : "dev";

export interface Share {
  readonly host: GmHost;
  /** The holding cards on the stack (PB5), for the line card's choices. */
  readonly cards: readonly HoldingCard[];
  readonly state: HostState & { room: string | null; link: string | null; waiting: PlayerRequest[]; plan: SeatPlan; shown: Shown; board: Board; can_undo: boolean; can_redo: boolean };
  /** Change the Switchboard's board (PB1): `f` gets the board and returns the next; it goes on the board's undo history and out to every screen. */
  act(f: (b: Board) => Board): void;
  /** The board's own Undo and Redo (tonight's changes to what screens show; never a reveal). */
  undo(): void;
  redo(): void;
  /** Where the GM is now goes to the destination being driven (PB3b), if any; not a step on the board's history. */
  follow(live: Shown): void;
  /** Share, showing the table `first` (the place in view when the GM shares). */
  start(first?: Shown): string;
  /** Send the table a place, or turn their screens off. Nothing reaches them until it is sent. Every seat follows: a seat's own send is cleared. */
  show(s: Shown): void;
  /** Send one seat a place, or turn its screens off (Looks 4D seat rows); null puts it back with the table. */
  showSeat(id: string, s: Shown | null): void;
  setClock(yr: number): void;
  /** V10: add a seat with its own audience; a fog-of-war audience's starting view is chronicled at once. */
  addSeat(o: Parameters<typeof addSeat>[1]): Promise<string | null>;
  removeSeat(id: string): void;
  /** The role card (PB7): change a seat's name, layout, names, brand or tabletops; its screens get it at once. */
  updateSeat(id: string, change: SeatChange): void;
  /** Quick overrides (V3 gap 25): set a seat's live switches now; they go to its screens at once and are never saved. */
  setLive(id: string, live: Record<string, boolean>): void;
  /** Mark a body (and what orbits it) discovered by an audience, or take it back. */
  discover(audience: Audience, address: string, known: boolean, at_yr: number): Promise<string | null>;
  /** Join links of a seat: one per screen it opens. */
  links(seatId: string): { screen: string | null; href: string }[];
  /** Where held screens' grounds come from (PC5): the campaign's bookmarks, and the postcard sets on the stack. */
  setGrounds(src: { bookmarks: () => readonly Bookmark[]; sets?: () => readonly GroundSet[] }): void;
}

function roomId(): string {
  const b = new Uint8Array(6);
  crypto.getRandomValues(b);
  return [...b].map((x) => x.toString(36).padStart(2, "0")).join("").slice(0, 10);
}

export async function createShare(session: Session, opening: { universe: { id: string; seed: string }; packs: unknown[]; opening_export?: Json; clock_yr: number; views: () => string[]; scope: string }, hubFor: (room: string, id: string) => Hub = (room, id) => multiHub(id, [broadcastHub(room, id), relayHub(document.baseURI, room, id)])): Promise<Share> {
  const run = roomId();
  const host = new GmHost(session, null, { run, kernel: await session.kernelVersion(), app: APP_VERSION, universe: opening.universe, packs: opening.packs, ...(opening.opening_export !== undefined ? { opening_export: opening.opening_export } : {}), opening_hash: await session.stateHash(), clock_yr: opening.clock_yr, seats: defaultPlan().seats });
  const hist = new BoardHistory(emptyBoard({ scope: opening.scope }));
  const st = $state({ ...host.state(), room: null as string | null, link: null as string | null, waiting: [] as PlayerRequest[], plan: defaultPlan(), shown: { scope: opening.scope } as Shown, board: hist.board, can_undo: false, can_redo: false });
  const root = `u:${opening.universe.id}`;
  const failed = (r: Awaited<ReturnType<Session["send"]>>) => (r.status === "applied" ? null : r.status === "refused" ? r.reason : r.status === "error" ? r.message : "not applied");
  host.onChange = (s) => Object.assign(st, s, { waiting: [...host.requests()] });
  let timer: ReturnType<typeof setInterval> | null = null;
  // Holding cards (PB5): the base presentation pack's (served beside the shell by scripts/build-shell.sh; not on the
  // stack, so no state hash changes), then any the stack's packs add or replace. The deck picks words once per hold.
  const presentation = (await fetch(document.baseURI.replace(/[^/]*$/, "") + "packs/base-presentation.json")
    .then((r) => (r.ok ? r.json() : null))
    .catch(() => null)) as PackDoc | null;
  const cards = holdingCardsOn([...(presentation ? [presentation] : []), ...session.stack().base]);
  const deck = new CardDeck(cards);
  // V10 part 2: each sealed seat has its own room, served finished scenes only.
  const sealed = new Map<string, { host: SealedHost; room: string }>();
  let clock = opening.clock_yr;
  function serveSealed() {
    if (!st.room) return;
    for (const seat of st.plan.seats) {
      if (!seat.sealed || sealed.has(seat.id)) continue;
      const room = roomId();
      sealed.set(seat.id, { room, host: new SealedHost(session, host, hubFor(room, "gm"), { seat: $state.snapshot(seat) as typeof seat, scope: "scope" in st.shown ? st.shown.scope : opening.scope, app: APP_VERSION, clock_yr: clock, data_views: opening.views() }) });
    }
  }
  // Grounds on held screens (PC5): which postcard each role's held screens show. A campaign postcard goes only to a
  // role that knows the place it looks at; what a role knows is asked of the kernel once and kept until the board
  // changes or the postcards turn. Postcards in turn move on every GROUND_TURN_MS while a held card has a set.
  let groundSrc: { bookmarks: () => readonly Bookmark[]; sets?: () => readonly GroundSet[] } | null = null;
  let turn = 0;
  const knownAt = new Map<string, Set<string> | "asking">();
  const audienceFor = (role: string) => (role === "" ? "everyone" : (st.plan.seats.find((x) => x.id === role)?.audience ?? "everyone"));
  function knows(audience: string, scope: string, about: string): boolean | null {
    if (audience === "all") return true;
    const k = `${audience}|${scope}`;
    const had = knownAt.get(k);
    if (had === "asking") return null;
    if (had) return had.has(about) || about === scope;
    knownAt.set(k, "asking");
    void session
      .scene(scope, { audience })
      .then((r) => knownAt.set(k, new Set(r.ok ? r.scene.subjects.map((x) => x.address) : [])))
      .catch(() => knownAt.set(k, new Set()))
      .then(() => push());
    return null;
  }
  function groundFor(card: HoldCard, role: string) {
    if (!groundSrc) return null;
    const list = groundCandidates(holdGround(card), { campaign: campaignGrounds(groundSrc.bookmarks(), stampFor), sets: groundSrc.sets?.() ?? [] });
    const audience = audienceFor(role);
    return pickGround(list, turn, (scope, about) => knows(audience, scope, about));
  }
  const turning = () => {
    const b = hist.board;
    return [b.all, ...Object.values(b.lines).map((l) => l?.held)].some((c) => c !== undefined && groundTurns(holdGround(c)));
  };
  setInterval(() => {
    if (!turning()) return;
    turn += 1;
    knownAt.clear();
    push();
  }, GROUND_TURN_MS);
  // The board goes out as the Shown players' screens already read: the table's, a seat's own when its role has a line
  // of its own, and each known screen's own. Sealed seats get theirs built on this machine (a held one gets nothing).
  function push() {
    const b = hist.board;
    const plan = $state.snapshot(st.plan) as SeatPlan;
    deck.settle(b);
    const wire = boardWire(b, plan.seats.map((x) => x.id), (c) => deck.said(c), groundFor);
    const seats = plan.seats.map(({ shown: _old, ...rest }) => (b.lines[`role:${rest.id}`] || b.steer?.[`role:${rest.id}`] ? { ...rest, shown: wire.roles[rest.id]! } : rest));
    st.plan = { ...plan, seats };
    st.shown = wire.shown;
    st.board = b;
    st.can_undo = hist.canUndo;
    st.can_redo = hist.canRedo;
    if (!st.link) return;
    host.setState({ shown: wire.shown, seats, screens: wire.screens });
    sealed.forEach((x, id) => void x.host.setShown(wire.roles[id] ?? wire.shown));
  }
  host.onScreen = (h) => {
    hist.board = screenHello(hist.board, h.id, h);
    push();
  };
  const act = (f: (b: Board) => Board) => {
    knownAt.clear();
    hist.do(f(hist.board));
    push();
  };
  return {
    host,
    cards,
    state: st,
    act,
    undo() {
      hist.undo();
      push();
    },
    redo() {
      hist.redo();
      push();
    },
    follow(live: Shown) {
      if (!hist.board.driving) return;
      const next = driveTo(hist.board, JSON.parse(JSON.stringify(live)) as Shown);
      if (JSON.stringify(next) === JSON.stringify(hist.board)) return;
      hist.board = next;
      push();
    },
    start(first?: Shown) {
      if (st.link) return st.link;
      if (first) hist.board = emptyBoard(JSON.parse(JSON.stringify(first)) as Shown);
      st.shown = hist.board.lines.everyone!.source;
      host.setState({ shown: JSON.parse(JSON.stringify(st.shown)) as Shown });
      const room = roomId();
      host.attach(hubFor(room, "gm"));
      timer ??= setInterval(() => (host.beat(), sealed.forEach((x) => x.host.beat())), 2000);
      addEventListener("pagehide", () => (host.close(), sealed.forEach((x) => x.host.close())));
      // Data-view state: the players' screens may look through the campaign's data views.
      host.setState({ data_views: opening.views() });
      st.room = room;
      st.link = joinLink(document.baseURI, room);
      serveSealed();
      push();
      return st.link;
    },
    show(s: Shown) {
      // The table's line, as before the Switchboard: every role and screen goes back with the table (PB2 turns
      // Everyone into a layer that lifts, with its own controls).
      act((b) => {
        const place = JSON.parse(JSON.stringify(s)) as Shown;
        const { all: _all, ...rest } = b;
        const floor: Board = { ...rest, over: [], lines: { everyone: { source: "off" in place ? (b.lines.everyone?.source ?? place) : place } } };
        return "off" in place ? hold(floor, "everyone", place.card ? { text: place.card } : "blank") : floor;
      });
    },
    showSeat(id: string, s: Shown | null) {
      act((b) => {
        if (!s) return rejoin(b, `role:${id}`);
        const place = JSON.parse(JSON.stringify(s)) as Shown;
        if ("off" in place) return hold(b, `role:${id}`, place.card ? { text: place.card } : "blank");
        return boardSend(release(b, `role:${id}`), `role:${id}`, place);
      });
    },
    setClock(yr: number) {
      clock = yr;
      host.setState({ clock_yr: yr });
      sealed.forEach((x) => x.host.setClock(yr));
    },
    async addSeat(o) {
      const next = oneLayoutPerRole(addSeat($state.snapshot(st.plan) as SeatPlan, o));
      const added = next.audiences.filter((a) => !st.plan.audiences.some((b) => b.id === a.id));
      for (const i of startIntents({ ...next, audiences: added }, root, opening.clock_yr)) {
        const bad = failed(await session.send(i));
        if (bad) return bad;
      }
      st.plan = next;
      host.setState({ seats: next.seats });
      serveSealed();
      return null;
    },
    setLive(id: string, live: Record<string, boolean>) {
      const plan = $state.snapshot(st.plan) as SeatPlan;
      const seats = plan.seats.map((s) => {
        if (s.id !== id) return s;
        const { live: _old, ...rest } = s;
        return Object.keys(live).length ? { ...rest, live: { ...live } } : rest;
      });
      st.plan = { ...plan, seats };
      host.setState({ seats });
    },
    updateSeat(id: string, change: SeatChange) {
      const next = updateSeat($state.snapshot(st.plan) as SeatPlan, id, change);
      st.plan = next;
      host.setState({ seats: next.seats });
    },
    removeSeat(id: string) {
      const next = removeSeat($state.snapshot(st.plan) as SeatPlan, id);
      sealed.get(id)?.host.close();
      sealed.delete(id);
      st.plan = next;
      host.setState({ seats: next.seats });
    },
    async discover(audience, address, known, at_yr) {
      return failed(await session.send(discoverIntent(audience, address, known, observeOf(session.stack().base)?.setup.campaign.before ?? "long-range", at_yr)));
    },
    setGrounds(src) {
      groundSrc = src;
      knownAt.clear();
      push();
    },
    links(seatId: string) {
      const seat = st.plan.seats.find((x) => x.id === seatId);
      if (!st.room || !seat) return [];
      const screens = seat.screens?.length ? seat.screens : [null];
      const own = sealed.get(seat.id);
      return screens.map((screen) => ({ screen, href: own ? sealedLink(document.baseURI, own.room, seat.id, screen ?? undefined) : joinLink(document.baseURI, st.room!, seat.id, screen ?? undefined) }));
    },
  };
}
