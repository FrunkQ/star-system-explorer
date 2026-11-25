// The living star with everything on it (SG1 to SG3): the face (churning cells, lumpy limb on giants, spots, flare
// patches) and the halo round it (the eye's glare, brightened by flares, with mass ejections leaving). Views that
// show a star turning on its own clock call this; star.ts and live-star.ts stay the parts it is made of.
import { thermalColour } from "../../ui-contract/src/index.ts";
import { coronaScaleHeight, drawStar, granulesShow, type Convection, type StarDraw } from "./star.ts";
import { spotsAt, turnAt, flarePlace, type LiveStar } from "./live-star.ts";
import { churningGranules, granuleBlur, limbLumps, narrateLumps } from "./star-churn.ts";
import { addEjections, ejectionsAt, flareBoost, flareWhiteLight, narrateEvents, patchContrast, patchRadiusDeg, shownFlares, type EjectionNow, type ShownFlare } from "./star-events.ts";
import { drawGlow, glareProfile, GLOW_REACH_DEG, narrateGlow } from "./star-glow.ts";

export interface StarNowAsk {
  /** Star time, seconds of age. */
  t: number;
  /** Star seconds per screen second (the view's own clock). */
  clock: number;
  /** Star seconds each picture stands for (a picture every tenth of a second: clock / 10). */
  span: number;
  /** The viewer asked for reduced motion: nothing flashes. */
  calm?: boolean;
  light?: StarDraw["light"];
  /** The disc's share of the face's square. */
  disc?: number;
  /** Tilt of the pole toward the viewer, degrees. */
  lat_deg?: number;
  /** The drawn disc's angular radius as the camera sees it, degrees (for the glare). */
  disc_deg?: number;
  /** Flux at the eye over the exposure's (star-glow.ts). */
  exposure?: number;
}

export interface StarNow {
  shown: ShownFlare[];
  ejections: EjectionNow[];
  /** The star's light now over its quiet light. */
  boost: number;
  /** The halo's half-width in stellar radii (GLOW_REACH_DEG over the disc's angular radius). */
  reach: number;
  /** Words for the note beside the view. */
  words: string[];
}

const profiles = new Map<number, Float64Array>();
/** The glare profile for a disc size, kept in quarter-octave steps of angular radius (it depends on nothing else). */
function profileFor(disc_deg: number): Float64Array {
  const step = Math.round(Math.log2(Math.max(1e-4, disc_deg)) * 4);
  const hit = profiles.get(step);
  if (hit) return hit;
  const p = glareProfile(2 ** (step / 4), 96);
  if (profiles.size > 32) profiles.clear();
  profiles.set(step, p);
  return p;
}

const decode = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);

/**
 * Draw the star at star time `ask.t` into `face` (the disc) and, given, `halo` (the glow and ejections, a square whose
 * half-width is GLOW_REACH_DEG of the camera's view). Returns what it showed, for the view's note and checks.
 */
export function drawStarNow(face: { width: number; data: Uint8ClampedArray }, halo: { width: number; data: Uint8ClampedArray } | null, s: LiveStar, ask: StarNowAsk): StarNow {
  const light = ask.light ?? "visible";
  const disc = ask.disc ?? 0.92;
  const lat = ask.lat_deg ?? 7;
  const conv = (s.activity.convection ?? null) as unknown as Convection | null;
  const discPx = face.width * disc;
  const shown = shownFlares(s, ask.t, ask.clock, ask.calm);
  // Spots drift by latitude (differential rotation); under reduced motion they hold at the star's present.
  const spots = spotsAt(s, ask.calm ? s.age_s : ask.t);
  const xr = Math.max(1, Number(s.moment.xray_w));
  const flares = shown.map(({ flare, index, k }) => {
    const at = flarePlace(s, flare);
    const sp = spots[index % Math.max(1, spots.length)];
    const wl = flareWhiteLight(flare);
    return {
      lat_deg: at?.lat_deg ?? sp?.lat_deg ?? 15,
      lon_deg: at?.lon_deg ?? sp?.lon_deg ?? 0,
      strength: (k * flare.peak_w) / xr,
      ...(wl ? { patch: { radius_deg: patchRadiusDeg(wl.area_m2, s.radius), contrast: patchContrast(wl.t_k, s.teff), k } } : {}),
    };
  });
  const showCells = !!conv && light === "visible" && granulesShow(conv.granules_across, discPx);
  // Under reduced motion the cells hold still at the star's present.
  const cellT = ask.calm ? s.age_s : ask.t;
  const granules = showCells ? churningGranules(s.address, conv!.granules_across, cellT, conv!.granule_lifetime_s) : null;
  const lumps = limbLumps(conv, s.radius);
  drawStar(face, { teff_k: s.teff, spots, lon_deg: turnAt(s, ask.t), lat_deg: lat, light, corona_h: coronaScaleHeight(s.mass, s.radius), flares, granules, granuleContrast: showCells && !ask.calm ? granuleBlur(conv!.granule_lifetime_s, ask.span) : 1, lumps }, disc);
  const boost = flareBoost(s, shown);
  const discDeg = Math.max(1e-4, ask.disc_deg ?? 0.25);
  const reach = GLOW_REACH_DEG / discDeg;
  const ejections = ejectionsAt(s, ask.t, reach, ask.calm);
  const words: string[] = [];
  if (halo && light === "visible") {
    const profile = profileFor(discDeg);
    drawGlow(halo, { teff_k: s.teff, disc_deg: discDeg, exposure: ask.exposure ?? 1, boost, profile });
    const rgb = (thermalColour(s.teff).match(/[0-9a-f]{2}/gi) ?? ["ff", "ff", "ff"]).map((h) => decode(parseInt(h, 16) / 255)) as [number, number, number];
    addEjections(halo, ejections, reach, rgb, lat);
    words.push(narrateGlow(profile, ask.exposure ?? 1));
  }
  const lumpWords = light === "visible" && granules ? narrateLumps(lumps, discPx) : "";
  if (lumpWords) words.push(lumpWords);
  words.push(narrateEvents(s, shown, light === "visible" ? ejections : [], !!ask.calm));
  return { shown, ejections, boost, reach, words };
}

/** Face sizes in pixels: drawn at about the size the disc shows, in steps so the texture is not remade at each zoom. */
export const FACE_STEPS = [16, 32, 64, 128, 256] as const;

const faces = new Map<string, { width: number; data: Uint8ClampedArray; version: number; drawn: string; now: StarNow }>();

/**
 * A star's face for a view's GPU item (gpu.ts `GpuItem.face`), for views that draw many bodies through true-body.ts:
 * one buffer per key, redrawn only when the moment or size changes, `px` the disc's diameter on screen in device
 * pixels. No halo: the caller draws the glare where it has a camera to measure it by.
 */
export function liveFace(key: string, s: LiveStar, ask: StarNowAsk & { px: number }): { face: { key: string; version: number; width: number; data: Uint8ClampedArray; disc: number }; now: StarNow } {
  const disc = ask.disc ?? 0.92;
  const width = FACE_STEPS.find((n) => n >= ask.px / disc) ?? FACE_STEPS[FACE_STEPS.length - 1]!;
  const drawn = `${width}|${ask.t}|${ask.light ?? "visible"}|${ask.calm ? 1 : 0}|${disc}|${ask.lat_deg ?? 7}`;
  let f = faces.get(key);
  if (!f || f.width !== width) {
    if (faces.size > 64) faces.clear();
    f = { width, data: new Uint8ClampedArray(width * width * 4), version: 0, drawn: "", now: { shown: [], ejections: [], boost: 1, reach: 0, words: [] } };
    faces.set(key, f);
  }
  if (f.drawn !== drawn) {
    f.now = drawStarNow(f, null, s, ask);
    f.drawn = drawn;
    f.version++;
  }
  return { face: { key: `star:${key}:${ask.light === "xray" ? "x" : "v"}`, version: f.version, width, data: f.data, disc }, now: f.now };
}
