// "Known to" (track SV, secrets and visibility design): who knows a thing, or one fact on it, read from the kernel's
// `known_to` and changed with one `set_known` intent. The eye shows three landmarks: GM only (closed), Heard of (half
// open) and Known (open). Pressing the eye reveals to the table, or hides again; everything finer is in the popup.
// While players are connected a reveal is held for two seconds, so Undo within that time means they never saw it.

/** The three landmarks every ladder has (design section 3). */
export const LANDMARKS = ["gm_only", "heard_of", "known"] as const;
export type Landmark = (typeof LANDMARKS)[number];
export const LANDMARK_WORDS: Record<Landmark, string> = { gm_only: "GM only", heard_of: "Heard of", known: "Known" };
/** What each landmark means on the players' screens, for the popup's titles. */
export const LANDMARK_MEANS: Record<Landmark, string> = {
  gm_only: "Not known to exist: nothing on their screens, and nothing where it would be",
  heard_of: "Known to exist, with a name and a place; the details held back",
  known: "Everything their view of it shows",
};

export interface KnownException {
  field: string;
  landmark: Landmark;
}

export interface KnownAudience {
  audience: string;
  label: string;
  /** The rung on the pack's own ladder ("long_range"), and its name ("Long range"). */
  rung: string;
  rung_label: string;
  landmark: Landmark;
  /** True when the thing stands on its ladder's top rung. */
  top: boolean;
  exceptions: KnownException[];
  /** Cover stories (SV5): a fact told in place of the true one. Empty until then. */
  cover: { field: string; told: string }[];
  /** The news reaches this audience only from this campaign time (seconds), when set. */
  from_s?: number;
}

export interface KnownTo {
  address: string;
  field?: string;
  at_s: number;
  audiences: KnownAudience[];
}

/** How far a reveal or hide reaches (design section 5): this thing, it and what is inside it, or a list. */
export type KnownScope = "this" | "children" | { list: string[] };

/** One change the control asks for: a set_known without its times. */
export interface KnownChange {
  audience: string;
  landmark: Landmark | null;
  field?: string;
  scope: KnownScope;
}

/** The audience a single press acts on: the table ("everyone"), else the first. */
export function tableOf(k: KnownTo | null | undefined): KnownAudience | null {
  if (!k || !k.audiences.length) return null;
  return k.audiences.find((a) => a.audience === "everyone") ?? k.audiences[0]!;
}

/** The landmark of one fact for one audience: its exception when the GM made one, else the thing's own. */
export function fieldLandmark(a: KnownAudience, field: string): Landmark {
  return a.exceptions.find((e) => e.field === field)?.landmark ?? a.landmark;
}

/** What one press does: an open eye hides again; any other reveals to the table. */
export function pressedLandmark(l: Landmark): Landmark {
  return l === "known" ? "gm_only" : "known";
}

/** The eye's state for the whole thing: open only when every audience knows it, closed when none has heard of it. */
export function knownOverall(k: KnownTo): Landmark {
  const ls = k.audiences.map((a) => a.landmark);
  if (!ls.length) return "known";
  if (ls.every((l) => l === "known")) return "known";
  if (ls.every((l) => l === "gm_only")) return "gm_only";
  return "heard_of";
}

/** The words a slip says after a change. */
export function knownChangedWords(l: Landmark, audienceLabel: string, held: boolean): string {
  const who = audienceLabel === "The table" || audienceLabel === "Everyone" ? "the table" : audienceLabel;
  if (l === "known") return `Shown to ${who}.`;
  if (l === "heard_of") return `${who[0]!.toUpperCase()}${who.slice(1)} ${who === "the table" ? "has" : "have"} heard of it.`;
  return held ? `Hidden from ${who}.` : `Hidden from ${who}. They have already seen it.`;
}

/** The marks show on the Build and Run desks only, and not until they matter (design 4, question 10). */
export function eyesShow(desk: "explore" | "build" | "run" | undefined, relevant: boolean): boolean {
  return (desk === "build" || desk === "run") && relevant;
}

/** Whether secrecy has become relevant for this thing: something is held back, or the campaign is shared. */
export function secrecyRelevant(k: KnownTo | null | undefined, shared: boolean): boolean {
  if (!k) return false;
  return shared || k.audiences.some((a) => a.landmark !== "known" || a.exceptions.length > 0);
}

/** The two-second hold (design 5, question 4). */
export const HOLD_MS = 2000;

export interface Held {
  /** True when it was still waiting: nothing was sent. */
  undo(): boolean;
  /** Send now, if it is still waiting. */
  flush(): void;
  readonly sent: boolean;
}

/**
 * Hold a reveal while players are connected: `send` runs after `ms`, unless `undo` comes first. With no players
 * connected (or a hide, which never needs holding) it is sent at once. `timer` is injectable for tests.
 */
export function holdReveal(send: () => void, opts: { hold: boolean; ms?: number; timer?: { set: (f: () => void, ms: number) => unknown; clear: (h: unknown) => void } }): Held {
  const t = opts.timer ?? { set: (f: () => void, ms: number) => setTimeout(f, ms), clear: (h: unknown) => clearTimeout(h as ReturnType<typeof setTimeout>) };
  let sent = false;
  let undone = false;
  const go = () => {
    if (sent || undone) return;
    sent = true;
    send();
  };
  const h = opts.hold ? t.set(go, opts.ms ?? HOLD_MS) : (go(), null);
  return {
    undo() {
      if (sent) return false;
      undone = true;
      if (h !== null) t.clear(h);
      return true;
    },
    flush() {
      if (h !== null) t.clear(h);
      go();
    },
    get sent() {
      return sent;
    },
  };
}
