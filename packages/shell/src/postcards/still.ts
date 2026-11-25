// The front of a postcard (PC3): a still drawn live on this device from the postcard's data, never a kept picture.
// The globe draws one through the renderer's still tier (R3, `photograph`); the system view in True through
// `photographView` (PC2 part 2); other views join as they gain stills, and until then the card says so plainly and
// shows the stamp if it has one.
import type { Globe, Postcard, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
import { COMPOSITE, COMPOSITE_WORD, globeOfPose, VIEW_BAND_WORDS } from "../../../ui-contract/src/index.ts";
import { LIBRARY, photograph, type Photo } from "../../../renderer/src/index.ts";
import { SHIPPED_RECIPES } from "../../../renderer/src/recipes.ts";
import { loadSurface } from "../../../views/body/src/surface.ts";
import { photographView } from "../../../views/system-chart/src/index.ts";

export type Still = { ok: true; photo: Photo; lon: number; lat: number } | { ok: false; why: string };

/** Can this postcard's front be drawn here, and if not, why not (one line)? */
export function stillBlocker(p: Postcard, scene: SystemScene): string | null {
  const b = p.bookmark;
  if (p.world.how !== "here" && b.scope !== scene.system) return "Its world is not open here, so the kept picture is shown.";
  if (b.scope !== scene.system) return "It is a postcard of another system in this campaign: open that system to draw it.";
  if (b.instrument !== "body" && b.instrument !== "system") return "A still of this view comes with the renderer's next step; the kept picture is shown until then.";
  if (!b.pose) return "It has no camera, so it cannot be drawn the same way again.";
  // The system view (PC2): drawn in True from its camera, whatever band (the view draws the bands it can and says so).
  if (b.instrument === "system") return scene.subjects.some((s) => s.address === b.pose!.about) ? null : "The world it shows is not in this system any more.";
  const band = b.view?.band ?? "visible";
  if (band !== "visible" && band !== "mag") return `The globe is drawn in visible light for now, not ${band === COMPOSITE ? COMPOSITE_WORD : VIEW_BAND_WORDS[band]}.`;
  if (!scene.subjects.some((s) => s.address === b.pose!.about)) return "The world it shows is not in this system any more.";
  return null;
}

/** Draw the front: the globe from the postcard's camera, through the still tier, at `size` pixels across. */
export async function drawStill(ctx: ViewContext, scene: SystemScene, p: Postcard, size: number, opts: { frames?: number; onProgress?: (done: number, of: number) => void } = {}): Promise<Still> {
  const why = stillBlocker(p, scene);
  if (why) return { ok: false, why };
  const pose = p.bookmark.pose!;
  const sub = scene.subjects.find((s) => s.address === pose.about)!;
  if (p.bookmark.instrument === "system") {
    // The same square front as the globe's.
    const shot = await photographView({ ctx, scene, pose, ...(p.bookmark.view ? { view: p.bookmark.view } : {}), width: size, height: size, ...(opts.onProgress ? { onProgress: () => opts.onProgress!(0, 1) } : {}) });
    opts.onProgress?.(1, 1);
    const photo: Photo = { canvas: shot.canvas, tier: "still", backend: shot.backend, recipe: "system-true", frames: 0, ms: shot.ms, note: shot.note };
    const { lon, lat } = globeOfPose(pose);
    return { ok: true, photo, lon, lat };
  }
  if (sub.kind === "star") return { ok: false, why: "A star's still comes with the renderer's next step." };
  const surf = await loadSurface(ctx, scene.system, pose.about);
  if (typeof surf === "string") return { ok: false, why: surf };
  const globe: Globe = [0, 1, 2, 3, 4, 5].map((f) => surf.tiles.find((t) => t.key.face === f && t.key.level === 0));
  if (!globe.some((t) => t)) return { ok: false, why: "The kernel gave no surface for this world." };
  const { lon, lat } = globeOfPose(pose);
  const recipe = p.bookmark.view?.style === "true" ? "true-colour" : "survey-relief";
  const photo = await photograph({ registry: SHIPPED_RECIPES, library: LIBRARY, recipe, knobs: { relief: 0.5, detail: 0.5 }, globe, air: surf.air, label: sub.label, lon, lat, size, frames: opts.frames ?? 16, ...(opts.onProgress ? { onProgress: opts.onProgress } : {}) });
  return { ok: true, photo, lon, lat };
}
