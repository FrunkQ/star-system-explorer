// The editing grammar (editing design section 3B, accepted 24 Nov; the Profile added by the ring and belt editing
// design, question 4, accepted 26 Nov): every editor is built from these parts and nothing else. A new editor is data
// in the handle sheet, naming the part each handle uses through its `scale`. tests/ui/editing-grammar.test.ts holds
// the editing components to this list: each one says which part it is, so a new component is a new part on purpose.

export interface GrammarPart {
  id: string;
  name: string;
  /** What it is, in the design's words. */
  is: string;
  /** The components in packages/views/editing/src that are this part (a box may be several). */
  files: string[];
  /** The handle sheet's `scale` values that open this part. */
  scales?: string[];
}

export const GRAMMAR_PARTS: readonly GrammarPart[] = [
  { id: "handle", name: "Handle", is: "a subject you can set, as a chip on the row", files: ["Handles.svelte"] },
  { id: "box", name: "Box", is: "what a handle opens, under the row", files: ["Box.svelte", "AirBox.svelte", "MakeupBox.svelte", "StarChooser.svelte"] },
  { id: "strip", name: "Strip", is: "a number on a line", files: ["Strip.svelte"], scales: ["log", "linear"] },
  { id: "bar", name: "Bar", is: "shares of a whole (air, makeup, any mixture)", files: [], scales: ["bar"] },
  { id: "ring", name: "Ring", is: "an angle or a place round a circle", files: ["Ring.svelte"], scales: ["ring"] },
  { id: "profile", name: "Profile", is: "an amount along a line: zones that sum to the whole, joins that drag sideways, heights up and down", files: ["Profile.svelte"], scales: ["profile"] },
  { id: "chips", name: "Chips", is: "named choices and shortcuts (Kinds, in step with, locked)", files: [] },
  { id: "verdict", name: "Verdict", is: "one sentence under any control, with at most one offered fix (Reach it)", files: ["Reach.svelte"] },
  { id: "menu", name: "Menu row", is: "the rare things for this control", files: [] },
  { id: "grip", name: "Grip", is: "a control on the chart itself", files: [] },
  { id: "ghost", name: "Ghost", is: "a thing not yet placed", files: [] },
  { id: "tree", name: "Tree", is: "who goes round what", files: [] },
  { id: "recipe", name: "Recipe", is: "steps toward something a world could be", files: ["CouldBe.svelte"] },
  { id: "pencil", name: "Pencil and ink", is: "pencil is a preview; ink is settled", files: [] },
  { id: "sheet", name: "Sheet", is: "every input as a typeable table", files: ["Sheet.svelte"] },
];

/** The grammar part a handle sheet `scale` opens, or null when no part takes it. */
export const partOfScale = (scale: string): GrammarPart | null => GRAMMAR_PARTS.find((p) => p.scales?.includes(scale)) ?? null;
