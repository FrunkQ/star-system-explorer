<!--
  Draws one mark as the symbology returned it (H8.4A). Knows shapes, fills,
  colours, rings and glyphs, never what they mean. A colour comes from the
  symbology (already fitted to the theme); everything else is theme ink.
-->
<script lang="ts">
  import type { Mark } from "../../ui-contract/src/index.ts";
  let { mark, cx, cy, prefix }: { mark: Mark; cx: number; cy: number; prefix: string } = $props();
  const r = $derived(mark.size);
  const ink = $derived(`var(--sse-${mark.tone === "accent" ? "accent" : mark.tone === "muted" ? "ink-muted" : "ink"})`);
  const patterned = $derived(mark.fill_pattern !== "solid" && mark.fill_pattern !== "hollow");
  // Under a pattern or a hollow mark: the colour if there is one, else the ground.
  const under = $derived(mark.colour ?? (mark.fill_pattern === "solid" ? ink : "var(--sse-chart-ground)"));
  const d = $derived.by(() => {
    if (mark.shape === "square") return `M${cx - r} ${cy - r}h${r * 2}v${r * 2}h${-r * 2}Z`;
    if (mark.shape === "diamond") return `M${cx} ${cy - r * 1.3}L${cx + r * 1.3} ${cy}L${cx} ${cy + r * 1.3}L${cx - r * 1.3} ${cy}Z`;
    if (mark.shape === "triangle") return `M${cx} ${cy - r * 1.3}L${cx + r * 1.2} ${cy + r * 0.8}L${cx - r * 1.2} ${cy + r * 0.8}Z`;
    return `M${cx - r} ${cy}a${r} ${r} 0 1 0 ${r * 2} 0a${r} ${r} 0 1 0 ${-r * 2} 0Z`;
  });
  const edge = $derived(mark.fill_pattern === "solid" && !mark.colour ? 0 : 1.3);
</script>

<g aria-hidden="true" data-mark="{mark.shape} {mark.fill_pattern}{mark.colour ? ` ${mark.colour}` : ''}{mark.ring ? ' ring' : ''}">
  <path {d} fill={under} stroke={ink} stroke-width={edge} />
  {#if patterned}<path {d} fill="url(#{prefix}-m-{mark.fill_pattern})" />{/if}
  {#if mark.ring}
    <ellipse {cx} {cy} rx={r * 2} ry={r * 0.6} fill="none" stroke={ink} stroke-width="1.2" />
  {/if}
  {#if mark.shape === "star"}
    <circle {cx} {cy} r={r * 1.9} fill="none" stroke={ink} stroke-width="1" />
    <path
      d="M{cx} {cy - r * 3.3}V{cy - r * 2.5}M{cx} {cy + r * 2.5}V{cy + r * 3.3}M{cx - r * 3.3} {cy}H{cx - r * 2.5}M{cx + r * 2.5} {cy}H{cx + r * 3.3}"
      stroke={ink}
      stroke-width="1"
    />
  {/if}
  {#if mark.outline === "double"}
    <circle {cx} {cy} r={r + 3} fill="none" stroke={ink} stroke-width="0.8" />
  {/if}
  {#if mark.glyph}<text x={cx + r + 3} y={cy + r + 9} class="glyph" fill={ink}>{mark.glyph}</text>{/if}
</g>

<style>
  .glyph {
    font-family: var(--sse-font-mono);
    font-size: 9px;
  }
</style>
