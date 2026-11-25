<!--
  The home star cell (V3 gap 17 part 2b): the 10 pc cell of the campaign's galaxy around home, its drawn stars seen
  from above, as the kernel's `star_cell` draws them now. Read only and GM only: what lies around the campaign. A star
  the GM has looked at closely (promoted) keeps its numbers and is ringed; one that is already a campaign system is
  filled. Growing a system at a drawn star waits on the kernel's `from_star` (formation from that star's own birth).
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { systemShort, voyageFrame, type StarCell, type StarCellSystem, type ViewSession } from "../../../ui-contract/src/index.ts";

  let { session, campaign }: { session: ViewSession; campaign: readonly string[] } = $props();

  const W = 320;
  let cell = $state<StarCell | null>(null);
  let failed = $state<string | null>(null);
  let hover = $state<StarCellSystem | null>(null);
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
  const ours = $derived(new Set(campaign));
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
        <circle class="st" class:ours={ours.has(s.address)} cx={p.x} cy={p.y} r={dot(s)} data-star={s.address} onpointerenter={() => (hover = s)} />
        {#if s.pinned}<circle class="pin" cx={p.x} cy={p.y} r={dot(s) + 3} />{/if}
      {/each}
      {#if pc > 4}<line class="bar" x1="10" y1={W - 10} x2={10 + pc} y2={W - 10} /><text class="lbl" x="10" y={W - 14}>1 pc</text>{/if}
    </svg>
    <p class="said" data-home-star={hover?.address ?? ""}>
      {#if hover}<b>{systemShort(hover.address)}</b> · {two(hover.mass_msun)} M☉{hover.companions.length ? ` with ${hover.companions.length} companion${hover.companions.length > 1 ? "s" : ""}` : ""} · [Fe/H] {two(hover.feh)}{ours.has(hover.address) ? " · in this campaign" : ""}
      {:else}Rest on a star for its mass and metals.{/if}
    </p>
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
