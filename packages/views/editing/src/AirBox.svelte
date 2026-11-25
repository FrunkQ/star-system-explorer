<!--
  The Air box (ED8 part 1, editing design section 11A), under the Air handle's amount strip: what the air is made of.
  One mix bar, the whole air by volume; gases under 1% sit below it as trace chips in ppm. Pressing a gas opens its own
  strip, its share on a log scale across eight decades: setting it trades it against the rest, which keep their
  proportions, so the mix always sums to one. The change is every moved share of `atmosphere.composition.<gas>` as one
  set of fields (`commitFields`), one Undo. The kernel's climate reads this mix (SOL5). Names come from the data sheet's
  substances; a gas it does not name is written as its formula.
-->
<script lang="ts">
  import { airKindEdits, airKinds, airMix, formulaOf, gasesToAdd, gasNames, MIX_FIELD, retrade, say, substanceNames, TRACE, type Depth, type Handle, type Json, type MixGas, type SolSheet, type ViewContext } from "../../../ui-contract/src/index.ts";
  import Strip from "./Strip.svelte";

  let {
    ctx,
    handle,
    facts,
    cls,
    depth,
    editable,
    sol = null,
    commitMix,
  }: {
    ctx: ViewContext;
    /** The Air handle (its strip is the amount; this box is the mix). */
    handle: Handle;
    /** The body's own facts. */
    facts: Record<string, Json>;
    cls: string;
    depth: Depth;
    editable: boolean;
    /** The catalogued Solar System, for Kinds for air. */
    sol?: SolSheet | null;
    /** One change of several shares; resolves to null when applied, or the reason it was not. */
    commitMix: (edits: { field: string; value: number; before: Json }[], display: string, before_display: string) => Promise<string | null>;
  } = $props();

  const mix = $derived(airMix(facts).filter((g) => g.share > 0));
  const bands = $derived(mix.filter((g) => g.share >= TRACE));
  const traces = $derived(mix.filter((g) => g.share < TRACE));

  // Names from the data sheet on the stack, or the sheet the shell ships when none is stacked.
  const stackedRef = $derived(substanceNames(ctx.session.stack().base).size ? ctx.session.stack().base : null);
  let shippedRef = $state.raw<unknown[] | null>(null);
  $effect(() => {
    if (stackedRef || shippedRef) return;
    void fetch("packs/base-reference.json")
      .then((r) => (r.ok ? r.json() : null))
      .then((p) => p && (shippedRef = [p]))
      .catch(() => {});
  });
  const refPacks = $derived(stackedRef ?? shippedRef ?? []);
  const names = $derived(substanceNames(refPacks));
  const nameOf = (gas: string) => names.get(gas) ?? formulaOf(gas);
  const share = (x: number) => say(x, "share", { class: cls }, { depth }).text;

  let open = $state<string | null>(null);
  const gasHandle = (gas: string): Handle => ({
    ...handle,
    id: `air.${gas}`,
    names: { curious: `${nameOf(gas)} in the air`, science: `${formulaOf(gas)}, share of the air` },
    part: { field: MIX_FIELD + gas, kind: "share", scale: "log", range: [1e-8, 1] },
    refs: [],
    box: undefined,
  });
  const wordsOf = (m: readonly MixGas[]) =>
    m
      .filter((g) => g.share >= TRACE)
      .slice(0, 3)
      .map((g) => `${formulaOf(g.gas)} ${share(g.share)}`)
      .join(", ");
  // Kinds for air (design 11A): a catalogued world's own pressure and mix, set as one change; from there you move it.
  const kinds = $derived(airKinds(sol));
  let said = $state<string | null>(null);
  async function useKind(id: string) {
    const k = kinds.find((x) => x.id === id);
    if (!k) return;
    const edits = airKindEdits(facts, k);
    if (!edits.length) return void (said = `It already has ${k.label.toLowerCase().replace(/^like /, "")}'s air.`);
    const p = say(k.pressure_pa, "pressure", { class: cls }, { depth }).text;
    said = await commitMix(edits, `${k.label}: ${p}, ${wordsOf(k.mix)}`, wordsOf(airMix(facts)));
  }
  // Add a gas: the data sheet's substances not in the air yet; it joins at 1,000 ppm and its strip opens.
  const toAdd = $derived(gasesToAdd(gasNames(refPacks, kinds.flatMap((k) => k.mix.map((g) => g.gas))), mix));
  async function addGas(gas: string) {
    if (!gas) return;
    said = await setShare(gas, 1e-3);
    if (!said) open = gas;
  }
  async function setShare(gas: string, x: number): Promise<string | null> {
    const all = airMix(facts);
    const next = retrade(all, gas, x);
    const was = new Map(all.map((g) => [g.gas, g.share]));
    const edits = next.filter((g) => was.get(g.gas) !== g.share).map((g) => ({ field: MIX_FIELD + g.gas, value: g.share, before: was.get(g.gas) ?? 0 }));
    if (!edits.length) return null;
    return commitMix(edits, `${nameOf(gas)} ${share(x)} (${wordsOf(next)})`, wordsOf(all));
  }
</script>

<div class="air" data-air-box>
  {#if mix.length}
    <p class="head">What it is made of, by volume</p>
    <div class="bar" role="group" aria-label="The air's mix" data-mix-bar>
      {#each bands as g (g.gas)}
        <button type="button" class="band" class:open={open === g.gas} style:flex-grow={g.share} title={`${nameOf(g.gas)}: ${share(g.share)}`} aria-pressed={open === g.gas} data-gas={g.gas} onclick={() => (open = open === g.gas ? null : g.gas)}>
          {#if g.share >= 0.08}<span>{formulaOf(g.gas)} {share(g.share)}</span>{/if}
        </button>
      {/each}
    </div>
    {#if traces.length}
      <div class="traces" role="group" aria-label="Traces">
        {#each traces as g (g.gas)}
          <button type="button" class="chip" class:open={open === g.gas} aria-pressed={open === g.gas} data-gas={g.gas} title={nameOf(g.gas)} onclick={() => (open = open === g.gas ? null : g.gas)}>{formulaOf(g.gas)} {share(g.share)}</button>
        {/each}
      </div>
    {/if}
    <!-- Honest note (design 11A): the surface chain does not yet read an authored mix (kernel ask K7). -->
    <p class="head note" data-air-note>The mix is kept and shown; it does not change the climate yet (that waits on the kernel's air chain).</p>
    {#if open}
      {@const g = mix.find((x) => x.gas === open)}
      <Strip handle={gasHandle(open)} value={g?.share ?? null} dots={[]} {cls} {depth} {editable} oncommit={(si) => setShare(open!, si)} onclose={() => (open = null)} />
    {/if}
  {:else}
    <p class="head">No mix is set for this air: the climate assumes one.</p>
  {/if}
  {#if editable && (kinds.length || toAdd.length)}
    <div class="more" role="group" aria-label="Start from, or add">
      {#if kinds.length}<span class="lead">Start from</span>{/if}
      {#each kinds as k (k.id)}<button type="button" class="chip" data-air-kind={k.id} title={`${k.label}, as measured (${k.src})`} onclick={() => useKind(k.id)}>{k.label}</button>{/each}
      {#if toAdd.length}
        <select aria-label="Add a gas" data-add-gas onchange={(e) => { const v = (e.currentTarget as HTMLSelectElement).value; (e.currentTarget as HTMLSelectElement).value = ""; void addGas(v); }}>
          <option value="">Add a gas…</option>
          {#each toAdd as g (g.gas)}<option value={g.gas}>{g.name} ({formulaOf(g.gas)})</option>{/each}
        </select>
      {/if}
    </div>
    {#if said}<p class="head note" role="status">{said}</p>{/if}
  {/if}
</div>

<style>
  .air {
    padding: 6px 18px 10px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
  }
  .head {
    margin: 0 0 6px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .more {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    margin-top: 8px;
    font-size: 12px;
  }
  .lead {
    color: var(--sse-ink-muted);
  }
  select {
    min-height: 26px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-size: 12px;
  }
  .note {
    margin: 6px 0 0;
  }
  .bar {
    display: flex;
    min-width: 0;
    height: 30px;
    border: var(--sse-line) solid var(--sse-ink-muted);
  }
  .band {
    flex-basis: 0;
    min-width: 3px;
    padding: 0 4px;
    border: 0;
    border-right: var(--sse-line) solid var(--sse-ground);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    cursor: pointer;
  }
  .band:nth-child(2n) {
    background: var(--sse-rule);
  }
  .band.open,
  .chip.open {
    outline: 2px solid var(--sse-accent);
    outline-offset: -2px;
  }
  .traces {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 6px;
  }
  .chip {
    min-height: 26px;
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    border-radius: 13px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-family: var(--sse-font-mono);
    font-size: 11px;
    cursor: pointer;
  }
  .air :global([data-strip]) {
    margin: 6px -18px 0;
  }
</style>
