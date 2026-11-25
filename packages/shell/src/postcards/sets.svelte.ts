// The postcard sets this page can show (PC6): the base presentation pack's (served beside the shell by
// scripts/build-shell.sh, not on the stack) and then any the stack's packs carry. Read once per stack; the wall shows
// them and held screens take their grounds from them.
import { postcardSetsOn, type PackDoc, type PostcardSet } from "../../../ui-contract/src/index.ts";

export const postcardSets = $state<{ list: PostcardSet[]; problems: string[] }>({ list: [], problems: [] });

let presentation: Promise<PackDoc | null> | null = null;
let readFor = "";

/** Read the sets on this stack (again only when the stack changes). */
export async function loadSets(stack: readonly PackDoc[]): Promise<void> {
  const key = stack.map((p) => `${p.id}@${p.version}`).join("|");
  if (key === readFor) return;
  readFor = key;
  presentation ??= fetch(document.baseURI.replace(/[^/#?]*([#?].*)?$/, "") + "packs/base-presentation.json")
    .then((r) => (r.ok ? (r.json() as Promise<PackDoc>) : null))
    .catch(() => null);
  const pres = await presentation;
  const r = postcardSetsOn([...(pres ? [pres] : []), ...stack]);
  postcardSets.list = r.sets;
  postcardSets.problems = r.problems;
}
