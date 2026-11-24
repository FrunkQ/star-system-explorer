// A construct's picture (Ships and stations card): the ship designed from its own parts by default, kept as a recipe
// (style, seed, hand overrides), or a picture the GM gives it. Both are plain facts on the construct, so they are
// dated, undoable and travel with the campaign. Presentation only: neither feeds physics.

/** The GM's own picture: a raster data URI (no SVG: it can carry script), or "" for none. */
export const PICTURE_FIELD = "construct.picture";
/** The designed ship's recipe: `{ style, seed, overrides? }` (renderer DesignRecipe). */
export const DESIGN_FIELD = "construct.design";
/** Fields the card draws as a picture rather than listing as words. */
export const PICTURE_FIELDS = [PICTURE_FIELD, DESIGN_FIELD];
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

/** The kept picture, if there is a good one. */
export function pictureOf(v: unknown): string | null {
  return typeof v === "string" && v !== "" && checkPicture(v) === null ? v : null;
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
