<!--
  Add here (V3 gap 2), on the Build desk: put one world (or moon) round the selected body by hand. It offers what
  this orbit would make, sampled from formation runs round a star like this one (classes are labels read off the
  results, so there is no list of types to keep in step), plus anything you like, marked impossible. Either goes in
  as one change: one Undo takes it back.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { anythingClip, clipAt, formsHere, freeOrbit, moonRecipes, say, worldKinds, type Clip, type FormsHere, type ViewSession } from "../../ui-contract/src/index.ts";

  let {
    session,
    system,
    host,
    orbits,
    at_au,
    packs,
    onadd,
  }: {
    session: ViewSession;
    system: string;
    host: {
      address: string;
      label: string;
      kind: string;
      mass_sun: number | null;
      planet?: { mass_earth: number; radius_earth: number; a_au: number | null; star_mass_sun: number | null };
    };
    /** Orbits already round the host, in AU. */
    orbits: number[];
    /** Where the Add ghost was dropped on the chart (ED4), in AU; otherwise the first free orbit. */
    at_au?: number;
    packs: { id: string; version: string }[];
    onadd: (clip: Clip, words: string) => Promise<string | null>;
  } = $props();

  const moon = $derived(host.kind !== "star");
  let a = $state(untrack(() => at_au ?? (host.kind === "star" ? freeOrbit(orbits) : freeOrbit(orbits, 0.003))));
  let name = $state("");
  let found = $state<FormsHere[] | null>(null);
  let runs = $state(0);
  let looking = $state(false);
  let problem = $state<string | null>(null);
  let adding = $state(false);

  async function look() {
    if (moon || !host.mass_sun || !(a > 0)) return;
    looking = true;
    problem = null;
    const r = await formsHere(session, { system, star_mass_sun: host.mass_sun, a_au: a, packs });
    looking = false;
    if (!r.ok) return void ((problem = r.message), (found = null));
    found = r.found;
    runs = r.runs;
  }
  $effect(() => {
    untrack(() => void look());
  });

  // Anything: a mass and a size you choose.
  let mass = $state(1);
  let radius = $state(1);

  const named = (c: Clip, fallback: string): Clip => {
    const n = name.trim() || fallback;
    return { ...c, events: c.events.map((e) => (e.node === c.root && e.field === "name" ? { ...e, value: n } : e)) };
  };
  async function add(c: Clip, words: string) {
    adding = true;
    problem = await onadd(clipAt(c, a), words);
    adding = false;
  }
  // A moon by hand: formation's own leftovers rules for this planet, as three fixed recipes until Ask K5 (ED7).
  const recipes = $derived(moon && host.planet ? moonRecipes({ name: host.label, ...host.planet }, orbits) : []);
  // Kinds (ED6, editing design section 9): each world class's "make one" recipe from the class atlases on the stack.
  // From the stack's atlases, or the base atlas the shell ships (as Names and classes offers it) when none is stacked.
  const stacked = $derived(moon ? [] : worldKinds(session.stack().base));
  let shipped = $state.raw<ReturnType<typeof worldKinds>>([]);
  $effect(() => {
    if (moon || stacked.length || shipped.length) return;
    void fetch("packs/base-atlas.json")
      .then((r) => (r.ok ? r.json() : null))
      .then((p) => p && (shipped = worldKinds([p])))
      .catch(() => {});
  });
  const kinds = $derived(stacked.length ? stacked : shipped);
  async function addRecipe(c: Clip, fallback: string) {
    adding = true;
    problem = await onadd(named(c, fallback), name.trim() || fallback);
    adding = false;
  }
  const fmt = (x: number) => Number(x.toPrecision(x < 0.1 ? 1 : 2));
  const fallbackName = $derived(moon ? `New moon of ${host.label}` : "New world");
</script>

<h2 id="add-title">Add round {host.label}</h2>
<div class="row">
  <label>Orbit <input type="number" min="0.0001" step="any" bind:value={a} onchange={look} data-add-orbit /> AU</label>
  <label>Name <input type="text" bind:value={name} placeholder={fallbackName} data-add-name /></label>
</div>

{#if !moon}
  <h3>What forms here</h3>
  {#if looking}<p class="hint" role="status">Running formation round a star like {host.label}…</p>
  {:else if found}
    {#if found.length}
      <p class="hint">From {runs} formation runs round a star of {fmt(host.mass_sun ?? 1)} solar masses, the world each made nearest {fmt(a)} AU.</p>
      <ul class="offers" data-forms-here>
        {#each found as f (f.seed)}
          <li>
            <span><b>{fmt(f.mass_earth)} Earth masses</b>, {fmt(f.radius_earth)} Earth radii. <small>Formed at {fmt(f.formed_au)} AU{f.surface_k !== null ? `, with a surface about ${Math.round(f.surface_k)} K there` : ""}; the physics works out its surface here.</small></span>
            <button type="button" disabled={adding} onclick={() => add(named(f.clip, fallbackName), name.trim() || fallbackName)}>Add this</button>
          </li>
        {/each}
      </ul>
    {:else}<p class="hint" data-forms-nothing>None of {runs} formation runs made a world near {fmt(a)} AU. Try another orbit, or add something of your own below.</p>{/if}
  {/if}
{:else if recipes.length}
  <h3>How moons form</h3>
  <p class="hint">The rules formation uses for every generated system, put to {host.label}: each goes in at its own orbit, and the physics works out the rest.</p>
  <ul class="offers" data-moon-recipes>
    {#each recipes as r (r.id)}
      <li data-moon-recipe={r.id}>
        <span><b>{r.title}</b>, {say(r.mass_kg, "mass", { class: "moon" }).text} at {say(r.a_m, "length", { class: "moon" }).text} from it. <small>{r.says} ({r.cite})</small></span>
        <button type="button" disabled={adding} onclick={() => addRecipe(r.clip, `${r.title.replace(/^An? /, "").replace(/^./, (x) => x.toUpperCase())} of ${host.label}`)}>Add this</button>
      </li>
    {/each}
  </ul>
{:else}
  <p class="hint">Moons are not sampled from formation yet: add one of your own below.</p>
{/if}

{#if kinds.length}
  <h3>Kinds</h3>
  <p class="hint">A world of each kind the class atlas knows, with its example's mass and size, at the orbit above. The physics works out the rest there, and reads its kind off the results, so the label may change.</p>
  <ul class="offers" data-kinds>
    {#each kinds as k (k.class)}
      <li data-kind={k.class}>
        <span><b>{k.name}</b>, like {k.like}: {say(k.mass_kg, "mass", { class: "planet" }).text}. <small>{k.says}</small></span>
        <button type="button" disabled={adding || !(a > 0)} onclick={() => add(named(k.clip, fallbackName), name.trim() || fallbackName)}>Add this</button>
      </li>
    {/each}
  </ul>
{/if}

<h3>Something else</h3>
<p class="hint">Any {moon ? "moon" : "world"} you like. It goes in marked impossible, because you chose it and the physics did not make it here; the physics still works out the rest from it.</p>
<div class="row">
  <label>Mass <input type="number" min="0" step="any" bind:value={mass} data-add-mass /> Earth masses</label>
  <label>Radius <input type="number" min="0" step="any" bind:value={radius} data-add-radius /> Earth radii</label>
  <button type="button" data-add-anything disabled={adding || !(mass > 0) || !(radius > 0)} onclick={() => add(anythingClip({ name: name.trim() || fallbackName, kind: moon ? "moon" : "planet", mass_earth: mass, radius_earth: radius }), name.trim() || fallbackName)}>Add it, marked impossible</button>
</div>
{#if problem}<p role="alert">{problem}</p>{/if}

<style>
  h2 {
    margin: 0 0 12px;
    font-family: var(--sse-font-display);
  }
  h3 {
    margin: 16px 0 4px;
    font-size: 15px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 16px;
    align-items: center;
  }
  .row input[type="number"] {
    width: 6em;
  }
  .hint {
    color: var(--sse-ink-muted);
    font-size: 13px;
    margin: 4px 0;
  }
  .offers {
    list-style: none;
    padding: 0;
    margin: 6px 0;
  }
  .offers li {
    display: flex;
    gap: 12px;
    align-items: center;
    justify-content: space-between;
    padding: 6px 0;
    border-top: var(--sse-line) solid var(--sse-rule);
    font-size: 14px;
  }
  button {
    height: 36px;
    padding: 0 14px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    font-family: var(--sse-font-body);
    flex: none;
  }
  button:disabled {
    opacity: 0.5;
  }
</style>
