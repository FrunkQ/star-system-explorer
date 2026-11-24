<!--
  Seats (V10, H8.7): more than one view at once, never in the way. With the
  default single table the panel says who sees what in one line and offers
  to add a seat; with several it lists them, each with its join link per
  screen. A fog-of-war seat starts knowing nothing; the GM marks what it has
  discovered, and the marks are read back from the kernel's own scene for
  that audience (nothing is remembered here).
-->
<script lang="ts">
  import { EVERYONE, LIVE_OVERRIDES, checkBrand, embedOrigins, fogViews, isDefaultPlan, observeOf, type Audience, type SceneSubject, type Shown, type ViewSession } from "../../ui-contract/src/index.ts";
  import type { Share } from "./share.svelte.ts";

  let { share, session, bodies, scope, at_yr, onchange, here }: { share: Share; session: ViewSession; bodies: SceneSubject[]; scope: string; at_yr: number; onchange: () => void; here?: () => Shown } = $props();
  // What a seat is shown, in words: its own send, else the table's.
  const seesWords = (x: Shown) => ("off" in x ? "nothing (screens off)" : (x.title ?? "the opening place"));

  const SCREENS = [
    { file: "players.screen.json", label: "Bridge" },
    { file: "players.survey.screen.json", label: "Tarn survey report" },
    { file: "players.bridge-3d.screen.json", label: "Bridge in 3D" },
  ];
  const setup = $derived(observeOf(session.stack().base)?.setup);
  const fog = $derived(fogViews(setup));
  const viewLabel = (id: string | undefined) => setup?.views.find((v) => v.id === id)?.label ?? id ?? "";
  const plan = $derived(share.state.plan);
  const single = $derived(isDefaultPlan(plan));

  let label = $state("");
  let withFog = $state(true);
  let names = $state(true);
  let autoNames = $state(false);
  let sealed = $state(false);
  let screens = $state<string[]>(["players.screen.json"]);
  let problem = $state("");
  // Tabletops this seat may sit inside (V3 gap 26), one web address per line.
  let tabletops = $state("");
  // A brand for the seat's screens (V3 gap 25): a faction's name and a small logo.
  let brandName = $state("");
  let brandLogo = $state<string | undefined>(undefined);
  function readLogo(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (!f) return void (brandLogo = undefined);
    const r = new FileReader();
    r.onload = () => (brandLogo = String(r.result));
    r.readAsDataURL(f);
  }

  async function add(e: SubmitEvent) {
    e.preventDefault();
    if (!label.trim()) return void (problem = "Give the seat a name, for example Empire: Vesh.");
    const vtt = embedOrigins(tabletops.split(/[\s,]+/));
    if (vtt.problems.length) return void (problem = `Tabletop addresses: ${vtt.problems.join("; ")}.`);
    const brand = brandName.trim() ? { name: brandName.trim(), ...(brandLogo ? { logo: brandLogo } : {}) } : undefined;
    const brandBad = brand ? checkBrand(brand) : [];
    if (brandBad.length) return void (problem = `Brand: ${brandBad.join("; ")}.`);
    const bad = await share.addSeat({ label: label.trim(), ...(withFog && fog ? { fog } : {}), names, autoNames: autoNames ? [scope] : [], screens: screens.length > 1 || screens[0] !== "players.screen.json" ? screens : [], sealed, embedOrigins: vtt.origins, ...(brand ? { brand } : {}) });
    problem = bad ?? "";
    if (!bad) {
      label = "";
      tabletops = "";
      brandName = "";
      brandLogo = undefined;
      onchange();
      await refresh();
    }
  }

  // What each fog-of-war audience knows, read from the kernel's scene for it.
  let known = $state<Record<string, string[]>>({});
  const foggy = $derived(plan.audiences.filter((a) => a.id !== EVERYONE && a.starts_with));
  async function refresh() {
    const out: Record<string, string[]> = {};
    for (const a of foggy) {
      const r = await session.scene(scope, { audience: a.id });
      out[a.id] = r.ok ? r.scene.subjects.map((s) => s.address) : [];
    }
    known = out;
  }
  $effect(() => {
    void foggy.length;
    void refresh();
  });
  async function toggle(a: Audience, address: string, on: boolean) {
    const bad = await share.discover(a, address, on, at_yr);
    problem = bad ?? "";
    onchange();
    await refresh();
  }
  const seatFor = (audience: string) => plan.seats.filter((s) => s.audience === audience).map((s) => s.label).join(", ");
</script>

<section class="seats" aria-labelledby="seats-title">
  <h2 id="seats-title">Seats</h2>
  {#if single}
    <p class="one" data-seats="1">Everyone sees: <b>{viewLabel(setup?.campaign.default_view)}</b>, until you scan a body. One link for the whole table.</p>
  {:else}
    <ul class="list" data-seats={plan.seats.length}>
      {#each plan.seats as s (s.id)}
        {@const a = plan.audiences.find((x) => x.id === s.audience)}
        <li data-seat={s.id}>
          <span class="name">{s.label}</span>
          <span class="meta">{s.audience === EVERYONE ? "sees what everyone has been shown" : a?.starts_with ? "fog of war: sees its own discoveries" : "its own knowledge"}{s.sealed ? " · sealed" : ""}{s.brand ? ` · branded ${s.brand.name}` : ""}{s.embed_origins?.length ? ` · may sit inside ${s.embed_origins.join(", ")}` : ""}{s.may_send.length ? (s.auto_approve?.length ? " · names apply without asking" : " · may suggest names") : ""}</span>
          <span class="links">
            {#each share.links(s.id) as l (l.href)}
              <a href={l.href} target="_blank" rel="noopener" data-seat-link={s.id}>{l.screen ? (SCREENS.find((x) => x.file === l.screen)?.label ?? l.screen) : "join link"}</a>
              {#if s.embed_origins?.length}<a href={`${l.href}&embed=1`} target="_blank" rel="noopener" data-seat-embed={s.id} title="Paste this into the tabletop's web or iframe tool">tabletop link</a>{/if}
            {/each}
          </span>
          <span class="live" title="Live: sent to this seat's screens now, never saved">
            {#each LIVE_OVERRIDES as o (o.param)}
              <label class="check"><input type="checkbox" data-live={`${s.id}:${o.param}`} checked={s.live?.[o.param] === o.on} onchange={(e) => { const { [o.param]: _x, ...rest } = s.live ?? {}; share.setLive(s.id, e.currentTarget.checked ? { ...rest, [o.param]: o.on } : rest); }} /> {o.label}</label>
            {/each}
          </span>
          {#if share.state.link}
            <!-- Looks 4D seat rows, first cut: send one seat the view, or blank it; "Send this view" in the footer sends everyone. -->
            <span class="sees" data-seat-sees={s.id}>
              Sees: {seesWords(s.shown ?? share.state.shown)}{s.shown ? "" : " (with the table)"}
              {#if here && JSON.stringify(s.shown ?? share.state.shown) !== JSON.stringify(here())}<button type="button" class="linkbtn" data-seat-send={s.id} onclick={() => share.showSeat(s.id, here!())}>Send this view</button>{/if}
              {#if !(s.shown && "off" in s.shown)}<button type="button" class="linkbtn quiet" data-seat-off={s.id} onclick={() => share.showSeat(s.id, { off: true })}>Screens off</button>{/if}
              {#if s.shown}<button type="button" class="linkbtn quiet" data-seat-rejoin={s.id} onclick={() => share.showSeat(s.id, null)}>Back with the table</button>{/if}
            </span>
          {/if}
          {#if s.id !== "table"}<button type="button" class="linkbtn" onclick={() => share.removeSeat(s.id)}>Remove</button>{/if}
        </li>
      {/each}
    </ul>
  {/if}

  {#if foggy.length}
    <h3>Discoveries</h3>
    <p class="hint">Tick what each seat has found. A planet brings its moons. Each tick is an event, so the history says when they learned it.</p>
    <table class="disc">
      <thead>
        <tr><th scope="col">Body</th>{#each foggy as a (a.id)}<th scope="col" title={seatFor(a.id)}>{a.label}</th>{/each}</tr>
      </thead>
      <tbody>
        {#each bodies as b (b.address)}
          <tr>
            <th scope="row">{b.label}</th>
            {#each foggy as a (a.id)}
              <td><input type="checkbox" aria-label="{b.label} known to {a.label}" checked={known[a.id]?.includes(b.address) ?? false} onchange={(e) => toggle(a, b.address, e.currentTarget.checked)} /></td>
            {/each}
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  <form class="add" onsubmit={add}>
    <h3>{single ? "More seats" : "Add a seat"}</h3>
    <label>Name<input bind:value={label} placeholder="Empire: Vesh, Sensors, Wall screen" /></label>
    <label class="check"><input type="checkbox" bind:checked={withFog} disabled={!fog} /> Fog of war: starts knowing nothing until you mark discoveries</label>
    {#if !fog}<p class="hint">The campaign's data views have none that shows nothing, so fog of war is not available.</p>{/if}
    <label class="check"><input type="checkbox" bind:checked={names} /> May suggest names</label>
    <label class="check"><input type="checkbox" bind:checked={autoNames} disabled={!names} /> Its names apply without asking</label>
    <label class="check"><input type="checkbox" bind:checked={sealed} /> Sealed: its screens get finished scenes only, never the seed (needs you online)</label>
    <label>Brand on its screens <input bind:value={brandName} maxlength="40" placeholder="A faction or ship name (optional)" /></label>
    {#if brandName.trim()}<label class="check">Logo <input type="file" accept="image/png,image/jpeg,image/webp" onchange={readLogo} /></label>{/if}
    <label>Tabletops it may sit inside <input bind:value={tabletops} placeholder="https://tabletop.example.com" /></label>
    <p class="hint">The exact web address of each virtual tabletop, separated by spaces. Its tabletop link then shows the view there with nothing else round it, and refuses any other site.</p>
    <fieldset>
      <legend>Screens (each opens in its own window or device)</legend>
      {#each SCREENS as sc (sc.file)}
        <label class="check"><input type="checkbox" value={sc.file} bind:group={screens} /> {sc.label}</label>
      {/each}
    </fieldset>
    <button type="submit">Add seat</button>
    {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
  </form>
</section>

<style>
  .seats {
    font-family: var(--sse-font-body);
    font-size: 14px;
    color: var(--sse-ink);
  }
  h2,
  h3 {
    margin: 0 0 10px;
    font-family: var(--sse-font-display);
    font-weight: normal;
  }
  h3 {
    margin-top: 18px;
    font-size: 17px;
  }
  .one b {
    font-weight: 600;
  }
  .list {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .list li {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px 12px;
    padding: 8px 0;
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .name {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 16px;
  }
  .sees {
    grid-column: 1 / -1;
    font-size: 13px;
    display: flex;
    flex-wrap: wrap;
    gap: 4px 10px;
    align-items: baseline;
  }
  .meta,
  .hint {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .links {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }
  .links a,
  .linkbtn {
    color: var(--sse-accent);
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    text-decoration: underline;
  }
  .disc {
    border-collapse: collapse;
    font-size: 13px;
  }
  .disc th,
  .disc td {
    padding: 4px 10px 4px 0;
    text-align: left;
    font-weight: normal;
  }
  .disc thead th {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .add {
    display: grid;
    gap: 8px;
  }
  .add input:not([type]) {
    display: block;
    width: 100%;
    margin-top: 4px;
    min-height: 36px;
    padding: 0 8px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
  }
  .check {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  fieldset {
    border: var(--sse-line) solid var(--sse-rule);
    padding: 6px 10px 8px;
  }
  legend {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  button[type="submit"] {
    justify-self: start;
    height: 40px;
    padding: 0 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    font-family: var(--sse-font-body);
    cursor: pointer;
  }
  .problem {
    color: var(--sse-accent-ink);
    margin: 0;
  }
</style>
