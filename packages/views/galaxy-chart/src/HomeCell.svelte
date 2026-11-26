<!--
  The home star cell (V3 gap 17 part 2b): the 10 pc cell of the campaign's galaxy around home, its drawn stars seen
  from above, as the kernel's `star_cell` draws them now. Read only and GM only: what lies around the campaign. A star
  the GM has looked at closely (promoted) keeps its numbers and is ringed; one that is already a campaign system is
  filled. On the GM's seat a pressed star can grow into a real system ("Grow a system here"): the kernel's `adopt` with
  `from_star`, so the system inherits that star's own mass, age and metals and the chart and the system agree.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { placeHash, systemShort, voyageFrame, type StarCell, type StarCellSystem, type ViewSession } from "../../../ui-contract/src/index.ts";

  let { session, campaign, cangrow = false, ongrown }: { session: ViewSession; campaign: readonly string[]; cangrow?: boolean; ongrown?: (system: string, title: string, note: string, state_hash: string) => void } = $props();

  const W = 320;
  let cell = $state<StarCell | null>(null);
  let failed = $state<string | null>(null);
  let hover = $state<StarCellSystem | null>(null);
  // The star pressed: it stays said, and on the GM's seat it can grow into a system.
  let chosen = $state<StarCellSystem | null>(null);
  let growing = $state(false);
  let grownNote = $state<string | null>(null);
  let growProblem = $state<string | null>(null);
  async function grow(s: StarCellSystem) {
    growing = true;
    growProblem = null;
    grownNote = null;
    // The seed is the star's own address, so growing the same star again in another campaign gives the same system.
    const r = await untrack(() => session).adopt({ formation: { seed: s.address, from_star: s.address } });
    growing = false;
    if (!r.ok) return void (growProblem = r.code === "already_adopted" ? "This star is already a system in the campaign." : r.message);
    const sys = r.result.systems[0] ?? s.address;
    grownNote = `${systemShort(s.address)} is now a system in the campaign, grown from its own birth: ${r.result.facts} facts.`;
    grownAt = sys;
    grownHere = [...grownHere, s.address];
    ongrown?.(sys, systemShort(s.address), grownNote, r.result.state_hash);
  }
  let grownAt = $state<string | null>(null);
  const said = $derived(hover ?? chosen);
  // Read when first opened: drawing a cell is the kernel's work, and most visits never look.
  let asked = false;
  function ontoggle(e: Event) {
    if (asked || !(e.currentTarget as HTMLDetailsElement).open) return;
    asked = true;
    untrack(() => session)
      .starCell({})
      .then((r) => (r.ok ? (cell = r.cell) : (failed = r.message)));
  }

  const stars = $derived(cell?.systems ?? []);
  const frame = $derived(voyageFrame(W, W, stars.map((s) => s.xyz_pc), 14));
  // Grown here this visit: filled at once (the kernel's `systems` list does not yet reach systems deep in the galaxy).
  let grownHere = $state<string[]>([]);
  const ours = $derived(new Set([...campaign, ...grownHere]));
  // Mass to a dot: the cube root keeps a red dwarf visible beside an A star.
  const dot = (s: StarCellSystem) => (stars.length > 200 ? 0.5 : 1) * (1.6 + 2.2 * Math.cbrt(Math.max(s.mass_msun, 0.05)));
  const two = (v: number) => Math.round(v * 100) / 100;
  const pc = $derived(cell ? frame.scale * 1 : 0);
</script>

<details class="home" data-home-cell={stars.length} {ontoggle}>
  <summary>Stars around home</summary>
  {#if failed}
    <p class="hint">The galaxy's stars could not be read here: {failed}</p>
  {:else if !cell}
    <p class="hint">Reading the home cell…</p>
  {:else}
    <p class="hint">{stars.length} stars drawn in the {cell.cell_pc} pc cell {cell.cell.join(".")} of the galaxy, seen from above. A ringed one has been looked at closely and keeps its numbers.</p>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <svg viewBox="0 0 {W} {W}" role="img" aria-label="The home star cell seen from above" onpointerleave={() => (hover = null)}>
      <rect class="box" x="1" y="1" width={W - 2} height={W - 2} />
      {#each stars as s (s.address)}
        {@const p = frame.at(s.xyz_pc)}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <circle class="st" class:ours={ours.has(s.address)} cx={p.x} cy={p.y} r={dot(s)} data-star={s.address} onpointerenter={() => (hover = s)} onclick={() => ((chosen = s), (grownNote = null), (growProblem = null))} class:chosen={chosen?.address === s.address} />
        {#if s.pinned}<circle class="pin" cx={p.x} cy={p.y} r={dot(s) + 3} />{/if}
      {/each}
      {#if pc > 4}<line class="bar" x1="10" y1={W - 10} x2={10 + pc} y2={W - 10} /><text class="lbl" x="10" y={W - 14}>1 pc</text>{/if}
    </svg>
    <p class="said" data-home-star={said?.address ?? ""}>
      {#if said}<b>{systemShort(said.address)}</b> · {two(said.mass_msun)} M☉{said.companions.length ? ` with ${said.companions.length} companion${said.companions.length > 1 ? "s" : ""}` : ""} · [Fe/H] {two(said.feh)}{ours.has(said.address) ? " · in this campaign" : ""}
      {:else}Rest on a star for its mass and metals{cangrow ? "; press one to grow a system there" : ""}.{/if}
    </p>
    {#if cangrow && chosen && !ours.has(chosen.address)}
      <p class="grow">
        <button type="button" onclick={() => chosen && grow(chosen)} disabled={growing} data-home-grow={chosen.address}>{growing ? "Growing…" : `Grow a system at ${systemShort(chosen.address)}`}</button>
        <span class="hint">Its planets form from this star's own birth, so what you fly into matches the chart.</span>
      </p>
    {/if}
    {#if growProblem}<p class="note" role="alert">{growProblem}</p>{/if}
    {#if grownNote && grownAt}<p class="hint" data-home-grown={grownAt}>{grownNote} <a href={placeHash({ scale: "system", system: grownAt })}>Open it ›</a></p>{/if}
  {/if}
</details>

<style>
  .home {
    margin-top: 12px;
    border-top: 1px solid var(--sse-rule);
    padding-top: 8px;
  }
  summary {
    cursor: pointer;
    font-weight: 600;
  }
  svg {
    width: 100%;
    max-width: 320px;
    aspect-ratio: 1;
    display: block;
  }
  .box {
    fill: none;
    stroke: var(--sse-rule);
  }
  .st {
    fill: var(--sse-ink-muted);
  }
  .st:hover {
    fill: var(--sse-accent);
  }
  .st.chosen {
    stroke: var(--sse-ink);
    stroke-width: 1.5;
  }
  .grow {
    margin: 6px 0;
  }
  .note {
    color: var(--sse-accent);
    font-size: 12px;
  }
  .st.ours {
    fill: var(--sse-accent);
  }
  .pin {
    fill: none;
    stroke: var(--sse-ink);
    pointer-events: none;
  }
  .bar {
    stroke: var(--sse-ink);
    stroke-width: 2;
  }
  .lbl {
    fill: var(--sse-ink-muted);
    font-size: 10px;
  }
  .hint,
  .said {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
</style>
