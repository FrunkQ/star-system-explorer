// The place line (top-level design 2.1): where the GM is, from the largest
// scale down to the one in view. One ladder serves every scale: going up is a
// breadcrumb, going down is the list of what is nested below the current step,
// and the steps below the current one that the GM came from stay on the line
// (quietly) so "back to where I was" is one press. A place is also a deep link
// (the address bar's hash), and the last few places are kept as "Recent".
// Presentation only: none of this reaches the kernel or the state hash.

/** The scales of the ladder, largest first. "world" is any body; "site" is a place on one (later). */
export const PLACE_SCALES = ["web", "cluster", "galaxy", "system", "world"] as const;
export type PlaceScale = (typeof PLACE_SCALES)[number];

/** The words for a scale when it has no name of its own. */
export const PLACE_WORDS: Record<PlaceScale, string> = {
  web: "Cosmic web",
  cluster: "Galaxies nearby",
  galaxy: "Galaxy",
  system: "System",
  world: "World",
};

/** Where the GM is: the scale in view, the system on the chart, the world (when at world scale) and the screen. */
export interface Place {
  scale: PlaceScale;
  /** The system's address ("" for the base system). */
  system: string;
  /** The world's address, at world scale. */
  world?: string;
  /** The screen in front, when it is not the scale's first. */
  screen?: string;
  /** Above a system: which one at that scale, in the scale view's own id ("cell:0.0.0", a halo address, "web:200"). */
  id?: string;
  /** Its name, or a world's name, when the place was visited, for the Recent list (not part of the link). */
  label?: string;
}

/** One step of the line. `current` is the scale in view; `below` marks a step under it that the GM came from. */
export interface PlaceStep {
  scale: PlaceScale;
  label: string;
  place: Place;
  current: boolean;
  below: boolean;
}

export function placeRank(s: PlaceScale): number {
  return PLACE_SCALES.indexOf(s);
}
export function isPlaceScale(s: unknown): s is PlaceScale {
  return typeof s === "string" && (PLACE_SCALES as readonly string[]).includes(s);
}

/**
 * The line for a place. Every scale above it is a step. The trail is the deepest
 * place the GM has been on this line: while the place in view is above it (same
 * system), the trail's steps below show too, marked `below`.
 */
export function placeSteps(here: Place, trail: Place | null, names: { system: string; world?: string | null; above?: Partial<Record<PlaceScale, { id: string; name: string }>> }): PlaceStep[] {
  const deepest = trail && trail.system === here.system && placeRank(trail.scale) > placeRank(here.scale) ? trail : here;
  const out: PlaceStep[] = [];
  for (const scale of PLACE_SCALES) {
    if (placeRank(scale) > placeRank(deepest.scale)) break;
    const known = names.above?.[scale];
    const place: Place = scale === "world" ? { scale, system: here.system, world: deepest.world! } : { scale, system: here.system, ...(known ? { id: known.id } : {}) };
    const label = scale === "system" ? names.system : scale === "world" ? (names.world ?? PLACE_WORDS.world) : (known?.name ?? PLACE_WORDS[scale]);
    out.push({ scale, label, place, current: scale === here.scale, below: placeRank(scale) > placeRank(here.scale) });
  }
  return out;
}

/** The same place? (The screen does not count: a place is where, not how.) */
export function samePlace(a: Place | null | undefined, b: Place | null | undefined): boolean {
  return !!a && !!b && a.scale === b.scale && a.system === b.system && (a.scale !== "world" || a.world === b.world) && (a.id ?? "") === (b.id ?? "");
}

/** The trail after moving to `next`: going down replaces it; going up keeps it while the system is the same. */
export function nextTrail(trail: Place | null, next: Place): Place {
  if (!trail || trail.system !== next.system || placeRank(next.scale) >= placeRank(trail.scale)) return next;
  return trail;
}

/** Recent places, newest first, without repeats, at most `max`. */
export function rememberPlace(recent: readonly Place[], p: Place, max = 6): Place[] {
  return [p, ...recent.filter((r) => !samePlace(r, p))].slice(0, max);
}

/**
 * A place as the address bar's hash: `#chart/<scale>[/<system>[/<world or id>]][~<screen>]`.
 * `#chart` alone is the base system (it also skips the start sheet).
 */
export function placeHash(p: Place): string {
  const parts = ["chart"];
  if (p.scale !== "system" || p.system || p.world) {
    parts.push(p.scale);
    const third = p.scale === "world" ? p.world : p.id;
    if (p.system || third) parts.push(encodeURIComponent(p.system));
    if (third) parts.push(encodeURIComponent(third));
  }
  return `#${parts.join("/")}${p.screen ? `~${encodeURIComponent(p.screen)}` : ""}`;
}

/** The place a hash names, or null when it names none (a bad scale, or not `#chart`). */
export function parsePlaceHash(hash: string): Place | null {
  const m = /^#chart(?:\/([^/~]*))?(?:\/([^/~]*))?(?:\/([^/~]*))?(?:~(.*))?$/.exec(hash);
  if (!m) return null;
  const scale = m[1] === undefined || m[1] === "" ? "system" : m[1];
  if (!isPlaceScale(scale)) return null;
  const dec = (s: string | undefined) => {
    try {
      return s ? decodeURIComponent(s) : "";
    } catch {
      return "";
    }
  };
  const p: Place = { scale, system: dec(m[2]) };
  if (scale === "world") {
    const w = dec(m[3]);
    if (!w) return null;
    p.world = w;
  } else if (scale !== "system" && m[3]) {
    const id = dec(m[3]);
    if (id) p.id = id;
  }
  const screen = dec(m[4]);
  if (screen) p.screen = screen;
  return p;
}

/** Recent places as kept on this device (a per-viewer convenience: bad entries are dropped). */
export function readRecent(json: string | null): Place[] {
  try {
    const v = JSON.parse(json ?? "[]");
    if (!Array.isArray(v)) return [];
    return v.filter((p): p is Place => !!p && typeof p === "object" && isPlaceScale(p.scale) && typeof p.system === "string" && (p.scale !== "world" || typeof p.world === "string")).slice(0, 6);
  } catch {
    return [];
  }
}
