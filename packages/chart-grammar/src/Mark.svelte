<!-- Draws one mark as the symbology returned it. Knows shapes and fills, never what they mean. -->
<script lang="ts">
  import type { Mark } from "../../ui-contract/src/index.ts";
  let { mark, cx, cy, prefix }: { mark: Mark; cx: number; cy: number; prefix: string } = $props();
  const r = $derived(mark.size);
  const fill = $derived(
    mark.fill_pattern === "solid"
      ? `var(--sse-${tone(mark.tone)})`
      : mark.fill_pattern === "hollow"
        ? "var(--sse-ground)"
        : `url(#${prefix}-${mark.fill_pattern})`,
  );
  function tone(t: Mark["tone"]): string {
    return t === "accent" ? "accent" : t === "muted" ? "ink-muted" : "ink";
  }
  const stroke = $derived(`var(--sse-${tone(mark.tone)})`);
</script>

<g aria-hidden="true">
  {#if mark.fill_pattern !== "solid" && mark.fill_pattern !== "hollow"}
    <circle {cx} {cy} {r} fill="var(--sse-ground)" />
  {/if}
  {#if mark.shape === "square"}
    <rect x={cx - r} y={cy - r} width={r * 2} height={r * 2} {fill} {stroke} stroke-width="1.4" />
  {:else if mark.shape === "diamond"}
    <path d="M{cx} {cy - r * 1.3}L{cx + r * 1.3} {cy}L{cx} {cy + r * 1.3}L{cx - r * 1.3} {cy}Z" {fill} {stroke} stroke-width="1.4" />
  {:else}
    <circle {cx} {cy} {r} {fill} {stroke} stroke-width={mark.fill_pattern === "solid" ? 0 : 1.4} />
  {/if}
  {#if mark.shape === "ring"}
    <ellipse {cx} {cy} rx={r * 2} ry={r * 0.6} fill="none" {stroke} stroke-width="1.2" />
  {/if}
  {#if mark.shape === "star"}
    <circle {cx} {cy} r={r * 1.9} fill="none" {stroke} stroke-width="1" />
    <path
      d="M{cx} {cy - r * 3.3}V{cy - r * 2.5}M{cx} {cy + r * 2.5}V{cy + r * 3.3}M{cx - r * 3.3} {cy}H{cx - r * 2.5}M{cx + r * 2.5} {cy}H{cx + r * 3.3}"
      {stroke}
      stroke-width="1"
    />
  {/if}
  {#if mark.outline === "double"}
    <circle {cx} {cy} r={r + 3} fill="none" {stroke} stroke-width="0.8" />
  {/if}
</g>
