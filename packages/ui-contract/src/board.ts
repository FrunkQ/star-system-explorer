// The Switchboard's board (track PB, `handover/design/SSE4-playbook-and-switchboard-design.md`, sections 3, 5, 6, 11).
// Everything the GM can show is a source; everything that can show it is a destination; a line joins them. This file
// is the board as plain data and the rules that move it: sending, holding, Everyone as a layer that lifts, a screen's
// own line, moving and swapping screens, redirects, and the board's own undo history. It lives on the GM's machine
// only and is never saved with the campaign (5.6). What reaches the wire is still `Shown`, worked out per role and per
// screen by `resolve`, so players' screens read one value whatever made it.
import type { Shown } from "./seat.ts";

/** A holding card (6.2): blank is "Screens off"; plain and joke draw from the pack's list; or words typed now. */
export type HoldCard = "blank" | "plain" | "joke" | { text: string };

/** Where a line goes: every role, one role, or one screen. */
export type Dest = "everyone" | `role:${string}` | `screen:${string}`;

export interface Line {
  /** What this destination is sent. */
  source: Shown;
  /** Which source on the board it came from ("live", a preset's id), for drawing its line; absent: the table's opening. */
  from?: string;
  /** Held: the destination shows this card and nothing of the source (6.1). */
  held?: HoldCard;
  /** What this destination showed before this send, for "press again to go back" (5.4). Absent: nothing of its own. */
  before?: Line | null;
  /** Made only to hold a destination that had no line of its own: Release takes the line away again. */
  for_hold?: true;
}

/** A screen the board knows (the screen hello, section 11): its own id kept on its device, its name and its role. */
export interface BoardScreen {
  name: string;
  role: string;
  /** On the GM's own machine (a tab opened from here), so a camera may reach it later (section 10). */
  local?: boolean;
}

export interface Board {
  /** Everyone layers over the set-up, top last (5.4). */
  over: Shown[];
  /** Which source each layer came from, beside `over`. */
  over_from?: string[];
  /** The floor ("everyone"), each role's own line and each screen's own line. */
  lines: Partial<Record<Dest, Line>>;
  screens: Record<string, BoardScreen>;
  /** Signal dial levels 0 to 1 by role or screen; a screen's own level wins over its role's (7.1). */
  signal: Partial<Record<Dest, number>>;
  /** Tonight's redirects (5.5): a send to the key role goes to the value role's screens. */
  redirects: Record<string, string>;
  /** Hold all (6.1): every destination shows this card; lines held before stay held after Release all. */
  all?: HoldCard;
}

/** The board at the start of a share: the table sees `floor`, nothing else is set. */
export function emptyBoard(floor: Shown): Board {
  return { over: [], lines: { everyone: { source: clone(floor) } }, screens: {}, signal: {}, redirects: {} };
}

/** Today's wire value read as a source: `{off: true}` is a blank hold. */
export function sourceOf(s: Shown): { source: Shown; held?: HoldCard } {
  return "off" in s ? { source: { off: true }, held: "blank" } : { source: clone(s) };
}

/**
 * A board from the state a share had before the Switchboard (the table's `shown` and each seat's own `shown`), so an
 * older session or a reopened one carries on where it was.
 */
export function upgradeBoard(table: Shown, seats: readonly { id: string; shown?: Shown }[]): Board {
  const t = sourceOf(table);
  const b: Board = { over: [], lines: { everyone: { source: t.source, ...(t.held ? { held: t.held } : {}) } }, screens: {}, signal: {}, redirects: {} };
  for (const s of seats) {
    if (!s.shown) continue;
    const o = sourceOf(s.shown);
    b.lines[`role:${s.id}`] = { source: o.source, ...(o.held ? { held: o.held } : {}), before: null };
  }
  return b;
}

const clone = <T>(x: T): T => JSON.parse(JSON.stringify(x)) as T;

/** Where a send to `dest` really lands tonight: a redirected role's sends go to the role it is on (5.5). */
export function landing(b: Board, dest: Dest): Dest {
  if (!dest.startsWith("role:")) return dest;
  let role = dest.slice(5);
  const seen = new Set<string>();
  while (b.redirects[role] && !seen.has(role)) (seen.add(role), (role = b.redirects[role]!));
  return `role:${role}`;
}

/** Whether any role or screen has a line of its own. */
const hasOwn = (b: Board) => Object.keys(b.lines).some((k) => k !== "everyone");

/**
 * Send `source` to `dest` (2, 5.4). To Everyone: when no role or screen has its own line and no layer is on, it is the
 * floor; otherwise a layer over the set-up that Lift takes off. To a role or a screen: its own line, keeping what it
 * had so pressing again goes back. A held line takes the new source and stays held (6.1: the line shows what waits).
 */
export function send(b: Board, dest: Dest, source: Shown, from?: string): Board {
  const n = clone(b);
  const to = landing(n, dest);
  const src = clone(source);
  const tag = from ? { from } : {};
  if (to === "everyone") {
    if (!n.over.length && !hasOwn(n)) {
      const was = n.lines.everyone;
      n.lines.everyone = { source: src, ...tag, ...(was?.held ? { held: was.held } : {}), before: was ? { source: was.source, ...(was.from ? { from: was.from } : {}) } : null };
    } else (n.over.push(src), (n.over_from ??= []).push(from ?? ""));
    return n;
  }
  const was = n.lines[to];
  const prior = was && !was.for_hold ? { source: was.source, ...(was.from ? { from: was.from } : {}), ...(was.before !== undefined ? { before: was.before } : {}) } : null;
  n.lines[to] = { source: src, ...tag, ...(was?.held ? { held: was.held } : {}), before: prior };
  return n;
}

/** Take the top Everyone layer off (5.4): every screen returns to what it had. */
export function lift(b: Board): Board {
  const n = clone(b);
  n.over.pop();
  n.over_from?.pop();
  return n;
}

/**
 * Go back one send on `dest` (a fired preset pressed again, or Back on a line card): it shows what it had before; a
 * line with nothing before it goes, and the destination follows its role or the table again.
 */
export function back(b: Board, dest: Dest): Board {
  const n = clone(b);
  const to = landing(n, dest);
  const line = n.lines[to];
  if (!line) return n;
  if (line.before) n.lines[to] = { ...line.before, ...(line.held ? { held: line.held } : {}) };
  else if (to !== "everyone") delete n.lines[to];
  return n;
}

/** Drop a screen's or a role's own line at once ("Back with Bridge", "Back with the table"). */
export function rejoin(b: Board, dest: Exclude<Dest, "everyone">): Board {
  const n = clone(b);
  delete n.lines[dest];
  return n;
}

/**
 * Hold `dest` (6.1): it shows `card` and nothing of its source. A destination with no line of its own is given one
 * holding what it showed, so releasing it gives that back.
 */
export function hold(b: Board, dest: Dest, card: HoldCard, seats: readonly string[] = []): Board {
  const n = clone(b);
  const to = landing(n, dest);
  const line = n.lines[to];
  if (line) line.held = card;
  else {
    const role = to.startsWith("role:") ? to.slice(5) : to.startsWith("screen:") ? (n.screens[to.slice(7)]?.role ?? seats[0] ?? "table") : null;
    const showing = role ? underneath(n, role, to.startsWith("screen:") ? to.slice(7) : undefined) : { off: true as const };
    n.lines[to] = { source: showing, held: card, before: null, for_hold: true };
  }
  return n;
}

/** Release `dest`: it shows its source again, the latest one, once (6.1). */
export function release(b: Board, dest: Dest): Board {
  const n = clone(b);
  const to = landing(n, dest);
  const line = n.lines[to];
  if (!line) return n;
  delete line.held;
  // A line made only to hold goes again: the destination follows its role or the table, as it did.
  if (to !== "everyone" && line.for_hold) delete n.lines[to];
  return n;
}

/** Hold everything (6.1, key H); Release all puts each line back as it was, lines held before staying held. */
export function holdAll(b: Board, card: HoldCard): Board {
  return { ...clone(b), all: card };
}
export function releaseAll(b: Board): Board {
  const n = clone(b);
  delete n.all;
  return n;
}

/** A screen says hello (section 11): known screens keep the role the GM gave them; new ones get the role they asked for. */
export function screenHello(b: Board, id: string, hello: { name: string; role: string; local?: boolean }): Board {
  const n = clone(b);
  const had = n.screens[id];
  n.screens[id] = { name: hello.name || had?.name || "Screen", role: had?.role ?? hello.role, ...(hello.local ? { local: true } : {}) };
  return n;
}

/** Move a screen to another role (drag onto a role's heading). Its own line, if any, goes with it. */
export function moveScreen(b: Board, id: string, role: string): Board {
  const n = clone(b);
  const s = n.screens[id];
  if (s) s.role = role;
  return n;
}

/** Swap two screens' roles (drag one onto the other). */
export function swapScreens(b: Board, a: string, c: string): Board {
  const n = clone(b);
  const x = n.screens[a];
  const y = n.screens[c];
  if (x && y) [x.role, y.role] = [y.role, x.role];
  return n;
}

/** Redirect a role's sends to another role's screens tonight (5.5); null puts it back. */
export function redirect(b: Board, role: string, to: string | null): Board {
  const n = clone(b);
  if (to && to !== role) n.redirects[role] = to;
  else delete n.redirects[role];
  return n;
}

/** Turn a signal dial (7.1); 0 removes it. */
export function setSignal(b: Board, dest: Dest, level: number): Board {
  const n = clone(b);
  const v = Math.max(0, Math.min(1, level));
  if (v > 0) n.signal[dest] = v;
  else delete n.signal[dest];
  return n;
}

/** What a role (or one of its screens) shows beneath any hold: Everyone's top layer, its own line, its role's, the floor. */
function underneath(b: Board, role: string, screen?: string): Shown {
  const top = b.over[b.over.length - 1];
  if (top) return clone(top);
  const own = (screen ? b.lines[`screen:${screen}`] : undefined) ?? b.lines[`role:${role}`] ?? b.lines.everyone;
  return own ? clone(own.source) : { off: true };
}

/** What a destination shows, as the wire carries it: a hold wins over everything, then layers, lines and the floor. */
export interface Showing {
  shown: Shown;
  /** Held: the card to draw (the shown value is then `{off: true}`, so nothing of the source travels). */
  held?: HoldCard;
}

/**
 * What a role's screens show, or one screen when `screen` is given (5.1, 6.1). A hold never lets its source through:
 * the shown value is then `{off: true}` and the card travels beside it.
 */
export function resolve(b: Board, role: string, screen?: string): Showing {
  const held = b.all ?? (screen ? b.lines[`screen:${screen}`]?.held : undefined) ?? b.lines[`role:${role}`]?.held ?? b.lines.everyone?.held;
  if (held) return { shown: { off: true }, held };
  return { shown: underneath(b, role, screen) };
}

/** The signal level a screen shows: its own dial, else its role's (7.1). */
export function signalOf(b: Board, role: string, screen?: string): number {
  return (screen ? b.signal[`screen:${screen}`] : undefined) ?? b.signal[`role:${role}`] ?? 0;
}

/**
 * The board's own undo history (5.4): tonight's changes to what screens show, not saved. Reveals are not on it (the
 * eye takes those back). `do` records the board before a change; Undo and Redo step through.
 */
export class BoardHistory {
  #past: Board[] = [];
  #future: Board[] = [];
  board: Board;
  readonly limit: number;
  constructor(board: Board, limit = 100) {
    this.board = board;
    this.limit = limit;
  }
  do(next: Board): Board {
    this.#past.push(this.board);
    if (this.#past.length > this.limit) this.#past.shift();
    this.#future = [];
    return (this.board = next);
  }
  get canUndo(): boolean {
    return this.#past.length > 0;
  }
  get canRedo(): boolean {
    return this.#future.length > 0;
  }
  undo(): Board {
    const prev = this.#past.pop();
    if (!prev) return this.board;
    this.#future.push(this.board);
    return (this.board = prev);
  }
  redo(): Board {
    const next = this.#future.pop();
    if (!next) return this.board;
    this.#past.push(this.board);
    return (this.board = next);
  }
}

/** The words a holding card shows, chosen on the GM's machine (PB5 brings the pack's list; until then, these). */
export function cardWords(card: HoldCard, pick: (kind: "plain" | "joke") => string = defaultPick): string {
  return card === "blank" ? "" : typeof card === "object" ? card.text : pick(card);
}
const PLAIN = ["Awaiting transmission", "Signal lost", "Stand by", "No carrier"];
const JOKES = ["Hold on: tracking down the missing gold atoms. I think the black hole swallowed them.", "Awaiting transmission. Light is doing its best.", "Signal lost. It was last seen heading for the Oort cloud.", "Recalculating. Someone moved the barycentre."];
let lastPick = "";
function defaultPick(kind: "plain" | "joke"): string {
  const list = kind === "plain" ? PLAIN : JOKES;
  const choices = list.filter((x) => x !== lastPick);
  return (lastPick = choices[Math.floor(Math.random() * choices.length)] ?? list[0]!);
}

/**
 * What goes on the wire for the board (the host state's `shown`, each seat's `shown` and each known screen's own): the
 * same `Shown` players' screens already read. A held destination gets `{off: true, card}` and nothing of its source.
 * `words` turns a card into its words once per change, so every screen of a role shows the same joke.
 */
export function boardWire(b: Board, roles: readonly string[], words: (card: HoldCard) => string = (c) => cardWords(c)): { shown: Shown; roles: Record<string, Shown>; screens: Record<string, Shown> } {
  const said = new Map<string, string>();
  const wire = (r: Showing): Shown => {
    if (!r.held) return r.shown;
    const key = JSON.stringify(r.held);
    if (!said.has(key)) said.set(key, words(r.held));
    const card = said.get(key)!;
    return card ? { off: true, card } : { off: true };
  };
  const table = wire(resolve(b, ""));
  const out: Record<string, Shown> = {};
  for (const role of roles) out[role] = wire(resolve(b, role));
  const screens: Record<string, Shown> = {};
  for (const [id, s] of Object.entries(b.screens)) screens[id] = wire(resolve(b, s.role, id));
  return { shown: table, roles: out, screens };
}

/**
 * The table as the Switchboard sees it (PB2): the board, the roles (seats), how many screens are connected, and the
 * ways to change it. The shell makes one from its share and puts it on the workspace (`Workspace.table`), so the
 * Switchboard view needs nothing else of the shell.
 */
export interface TablePort {
  readonly state: {
    board: Board;
    roles: readonly { id: string; label: string; sealed: boolean }[];
    /** Screens connected now (players' pages that said hello and have not gone). */
    players: number;
    /** The players' link once on air, else null. */
    link: string | null;
    can_undo: boolean;
    can_redo: boolean;
    /** Players' requests waiting for the GM. */
    waiting: number;
  };
  act(f: (b: Board) => Board): void;
  undo(): void;
  redo(): void;
  /** Go on air (share), showing the table the live view; returns the players' link. */
  start(): string;
  /** Join links for a role: one per screen document it opens. */
  links(role: string): { screen: string | null; href: string }[];
  /** The live source: where the GM is now, as the table would be sent it. */
  live(): Shown;
}
