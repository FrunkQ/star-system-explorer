// The session: the one door from any interface to the kernel. Views and
// shells receive a ViewSession and never see the kernel client behind it.
// Only this file and host-browser.ts may import the kernel client (checked by
// tests/ui/boundary.test.ts).
import type { OrbitPlaced } from "./orbits.ts";
/** A pack's calendar as the kernel serves it (`provides.other.calendar`). */
/** Where and how a sky is asked for (`sky_from_here`, `sky_glow`): a system address or a galactic position in pc (neither: the Sun's place). */
export interface SkyAsk {
  observer?: string;
  position_pc?: [number, number, number];
  t_myr?: number;
  limit_mag?: number;
  band?: string;
  bands?: string[];
}

/** One ask of the kernel `weather` op (presentation reads; nothing is recorded). */
export interface WeatherAsk {
  want: "climate" | "systems" | "field" | "report" | "column" | "sound";
  world?: string;
  system?: string;
  body?: string;
  at_s?: number;
  day?: number;
  /** [face 0-5, level, x, y] on the surface atlas. */
  tile?: [number, number, number, number];
  /** Cells across the tile, 1 to 64. */
  n?: number;
  lat?: number;
  lon?: number;
}

export interface PackCalendar {
  name: string;
  epoch_s: number;
  epoch_year?: number;
  day_s: number;
  year_days: number;
  months?: { name: string; days: number }[];
  format: string;
}
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
  | { status: "applied"; event_id: string; state_hash: string; affected: string[]; view?: string | null; reply?: Record<string, Json> }
  | { status: "pending"; intent: Intent }
  | { status: "refused"; reason: string }
  | { status: "error"; code: string; message: string };

export interface HistoryEvent {
  id: string;
  kind: string;
  at_yr: number;
  evidence: string;
  subject: string[];
  /** The facts it set or unset at the asked address (only when asked): a value too large to send is marked `large`. */
  changes?: { field: string; value: unknown; unset?: boolean; large?: boolean }[];
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
/** One atom as the kernel describes it (op `atoms`). */
export interface AtomInfo {
  id: string;
  version: string;
  cost_class: string;
  inputs: { name: string; kind: string; dim: string; valid: [number, number] | null }[];
  outputs: { name: string; kind: string; dim: string; valid: [number, number] | null }[];
  params: { name: string; default: number; range: [number, number] | null; dim: string; description: string }[];
  references: { text: string; doi?: string; url?: string }[];
  explain: { curious?: string; science?: string; formula?: string };
}

export interface ViewSession {
  readonly seat: Readonly<Seat>;
  readonly universe: Readonly<{ id: string; seed: string; root: string }>;
  send(intent: Intent): Promise<IntentResult>;
  /** The events at an address, oldest first; with `changes`, each event's fact changes at exactly that address. */
  history(address: string, opts?: { changes?: boolean }): Promise<HistoryEvent[]>;
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
  /** Tiles of a world at any level (S1, through resolved_system; at most 24 a call), with the kernel's refusals; GM only. The ground round a landing site (SL1). */
  surfaceTilesAt(system: string, body: string, keys: readonly { face: number; level: number; x: number; y: number }[]): Promise<{ ok: true; tiles: SurfaceTile[]; refused: { face: number; level: number; x: number; y: number; why: string }[] } | { ok: false; code: string; message: string }>;
  /** A world's landmarks (SL1): its named landscape from a quick survey (`survey_surface`'s register, the same world the globe shows); GM only. */
  surfaceLandmarks(system: string, body: string, limit?: number): Promise<{ ok: true; landmarks: import("./landing.ts").Landmark[]; total: number } | { ok: false; code: string; message: string }>;
  /**
   * Let this seat read one world's ground while the table watches its landing (the landing run-through): its tiles
   * and a descent planned to a site, on this seat's own kernel, for that body only. Null ends it. A GM needs none.
   */
  grantLanding?(g: { system: string; body: string } | null): void;
  /** A descent planned to any site (SL1, kernel `plan_descent` on the quick survey): the site judged, a lander flown down; GM only. */
  planDescent(ask: { system: string; body: string; lat_deg: number; lon_deg: number; lander: string | Record<string, unknown> }): Promise<{ ok: true; plan: import("./landing.ts").DescentPlan } | { ok: false; code: string; message: string }>;
  /** A world's one climate (W7a), from a quick survey (kernel `survey_surface`, about half a second, cached by the kernel); GM only. */
  surveyClimate(system: string, body: string): Promise<{ ok: true; climate: import("./climate.ts").ClimateSummary } | { ok: false; code: string; message: string }>;
  /** The open universe as a document to keep (U10, kernel `export_chronicle`); GM only. */
  exportChronicle(): Promise<{ ok: true; export: Json } | { ok: false; code: string; message: string }>;
  /** Replace the open universe with a kept one (U10, kernel `import_chronicle`). */
  importChronicle(doc: Json): Promise<{ ok: true; state_hash: string; events: number; stack_matches: boolean } | { ok: false; code: string; message: string }>;
  /** One world's life (U7, kernel `life_view`): a body once systems carry habitat graphs, or one of the example fixtures. GM only. */
  /** A world's life; with `dials`, a read-only what-if under other dial values (the kernel changes nothing). */
  lifeView(by: { body?: string; fixture?: string; dials?: Record<string, number> }): Promise<{ ok: true; source: string; view: LifeView } | { ok: false; code: string; message: string }>;
  /** Bodies on their orbits at given times (kernel `orbits_at`, U11 part 2): relative to each host, up to 64 times. `bodies` places a scene the kernel does not hold. */
  orbitsAt(system: string, at_s: readonly bigint[], bodies?: { address: string; facts: Record<string, number | string | bigint> }[]): Promise<{ ok: true; bodies: OrbitPlaced[] } | { ok: false; code: string; message: string }>;
  /** The campaign's now kept in the universe (kernel `campaign_now`); absent if never set. */
  campaignNow(): Promise<{ ok: true; at_s?: bigint } | { ok: false; code: string; message: string }>;
  /** Move the campaign's now (kernel `set_campaign_now`); GM only. */
  setCampaignNow(at_s: bigint): Promise<{ ok: true; state_hash: string } | { ok: false; code: string; message: string }>;
  /** The top pack's calendar (kernel `calendar`), if any pack gives one. */
  calendar(): Promise<{ ok: true; calendar?: PackCalendar } | { ok: false; code: string; message: string }>;
  /** Every system in the open universe that has facts (kernel `systems`): adopted ones and authored ones. */
  listSystems(): Promise<{ ok: true; systems: ListedSystem[] } | { ok: false; code: string; message: string }>;
  /** The links between places the record holds (kernel `links_in`, HW1): gates, lanes, routes; the GM sees all. Read only. */
  linksIn?(ask?: { bounds?: string; audience?: string }): Promise<{ ok: true; links: import("./your-systems.ts").RecordLink[] } | { ok: false; code: string; message: string }>;
  /** Candidate courses (V8, kernel `plan_route`, T3): from an inline system any seat may ask about; an adopted `system` is GM only. Writes nothing. */
  planRoute(ask: import("./route.ts").RouteAsk): Promise<{ ok: true; plan: import("./route.ts").RoutePlan } | { ok: false; code: string; message: string }>;
  /** What a ship can do at each of some worlds (kernel `ship_at_world`): aerocapture, entry, landing, take-off, with dangers, margins and rough chances; nothing rolled. An adopted `system` is GM only. Writes nothing. */
  /** V8 flight log: write one entry of a construct's log on the record (GM only). */
  logFlight(ask: { construct: string; at_s: number } & ({ piece: Json } | { event: Json } | { name?: string; about?: string } | { withdraw: string })): Promise<{ ok: true; event_id: string; seeded: string[]; notes: string[] } | { ok: false; code: string; message: string }>;
  /** V8 flight log: a construct's log as written, and with `where_at` and a system, where the craft was then (GM only). */
  flightLog(ask: { construct: string; at_s?: number; where_at?: number; fixture?: string; nav?: unknown; system?: string }): Promise<{ ok: true; log: Json; entries: Json[]; at?: Json } | { ok: false; code: string; message: string }>;
  shipAtWorld(ask: import("./ship-at-world.ts").ShipAtWorldAsk): Promise<{ ok: true; report: import("./ship-at-world.ts").ShipAtWorldReport } | { ok: false; code: string; message: string }>;
  /** One galaxy grown from the universe's seed (U6, kernel `galaxy_chart`, G1 to G4). Read only; GM only. */
  galaxyChart(ask: import("./galaxy.ts").GalaxyAsk): Promise<{ ok: true; chart: import("./galaxy.ts").GalaxyChart } | { ok: false; code: string; message: string }>;
  /**
   * Which body lands in which class of `scheme` under a candidate stack (kernel `classify_preview`), without loading it.
   * Read only; GM only. Used to preview a drafted game table.
   */
  classifyPreview?(ask: ClassifyAsk): Promise<{ ok: true; result: ClassifyPreview } | { ok: false; code: string; message: string }>;
  /** The story finder (U8, kernel `sift`): sifters over the campaign, hits with evidence. GM only. */
  sift(ask: import("./story.ts").SiftAsk): Promise<{ ok: true; result: import("./story.ts").SiftResult } | { ok: false; code: string; message: string }>;
  /** What a ship's sensor sees from beside a body (V8 Window, kernel `window_view`). Read only; any seat, on the system it knows. */
  windowView(ask: import("./window.ts").WindowAsk): Promise<{ ok: true; view: import("./window.ts").WindowView } | { ok: false; code: string; message: string }>;
  /** One halo cell's galaxies (V9 cluster map, kernel `halo_cell`, G2). Read only; GM only. */
  haloCell(ask: import("./galaxy.ts").HaloCellAsk): Promise<{ ok: true; cell: import("./galaxy.ts").HaloCell } | { ok: false; code: string; message: string }>;
  /** One 10 pc star cell of the campaign's galaxy (kernel `star_cell`, Y7a). Read only; GM only. */
  starCell(ask: { cell?: [number, number, number]; t_myr?: number }): Promise<{ ok: true; cell: import("./voyage.ts").StarCell } | { ok: false; code: string; message: string }>;
  /** A voyage between stars for one ship (kernel `plan_voyage`, V8 part 3). Read only; GM only for now (players ask through their console later). */
  /** Effect at range (RK12, kernel `effect_at_range`): each ask's reading and the distances where it changes. */
  effectAtRange(asks: import("./range-rings.ts").RangeAsk[]): Promise<{ ok: true; effects: import("./range-rings.ts").RangeEffect[] } | { ok: false; code: string; message: string }>;
  /** A ship's power books (RK9 kernel `ship_power`), for the Power card (RK13). */
  shipPower(ask: { ship: string | Json; flux_w_m2?: number; at_s?: bigint }): Promise<{ ok: true; power: import("./power.ts").PowerBooks } | { ok: false; code: string; message: string }>;
  planVoyage(ask: import("./voyage.ts").VoyageAsk): Promise<{ ok: true; plan: import("./voyage.ts").VoyagePlan | import("./voyage.ts").SystemVoyagePlan } | { ok: false; code: string; message: string }>;
  /** The stacked packs' templates for ships, stations, sites and the like (kernel `templates`, J7). Read only. */
  templates?(): Promise<{ ok: true; templates: import("./constructs.ts").ConstructTemplate[] } | { ok: false; code: string; message: string }>;
  /** Make an object from a template under `under` (kernel `create`): its facts and tags as one universe event, its kept values in the record. GM only. */
  createFrom?(ask: { template: string; under: string; key?: string; name?: string; at_s?: number | bigint }): Promise<{ ok: true; result: import("./constructs.ts").Created } | { ok: false; code: string; message: string }>;
  /** Take an object made from a template out of play (kernel `retire`); nothing is deleted. GM only. */
  retire?(address: string, reason?: string, at_s?: number | bigint): Promise<{ ok: true; event_id: string } | { ok: false; code: string; message: string }>;
  /** The record at `address` (kernel `record`): its notes, kept values with their history, and actors. GM only: seats get disclosures. */
  record?(address: string, at_s?: number | bigint): Promise<{ ok: true; record: import("./constructs.ts").RecordRead } | { ok: false; code: string; message: string }>;
  /** Write a note in the record (kernel `add_note`, J1) about `address`, for `audience` ("everyone", "gm" or audience ids). `narrative` marks a storyteller's text. Notes never change the universe's state. GM only. */
  /** Track MC: add a record actor (a person or a lot aboard, an empire, a crew) at the universe root or under another actor. GM only. */
  addActor?(a: { id: string; name: string; kind: string; parent?: string; at_s?: number | bigint }): Promise<{ ok: true; address: string } | { ok: false; code: string; message: string }>;
  /** Track MC: change one kept value (kernel `set_kept`; null clears it), with a reason for the record. GM only. */
  setKept?(k: { address: string; kind: string; value: Json | null; reason?: string; at_s?: number | bigint }): Promise<{ ok: true; change_id: string } | { ok: false; code: string; message: string }>;
  addNote?(n: { address: string; text: string; audience: "everyone" | "gm" | string[]; tags?: string[]; narrative?: boolean; supersedes?: string; at_s?: number | bigint }): Promise<{ ok: true; note_id: string } | { ok: false; code: string; message: string }>;
  /** Game-system codes (X3a, kernel `codes`) for every body at or under `scope`, read by a code sheet (the first in the stack when none is named). Read only; GM only. */
  codes?(ask: { scope: string; sheet?: string; at_s?: number | bigint }): Promise<{ ok: true; result: import("./sector.ts").CodesRead } | { ok: false; code: string; message: string }>;
  /** The hex map's worlds (X3c, kernel `sector_map`): each mapped system's main world with its reading, by hex. Read only; GM only. */
  sectorMap?(ask: { scope: string; sheet?: string; at_s?: number | bigint }): Promise<{ ok: true; result: import("./sector.ts").SectorMap } | { ok: false; code: string; message: string }>;
  /** Read a sector file (X3c, kernel `import_sector`): a preview without `apply`; with it, every world written as history (and with `grow`, its whole system grown first). GM only. */
  importSector?(ask: import("./sector.ts").SectorImportAsk): Promise<{ ok: true; result: import("./sector.ts").SectorImported } | { ok: false; code: string; message: string }>;
  /** Write the mapped systems as a sector file (X3c, kernel `export_sector`). Read only; GM only. */
  exportSector?(ask: { scope: string; format: import("./sector.ts").SectorFormat; sheet?: string; at_s?: number | bigint }): Promise<{ ok: true; result: import("./sector.ts").SectorExported } | { ok: false; code: string; message: string }>;
  /** A slice of the cosmic web at one epoch (V9, kernel `web_slice`, G1). Read only; GM only. */
  webSlice(ask: import("./galaxy.ts").SliceAsk): Promise<{ ok: true; web: import("./galaxy.ts").WebSlice } | { ok: false; code: string; message: string }>;
  /** A cube of the cosmic web for the 3D view (kernel `web_slice` with `cube`): n^3 parcels moved by the truncated Zel'dovich approximation. Read only; GM only. */
  webCube(ask: import("./galaxy.ts").SliceAsk): Promise<{ ok: true; web: import("./galaxy.ts").WebCube } | { ok: false; code: string; message: string }>;
  /** The galaxies in the same cube (kernel `web_slice` with `cube` and `haloes`, V9w): what True lights. Read only; GM only. */
  webGalaxies?(ask: import("./galaxy.ts").SliceAsk): Promise<{ ok: true; web: import("./galaxy.ts").WebGalaxies } | { ok: false; code: string; message: string }>;
  /** A system as the universe now holds it: facts, a SystemSpec for look-ahead, and S0 surfaces. */
  resolvedSystem(system: string, at_s?: number): Promise<{ ok: true; result: SystemResolved } | { ok: false; code: string; message: string }>;
  /** Air optics for the renderer's sky (kernel `read_air_optics`, light track): Rayleigh scattering and optical depth per wavelength. Read only; any seat. */
  readAirOptics(airs: Json[], wavelengths_m?: number[]): Promise<{ ok: true; airs: Json[] } | { ok: false; code: string; message: string }>;
  /** Stellar activity (kernel `read_activity`, light track O10): regime, spin, X-rays, spots, cycle, and flares in a window. Read only; any seat. */
  /** Time-lapse keyframes of a world's surface (surface_frames): a fixture, or a body of an adopted system. Read only. */
  surfaceFrames(ask: { fixture?: string; system?: string; body?: string; at_s?: number; detail?: "orbit" | "survey"; frames_max?: number }): Promise<{ ok: true; frames: Json; label: string; source: string } | { ok: false; code: string; message: string }>;
  /** The kernel `weather` op: a fixture `world`, or a `body` of an adopted `system` at `at_s`; `want` picks climate, systems, field (a cube-sphere `tile`), report, column or sound, at `day` (days since the campaign epoch). */
  /** Y2: the stars seen from a place at a time down to a limiting magnitude (kernel `sky_from_here`); read only. */
  skyFromHere(ask: SkyAsk & { cone?: { l_deg: number; b_deg: number; half_angle_deg: number }; max_stars?: number }): Promise<{ ok: true; sky: Json } | { ok: false; code: string; message: string }>;
  /** Y3: the light of every star the sky draw leaves out, as a HEALPix map per band (kernel `sky_glow`); read only. */
  skyGlow(ask: Omit<SkyAsk, "observer"> & { nside?: number }): Promise<{ ok: true; glow: Json } | { ok: false; code: string; message: string }>;
  weather(ask: WeatherAsk): Promise<{ ok: true; want: string; world: string; provenance: Json; data: Json } | { ok: false; code: string; message: string }>;
  readActivity(ask: { stars: Json[]; seed?: string; at_s?: number; window?: { from_s: number; to_s: number; min_energy_j?: number } }): Promise<{ ok: true; stars: Json[] } | { ok: false; code: string; message: string }>;
  /** What stops the wind at each body, and its field (read only): `fixture: "solar-system"` or handed `bodies`. */
  readField(ask: { fixture?: string; bodies?: Json[] }): Promise<{ ok: true; bodies: Json[] } | { ok: false; code: string; message: string }>;
  /** SV2: who knows a thing, or one fact on it (kernel `known_to`): per audience its landmark, rung and exceptions. GM only. */
  knownTo(address: string, field?: string, at_s?: bigint): Promise<{ ok: true; value: import("./known.ts").KnownTo } | { ok: false; code: string; message: string }>;
  /** NM6: every name a thing has, by layer (kernel `read_names`); a player seat sees only the layers its view discloses. */
  readNames(address: string): Promise<{ ok: true; value: import("./names.ts").NamesRead } | { ok: false; code: string; message: string }>;
  /** NM6: a sample of names from one scheme on the stack (kernel `name_sample`), the same on every device. Read only. */
  nameSample(ask: import("./names.ts").SampleAsk): Promise<{ ok: true; value: import("./names.ts").NameSample } | { ok: false; code: string; message: string }>;
  /** MP10: the master data sheet as the stack has it (kernel `read_matter`): the listing, a table's ids, or one row with sources and readers. Read only. */
  readMatter(ask: { table?: string; id?: string }): Promise<{ ok: true; value: Json } | { ok: false; code: string; message: string }>;
  kernelVersion(): Promise<string>;
  /** Derived values of a flow (read only; no event). */
  derive(ref: DeriveRef): Promise<{ ok: true; values: Record<string, Derived> } | { ok: false; message: string }>;
  /** Why one derived quantity has its value. */
  explain(ref: DeriveRef, kind: string): Promise<{ ok: true; explanation: Explanation } | { ok: false; message: string }>;
  /** Flows the loaded pack stack offers. */
  flows(): readonly string[];
  /** The atoms the kernel knows (kernel `atoms`): inputs, outputs and parameters with their ranges, cost, references. Read only. */
  atoms?(ids?: readonly string[]): Promise<{ ok: true; atoms: AtomInfo[] } | { ok: false; message: string }>;
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

export interface ClassifyAsk {
  scheme: string;
  /** The stack to classify under; whatever it leaves out stays as it is in force. */
  candidate?: StackCandidate;
  scope?: string;
  at_s?: number;
}

export interface ClassifyPreview {
  scheme: string;
  atlas_hash: string;
  at_s?: number;
  bodies: { address: string; base: string | null; fit: number; modifiers: string[] }[];
  /** How many bodies in scope took no base class. */
  unclassified: number;
  failing_examples: string[];
}

export interface StackCandidate {
  added?: readonly PackDoc[];
  campaign?: PackDoc | null;
}

/** What a trial's probe may do: derive, see the candidate stack, and read the packs' own reports (ND6). */
export type Probe = Pick<ViewSession, "derive" | "stack" | "nameSample"> & {
  /** Sources, fudges and reproduction checks of one pack of the candidate stack (or every pack), worked out when it loaded. */
  packReports(pack?: string): Promise<{ ok: true; reports: unknown[] } | { ok: false; message: string }>;
};

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
  /** The kernel's answer to the last adopt, for `adopt()` to hand back. */
  #adopted: Adopted | null = null;
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
        async nameSample(ask) {
          const r = await call(client, { op: "name_sample", ...ask } as never);
          return r.type === "names" ? { ok: true as const, value: (r as unknown as { value: import("./names.ts").NameSample }).value } : { ok: false as const, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
        },
        async packReports(pack) {
          const r = await call(client, { op: "pack_reports", ...(pack ? { pack } : {}) });
          return r.type === "pack_reports" && Array.isArray(r.reports) ? { ok: true as const, reports: r.reports as unknown[] } : { ok: false as const, message: String(r.message ?? JSON.stringify(r)) };
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

  async atoms(ids?: readonly string[]) {
    const r = await this.#call({ op: "atoms", ...(ids ? { ids: [...ids] } : {}) });
    if (r.type === "atoms") return { ok: true as const, atoms: r.atoms as AtomInfo[] };
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

  // Intents apply one at a time, in the order they were sent: each one's state hash is the state after it, so two
  // sent together (a flight log entry and a tank level) must not interleave, or the players' copy replays them in a
  // different order from the one the hashes were taken in and falls out of step.
  #applying: Promise<unknown> = Promise.resolve();
  #apply(intent: Intent): Promise<IntentResult> {
    const run = this.#applying.then(async () => {
      const r = await this.#applyRaw(intent);
      if (r.status === "applied") for (const l of [...this.#listeners]) l(intent, r);
      return r;
    });
    this.#applying = run.catch(() => undefined);
    return run;
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
    if (intent.kind === "adopt" && r.type === "adopted") {
      const a = r as unknown as Adopted;
      this.#adopted = a;
      return { status: "applied", event_id: a.events[0] ?? `adopt:${a.state_hash}`, state_hash: a.state_hash, affected: a.systems };
    }
    if (r.type === "authored")
      return { status: "applied", event_id: String(r.event_id), state_hash: String(r.state_hash), affected: r.affected as string[] };
    if (intent.kind === "record" && r.type !== "error") {
      const address = String((intent.ask as { construct?: string; address?: string }).construct ?? (intent.ask as { address?: string }).address ?? "");
      return { status: "applied", event_id: String(r.event_id ?? r.change_id ?? ""), state_hash: String(r.state_hash ?? (await this.stateHash())), affected: address ? [address] : [], reply: r as Record<string, Json> };
    }
    if ((intent.kind === "set_known" || intent.kind === "undo_known") && (r.type === "known_set" || r.type === "known_undone"))
      return { status: "applied", event_id: String(r.undo ?? r.event_id ?? ""), state_hash: String(r.state_hash ?? (await this.stateHash())), affected: intent.kind === "set_known" ? [intent.address] : [] };
    if (r.type === "error") return { status: "error", code: String(r.code), message: String(r.message) };
    return { status: "error", code: "unexpected", message: JSON.stringify(r) };
  }

  async history(address: string, opts: { changes?: boolean } = {}): Promise<HistoryEvent[]> {
    const r = await this.#call({ op: "history", address, ...(opts.changes ? { changes: true } : {}) });
    if (r.type !== "history") return [];
    return (r.events as { id: string; kind: string; occurred_at_s: number; evidence: string; subject: string[]; changes?: HistoryEvent["changes"] }[]).map((e) => ({
      id: e.id,
      kind: e.kind,
      at_yr: secondsToYears(e.occurred_at_s),
      evidence: e.evidence,
      subject: e.subject,
      ...(e.changes ? { changes: e.changes } : {}),
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
    // Through the intent path, so players' screens replay it (and a kept log holds it).
    // Plain data: a caller's reactive object would not survive the copy every applied intent makes for the log.
    const plain = JSON.parse(JSON.stringify(source)) as { import_v3: string } | { formation: Json };
    const res = await this.#apply({ kind: "adopt", source: plain, ...(occurred_at_s !== undefined ? { occurred_at_s } : {}) });
    const got = this.#adopted;
    this.#adopted = null;
    if (res.status === "applied" && got) return { ok: true, result: got };
    return { ok: false, code: res.status === "error" ? res.code : res.status, message: res.status === "error" ? res.message : res.status === "refused" ? res.reason : "not applied" };
  }

  async readWorldStack(by: { body?: string; fixture?: string }): Promise<{ ok: true; result: WorldStackRead } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "read_world_stack", ...(by.body ? { body: by.body } : {}), ...(by.fixture ? { fixture: by.fixture } : {}) });
    return r.type === "world_stack_read" ? { ok: true, result: r as unknown as WorldStackRead } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  // The one world a player seat may read the ground of, while the table watches its landing.
  #landing: { system: string; body: string } | null = null;
  grantLanding(g: { system: string; body: string } | null): void {
    this.#landing = g ? { system: g.system, body: g.body } : null;
  }
  #mayLand(system: string, body: string): boolean {
    return this.seat.authority === "gm" || (this.#landing !== null && this.#landing.system === system && this.#landing.body === body);
  }

  async surfaceTiles(system: string, body: string): Promise<{ ok: true; tiles: SurfaceTile[]; refused: Json[]; regime?: string } | { ok: false; code: string; message: string }> {
    if (!this.#mayLand(system, body)) return { ok: false, code: "not_gm", message: "only the game master reads surface tiles" };
    const tiles = [0, 1, 2, 3, 4, 5].map((face) => ({ body, face, level: 0, x: 0, y: 0 }));
    const r = await this.#call({ op: "resolved_system", system, tiles });
    if (r.type !== "system_resolved") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const x = r as unknown as { tiles?: SurfaceTile[]; refused?: Json[]; surfaces?: { address: string; regime_label?: string }[] };
    return { ok: true, tiles: x.tiles ?? [], refused: x.refused ?? [], regime: x.surfaces?.find((s2) => s2.address === body)?.regime_label };
  }

  async surfaceTilesAt(system: string, body: string, keys: readonly { face: number; level: number; x: number; y: number }[]): Promise<{ ok: true; tiles: SurfaceTile[]; refused: { face: number; level: number; x: number; y: number; why: string }[] } | { ok: false; code: string; message: string }> {
    if (!this.#mayLand(system, body)) return { ok: false, code: "not_gm", message: "only the game master reads surface tiles" };
    const r = await this.#call({ op: "resolved_system", system, tiles: keys.map((k) => ({ body, face: k.face, level: k.level, x: k.x, y: k.y })) });
    if (r.type !== "system_resolved") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const x = r as unknown as { tiles?: SurfaceTile[]; refused?: { face: number; level: number; x: number; y: number; why: string }[] };
    return { ok: true, tiles: x.tiles ?? [], refused: x.refused ?? [] };
  }

  async surfaceLandmarks(system: string, body: string, limit = 60): Promise<{ ok: true; landmarks: import("./landing.ts").Landmark[]; total: number } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master surveys a world" };
    const r = await this.#call({ op: "survey_surface", system, body, quick: true, limit });
    if (r.type !== "surface_surveyed") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const reg = (r as unknown as { survey?: { register?: { features?: import("./landing.ts").Landmark[]; features_total?: number } } }).survey?.register;
    return { ok: true, landmarks: reg?.features ?? [], total: reg?.features_total ?? 0 };
  }

  async planDescent(ask: { system: string; body: string; lat_deg: number; lon_deg: number; lander: string | Record<string, unknown> }): Promise<{ ok: true; plan: import("./landing.ts").DescentPlan } | { ok: false; code: string; message: string }> {
    if (!this.#mayLand(ask.system, ask.body)) return { ok: false, code: "not_gm", message: "only the game master plans a descent against a world's full record" };
    const r = await this.#call({ op: "plan_descent", system: ask.system, body: ask.body, quick: true, lander: ask.lander as Json, lat_deg: ask.lat_deg, lon_deg: ask.lon_deg });
    return r.type === "descent_planned" ? { ok: true, plan: (r as unknown as { descent: import("./landing.ts").DescentPlan }).descent } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async surveyClimate(system: string, body: string): Promise<{ ok: true; climate: import("./climate.ts").ClimateSummary } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master surveys a world" };
    // quick: the Orbit level, the same world the globe shows; limit 1: the feature register is not wanted here.
    const r = await this.#call({ op: "survey_surface", system, body, quick: true, limit: 1 });
    if (r.type !== "surface_surveyed") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const c = (r as unknown as { survey?: { climate?: import("./climate.ts").ClimateSummary } }).survey?.climate;
    return c ? { ok: true, climate: c } : { ok: false, code: "no_climate", message: "this world has no climate in its survey" };
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

  async logFlight(ask: { construct: string; at_s: number } & ({ piece: Json } | { event: Json } | { name?: string; about?: string } | { withdraw: string })): Promise<{ ok: true; event_id: string; seeded: string[]; notes: string[] } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "flight logs are written on the game master's machine" };
    const w = await this.#record("log_flight", { ...ask, at_s: Math.round(ask.at_s) });
    if (!w.ok) return w;
    const r = w.reply;
    return r.type === "flight_logged" ? { ok: true, event_id: String(r.event_id), seeded: (r.seeded as string[]) ?? [], notes: (r.notes as string[]) ?? [] } : { ok: false, code: "unexpected", message: JSON.stringify(r) };
  }

  async flightLog(ask: { construct: string; at_s?: number; where_at?: number; fixture?: string; nav?: unknown; system?: string }): Promise<{ ok: true; log: Json; entries: Json[]; at?: Json } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "flight logs are read on the game master's machine" };
    const r = await this.#call({ op: "flight_log", ...ask } as Json);
    return r.type === "flight_log_is" ? { ok: true, log: r.log as Json, entries: (r.entries as Json[]) ?? [], ...(r.at !== undefined ? { at: r.at as Json } : {}) } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async shipAtWorld(ask: import("./ship-at-world.ts").ShipAtWorldAsk): Promise<{ ok: true; report: import("./ship-at-world.ts").ShipAtWorldReport } | { ok: false; code: string; message: string }> {
    if (ask.system !== undefined && this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master judges a ship against a system's full record" };
    const r = await this.#call({ op: "ship_at_world", ...ask });
    return r.type === "ship_at_world_read" ? { ok: true, report: (r as unknown as { report: import("./ship-at-world.ts").ShipAtWorldReport }).report } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async sift(ask: import("./story.ts").SiftAsk): Promise<{ ok: true; result: import("./story.ts").SiftResult } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the story finder reads the whole campaign, so it runs on the game master's machine" };
    const r = await this.#call({ op: "sift", ...ask });
    return r.type === "sifted" ? { ok: true, result: r as unknown as import("./story.ts").SiftResult } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async classifyPreview(ask: ClassifyAsk): Promise<{ ok: true; result: ClassifyPreview } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "a classification preview reads the whole campaign, so it runs on the game master's machine" };
    const added = ask.candidate?.added ?? this.#added;
    const campaign = ask.candidate?.campaign === undefined ? this.#campaign : ask.candidate.campaign;
    const r = await this.#call({ op: "classify_preview", packs: this.#packs(added, campaign), scheme: ask.scheme, ...(ask.scope ? { scope: ask.scope } : {}), ...(ask.at_s !== undefined ? { at_s: ask.at_s } : {}) });
    return r.type === "classify_previewed" ? { ok: true, result: r as unknown as ClassifyPreview } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async readAirOptics(airs: Json[], wavelengths_m?: number[]): Promise<{ ok: true; airs: Json[] } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "read_air_optics", airs, ...(wavelengths_m ? { wavelengths_m } : {}) });
    return r.type === "air_optics_read" ? { ok: true, airs: (r as unknown as { airs: Json[] }).airs } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async surfaceFrames(ask: { fixture?: string; system?: string; body?: string; at_s?: number; detail?: "orbit" | "survey"; frames_max?: number }): Promise<{ ok: true; frames: Json; label: string; source: string } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "surface_frames", ...ask } as Json);
    if (r.type !== "frames_made") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const f = (r as unknown as { frames: { label: string; source: string; frames: Json } }).frames;
    return { ok: true, frames: f.frames, label: f.label, source: f.source };
  }

  async skyFromHere(ask: SkyAsk & { cone?: { l_deg: number; b_deg: number; half_angle_deg: number }; max_stars?: number }): Promise<{ ok: true; sky: Json } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "sky_from_here", ...ask } as Json);
    return r.type === "sky" ? { ok: true, sky: (r as unknown as { sky: Json }).sky } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async skyGlow(ask: Omit<SkyAsk, "observer"> & { nside?: number }): Promise<{ ok: true; glow: Json } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "sky_glow", ...ask } as Json);
    return r.type === "sky_glowed" ? { ok: true, glow: (r as unknown as { glow: Json }).glow } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async weather(ask: WeatherAsk): Promise<{ ok: true; want: string; world: string; provenance: Json; data: Json } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "weather", ...ask } as Json);
    if (r.type !== "weather_read") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const x = r as unknown as { want: string; world: string; provenance: Json; data: Json };
    return { ok: true, want: x.want, world: x.world, provenance: x.provenance, data: x.data };
  }

  async readActivity(ask: { stars: Json[]; seed?: string; at_s?: number; window?: { from_s: number; to_s: number; min_energy_j?: number } }): Promise<{ ok: true; stars: Json[] } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "read_activity", ...ask } as Json);
    return r.type === "activity_read" ? { ok: true, stars: (r as unknown as { stars: Json[] }).stars } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async readField(ask: { fixture?: string; bodies?: Json[] }): Promise<{ ok: true; bodies: Json[] } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "read_field", ...ask } as Json);
    return r.type === "field_read" ? { ok: true, bodies: (r as unknown as { bodies: Json[] }).bodies } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async knownTo(address: string, field?: string, at_s?: bigint): Promise<{ ok: true; value: import("./known.ts").KnownTo } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master sees who knows what" };
    const r = await this.#call({ op: "known_to", address, ...(field ? { field } : {}), ...(at_s !== undefined ? { at_s } : {}) } as unknown as Json);
    return r.type === "known_to" ? { ok: true, value: (r as unknown as { value: import("./known.ts").KnownTo }).value } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async readNames(address: string): Promise<{ ok: true; value: import("./names.ts").NamesRead } | { ok: false; code: string; message: string }> {
    const own = audienceOf(this.seat);
    const r = await this.#call({ op: "read_names", address, ...(this.seat.authority === "gm" ? {} : { audience: own }) } as Json);
    return r.type === "names" ? { ok: true, value: (r as unknown as { value: import("./names.ts").NamesRead }).value } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async nameSample(ask: import("./names.ts").SampleAsk): Promise<{ ok: true; value: import("./names.ts").NameSample } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "name_sample", ...ask } as Json);
    return r.type === "names" ? { ok: true, value: (r as unknown as { value: import("./names.ts").NameSample }).value } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async readMatter(ask: { table?: string; id?: string }): Promise<{ ok: true; value: Json } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "read_matter", ...ask } as Json);
    return r.type === "matter" ? { ok: true, value: (r as unknown as { value: Json }).value } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async windowView(ask: import("./window.ts").WindowAsk): Promise<{ ok: true; view: import("./window.ts").WindowView } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "window_view", ...ask });
    return r.type === "window_viewed" ? { ok: true, view: (r as unknown as { view: import("./window.ts").WindowView }).view } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async galaxyChart(ask: import("./galaxy.ts").GalaxyAsk): Promise<{ ok: true; chart: import("./galaxy.ts").GalaxyChart } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the galaxy's history is the game master's chart; players see it through data views" };
    const r = await this.#call({ op: "galaxy_chart", ...ask });
    return r.type === "galaxy_charted" ? { ok: true, chart: (r as unknown as { chart: import("./galaxy.ts").GalaxyChart }).chart } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async haloCell(ask: import("./galaxy.ts").HaloCellAsk): Promise<{ ok: true; cell: import("./galaxy.ts").HaloCell } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the universe's galaxies are charted on the game master's machine" };
    const r = await this.#call({ op: "halo_cell", ...ask });
    return r.type === "halo_celled" ? { ok: true, cell: (r as unknown as { cell: import("./galaxy.ts").HaloCell }).cell } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async starCell(ask: { cell?: [number, number, number]; t_myr?: number }): Promise<{ ok: true; cell: import("./voyage.ts").StarCell } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the galaxy's stars are drawn on the game master's machine" };
    const r = await this.#call({ op: "star_cell", ...ask });
    return r.type === "star_celled" ? { ok: true, cell: (r as unknown as { cell: import("./voyage.ts").StarCell }).cell } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  /** Effect at range (RK12, kernel `effect_at_range`): what arrives, the reading, the odds, and the rings. */
  async effectAtRange(asks: import("./range-rings.ts").RangeAsk[]): Promise<{ ok: true; effects: import("./range-rings.ts").RangeEffect[] } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "effect_at_range", asks } as unknown as Json);
    return r.type === "effects_at_range" ? { ok: true, effects: (r as unknown as { effects: import("./range-rings.ts").RangeEffect[] }).effects } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  /** A ship's power books (RK9 kernel `ship_power`): what it makes, what takes it, where it ends up. A ship is an id or a record; `flux_w_m2` is the starlight where it is. */
  async shipPower(ask: { ship: string | Json; flux_w_m2?: number; at_s?: bigint }): Promise<{ ok: true; power: import("./power.ts").PowerBooks } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "ship_power", ...ask } as unknown as Json);
    return r.type === "ship_power_is" ? { ok: true, power: (r as unknown as { power: import("./power.ts").PowerBooks }).power } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async planVoyage(ask: import("./voyage.ts").VoyageAsk): Promise<{ ok: true; plan: import("./voyage.ts").VoyagePlan | import("./voyage.ts").SystemVoyagePlan } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "voyages between stars are planned on the game master's machine for now" };
    const r = await this.#call({ op: "plan_voyage", ...ask });
    return r.type === "voyage_planned" ? { ok: true, plan: (r as unknown as { voyage: import("./voyage.ts").VoyagePlan }).voyage } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async templates(): Promise<{ ok: true; templates: import("./constructs.ts").ConstructTemplate[] } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "templates" });
    return r.type === "templates_listed" ? { ok: true, templates: (r as unknown as { templates: import("./constructs.ts").ConstructTemplate[] }).templates } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  /** A record write as a `record` intent (it travels to players' screens): the kernel's reply, or why not. */
  async #record(op: import("./intent.ts").RecordOp, ask: Record<string, unknown>): Promise<{ ok: true; reply: Record<string, Json> } | { ok: false; code: string; message: string }> {
    const plain = Object.fromEntries(Object.entries(ask).map(([k, v]) => [k, typeof v === "bigint" ? Number(v) : v])) as Json;
    const res = await this.send({ kind: "record", op, ask: plain });
    if (res.status === "applied") return { ok: true, reply: res.reply ?? {} };
    return { ok: false, code: res.status === "error" ? res.code : res.status, message: res.status === "error" ? res.message : res.status === "refused" ? res.reason : "waiting for the game master" };
  }

  async createFrom(ask: { template: string; under: string; key?: string; name?: string; at_s?: number | bigint }): Promise<{ ok: true; result: import("./constructs.ts").Created } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "ships and stations are made on the game master's machine" };
    const w = await this.#record("create", { template: ask.template, under: ask.under, ...(ask.key ? { key: ask.key } : {}), ...(ask.name ? { name: ask.name } : {}), at_s: ask.at_s ?? 0 });
    return !w.ok ? w : w.reply.type === "created" ? { ok: true, result: w.reply as unknown as import("./constructs.ts").Created } : { ok: false, code: "unexpected", message: JSON.stringify(w.reply) };
  }

  async retire(address: string, reason?: string, at_s?: number | bigint): Promise<{ ok: true; event_id: string } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master takes things out of play" };
    const w = await this.#record("retire", { address, ...(reason ? { reason } : {}), at_s: at_s ?? 0 });
    return !w.ok ? w : w.reply.type === "retired" ? { ok: true, event_id: String(w.reply.event_id) } : { ok: false, code: "unexpected", message: JSON.stringify(w.reply) };
  }

  async addNote(n: { address: string; text: string; audience: "everyone" | "gm" | string[]; tags?: string[]; narrative?: boolean; supersedes?: string; at_s?: number | bigint }): Promise<{ ok: true; note_id: string } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the game master writes the record; seats ask" };
    const w = await this.#record("add_note", { address: n.address, text: n.text, audience: n.audience, ...(n.tags?.length ? { tags: n.tags } : {}), ...(n.narrative ? { narrative: true } : {}), ...(n.supersedes ? { supersedes: n.supersedes } : {}), at_s: n.at_s ?? 0 });
    return !w.ok ? w : w.reply.type === "note_added" ? { ok: true, note_id: String(w.reply.note_id) } : { ok: false, code: "unexpected", message: JSON.stringify(w.reply) };
  }

  async addActor(a: { id: string; name: string; kind: string; parent?: string; at_s?: number | bigint }): Promise<{ ok: true; address: string } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the game master writes the record; seats ask" };
    const w = await this.#record("add_actor", { id: a.id, name: a.name, kind: a.kind, ...(a.parent ? { parent: a.parent } : {}), at_s: a.at_s ?? 0 });
    return !w.ok ? w : w.reply.type === "actor_added" ? { ok: true, address: String(w.reply.address) } : { ok: false, code: "unexpected", message: JSON.stringify(w.reply) };
  }

  async setKept(k: { address: string; kind: string; value: Json | null; reason?: string; at_s?: number | bigint }): Promise<{ ok: true; change_id: string } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the game master writes the record; seats ask" };
    const w = await this.#record("set_kept", { address: k.address, kind: k.kind, value: k.value, ...(k.reason ? { reason: k.reason } : {}), at_s: k.at_s ?? 0 });
    if (!w.ok) return w;
    return w.reply.type === "kept_set" ? { ok: true, change_id: String(w.reply.change_id) } : { ok: false, code: "unexpected", message: JSON.stringify(w.reply) };
  }

  async record(address: string, at_s?: number | bigint): Promise<{ ok: true; record: import("./constructs.ts").RecordRead } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the record is read on the game master's machine; seats get what is disclosed to them" };
    const r = await this.#call({ op: "record", address, ...(at_s !== undefined ? { at_s } : {}) });
    return r.type === "record_read" ? { ok: true, record: r as unknown as import("./constructs.ts").RecordRead } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async codes(ask: { scope: string; sheet?: string; at_s?: number | bigint }): Promise<{ ok: true; result: import("./sector.ts").CodesRead } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "codes are read on the game master's machine" };
    const r = await this.#call({ op: "codes", scope: ask.scope, ...(ask.sheet ? { sheet: ask.sheet } : {}), at_s: ask.at_s ?? 0 });
    return r.type === "codes" ? { ok: true, result: r as unknown as import("./sector.ts").CodesRead } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async sectorMap(ask: { scope: string; sheet?: string; at_s?: number | bigint }): Promise<{ ok: true; result: import("./sector.ts").SectorMap } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the sector map is read on the game master's machine" };
    const r = await this.#call({ op: "sector_map", scope: ask.scope, ...(ask.sheet ? { sheet: ask.sheet } : {}), at_s: ask.at_s ?? 0 });
    return r.type === "sector_mapped" ? { ok: true, result: r as unknown as import("./sector.ts").SectorMap } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async importSector(ask: import("./sector.ts").SectorImportAsk): Promise<{ ok: true; result: import("./sector.ts").SectorImported } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "sectors are imported on the game master's machine" };
    const r = await this.#call({ op: "import_sector", text: ask.text, ...(ask.format ? { format: ask.format } : {}), ...(ask.sheet ? { sheet: ask.sheet } : {}), apply: !!ask.apply, grow: !!ask.grow, at_s: ask.at_s ?? 0 });
    return r.type === "sector_import" ? { ok: true, result: r as unknown as import("./sector.ts").SectorImported } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async exportSector(ask: { scope: string; format: import("./sector.ts").SectorFormat; sheet?: string; at_s?: number | bigint }): Promise<{ ok: true; result: import("./sector.ts").SectorExported } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "sectors are written on the game master's machine" };
    const r = await this.#call({ op: "export_sector", scope: ask.scope, format: ask.format, ...(ask.sheet ? { sheet: ask.sheet } : {}), at_s: ask.at_s ?? 0 });
    return r.type === "sector_export" ? { ok: true, result: r as unknown as import("./sector.ts").SectorExported } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async webSlice(ask: import("./galaxy.ts").SliceAsk): Promise<{ ok: true; web: import("./galaxy.ts").WebSlice } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the cosmic web is read on the game master's machine" };
    const r = await this.#call({ op: "web_slice", ...ask });
    return r.type === "web_sliced" ? { ok: true, web: (r as unknown as { web: import("./galaxy.ts").WebSlice }).web } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async webCube(ask: import("./galaxy.ts").SliceAsk): Promise<{ ok: true; web: import("./galaxy.ts").WebCube } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the cosmic web is read on the game master's machine" };
    const r = await this.#call({ op: "web_slice", cube: true, ...ask });
    return r.type === "web_sliced" ? { ok: true, web: (r as unknown as { web: import("./galaxy.ts").WebCube }).web } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async webGalaxies(ask: import("./galaxy.ts").SliceAsk): Promise<{ ok: true; web: import("./galaxy.ts").WebGalaxies } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "the cosmic web is read on the game master's machine" };
    const r = await this.#call({ op: "web_slice", cube: true, haloes: true, ...ask });
    return r.type === "web_sliced" ? { ok: true, web: (r as unknown as { web: import("./galaxy.ts").WebGalaxies }).web } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async lifeView(by: { body?: string; fixture?: string; dials?: Record<string, number> }): Promise<{ ok: true; source: string; view: LifeView } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master reads a world's whole biosphere; players see it through data views" };
    const r = await this.#call({ op: "life_view", ...(by.body !== undefined ? { body: by.body } : {}), ...(by.fixture !== undefined ? { fixture: by.fixture } : {}), ...(by.dials !== undefined ? { dials: by.dials } : {}) });
    return r.type === "life_viewed" ? { ok: true, source: String(r.source), view: (r as unknown as { view: LifeView }).view } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async orbitsAt(system: string, at_s: readonly bigint[], bodies?: { address: string; facts: Record<string, number | string | bigint> }[]): Promise<{ ok: true; bodies: OrbitPlaced[] } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "orbits_at", system, at_s: [...at_s], ...(bodies ? { bodies } : {}) });
    return r.type === "orbits_placed" ? { ok: true, bodies: (r as unknown as { bodies: OrbitPlaced[] }).bodies } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async campaignNow(): Promise<{ ok: true; at_s?: bigint } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "campaign_now" });
    if (r.type !== "campaign_now_is") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const at = (r as unknown as { at_s?: number | bigint | string }).at_s;
    return at === undefined || at === null ? { ok: true } : { ok: true, at_s: BigInt(at) };
  }

  async setCampaignNow(at_s: bigint): Promise<{ ok: true; state_hash: string } | { ok: false; code: string; message: string }> {
    if (this.seat.authority !== "gm") return { ok: false, code: "not_gm", message: "only the game master moves the campaign's now" };
    const r = await this.#call({ op: "set_campaign_now", at_s });
    return r.type === "campaign_now_is" ? { ok: true, state_hash: String((r as unknown as { state_hash?: string }).state_hash ?? "") } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
  }

  async calendar(): Promise<{ ok: true; calendar?: PackCalendar } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "calendar" });
    if (r.type !== "calendar_is") return { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
    const c = (r as unknown as { calendar?: PackCalendar }).calendar;
    return c ? { ok: true, calendar: c } : { ok: true };
  }

  async linksIn(ask: { bounds?: string; audience?: string } = {}): Promise<{ ok: true; links: import("./your-systems.ts").RecordLink[] } | { ok: false; code: string; message: string }> {
    const r = await this.#call({ op: "links_in", ...ask });
    return r.type === "links_listed" ? { ok: true, links: (r as unknown as { links: import("./your-systems.ts").RecordLink[] }).links } : { ok: false, code: String(r.code ?? "unexpected"), message: String(r.message ?? JSON.stringify(r)) };
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
