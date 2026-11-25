// Composite (exotic objects design 6A, package XO17; Alex 24 Nov 2026: "the composite view should be everywhere!").
// The last entry of the band switch on every view: several bands laid into one picture, as observatory press images
// are. One mechanism for every view that declares bands; nothing here knows what is being drawn. Presentation only:
// the viewer's mix is kept on their device, never in the campaign.
//
// - A layer is a band and the colour it is laid in. Visible keeps its natural colours ("natural").
// - Each band is scaled to its own brightest before the layers are added, so a faint radio jet is not drowned by a
//   bright disc (the drawing side: packages/renderer/src/composite.ts).
// - The legend lists every layer and its colour and says "mapped" as soon as any band but Visible is in.
// - A seat ticks only the bands its sensors carry; a phone stops at two layers and says why on the third.
import doc from "./composite-presets.json" with { type: "json" };
import type { ViewBand } from "./view-controls.ts";

/** The colour a layer is laid in: a CSS hex colour, or "natural" (Visible's own colours). */
export type LayerColour = string;
export interface CompositeLayer {
  band: ViewBand;
  colour: LayerColour;
}
/** The viewer's mix: its layers in drawing order (first at the bottom) and the preset it started from, if any. */
export interface CompositeMix {
  layers: CompositeLayer[];
  preset?: string;
}
export interface CompositePreset {
  id: string;
  name: string;
  layers: ViewBand[];
  why: string;
}

/** The starting colour of each band (observatory habit). Visible is natural. */
export const LAYER_COLOURS: Record<ViewBand, LayerColour> = { visible: "natural", ...(doc.colours as Record<Exclude<ViewBand, "visible">, string>) };

let PRESETS: CompositePreset[] = (doc.presets as CompositePreset[]).map((p) => ({ ...p, layers: [...p.layers] }));
/** The presets on offer: the base ones, then any a pack or Orac added. */
export function compositePresets(): readonly CompositePreset[] {
  return PRESETS;
}
/** Add presets (a pack's or Orac's); one with a known id replaces it. */
export function withCompositePresets(more: CompositePreset[]): void {
  const byId = new Map(PRESETS.map((p) => [p.id, p]));
  for (const p of more) byId.set(p.id, { ...p, layers: [...p.layers] });
  PRESETS = [...byId.values()];
}

/** A preset as a mix, in the starting colours. */
export function presetMix(id: string): CompositeMix | undefined {
  const p = PRESETS.find((x) => x.id === id);
  return p ? { preset: p.id, layers: p.layers.map((band) => ({ band, colour: LAYER_COLOURS[band] })) } : undefined;
}
/** What Composite shows the first time it is pressed: the Observatory preset. */
export const DEFAULT_MIX: CompositeMix = presetMix("observatory") ?? { layers: [{ band: "visible", colour: "natural" }] };

/** Most layers a phone draws at once (design 6A; open point 14). */
export const PHONE_LAYERS = 2;
export const WHY_PHONE_LAYERS = `A phone draws ${PHONE_LAYERS} bands at once: each band is a whole picture worked out and added`;

/** This device's layer limit: PHONE_LAYERS on a phone (a small touch screen), none elsewhere. Read once per call;
 * outside a browser (tests, the kernel) there is no limit. */
export function deviceMaxLayers(): number | undefined {
  try {
    const mm = (globalThis as { matchMedia?: (q: string) => { matches: boolean } }).matchMedia;
    return mm?.("(max-width: 760px) and (pointer: coarse)").matches ? PHONE_LAYERS : undefined;
  } catch {
    return undefined;
  }
}

/** What may be ticked here: bands the view draws (and the seat's sensors carry), at most `max` layers. */
export interface CompositeLimits {
  /** The bands this view draws and this seat can see. */
  can: readonly ViewBand[];
  /** Most layers at once (a phone: PHONE_LAYERS). */
  max?: number;
}

/** A mix cut to what can be drawn here: unknown or unseen bands dropped, a band once, at most `max` layers. */
export function fitMix(mix: CompositeMix, limits: CompositeLimits): CompositeMix {
  const seen = new Set<ViewBand>();
  const layers: CompositeLayer[] = [];
  for (const l of mix.layers) {
    if (!limits.can.includes(l.band) || seen.has(l.band)) continue;
    if (layers.length >= (limits.max ?? Infinity)) break;
    seen.add(l.band);
    layers.push({ band: l.band, colour: l.band === "visible" ? "natural" : validColour(l.colour) ?? LAYER_COLOURS[l.band] });
  }
  return { ...(mix.preset ? { preset: mix.preset } : {}), layers };
}

/** Tick or untick a band in a mix. Ticking one past the limit gives the reason instead. */
export function toggleLayer(mix: CompositeMix, band: ViewBand, limits: CompositeLimits): CompositeMix | { refused: string } {
  const has = mix.layers.some((l) => l.band === band);
  if (has) return { layers: mix.layers.filter((l) => l.band !== band) };
  if (!limits.can.includes(band)) return { refused: "This band is not drawn here" };
  if (limits.max !== undefined && mix.layers.length >= limits.max) return { refused: limits.max === PHONE_LAYERS ? WHY_PHONE_LAYERS : `At most ${limits.max} bands at once here` };
  return { layers: [...mix.layers, { band, colour: LAYER_COLOURS[band] }] };
}

/** Change one layer's colour (Visible stays natural). */
export function recolour(mix: CompositeMix, band: ViewBand, colour: string): CompositeMix {
  const c = validColour(colour);
  if (!c || band === "visible") return mix;
  return { layers: mix.layers.map((l) => (l.band === band ? { band, colour: c } : l)) };
}

function validColour(c: string | undefined): string | undefined {
  return typeof c === "string" && /^#[0-9a-fA-F]{6}$/.test(c) ? c.toLowerCase() : undefined;
}

/** A kept mix, read back (anything odd is dropped; an empty result is undefined). */
export function readMix(v: unknown, bands: readonly string[]): CompositeMix | undefined {
  const o = v as { layers?: unknown; preset?: unknown } | null;
  if (!o || !Array.isArray(o.layers)) return undefined;
  const layers = o.layers
    .map((l) => l as { band?: unknown; colour?: unknown })
    .filter((l) => typeof l.band === "string" && bands.includes(l.band))
    .map((l) => ({ band: l.band as ViewBand, colour: l.band === "visible" ? "natural" : (validColour(l.colour as string) ?? LAYER_COLOURS[l.band as ViewBand]) }));
  return layers.length ? { ...(typeof o.preset === "string" ? { preset: o.preset } : {}), layers } : undefined;
}

/** The colour names said in the legend, close to the starting colours; anything else is said as its hex. */
const COLOUR_WORDS: Record<string, string> = { "#ff4a3d": "red", "#4f7dff": "blue", "#b36bff": "violet", "#ff9a1f": "orange", "#38d6c4": "teal" };

/** The legend chip for a mix (design 6A): every layer and its colour, "mapped" once any band but Visible is in. */
export function compositeLegend(mix: CompositeMix, words: Record<ViewBand, string>): { text: string; mapped: boolean } {
  const parts = mix.layers.map((l) => (l.band === "visible" ? `${words.visible} actual` : `${words[l.band]} ${COLOUR_WORDS[l.colour] ?? l.colour}`));
  const mapped = mix.layers.some((l) => l.band !== "visible");
  const tail = mix.layers.length > 1 ? " · each band scaled to its own brightest" : "";
  return { text: `Composite: ${parts.join(" · ")}${tail}${mapped ? " · mapped" : ""}`, mapped };
}
