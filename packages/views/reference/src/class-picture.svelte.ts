// The class gallery's pictures (V14i, V3 gaps item 35): each class drawn from a sample world the kernel makes for it
// (`class_sample`: a mid-band world of the class, hand-set on a what-if branch), its face in True from the one door every
// True view uses (`readTrueFace` and `trueLook`), drawn alone as a still (`trueStill`). A picture is a look at one world that is this
// class, never the class itself, and the caption says which and how well it fits. Nothing is committed: the branch is
// dropped once its worlds are drawn. GM seat only (a branch is the GM's); elsewhere the chart mark stands.
import type { ClassSample, PackDoc, ViewSession } from "../../../ui-contract/src/index.ts";
import { readTrueFace, trueStill, type FaceReader } from "../../../renderer/src/index.ts";

export type ClassPicture = { state: "drawing" } | { state: "drawn"; url: string; caption: string } | { state: "none"; why: string };

/** Pictures by `atlas|class`, filled as they are drawn. */
export const classPictures: Record<string, ClassPicture> = $state({});

const keyOf = (atlas: string, cls: string) => `${atlas}|${cls}`;
export const pictureOf = (atlas: string, cls: string): ClassPicture | undefined => classPictures[keyOf(atlas, cls)];

// One drawing at a time: each draws on its own GPU canvas and reads a world's tiles (a second or more).
let queue: Promise<void> = Promise.resolve();

/**
 * Draw the pictures of these classes that are not drawn or drawing yet: one `class_sample` for them all, then each
 * sample's still in turn. `extra` are the atlas packs shown but not on the stack. `size` is pixels across.
 */
export function drawClassPictures(session: ViewSession, atlas: string, classes: readonly string[], extra: readonly PackDoc[] = [], size = 192): void {
  if (session.seat.authority !== "gm" || !session.classSample) return;
  const ask = classes.filter((c) => !classPictures[keyOf(atlas, c)]);
  if (!ask.length) return;
  for (const c of ask) classPictures[keyOf(atlas, c)] = { state: "drawing" };
  queue = queue.then(() => drawAll(session, atlas, ask, extra, size)).catch((e: unknown) => {
    const why = e instanceof Error ? e.message : String(e);
    for (const c of ask) if (classPictures[keyOf(atlas, c)]?.state === "drawing") classPictures[keyOf(atlas, c)] = { state: "none", why };
  });
}

async function drawAll(session: ViewSession, atlas: string, ask: string[], extra: readonly PackDoc[], size: number): Promise<void> {
  const r = await session.classSample!(ask, extra);
  if (!r.ok) {
    for (const c of ask) classPictures[keyOf(atlas, c)] = { state: "none", why: r.message };
    return;
  }
  const { branch, samples, refused } = r.result;
  for (const x of refused) classPictures[keyOf(atlas, x.class)] = { state: "none", why: x.why };
  try {
    for (const s of samples) classPictures[keyOf(atlas, s.class)] = await drawOne(session, s, branch, size);
  } finally {
    await session.discardBranch(branch);
  }
  for (const c of ask) if (classPictures[keyOf(atlas, c)]?.state === "drawing") classPictures[keyOf(atlas, c)] = { state: "none", why: "the simulation made no sample world for it" };
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

async function drawOne(session: ViewSession, s: ClassSample, branch: string, size: number): Promise<ClassPicture> {
  if (!s.surface) return { state: "none", why: "its sample world has no ground to draw yet" };
  // A body with no size has no tiles to read (the kernel would keep working on them): the mark stands.
  if (!(s.radius_m && s.radius_m > 0)) return { state: "none", why: `its sample world, after ${s.example}, has no size to draw it at yet` };
  // Its True face, from the same reader and door as the system view's and the size comparison's (true-air.ts, true-body.ts).
  const look = await readTrueFace(onBranch(session, branch), s.system, s.body);
  if (typeof look === "string") return { state: "none", why: look };
  const still = await trueStill(
    { key: `class|${branch}|${s.body}`, star: false, colour: "grey", ...(s.radius_m ? { radius_m: s.radius_m } : {}), surface: look.tex, ...(look.air ? { air: look.air } : {}) },
    { size, ...(s.tilt_rad !== null ? { tilt_rad: s.tilt_rad } : {}), frames: 8 },
  );
  if (!still) return { state: "none", why: "this device has no GPU to draw True with" };
  // A thick air with no clouds to light it (a sample world has no weather yet) draws its disc dark: no use as a picture.
  if (!still.lit) return { state: "none", why: `its sample world, after ${s.example}, draws dark in True without its clouds, which come with its weather` };
  const said = look.said.filter((x) => !/weather/.test(x));
  const caption = `A world made to be this class, after ${s.example} (${fitWords(s)}), in True as the system view draws it, without clouds.${said.length ? ` ${said.join(" ")}` : ""}`;
  return { state: "drawn", url: still.canvas.toDataURL("image/png"), caption };
}
