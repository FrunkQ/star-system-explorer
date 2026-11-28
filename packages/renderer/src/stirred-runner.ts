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
/** Steps each draw takes: a few, so a 1024-wide face's draw stays well inside a frame on a laptop's own graphics. */
const STEPS_A_DRAW = 4;
const kept = new Map<string, TrueTexture>();
let gpu: Promise<WebGPURenderer | null> | null = null;
let queue: Promise<unknown> = Promise.resolve();

function renderer(): Promise<WebGPURenderer | null> {
  gpu ??= (async () => {
    try {
      const canvas = document.createElement("canvas");
      canvas.width = canvas.height = 1;
      const r = new WebGPURenderer({ canvas, antialias: false });
      await r.init();
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
      // A frame between draws, so the view's own frames keep coming while the still is built.
      const read = await carryOnGpu(r, packed, placesFor(face.width, face.height), face.width, face.height, { chunk: STEPS_A_DRAW, wait: 1, each: frame });
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
