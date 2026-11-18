<!--
  Pattern definitions for hatching, stipple, bands, cross-hatch, grid and
  dashes. Ids are prefixed so several charts can share a page. Each pattern
  comes twice: a light one for areas ("{prefix}-hatch") and a fine, full-ink
  one for marks ("{prefix}-m-hatch"), so a pattern still reads on a small body.
-->
<script lang="ts">
  let { prefix }: { prefix: string } = $props();
  const SETS = [
    { id: "", s: 1, cls: "" },
    { id: "m-", s: 0.6, cls: "mk" },
  ];
</script>

<defs>
  {#each SETS as v (v.id)}
    <pattern id="{prefix}-{v.id}hatch" width={6 * v.s} height={6 * v.s} patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
      <line x1="0" y1="0" x2="0" y2={6 * v.s} class="p {v.cls}" />
    </pattern>
    <pattern id="{prefix}-{v.id}cross" width={7 * v.s} height={7 * v.s} patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
      <path d="M0 0V{7 * v.s}M0 0H{7 * v.s}" class="p {v.cls}" />
    </pattern>
    <pattern id="{prefix}-{v.id}stipple" width={4 * v.s} height={4 * v.s} patternUnits="userSpaceOnUse">
      <circle cx={1 * v.s} cy={1 * v.s} r={v.s === 1 ? 0.7 : 0.75} class="dot {v.cls}" />
      <circle cx={3 * v.s} cy={3 * v.s} r={v.s === 1 ? 0.7 : 0.75} class="dot {v.cls}" />
    </pattern>
    <pattern id="{prefix}-{v.id}band" width={4 * v.s} height={4 * v.s} patternUnits="userSpaceOnUse">
      <line x1="0" y1={1 * v.s} x2={4 * v.s} y2={1 * v.s} class="p {v.cls}" />
    </pattern>
    <pattern id="{prefix}-{v.id}grid" width={4 * v.s} height={4 * v.s} patternUnits="userSpaceOnUse">
      <path d="M0 {0.5 * v.s}H{4 * v.s}M{0.5 * v.s} 0V{4 * v.s}" class="p {v.cls}" />
    </pattern>
    <pattern id="{prefix}-{v.id}dash" width={6 * v.s} height={4 * v.s} patternUnits="userSpaceOnUse">
      <line x1="0" y1={1 * v.s} x2={3 * v.s} y2={1 * v.s} class="p {v.cls}" />
      <line x1={3 * v.s} y1={3 * v.s} x2={6 * v.s} y2={3 * v.s} class="p {v.cls}" />
    </pattern>
  {/each}
</defs>

<style>
  .p {
    stroke: var(--sse-hatch);
    stroke-width: 1;
    stroke-opacity: 0.55;
  }
  .dot {
    fill: var(--sse-hatch);
    fill-opacity: 0.7;
  }
  .p.mk {
    stroke: var(--sse-ink);
    stroke-width: 0.9;
    stroke-opacity: 1;
  }
  .dot.mk {
    fill: var(--sse-ink);
    fill-opacity: 1;
  }
</style>
