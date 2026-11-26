// The class gallery's pictures (V14i, V3 gaps item 35): each class drawn from a sample world the kernel makes for it
// (`class_sample`: a mid-band world of the class, hand-set on a what-if branch), its face in True from the one door every
// True view uses (`readTrueFace` and `trueLook`), drawn as one turn on its own axis (`trueTurn`): a strip of stills that
// a tile steps through, so a page of globes turns with no GPU and no kernel of its own. A picture is a look at one world
// that is this class, never the class itself, and the caption says which and how well it fits. Nothing is committed: the
// branch is dropped once its world is drawn.
//
// Part 2: the shipped atlases' worlds are fixed (their examples are the atlas's own), so their strips are drawn once,
// ahead of time, by scripts/build-class-pictures.mjs, and shipped beside the base presentation pack
// (`class-pictures.json`, served as `packs/base-class-pictures.json` by scripts/build-shell.sh). A gallery showing a shipped atlas at the version they were drawn for shows them at once, on
// any seat and any device. Any other atlas (an edited one, a newer version) is drawn live on the game master's seat.
import type { ClassSample, PackDoc, ViewSession } from "../../../ui-contract/src/index.ts";
import { readTrueFace, trueTurn, type FaceReader } from "../../../renderer/src/index.ts";

/** A turning globe: `sheet` is `frames` squares side by side, one turn; `spin` -1 turns it backwards. */
export type ClassPicture = { state: "drawing" } | { state: "drawn"; sheet: string; frames: number; size: number; caption: string; baked: boolean } | { state: "none"; why: string };

/** Pictures by `atlas|class`, filled as they are drawn. */
export const classPictures: Record<string, ClassPicture> = $state({});

const keyOf = (atlas: string, cls: string) => `${atlas}|${cls}`;
export const pictureOf = (atlas: string, cls: string): ClassPicture | undefined => classPictures[keyOf(atlas, cls)];

/** The smallest sample world drawn live, metres of radius (see drawSample). */
const SMALLEST_DRAWN_M = 50_000;

/** The strip's size and length, here and in the pre-drawn set. */
export const TURN = { size: 112, frames: 36 } as const;

// ---- The pre-drawn set (sse.class-pictures/1) ----

export interface ClassPictureSet {
  schema: "sse.class-pictures/1";
  note?: string;
  /** The look they were drawn with (scripts/class-pictures-look.mjs); shown only while the build's look is the same. */
  look?: string;
  /** Per atlas pack: the version drawn for, and each class's strip. */
  atlases: { pack: string; version: string; size: number; frames: number; pictures: Record<string, { sheet: string; caption: string } | { none: string }> }[];
}

// The build's look (packages/shell/vite.config.ts); "dev" where the gallery runs outside the shell's build.
declare const __SSE_LOOK__: string;
const LOOK: string = typeof __SSE_LOOK__ === "string" ? __SSE_LOOK__ : "dev";

/** Where the shell serves the pre-drawn set (scripts/build-shell.sh). */
const SET_AT = "packs/base-class-pictures.json";

let shipped: Promise<ClassPictureSet | null> | null = null;
/** The pre-drawn set, fetched once, on first need (it is a few megabytes; the gallery's first page waits on it, nothing else does). */
function shippedSet(): Promise<ClassPictureSet | null> {
  let live = false;
  try {
    live = localStorage.getItem("sse.class-pictures.live") === "1";
  } catch {}
  if (live) return Promise.resolve(null);
  shipped ??= fetch(new URL(SET_AT, document.baseURI))
    .then((r) => (r.ok ? (r.json() as Promise<ClassPictureSet>) : Promise.reject(new Error(`HTTP ${r.status}`))))
    // Drawn with another look (the renderer or the surfaces have moved on since): drawn live until the set is drawn again.
    .then((set) => (set.look === LOOK ? set : null))
    .catch(() => null);
  return shipped;
}

/** Fill pictures for these classes from the pre-drawn set where it holds the class's atlas at its version; the rest are returned. */
async function fromShipped(atlas: string, classes: readonly string[], docOf: (cls: string) => PackDoc | null): Promise<string[]> {
  const set = await shippedSet();
  const rest: string[] = [];
  for (const c of classes) {
    const doc = docOf(c);
    const a = doc && set?.atlases.find((x) => x.pack === doc.id && x.version === String(doc.version ?? ""));
    const p = a?.pictures[c];
    if (!a || !p) rest.push(c);
    else if ("none" in p) classPictures[keyOf(atlas, c)] = { state: "none", why: p.none };
    else classPictures[keyOf(atlas, c)] = { state: "drawn", sheet: p.sheet, frames: a.frames, size: a.size, caption: p.caption, baked: true };
  }
  return rest;
}

// ---- Live drawing, one class at a time ----

interface Job {
  session: ViewSession;
  atlas: string;
  cls: string;
  extra: readonly PackDoc[];
}
// Waiting jobs, first first: a class opened jumps the queue; a page's tiles go on the end in order.
const waiting: Job[] = [];
let running = false;
/** The classes still wanted (the page showing, the class opened): a job no longer wanted is dropped, to be asked again. */
let wanted = new Set<string>();

/**
 * Show pictures for these classes: from the pre-drawn set where it holds them, else drawn live on the game master's
 * seat, one at a time (each reads a world's tiles, a second or more). `first` puts them ahead of the page's (the class
 * opened). `extra` are atlas packs shown but not on the stack; `docOf` gives the pack a class comes from, for its version.
 */
export function drawClassPictures(session: ViewSession, atlas: string, classes: readonly string[], extra: readonly PackDoc[] = [], o: { first?: boolean; docOf?: (cls: string) => PackDoc | null; page?: boolean; live?: boolean } = {}): void {
  // Drawn again live (the game master's "Draw again"): this device's drawing, never the pre-drawn set.
  if (o.live) for (const c of classes) if (classPictures[keyOf(atlas, c)]?.state !== "drawing") delete classPictures[keyOf(atlas, c)];
  const ask = classes.filter((c) => !classPictures[keyOf(atlas, c)]);
  if (o.page) wanted = new Set(classes.map((c) => keyOf(atlas, c)));
  for (const c of classes) wanted.add(keyOf(atlas, c));
  if (!ask.length) return;
  for (const c of ask) classPictures[keyOf(atlas, c)] = { state: "drawing" };
  void (o.live ? Promise.resolve(ask) : fromShipped(atlas, ask, o.docOf ?? (() => null))).then((rest) => {
    if (!rest.length) return;
    if (session.seat.authority !== "gm" || !session.classSample) {
      for (const c of rest) delete classPictures[keyOf(atlas, c)];
      return;
    }
    const jobs = rest.map((cls) => ({ session, atlas, cls, extra }));
    if (o.first) waiting.unshift(...jobs);
    else waiting.push(...jobs);
    void run();
  });
}

async function run(): Promise<void> {
  if (running) return;
  running = true;
  try {
    for (let j = waiting.shift(); j; j = waiting.shift()) {
      const key = keyOf(j.atlas, j.cls);
      if (!wanted.has(key)) {
        if (classPictures[key]?.state === "drawing") delete classPictures[key];
        continue;
      }
      try {
        const t0 = performance.now();
        console.debug(`class picture: drawing ${key}`);
        classPictures[key] = await drawLive(j);
        console.debug(`class picture: ${key} ${classPictures[key]?.state} in ${Math.round(performance.now() - t0)} ms`);
      } catch (e) {
        classPictures[key] = { state: "none", why: e instanceof Error ? e.message : String(e) };
      }
    }
  } finally {
    running = false;
  }
}

async function drawLive(j: Job): Promise<ClassPicture> {
  const r = await j.session.classSample!([j.cls], j.extra);
  if (!r.ok) return { state: "none", why: r.message };
  const { branch, samples, refused } = r.result;
  try {
    if (refused[0]) return { state: "none", why: refused[0].why };
    const s = samples[0];
    if (!s) return { state: "none", why: "the simulation made no sample world for it" };
    const d = await drawSample(j.session, s, branch, TURN.size, TURN.frames);
    return "why" in d ? { state: "none", why: d.why } : { state: "drawn", sheet: URL.createObjectURL(d.blob), frames: d.frames, size: d.size, caption: d.caption, baked: false };
  } finally {
    await j.session.discardBranch(branch);
  }
}

/** In words, how well the class fits its sample. */
export function fitWords(s: Pick<ClassSample, "fit" | "limiting">): string {
  if (s.fit >= 0.999) return "fully this class";
  const pc = Math.round(s.fit * 100);
  return `${pc} per cent this class${s.limiting ? `, held back by ${s.limiting.why}` : ""}`;
}

// The kernel doors a True face reads, with the sample's world read on its branch. Its weather is the campaign's line
// alone, where a sample world does not exist, so it is drawn without clouds and the caption says so.
function onBranch(session: ViewSession, branch: string): FaceReader {
  const k = session as unknown as Record<string, unknown>;
  return new Proxy(session as unknown as FaceReader, {
    get(_t, p) {
      if (p === "surfaceTiles") return (system: string, body: string) => session.surfaceTiles(system, body, branch);
      if (p === "resolvedSystem") return (system: string) => session.resolvedSystem(system, undefined, branch);
      if (p === "weather") return async () => ({ ok: false as const, message: "a sample world has no weather of its own yet" });
      const v = k[p as string];
      return typeof v === "function" ? (v as (...a: unknown[]) => unknown).bind(session) : v;
    },
  });
}

/**
 * One sample world's turn as a WebP strip, with its caption, or why there is none. Shared by the live gallery and the
 * script that draws the pre-drawn set.
 */
export async function drawSample(session: ViewSession, s: ClassSample, branch: string, size: number, frames: number): Promise<{ blob: Blob; frames: number; size: number; caption: string } | { why: string }> {
  if (!s.surface) return { why: "its sample world has no ground to draw yet" };
  // A body with no size has no tiles to read (the kernel would keep working on them): the mark stands.
  if (!(s.radius_m && s.radius_m > 0)) return { why: `its sample world, after ${s.example}, has no size to draw it at yet` };
  // A body a few kilometres across: the kernel's level-0 tiles for it do not come back within a minute yet (Eros and
  // Bennu, 26 Nov; Psyche at 108 km takes six seconds), and the one kernel worker would be held for the whole gallery.
  if (s.radius_m < SMALLEST_DRAWN_M) return { why: `its sample world, after ${s.example}, is too small for its ground to be read in good time yet` };
  const look = await readTrueFace(onBranch(session, branch), s.system, s.body);
  if (typeof look === "string") return { why: look };
  const turn = await trueTurn(
    { key: `class|${branch}|${s.body}`, star: false, colour: "grey", radius_m: s.radius_m, surface: look.tex, ...(look.air ? { air: look.air } : {}) },
    { size, frames, ...(s.tilt_rad !== null ? { tilt_rad: s.tilt_rad } : {}), spin: s.day_s !== null && s.day_s < 0 ? -1 : 1 },
  );
  if (!turn) return { why: "this device has no GPU to draw True with" };
  // A thick air with no clouds to light it (a sample world has no weather yet) draws its disc dark: no use as a picture.
  if (!turn.lit) return { why: `its sample world, after ${s.example}, draws dark in True without its clouds, which come with its weather` };
  const blob = await new Promise<Blob | null>((r) => turn.sheet.toBlob(r, "image/webp", 0.8));
  if (!blob) return { why: "this browser could not keep the picture" };
  const said = look.said.filter((x) => !/weather/.test(x));
  const caption = `A world made to be this class, after ${s.example} (${fitWords(s)}), in True as the system view draws it, without clouds.${said.length ? ` ${said.join(" ")}` : ""}`;
  return { blob, frames: turn.frames, size: turn.size, caption };
}
