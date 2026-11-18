// A theme's filter chain (H8.4) as drawing instructions: overlays laid over
// the whole screen, plus a style for the screen itself. Everything here is
// CSS that the browser composites on the GPU; nothing reads or changes data.
// Motion (flicker) is left out when the viewer asks for reduced motion, and
// the whole chain is left out when they ask for more contrast or turn
// effects off.
import type { Filter } from "../../chart-room/src/index.ts";

export interface FilterLayer {
  kind: Filter["kind"];
  /** Inline style for an overlay covering the screen; empty for root-only steps. */
  overlay: string;
  /** Inline style added to the screen container. */
  root: string;
  /** True when the overlay animates (it carries the `sse-flicker` animation). */
  moves: boolean;
}

export interface FilterOptions {
  reducedMotion?: boolean;
  /** The viewer turned screen effects off, or asked for more contrast. */
  off?: boolean;
}

const clamp = (x: number) => Math.min(1, Math.max(0, Number.isFinite(x) ? x : 0));
const r3 = (x: number) => Math.round(x * 1000) / 1000;

const GRAIN =
  "url(\"data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='160' height='160'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.9' numOctaves='2' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 0.2 0 0 0 0 0.15 0 0 0 0 0.1 0 0 0 0.9 0'/></filter><rect width='100%' height='100%' filter='url(%23n)'/></svg>\")";

/** Drawing instructions for a theme's chain, in order. Unknown kinds are an error. */
export function filterLayers(filters: readonly Filter[] = [], opts: FilterOptions = {}): FilterLayer[] {
  if (opts.off) return [];
  const out: FilterLayer[] = [];
  for (const f of filters) {
    const s = clamp(f.strength);
    switch (f.kind) {
      case "scanlines": {
        const gap = Math.max(2, Math.round(f.spacing ?? 3));
        out.push({ kind: f.kind, moves: false, root: "", overlay: `background: repeating-linear-gradient(to bottom, rgba(0, 0, 0, ${r3(0.6 * s)}) 0 1px, transparent 1px ${gap}px);` });
        break;
      }
      case "glow":
        out.push({ kind: f.kind, moves: false, overlay: "", root: `filter: drop-shadow(0 0 ${r3(1 + 5 * s)}px color-mix(in srgb, var(--sse-ink) ${Math.round(45 * s)}%, transparent));` });
        break;
      case "grain":
        out.push({ kind: f.kind, moves: false, root: "", overlay: `background-image: ${GRAIN}; background-size: 160px 160px; opacity: ${r3(0.5 * s)}; mix-blend-mode: multiply;` });
        break;
      case "vignette":
        out.push({ kind: f.kind, moves: false, root: "", overlay: `background: radial-gradient(ellipse at center, transparent 55%, rgba(0, 0, 0, ${r3(0.75 * s)}) 100%);` });
        break;
      case "tint":
        if (!/^#[0-9a-f]{6}$/i.test(f.colour)) throw new Error(`tint colour must be #RRGGBB, not ${f.colour}`);
        out.push({ kind: f.kind, moves: false, root: "", overlay: `background: ${f.colour}; opacity: ${r3(0.4 * s)}; mix-blend-mode: color;` });
        break;
      case "flicker":
        if (!opts.reducedMotion) out.push({ kind: f.kind, moves: true, root: "", overlay: `background: rgba(255, 255, 255, ${r3(0.06 * s)}); animation: sse-flicker 0.12s steps(2) infinite;` });
        break;
      default:
        throw new Error(`unknown filter ${(f as { kind: string }).kind}`);
    }
  }
  return out;
}
