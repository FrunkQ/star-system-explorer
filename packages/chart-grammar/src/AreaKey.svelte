<!--
  The key of a map layer (territory OV4, design 6): one row per answer in view, with its colour (and its pattern where
  colour cannot carry it), its share of what is in view, and how it stands there (holds, claims, influences). Pointing
  at a row lifts that territory on the map and quietens the rest (bind `lift`); pressing a row calls `onpick` with
  the answer's key (the host opens the actor's card). An answer the table does not know is in pencil. On a phone the
  host shows it in a bottom sheet; with `compact` it collapses to colour chips. Set `floating` when it sits over a
  chart, so the chart keeps names clear of it.
-->
<script lang="ts">
  import { areaShares, mapSlotToken, type AreaItem, type AreaStatus, type OverlayInk } from "../../ui-contract/src/index.ts";

  let {
    items,
    nameOf = (k: string) => k,
    lift = $bindable(null),
    onpick,
    compact = false,
    floating = false,
    title = "Key",
  }: {
    items: readonly AreaItem[];
    nameOf?: (key: string) => string;
    lift?: string | null;
    onpick?: (key: string) => void;
    compact?: boolean;
    floating?: boolean;
    title?: string;
  } = $props();

  const colourOf = (k: OverlayInk) => (typeof k === "string" ? (k === "accent" ? "var(--sse-accent)" : "var(--sse-ink)") : `var(${mapSlotToken(k.slot)}, var(--sse-ink))`);
  const rows = $derived.by(() => {
    const shares = new Map(areaShares(items).map((s) => [s.key, s.share]));
    const keys = [...new Set(items.map((i) => i.key))];
    return keys
      .map((key) => {
        const mine = items.filter((i) => i.key === key);
        const how = (["holds", "claims", "influences"] as AreaStatus[]).filter((s) => mine.some((i) => i.status === s));
        const ink = mine[0]!.ink;
        return { key, ink, share: shares.get(key) ?? 0, how, pencil: mine.every((i) => i.pencil), pattern: typeof ink === "string" ? undefined : ink.pattern };
      })
      .sort((a, b) => b.share - a.share || (a.key < b.key ? -1 : 1));
  });
  const pct = (x: number) => (x <= 0 ? "" : x < 0.01 ? "<1%" : `${Math.round(x * 100)}%`);
  const HOW: Record<AreaStatus, string> = { holds: "holds", claims: "claims", influences: "influence" };
</script>

<div class="areakey" class:compact role="group" aria-label={title} data-area-key={rows.length} data-chart-overlay={floating || undefined}>
  {#if !compact}<span class="k">{title}</span>{/if}
  <ul>
    {#each rows as r (r.key)}
      <li class:pencil={r.pencil} class:lifted={lift === r.key}>
        <button
          type="button"
          data-area-row={r.key}
          aria-label="{nameOf(r.key)}: {r.how.map((h) => HOW[h]).join(", ")}{r.share ? `, ${pct(r.share)} of what is held in view` : ''}{r.pencil ? ', not known to the table' : ''}"
          onpointerenter={() => (lift = r.key)}
          onpointerleave={() => (lift = null)}
          onfocus={() => (lift = r.key)}
          onblur={() => (lift = null)}
          onclick={() => onpick?.(r.key)}
        >
          <svg class="chip" width="22" height="14" viewBox="0 0 22 14" aria-hidden="true">
            <rect x="1" y="1" width="20" height="12" rx="2" style:fill={r.pencil ? "none" : colourOf(r.ink)} style:fill-opacity={r.pencil ? 0 : 0.28} style:stroke={r.pencil ? "var(--sse-ink-muted)" : colourOf(r.ink)} stroke-dasharray={r.pencil ? "1.5 2" : r.how[0] === "claims" ? "4 2" : r.how[0] === "influences" ? "1 2" : undefined} />
            {#if r.pattern && r.pattern !== "plain" && !r.pencil}<text x="11" y="10.5" text-anchor="middle" class="pat" style:fill={colourOf(r.ink)}>{r.pattern.slice(0, 2)}</text>{/if}
          </svg>
          {#if !compact}
            <span class="name">{nameOf(r.key)}</span>
            <span class="how">{r.how.map((h) => HOW[h]).join(" · ")}</span>
            <span class="share">{pct(r.share)}</span>
          {/if}
        </button>
      </li>
    {/each}
  </ul>
</div>

<style>
  .areakey {
    font-size: 13px;
    color: var(--sse-ink);
  }
  .k {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  ul {
    list-style: none;
    margin: 4px 0 0;
    padding: 0;
    display: grid;
    gap: 2px;
  }
  .compact ul {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  button {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr) auto auto;
    gap: 8px;
    align-items: center;
    width: 100%;
    height: auto;
    min-height: 28px;
    padding: 2px 6px;
    border: var(--sse-line) solid transparent;
    background: none;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: border-color 120ms ease, background 120ms ease;
  }
  .compact button {
    grid-template-columns: auto;
    padding: 2px;
    min-height: 24px;
  }
  button:hover,
  button:focus-visible,
  .lifted button {
    border-color: var(--sse-ink);
    background: var(--sse-ground-raised);
  }
  .name {
    font-family: var(--sse-font-display);
  }
  .how,
  .share {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .share {
    font-family: var(--sse-font-mono);
  }
  .pencil .name {
    font-style: italic;
    color: var(--sse-ink-muted);
    text-decoration: underline dotted;
    text-underline-offset: 3px;
  }
  .pat {
    font-family: var(--sse-font-mono);
    font-size: 7px;
  }
  @media (pointer: coarse) {
    button {
      min-height: 44px;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    button {
      transition: none;
    }
  }
</style>
