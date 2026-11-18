<!-- A card pinned to the chart: one subject's facts on raised paper with a hard shadow. -->
<script lang="ts">
  import type { Snippet } from "svelte";
  let {
    title,
    subtitle,
    children,
    footer,
    stamps,
  }: { title: string; subtitle?: string; children: Snippet; footer?: Snippet; stamps?: Snippet } = $props();
</script>

<section class="card" aria-label={title}>
  <header>
    <h2>{title}</h2>
    {#if subtitle}<span class="sub">{subtitle}</span>{/if}
  </header>
  {#if stamps}<div class="stamps">{@render stamps()}</div>{/if}
  {@render children()}
  {#if footer}<footer>{@render footer()}</footer>{/if}
</section>

<style>
  .card {
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    box-shadow: var(--sse-card-shadow);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  header {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    padding: 14px 18px 10px;
    border-bottom: var(--sse-line) solid var(--sse-ink);
  }
  h2 {
    margin: 0;
    font-family: var(--sse-font-display);
    font-size: 28px;
    font-weight: 600;
    line-height: 1.1;
  }
  .sub {
    font-size: 12px;
    color: var(--sse-ink-muted);
    text-align: right;
  }
  .stamps {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    padding: 10px 18px 0;
  }
  footer {
    border-top: var(--sse-line) solid var(--sse-ink);
  }
</style>
