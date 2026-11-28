// The stirred still's work on the processor, off the page's own thread (track WS, package WS5): filing the carry
// table for the GPU (Saturn's takes most of a second) and composing the still from where its pixels are read (a
// 1024-wide face, about a second). On the page's thread either held the tab still for seconds (AC-ProArt, 28 Nov).

import type { CarryTable } from "./stirred-carry.ts";
import { packCarry, type PackedCarry } from "./stirred-pack.ts";
import { composeStill, twinOf } from "./stirred-still.ts";
import type { TrueTexture } from "./true-look.ts";

export type StirWork = { op: "pack"; table: CarryTable } | { op: "twin"; table: CarryTable; places: Float32Array } | { op: "compose"; face: TrueTexture; table: CarryTable; read: Float32Array };
export type StirAsk = StirWork & { id: number };
export type StirAnswer = { id: number; packed?: Omit<PackedCarry, "table">; twin?: Float32Array; still?: TrueTexture; error?: string };


const post = (a: StirAnswer, move: Transferable[]) => (self as unknown as { postMessage(m: unknown, t: Transferable[]): void }).postMessage(a, move);

self.onmessage = (ev: MessageEvent<StirAsk>) => {
  const q = ev.data;
  try {
    if (q.op === "pack") {
      const { table: _t, ...packed } = packCarry(q.table);
      post({ id: q.id, packed }, [packed.u, packed.psiN, packed.psiS, packed.storms, packed.eddies, packed.grid, packed.rates, packed.list, packed.ages].map((x) => x.buffer));
    } else if (q.op === "twin") {
      const twin = twinOf(q.table, q.places);
      post({ id: q.id, twin }, [twin.buffer]);
    } else {
      const still = composeStill(q.face, q.table, q.read);
      post({ id: q.id, still }, [still.data.buffer]);
    }
  } catch (e) {
    post({ id: q.id, error: String(e) }, []);
  }
};
