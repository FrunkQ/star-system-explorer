// Where a giant's stirred still is built and kept (track WS, package WS5; design 11A).
//
// One GPU of its own, off screen, made on first use, so building a still never takes the view's renderer from a frame;
// one still is built at a time. Each finished still is kept in memory only (never in a file, Alex 27 Nov), by world,
// date and size, the oldest let go past a few dozen.

import { WebGPURenderer } from "three/webgpu";
import type { TrueTexture } from "./true-look.ts";
import type { CarryTable } from "./stirred-carry.ts";
import { carryOnGpu, packCarry } from "./stirred-gpu.ts";
import { composeStill, placesFor } from "./stirred-still.ts";

const KEPT = 32;
const kept = new Map<string, TrueTexture>();
let gpu: Promise<WebGPURenderer | null> | null = null;
let queue: Promise<unknown> = Promise.resolve();

/** Whether this page can build a still on a GPU (a browser with a canvas; WebGPU, else WebGL2). */
export function canStir(): boolean {
  return typeof document !== "undefined" && typeof document.createElement === "function";
}

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

/** A still kept from before, by its key. */
export const keptStill = (key: string): TrueTexture | undefined => kept.get(key);

/**
 * Today's stirred face for an unstirred `face` (its winding and curls left out, giant-detail.ts `source`) and the
 * kernel's carry table for the date: built on the GPU, kept by `key`. Null where no GPU could be had or the carrying
 * failed; the view keeps the face it has.
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
      // A frame's wait every eight steps, so the view's own frames keep coming while the still is built.
      let n = 0;
      const frame = () => new Promise<void>((done) => (typeof requestAnimationFrame === "function" ? requestAnimationFrame(() => done()) : setTimeout(done, 0)));
      const read = await carryOnGpu(r, packCarry(table), placesFor(face.width, face.height), face.width, face.height, {
        wait: 4,
        each: () => (++n % 8 === 0 ? frame() : Promise.resolve()),
      });
      const still = composeStill(face, table, read);
      kept.set(key, still);
      while (kept.size > KEPT) kept.delete(kept.keys().next().value!);
      return still;
    } catch {
      return null;
    }
  });
  queue = run.catch(() => null);
  return run;
}
