// One list of the things a GM can do to an object (V3 gap 38, second audit, accepted by Alex 24 Nov 2026).
// The press-and-hold or right-click menu on any object, the Build tools and the edit group of More all read
// this list, so an action added here appears in all three, with the same words and the same key.
//
// The menu is desk-aware (top-level design 2.3): Explore offers looking and sending to the table, Build offers
// changing, Run offers ships and the table. An action that belongs to the desk but cannot be done now stays in
// the menu, greyed, with the reason in words. Actions that belong to another desk are not listed one by one; the
// menu ends with one quiet line per other desk ("On Build: Paste here, Cut") that switches desk.
//
// Presentation only: this file says what is offered and why not. What each action does is the shell's, by id.

export const DESKS = ["explore", "build", "run"] as const;
export type Desk = (typeof DESKS)[number];

/** The desks' names as the desk switch says them. */
export const DESK_WORDS: Record<Desk, string> = { explore: "Explore", build: "Build", run: "Run" };

/** How the menu groups its items, in this order, with a rule between groups. */
export const ACTION_GROUPS = ["look", "table", "change", "ships"] as const;
export type ActionGroup = (typeof ACTION_GROUPS)[number];

/** The object an action is offered on. */
export interface ActionTarget {
  address: string;
  /** Object kind (star, planet, moon, belt, zone, ship, station, ...). */
  kind: string;
  label: string;
}

/** What the shell knows right now that decides whether an action can be done. Absent means no. */
export interface ActionState {
  desk: Desk;
  /** The campaign has been shared with players (a players' link exists). */
  sharing?: boolean;
  /** Minerva can describe things here (an agent key is set up, or the panel can ask for one). */
  canDescribe?: boolean;
  /** The target can be withdrawn: a body the campaign added, not one drawn from a sample's own file. */
  canCut?: boolean;
  /** Edits are being tried on a what-if branch, where pasting and adding wait until it is kept or thrown away. */
  branch?: boolean;
}

export interface ActionDef {
  id: string;
  /** The words, given the target ("Open Tarn"). */
  label: (t: ActionTarget) => string;
  group: ActionGroup;
  desks: readonly Desk[];
  /** The kinds it applies to; absent: any object. */
  kinds?: readonly string[];
  /** Kinds it never applies to. */
  notKinds?: readonly string[];
  /** The key that does the same thing, as `aria-keyshortcuts` writes it. */
  key?: string;
  /** Why it cannot be done now, in words; null when it can. */
  why?: (t: ActionTarget, s: ActionState) => string | null;
}

/** Things that are places in a system rather than bodies with a surface or a close-up of their own. */
const AREAS = ["belt", "zone"] as const;
/** Things a GM can add a world or moon round (Add here, V3 gap 2). */
const HOSTS = ["star", "planet"] as const;
/** Worlds with ground under them, or at least a world to fly down to (the Land screen picks a site). */
const LANDABLE = ["planet", "moon", "dwarf", "dwarf_planet", "asteroid", "comet"] as const;

const onBranch = (s: ActionState) => (s.branch ? "Keep or throw away the what-if first" : null);

/** Every action on an object, in menu order within each group. */
export const OBJECT_ACTIONS: readonly ActionDef[] = [
  { id: "open", label: (t) => `Open ${t.label}`, group: "look", desks: DESKS, notKinds: AREAS },
  {
    id: "describe",
    label: () => "Describe it",
    group: "look",
    desks: DESKS,
    why: (_t, s) => (s.canDescribe ? null : "Minerva needs an agent key: set one up under Ask"),
  },
  { id: "link", label: () => "Copy a link to it", group: "look", desks: DESKS, notKinds: AREAS },
  {
    id: "send",
    label: (t) => `Show the players ${t.label}`,
    group: "table",
    desks: ["explore", "run"],
    notKinds: AREAS,
    why: (_t, s) => (s.sharing ? null : "Share with the players first (on the Run desk)"),
  },
  { id: "copy", label: () => "Copy", group: "change", desks: ["explore", "build"], key: "Control+C" },
  { id: "paste", label: () => "Paste here", group: "change", desks: ["build"], key: "Control+V", why: (_t, s) => onBranch(s) },
  { id: "add", label: () => "Add here…", group: "change", desks: ["build"], kinds: HOSTS, key: "A", why: (_t, s) => onBranch(s) },
  {
    id: "cut",
    label: () => "Cut",
    group: "change",
    desks: ["build"],
    key: "Control+X",
    why: (_t, s) => onBranch(s) ?? (s.canCut ? null : "Only a body added to this campaign can be cut; the sample's own come from its file"),
  },
  { id: "voyage", label: (t) => `Voyage from ${t.label}`, group: "ships", desks: ["run"], notKinds: AREAS },
  { id: "land", label: (t) => `Land on ${t.label}`, group: "ships", desks: ["run", "explore"], kinds: LANDABLE },
];

/** The action with this id. */
export function actionById(id: string): ActionDef | undefined {
  return OBJECT_ACTIONS.find((a) => a.id === id);
}

/** True when the action makes sense on this kind of object at all (on some desk). */
export function appliesTo(a: ActionDef, t: ActionTarget): boolean {
  return (!a.kinds || a.kinds.includes(t.kind)) && !a.notKinds?.includes(t.kind);
}

export interface MenuItem {
  id: string;
  label: string;
  group: ActionGroup;
  key?: string;
  /** Why it is greyed, in words; absent when it can be done. */
  why?: string;
}

/** One quiet line at the foot of the menu: what another desk offers here, and the desk to switch to. */
export interface OtherDesk {
  desk: Desk;
  words: string;
}

export interface ObjectMenu {
  items: MenuItem[];
  /** The other desks that offer more on this object, in desk order. */
  elsewhere: OtherDesk[];
}

/** The menu for an object on the desk in front: this desk's items in group order, then the other desks in a line each. */
export function menuFor(t: ActionTarget, s: ActionState, actions: readonly ActionDef[] = OBJECT_ACTIONS): ObjectMenu {
  const fits = actions.filter((a) => appliesTo(a, t));
  const items: MenuItem[] = [];
  for (const g of ACTION_GROUPS) {
    for (const a of fits) {
      if (a.group !== g || !a.desks.includes(s.desk)) continue;
      const why = a.why?.(t, s) ?? null;
      items.push({ id: a.id, label: a.label(t), group: a.group, ...(a.key ? { key: a.key } : {}), ...(why ? { why } : {}) });
    }
  }
  const elsewhere: OtherDesk[] = [];
  for (const d of DESKS) {
    if (d === s.desk) continue;
    const only = fits.filter((a) => a.desks.includes(d) && !a.desks.includes(s.desk));
    if (only.length) elsewhere.push({ desk: d, words: `On ${DESK_WORDS[d]}: ${listWords(only.map((a) => shortLabel(a, t)))}` });
  }
  return { items, elsewhere };
}

/** An action's words without the object's name, for the line about another desk ("Voyage from Tarn" → "Voyage"). */
function shortLabel(a: ActionDef, t: ActionTarget): string {
  const w = a.label(t).replace(/…$/, "");
  const name = ` ${t.label}`;
  const i = w.lastIndexOf(name);
  const cut = i > 0 && i + name.length === w.length ? w.slice(0, i) : w;
  return cut.replace(/ (from|on|to)$/, "");
}

/** "a", "a and b", "a, b and c". */
export function listWords(xs: readonly string[]): string {
  return xs.length <= 1 ? (xs[0] ?? "") : `${xs.slice(0, -1).join(", ")} and ${xs.at(-1)}`;
}

/** The key as the menu prints it ("Control+C" → "Ctrl+C"). */
export function keyWords(key: string): string {
  return key.replace(/Control/g, "Ctrl");
}
