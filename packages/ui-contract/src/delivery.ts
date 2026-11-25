// How a screen arrives and leaves: screen delivery, part of a look (atlas design 11A.4a, AT15a; Playbook design 5.2,
// PB10). Alex, 24 Nov 20:08: "reuse some of the stuff we had around transitions... greenscreens building up rather than
// just flicking - the slow warm up and the disappear to a dot." SSE3 had wipe, fade, crt_collapse, scanline and
// static_dissolve (0.7 to 2 s). SSE4 keeps one mechanism: a look names a delivery for each of five moments, and every
// screen plays it the same way.
//
// This file is data and rules only: which deliveries exist, what each moment of a look asks for, and how that becomes a
// plan to play (with the short form for later arrivals and turns, and a plain fade under reduced motion). Playing it is
// chart-grammar's deliver.ts. Print ignores all of it.

/** The five moments a look can dress. */
export const MOMENTS = ["power_on", "arrive", "turn", "leave", "power_off"] as const;
export type Moment = (typeof MOMENTS)[number];

/** Moments where something goes away rather than arrives. */
export const EXIT_MOMENTS: readonly Moment[] = ["leave", "power_off"];

export const MOMENT_WORDS: Record<Moment, string> = {
  power_on: "Power on",
  arrive: "Arrive",
  turn: "Turn a page",
  leave: "Leave",
  power_off: "Power off",
};

/** How much of the screen a delivery acts on at once. */
export const DELIVERY_SCALES = ["screen", "parts", "inside"] as const;
export type DeliveryScale = (typeof DELIVERY_SCALES)[number];

export const DELIVERY_KINDS = [
  "none",
  "fade",
  "wipe",
  "stepped_wipe",
  "build_up",
  "type_on",
  "scan_in",
  "warm_up",
  "collapse_dot",
  "static_burst",
  "dissolve_blocks",
  "slide",
  "page_turn",
  "iris",
] as const;
export type DeliveryKind = (typeof DELIVERY_KINDS)[number];

export type Direction = "down" | "up" | "left" | "right";

/** One delivery as a look names it: a kind and a few settings, all optional but the kind. */
export interface Delivery {
  kind: DeliveryKind;
  /** Full length in milliseconds. */
  ms?: number;
  /** The short form's length (later arrivals, every Turn). */
  short_ms?: number;
  /** Screen, each part in turn, or inside each part. */
  scale?: DeliveryScale;
  /** Wipe, slide, stepped wipe: which way the picture comes in. */
  direction?: Direction;
  /** Stepped wipe: how many steps (a cheap screen redrawing in bands). */
  steps?: number;
  /** Type on: characters a second. */
  cps?: number;
  /** What follows when this one ends (Warm up, then Build up). */
  then?: Delivery;
}

/** A look's deliveries, one per moment; a moment left out takes the default (Fade, or nothing to leave). */
export type LookDelivery = Partial<Record<Moment, Delivery>>;

export interface DeliveryInfo {
  kind: DeliveryKind;
  words: string;
  /** What it looks like, for the look card and for Orac. */
  says: string;
  ms: number;
  short_ms: number;
  /** The scales it can act at. */
  scales: readonly DeliveryScale[];
  /** It can play as a picture goes away (Leave, Power off). */
  exits: boolean;
  /** It can play as a picture arrives. */
  enters: boolean;
}

/** Every delivery in the toolkit, with its defaults. */
export const DELIVERIES: Record<DeliveryKind, DeliveryInfo> = {
  none: { kind: "none", words: "None", says: "The picture is simply there.", ms: 0, short_ms: 0, scales: ["screen"], exits: true, enters: true },
  fade: { kind: "fade", words: "Fade", says: "The picture fades in or out.", ms: 400, short_ms: 180, scales: ["screen", "parts", "inside"], exits: true, enters: true },
  wipe: { kind: "wipe", words: "Wipe", says: "An edge sweeps across, uncovering the picture.", ms: 700, short_ms: 260, scales: ["screen", "parts"], exits: true, enters: true },
  stepped_wipe: { kind: "stepped_wipe", words: "Stepped wipe", says: "A cheap screen redrawing in bands, top to bottom.", ms: 600, short_ms: 280, scales: ["screen", "parts"], exits: true, enters: true },
  build_up: { kind: "build_up", words: "Build up", says: "The frame, then lines, then labels and figures, top to bottom, as a terminal draws.", ms: 1400, short_ms: 360, scales: ["screen", "parts", "inside"], exits: false, enters: true },
  type_on: { kind: "type_on", words: "Type on", says: "Text arrives a character at a time.", ms: 1200, short_ms: 320, scales: ["inside"], exits: false, enters: true },
  scan_in: { kind: "scan_in", words: "Scan in", says: "A bright line sweeps down, leaving the picture behind it.", ms: 900, short_ms: 300, scales: ["screen", "parts"], exits: false, enters: true },
  warm_up: { kind: "warm_up", words: "Warm up", says: "Black, a faint glow that swells and steadies, like an old tube warming.", ms: 2000, short_ms: 400, scales: ["screen"], exits: false, enters: true },
  collapse_dot: { kind: "collapse_dot", words: "Collapse to a dot", says: "The picture squeezes to a bright line, the line to a dot, the dot fades.", ms: 700, short_ms: 300, scales: ["screen"], exits: true, enters: false },
  static_burst: { kind: "static_burst", words: "Static burst", says: "A burst of static, and the picture underneath.", ms: 600, short_ms: 240, scales: ["screen"], exits: true, enters: true },
  dissolve_blocks: { kind: "dissolve_blocks", words: "Dissolve in blocks", says: "The picture appears block by block.", ms: 800, short_ms: 300, scales: ["screen"], exits: true, enters: true },
  slide: { kind: "slide", words: "Slide", says: "Parts ease up into place.", ms: 450, short_ms: 220, scales: ["screen", "parts"], exits: true, enters: true },
  page_turn: { kind: "page_turn", words: "Page turn", says: "The paper curls over from its bottom outer corner, as if leafed by hand, showing its pale back and casting a soft shadow.", ms: 700, short_ms: 700, scales: ["screen"], exits: false, enters: true },
  iris: { kind: "iris", words: "Iris", says: "A circle opens from the middle.", ms: 700, short_ms: 260, scales: ["screen"], exits: true, enters: true },
};

/** Turn is never longer than this, so stepping through pages is never a wait (11A.4a rule 2). */
export const TURN_MAX_MS = 400;
/**
 * Page turn alone may take longer (Alex, 25 Nov: "have the paper curl as you turn it - less mechanical - like you are
 * leafing through it"; atlas design 14A note 5: about 0.7 s). It is still never a wait: a tap finishes it at once.
 */
export const PAGE_TURN_MAX_MS = 800;
/** Under reduced motion every delivery is this plain fade (rule 3). */
export const REDUCED_FADE_MS = 150;
/** No delivery longer than this, whatever a look says. */
export const DELIVERY_MAX_MS = 4000;

/** A delivery ready to play. */
export interface DeliveryPlan {
  kind: DeliveryKind;
  moment: Moment;
  /** True when the picture is going away. */
  exit: boolean;
  ms: number;
  scale: DeliveryScale;
  direction: Direction;
  steps: number;
  cps: number;
  then?: DeliveryPlan;
}

export interface ResolveOptions {
  /** The first time this screen gets this send: play the full form (rule 2). */
  first?: boolean;
  reducedMotion?: boolean;
  /** Stepping backwards (a page turns the other way). */
  back?: boolean;
}

const clampMs = (x: number) => Math.max(0, Math.min(DELIVERY_MAX_MS, Math.round(Number.isFinite(x) ? x : 0)));

/**
 * What to play at a moment: the look's choice with its defaults filled in, in its full or short form. Power on and the
 * first Arrive play in full; a later Arrive and every Turn play short (Turn capped at TURN_MAX_MS). Reduced motion makes
 * every delivery a plain fade. A moment the look leaves out arrives by Fade and leaves with nothing.
 */
export function resolveDelivery(look: LookDelivery | undefined, moment: Moment, opts: ResolveOptions = {}): DeliveryPlan {
  const exit = EXIT_MOMENTS.includes(moment);
  const fallback: Delivery = { kind: exit ? "none" : "fade" };
  let d = look?.[moment] ?? fallback;
  const info = DELIVERIES[d.kind];
  // A look naming an arriving delivery for a leaving moment (or the other way) gets the safe default instead.
  if (!info || (exit && !info.exits) || (!exit && !info.enters)) d = fallback;
  if (opts.reducedMotion) return plan({ kind: d.kind === "none" ? "none" : "fade", ms: REDUCED_FADE_MS }, moment, exit, false, opts.back);
  const short = moment === "turn" || (moment === "arrive" && opts.first === false);
  return plan(d, moment, exit, short, opts.back);
}

function plan(d: Delivery, moment: Moment, exit: boolean, short: boolean, back = false): DeliveryPlan {
  const info = DELIVERIES[d.kind];
  let ms = clampMs(short ? (d.short_ms ?? info.short_ms) : (d.ms ?? info.ms));
  if (moment === "turn") ms = Math.min(ms, d.kind === "page_turn" ? PAGE_TURN_MAX_MS : TURN_MAX_MS);
  const scale = d.scale && info.scales.includes(d.scale) ? d.scale : info.scales[0]!;
  const dir: Direction = d.direction ?? (d.kind === "wipe" || d.kind === "slide" ? (d.kind === "slide" ? "up" : "right") : "down");
  const out: DeliveryPlan = {
    kind: d.kind,
    moment,
    exit,
    ms,
    scale,
    direction: back ? opposite(dir) : dir,
    steps: Math.max(2, Math.min(24, Math.round(d.steps ?? 6))),
    cps: Math.max(10, Math.min(400, d.cps ?? 90)),
  };
  // A follow-on plays only in the full form: the short form is one quick thing.
  if (d.then && !short && !exit) out.then = plan(d.then, moment, exit, false, back);
  return out;
}

function opposite(d: Direction): Direction {
  return d === "left" ? "right" : d === "right" ? "left" : d === "up" ? "down" : "up";
}

/** The whole time a plan takes, follow-ons included. */
export function planMs(p: DeliveryPlan): number {
  return p.ms + (p.then ? planMs(p.then) : 0);
}

/** Problems with a look's deliveries, in words (for the loader and for Orac); empty when it is good. */
export function checkLookDelivery(look: unknown): string[] {
  const errs: string[] = [];
  if (look === undefined) return errs;
  if (!look || typeof look !== "object" || Array.isArray(look)) return ["delivery must be an object of moments"];
  for (const [m, d] of Object.entries(look as Record<string, unknown>)) {
    if (!(MOMENTS as readonly string[]).includes(m)) {
      errs.push(`delivery: "${m}" is not a moment (${MOMENTS.join(", ")})`);
      continue;
    }
    checkOne(d, `delivery.${m}`, EXIT_MOMENTS.includes(m as Moment), errs, 0);
  }
  return errs;
}

function checkOne(d: unknown, at: string, exit: boolean, errs: string[], depth: number) {
  if (!d || typeof d !== "object") return void errs.push(`${at} must be an object with a kind`);
  const o = d as Record<string, unknown>;
  const info = DELIVERIES[o.kind as DeliveryKind];
  if (!info) return void errs.push(`${at}: "${String(o.kind)}" is not a delivery (${DELIVERY_KINDS.join(", ")})`);
  if (exit && !info.exits) errs.push(`${at}: ${info.words} only plays as a picture arrives`);
  if (!exit && !info.enters) errs.push(`${at}: ${info.words} only plays as a picture goes away`);
  for (const k of ["ms", "short_ms"] as const) {
    const v = o[k];
    if (v !== undefined && (typeof v !== "number" || !(v >= 0) || v > DELIVERY_MAX_MS)) errs.push(`${at}.${k} must be 0 to ${DELIVERY_MAX_MS} ms`);
  }
  if (o.scale !== undefined && !info.scales.includes(o.scale as DeliveryScale)) errs.push(`${at}.scale: ${info.words} works at ${info.scales.join(", ")}`);
  if (o.direction !== undefined && !["down", "up", "left", "right"].includes(o.direction as string)) errs.push(`${at}.direction must be down, up, left or right`);
  const known = new Set(["kind", "ms", "short_ms", "scale", "direction", "steps", "cps", "then"]);
  for (const k of Object.keys(o)) if (!known.has(k)) errs.push(`${at}: unknown setting "${k}"`);
  if (o.then !== undefined) {
    if (depth >= 2) errs.push(`${at}.then: at most two follow-ons`);
    else checkOne(o.then, `${at}.then`, exit, errs, depth + 1);
  }
}

/**
 * The starters' deliveries (11A.3, 11A.4a). Hauler warms up then builds up, types on as pages turn, and collapses to a
 * dot. Corporate fades with sliding cards and opens by iris. Deck lights its panels in turn and scans in. Paper editions
 * turn the page. The Guide is cheap electronics, not paper: it redraws in bands (a stepped wipe).
 */
export const STARTER_DELIVERIES: Record<"hauler" | "corporate" | "deck" | "paper" | "guide", LookDelivery> = {
  hauler: {
    power_on: { kind: "warm_up", ms: 2000, then: { kind: "build_up", scale: "inside", ms: 1400 } },
    arrive: { kind: "build_up", scale: "inside" },
    turn: { kind: "type_on", short_ms: 320 },
    leave: { kind: "none" },
    power_off: { kind: "collapse_dot" },
  },
  corporate: {
    power_on: { kind: "iris" },
    arrive: { kind: "slide", scale: "parts" },
    turn: { kind: "fade" },
    leave: { kind: "fade" },
    power_off: { kind: "iris" },
  },
  deck: {
    power_on: { kind: "build_up", scale: "parts" },
    arrive: { kind: "scan_in", scale: "parts" },
    turn: { kind: "fade" },
    leave: { kind: "none" },
    power_off: { kind: "fade" },
  },
  paper: {
    power_on: { kind: "fade" },
    arrive: { kind: "fade" },
    turn: { kind: "page_turn" },
    leave: { kind: "none" },
    power_off: { kind: "fade" },
  },
  guide: {
    power_on: { kind: "stepped_wipe", steps: 8, ms: 700 },
    arrive: { kind: "stepped_wipe", steps: 6 },
    turn: { kind: "stepped_wipe", steps: 4, short_ms: 240 },
    leave: { kind: "none" },
    power_off: { kind: "fade" },
  },
};
