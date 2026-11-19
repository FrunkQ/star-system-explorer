<!-- A stamped evidence label, set slightly off square like a rubber stamp. -->
<script lang="ts">
  import { HONESTY, stampLabel } from "./stamps.ts";
  let { kind, tilt = true, title }: { kind: string; tilt?: boolean; title?: string } = $props();
</script>

<span class="stamp" class:honesty={HONESTY.has(kind)} class:tilt {title}>{stampLabel(kind)}</span>

<style>
  .stamp {
    display: inline-block;
    border: 1.5px solid var(--sse-ink-muted);
    color: var(--sse-ink-muted);
    font-family: var(--sse-font-body);
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 0.14em;
    line-height: 1.5;
    padding: 0 6px;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .honesty {
    border-color: var(--sse-accent-ink);
    color: var(--sse-accent-ink);
  }
  .tilt {
    transform: rotate(-3deg);
  }
</style>
