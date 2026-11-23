<!--
  "Why?" for a world's climate (W7a): the kernel's one climate, in plain words.
  The regime, a short table of temperature and rain by 30° belt, then the
  kernel's own why lines (an extrapolation carries the "out of range" mark)
  and what it assumed. Read-only; the Workshop's climate blocks come in W12.
-->
<script lang="ts">
  import { climateBelts, constant, isExtrapolation, regimeWords, type ClimateSummary, type Depth } from "../../../ui-contract/src/index.ts";
  import { Stamp, opensFrom } from "../../../chart-grammar/src/index.ts";

  let { climate, label, depth = "curious", from, onclose }: { climate: ClimateSummary; label: string; depth?: Depth; from?: () => Element | null | undefined; onclose: () => void } = $props();

  const belts = $derived(climateBelts(climate));
  // The kernel repeats its assumptions among the why lines; they are listed once, under Assumed.
  const lines = $derived(climate.assumed.length ? climate.why.filter((w) => !/^assumed:/i.test(w)) : climate.why);
  const c = (k: number) => Math.round(k - constant("T0C"));
  const mm = (m: number) => (m <= 0 ? "none" : m < 0.01 ? "a trace" : `${Math.round(m * 1000).toLocaleString("en-GB")} mm`);
  const n = (x: number, d = 1) => x.toLocaleString("en-GB", { maximumFractionDigits: d });
</script>

<div class="climate-why" use:opensFrom={() => from?.() ?? null} role="region" aria-label="Why {label}'s climate" data-climate-why={climate.regime}>
  <div class="top">
    <span class="k">Why</span>
    <span class="t">{label}'s climate: <b data-climate-regime>{regimeWords(climate.regime)}</b></span>
    <button type="button" class="close" onclick={onclose} aria-label="Close the climate explanation">×</button>
  </div>
  <table data-climate-belts>
    <caption>Year-round mean by latitude</caption>
    <thead><tr><th scope="col">Belt</th><th scope="col">Temperature</th><th scope="col">Rain a year</th></tr></thead>
    <tbody>
      {#each belts as b (b.label)}
        <tr>
          <th scope="row">{b.label}</th>
          <td class="num">{c(b.t_k)} °C{#if depth !== "curious"} <span class="muted">({Math.round(b.t_k)} K)</span>{/if}</td>
          <td class="num">{mm(b.precip_m_yr)}</td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if depth !== "curious"}
    <p class="muted" data-climate-numbers>
      {climate.cells} circulation cell{climate.cells === 1 ? "" : "s"} a hemisphere, Hadley edge {n(climate.hadley_edge_deg, 0)}°; air cools {n(climate.lapse_k_km)} K per km; surface pressure {n(climate.pressure_bar, 3)} bar{climate.liquid ? `; surface liquid ${climate.liquid}` : ""}.
    </p>
  {/if}
  <ul class="lines" data-climate-lines>
    {#each lines as w, i (i)}
      <li data-extrapolated={isExtrapolation(w) || undefined}>{#if isExtrapolation(w)}<Stamp kind="out-of-range" title="Outside the range the climate model was tested on: an extrapolation, shown but not trusted like the rest." /> {/if}{w}</li>
    {/each}
  </ul>
  {#if climate.assumed.length}
    <p class="k sub">Assumed</p>
    <ul class="lines" data-climate-assumed>
      {#each climate.assumed as a, i (i)}<li>{a}</li>{/each}
    </ul>
  {/if}
  {#if depth === "engine"}<p class="muted key" data-climate-key>{climate.key}</p>{/if}
</div>

<style>
  .climate-why {
    margin-top: 8px;
    padding: 10px 14px 12px;
    background: var(--sse-ground);
    border-top: var(--sse-line) solid var(--sse-ink);
    border-bottom: var(--sse-line) solid var(--sse-ink);
    font-size: 13px;
    line-height: 1.5;
  }
  .top {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .t {
    flex: 1;
  }
  .k {
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-accent-ink);
  }
  .sub {
    margin: 8px 0 0;
  }
  .close {
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font-size: 18px;
    min-width: 32px;
    min-height: 28px;
    cursor: pointer;
  }
  table {
    border-collapse: collapse;
    margin: 6px 0;
    width: 100%;
  }
  caption {
    text-align: left;
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  th,
  td {
    text-align: left;
    padding: 2px 10px 2px 0;
    border-bottom: 1px solid var(--sse-ink-faint);
    font-weight: normal;
  }
  thead th {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .num {
    font-variant-numeric: tabular-nums;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .lines {
    margin: 4px 0 0;
    padding-left: 18px;
  }
  .key {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    word-break: break-all;
  }
</style>
