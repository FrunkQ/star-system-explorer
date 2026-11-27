// The three desks as data (track TM1, three desks design section 10, accepted by Alex 25 Nov 2026 13:30 UTC).
//
// A desk is three answers, the same on every display (design section 1): where a change goes (`writes`), what time
// can do (`time`) and how much is on screen (the `desks` field on everything that can be on screen). The three shipped
// desks are three `sse.desk/1` records, and the desk switch reads its words from them, so renaming a desk is a change
// of data.
//
// "Is this on screen on this desk?" has one answer, `shows(thing, desk)`. A rail entry (`RailEntry.desks`), a part's
// control (`PartControl.desks`) and the shell's own parts (`SHELL_PARTS` below) carry the field; left out, the thing is
// on all three desks. No screen tests the desk by name: `tests/ui/desk-ledger.test.ts` counts the places that still do,
// and the count only goes down.
//
// SHELL_PARTS says what each part does on screen now. TM1 wrote it as the shell was (design section 2); TM2 moves each
// row to its section 3.1 desks as the shell starts reading `partShows()` for it, one small change at a time.

import type { PartControl } from "./part-controls.ts";

/** The desks, in the order the switch shows them. */
export const DESKS = ["explore", "build", "run"] as const;
export type Desk = (typeof DESKS)[number];

export const DESK_SCHEMA = "sse.desk/1";

/** Where a change goes: onto a what-if and not kept; into the campaign; or only the acts of play (clock, reveal, send, fuel, crew, cargo). */
export const DESK_WRITES = ["try", "campaign", "play"] as const;
export type DeskWrites = (typeof DESK_WRITES)[number];
/** What time can do: look anywhere; look and set dates; the campaign's now in human time. */
export const DESK_TIMES = ["look", "set", "clock"] as const;
export type DeskTime = (typeof DESK_TIMES)[number];

export interface DeskDoc {
  schema: typeof DESK_SCHEMA;
  id: Desk;
  /** As the desk switch says it. */
  label: string;
  /** The key that switches to it (one letter). */
  key: string;
  /** The switch's tooltip. */
  hint: string;
  /** The desk in one line (design 1.3), for the Reference and Orac. */
  line: string;
  writes: DeskWrites;
  time: DeskTime;
  /** The lens the desk opens with; left out: the device's default lens. */
  lens?: string;
}

/**
 * The three shipped desks. The hints are the switch's words today; `writes` and `time` are the accepted design's
 * answers. The shell reads `writes` as Build arms editing ("campaign"); on Explore ("try") the handles and grips are live
 * and the first change opens a what-if by itself (`deskTries`, TM4); TM3 makes the time answers true on screen.
 */
export const SHIPPED_DESKS: readonly DeskDoc[] = [
  { schema: DESK_SCHEMA, id: "explore", label: "Explore", key: "V", hint: "Look around and try anything: nothing is kept by accident (V)", line: "Try anything, nothing is kept.", writes: "try", time: "look" },
  { schema: DESK_SCHEMA, id: "build", label: "Build", key: "E", hint: "Change things: values edit, bodies drag, undo and redo (E)", line: "What you change is the campaign.", writes: "campaign", time: "set", lens: "building" },
  { schema: DESK_SCHEMA, id: "run", label: "Run", key: "R", hint: "At the table: what players know, their screens and requests (R)", line: "The picture, the place and the clock.", writes: "play", time: "clock", lens: "running" },
];

export function isDesk(v: unknown): v is Desk {
  return typeof v === "string" && (DESKS as readonly string[]).includes(v);
}

/** A shipped desk's record. */
export function deskDoc(id: Desk): DeskDoc {
  return SHIPPED_DESKS.find((d) => d.id === id)!;
}

/** Whether a desk's changes go into the campaign (today: whether the chart is armed for editing). */
export function deskKeeps(id: Desk): boolean {
  return deskDoc(id).writes === "campaign";
}

/**
 * Whether a desk's changes are a try (Explore, three desks TM4): its handles and grips are live, and the first change
 * opens a what-if by itself, so nothing reaches the campaign unless "Keep it" is pressed. Left out, no desk (a player).
 */
export function deskTries(id: Desk | undefined): boolean {
  return !!id && deskDoc(id).writes === "try";
}

/** Whether things can be changed on a desk at all, kept or tried (Build, Explore); Run changes only the acts of play. */
export function deskChanges(id: Desk | undefined): boolean {
  return !!id && deskDoc(id).writes !== "play";
}

/** Problems with a desk record; empty when it is fine. */
export function checkDesk(doc: unknown): string[] {
  if (!doc || typeof doc !== "object" || Array.isArray(doc)) return ["a desk must be an object"];
  const d = doc as Partial<DeskDoc>;
  const errors: string[] = [];
  if (d.schema !== DESK_SCHEMA) errors.push(`unknown desk schema ${String(d.schema)}`);
  if (!isDesk(d.id)) errors.push(`unknown desk ${String(d.id)} (desks are ${DESKS.join(", ")})`);
  if (typeof d.label !== "string" || !d.label.trim()) errors.push("a desk needs a label");
  if (typeof d.key !== "string" || !/^[A-Z]$/.test(d.key)) errors.push("a desk's key is one capital letter");
  if (typeof d.hint !== "string" || !d.hint.trim()) errors.push("a desk needs a hint");
  if (typeof d.line !== "string" || !d.line.trim()) errors.push("a desk needs its one line");
  if (!(DESK_WRITES as readonly string[]).includes(d.writes as string)) errors.push(`writes must be one of ${DESK_WRITES.join(", ")}`);
  if (!(DESK_TIMES as readonly string[]).includes(d.time as string)) errors.push(`time must be one of ${DESK_TIMES.join(", ")}`);
  if (d.lens !== undefined && (typeof d.lens !== "string" || !d.lens)) errors.push("a desk's lens is a lens id");
  return errors;
}

/** Anything that can be on screen: it says which desks it shows on; left out, all three. */
export interface OnDesks {
  desks?: readonly Desk[];
}

/** The one answer to "is this on screen on this desk?". */
export function shows(thing: OnDesks | undefined, desk: Desk): boolean {
  return !thing?.desks || thing.desks.includes(desk);
}

/** Problems with a `desks` field (`where` names the thing); empty when it is fine or left out. */
export function checkDesks(desks: unknown, where: string): string[] {
  if (desks === undefined) return [];
  if (!Array.isArray(desks) || !desks.length) return [`${where}: desks must be a list of one or more of ${DESKS.join(", ")} (leave it out for all three)`];
  const errors: string[] = [];
  for (const d of desks) if (!isDesk(d)) errors.push(`${where}: unknown desk ${String(d)}`);
  if (new Set(desks).size !== desks.length) errors.push(`${where}: a desk is listed twice`);
  return errors;
}

/** The desks a thing is on, in words, when it is not on all three ("Build and Run"); empty when it is on all three. */
export function deskWords(thing: OnDesks | undefined): string {
  const on = DESKS.filter((d) => shows(thing, d)).map((d) => deskDoc(d).label);
  if (on.length === DESKS.length) return "";
  return on.length > 1 ? `${on.slice(0, -1).join(", ")} and ${on[on.length - 1]}` : (on[0] ?? "no desk");
}

/** One of the shell's own parts round the picture (design 3.1). */
export interface ShellPart extends OnDesks {
  id: string;
  /** In plain words, for the Reference and Orac. */
  words: string;
  /** The part's own presses, where they differ by desk: the same shape as a player part's controls (AT13). */
  controls?: readonly PartControl[];
}

/**
 * The shell's parts and the desks each shows on, as the shell draws them now. TM2a gave the rail entries their desks;
 * TM2b took the depth dial, the Workshop door, Select, Keyboard shortcuts, the seat and the engine line off Run; TM2c folded the rail on Run; TM3a put the clock in the ruler's place on Run and took Light with it; TM7 made the place line the stepper on Run. The rest
 * are on every desk until their package says otherwise. A form that changes on one desk is a part of its own (the rail
 * open, the stepper), so the shell asks `partShows` and never tests the desk.
 */
export const SHELL_PARTS: readonly ShellPart[] = [
  { id: "place-line", words: "The place line: where you are, step by step" },
  // On Run the place line is the main way to move (design 3.2, TM7): every level drawn, Out and In at its end.
  { id: "stepper", words: "The stepper: the place line with every level drawn, and Out and In (Page Up, Page Down)", desks: ["run"] },
  { id: "subtitle", words: "The line under the name: the place's subtitle and the date" },
  { id: "desk-switch", words: "The desk switch: Explore, Build, Run" },
  { id: "see-as", words: "See as: the lens the chart is seen through" },
  // On Run the dial leaves the bar and is in More (design 3.1); More always carries it on Run.
  { id: "depth", words: "Depth: how much every card, number and Why? says (on Run, in More)", desks: ["explore", "build"] },
  { id: "ask", words: "Ask: Orac and Minerva" },
  { id: "more", words: "More: campaign, Workshop, help and the rest" },
  { id: "rail", words: "The instrument rail" },
  // On Run the rail folds to one button at the head of the view controls that names the instrument in view, and opens
  // the rail as a list in words (design 3.1, question 6); the picture gets the width. A phone keeps its chips (TM9).
  { id: "rail-open", words: "The rail open beside the picture (on Run, folded to one button at the head of the view controls)", desks: ["explore", "build"] },
  // The Workshop changes the tool, not the universe: on Run its door is in More only (design section 5).
  { id: "workshop-door", words: "The Workshop door at the foot of the rail (on Run, in More)", desks: ["explore", "build"] },
  { id: "view-controls", words: "View style, Seen from, Scale, HUD and Band" },
  {
    id: "tools",
    words: "The stage's tools",
    controls: [
      { id: "select", words: "Select", kind: "button", default: true, desks: ["explore", "build"] },
      { id: "undo", words: "Undo (also on Explore while a what-if is open)", kind: "button", default: true, desks: ["build"] },
      { id: "redo", words: "Redo (also on Explore while a what-if is open)", kind: "button", default: true, desks: ["build"] },
      { id: "copy", words: "Copy", kind: "button", default: true, desks: ["build"] },
      { id: "add", words: "Add here", kind: "button", default: true, desks: ["build"] },
      { id: "cut", words: "Cut", kind: "button", default: true, desks: ["build"] },
      { id: "keys", words: "Keyboard shortcuts (the ? key still opens them on Run)", kind: "button", default: true, desks: ["explore", "build"] },
    ],
  },
  { id: "card", words: "The card for the thing selected" },
  { id: "eyes", words: "The \"Known to\" eyes and pencil", desks: ["build", "run"] },
  { id: "what-if-stamp", words: "The what-if stamp and dashed frame" },
  // On Run the clock takes the ruler's place (design 7.3, TM3a): the date, step chips and Run ▸; no deep time, What if
  // or Look ahead. Light goes with the ruler (on Run it is the Band control); Sound stays beside the clock.
  { id: "ruler", words: "The time ruler, with What if and Look ahead (on Run, the clock)", desks: ["explore", "build"] },
  { id: "clock", words: "The campaign clock: the date, step chips and Run ▸", desks: ["run"] },
  { id: "light", words: "Light (on Run, the Band control)", desks: ["explore", "build"] },
  { id: "sound", words: "Sound" },
  // F on the Run desk (design question 11, TM9a): the picture alone; the bar comes back at the top edge.
  { id: "fill-screen", words: "Fill the screen (F): the picture alone, the bar back at the top edge", desks: ["run"] },
  { id: "card-sheet", words: "On a phone, the card as a sheet that comes up only on a press, with Close", desks: ["run"] },
  {
    id: "status-line",
    words: "The status line",
    // On Run it says whether the campaign is kept, and the table's own line; the on-air chip is in the bar (design 3.1).
    controls: [
      { id: "seat", words: "Who is at this screen", kind: "link", default: true, desks: ["explore", "build"] },
      { id: "kept", words: "Whether the campaign is kept", kind: "link", default: true },
      { id: "engine", words: "The engine's numbers, by depth", kind: "link", default: true, desks: ["explore", "build"] },
    ],
  },
];

/** A shell part by id (throws on an unknown id, so a typo fails its test, not the screen). */
export function shellPart(id: string): ShellPart {
  const p = SHELL_PARTS.find((x) => x.id === id);
  if (!p) throw new Error(`no shell part ${id}`);
  return p;
}

/** Whether one of the shell's parts, or one of its controls, is on screen on this desk. */
export function partShows(id: string, desk: Desk, control?: string): boolean {
  const p = shellPart(id);
  if (!shows(p, desk)) return false;
  if (control === undefined) return true;
  const c = p.controls?.find((x) => x.id === control);
  if (!c) throw new Error(`shell part ${id} has no control ${control}`);
  return shows(c, desk);
}

/** The seven levels of the place line (design 3.2). Local stars and Site are new steps (grand zoom sections 8 and 9). */
export const DESK_LEVELS = ["web", "cluster", "galaxy", "local", "system", "world", "site"] as const;
export type DeskLevel = (typeof DESK_LEVELS)[number];

/**
 * What each desk remembers on this device (design question 9): the view choice (View style, Seen from, Scale, HUD,
 * Band, Motion, Aids) and the ruler's span for each level, and the depth for the desk as a whole.
 */
export const DESK_MEMORY = { view: "level", "ruler-span": "level", depth: "desk" } as const;
export type DeskMemory = keyof typeof DESK_MEMORY;

/**
 * The device key a desk remembers one thing under ("desk.run.system.view", "desk.run.depth"), for `loadPref` and
 * `savePref`. A per-level thing needs its level; a per-desk thing takes none.
 */
export function deskMemoryKey(desk: Desk, what: DeskMemory, level?: DeskLevel): string {
  const per = DESK_MEMORY[what];
  if (per === "level" && level === undefined) throw new Error(`${what} is remembered per level: give the level`);
  if (per === "desk" && level !== undefined) throw new Error(`${what} is remembered per desk, not per level`);
  return level === undefined ? `desk.${desk}.${what}` : `desk.${desk}.${level}.${what}`;
}

/** The level a rail scale remembers under; the Workshop changes the tool, not the universe, and has none. */
export function deskLevelOf(scale: string): DeskLevel | null {
  return (DESK_LEVELS as readonly string[]).includes(scale) ? (scale as DeskLevel) : null;
}

/**
 * What a desk remembers, read back (TM6, design question 9): the desk's own value if it has one, else the last value
 * chosen anywhere on this device (the plain key), so a desk or level visited for the first time starts from the
 * viewer's latest choice rather than from nothing. `load` is the shell's `loadPref`.
 */
export function recallForDesk(load: (key: string) => string | null, desk: Desk, what: DeskMemory, level?: DeskLevel | null): { value: string | null; own: boolean } {
  if (DESK_MEMORY[what] === "level" && !level) return { value: load(what), own: false };
  const own = load(deskMemoryKey(desk, what, DESK_MEMORY[what] === "level" ? level! : undefined));
  return own !== null ? { value: own, own: true } : { value: load(what), own: false };
}

/** The keys a choice is kept under: the desk's own (when it has a level, or needs none) and the plain key. */
export function keepForDesk(desk: Desk, what: DeskMemory, level?: DeskLevel | null): string[] {
  if (DESK_MEMORY[what] === "level" && !level) return [what];
  return [deskMemoryKey(desk, what, DESK_MEMORY[what] === "level" ? level! : undefined), what];
}

/**
 * What the ruler opens on (top-level design 12.1), for a desk at a level: the span this desk last chose there, else the
 * desk's default. Explore at a system, world or site opens on human time ("week", a Show id the ruler knows), with
 * "Whole life ›" beside it; Build, and every desk above the system, on the whole history (null). Run has the clock, not
 * the ruler. `kept` is what `keepRulerSpan` wrote: a length of time, or "whole".
 */
export function rulerOpens(desk: Desk, level: DeskLevel | null, kept: string | null): number | string | null {
  if (kept === "whole") return null;
  const n = kept === null ? NaN : Number(kept);
  if (Number.isFinite(n) && n > 0) return n;
  const human = level === "system" || level === "world" || level === "site";
  return desk === "explore" && human ? "week" : null;
}

/** A span the GM chose, as `rulerOpens` reads it back. */
export function keepRulerSpan(span: number | null): string {
  return span === null || !(span > 0) || !Number.isFinite(span) ? "whole" : String(span);
}
