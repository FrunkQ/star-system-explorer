<!--
  The Reference screen (matter-pack design section 8, MP10 part 1): the master data sheet as the stack has it, read
  through the kernel's read_matter. Constants in the sheet's own groups (nature, conventions, factors, cosmos), with
  the two numbers no change of units can move at the top; Units under "This setting". Every row opens a card with
  its sources and how it is known. A row the sheet marks theoretical says so, with its note, at every depth.
  Read only: editing arrives with MP9.
-->
<script lang="ts">
  import { tick, untrack } from "svelte";
  import { findInSheet, readElementsPage, type ElementRow, type RefHit, dimensionless, formatUnit, formatValue, measureWords, readConstantsPage, readRow, readSource, rowNote, showReference, siUnits, sourceName, unitFactor, type ConstantRow, type ConstantsPage, type Json, type RowDetail, type SheetSource, type SystemScene, type UnitRow, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { Stamp } from "../../../chart-grammar/src/index.ts";
  import Elements from "./Elements.svelte";
  import Names from "./Names.svelte";

  let { ctx, params }: { ctx: ViewContext; scene: SystemScene; params: Record<string, Json> } = $props();
  const depth = $derived(ctx.workspace.depth ?? "science");
  const full = $derived(depth !== "curious");

  type PageId = "constants" | "elements" | "units" | "names";
  const PAGES: { group: string; pages: { id: PageId; label: string }[] }[] = [
    { group: "Nature", pages: [{ id: "constants", label: "Constants" }, { id: "elements", label: "Elements" }] },
    { group: "Names and classes", pages: [{ id: "names", label: "Names" }] },
    { group: "This setting", pages: [{ id: "units", label: "Units" }] },
  ];
  let page = $state<PageId>(untrack(() => (params.page === "units" || params.page === "elements" || params.page === "names" ? params.page : "constants")));

  let sheet = $state.raw<ConstantsPage | null>(null);
  let failed = $state<string | null>(null);
  $effect(() => {
    const s = ctx.session;
    untrack(() => {
      void readConstantsPage(s).then((r) => (r.ok ? (sheet = r.page) : (failed = r.message)));
    });
  });
  // The group notes name their sources by id; each is looked up once, and reads as "Clowe 2006" until it is.
  let cited = $state.raw<Map<string, SheetSource>>(new Map());
  $effect(() => {
    const ids = [...new Set((sheet?.groups ?? []).flatMap((g) => (g.notes ?? []).flatMap((n) => [n.src, ...(n.also ?? [])])))];
    const s = ctx.session;
    untrack(() => {
      void Promise.all(ids.map((id) => readSource(s, id))).then((found) => (cited = new Map(found.filter((x): x is SheetSource => x !== null).map((x) => [x.id, x]))));
    });
  });
  const values = $derived(new Map((sheet?.rows ?? []).map((r) => [r.id, r.value.v])));
  const pure = $derived(dimensionless(values));
  const si = $derived(siUnits(sheet?.units ?? []));
  // A worked-out row or factor carries the float's noise past ten figures; the sheet's own numbers do not.
  const shown = (r: ConstantRow) => (r.derived_from ? Number(r.value.v.toPrecision(10)) : r.value.v);
  // A factor worked out from constants carries the float's noise past ten figures; the sheet's own numbers do not.
  const shownFactor = (u: UnitRow) => {
    const f = unitFactor(u, values);
    return f === null ? null : u.factor_from ? Number(f.toPrecision(10)) : f;
  };
  // Measures in the order a reader reaches for them, then the rest by name; in each, the SI unit first, then by size.
  const FIRST = ["length", "mass", "time", "temperature", "velocity", "acceleration", "force", "energy", "power", "pressure", "density"];
  const measures = $derived.by(() => {
    const out = new Map<string, UnitRow[]>();
    for (const u of sheet?.units ?? []) out.set(u.measures, [...(out.get(u.measures) ?? []), u]);
    const at = (m: string) => (FIRST.includes(m) ? FIRST.indexOf(m) : FIRST.length);
    return [...out]
      .sort(([a], [b]) => at(a) - at(b) || a.localeCompare(b))
      .map(([m, us]) => [m, [...us].sort((a, b) => Number(si.get(m)?.id !== a.id) - Number(si.get(m)?.id !== b.id) || (shownFactor(a) ?? 0) - (shownFactor(b) ?? 0))] as const);
  });

  // The card: one row, read alone for its sources and readers.
  type Open = { table: "constants"; row: ConstantRow } | { table: "units"; row: UnitRow };
  let open = $state.raw<Open | null>(null);
  let detail = $state.raw<RowDetail<unknown> | null>(null);
  let opener: HTMLElement | null = null;
  async function show(o: Open, from: HTMLElement) {
    opener = from;
    open = o;
    detail = null;
    detail = await readRow(ctx.session, o.table, o.row.id);
  }
  async function close() {
    open = null;
    await tick();
    opener?.focus();
  }
  const sentence = (s: string) => (s ? s[0]!.toUpperCase() + s.slice(1) : s);

  // Find: one box for an element, a constant or a unit, so looking one up is a word, not three presses. On a desk
  // with a mouse it takes the typing straight away; on a touch screen it waits to be pressed (no keyboard pops up).
  let query = $state("");
  let elements = $state.raw<ElementRow[]>([]);
  $effect(() => {
    const s = ctx.session;
    untrack(() => void readElementsPage(s).then((r) => r.ok && (elements = r.page.elements)));
  });
  const hits = $derived(findInSheet(query, { elements, constants: sheet?.rows ?? [], units: sheet?.units ?? [] }));
  let pick = $state<string | null>(null);
  let findBox: HTMLInputElement | null = $state(null);
  $effect(() => {
    if (findBox && matchMedia("(pointer: fine)").matches) untrack(() => findBox?.focus({ preventScroll: true }));
  });
  function go(h: RefHit) {
    query = "";
    if (h.table === "elements") {
      open = null;
      page = "elements";
      pick = null;
      void tick().then(() => (pick = h.id));
      return;
    }
    const row = h.table === "constants" ? sheet?.rows.find((r) => r.id === h.id) : sheet?.units.find((u) => u.id === h.id);
    if (!row || !findBox) return;
    page = h.table;
    void show(h.table === "constants" ? { table: "constants", row: row as ConstantRow } : { table: "units", row: row as UnitRow }, findBox);
  }
</script>

<!-- Escape closes the card first, wherever focus is; only with no card open does it reach the shell (which goes up a step). -->
<svelte:window
  onkeydowncapture={(e) => {
    if (open && e.key === "Escape") {
      e.stopPropagation();
      e.preventDefault();
      void close();
    }
  }}
/>

<section
  class="reference"
  aria-label="Reference"
  data-reference={sheet ? page : failed ? "failed" : "loading"} data-depth={depth}>
  <header>
    <div class="title">
      <h1>Reference</h1>
      <p class="lede">{page === "units" ? "How this setting counts and shows things. Nothing here changes the physics." : page === "names" ? "What this campaign calls things. Names never change the physics." : "What nature gives the simulation. Real unless marked."}</p>
    </div>
    <div class="find" role="search">
      <input
        type="search"
        bind:this={findBox}
        bind:value={query}
        placeholder="Find an element, constant or unit"
        aria-label="Find in Look up"
        data-ref-find
        autocomplete="off"
        onkeydown={(e) => {
          if (e.key === "Enter" && hits[0]) (e.preventDefault(), go(hits[0]));
          if (e.key === "Escape" && query) (e.stopPropagation(), (query = ""));
        }}
      />
      {#if query.trim()}
        <ul class="hits" aria-label="Found">
          {#each hits as h (h.table + h.id)}
            <li><button type="button" data-ref-hit={`${h.table}:${h.id}`} onclick={() => go(h)}>{h.label} <small>{h.hint}</small></button></li>
          {:else}
            <li class="none">Nothing in the sheet is called that.</li>
          {/each}
        </ul>
      {/if}
    </div>
    <nav class="pages" aria-label="Reference pages">
      {#each PAGES as g (g.group)}
        <span class="k">{g.group}</span>
        {#each g.pages as p (p.id)}
          <button type="button" aria-current={page === p.id ? "page" : undefined} data-page={p.id} onclick={() => ((page = p.id), (open = null))}>{p.label}</button>
        {/each}
      {/each}
    </nav>
  </header>

  {#if page === "elements"}
    <div class="body"><Elements session={ctx.session} {depth} {pick} /></div>
  {:else if page === "names"}
    <div class="body"><Names session={ctx.session} workspace={ctx.workspace} {depth} offer={typeof params.names_offer === "string" ? params.names_offer : ""} /></div>
  {:else if failed}
    <p class="note" data-reference-error>Could not read the data sheet: {failed}</p>
  {:else if !sheet}
    <p class="note" aria-busy="true">Reading the data sheet…</p>
  {:else}
    <div class="body" class:with-card={open}>
      <div class="pagebody">
        {#if page === "constants"}
          {#if pure.length}
            <div class="pure" data-dimensionless>
              <p class="note">A change of units is not a change of physics. A change in these numbers is.</p>
              {#each pure as d (d.id)}<div class="num"><span class="k2">{d.name}</span><span class="v">{d.shown}</span></div>{/each}
            </div>
          {/if}
          {#each sheet.groups as g (g.id)}
            {@const rows = sheet.rows.filter((r) => r.group === g.id)}
            {#if rows.length}
              <section class="group" aria-labelledby="g-{g.id}" data-group={g.id}>
                <h2 id="g-{g.id}">{g.name}</h2>
                {#if g.about}<p class="about">{g.about}</p>{/if}
                <ul class="rows">
                  {#each rows as r (r.id)}
                    <li class:marked={r.mark} data-row={r.id} data-mark={r.mark}>
                      <button type="button" class="row" aria-expanded={open?.row.id === r.id} onclick={(e) => show({ table: "constants", row: r }, e.currentTarget)}>
                        <span class="name">{sentence(r.name)}</span>
                        <span class="val">{formatValue(shown(r), full)} <span class="unit">{formatUnit(r.value.unit)}</span></span>
                        {#if r.mark}<span class="mark"><Stamp kind={r.mark} tilt={false} /></span>{:else if full}<span class="how">{rowNote(r)}</span>{/if}
                        {#if depth === "engine"}<code class="id">{r.id}</code>{/if}
                      </button>
                      {#if r.mark_note}<p class="mark-note" data-mark-note>{r.mark_note}</p>{/if}
                    </li>
                  {/each}
                </ul>
                {#if g.notes?.length}
                  <ul class="group-notes" aria-label="Notes on {g.name}" data-group-notes>
                    {#each g.notes as n, i (i)}
                      <li>{n.text}{#if full}<span class="cite"> ({#each [n.src, ...(n.also ?? [])] as id, j (id)}{@const c = cited.get(id)}{@const ref = c ? showReference(c) : null}{j ? "; " : ""}{#if ref?.href}<a href={ref.href} target="_blank" rel="noopener" title={c?.text}>{sourceName(id)}</a>{:else}<span title={c?.text}>{sourceName(id)}</span>{/if}{/each})</span>{/if}</li>
                    {/each}
                  </ul>
                {/if}
              </section>
            {/if}
          {/each}
        {:else}
          <p class="note">Every unit the sheet knows, each against the SI unit of what it measures. A campaign's own units and unit schemes arrive with editing.</p>
          {#each measures as [m, us] (m)}
            {@const base = si.get(m)}
            <section class="group" aria-labelledby="m-{m}" data-measure={m}>
              <h2 id="m-{m}">{sentence(measureWords(m))}</h2>
              <ul class="rows">
                {#each us as u (u.id)}
                  {@const f = shownFactor(u)}
                  <li data-unit={u.id}>
                    <button type="button" class="row" aria-expanded={open?.row.id === u.id} onclick={(e) => show({ table: "units", row: u }, e.currentTarget)}>
                      <span class="name">{sentence(u.name)}{#if u.symbol}<span class="sym">{u.symbol}</span>{/if}</span>
                      <span class="val">
                        {#if base?.id === u.id}SI{:else if f === null}{u.factor_from}{:else}1 {u.symbol || u.name} = {formatValue(f, full)} {base ? base.symbol : "in SI"}{#if u.offset}, from {formatValue(u.offset, full)}{/if}{/if}
                      </span>
                      {#if full}<span class="how">{u.factor_from ? (/^[A-Za-z_]\w*$/.test(u.factor_from) ? "a constant" : "worked out") : u.group === "working" ? "defined" : "SI"}</span>{/if}
                      {#if depth === "engine"}<code class="id">{u.id}</code>{/if}
                    </button>
                  </li>
                {/each}
              </ul>
            </section>
          {/each}
        {/if}
      </div>

      {#if open}
        <aside class="card" aria-label="About {open.row.name}" data-card={open.row.id}>
          <div class="grip" aria-hidden="true"></div>
          <div class="card-head">
            <h2>{sentence(open.row.name)}</h2>
            <button type="button" class="close" aria-label="Close" onclick={close}>×</button>
          </div>
          {#if open.table === "constants"}
            {@const r = open.row}
            <p class="big">{formatValue(shown(r))} <span class="unit">{formatUnit(r.value.unit)}</span></p>
            {#if r.mark}<div class="card-mark"><Stamp kind={r.mark} tilt={false} /><p data-card-mark-note>{r.mark_note}</p></div>{/if}
            {#if r.note}<p>{r.note}</p>{/if}
            {#if r.derived_from}<p class="how-known">Worked out from the sheet: <code>{r.derived_from}</code>. Edit a row it rests on and this follows.</p>
            {:else if r.value.exact}<p class="how-known">Exact by definition: an agreement, not a measurement.</p>
            {:else if r.value.uncertainty}<p class="how-known">Measured, ± {[formatValue(r.value.uncertainty), formatUnit(r.value.unit)].filter(Boolean).join(" ")}.</p>{/if}
            {#if full && r.value.note}<p class="muted">{r.value.note}</p>{/if}
          {:else}
            {@const u = open.row}
            {@const base = si.get(u.measures)}
            {@const f = shownFactor(u)}
            <p class="big">{#if base && f !== null && base.id !== u.id}1 {u.symbol || u.name} = {formatValue(f)} {base.symbol}{:else}{u.symbol}{/if}</p>
            <p>Measures {measureWords(u.measures)}.{#if u.factor_from} Worked out from the sheet: <code>{u.factor_from}</code>.{/if}{#if u.offset} Counted from {formatValue(u.offset)} {base?.symbol ?? ""}.{/if}</p>
          {/if}
          <div class="why" data-card-why>
            <h3>Why this number?</h3>
            {#if !detail}
              <p class="muted" aria-busy="true">Reading its sources…</p>
            {:else}
              {#if detail.sources.length}
                <ul class="sources">
                  {#each detail.sources as s (s.id)}
                    {@const ref = showReference(s)}
                    <li data-source={s.id}>{#if ref.href}<a href={ref.href} target="_blank" rel="noopener">{ref.text}</a>{:else}{ref.text}{/if}{#if full && s.status}{" "}<span class="status">({s.status})</span>{/if}</li>
                  {/each}
                </ul>
              {:else}
                <p class="muted">{open.table === "units" ? "A definition: it follows from the rows it is worked out from." : "No source on the sheet."}</p>
              {/if}
              {#if depth === "engine"}
                <dl class="engine" data-depth="engine">
                  <dt>Id</dt><dd><code>{detail.id}</code> in {detail.table}</dd>
                  <dt>Set by</dt><dd>{detail.packs.join(", ") || "—"}</dd>
                  <dt>Read by</dt><dd>{detail.used_by.join(", ") || "no block declares it yet"}</dd>
                  {#if detail.skipped_by.length}<dt>Skipped by</dt><dd>{detail.skipped_by.join(", ")}</dd>{/if}
                  {#if open.table === "constants" && open.row.value.check}<dt>Checked</dt><dd>{open.row.value.check}</dd>{/if}
                </dl>
              {/if}
            {/if}
          </div>
        </aside>
      {/if}
    </div>
  {/if}
</section>

<style>
  .reference {
    container-type: inline-size;
    padding: 18px 24px 24px;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
    min-width: 0;
  }
  header {
    display: flex;
    flex-wrap: wrap;
    align-items: flex-end;
    justify-content: space-between;
    gap: 10px 24px;
    padding-bottom: 12px;
    border-bottom: var(--sse-line) solid var(--sse-ink);
  }
  h1 {
    margin: 0;
    font-family: var(--sse-font-display);
    font-weight: 500;
    font-size: 30px;
    line-height: 1.1;
  }
  .lede,
  .note,
  .about,
  .muted {
    margin: 2px 0 0;
    color: var(--sse-ink-muted);
  }
  .find {
    position: relative;
    margin: 10px 0 4px;
    max-width: 420px;
  }
  .find input {
    box-sizing: border-box;
    width: 100%;
    height: 36px;
    padding: 0 10px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font: inherit;
  }
  .hits {
    position: absolute;
    z-index: 5;
    left: 0;
    right: 0;
    margin: 2px 0 0;
    padding: 4px 0;
    list-style: none;
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
  }
  .hits button {
    display: block;
    width: 100%;
    padding: 6px 10px;
    border: none;
    background: none;
    color: var(--sse-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }
  .hits button:hover,
  .hits li:first-child button {
    background: var(--sse-zone);
  }
  .hits small {
    color: var(--sse-ink-muted);
  }
  .hits .none {
    padding: 6px 10px;
    color: var(--sse-ink-muted);
  }
  .pages {
    display: flex;
    align-items: center;
    gap: 4px 14px;
    overflow-x: auto;
    max-width: 100%;
  }
  .pages .k {
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
    white-space: nowrap;
  }
  .pages .k + button ~ .k,
  .pages button + .k {
    border-left: var(--sse-line) solid var(--sse-ink);
    padding-left: 14px;
  }
  .pages button {
    font: inherit;
    font-size: 14px;
    border: 0;
    background: none;
    color: var(--sse-ink-muted);
    padding: 6px 2px;
    min-height: 36px;
    cursor: pointer;
    white-space: nowrap;
  }
  .pages button[aria-current="page"] {
    color: var(--sse-ink);
    font-weight: 600;
    box-shadow: inset 0 -2px 0 var(--sse-ink);
  }
  .body {
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 24px;
    margin-top: 14px;
  }
  .body.with-card {
    grid-template-columns: minmax(0, 1fr) minmax(260px, 380px);
  }
  .pure {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 28px;
    align-items: baseline;
    padding-bottom: 12px;
    border-bottom: 1px solid var(--sse-rule);
  }
  .pure .note {
    flex: 0 1 250px;
  }
  .num {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .k2 {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .num .v {
    font-family: var(--sse-font-mono);
    font-size: 16px;
  }
  .group {
    margin-top: 18px;
  }
  .group h2 {
    margin: 0;
    padding-bottom: 6px;
    border-bottom: var(--sse-line) solid var(--sse-ink);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .group .about {
    font-size: 12px;
    padding: 6px 0 2px;
  }
  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  .rows li {
    border-bottom: 1px solid var(--sse-rule);
  }
  .row {
    font: inherit;
    color: inherit;
    width: 100%;
    display: grid;
    grid-template-columns: minmax(0, 15em) minmax(0, 1fr) auto auto;
    gap: 4px 12px;
    align-items: baseline;
    text-align: left;
    border: 0;
    background: none;
    padding: 7px 8px;
    min-height: 36px;
    cursor: pointer;
  }
  .row:hover,
  .row[aria-expanded="true"] {
    background: var(--sse-ground-raised);
  }
  .row[aria-expanded="true"] {
    box-shadow: inset 2px 0 0 var(--sse-accent);
  }
  .val {
    font-family: var(--sse-font-mono);
    overflow-wrap: anywhere;
  }
  .sym {
    margin-left: 0.5em;
  }
  .unit,
  .sym,
  .how,
  .id {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .how {
    font-family: var(--sse-font-body);
    text-align: right;
  }
  .marked .row {
    background: var(--sse-ground-raised);
  }
  .mark-note {
    margin: 0;
    padding: 0 8px 8px;
    font-size: 12px;
    color: var(--sse-ink-muted);
    background: var(--sse-ground-raised);
  }
  .group-notes {
    margin: 10px 0 0;
    padding-left: 18px;
    display: grid;
    gap: 6px;
    font-size: 12px;
    line-height: 1.45;
  }
  .cite,
  .cite a {
    color: var(--sse-ink-muted);
  }
  .card {
    align-self: start;
    position: sticky;
    top: 12px;
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
    padding: 16px 18px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    animation: rise var(--sse-motion, 180ms) var(--sse-ease, ease-out);
  }
  .grip {
    display: none;
  }
  .card-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 8px;
  }
  .card h2 {
    margin: 0;
    font-family: var(--sse-font-display);
    font-weight: 500;
    font-size: 22px;
    line-height: 1.15;
  }
  .card p {
    margin: 0;
    line-height: 1.45;
  }
  .close {
    font: inherit;
    font-size: 20px;
    line-height: 1;
    border: 0;
    background: none;
    color: var(--sse-ink);
    min-width: 36px;
    min-height: 36px;
    cursor: pointer;
  }
  .big {
    font-family: var(--sse-font-mono);
    font-size: 16px;
    overflow-wrap: anywhere;
  }
  .card-mark {
    display: flex;
    flex-direction: column;
    gap: 4px;
    align-items: flex-start;
  }
  .how-known {
    color: var(--sse-ink-muted);
  }
  .why {
    border-top: 1px solid var(--sse-rule);
    padding-top: 8px;
  }
  .why h3 {
    margin: 0 0 4px;
    font-size: 12px;
    font-weight: 600;
  }
  .sources {
    margin: 0;
    padding-left: 18px;
    display: grid;
    gap: 4px;
    font-size: 12px;
  }
  .sources a {
    color: var(--sse-ink);
  }
  .status {
    color: var(--sse-ink-muted);
  }
  .engine {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 2px 10px;
    margin: 8px 0 0;
    font-size: 12px;
  }
  .engine dt {
    color: var(--sse-ink-muted);
  }
  .engine dd {
    margin: 0;
    overflow-wrap: anywhere;
  }
  @keyframes rise {
    from {
      opacity: 0;
      transform: translateY(6px);
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .card {
      animation: none;
    }
  }
  /* A phone: rows become two lines, page chips scroll sideways, the card is a sheet from the foot. */
  @container (max-width: 640px) {
    .reference {
      padding: 12px 16px 24px;
    }
    h1 {
      font-size: 24px;
    }
    .pages button {
      min-height: 44px;
    }
    .body.with-card {
      grid-template-columns: minmax(0, 1fr);
    }
    .row {
      grid-template-columns: minmax(0, 1fr) auto;
      min-height: 44px;
    }
    .row .val {
      grid-column: 1 / -1;
      grid-row: 2;
    }
    .card {
      position: fixed;
      left: 0;
      right: 0;
      bottom: 0;
      top: auto;
      max-height: 70vh;
      overflow-y: auto;
      z-index: 20;
      border-width: var(--sse-line) 0 0;
    }
    .grip {
      display: block;
      align-self: center;
      width: 40px;
      height: 3px;
      background: var(--sse-ink-muted);
    }
    .close {
      min-width: 44px;
      min-height: 44px;
    }
  }
</style>
