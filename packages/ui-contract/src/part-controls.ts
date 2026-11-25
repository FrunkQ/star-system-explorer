// Controls that switch (track AT13, atlas design 11A.4): every player part declares the controls it can draw, each with
// an id, plain words, a kind and a default; a layout item's `controls` turns each on or off ("none" for a pure display,
// "all" for everything). A part never draws a control it has not declared, and never one that is off. Off means off for
// the player: the GM steering the screen from the Switchboard still can.
//
// A control is declared as a kind, never as a drawing, so each look can draw every kind its own way (the control sets
// of AT15). The view controls (View style, Seen from, Scale, HUD, Band) stay where they are (`ViewModule.controls`):
// these are the part's own presses.

/** The kinds of control a part may declare (11A.4 rule 5). */
export const PART_CONTROL_KINDS = ["button", "switch", "choice", "step", "dial", "tab", "link"] as const;
export type PartControlKind = (typeof PART_CONTROL_KINDS)[number];

export interface PartControl {
  /** Short and stable: what a layout names ("sort", "select"). */
  id: string;
  /** In plain words, for the parts catalogue, the layout editor and Orac ("Sort by a column"). */
  words: string;
  kind: PartControlKind;
  /** On unless a layout says otherwise (true), or off unless asked for (false). */
  default: boolean;
}

/** What a layout item says about its part's controls: each by id, or all of them at once. */
export type ItemControls = "all" | "none" | Record<string, boolean>;

/** Problems with an item's `controls` against what its part declares; empty when it is fine. */
export function checkItemControls(declared: readonly PartControl[] | undefined, given: unknown): string[] {
  if (given === undefined || given === "all" || given === "none") return [];
  if (!given || typeof given !== "object" || Array.isArray(given)) return ['controls must be "all", "none" or an object of control ids and true or false'];
  const ids = (declared ?? []).map((c) => c.id);
  const errors: string[] = [];
  for (const [k, v] of Object.entries(given as Record<string, unknown>)) {
    if (!ids.includes(k)) errors.push(`no control ${k}${ids.length ? ` (it has ${ids.join(", ")})` : " (it has none)"}`);
    else if (typeof v !== "boolean") errors.push(`control ${k} must be true or false`);
  }
  return errors;
}

/** Which of a part's controls are on for one item: the item's word, else each control's default. */
export function resolveControls(declared: readonly PartControl[] | undefined, given: ItemControls | undefined): Record<string, boolean> {
  const out: Record<string, boolean> = {};
  for (const c of declared ?? []) out[c.id] = given === "all" ? true : given === "none" ? false : typeof given === "object" && typeof given[c.id] === "boolean" ? given[c.id]! : c.default;
  return out;
}

/** Whether a part should draw one control: on unless the screen turned it off. A host that says nothing leaves it on. */
export function controlOn(controls: Readonly<Record<string, boolean>> | undefined, id: string): boolean {
  return controls?.[id] !== false;
}

/**
 * A part with every control off is a picture (rule 6): what a wall screen or a stream wants. The screen makes the
 * whole part unreachable (`inert`), so nothing in it can be pressed or reached by keyboard either.
 */
export function isPicture(declared: readonly PartControl[] | undefined, given: ItemControls | undefined): boolean {
  if (given === "none") return true;
  const on = resolveControls(declared, given);
  const ids = Object.keys(on);
  return ids.length > 0 && ids.every((k) => !on[k]);
}

/** The parts catalogue's line for a part's controls (Reference and Orac): "sort (button, on): Sort by a column". */
export function controlLines(declared: readonly PartControl[] | undefined): string {
  return (declared ?? []).map((c) => `- ${c.id} (${c.kind}, ${c.default ? "on" : "off"} by default): ${c.words}`).join("\n");
}
