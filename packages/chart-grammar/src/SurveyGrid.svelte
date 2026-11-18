<!-- Survey grid with cartographic edge references (A, B, C... across; 1, 2, 3... down). Decorative only. -->
<script lang="ts">
  let { width, height, step = 160, labels = true }: { width: number; height: number; step?: number; labels?: boolean } = $props();
  const xs = $derived(Array.from({ length: Math.max(0, Math.floor(width / step)) }, (_, i) => (i + 1) * step));
  const ys = $derived(Array.from({ length: Math.max(0, Math.floor(height / step)) }, (_, i) => (i + 1) * step));
  const letter = (i: number) => String.fromCharCode(65 + (i % 26));
</script>

<g class="grid" aria-hidden="true">
  {#each xs as x (x)}<line x1={x} y1="0" x2={x} y2={height} />{/each}
  {#each ys as y (y)}<line x1="0" y1={y} x2={width} y2={y} />{/each}
  {#if labels}
    {#each [0, ...xs] as x, i (x)}<text x={x + 6} y={height - 8}>{letter(i)}</text>{/each}
    {#each [0, ...ys] as y, i (y)}<text x={width - 14} y={y + 16}>{i + 1}</text>{/each}
  {/if}
</g>

<style>
  line {
    stroke: var(--sse-grid);
    stroke-width: var(--sse-line);
  }
  text {
    fill: var(--sse-ink-faint);
    font-family: var(--sse-font-mono);
    font-size: 10px;
    letter-spacing: 0.1em;
  }
</style>
