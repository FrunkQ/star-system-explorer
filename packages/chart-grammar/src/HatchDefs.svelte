<!-- Pattern definitions for hatching, stipple, bands and cross-hatch. Ids are prefixed so several charts can share a page. -->
<script lang="ts">
  let { prefix }: { prefix: string } = $props();
</script>

<defs>
  <pattern id="{prefix}-hatch" width="6" height="6" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
    <line x1="0" y1="0" x2="0" y2="6" class="p" />
  </pattern>
  <pattern id="{prefix}-cross" width="7" height="7" patternUnits="userSpaceOnUse" patternTransform="rotate(45)">
    <path d="M0 0V7M0 0H7" class="p" />
  </pattern>
  <pattern id="{prefix}-stipple" width="4" height="4" patternUnits="userSpaceOnUse">
    <circle cx="1" cy="1" r="0.7" class="dot" />
    <circle cx="3" cy="3" r="0.7" class="dot" />
  </pattern>
  <pattern id="{prefix}-band" width="4" height="4" patternUnits="userSpaceOnUse">
    <line x1="0" y1="1" x2="4" y2="1" class="p" />
  </pattern>
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
</style>
