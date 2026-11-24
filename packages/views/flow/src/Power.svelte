<!--
  The Power card (RK13, Atomic Rockets plan 16, question 18): a ship's power books (kernel `ship_power`, RK9) as a
  Flow view in three columns (comes from, used by, ends up as) with five plain readings under it. It is live: slide
  the starlight and a solar craft's flow thins; switch a load off and the engines get more. Trying things here keeps
  nothing. Broken books arrive as a dashed band from "Nowhere" with one sentence saying how much, on the GM's seat only
  (an honesty mark); players see their own ship's books without it.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { POWER_COLUMNS, constant, nowhereWords, powerColumn, powerFlowRows, powerRank, powerReadings, shipRecordOf, wattsText, type Json, type NavShip, type PowerBooks, type RadiatorRule, type ViewContext } from "../../../ui-contract/src/index.ts";
  import Flow from "./Flow.svelte";

  let { ctx, nav }: { ctx: ViewContext; nav: string | NavShip | null } = $props();
  const session = untrack(() => ctx.session);
  const gm = session.seat.authority === "gm";
  const depth = $derived(ctx.workspace.depth ?? "science");

  // Radiator panels as the base-ships pack has them (served beside the page, scripts/build-shell.sh).
  let rule = $state<RadiatorRule | null>(null);
  const SHIPS_PACK = "packs/base-ships.json";
  $effect(() => {
    void fetch(new URL(SHIPS_PACK, document.baseURI))
      .then((r) => r.json())
      .then((p: { provides: { other: { ship_design: { radiator: RadiatorRule } } } }) => (rule = p.provides.other.ship_design.radiator))
      .catch(() => (rule = null));
  });

  type Load = { name: string; electric_w: number; off?: boolean };
  type Source = { id?: string; kind: string; law?: { kind: string } };
  let record = $state<Record<string, unknown> | null>(null);
  let note = $state<string | null>(null);
  $effect(() => {
    const n = nav;
    if (!n) return void (record = null);
    void shipRecordOf(session, n).then((r) => {
      if (r.ok) ((record = r.ship), (offs = Object.fromEntries(loadsOf(r.ship).map((l) => [l.name, !!l.off]))), (au = 1));
      else ((record = null), (note = r.message));
    });
  });
  const loadsOf = (s: Record<string, unknown>): Load[] => (Array.isArray(s.loads) ? (s.loads as Load[]) : []);
  const sourcesOf = (s: Record<string, unknown>): Source[] => (Array.isArray(s.power) ? (s.power as Source[]) : []);
  const fitted = $derived(record !== null && (sourcesOf(record).length > 0 || (Array.isArray(record.engines) && record.engines.length > 0)));
  const sunlit = $derived(record !== null && sourcesOf(record).some((s) => s.law?.kind === "sunlight"));

  // What is being tried: loads switched off, and the starlight as at a distance from the Sun.
  let offs = $state<Record<string, boolean>>({});
  let au = $state(1);
  let books = $state<PowerBooks | null>(null);
  let asked = $state<Json | null>(null);
  let gen = 0;
  $effect(() => {
    const r = record;
    if (!r || !fitted) return void (books = null);
    const ship = { ...r, loads: loadsOf(r).map((l) => ({ ...l, off: !!offs[l.name] })) } as unknown as Json;
    const flux = constant("S_sun") / (au * au);
    const panels = rule;
    const g = ++gen;
    void session.shipPower({ ship, flux_w_m2: flux, ...(panels ? { radiator: panels } : {}) }).then((res) => {
      if (g !== gen) return;
      if (res.ok) ((books = res.power), (asked = ship), (note = null));
      else ((books = null), (note = res.message));
    });
  });

  const rows = $derived(books ? powerFlowRows(books, { gm, ship: asked }) : []);
  const readings = $derived(books && rule ? powerReadings(books, rule) : []);
  const nowhere = $derived(gm && books ? nowhereWords(books) : null);
  const tried = $derived(au !== 1 || (record !== null && loadsOf(record).some((l) => !!offs[l.name] !== !!l.off)));
</script>

<section class="power" aria-label="Power" data-power-card>
  <header>
    <h4>Power</h4>
    <p class="small muted">Where its power comes from, what uses it, and where it ends up.</p>
  </header>
  {#if !nav}
    <p class="small muted">This craft has no ship record, so it has no power books.</p>
  {:else if record && !fitted}
    <p class="small muted" data-power-typed>Its drive is typed figures rather than parts, so there are no books to draw. Fit it a power plant and engines on the Build desk and they appear here.</p>
  {:else if books}
    {#if sunlit || (record && loadsOf(record).length)}
      <div class="bar">
        {#if sunlit}
          <label class="ctl">
            <span>Starlight as at</span>
            <input type="range" min="0.3" max="10" step="0.1" bind:value={au} data-power-au />
            <output>{au.toFixed(1)} au</output>
          </label>
        {/if}
        {#if record}
          {#each loadsOf(record) as l (l.name)}
            <button type="button" class="tog" aria-pressed={!offs[l.name]} onclick={() => (offs = { ...offs, [l.name]: !offs[l.name] })} data-power-load={l.name}>{l.name}</button>
          {/each}
        {/if}
        {#if tried}<span class="small muted" data-power-tried>Trying it out: nothing is kept.</span>{/if}
      </div>
    {/if}
    {#if rows.length}
      <Flow {rows} titles={POWER_COLUMNS} column={powerColumn(rows)} rank={powerRank(books)} amountWords={wattsText} figures={depth !== "curious"} label="Power flow" />
    {:else}
      <p class="small muted">Nothing on board makes or uses power.</p>
    {/if}
    {#if nowhere}<p class="nowhere" data-nowhere title={books.balance.says ?? ""}>{nowhere}</p>{/if}
    <dl class="read" data-power-readings>
      {#each readings as r, i (i)}
        <div data-reading={r.id} title={r.block && depth !== "curious" ? `Worked out by the ${r.block} block` : undefined}>
          <dt>{r.label}</dt>
          <dd><b>{r.value}</b> {r.words}</dd>
        </div>
      {/each}
    </dl>
    {#if depth === "engine" && (books.ledger.why.length || books.why.length)}
      <ul class="why small muted" data-power-why>
        {#each [...books.ledger.why, ...books.why] as w, i (i)}<li>{w}</li>{/each}
      </ul>
    {/if}
  {:else if note}
    <p class="small muted">{note}</p>
  {:else}
    <p class="small muted">Reading its books…</p>
  {/if}
</section>

<style>
  .power {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }
  header h4 {
    margin: 0;
  }
  header p {
    margin: 0;
  }
  .small {
    font-size: 12px;
  }
  .muted {
    color: var(--sse-ink-muted);
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px 14px;
  }
  .ctl {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .ctl output {
    font-family: var(--sse-font-mono);
    min-width: 6ch;
    color: var(--sse-ink);
  }
  input[type="range"] {
    accent-color: var(--sse-accent);
    width: 120px;
  }
  .tog {
    font: inherit;
    font-size: 12px;
    color: var(--sse-ink);
    background: transparent;
    border: 1px solid var(--sse-line);
    border-radius: var(--sse-radius, 0);
    padding: 3px 8px;
    cursor: pointer;
    text-decoration: line-through;
    opacity: 0.7;
  }
  .tog[aria-pressed="true"] {
    text-decoration: none;
    opacity: 1;
    border-color: var(--sse-ink);
  }
  .nowhere {
    margin: 0;
    font-size: 13px;
    color: var(--sse-alert, var(--sse-accent));
  }
  .read {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 6px 12px;
    margin: 0;
  }
  .read div {
    min-width: 0;
  }
  .read dt {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .read dd {
    margin: 0;
    font-size: 13px;
  }
  .read dd b {
    font-family: var(--sse-font-mono);
    font-weight: 500;
  }
  .why {
    margin: 0;
    padding-left: 18px;
  }
</style>
