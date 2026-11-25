<!--
  A mark in pencil or ink (SV3), for any SVG chart: wrap a mark (a Mark, a dot, a glyph) and give its pen and where
  it sits. Pencil draws the mark faint inside a dotted ring; heard of draws it as an outline inside a thin solid ring;
  ink draws it as it is. A pencilled mark says so to a screen reader. The pen comes from `penOf(landmark)`.
-->
<script lang="ts">
  import type { Snippet } from "svelte";
  import { PEN_WORDS, penRing, penStyle, type Pen } from "./pencil.ts";

  let { pen, cx, cy, r, children }: { pen: Pen; cx: number; cy: number; r: number; children: Snippet } = $props();
  const ring = $derived(penRing(pen));
</script>

<g data-pen={pen} style={penStyle(pen, "mark") || undefined}>
  {@render children()}
  {#if PEN_WORDS[pen]}<title>{PEN_WORDS[pen]}</title>{/if}
</g>
{#if ring}
  <circle {cx} {cy} r={r + 3} fill="none" stroke="var(--sse-ink-muted)" stroke-width={ring.width} stroke-dasharray={ring.dash ?? undefined} pointer-events="none" aria-hidden="true" data-pen-ring={pen} />
{/if}
