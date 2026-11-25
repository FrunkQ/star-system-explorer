<!--
  Find a home (V9, U6 part 2b): search the cells around this one, nearest first, for a galaxy whose history fits
  what the GM wants: its halo's mass, how long since its last major merger, a big neighbour bound to it or none.
  Starts from "Like the Milky Way" (cited in home-finder.ts); every wish can be changed. Each cell is the kernel's
  halo_cell, read one at a time so the page stays live; Stop ends the search with what it has.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { LIKE_THE_MILKY_WAY, cellsAround, findHomes, sci, type HaloCell, type HomeFit, type HomeWish, type ViewSession } from "../../../ui-contract/src/index.ts";

  let { session, from, ongo }: { session: ViewSession; from: [number, number, number]; ongo: (fit: HomeFit) => void } = $props();

  let wish = $state<HomeWish>({ ...LIKE_THE_MILKY_WAY });
  let read = $state<HaloCell[]>([]);
  let searching = $state(false);
  let done = $state(false);
  let failed = $state<string | null>(null);
  let run = 0;
  const RADIUS = 1;
  const ENOUGH = 6;
  const found = $derived(findHomes(read, wish));
  const MASSES = [1e11, 2e11, 5e11, 1e12, 2e12, 5e12, 1e13] as const;

  async function search() {
    const r = ++run;
    const around = cellsAround(untrack(() => from), RADIUS);
    read = [];
    done = false;
    failed = null;
    searching = true;
    for (const c of around) {
      const got = await session.haloCell({ cell: c, min_mass_msun: wish.mass_min });
      if (r !== run) return;
      if (!got.ok) {
        failed = got.message;
        break;
      }
      read = [...read, got.cell];
      if (findHomes(read, wish).length >= ENOUGH) break;
    }
    searching = false;
    done = true;
  }
  function stop() {
    run++;
    searching = false;
    done = true;
  }
  const nameOf = (a: string) => a.split("/").at(-1)?.replace("halo:", "Halo ") ?? a;
</script>

<details class="find" data-find-home={found.length}>
  <summary>Find a home</summary>
  <p class="hint">Search the cells around this one, nearest first, for a galaxy whose history fits. It starts from one like the Milky Way; change any wish.</p>
  <div class="wishes">
    <label>Halo from
      <select bind:value={wish.mass_min} data-wish="mass_min">{#each MASSES as m (m)}<option value={m}>{sci(m, 1)}</option>{/each}</select>
    </label>
    <label>to
      <select bind:value={wish.mass_max} data-wish="mass_max">{#each MASSES as m (m)}<option value={m}>{sci(m, 1)}</option>{/each}</select> suns
    </label>
    <label>Quiet for at least <input type="number" min="0" max="13" step="0.5" bind:value={wish.quiet_gyr} data-wish="quiet_gyr" /> Gyr</label>
    <label>Neighbour
      <select bind:value={wish.neighbour} data-wish="neighbour">
        <option value="bound">a big one, bound to it</option>
        <option value="alone">none close</option>
        <option value="any">no wish</option>
      </select>
    </label>
    {#if wish.neighbour !== "any"}<label>within <input type="number" min="0.1" max="10" step="0.1" bind:value={wish.neighbour_mpc} data-wish="neighbour_mpc" /> Mpc</label>{/if}
  </div>
  <div class="act">
    {#if searching}
      <button type="button" onclick={stop} data-find-stop>Stop</button>
      <span class="hint">read {read.length} of {(2 * RADIUS + 1) ** 3} cells…</span>
    {:else}
      <button type="button" class="go" onclick={search} data-find-search>Search nearby cells</button>
      {#if done}<span class="hint" data-find-said>{found.length ? `${found.length} found in ${read.length} cells, nearest the middle of the mass range first.` : `None in ${read.length} cells. Widen a wish and search again.`}</span>{/if}
    {/if}
  </div>
  {#if failed}<p class="note" role="alert">A cell could not be read: {failed}</p>{/if}
  {#if found.length}
    <ol class="fits">
      {#each found.slice(0, 12) as f (f.halo.address)}
        <li>
          <button type="button" onclick={() => ongo(f)} data-find-go={f.halo.address}>{nameOf(f.halo.address)} in cell {f.cell.join(" · ")}</button>
          <span class="why">{f.why.join("; ")}</span>
        </li>
      {/each}
    </ol>
  {/if}
</details>

<style>
  .find {
    margin: 12px 0;
    border-top: 1px solid var(--sse-rule);
    padding-top: 8px;
  }
  summary {
    cursor: pointer;
    font-weight: 600;
  }
  .wishes {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 12px;
    margin: 6px 0;
  }
  input[type="number"] {
    width: 4.5em;
  }
  .act {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
  }
  .fits {
    padding-left: 18px;
    margin: 8px 0 0;
  }
  .fits li {
    margin-bottom: 6px;
  }
  .fits button {
    background: none;
    border: none;
    padding: 0;
    color: var(--sse-accent);
    text-decoration: underline;
    cursor: pointer;
    font: inherit;
  }
  .why,
  .hint {
    display: block;
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .note {
    color: var(--sse-accent);
  }
</style>
