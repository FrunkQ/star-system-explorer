<!--
  The role card (PB7, design 7.3): opens from a role's heading on the Switchboard, or from "+ Add a role", and holds
  what the old Seats dialog held, in order of use: the name; what it knows (its own knowledge, starting with nothing
  for fog of war, or what another role knows); the one layout its screens open; its links; and under More, what it
  may ask for, a brand, tabletops, sealed (when it is made), the quick switches, its discoveries and tonight's
  redirect. Look and sensors come with looks (PB10) and bands on players' screens; the card says so.
-->
<script lang="ts">
  import { EVERYONE, LIVE_OVERRIDES, checkBrand, embedOrigins, redirect, type Board, type SeatBrand, type SeatPort, type TablePort } from "../../../ui-contract/src/index.ts";

  let { table, seats, role, onclose }: { table: TablePort; seats: SeatPort; role: string | null; onclose: (made?: string) => void } = $props();

  const LAYOUTS = [
    { file: "", label: "Bridge (the players' chart)" },
    { file: "players.survey.screen.json", label: "Tarn survey report" },
    { file: "players.bridge-3d.screen.json", label: "Bridge in 3D" },
  ];

  const plan = $derived(seats.plan());
  const seat = $derived(role ? plan.seats.find((s) => s.id === role) : undefined);
  const making = $derived(role === null);
  const fog = $derived(seats.fog());
  const board = $derived(table.state.board as Board);
  const others = $derived(plan.seats.filter((s) => s.authority !== "gm" && s.id !== role));
  const labelOf = (id: string) => (id === "table" ? "Table" : (plan.seats.find((s) => s.id === id)?.label ?? id));

  // A new role's fields.
  let label = $state("");
  let knows = $state<"fog" | "own" | string>("fog");
  let layout = $state("");
  let sealed = $state(false);
  let problem = $state("");
  // More, closed by default.
  let more = $state(false);
  let names = $state(true);
  let autoNames = $state(false);
  let brandName = $state("");
  let brandLogo = $state<string | undefined>(undefined);
  let brandAccent = $state("");
  let brandMotto = $state("");
  let tabletops = $state("");
  $effect(() => {
    if (!seat) return;
    names = seat.may_send.includes("set_value");
    autoNames = !!seat.auto_approve?.length;
    brandName = seat.brand?.name ?? "";
    brandLogo = seat.brand?.logo;
    brandAccent = seat.brand?.accent ?? "";
    brandMotto = seat.brand?.motto ?? "";
    tabletops = (seat.embed_origins ?? []).join(" ");
  });
  $effect(() => {
    if (making && !fog && knows === "fog") knows = "own";
  });

  function readLogo(e: Event) {
    const f = (e.currentTarget as HTMLInputElement).files?.[0];
    if (!f) return void (brandLogo = undefined);
    const r = new FileReader();
    r.onload = () => ((brandLogo = String(r.result)), saveBrand());
    r.readAsDataURL(f);
  }
  function brandOf(): { brand?: SeatBrand; bad: string[] } {
    const brand = brandName.trim() ? { name: brandName.trim(), ...(brandLogo ? { logo: brandLogo } : {}), ...(brandAccent ? { accent: brandAccent } : {}), ...(brandMotto.trim() ? { motto: brandMotto.trim() } : {}) } : undefined;
    return { ...(brand ? { brand } : {}), bad: brand ? checkBrand(brand) : [] };
  }
  function origins(): { list: string[]; bad: string[] } {
    const vtt = embedOrigins(tabletops.split(/[\s,]+/));
    return { list: vtt.origins, bad: vtt.problems };
  }
  function saveBrand() {
    if (!seat) return;
    const b = brandOf();
    if (b.bad.length) return void (problem = `Brand: ${b.bad.join("; ")}.`);
    problem = "";
    seats.update(seat.id, { brand: b.brand ?? null });
  }
  function saveTabletops() {
    if (!seat) return;
    const o = origins();
    if (o.bad.length) return void (problem = `Tabletop addresses: ${o.bad.join("; ")}.`);
    problem = "";
    seats.update(seat.id, { embedOrigins: o.list });
  }

  async function make(e: SubmitEvent) {
    e.preventDefault();
    if (!label.trim()) return void (problem = "Give the role a name, for example Sensors, Empire: Vesh or Wall screen.");
    const o = origins();
    if (o.bad.length) return void (problem = `Tabletop addresses: ${o.bad.join("; ")}.`);
    const b = brandOf();
    if (b.bad.length) return void (problem = `Brand: ${b.bad.join("; ")}.`);
    const scope = table.live();
    const bad = await seats.add({
      label: label.trim(),
      ...(knows === "fog" && fog ? { fog } : {}),
      ...(knows !== "fog" && knows !== "own" ? { audience: knows } : {}),
      names,
      autoNames: autoNames && "scope" in scope ? [scope.scope] : [],
      screens: layout ? [layout] : [],
      sealed,
      embedOrigins: o.list,
      ...(b.brand ? { brand: b.brand } : {}),
    });
    problem = bad ?? "";
    if (!bad) onclose(seats.plan().seats.at(-1)?.id);
  }

  // Discoveries of a fog-of-war role, read back from the kernel's scene for its audience.
  const foggy = $derived(seat ? plan.audiences.find((a) => a.id === seat.audience && a.id !== EVERYONE && a.starts_with) : undefined);
  let found = $state<string[]>([]);
  $effect(() => {
    const a = foggy?.id;
    if (a) void seats.discovered(a).then((x) => (found = x));
  });
  async function mark(address: string, on: boolean) {
    if (!foggy) return;
    problem = (await seats.discover(foggy.id, address, on)) ?? "";
    found = await seats.discovered(foggy.id);
  }

  const knowsWords = $derived(!seat ? "" : seat.audience === EVERYONE ? "what everyone has been shown" : seat.audience === seat.id ? (foggy ? "its own discoveries (fog of war)" : "its own knowledge") : `what ${plan.audiences.find((a) => a.id === seat.audience)?.label ?? seat.audience} knows`);
  const requests = $derived(seat ? seats.requests().filter((r) => r.seat === seat.id) : []);
  const key = (e: KeyboardEvent) => e.key === "Escape" && (e.stopPropagation(), onclose());
</script>

<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
<section class="card" role="group" aria-label={making ? "Add a role" : `Role: ${seat?.label ?? role}`} data-role-card={role ?? "new"} onkeydown={key}>
  <header>
    <b>{making ? "Add a role" : role === "table" ? "Table" : seat?.label}</b>
    <button type="button" class="quiet" aria-label="Close" data-role-card-close onclick={() => onclose()}>×</button>
  </header>

  {#if making}
    <form onsubmit={make}>
      <label>Name <input bind:value={label} placeholder="Sensors, Empire: Vesh, Wall screen" data-role-name /></label>
      <label
        >Knows
        <select bind:value={knows} data-role-knows>
          {#if fog}<option value="fog">Its own: nothing until you mark discoveries (fog of war)</option>{/if}
          <option value="own">Its own knowledge</option>
          <option value={EVERYONE}>What everyone has been shown</option>
          {#each plan.audiences.filter((a) => a.id !== EVERYONE) as a (a.id)}<option value={a.id}>What {a.label} knows</option>{/each}
        </select>
      </label>
      {#if !fog}<p class="hint">The campaign's data views have none that shows nothing, so fog of war is not on offer.</p>{/if}
      <label
        >Layout
        <select bind:value={layout} data-role-layout>
          {#each LAYOUTS as l (l.file)}<option value={l.file}>{l.label}</option>{/each}
        </select>
      </label>
      <label class="check"><input type="checkbox" bind:checked={sealed} data-role-sealed /> Sealed: its screens get finished scenes only, never the seed (needs you online)</label>
      <details bind:open={more}>
        <summary>More</summary>
        {@render moreFields()}
      </details>
      <button type="submit" class="pri" data-role-add>Add role</button>
    </form>
  {:else if seat}
    <div class="rows">
      <label>Name <input value={seat.label} disabled={seat.id === "table"} data-role-name onchange={(e) => seats.update(seat.id, { label: e.currentTarget.value })} /></label>
      <p class="row"><span class="k">Knows</span> <span data-role-knows>{knowsWords}</span>{#if seat.sealed}<span class="sub"> · sealed</span>{/if}</p>
      <label
        >Layout
        <select value={seat.screens?.[0] ?? ""} data-role-layout onchange={(e) => seats.update(seat.id, { screen: e.currentTarget.value || null })}>
          {#each LAYOUTS as l (l.file)}<option value={l.file}>{l.label}</option>{/each}
          {#if seat.screens?.[0] && !LAYOUTS.some((l) => l.file === seat.screens?.[0])}<option value={seat.screens[0]}>{seat.screens[0]}</option>{/if}
        </select>
      </label>
      <p class="row sub">Look and sensors: a role's look comes with looks on the board; its bands with sensors.</p>
      {#if table.state.link}
        <p class="row links">
          <span class="k">Links</span>
          {#each table.links(seat.id) as l (l.href)}
            <a href={l.href} target="_blank" rel="noopener" data-seat-link={seat.id}>join link</a>
            {#if seat.embed_origins?.length}<a href={`${l.href}&embed=1`} target="_blank" rel="noopener" data-seat-embed={seat.id} title="Paste this into the tabletop's web or iframe tool">tabletop link</a>{/if}
          {/each}
        </p>
      {/if}
      {#each requests as r (r.from + r.id)}
        <p class="row req" data-request={r.id}>
          Asks: {r.words}
          <button type="button" onclick={() => seats.answer(r.id, r.from, true)}>Approve</button>
          <button type="button" class="quiet" onclick={() => seats.answer(r.id, r.from, false)}>Decline</button>
        </p>
      {/each}
      <details bind:open={more}>
        <summary>More</summary>
        {@render moreFields()}
        <div class="live" title="Live: sent to this role's screens now, never saved">
          {#each LIVE_OVERRIDES as o (o.param)}
            <label class="check"
              ><input
                type="checkbox"
                data-live={`${seat.id}:${o.param}`}
                checked={seat.live?.[o.param] === o.on}
                onchange={(e) => {
                  const { [o.param]: _x, ...rest } = seat.live ?? {};
                  seats.setLive(seat.id, e.currentTarget.checked ? { ...rest, [o.param]: o.on } : rest);
                }}
              />
              {o.label}</label
            >
          {/each}
        </div>
        <label
          >Tonight its sends go to
          <select value={board.redirects[seat.id] ?? ""} data-role-redirect onchange={(e) => table.act((b) => redirect(b, seat.id, e.currentTarget.value || null))}>
            <option value="">its own screens</option>
            {#each others as o (o.id)}<option value={o.id}>{labelOf(o.id)}'s screens</option>{/each}
          </select>
        </label>
        {#if foggy}
          <fieldset class="disc">
            <legend>Discoveries</legend>
            <p class="hint">Tick what {seat.label} has found. A planet brings its moons. Each tick is an event, so the history says when they learned it.</p>
            {#each seats.bodies() as b (b.address)}
              <label class="check"><input type="checkbox" aria-label="{b.label} known to {seat.label}" checked={found.includes(b.address)} onchange={(e) => mark(b.address, e.currentTarget.checked)} /> {b.label}</label>
            {/each}
          </fieldset>
        {/if}
        {#if seat.id !== "table"}<button type="button" class="quiet danger" data-role-remove onclick={() => (seats.remove(seat.id), onclose())}>Remove this role</button>{/if}
      </details>
    </div>
  {/if}
  {#if problem}<p class="problem" role="alert">{problem}</p>{/if}
</section>

{#snippet moreFields()}
  <label class="check"><input type="checkbox" bind:checked={names} data-role-names onchange={() => seat && seats.update(seat.id, { names })} /> May suggest names</label>
  <label class="check"><input type="checkbox" bind:checked={autoNames} disabled={!names} onchange={() => { const sc = table.live(); if (seat) seats.update(seat.id, { autoNames: autoNames && "scope" in sc ? [sc.scope] : [] }); }} /> Its names apply without asking</label>
  <label>Brand on its screens <input bind:value={brandName} maxlength="40" placeholder="A faction or ship name (optional)" data-role-brand onchange={saveBrand} /></label>
  {#if brandName.trim()}
    <label class="check">Logo <input type="file" accept="image/png,image/jpeg,image/webp" onchange={readLogo} /></label>
    <label>Motto, one line under the name <input bind:value={brandMotto} maxlength="60" placeholder="Moving cargo since 2240" data-role-motto onchange={saveBrand} /></label>
    <label class="check">Accent <input type="color" value={brandAccent || "#b5462b"} data-role-accent onchange={(e) => ((brandAccent = e.currentTarget.value), saveBrand())} /> <span class="sub">for rules and stamps on its letterhead</span></label>
  {/if}
  <label>Tabletops it may sit inside <input bind:value={tabletops} placeholder="https://tabletop.example.com" data-role-tabletops onchange={saveTabletops} /></label>
  <p class="hint">The exact web address of each virtual tabletop, separated by spaces. Its tabletop link then shows the view there with nothing else round it, and refuses any other site.</p>
{/snippet}

<style>
  .card {
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    box-shadow: var(--sse-card-shadow);
    padding: 0 12px 12px;
    font-size: 13px;
    animation: grow var(--sse-motion) var(--sse-ease);
    transform-origin: top left;
  }
  @keyframes grow {
    from {
      opacity: 0;
      transform: scale(0.97);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .card {
      animation: none;
    }
  }
  header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 0;
    font-family: var(--sse-font-display);
    font-size: 16px;
  }
  form,
  .rows {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 2px;
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  label.check {
    flex-direction: row;
    align-items: center;
    gap: 6px;
    color: var(--sse-ink);
    font-size: 13px;
  }
  .row {
    margin: 0;
  }
  .k {
    color: var(--sse-ink-muted);
    font-size: 12px;
    margin-right: 6px;
  }
  .sub,
  .hint {
    color: var(--sse-ink-muted);
    font-size: 12px;
    margin: 0;
  }
  .links a {
    margin-right: 8px;
    color: var(--sse-ink);
  }
  .req {
    padding: 4px 6px;
    border: var(--sse-line) solid var(--sse-accent);
    border-radius: var(--sse-radius);
  }
  details {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  details[open] > :global(*) + :global(*) {
    margin-top: 8px;
  }
  summary {
    cursor: pointer;
    color: var(--sse-ink-muted);
  }
  .disc {
    border: var(--sse-line) solid var(--sse-rule);
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  input,
  select,
  button {
    font: inherit;
    font-size: 13px;
    color: var(--sse-ink);
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    padding: 3px 8px;
  }
  input[type="checkbox"] {
    padding: 0;
  }
  button {
    cursor: pointer;
    align-self: flex-start;
  }
  button.pri {
    background: var(--sse-accent);
    border-color: var(--sse-accent);
    color: var(--sse-on-ink);
  }
  button.quiet {
    border-color: var(--sse-rule);
    color: var(--sse-ink-muted);
    background: transparent;
  }
  .danger:hover {
    color: var(--sse-accent);
  }
  .problem {
    color: var(--sse-accent);
    margin: 8px 0 0;
  }
</style>
