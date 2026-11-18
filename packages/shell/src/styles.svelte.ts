// Style documents on this device (H8.9, V11): the shipped ones and any the
// GM keeps. Presentation only: never reaches the kernel or the state hash.
import type { StyleDoc, StylesHost, ViewModule } from "../../ui-contract/src/index.ts";
import stylesDoc from "./documents/styles.json";
import { loadPref, savePref } from "./prefs.ts";

export const SHIPPED_STYLES: readonly StyleDoc[] = stylesDoc.styles as unknown as StyleDoc[];

export function keptStyles(): StyleDoc[] {
  try {
    const raw = JSON.parse(loadPref("styles.kept") ?? "[]") as StyleDoc[];
    return Array.isArray(raw) ? raw.filter((d) => d && d.schema === "sse.style/1") : [];
  } catch {
    return [];
  }
}

/** Every style this device knows: shipped, then kept (a kept style with a shipped id replaces it). */
export function allStyles(kept: readonly StyleDoc[] = keptStyles()): StyleDoc[] {
  return [...SHIPPED_STYLES.filter((d) => !kept.some((k) => k.id === d.id)), ...kept];
}

export function createStyles(views: () => readonly ViewModule<unknown, unknown>[], tier: () => string): StylesHost {
  const st = $state({ kept: keptStyles() });
  const registry = $derived(allStyles(st.kept));
  return {
    registry: () => registry,
    tier,
    views,
    keep(doc) {
      st.kept = doc === null ? [] : [...st.kept.filter((k) => k.id !== doc.id), doc];
      savePref("styles.kept", JSON.stringify(st.kept));
    },
  };
}
