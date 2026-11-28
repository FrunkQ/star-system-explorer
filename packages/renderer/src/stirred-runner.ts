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
import { composeStill, placesFor, twinOf } from "./stirred-still.ts";
import type { StirAnswer, StirWork } from "./stirred-worker.ts";

const KEPT = 32;
/** How long the carrying's draws may hold the GPU each frame, ms: about half a frame at 60 a second. */
const BUDGET_MS = 8;
/** Latitudes (degrees) of the places checked against the processor, one in each kind of band, longitudes spread. */
const CHECKED = [-62, -27, -6, 9, 33, 64];
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
    if (q.op === "twin") return { id: 0, twin: twinOf(q.table, q.places) };
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

/** A number to so many places, for the console's warning (developers' words, not the GM's). */
const r2 = (x: number) => Math.round(x * 100) / 100;
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
      // A few places carried by the processor too, alongside: a shader three.js fails to build only says so on the
      // console and reads back zeros, so the GPU's answer is held to the processor's before it is shown (AC-ProArt,
      // 28 Nov). Only the younger generation's latitude is held, to a pixel of the face, and at most two places may
      // miss. The older generation cannot be held at all: Jupiter's eddies make its path chaotic, a nudge of 0.00001°
      // at the start moving it by up to 3.5° of latitude (WS5.md), so float32 and float64 part company there and
      // either is as good a picture. Along the bands both are read where they drifted for their age, so a float's last
      // digit of latitude times the jets' shear moves them a long way east or west. A broken shader misses everywhere.
      const places = placesFor(face.width, face.height);
      const at = CHECKED.map((lat, n) => {
        const j = Math.min(face.height - 1, Math.max(0, Math.floor(((lat + 90) / 180) * face.height)));
        const i = Math.floor(((n * 0.37 + 0.11) % 1) * face.width);
        return j * face.width + i;
      });
      const sample = new Float32Array(at.flatMap((k) => [places[k * 2]!, places[k * 2 + 1]!]));
      const twin = work({ op: "twin", table, places: sample });
      // A frame between batches of draws, so the view's own frames keep coming while the still is built.
      const read = await carryOnGpu(r, packed, places, face.width, face.height, { budgetMs: BUDGET_MS, each: frame });
      const cpu = (await twin).twin;
      if (!cpu) throw new Error("the processor's check could not be made");
      const pixelDeg = 180 / face.height;
      const misses = at.flatMap((k, n) => {
        const [la, lo, cla, clo] = [read[k * 4]!, read[k * 4 + 1]!, cpu[n * 4]!, cpu[n * 4 + 1]!];
        const sane = [0, 1, 2, 3].every((q) => Number.isFinite(read[k * 4 + q]!)) && Math.abs(read[k * 4 + 2]!) <= 90;
        return sane && Math.abs(la - cla) <= pixelDeg ? [] : [`the GPU read ${r2(la)}°, ${r2(lo)}° where the processor reads ${r2(cla)}°, ${r2(clo)}° (place ${r1(sample[n * 2]!)}°, ${r1(sample[n * 2 + 1]!)}°)`];
      });
      if (misses.length > 2) throw new Error(`${misses.length} of ${at.length} places checked disagree: ${misses.join("; ")}`);
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
