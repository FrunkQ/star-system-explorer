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
import type { Intent } from "./intent.ts";

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
}

export type Wire =
  | { t: "hello"; from: string; protocol: string }
  | { t: "manifest"; to: string; manifest: Manifest }
  | { t: "event"; run: string; entry: LogEntry }
  | { t: "state"; run: string; clock_yr: number; data_views: string[] }
  | { t: "beat"; run: string; seq: number; state_hash: string }
  | { t: "snapshot"; from: string }
  | { t: "bye"; run: string };

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

export interface HostState {
  run: string;
  seq: number;
  state_hash: string;
  players: number;
}

/**
 * The GM side: records every intent the GM's session applies from the moment
 * it is made (make it right after opening, before entering any facts), and
 * serves players once a room is attached. `beat()` is called on a timer.
 */
export class GmHost {
  readonly #session: Session;
  #hub: Hub | null = null;
  readonly #base: Omit<Manifest, "log" | "state_hash" | "clock_yr" | "data_views">;
  readonly #log: LogEntry[] = [];
  #opening_hash: string;
  #clock: number;
  #views: string[];
  readonly #players = new Set<string>();
  readonly #off: (() => void)[] = [];
  onChange: (s: HostState) => void = () => {};

  constructor(session: Session, hub: Hub | null, base: { run: string; kernel: string; app: string; universe: { id: string; seed: string }; packs: unknown[]; opening_hash: string; clock_yr: number; data_views?: string[] }) {
    this.#session = session;
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
        }
      }),
    );
  }

  #changed() {
    this.onChange(this.state());
  }

  state(): HostState {
    return { run: this.#base.run, seq: this.#log.length, state_hash: this.#hash(), players: this.#players.size };
  }

  #hash(): string {
    return this.#log.length ? this.#log[this.#log.length - 1]!.state_hash : this.#opening_hash;
  }

  manifest(): Manifest {
    return { ...structuredClone(this.#base), log: structuredClone(this.#log), state_hash: this.#hash(), clock_yr: this.#clock, data_views: [...this.#views] };
  }

  /** The session clock and the data views players' screens may use. */
  setState(s: { clock_yr?: number; data_views?: string[] }) {
    if (s.clock_yr !== undefined) this.#clock = s.clock_yr;
    if (s.data_views) this.#views = [...s.data_views];
    this.#hub?.send({ t: "state", run: this.#base.run, clock_yr: this.#clock, data_views: [...this.#views] });
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
  onStatus: (s: ReplicaStatus) => void = () => {};
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
    this.#lastHeard = this.#now();
    this.onSession(s);
    this.#live();
    this.onUpdate();
  }
}
