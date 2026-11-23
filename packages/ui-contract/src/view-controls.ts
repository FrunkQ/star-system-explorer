// The standard view controls (interface style rule 12, Alex 22 Nov 2026): every diagram display offers the same two
// choices, in the same place, with the same words. **View style**: Chart, 3D, True, Wireframe. **Seen from**: Above,
// Tilted, Edge-on, Free. A view declares what it can draw (`ViewModule.controls`); what it cannot is still listed,
// quiet, with the reason, so the control never rearranges itself from one screen to the next. The choice belongs to
// the viewer (any seat), is kept on their device, and later per panel. Presentation only: never in the campaign.

export const VIEW_STYLES = ["chart", "3d", "true", "wireframe"] as const;
export type ViewStyle = (typeof VIEW_STYLES)[number];
export const SEEN_FROM = ["above", "tilted", "edge", "free"] as const;
export type SeenFrom = (typeof SEEN_FROM)[number];

/** Scale (Alex, 23 Nov 2026): Readable exaggerates sizes and spacing so everything can be seen (the default); Real
 * draws sizes and distances as they truly are. */
export const SCALES = ["readable", "real"] as const;
export type ViewScale = (typeof SCALES)[number];
export const SCALE_WORDS: Record<ViewScale, string> = { readable: "Readable", real: "Real" };

/** HUD (Alex, 23 Nov 2026): one switch for the clutter over a True view (orbit lines, names, outlines). Off looks
 * like a photograph, on like a heads-up overview. Off by default. */
export const HUD_WORDS: Record<"off" | "on", string> = { off: "Off", on: "On" };

/** Band (the instrument): what light, or field, the view is seen in. Visible by default. Mag replaces the picture with
 * the magnetic field (magnetic fields design, 22 Nov 2026). */
export const VIEW_BANDS = ["visible", "infrared", "ultraviolet", "xray", "radio", "mag"] as const;
export type ViewBand = (typeof VIEW_BANDS)[number];
export const VIEW_BAND_WORDS: Record<ViewBand, string> = { visible: "Visible", infrared: "Infrared", ultraviolet: "Ultraviolet", xray: "X-ray", radio: "Radio", mag: "Mag" };

export const VIEW_STYLE_WORDS: Record<ViewStyle, string> = { chart: "Chart", "3d": "3D", true: "True", wireframe: "Wireframe" };
export const SEEN_WORDS: Record<SeenFrom, string> = { above: "Above", tilted: "Tilted", edge: "Edge-on", free: "Free" };
/** How Free is worked, said the same way everywhere. */
export const FREE_HINT = "Free view: drag to turn and tilt, Shift-drag or two fingers to slide, wheel or pinch to zoom.";

/** `true`: the view draws it. A string: it does not, and this is why (shown as the option's title). */
export type Can = true | string;

export interface ViewControlsSpec {
  /** What the view draws in each style. A style left out is "not drawn this way yet". */
  styles: Partial<Record<ViewStyle, Can>>;
  /** Chart is drawn flat, seen from above (the system's log chart, a slice). Turning it switches to 3D when it can. */
  flat?: boolean;
  /** Which ways it can be seen from, in its turnable styles. Left out: all four. */
  seen?: Partial<Record<SeenFrom, Can>>;
  /** Which scales it draws. Left out: Readable only, Real "not drawn to scale here yet". */
  scale?: Partial<Record<ViewScale, Can>>;
  /** Which bands it is seen in. Left out: Visible only, the others "not seen in this band here yet". */
  bands?: Partial<Record<ViewBand, Can>>;
  /** Whether it can turn its HUD (orbit lines, names, outlines) off and on, per style. Left out: not in any style
   * (the option is listed quiet with the reason, and the view shows its labels). */
  hud?: Partial<Record<ViewStyle, Can>>;
  /** What the view opens on each time it is opened, whatever the viewer last chose elsewhere (the size comparison
   * opens at Real, the point of the tool). The viewer can change it there; their own choice comes back when they leave. */
  opens?: { scale?: ViewScale };
}

/** The viewer's choice, kept on the device. */
export interface ViewChoice {
  style: ViewStyle;
  seen: SeenFrom;
  /** Readable when left out (choices kept before the Scale switch). */
  scale?: ViewScale;
  /** HUD on or off. Left out: off in True (a photograph), on in the diagrams; once chosen it holds in every style. */
  hud?: boolean;
  /** Visible when left out. */
  band?: ViewBand;
}
export const DEFAULT_VIEW: ViewChoice = { style: "chart", seen: "tilted", scale: "readable" };

export interface ViewOption<T extends string> {
  value: T;
  label: string;
  unavailable?: string;
}

const NOT_YET = (s: ViewStyle) => `${VIEW_STYLE_WORDS[s]} is not drawn here yet`;
const can = (c: Can | undefined): c is true => c === true;

/** The style a view shows for a choice: the chosen one when it can, else Chart, else its first. */
export function shownStyle(spec: ViewControlsSpec, choice: ViewChoice): ViewStyle {
  if (can(spec.styles[choice.style])) return choice.style;
  if (can(spec.styles.chart)) return "chart";
  return VIEW_STYLES.find((s) => can(spec.styles[s])) ?? "chart";
}

/** What the control lists for this view and choice, and what is shown. Every option is always listed. */
export function viewOptions(spec: ViewControlsSpec, choice: ViewChoice): { styles: ViewOption<ViewStyle>[]; seen: ViewOption<SeenFrom>[]; scales: ViewOption<ViewScale>[]; huds: ViewOption<"off" | "on">[]; bands: ViewOption<ViewBand>[]; shown: Required<ViewChoice>; turnable: boolean } {
  const style = shownStyle(spec, choice);
  const flat = !!spec.flat && style === "chart";
  const turnTo = flat && can(spec.styles["3d"]);
  const styles = VIEW_STYLES.map((s) => {
    const c = spec.styles[s];
    return { value: s, label: VIEW_STYLE_WORDS[s], ...(can(c) ? {} : { unavailable: c ?? NOT_YET(s) }) };
  });
  const seen = SEEN_FROM.map((s) => {
    const c = spec.seen ? spec.seen[s] : true;
    let why: string | undefined = c === true ? undefined : (c ?? `Not seen ${SEEN_WORDS[s].toLowerCase()} here yet`);
    if (!why && flat && s !== "above" && !turnTo) why = `This ${VIEW_STYLE_WORDS[style].toLowerCase()} is flat, seen from above`;
    return { value: s, label: SEEN_WORDS[s], ...(why ? { unavailable: why } : {}) };
  });
  const ok = (s: SeenFrom) => !seen.find((o) => o.value === s)!.unavailable;
  const shownSeen: SeenFrom = flat ? "above" : ok(choice.seen) ? choice.seen : (SEEN_FROM.find(ok) ?? "above");
  const scales = scaleOptions(spec);
  const want = choice.scale ?? "readable";
  // A flat chart that turns into 3D shows Real there (pickScale); one that cannot turn (a slice) draws Real itself
  // when it says it can.
  const shownScale: ViewScale = !(flat && turnTo) && scales.find((o) => o.value === want && !o.unavailable) ? want : "readable";
  // A diagram always shows its labels; the HUD switch is for a view that can look like a photograph.
  const h = spec.hud?.[style];
  const hudWhy = h === true ? undefined : (h ?? `${VIEW_STYLE_WORDS[style]} always shows its labels`);
  const huds: ViewOption<"off" | "on">[] = (["off", "on"] as const).map((v) => ({ value: v, label: HUD_WORDS[v], ...(hudWhy ? { unavailable: hudWhy } : {}) }));
  const shownHud = hudWhy ? true : (choice.hud ?? style !== "true");
  const bands: ViewOption<ViewBand>[] = VIEW_BANDS.map((b) => {
    const c = spec.bands ? spec.bands[b] : b === "visible" ? true : undefined;
    const why = c === true ? undefined : (c ?? `Not seen in ${VIEW_BAND_WORDS[b] === "Mag" ? "Mag" : VIEW_BAND_WORDS[b].toLowerCase()} here yet`);
    return { value: b, label: VIEW_BAND_WORDS[b], ...(why ? { unavailable: why } : {}) };
  });
  const wantBand = choice.band ?? "visible";
  const shownBand: ViewBand = bands.find((o) => o.value === wantBand && !o.unavailable) ? wantBand : "visible";
  return { styles, seen, scales, huds, bands, shown: { style, seen: shownSeen, scale: shownScale, hud: shownHud, band: shownBand }, turnable: !flat || turnTo };
}

const NOT_TO_SCALE = "Not drawn to scale here yet";
function scaleOptions(spec: ViewControlsSpec): ViewOption<ViewScale>[] {
  return SCALES.map((s) => {
    const c = spec.scale ? spec.scale[s] : s === "readable" ? true : NOT_TO_SCALE;
    return { value: s, label: SCALE_WORDS[s], ...(c === true ? {} : { unavailable: c ?? NOT_TO_SCALE }) };
  });
}

/** Choosing a scale keeps the style, except that Real on a flat chart turns it into 3D (a flat log chart has no real scale). */
export function pickScale(spec: ViewControlsSpec, choice: ViewChoice, scale: ViewScale): ViewChoice {
  const style = shownStyle(spec, choice);
  if (scale === "real" && spec.flat && style === "chart" && can(spec.styles["3d"])) return { ...choice, style: "3d", seen: choice.seen === "above" ? "tilted" : choice.seen, scale };
  return { ...choice, scale };
}

/** Choosing a way to see it from: on a flat chart, anything but Above turns it into 3D (when the view has 3D). */
export function pickSeen(spec: ViewControlsSpec, choice: ViewChoice, seen: SeenFrom): ViewChoice {
  const style = shownStyle(spec, choice);
  const flat = !!spec.flat && style === "chart";
  if (flat && seen !== "above" && can(spec.styles["3d"])) return { style: "3d", seen };
  if (flat && seen === "above") return choice;
  return { ...choice, style, seen };
}

/** Choosing a style keeps the way it is seen from. */
export function pickStyle(choice: ViewChoice, style: ViewStyle): ViewChoice {
  return { ...choice, style };
}

/** A kept choice, read back (a per-viewer convenience: anything odd falls back to the default). */
export function readViewChoice(json: string | null): ViewChoice {
  try {
    const v = JSON.parse(json ?? "null") as Partial<ViewChoice> | null;
    return {
      style: (VIEW_STYLES as readonly string[]).includes(String(v?.style)) ? (v!.style as ViewStyle) : DEFAULT_VIEW.style,
      seen: (SEEN_FROM as readonly string[]).includes(String(v?.seen)) ? (v!.seen as SeenFrom) : DEFAULT_VIEW.seen,
      scale: (SCALES as readonly string[]).includes(String(v?.scale)) ? (v!.scale as ViewScale) : "readable",
      ...(typeof v?.hud === "boolean" ? { hud: v.hud } : {}),
      ...((VIEW_BANDS as readonly string[]).includes(String(v?.band)) ? { band: v!.band as ViewBand } : {}),
    };
  } catch {
    return { ...DEFAULT_VIEW };
  }
}

/** The fixed views as an elevation, for views that orbit a camera: radians above the plane. Free keeps its own. */
export const SEEN_ELEVATION: Record<Exclude<SeenFrom, "free">, number> = { above: 1.45, tilted: 0.55, edge: 0.02 };

/** Shared reasons, so the same gap reads the same everywhere. */
export const WHY_TRUE_HELD = "True is coming: the photoreal views are held for now";
export const WHY_WIREFRAME_LATER = "Wireframe is coming with the view styles";
/** A diagram drawn flat for now: Chart only, seen from above. */
export function flatChart(why3d = "3D is not drawn here yet"): ViewControlsSpec {
  return { styles: { chart: true, "3d": why3d, true: WHY_TRUE_HELD, wireframe: WHY_WIREFRAME_LATER }, flat: true };
}

/** The lens layers a diagram hides when HUD is off: the overlay of scale, survey, distances, orbit lines and names. */
export const HUD_OVERLAY_LAYERS: readonly string[] = ["grid", "unsurveyed", "distances", "orbits", "labels"];
/** Whether a lens layer is drawn under this HUD choice. */
export const hudShows = (hud: boolean, layer: string): boolean => hud || !HUD_OVERLAY_LAYERS.includes(layer);
