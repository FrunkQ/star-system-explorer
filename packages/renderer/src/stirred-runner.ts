// Where a giant's stirred still is built and kept (track WS, package WS5; design 11A).
//
// One GPU of its own, off screen, made on first use, so building a still never takes the view's renderer from a frame;
// one still is built at a time. The work on the processor (filing the table, composing the still) runs in a worker
// where the page has one. Each finished still is kept in memory only (never in a file, Alex 27 Nov), by world, date
// and size, the oldest let go past a few dozen.

import { WebGPURenderer } from "three/webgpu";
import type { TrueTexture } from "./true-look.ts";
import type { CarryTable } from "./stirred-carry.ts";
import { carryOnGpu } from "./stirred-gpu.ts";
import { packCarry, type PackedCarry } from "./stirred-pack.ts";
import { composeStill, placesFor } from "./stirred-still.ts";
import type { StirAnswer, StirWork } from "./stirred-worker.ts";

const KEPT = 32;
/** How long the carrying's draws may hold the GPU each frame, ms: about half a frame at 60 a second. */
const BUDGET_MS = 8;
/** The most the younger cloud's median shift north or south may be before the carrying is taken as broken, degrees. */
const MOST_SHIFT_DEG = 10;
const kept = new Map<string, TrueTexture>();
let gpu: Promise<WebGPURenderer | null> | null = null;
let queue: Promise<unknown> = Promise.resolve();

/** Whether the renderer's device is drawn in software. */
function softwareDevice(r: WebGPURenderer): boolean {
  const b = r.backend as { gl?: WebGL2RenderingContext; device?: { adapterInfo?: { vendor?: string; architecture?: string; description?: string } } | null };
  let name = "";
  if (b.gl) {
    const ext = b.gl.getExtension("WEBGL_debug_renderer_info");
    name = String(b.gl.getParameter(ext ? ext.UNMASKED_RENDERER_WEBGL : b.gl.RENDERER) ?? "");
  } else if (b.device?.adapterInfo) {
    const i = b.device.adapterInfo;
    name = `${i.vendor ?? ""} ${i.architecture ?? ""} ${i.description ?? ""}`;
  }
  return /swiftshader|llvmpipe|software/i.test(name);
}

function renderer(): Promise<WebGPURenderer | null> {
  gpu ??= (async () => {
    try {
      const canvas = document.createElement("canvas");
      canvas.width = canvas.height = 1;
      const r = new WebGPURenderer({ canvas, antialias: false });
      await r.init();
      // A software device (SwiftShader, llvmpipe: no graphics card, as in CI) takes minutes over a still and holds
      // the page's own drawing up meanwhile; it keeps the unstirred face (the processor's still is WS5b).
      if (softwareDevice(r)) {
        r.dispose();
        return null;
      }
      return r;
    } catch {
      return null;
    }
  })();
  return gpu;
}

// ------------------------------------------------------------------ the worker
let worker: Worker | null | undefined;
let asked = 0;
const waiting = new Map<number, (a: StirAnswer) => void>();
function theWorker(): Worker | null {
  if (worker !== undefined) return worker;
  try {
    worker = new Worker(new URL("./stirred-worker.ts", import.meta.url), { type: "module" });
    worker.onmessage = (ev: MessageEvent<StirAnswer>) => {
      waiting.get(ev.data.id)?.(ev.data);
      waiting.delete(ev.data.id);
    };
    worker.onerror = () => {
      for (const done of waiting.values()) done({ id: -1, error: "the worker failed" });
      waiting.clear();
      worker = null;
    };
  } catch {
    worker = null;
  }
  return worker;
}
/** One piece of processor work, in the worker where there is one, else here. */
async function work(q: StirWork, move: Transferable[] = []): Promise<StirAnswer> {
  const w = theWorker();
  if (!w) {
    if (q.op === "pack") return { id: 0, packed: packCarry(q.table) };
    return { id: 0, still: composeStill(q.face, q.table, q.read) };
  }
  const id = ++asked;
  return new Promise((done) => {
    waiting.set(id, done);
    w.postMessage({ ...q, id }, move);
  });
}

/** A still kept from before, by its key. */
export const keptStill = (key: string): TrueTexture | undefined => kept.get(key);

/** A number to a place, for the console's warning (developers' words, not the GM's). */
const r1 = (x: number) => Math.round(x * 10) / 10;

const frame = () => new Promise<void>((done) => (typeof requestAnimationFrame === "function" ? requestAnimationFrame(() => done()) : setTimeout(done, 0)));

/**
 * Today's stirred face for an unstirred `face` (its winding and curls left out, giant-detail.ts `source`) and the
 * kernel's carry table for the date: built on the GPU, kept by `key`. Null where no GPU could be had or the carrying
 * was refused; the view keeps the face it has.
 */
export function stirFace(key: string, face: TrueTexture, table: CarryTable): Promise<TrueTexture | null> {
  const have = kept.get(key);
  if (have) return Promise.resolve(have);
  const run = queue.then(async () => {
    const again = kept.get(key);
    if (again) return again;
    const r = await renderer();
    if (!r) return null;
    try {
      const filed = await work({ op: "pack", table });
      if (!filed.packed) throw new Error(filed.error ?? "the table could not be filed");
      const packed: PackedCarry = { ...filed.packed, table };
      // A shader three.js fails to build only says so on the console and reads back zeros (AC-ProArt, 28 Nov), so the
      // answer is looked over before it is shown. It cannot be held to the processor's place by place: Jupiter's eddies
      // make the paths chaotic, a nudge of 0.00001° at the start moving where the cloud was made by up to 3.5° of
      // latitude, so float32 and float64 part company and either is as good a picture (WS5.md). What does hold: the
      // younger generation moves only a few pixels north or south (WS5.md, how far cloud moves), so its median shift
      // is under 10°; a shader that read back zeros puts every pixel on the equator, a median of about 45°.
      const places = placesFor(face.width, face.height);
      // A frame between batches of draws, so the view's own frames keep coming while the still is built.
      const read = await carryOnGpu(r, packed, places, face.width, face.height, { budgetMs: BUDGET_MS, each: frame });
      const shifts: number[] = [];
      let broken = 0;
      for (let k = 0; k < face.width * face.height; k += 97) {
        if (![0, 1, 2, 3].every((q) => Number.isFinite(read[k * 4 + q]!)) || Math.abs(read[k * 4]!) > 90 || Math.abs(read[k * 4 + 2]!) > 90) broken++;
        else shifts.push(Math.abs(read[k * 4]! - places[k * 2]!));
      }
      shifts.sort((x, y) => x - y);
      const median = shifts[shifts.length >> 1] ?? 90;
      if (broken > 0 || median > MOST_SHIFT_DEG) throw new Error(`the GPU's carrying looks broken: ${broken} places unreadable, the younger cloud's median shift ${r1(median)}° (at most ${MOST_SHIFT_DEG}°)`);
      const made = await work({ op: "compose", face, table, read }, [read.buffer]);
      if (!made.still) throw new Error(made.error ?? "the still could not be composed");
      kept.set(key, made.still);
      while (kept.size > KEPT) kept.delete(kept.keys().next().value!);
      return made.still;
    } catch (e) {
      console.warn(`stirred still not built: ${String(e)}`);
      return null;
    }
  });
  queue = run.catch(() => null);
  return run;
}
