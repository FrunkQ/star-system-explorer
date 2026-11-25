// The front of a postcard (PC3): a still drawn live on this device from the postcard's data, never a kept picture.
// The globe draws one today through the renderer's still tier (R3, `photograph`); other views join as the renderer's
// still of any True view arrives (PC2), and until then the card says so plainly and shows the stamp if it has one.
import type { Globe, Postcard, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
import { COMPOSITE, COMPOSITE_WORD, globeOfPose, VIEW_BAND_WORDS } from "../../../ui-contract/src/index.ts";
import { LIBRARY, photograph, type Photo } from "../../../renderer/src/index.ts";
import { SHIPPED_RECIPES } from "../../../renderer/src/recipes.ts";
import { loadSurface } from "../../../views/body/src/surface.ts";
import { layVisit } from "./visiting.ts";

export type Still = { ok: true; photo: Photo; lon: number; lat: number } | { ok: false; why: string };

/** What in the look itself keeps the front from being drawn (the view, the camera, the band); null when it can be. */
function lookBlocker(p: Postcard): string | null {
  const b = p.bookmark;
  if (b.instrument !== "body") return "A still of this view comes with the renderer's next step; the kept picture is shown until then.";
  if (!b.pose) return "It has no camera, so it cannot be drawn the same way again.";
  const band = b.view?.band ?? "visible";
  if (band !== "visible" && band !== "mag") return `The globe is drawn in visible light for now, not ${band === COMPOSITE ? COMPOSITE_WORD : VIEW_BAND_WORDS[band]}.`;
  return null;
}

/** Is this a postcard from elsewhere whose world travels with it, drawn in a visiting universe (PC4 part 2)? */
export const drawnInVisit = (p: Postcard, scene: SystemScene) => p.world.how === "clip" && p.bookmark.scope !== scene.system;

/** Can this postcard's front be drawn here, and if not, why not (one line)? */
export function stillBlocker(p: Postcard, scene: SystemScene): string | null {
  const b = p.bookmark;
  if (drawnInVisit(p, scene)) return lookBlocker(p);
  if (p.world.how === "address" && b.scope !== scene.system) return "A place from another universe is drawn once visits can open one; the kept picture is shown until then.";
  if (b.scope !== scene.system) return "It is a postcard of another system in this campaign: open that system to draw it.";
  const look = lookBlocker(p);
  if (look) return look;
  if (!scene.subjects.some((s) => s.address === b.pose!.about)) return "The world it shows is not in this system any more.";
  return null;
}

/** Draw the front: the globe from the postcard's camera, through the still tier, at `size` pixels across. A postcard
 * from elsewhere is first laid into the visiting universe and drawn from there. */
export async function drawStill(ctx: ViewContext, scene: SystemScene, p: Postcard, size: number, opts: { frames?: number; onProgress?: (done: number, of: number) => void } = {}): Promise<Still> {
  const why = stillBlocker(p, scene);
  if (why) return { ok: false, why };
  const pose = p.bookmark.pose!;
  let where: { ctx: ViewContext; system: string; about: string; kind: string; label: string };
  if (drawnInVisit(p, scene)) {
    const v = await layVisit(p, ctx.session.stack().base);
    if (typeof v === "string") return { ok: false, why: v };
    where = { ctx: { ...ctx, session: v.session }, system: v.system, about: v.about, kind: v.kind, label: v.label };
  } else {
    const sub = scene.subjects.find((s) => s.address === pose.about)!;
    where = { ctx, system: scene.system, about: pose.about, kind: sub.kind, label: sub.label };
  }
  if (where.kind === "star") return { ok: false, why: "A star's still comes with the renderer's next step." };
  const surf = await loadSurface(where.ctx, where.system, where.about);
  if (typeof surf === "string") return { ok: false, why: surf };
  const globe: Globe = [0, 1, 2, 3, 4, 5].map((f) => surf.tiles.find((t) => t.key.face === f && t.key.level === 0));
  if (!globe.some((t) => t)) return { ok: false, why: "The kernel gave no surface for this world." };
  const { lon, lat } = globeOfPose(pose);
  const recipe = p.bookmark.view?.style === "true" ? "true-colour" : "survey-relief";
  const photo = await photograph({ registry: SHIPPED_RECIPES, library: LIBRARY, recipe, knobs: { relief: 0.5, detail: 0.5 }, globe, air: surf.air, label: where.label, lon, lat, size, frames: opts.frames ?? 16, ...(opts.onProgress ? { onProgress: opts.onProgress } : {}) });
  return { ok: true, photo, lon, lat };
}
