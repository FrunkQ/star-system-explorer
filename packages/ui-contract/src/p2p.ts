// Peer-to-peer play (V4, H9.2). The GM is the single writer. Over a hub (one
// room: same-browser BroadcastChannel, or WebRTC data channels joined through
// the v4 Worker) the GM sends the run manifest (universe, opening packs,
// kernel and app versions), every authored event as it happens (the intent it
// applied, its event id and the state hash after it), the session clock, the
// data views the players' screens may use, and a heartbeat with the state
// hash. Players run the same kernel (Class A, bit-exact) and replay each
// intent locally, so their state hash matches the GM's at every step. A
// player whose versions differ is told to reload; one whose hash differs asks
// for a snapshot and rebuilds; when the GM goes quiet, the player's views go
// read-only on the last state with a banner.
//
// Nothing here measures wall-clock time itself: the host passes `now()`.
import { APPLIED, GRANT_VIEWS, REPLAY, type Session } from "./session.ts";
import { checkIntent, type Intent } from "./intent.ts";
import { decide, type Seat } from "./seat.ts";

export const P2P_PROTOCOL = "sse.p2p/1";

/** One authored event as it travels: the intent the GM applied and what the kernel said. */
export interface LogEntry {
  seq: number;
  intent: Intent;
  event_id: string;
  state_hash: string;
}

export interface Manifest {
  protocol: typeof P2P_PROTOCOL;
  /** One GM session; a new id means a new run (players start again). */
  run: string;
  kernel: string;
  app: string;
  universe: { id: string; seed: string };
  /** The packs the GM opened with, bottom to top (later stack changes travel as events). */
  packs: unknown[];
  log: LogEntry[];
  /** State hash after the last entry (or after opening, with no entries). */
  state_hash: string;
  clock_yr: number;
  data_views: string[];
  /** The seats of this session (V5: what each may ask for; the table is "table"). */
  seats: Seat[];
}

export type Wire =
  | { t: "hello"; from: string; protocol: string }
  | { t: "manifest"; to: string; manifest: Manifest }
  | { t: "event"; run: string; entry: LogEntry }
  /** Session state for screens: the clock, the data views granted, and (V10) the seats when the GM changes them. */
  | { t: "state"; run: string; clock_yr: number; data_views: string[]; seats?: Seat[] }
  | { t: "beat"; run: string; seq: number; state_hash: string }
  | { t: "snapshot"; from: string }
  | { t: "bye"; run: string }
  /** V5: a player's request, by seat; it becomes an event only if the GM approves it. */
  | { t: "request"; run: string; from: string; seat: string; id: string; intent: Intent }
  | { t: "answer"; run: string; to: string; id: string; status: "approved" | "declined"; reason?: string; event_id?: string };

/** A player's request as the GM sees it. */
export interface PlayerRequest {
  id: string;
  from: string;
  seat: string;
  intent: Intent;
}

/** A player's own request as that player sees it. */
export interface OwnRequest {
  id: string;
  intent: Intent;
  status: "waiting" | "approved" | "declined";
  reason?: string;
}

/** A room: every member hears every message (messages may name one recipient with `to`). */
export interface Hub {
  readonly id: string;
  send(m: Wire): void;
  onMessage(cb: (m: Wire) => void): () => void;
  /** Peers seen in the room (players); the GM host uses it for its count. */
  close(): void;
}

/** Two or more hubs wired together in memory (tests). */
export function memoryRoom(): { hub(id: string): Hub; drop(id: string): void } {
  const members = new Map<string, ((m: Wire) => void)[]>();
  const dropped = new Set<string>();
  const deliver = (from: string, m: Wire) => {
    if (dropped.has(from)) return;
    const copy = JSON.parse(JSON.stringify(m)) as Wire;
    for (const [id, cbs] of [...members.entries()]) if (id !== from && !dropped.has(id)) queueMicrotask(() => cbs.forEach((cb) => cb(copy)));
  };
  return {
    hub(id: string): Hub {
      members.set(id, []);
      return {
        id,
        send: (m) => deliver(id, m),
        onMessage(cb) {
          members.get(id)!.push(cb);
          return () => members.set(id, (members.get(id) ?? []).filter((x) => x !== cb));
        },
        close: () => void members.delete(id),
      };
    },
    drop: (id) => void dropped.add(id),
  };
}

/** Every window and screen of this browser in one room (multiscreen, role stations, tests). */
export function broadcastHub(room: string, id: string): Hub {
  const ch = new BroadcastChannel(`sse4.room.${room}`);
  return {
    id,
    send: (m) => ch.postMessage(m),
    onMessage(cb) {
      const f = (e: MessageEvent) => cb(e.data as Wire);
      ch.addEventListener("message", f);
      return () => ch.removeEventListener("message", f);
    },
    close: () => ch.close(),
  };
}

/**
 * A room through the site's signalling Worker (worker/index.js): a WebSocket
 * to /signal/<room>, relayed by the room's Durable Object, which stores
 * nothing. Reconnects with backoff; messages sent while closed wait.
 * `opened` settles true on the first connection, false if none within
 * `timeoutMs` (the caller may then use a same-browser room instead).
 */
export function relayHub(site: string, room: string, id: string, opts: { timeoutMs?: number } = {}): Hub & { opened: Promise<boolean> } {
  const url = new URL(`signal/${room}?peer=${encodeURIComponent(id)}`, site);
  url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
  const cbs: ((m: Wire) => void)[] = [];
  const waiting: string[] = [];
  let ws: WebSocket | null = null;
  let closed = false;
  let delay = 500;
  let settle: (ok: boolean) => void = () => {};
  const opened = new Promise<boolean>((r) => (settle = r));
  const timer = setTimeout(() => settle(false), opts.timeoutMs ?? 4000);
  const connect = () => {
    if (closed) return;
    const s = new WebSocket(url);
    ws = s;
    s.onopen = () => {
      delay = 500;
      clearTimeout(timer);
      settle(true);
      for (const m of waiting.splice(0)) s.send(m);
    };
    s.onmessage = (e) => {
      try {
        const m = JSON.parse(String(e.data)) as Wire & { from?: string };
        if (m && typeof m === "object" && typeof m.t === "string") cbs.forEach((cb) => cb(m));
      } catch {
        // not ours
      }
    };
    s.onclose = () => {
      ws = null;
      if (!closed) setTimeout(connect, (delay = Math.min(delay * 2, 15000)));
    };
  };
  connect();
  return {
    id,
    opened,
    send(m) {
      const text = JSON.stringify(m);
      if (ws && ws.readyState === 1) ws.send(text);
      else waiting.push(text);
    },
    onMessage(cb) {
      cbs.push(cb);
      return () => cbs.splice(cbs.indexOf(cb), 1);
    },
    close() {
      closed = true;
      clearTimeout(timer);
      ws?.close();
    },
  };
}

/** Several rooms as one (the GM serves this browser's screens and the relay at once). */
export function multiHub(id: string, hubs: Hub[]): Hub {
  return {
    id,
    send: (m) => hubs.forEach((h) => h.send(m)),
    onMessage(cb) {
      const offs = hubs.map((h) => h.onMessage(cb));
      return () => offs.forEach((f) => f());
    },
    close: () => hubs.forEach((h) => h.close()),
  };
}

export interface HostState {
  run: string;
  seq: number;
  state_hash: string;
  players: number;
  requests: number;
}

/**
 * The GM side: records every intent the GM's session applies from the moment
 * it is made (make it right after opening, before entering any facts), and
 * serves players once a room is attached. `beat()` is called on a timer.
 */
export class GmHost {
  readonly #session: Session;
  #hub: Hub | null = null;
  readonly #base: Omit<Manifest, "log" | "state_hash" | "clock_yr" | "data_views" | "seats">;
  readonly #log: LogEntry[] = [];
  #opening_hash: string;
  #clock: number;
  #views: string[];
  readonly #players = new Set<string>();
  readonly #off: (() => void)[] = [];
  readonly #seats: Seat[];
  readonly #requests: PlayerRequest[] = [];
  onChange: (s: HostState) => void = () => {};

  constructor(session: Session, hub: Hub | null, base: { run: string; kernel: string; app: string; universe: { id: string; seed: string }; packs: unknown[]; opening_hash: string; clock_yr: number; data_views?: string[]; seats?: Seat[] }) {
    this.#session = session;
    this.#seats = structuredClone(base.seats ?? []);
    this.#hub = hub;
    this.#base = { protocol: P2P_PROTOCOL, run: base.run, kernel: base.kernel, app: base.app, universe: base.universe, packs: base.packs };
    this.#opening_hash = base.opening_hash;
    this.#clock = base.clock_yr;
    this.#views = [...(base.data_views ?? [])];
    this.#off.push(
      session[APPLIED]((intent, r) => {
        const entry: LogEntry = { seq: this.#log.length + 1, intent: structuredClone(intent), event_id: r.event_id, state_hash: r.state_hash };
        this.#log.push(entry);
        this.#hub?.send({ t: "event", run: base.run, entry });
        this.#changed();
      }),
    );
    if (hub) this.attach(hub);
  }

  /** Start serving players in a room (more than one room may be attached in turn; the last one serves). */
  attach(hub: Hub) {
    this.#hub = hub;
    this.#off.push(
      hub.onMessage((m) => {
        if (m.t === "hello" || m.t === "snapshot") {
          this.#players.add(m.from);
          hub.send({ t: "manifest", to: m.from, manifest: this.manifest() });
          this.#changed();
        } else if (m.t === "request" && m.run === this.#base.run) this.#request(m);
      }),
    );
  }

  #changed() {
    this.onChange(this.state());
  }

  #answer(r: PlayerRequest, status: "approved" | "declined", extra: { reason?: string; event_id?: string } = {}) {
    this.#hub?.send({ t: "answer", run: this.#base.run, to: r.from, id: r.id, status, ...extra });
  }

  /** A player's request: checked against the seat the GM gave it, then queued for the GM (never applied here). */
  #request(m: Extract<Wire, { t: "request" }>) {
    const r: PlayerRequest = { id: m.id, from: m.from, seat: m.seat, intent: m.intent };
    if (this.#requests.some((x) => x.from === r.from && x.id === r.id)) return;
    const seat = this.#seats.find((x) => x.id === m.seat);
    const bad = checkIntent(m.intent);
    const d = seat && !bad ? decide(seat, m.intent) : null;
    const why = !seat ? `no seat ${m.seat} in this session` : bad ? bad : d!.action === "refuse" ? (d as { reason: string }).reason : null;
    if (why) return this.#answer(r, "declined", { reason: why });
    this.#requests.push(r);
    // V10: what the seat's auto-approve scopes cover is applied at once, through the GM's own session.
    if (d!.action === "apply") return void this.approve(r.id, r.from);
    this.#changed();
  }

  /** Requests waiting for the GM, oldest first. */
  requests(): readonly PlayerRequest[] {
    return structuredClone(this.#requests);
  }

  /** The GM approves: the intent is applied through the GM's own session (and so travels to every player as an event). */
  async approve(id: string, from: string): Promise<void> {
    const i = this.#requests.findIndex((x) => x.id === id && x.from === from);
    if (i < 0) return;
    const [r] = this.#requests.splice(i, 1);
    const res = await this.#session.send(r!.intent);
    if (res.status === "applied") this.#answer(r!, "approved", { event_id: res.event_id });
    else this.#answer(r!, "declined", { reason: res.status === "refused" ? res.reason : res.status === "error" ? res.message : "the GM's session could not apply it" });
    this.#changed();
  }

  decline(id: string, from: string, reason = "The game master declined."): void {
    const i = this.#requests.findIndex((x) => x.id === id && x.from === from);
    if (i < 0) return;
    const [r] = this.#requests.splice(i, 1);
    this.#answer(r!, "declined", { reason });
    this.#changed();
  }

  state(): HostState {
    return { run: this.#base.run, seq: this.#log.length, state_hash: this.#hash(), players: this.#players.size, requests: this.#requests.length };
  }

  #hash(): string {
    return this.#log.length ? this.#log[this.#log.length - 1]!.state_hash : this.#opening_hash;
  }

  manifest(): Manifest {
    return { ...structuredClone(this.#base), log: structuredClone(this.#log), state_hash: this.#hash(), clock_yr: this.#clock, data_views: [...this.#views], seats: structuredClone(this.#seats) };
  }

  /** The session clock, the data views players' screens may use and (V10) the seats. */
  setState(s: { clock_yr?: number; data_views?: string[]; seats?: Seat[] }) {
    if (s.clock_yr !== undefined) this.#clock = s.clock_yr;
    if (s.data_views) this.#views = [...s.data_views];
    if (s.seats) this.#seats.splice(0, this.#seats.length, ...structuredClone(s.seats));
    this.#hub?.send({ t: "state", run: this.#base.run, clock_yr: this.#clock, data_views: [...this.#views], ...(s.seats ? { seats: structuredClone(this.#seats) } : {}) });
  }

  seats(): readonly Seat[] {
    return structuredClone(this.#seats);
  }

  beat() {
    this.#hub?.send({ t: "beat", run: this.#base.run, seq: this.#log.length, state_hash: this.#hash() });
  }

  close() {
    this.#hub?.send({ t: "bye", run: this.#base.run });
    this.#off.forEach((f) => f());
  }
}

export type ReplicaStatus =
  /** Waiting for the GM's manifest. */
  | { state: "joining" }
  /** In step with the GM. */
  | { state: "live"; seq: number; state_hash: string; clock_yr: number }
  /** Rebuilding from a snapshot after a mismatch or a gap. */
  | { state: "resync"; reason: string }
  /** The GM has gone quiet: read-only on the last state. */
  | { state: "offline"; seq: number; state_hash: string; since: number }
  /** This app or kernel differs from the GM's: reload, never drift. */
  | { state: "reload"; reason: string };

/**
 * The player side. `open(manifest)` starts a fresh kernel session with the
 * manifest's universe and packs (a player seat); the replica replays the log
 * through it and keeps it in step.
 */
export class PlayerReplica {
  readonly #hub: Hub;
  readonly #open: (m: Manifest) => Promise<Session>;
  readonly #versions: { kernel: string; app: string };
  readonly #now: () => number;
  readonly #offlineAfter: number;
  #session: Session | null = null;
  #run = "";
  #seq = 0;
  #hash = "";
  #clock = 0;
  #lastHeard = 0;
  #status: ReplicaStatus = { state: "joining" };
  #queue: Promise<void> = Promise.resolve();
  readonly #off: () => void;
  readonly #own: OwnRequest[] = [];
  #seats: Seat[] = [];
  #nextId = 1;
  onStatus: (s: ReplicaStatus) => void = () => {};
  /** The player's own requests changed (sent, approved, declined). */
  onRequests: (r: readonly OwnRequest[]) => void = () => {};
  /** A new session was built (on join or after a resync): views should rebind to it. */
  onSession: (s: Session) => void = () => {};
  /** The state moved (an event or the clock): views should refresh. */
  onUpdate: () => void = () => {};

  constructor(hub: Hub, open: (m: Manifest) => Promise<Session>, opts: { kernel: string; app: string; now: () => number; offlineAfterMs?: number }) {
    this.#hub = hub;
    this.#open = open;
    this.#versions = { kernel: opts.kernel, app: opts.app };
    this.#now = opts.now;
    this.#offlineAfter = opts.offlineAfterMs ?? 6000;
    this.#off = hub.onMessage((m) => this.#serial(() => this.#receive(m)));
  }

  get status(): ReplicaStatus {
    return this.#status;
  }
  get session(): Session | null {
    return this.#session;
  }

  join() {
    this.#hub.send({ t: "hello", from: this.#hub.id, protocol: P2P_PROTOCOL });
  }

  /**
   * Ask the GM for a change (V5). The seat decides locally first: what it may
   * never send is refused here; what needs the GM goes to the GM, and becomes
   * an event (replayed like any other) only if the GM approves. Not while the
   * GM is away: the queue is disabled then.
   */
  async request(intent: Intent): Promise<{ ok: true; id: string } | { ok: false; reason: string }> {
    if (this.#status.state !== "live" || !this.#session) return { ok: false, reason: "The game master is not connected; requests wait until they are." };
    // The seat decides first; even what its auto-approve covers is applied by the GM (the single writer), never here.
    const bad = checkIntent(intent);
    if (bad) return { ok: false, reason: bad };
    const seat = this.#seats.find((x) => x.id === this.#session!.seat.id) ?? (this.#session.seat as Seat);
    const d = decide(seat, intent);
    if (d.action === "refuse") return { ok: false, reason: d.reason };
    const id = `${this.#hub.id}-${this.#nextId++}`;
    this.#own.push({ id, intent: structuredClone(intent), status: "waiting" });
    this.#hub.send({ t: "request", run: this.#run, from: this.#hub.id, seat: this.#session.seat.id, id, intent });
    this.onRequests(structuredClone(this.#own));
    return { ok: true, id };
  }

  requests(): readonly OwnRequest[] {
    return structuredClone(this.#own);
  }

  /** The session's seats as the GM last sent them (V10). */
  seats(): readonly Seat[] {
    return structuredClone(this.#seats);
  }

  /** Called by the host on a timer: notices a GM that has gone quiet. */
  tick() {
    const s = this.#status;
    if (s.state === "live" && this.#now() - this.#lastHeard > this.#offlineAfter) this.#set({ state: "offline", seq: this.#seq, state_hash: this.#hash, since: this.#lastHeard });
  }

  close() {
    this.#off();
  }

  #serial(f: () => Promise<void>) {
    this.#queue = this.#queue.then(f, f).catch(() => undefined);
  }

  #set(s: ReplicaStatus) {
    this.#status = s;
    this.onStatus(s);
  }

  #live() {
    this.#set({ state: "live", seq: this.#seq, state_hash: this.#hash, clock_yr: this.#clock });
  }

  #heard() {
    this.#lastHeard = this.#now();
    if (this.#status.state === "offline") this.#live();
  }

  #resync(reason: string) {
    this.#set({ state: "resync", reason });
    this.#hub.send({ t: "snapshot", from: this.#hub.id });
  }

  async #receive(m: Wire) {
    if (this.#status.state === "reload") return;
    if (m.t === "manifest") {
      if (m.to !== this.#hub.id) return;
      return this.#load(m.manifest);
    }
    if (!("run" in m) || m.run !== this.#run || !this.#session) return;
    if (m.t === "answer") {
      if (m.to !== this.#hub.id) return;
      const r = this.#own.find((x) => x.id === m.id);
      if (r) {
        r.status = m.status;
        if (m.reason) r.reason = m.reason;
        this.onRequests(structuredClone(this.#own));
      }
      return;
    }
    if (m.t === "request") return;
    if (m.t === "bye") return this.#set({ state: "offline", seq: this.#seq, state_hash: this.#hash, since: this.#now() });
    this.#heard();
    if (this.#status.state === "resync") return;
    if (m.t === "event") {
      const e = m.entry;
      if (e.seq <= this.#seq) return;
      if (e.seq !== this.#seq + 1) return this.#resync(`missed events ${this.#seq + 1} to ${e.seq - 1}`);
      const r = await this.#session[REPLAY](e.intent);
      const got = r.status === "applied" ? r.state_hash : "";
      if (got !== e.state_hash) return this.#resync(`state hash after event ${e.seq} differs from the GM's`);
      this.#seq = e.seq;
      this.#hash = got;
      this.#live();
      this.onUpdate();
    } else if (m.t === "state") {
      this.#clock = m.clock_yr;
      this.#session[GRANT_VIEWS](m.data_views);
      if (m.seats) this.#seats = structuredClone(m.seats);
      this.#live();
      this.onUpdate();
    } else if (m.t === "beat") {
      if (m.seq > this.#seq) return this.#resync(`behind the GM (${this.#seq} of ${m.seq} events)`);
      if (m.seq === this.#seq && m.state_hash !== this.#hash) return this.#resync("state hash differs from the GM's");
      if (this.#status.state !== "live") this.#live();
    }
  }

  async #load(man: Manifest) {
    const mismatch = (kernel: string) =>
      this.#set({ state: "reload", reason: `The game master runs ${man.app} (kernel ${man.kernel}); this screen has ${this.#versions.app} (kernel ${kernel}). Reload to match.` });
    // An empty kernel version means "not known until a kernel starts": checked right after opening.
    if (man.protocol !== P2P_PROTOCOL || man.app !== this.#versions.app || (this.#versions.kernel && man.kernel !== this.#versions.kernel)) return mismatch(this.#versions.kernel || "unknown");
    const s = await this.#open(man);
    if (!this.#versions.kernel) {
      const k = await s.kernelVersion();
      if (k !== man.kernel) return mismatch(k);
    }
    for (const e of man.log) {
      const r = await s[REPLAY](e.intent);
      if (r.status !== "applied" || r.state_hash !== e.state_hash) {
        return this.#set({ state: "reload", reason: `Replaying the game master's event ${e.seq} gave a different state. Reload to try again.` });
      }
    }
    s[GRANT_VIEWS](man.data_views);
    const hash = man.log.length ? man.log[man.log.length - 1]!.state_hash : await s.stateHash();
    if (hash !== man.state_hash) return this.#set({ state: "reload", reason: "This screen's kernel opened the universe differently from the game master's. Reload to match." });
    this.#session = s;
    this.#run = man.run;
    this.#seq = man.log.length;
    this.#hash = hash;
    this.#clock = man.clock_yr;
    this.#seats = structuredClone(man.seats);
    this.#lastHeard = this.#now();
    this.onSession(s);
    this.#live();
    this.onUpdate();
  }
}
