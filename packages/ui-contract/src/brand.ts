// Branding and join words on player views (V3 gap 25). A seat may carry a
// brand (a faction's name and logo) that its screens show; the words a screen
// says while it joins come from the stacked packs (`provides.other.vocabulary.join`, the vocabulary layer of `words.ts`;
// the topmost pack's wording wins, line by line), else the app's plain words.
// Presentation only: none of it reaches the kernel or the campaign's physics.

/** A seat's brand: a name, and optionally a small raster logo as a data URI (no SVG: it can carry script). */
export interface SeatBrand {
  name: string;
  logo?: string;
  /** One colour for rules and stamps on the role's letterhead (Atlas design 7.1, AT6): "#rrggbb". */
  accent?: string;
  /** One line under the name on its letterhead ("Moving cargo since 2240"). */
  motto?: string;
}

export const BRAND_NAME_MAX = 40;
export const BRAND_MOTTO_MAX = 60;
/** About 64 KB of image once decoded. */
export const BRAND_LOGO_MAX = 90_000;
const LOGO = /^data:image\/(png|jpeg|webp);base64,[A-Za-z0-9+/]+=*$/;

/** What is wrong with a brand, one line each; empty when it is fine. */
export function checkBrand(b: SeatBrand): string[] {
  const out: string[] = [];
  if (!b.name.trim()) out.push("a brand needs a name");
  if (b.name.length > BRAND_NAME_MAX) out.push(`the brand name is longer than ${BRAND_NAME_MAX} characters`);
  if (b.logo !== undefined) {
    if (!LOGO.test(b.logo)) out.push("the logo must be a PNG, JPEG or WebP picture");
    else if (b.logo.length > BRAND_LOGO_MAX) out.push("the logo is too big: keep it under about 64 KB");
  }
  if (b.accent !== undefined && !/^#[0-9a-f]{6}$/i.test(b.accent)) out.push("the accent must be a colour written #rrggbb");
  if (b.motto !== undefined && b.motto.length > BRAND_MOTTO_MAX) out.push(`the motto is longer than ${BRAND_MOTTO_MAX} characters`);
  return out;
}

/** The quick overrides the Seats panel offers (V3 gap 25): a player view parameter, the switch's words and the value it sets while on. */
export const LIVE_OVERRIDES: readonly { param: string; label: string; on: boolean }[] = [{ param: "labels", label: "Hide labels", on: false }];

/** A screen's parameters with a seat's live overrides laid over them, only where the view has that parameter. */
export function withLive<P extends Record<string, unknown>>(params: P, declared: readonly { name: string }[], live: Record<string, boolean> | undefined): P {
  if (!live) return params;
  const out: Record<string, unknown> = { ...params };
  for (const [k, v] of Object.entries(live)) if (declared.some((d) => d.name === k)) out[k] = v;
  return out as P;
}

/** What a player screen says while it joins and when the GM is away. `{seat}` is the seat's name, `{brand}` its brand's. */
export interface JoinWords {
  joining: string;
  waiting: string;
  away: string;
}
export const DEFAULT_JOIN: JoinWords = {
  joining: "Joining the game master…",
  waiting: "Waiting for the scene…",
  away: "The game master is away. This is the last state they shared, read only. It updates when they are back.",
};

/** The join words of a stack, bottom to top: each line from the topmost pack that words it, else the default. */
export function joinWordsOf(packs: readonly unknown[]): JoinWords {
  const out: JoinWords = { ...DEFAULT_JOIN };
  for (const p of packs) {
    const j = ((p as { provides?: { other?: { vocabulary?: { join?: Partial<Record<keyof JoinWords, unknown>> } } } }).provides?.other?.vocabulary?.join) ?? {};
    for (const k of ["joining", "waiting", "away"] as const) {
      const v = j[k];
      if (typeof v === "string" && v.trim()) out[k] = v;
    }
  }
  return out;
}

/** A join line with its places filled. */
export function joinLine(text: string, at: { seat?: string; brand?: string }): string {
  return text.replace(/\{seat\}/g, at.seat || "this seat").replace(/\{brand\}/g, at.brand || at.seat || "this seat");
}
