// A construct's pictures (Ships and stations card): the ship designed from its own parts, seen as plans or as a 3D
// model, kept as a recipe (style, seed, hand overrides), and any pictures the GM gives it. A selector shows any of
// them, and the last one chosen on the Build desk is the one it opens on (Alex, 24 Nov). All are plain facts on the
// construct, so they are dated, undoable and travel with the campaign. Presentation only: none feeds physics.

/** The GM's own pictures: a list of raster data URIs (no SVG: it can carry script). */
export const PICTURES_FIELD = "construct.pictures";
/** The designed ship's recipe: `{ style, seed, overrides? }` (renderer DesignRecipe). */
export const DESIGN_FIELD = "construct.design";
/** Which one it opens on: "plans", "model", or "picture:<n>" (0-based). */
export const SHOWN_FIELD = "construct.picture_shown";
/** Fields the card draws as a picture rather than listing as words. */
export const PICTURE_FIELDS = [PICTURES_FIELD, DESIGN_FIELD, SHOWN_FIELD];
/** Own pictures one construct keeps: a few views, not an album. */
export const PICTURES_MAX = 4;
/** About 150 KB of image once decoded: a card picture, not a poster. */
export const PICTURE_MAX = 200_000;
/** The longest side a given picture is scaled to before it is kept. */
export const PICTURE_SIDE = 960;
const PICTURE = /^data:image\/(png|jpeg|webp);base64,[A-Za-z0-9+/]+=*$/;

/** What is wrong with a picture, one line; null when it is fine. */
export function checkPicture(uri: string): string | null {
  if (!PICTURE.test(uri)) return "the picture must be a PNG, JPEG or WebP";
  if (uri.length > PICTURE_MAX) return "the picture is too big even when scaled down: try a simpler one";
  return null;
}

/** The kept pictures that are good ones, in order. */
export function picturesOf(v: unknown): string[] {
  return Array.isArray(v) ? v.filter((x): x is string => typeof x === "string" && checkPicture(x) === null).slice(0, PICTURES_MAX) : [];
}

export type PictureShown = "plans" | "model" | `picture:${number}`;

/** What to open on: the kept choice if it still exists, else the plans of a designed ship, else its first picture. */
export function shownOf(v: unknown, pictures: number, designed: boolean): PictureShown | null {
  if ((v === "plans" || v === "model") && designed) return v;
  const m = typeof v === "string" ? /^picture:(\d+)$/.exec(v) : null;
  if (m && Number(m[1]) < pictures) return v as PictureShown;
  return designed ? "plans" : pictures ? "picture:0" : null;
}

/** The kept design recipe, if it reads as one. Unknown keys are dropped; overrides pass through for the renderer to check. */
export function designOf(v: unknown): { style: string; seed: number; overrides?: Record<string, unknown> } | null {
  if (!v || typeof v !== "object" || Array.isArray(v)) return null;
  const o = v as Record<string, unknown>;
  if (typeof o.style !== "string" || !o.style || typeof o.seed !== "number" || !Number.isInteger(o.seed)) return null;
  const ov = o.overrides && typeof o.overrides === "object" && !Array.isArray(o.overrides) ? (o.overrides as Record<string, unknown>) : null;
  return { style: o.style, seed: o.seed, ...(ov && Object.keys(ov).length ? { overrides: ov } : {}) };
}

/** Width and height that fit `side` on the longer edge, never scaling up. */
export function fitSide(w: number, h: number, side = PICTURE_SIDE): [number, number] {
  const k = Math.min(1, side / Math.max(w, h, 1));
  return [Math.max(1, Math.round(w * k)), Math.max(1, Math.round(h * k))];
}
