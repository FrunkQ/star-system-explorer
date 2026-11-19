// The session: the one door from any interface to the kernel. Views and
// shells receive a ViewSession and never see the kernel client behind it.
// Only this file and host-browser.ts may import the kernel client (checked by
// tests/ui/boundary.test.ts).
import type { KernelClient } from "../../kernel-client/src/index.ts";
import { encodeRequest, type Json } from "./json.ts";
import { checkIntent, intentToRequest, secondsToYears, yearsToSeconds, type Intent } from "./intent.ts";
import { audienceOf, decide, type Seat } from "./seat.ts";
import { providerOf, type FlowDoc, type PackDoc } from "./physics.ts";
import type { PlayerScene } from "./player-scene.ts";
import type { WorldStackRead } from "./world.ts";
import type { SurfaceTile } from "./globe.ts";
import type { LifeView } from "./life.ts";
import type { Adopted, FormationRan, FormationRequest, ListedSystem, SystemResolved, V3Imported } from "./make.ts";
import type { LookedAhead, SystemSpec } from "./fate.ts";

export type IntentResult =
  /** For a stack change the kernel chronicles a `stack.adopt` event; `view` is the data view a scan_toggle left in force. */
  | { status: "applied"; event_id: string; state_hash: string; affected: string[]; view?: string | null }
  | { status: "pending"; intent: Intent }
  | { status: "refused"; reason: string }
  | { status: "error"; code: string; message: string };

export interface HistoryEvent {
  id: string;
  kind: string;
  at_yr: number;
  evidence: string;
  subject: string[];
}

/** One input to a derivation, as the kernel takes it (H5.2 quantities with units). */
export interface InputValue {
  value: number;
  unit?: string;
  source?: "catalogued" | "resolved" | "derived" | "authored" | "authored-impossible";
}

/** What to derive: a flow and its inputs. Also the address of a "why?". */
export interface DeriveRef {
  flow: string;
  inputs: Record<string, InputValue>;
}

export interface Derived {
  /** SI value. */
  value: number;
  dim: string;
  validity: string;
  fictional: boolean;
  publisher: string;
}

/** The kernel's explanation bundle (H6.5), as shown by a "why?". */
export interface Explanation {
  subject: string;
  value: { value: number; dim: number[] };
  formula_display: string;
  lenses: { curious?: string; science?: string; engine?: string };
  inputs: { port: string; quantity: string; source: string; value: { value: number; dim: number[] }; explanation_ref: string | null }[];
  params: Record<string, number>;
  /** Text, or text with an optional DOI or URL (see reference.ts). */
  references: import("./reference.ts").ReferenceIn[];
  validity: string;
  out_of_range: string[];
  publisher: Record<string, string>;
  fictional: boolean;
}

export interface DeviceEdge {
  tier: string;
  unitsPerSecond: number;
}

/** What a view or shell may do. Nothing here can reach the kernel another way. */
export interface ViewSession {
  readonly seat: Readonly<Seat>;
  readonly universe: Readonly<{ id: string; seed: string; root: string }>;
  send(intent: Intent): Promise<IntentResult>;
  history(address: string): Promise<HistoryEvent[]>;
  stateHash(at_yr?: number): Promise<string>;
  /**
   * What-if branches (U4, H6.3): a branch of the chronicle the GM edits
   * without touching the main line. GM seat only; players never see one.
   */
  openBranch(label: string): Promise<{ ok: true; branch: string } | { ok: false; message: string }>;
  /** Apply a set_value on a branch only. The main chronicle and its state hash do not move. */
  branchSend(branch: string, intent: Intent): Promise<IntentResult>;
  branchStateHash(branch: string, at_yr?: number): Promise<string>;
  /** Drop a branch and everything on it. */
  discardBranch(branch: string): Promise<boolean>;
  /** Read a V3 save into facts and a difference report; writes nothing (GM only). */
  importV3(save: string, opts?: { universe_id?: string; reconstruct?: number }): Promise<{ ok: true; result: V3Imported } | { ok: false; code: string; message: string }>;
  /** One system through the formation pipeline; writes nothing (GM only). */
  runFormation(req: FormationRequest): Promise<{ ok: true; result: FormationRan } | { ok: false; code: string; message: string }>;
  /** Age a system forward on a branch (T2); GM only. */
  lookAhead(system: SystemSpec, target_myr: number): Promise<{ ok: true; result: LookedAhead } | { ok: false; code: string; message: string }>;
  /** Keep a look-ahead as history, optionally moving the system's "now"; GM only. */
  commitLookAhead(branch: string, move_now_myr?: number, write_back?: boolean): Promise<{ ok: true; event_id: string; state_hash: string; aged_event?: string; aged_at_myr?: number } | { ok: false; code: string; message: string }>;
  /** Write an imported save or a formation run into the open universe as history (GM only). */
  adopt(source: { import_v3: string } | { formation: FormationRequest }, occurred_at_s?: number): Promise<{ ok: true; result: Adopted } | { ok: false; code: string; message: string }>;
  /** A body's world stack (P4) with the kernel's ladder and physics notes, by body address or fixture name. */
  readWorldStack(by: { body?: string; fixture?: string }): Promise<{ ok: true; result: WorldStackRead } | { ok: false; code: string; message: string }>;
  /** A world's six level-0 surface tiles (S1, at the GM's disclosure), through resolved_system. Heavy: about 8 MB and a second or more. */
  surfaceTiles(system: string, body: string): Promise<{ ok: true; tiles: SurfaceTile[]; refused: Json[]; regime?: string } | { ok: false; code: string; message: string }>;
  /** The open universe as a document to keep (U10, kernel `export_chronicle`); GM only. */
  exportChronicle(): Promise<{ ok: true; export: Json } | { ok: false; code: string; message: string }>;
  /** Replace the open universe with a kept one (U10, kernel `import_chronicle`). */
  importChronicle(doc: Json): Promise<{ ok: true; state_hash: string; events: number; stack_matches: boolean } | { ok: false; code: string; message: string }>;
  /** One world's life (U7, kernel `life_view`): a body once systems carry habitat graphs, or one of the example fixtures. GM only. */
  lifeView(by: { body?: string; fixture?: string }): Promise<{ ok: true; source: string; view: LifeView } | { ok: false; code: string; message: string }>;
  /** Every system in the open universe that has facts (kernel `systems`): adopted ones and authored ones. */
  listSystems(): Promise<{ ok: true; systems: ListedSystem[] } | { ok: false; code: string; message: string }>;
  /** Candidate courses (V8, kernel `plan_route`, T3): from an inline system any seat may ask about; an adopted `system` is GM only. Writes nothing. */
  planRoute(ask: import("./route.ts").RouteAsk): Promise<{ ok: true; plan: import("./route.ts").RoutePlan } | { ok: false; code: string; message: string }>;
  /** One galaxy grown from the universe's seed (U6, kernel `galaxy_chart`, G1 to G4). Read only; GM only. */
  galaxyChart(ask: import("./galaxy.ts").GalaxyAsk): Promise<{ ok: true; chart: import("./galaxy.ts").GalaxyChart } | { ok: false; code: string; message: string }>;
  /** A slice of the cosmic web at one epoch (V9, kernel `web_slice`, G1). Read only; GM only. */
  webSlice(ask: import("./galaxy.ts").SliceAsk): Promise<{ ok: true; web: import("./galaxy.ts").WebSlice } | { ok: false; code: string; message: string }>;
  /** A system as the universe now holds it: facts, a SystemSpec for look-ahead, and S0 surfaces. */
  resolvedSystem(system: string, at_s?: number): Promise<{ ok: true; result: SystemResolved } | { ok: false; code: string; message: string }>;
  kernelVersion(): Promise<string>;
  /** Derived values of a flow (read only; no event). */
  derive(ref: DeriveRef): Promise<{ ok: true; values: Record<string, Derived> } | { ok: false; message: string }>;
  /** Why one derived quantity has its value. */
  explain(ref: DeriveRef, kind: string): Promise<{ ok: true; explanation: Explanation } | { ok: false; message: string }>;
  /** Flows the loaded pack stack offers. */
  flows(): readonly string[];
  device(): DeviceEdge | undefined;
  /** Intents waiting for the GM (V5 carries them; here they wait locally). */
  pending(): readonly Intent[];
  /** The packs in force, bottom to top, and the hash the kernel gave the stack. */
  stack(): StackInfo;
  /** The flow document the stack uses for `flow` and the pack that provides it. */
  flowDoc(flow: string): { doc: FlowDoc; pack: string } | undefined;
  /**
   * Try a candidate stack without adopting it: the kernel validates the stack
   * it would make, `probe` derives under it, then the stack in force is
   * restored. Read only; nothing is chronicled. Whatever the candidate leaves
   * out stays as it is in force.
   */
  trial<T>(candidate: StackCandidate, probe: (s: Probe) => Promise<T>): Promise<TrialResult<T>>;
  /**
   * What an audience is shown of everything under `scope` (K9 scene). A GM
   * seat may ask for any audience and may look through any data view; any
   * other seat gets only its own audience.
   */
  scene(scope: string, opts?: SceneOptions): Promise<{ ok: true; scene: PlayerScene; hash: string } | { ok: false; message: string }>;
}

export interface SceneOptions {
  /** Whose knowledge; defaults to the seat's own ("gm" for the GM seat). */
  audience?: string;
  /** Show every subject through this data view, whatever the grants say (a lens or a preview). */
  data_view?: string;
  /** A draft data-view setup (`provides.other.observe`) to build with instead of the stack's. Records nothing. */
  observe?: unknown;
  at_yr?: number;
}

export interface StackInfo {
  /** Everything under the campaign pack, bottom to top: the packs the session opened with, then `added`. */
  base: readonly PackDoc[];
  /** Packs the GM stacked on top of the opening packs (Packs screen). */
  added: readonly PackDoc[];
  /** The campaign's own physics pack over the base, if one is adopted. */
  campaign: PackDoc | null;
  hash: string;
}

export interface StackCandidate {
  added?: readonly PackDoc[];
  campaign?: PackDoc | null;
}

/** What a trial's probe may do: derive, and see the candidate stack. */
export type Probe = Pick<ViewSession, "derive" | "stack">;

export type TrialResult<T> = { ok: true; stack_hash: string; result: T } | { ok: false; report: string; issues: string[] };

type Raw = { type: string; [k: string]: unknown };

/**
 * Peer-to-peer hooks (V4), for p2p.ts only: not exported from the package
 * index, so views and shells cannot reach them. APPLIED watches every intent
 * the session applies (the GM's chronicle as it happens); REPLAY applies an
 * intent the GM already applied, whatever this seat may send (a player's
 * replica follows the single writer); GRANT_VIEWS names the data views the GM
 * lets this seat's screens look through.
 */
export const APPLIED = Symbol("sse.session.applied");
export const REPLAY = Symbol("sse.session.replay");
export const GRANT_VIEWS = Symbol("sse.session.grant-views");
export type AppliedListener = (intent: Intent, result: Extract<IntentResult, { status: "applied" }>) => void;

export class Session implements ViewSession {
  readonly #client: KernelClient;
  /** A second kernel with no universe open, for trials: loading a candidate stack there chronicles nothing. */
  readonly #scratch: (() => KernelClient) | undefined;
  #scratchClient: KernelClient | undefined;
  readonly #pending: Intent[] = [];
  readonly #listeners: AppliedListener[] = [];
  #grantedViews: readonly string[] = [];
  #flows: string[] = [];
  #opening: PackDoc[] = [];
  #added: PackDoc[] = [];
  #campaign: PackDoc | null = null;
  #stackHash = "";
  /** Requests run one after another, so a trial's temporary stack is never seen by another call. */
  #queue: Promise<unknown> = Promise.resolve();
  readonly seat: Readonly<Seat>;
  readonly universe: Readonly<{ id: string; seed: string; root: string }>;

  private constructor(client: KernelClient, seat: Seat, universe: { id: string; seed: string; root: string }, scratch?: () => KernelClient) {
    this.#client = client;
    this.#scratch = scratch;
    this.seat = Object.freeze({ ...seat });
    this.universe = Object.freeze(universe);
  }

  /**
   * Open a universe on a kernel and hand back a session for one seat.
   * `scratch` makes a second kernel (started on first use) where trials load
   * candidate stacks, since a stack loaded into the open universe is chronicled.
   */
  static async open(client: KernelClient, seat: Seat, universe: { id: string; seed: string }, packs: unknown[] = [], scratch?: () => KernelClient): Promise<Session> {
    let flows: string[] = [];
    let hash = "";
    if (packs.length) {
      const st = await call(client, { op: "load_stack", packs });
      if (st.type !== "stack_loaded") throw new Error(`kernel rejected the pack stack: ${String(st.report ?? st.message ?? JSON.stringify(st))}`);
      flows = st.flows as string[];
      hash = String(st.stack_hash);
    }
    const r = await call(client, { op: "open_universe", id: universe.id, seed: universe.seed });
    if (r.type !== "universe_opened") throw new Error(`kernel would not open the universe: ${JSON.stringify(r)}`);
    const s = new Session(client, seat, { ...universe, root: String(r.root) }, scratch);
    s.#flows = flows;
    s.#opening = structuredClone(packs) as PackDoc[];
    s.#stackHash = hash;
    return s;
  }

  /** Run `f` after every request already queued, and before any queued after it. */
  #serial<T>(f: () => Promise<T>): Promise<T> {
    const next = this.#queue.then(f, f);
    this.#queue = next.catch(() => undefined);
    return next;
  }

  #call(request: unknown): Promise<Raw> {
    return this.#serial(() => call(this.#client, request));
  }

  flows(): readonly string[] {
    return [...this.#flows];
  }

  /** The full stack in force, bottom to top. */
  #packs(added: readonly PackDoc[] = this.#added, campaign: PackDoc | null = this.#campaign): PackDoc[] {
    return [...this.#opening, ...added, ...(campaign ? [campaign] : [])];
  }

  stack(): StackInfo {
    return { base: structuredClone([...this.#opening, ...this.#added]), added: structuredClone(this.#added), campaign: structuredClone(this.#campaign), hash: this.#stackHash };
  }

  flowDoc(flow: string) {
    const p = providerOf(this.#packs(), flow);
    return p ? { doc: structuredClone(p.doc), pack: p.pack.id } : undefined;
  }

  trial<T>(candidate: StackCandidate, probe: (s: Probe) => Promise<T>): Promise<TrialResult<T>> {
    if (!this.#scratch) return Promise.reject(new Error("this session has no scratch kernel for trials"));
    const client = (this.#scratchClient ??= this.#scratch());
    const added = candidate.added ?? this.#added;
    const campaign = candidate.campaign === undefined ? this.#campaign : candidate.campaign;
    const packs = this.#packs(added, campaign);
    // Serial too, so trials never interleave on the scratch kernel.
    return this.#serial(async () => {
      const st = await call(client, { op: "load_stack", packs });
      if (st.type !== "stack_loaded") {
        return { ok: false as const, report: String(st.report ?? st.message ?? ""), issues: (st.issues as string[] | undefined) ?? [] };
      }
      const info: StackInfo = { base: structuredClone([...this.#opening, ...added]), added: structuredClone([...added]), campaign: structuredClone(campaign), hash: String(st.stack_hash) };
      const p: Probe = {
        stack: () => info,
        async derive(ref) {
          const r = await call(client, { op: "derive", flow: ref.flow, inputs: ref.inputs });
          return r.type === "derived" ? { ok: true as const, values: r.values as Record<string, Derived> } : { ok: false as const, message: String(r.message ?? JSON.stringify(r)) };
        },
      };
      return { ok: true as const, stack_hash: String(st.stack_hash), result: await probe(p) };
    });
  }

  async scene(scope: string, opts: SceneOptions = {}) {
    const own = audienceOf(this.seat);
    const audience = opts.audience ?? own;
    const viewOk = opts.data_view === undefined || this.#grantedViews.includes(opts.data_view);
    if (this.seat.authority !== "gm" && (audience !== own || !viewOk || opts.observe !== undefined)) {
      return { ok: false as const, message: `seat ${this.seat.label} sees only its own audience` };
    }
    const at_s = opts.at_yr === undefined ? 9_223_372_036_854_775_807n : yearsToSeconds(opts.at_yr);
    const r = await this.#call({ op: "scene", audience, scope, ...(opts.data_view ? { data_view: opts.data_view } : {}), ...(opts.observe ? { observe: opts.observe } : {}), at_s });
    if (r.type === "scene") return { ok: true as const, scene: r.scene as PlayerScene, hash: String(r.scene_hash) };
    return { ok: false as const, message: String(r.message ?? JSON.stringify(r)) };
  }

  async derive(ref: DeriveRef) {
    const r = await this.#call({ op: "derive", flow: ref.flow, inputs: ref.inputs });
    if (r.type === "derived") return { ok: true as const, values: r.values as Record<string, Derived> };
    return { ok: false as const, message: String(r.message ?? JSON.stringify(r)) };
  }

  async explain(ref: DeriveRef, kind: string) {
    const r = await this.#call({ op: "explain", flow: ref.flow, inputs: ref.inputs, kind });
    if (r.type === "explanation") return { ok: true as const, explanation: { ...(r.explanation as Explanation), fictional: Boolean(r.fictional) } };
    return { ok: false as const, message: String(r.message ?? JSON.stringify(r)) };
  }

  [APPLIED](listener: AppliedListener): () => void {
    this.#listeners.push(listener);
    return () => {
      const i = this.#listeners.indexOf(listener);
      if (i >= 0) this.#listeners.splice(i, 1);
    };
  }

  [GRANT_VIEWS](views: readonly string[]): void {
    this.#grantedViews = [...views];
  }

  /** Apply an intent the GM already applied (no seat check). */
  [REPLAY](intent: Intent): Promise<IntentResult> {
    const bad = checkIntent(intent);
    if (bad) return Promise.resolve({ status: "refused", reason: bad });
    return this.#apply(intent);
  }

  async send(intent: Intent): Promise<IntentResult> {
    const bad = checkIntent(intent);
    if (bad) return { status: "refused", reason: bad };
    const d = decide(this.seat, intent);
    if (d.action === "refuse") return { status: "refused", reason: d.reason };
    if (d.action === "ask_gm") {
      this.#pending.push(intent);
      return { status: "pending", intent };
    }
    return this.#apply(intent);
  }

  async #apply(intent: Intent): Promise<IntentResult> {
    const r = await this.#applyRaw(intent);
    if (r.status === "applied") for (const l of [...this.#listeners]) l(intent, r);
    return r;
  }

  async #applyRaw(intent: Intent): Promise<IntentResult> {
    const r = await this.#call(intentToRequest(intent, { opening: this.#opening, added: this.#added, campaign: this.#campaign }));
    if (intent.kind === "adopt_physics" || intent.kind === "stack_packs") {
      if (r.type !== "stack_loaded") return { status: "refused", reason: String(r.report ?? r.message ?? JSON.stringify(r)) };
      if (intent.kind === "adopt_physics") this.#campaign = intent.pack === null ? null : (structuredClone(intent.pack) as unknown as PackDoc);
      else this.#added = structuredClone(intent.packs) as unknown as PackDoc[];
      this.#flows = r.flows as string[];
      this.#stackHash = String(r.stack_hash);
      const adopted = r.adopted as { event_id: string; state_hash: string } | undefined;
      if (adopted) return { status: "applied", event_id: adopted.event_id, state_hash: adopted.state_hash, affected: [this.universe.root] };
      return { status: "applied", event_id: `stack:${this.#stackHash}`, state_hash: await this.stateHash(), affected: [] };
    }
    if (intent.kind === "scan_toggle" && r.type === "scan_toggled")
      return { status: "applied", event_id: String(r.event_id), state_hash: String(r.state_hash), affected: [intent.address], view: (r.view as string | null) ?? null };
    if (intent.kind === "grant" && r.type === "granted")
      return { status: "applied", event_id: String(r.event_id), state_hash: String(r.state_hash), affected: [intent.address], view: (r.view as string | null) ?? null };
    if (r.type === "authored")
      return { status: "applied", event_id: String(r.event_id), state_hash: String(r.state_hash), affected: r.affected as string[] };
    if (r.type === "error") return { status: "error", code: String(r.code), message: String(r.message) };
    return { status: "error", code: "unexpected", message: JSON.stringify(r) };
  }

  async history(address: string): Promise<HistoryEvent[]> {
    const r = await this.#call({ op: "history", address });
    if (r.type !== "history") return [];
    return (r.events as { id: string; kind: string; occurred_at_s: number; evidence: string; subject: string[] }[]).map((e) => ({
      id: e.id,
      kind: e.kind,
      at_yr: secondsToYears(e.occurred_at_s),
      evidence: e.evidence,
      subject: e.subject,
    }));
  }

  async stateHash(at_yr?: number): Promise<string> {
    const at_s = at_yr === undefined ? 9_223_372_036_854_775_807n : yearsToSeconds(at_yr);
    const r = await this.#call({ op: "state_hash", at_s });
    return r.type === "state_hash" ? String(r.hash) : "";
  }

  async openBranch(label: string): Promise<{ ok: true; branch: string } | { ok: false; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, message: "only the game master opens a what-if branch" };
    const r = await this.#call({ op: "branch", label });
    return r.type === "branch_opened" ? { ok: true, branch: String(r.branch) } : { ok: false, message: String(r.message ?? JSON.stringify(r)) };
  }

  async branchSend(branch: string, intent: Intent): Promise<IntentResult> {
    if (this.seat.authority !== "gm") return { status: "refused", reason: "only the game master edits a what-if branch" };
    if (intent.kind !== "set_value") return { status: "refused", reason: "a what-if branch takes value edits only" };
    const bad = checkIntent(intent);
    if (bad) return { status: "refused", reason: bad };
    const req = intentToRequest(intent) as { edit: unknown };
    const r = await this.#call({ op: "branch_set", branch, edit: req.edit });
    if (r.type === "authored") return { status: "applied", event_id: String(r.event_id), state_hash: String(r.state_hash), affected: r.affected as string[] };
    return { status: "error", code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async branchStateHash(branch: string, at_yr?: number): Promise<string> {
    const at_s = at_yr === undefined ? 9_223_372_036_854_775_807n : yearsToSeconds(at_yr);
    const r = await this.#call({ op: "branch_state_hash", branch, at_s });
    return r.type === "state_hash" ? String(r.hash) : "";
  }

  async discardBranch(branch: string): Promise<boolean> {
    const r = await this.#call({ op: "discard", branch });
    return r.type === "discarded";
  }

  /**
   * Read a V3 save (the text of a starmap.json or system.json) into facts and a difference report.
   * Pure: nothing is written to the open universe. `reconstruct` (1 to 64) also solves candidate histories.
   */
  async importV3(save: string, opts: { universe_id?: string; reconstruct?: number } = {}): Promise<{ ok: true; result: V3Imported } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master imports saves" };
    const r = await this.#call({ op: "import_v3", save, ...(opts.universe_id ? { universe_id: opts.universe_id } : {}), ...(opts.reconstruct ? { reconstruct: opts.reconstruct } : {}) });
    return r.type === "v3_imported" ? { ok: true, result: r as unknown as V3Imported } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  /** One system through the formation pipeline. Pure: nothing is written to the open universe. */
  async runFormation(req: FormationRequest): Promise<{ ok: true; result: FormationRan } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master generates systems" };
    const r = await this.#call({ op: "run_formation", ...req });
    return r.type === "formation_ran" ? { ok: true, result: r as unknown as FormationRan } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  /** Age a system forward on a branch of its own (T2). The main line does not move until the branch is committed. */
  async lookAhead(system: SystemSpec, target_myr: number): Promise<{ ok: true; result: LookedAhead } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master looks ahead" };
    const r = await this.#call({ op: "look_ahead", system, target_myr });
    return r.type === "looked_ahead" ? { ok: true, result: r as unknown as LookedAhead } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  /**
   * Keep a look-ahead as history; with `move_now_myr`, the system's "now" moves to that star age.
   * `write_back` also writes the aged star and surviving orbits as one `time.aged` event.
   */
  async commitLookAhead(branch: string, move_now_myr?: number, write_back?: boolean): Promise<{ ok: true; event_id: string; state_hash: string; aged_event?: string; aged_at_myr?: number } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master keeps a look-ahead" };
    const r = await this.#call({ op: "commit_look_ahead", branch, ...(move_now_myr !== undefined ? { move_now_myr } : {}), ...(write_back ? { write_back: true } : {}) });
    if (r.type !== "committed") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    return { ok: true, event_id: String(r.event_id), state_hash: String(r.state_hash), ...(r.aged_event ? { aged_event: String(r.aged_event), aged_at_myr: Number(r.aged_at_myr) } : {}) };
  }

  async adopt(source: { import_v3: string } | { formation: FormationRequest }, occurred_at_s?: number): Promise<{ ok: true; result: Adopted } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master adds systems to the universe" };
    const r = await this.#call({ op: "adopt", ...source, ...(occurred_at_s !== undefined ? { occurred_at_s } : {}) } as never);
    return r.type === "adopted" ? { ok: true, result: r as unknown as Adopted } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async readWorldStack(by: { body?: string; fixture?: string }): Promise<{ ok: true; result: WorldStackRead } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "read_world_stack", ...(by.body ? { body: by.body } : {}), ...(by.fixture ? { fixture: by.fixture } : {}) });
    return r.type === "world_stack_read" ? { ok: true, result: r as unknown as WorldStackRead } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async surfaceTiles(system: string, body: string): Promise<{ ok: true; tiles: SurfaceTile[]; refused: Json[]; regime?: string } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master reads surface tiles" };
    const tiles = [0, 1, 2, 3, 4, 5].map((face) => ({ body, face, level: 0, x: 0, y: 0 }));
    const r = await this.#call({ op: "resolved_system", system, tiles });
    if (r.type !== "system_resolved") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const x = r as unknown as { tiles?: SurfaceTile[]; refused?: Json[]; surfaces?: { address: string; regime_label?: string }[] };
    return { ok: true, tiles: x.tiles ?? [], refused: x.refused ?? [], regime: x.surfaces?.find((s2) => s2.address === body)?.regime_label };
  }

  /** The open universe as a document to keep (kernel `export_chronicle`, `sse.chronicle-export/1`); GM only. Open branches are not in it. */
  async exportChronicle(): Promise<{ ok: true; export: Json } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master saves the campaign" };
    const r = await this.#call({ op: "export_chronicle" });
    return r.type === "chronicle_exported" ? { ok: true, export: (r as unknown as { export: Json }).export } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  /**
   * Replace the open universe with a kept one (kernel `import_chronicle`).
   * `stack_matches` false: the packs loaded now differ from the ones it was
   * saved with; the caller warns. A player's screen uses this to start from
   * the GM's reopened campaign.
   */
  async importChronicle(doc: Json): Promise<{ ok: true; state_hash: string; events: number; stack_matches: boolean } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "import_chronicle", export: doc });
    if (r.type !== "chronicle_imported") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const x = r as unknown as { state_hash: string; events: number | unknown[]; stack_matches: boolean };
    return { ok: true, state_hash: String(x.state_hash), events: Array.isArray(x.events) ? x.events.length : Number(x.events), stack_matches: x.stack_matches !== false };
  }

  async planRoute(ask: import("./route.ts").RouteAsk): Promise<{ ok: true; plan: import("./route.ts").RoutePlan } | { ok: false; code: string; message: string }> {
    if (ask.system !== undefined && this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master plans against a system's full record; players plan from what they know" };
    const r = await this.#call({ op: "plan_route", ...ask });
    return r.type === "route_planned" ? { ok: true, plan: (r as unknown as { plan: import("./route.ts").RoutePlan }).plan } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async galaxyChart(ask: import("./galaxy.ts").GalaxyAsk): Promise<{ ok: true; chart: import("./galaxy.ts").GalaxyChart } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the galaxy's history is the game master's chart; players see it through data views" };
    const r = await this.#call({ op: "galaxy_chart", ...ask });
    return r.type === "galaxy_charted" ? { ok: true, chart: (r as unknown as { chart: import("./galaxy.ts").GalaxyChart }).chart } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async webSlice(ask: import("./galaxy.ts").SliceAsk): Promise<{ ok: true; web: import("./galaxy.ts").WebSlice } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the cosmic web is read on the game master's machine" };
    const r = await this.#call({ op: "web_slice", ...ask });
    return r.type === "web_sliced" ? { ok: true, web: (r as unknown as { web: import("./galaxy.ts").WebSlice }).web } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async lifeView(by: { body?: string; fixture?: string }): Promise<{ ok: true; source: string; view: LifeView } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master reads a world's whole biosphere; players see it through data views" };
    const r = await this.#call({ op: "life_view", ...(by.body !== undefined ? { body: by.body } : {}), ...(by.fixture !== undefined ? { fixture: by.fixture } : {}) });
    return r.type === "life_viewed" ? { ok: true, source: String(r.source), view: (r as unknown as { view: LifeView }).view } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async listSystems(): Promise<{ ok: true; systems: ListedSystem[] } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "systems" });
    return r.type === "systems_listed" ? { ok: true, systems: (r as unknown as { systems: ListedSystem[] }).systems } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async resolvedSystem(system: string, at_s?: number): Promise<{ ok: true; result: SystemResolved } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master reads resolved systems" };
    const r = await this.#call({ op: "resolved_system", system, ...(at_s !== undefined ? { at_s } : {}) });
    return r.type === "system_resolved" ? { ok: true, result: r as unknown as SystemResolved } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async kernelVersion(): Promise<string> {
    return (await this.#client.info()).kernel_version;
  }

  device(): DeviceEdge | undefined {
    const c = this.#client.capability;
    return c ? { tier: c.tier, unitsPerSecond: c.unitsPerSecond } : undefined;
  }

  pending(): readonly Intent[] {
    return [...this.#pending];
  }
}

async function call(client: KernelClient, request: unknown): Promise<Raw> {
  // Times are bigints in the protocol types and are written as exact digits.
  return JSON.parse(await client.callRaw(encodeRequest(request))) as Raw;
}
