// PC2 part 2: a still of the system view in True from a postcard's camera (postcards design, accepted 25 Nov). The
// same view the GM uses is drawn off screen with a private workspace (the GM's selection, camera and view are never
// touched): the pose's world chosen and at the centre, the camera put where the pose says, HUD off, the pose's
// exposure. Its worlds' surfaces are read as the live view reads them (and kept for it). The picture is the frame the
// finish calls sharp (its running mean of jittered frames, VP4/VP5), or, where the view never holds still (a living
// star), a few frames averaged here. Presentation only: a still never touches the kernel's state or the state hash.
import type { Pose, PostcardView, SystemScene, ViewContext } from "../../../ui-contract/src/index.ts";
import { DEFAULT_VIEW } from "../../../ui-contract/src/index.ts";
import { systemChart } from "./index.svelte.ts";

export interface ViewPhoto {
  canvas: HTMLCanvasElement;
  backend: "webgpu" | "webgl2";
  /** Sharp: the finish's own mean of jittered frames was done; otherwise frames were averaged here. */
  sharp: boolean;
  ms: number;
  /** How it was made, in words, for the caption. */
  note: string;
}

export interface ViewPhotoOptions {
  ctx: ViewContext;
  scene: SystemScene;
  pose: Pose;
  /** The view it was made in (band, scale); style, HUD and "Seen from" are set for a photograph. */
  view?: PostcardView;
  /** Pixels across and down. */
  width: number;
  height: number;
  /** Longest to wait for the world's surface and a sharp frame, ms (default 30 s); the last frame is kept then. */
  budget_ms?: number;
  /** Once frames have come, how long without a new one means the view is done (the finish stopped sharpening), ms. */
  quiet_ms?: number;
  /** Most frames averaged when the view never holds still long enough to call a frame sharp (default 8). */
  frames?: number;
  onProgress?: (words: string) => void;
}

/** Draw the system view in True from `pose`, off screen, and keep its sharp frame. */
export async function photographView(o: ViewPhotoOptions): Promise<ViewPhoto> {
  const t0 = performance.now();
  const budget = o.budget_ms ?? 30000;
  const quiet = o.quiet_ms ?? 1200;
  const host = document.createElement("div");
  host.setAttribute("aria-hidden", "true");
  host.dataset.stillOf = o.pose.about;
  // Off the page but laid out and painted, so the view sizes itself and its frames run.
  host.style.cssText = `position:fixed;left:-${o.width + 100}px;top:0;width:${o.width}px;height:${o.height}px;overflow:hidden;pointer-events:none;`;
  document.body.append(host);
  const view = { ...DEFAULT_VIEW, ...(o.view ?? {}), style: "true" as const, seen: "free" as const, hud: false, exposure: o.pose.exposure.mode };
  const ws = { ...o.ctx.workspace, selection: o.pose.about, view, aim: o.pose, camera: null, select: () => {}, turned: () => {} };
  const ctx: ViewContext = { ...o.ctx, workspace: ws as ViewContext["workspace"] };
  const out = document.createElement("canvas");
  out.width = o.width;
  out.height = o.height;
  const g = out.getContext("2d")!;
  // Frames are kept once the view has taken the pose's camera and the pose's world has been read. While the
  // view sharpens, each frame on its canvas is the finish's running mean of jittered frames so far (VP4/VP5), so the
  // latest is the best: it is kept, and the picture is done when the finish says sharp or the frames stop (its time
  // ran out). Where the view never holds still (a living star turning on its own clock), up to `frames` frames are
  // averaged here instead.
  const read = () => host.querySelector("[data-true-read]")?.getAttribute("data-true-read") ?? "";
  // The pose's world is read first; it is the one the picture is about. A seat that reads no surfaces (a visit, a
  // player) has nothing to wait for.
  const worldRead = () => {
    const r = read();
    if (r === "") return true;
    const [done, of] = r.split("/").map(Number);
    return (done ?? 0) >= 1 || of === 0;
  };
  const allRead = () => {
    const r = read();
    if (r === "") return true;
    const [done, of] = r.split("/").map(Number);
    return done === of;
  };
  let taking = false;
  let k = 0;
  let moving = 0;
  let sharp = false;
  let last = 0;
  let backend: "webgpu" | "webgl2" = "webgl2";
  const most = Math.max(1, Math.round(o.frames ?? 8));
  const onFrame = (e: Event) => {
    const d = (e as CustomEvent<{ canvas: HTMLCanvasElement; rest: string; backend: "webgpu" | "webgl2" }>).detail;
    if (!d?.canvas) return;
    // Taking once the camera is where the pose says (the view marks the aim taken) and its world is read.
    if (!taking && host.querySelector("[data-aimed]")?.getAttribute("data-aimed") === o.pose.about && worldRead()) taking = true;
    if (!taking) return;
    backend = d.backend;
    last = performance.now();
    k++;
    // Same task as the draw: the canvas still holds the frame.
    if (d.rest === "sharpening" || d.rest === "sharp") {
      moving = 0;
      g.globalAlpha = 1;
      g.drawImage(d.canvas, 0, 0, o.width, o.height);
      sharp = d.rest === "sharp";
      return;
    }
    sharp = false;
    // Waiting to sharpen: a plain frame, the picture so far. Moving (it never holds still): averaged here.
    g.globalAlpha = d.rest === "moving" ? 1 / (moving + 1) : 1;
    g.drawImage(d.canvas, 0, 0, o.width, o.height);
    moving = d.rest === "moving" ? moving + 1 : 0;
  };
  host.addEventListener("sse-frame", onFrame);
  const mounted = systemChart.mount(host, ctx, o.scene);
  try {
    for (;;) {
      await new Promise((r) => setTimeout(r, 50));
      const now = performance.now();
      // Done: sharp with every world read (the far ones change the picture a little too), or quiet since the last
      // frame (the finish ran out of time to sharpen), or enough frames of a view that never holds still.
      if (now - t0 > budget || moving >= most || (allRead() && (sharp || (k > 0 && now - last >= quiet)))) break;
      o.onProgress?.(!worldRead() ? "Reading the world's surface…" : taking ? "Taking the picture…" : "Framing…");
    }
  } finally {
    host.removeEventListener("sse-frame", onFrame);
    mounted.dispose();
    host.remove();
  }
  g.globalAlpha = 1;
  const kept = k > 0;
  if (!kept) throw new Error("the view drew no frame in True on this device");
  const ms = performance.now() - t0;
  const secs = Math.round(ms / 100) / 10;
  const how = moving > 0 ? `${moving} frame${moving === 1 ? "" : "s"} averaged (the view does not hold still)` : sharp ? "held still until the finish had sharpened it" : "held still while the finish sharpened it as far as this device had time for";
  const note = `The system view in True from the postcard's camera, drawn off screen at ${o.width} by ${o.height} px, ${how} (${secs} s).`;
  return { canvas: out, backend, sharp, ms, note };
}
