// The symbols in force on this device (H8.4A, V12): which symbology the GM
// chose and any edited copy kept. Presentation only, kept as a device
// preference: it never reaches the kernel and never changes the state hash.
import { checkSymbology, markerFor, type V3TokenMap, type Marker, type SymbologyDoc, type SymbologyHost, type ThemeCapability } from "../../ui-contract/src/index.ts";
import diagrammatic from "./documents/symbology.diagrammatic.json";
import colourCoded from "./documents/symbology.colour-coded.json";
import mixed from "./documents/symbology.mixed.json";
import namesDoc from "./documents/symbology.names.json";
import v3Tokens from "./documents/v3-colour-tokens.json";
import { loadPref, savePref } from "./prefs.ts";

export const SHIPPED: readonly SymbologyDoc[] = [diagrammatic, colourCoded, mixed] as unknown as SymbologyDoc[];
export const CLASS_NAMES: Readonly<Record<string, string>> = namesDoc.names;

function keptDocs(): SymbologyDoc[] {
  try {
    const raw = JSON.parse(loadPref("symbology.kept") ?? "[]") as SymbologyDoc[];
    return raw.filter((d) => checkSymbology(d, [...SHIPPED, ...raw]) === null);
  } catch {
    return [];
  }
}

/** A symbology host for the GM shell. `theme` says what the current theme can carry. */
export function createSymbols(fallback: string, theme: () => ThemeCapability, previews?: SymbologyHost["previews"]): SymbologyHost {
  const st = $state({ chosen: loadPref("symbology") ?? fallback, kept: keptDocs() });
  const registry = $derived<readonly SymbologyDoc[]>([...SHIPPED.filter((d) => !st.kept.some((k) => k.id === d.id)), ...st.kept]);
  const marker = $derived.by((): Marker => {
    const id = registry.some((d) => d.id === st.chosen) ? st.chosen : fallback;
    return markerFor(id, registry, theme());
  });
  return {
    registry: () => registry,
    chosen: () => marker.id,
    marker: () => marker,
    theme,
    names: () => CLASS_NAMES,
    previews,
    v3Tokens: () => v3Tokens as unknown as V3TokenMap,
    choose(id) {
      st.chosen = id;
      savePref("symbology", id);
    },
    keep(doc) {
      st.kept = doc === null ? [] : [...st.kept.filter((k) => k.id !== doc.id), doc];
      savePref("symbology.kept", JSON.stringify(st.kept));
    },
  };
}
