<!--
  World data sheet (X3a): one body's code lines, each digit with why it is
  that digit, and the rule groups as chips; or a list of many bodies. Drawn
  from the kernel's `codes` response and the pack's code sheet only.
-->
<script lang="ts">
  import { bodyRows, chipGroups, lineRows, segmentRows, textList, textSheet, type WorldCodesInput } from "./sheet.ts";

  let { scene, params, onpick }: { scene: WorldCodesInput; params: Record<string, unknown>; onpick?: (address: string) => void } = $props();

  const readings = $derived(scene.response.readings);
  const tier = $derived(String(params.tier ?? "canvas"));
  const mode = $derived(String(params.mode ?? "auto"));
  // Picked here: an address, or null for "back to the list"; undefined until a pick.
  let picked = $state<string | null | undefined>(undefined);
  // The body whose sheet is open: picked here, else given, else the only one.
  const current = $derived.by((): string | null => {
    const has = (a: string | null | undefined): a is string => !!a && readings.some((r) => r.address === a);
    if (has(picked)) return picked;
    if (picked === null && mode === "auto") return null;
    if (has(scene.selected)) return scene.selected;
    if (mode === "sheet" || readings.length === 1) return readings[0]?.address ?? null;
    return null;
  });
  const showList = $derived(mode === "list" || (mode === "auto" && current === null && readings.length > 1));
  const reading = $derived(readings.find((r) => r.address === current)?.reading ?? null);
  const name = (a: string) => scene.names?.[a] ?? a;
  const lines = $derived(reading ? lineRows(reading, scene.sheet) : []);
  const rows = $derived(reading ? segmentRows(reading, scene.sheet, scene.classes, scene.names) : []);
  const groups = $derived(reading ? chipGroups(reading, scene.sheet, scene.vocabulary) : []);
  const list = $derived(bodyRows(scene));
  const groupCols = $derived((scene.sheet.groups ?? []).filter((g) => list.some((b) => b.groups.some((x) => x.id === g.id && x.codes.length))));
  const layoutIds = $derived(Object.keys(scene.sheet.layouts));
  let open = $state<string | null>(null);

  function pick(a: string | null) {
    picked = a;
    open = null;
    if (a) onpick?.(a);
  }
</script>

<section class="wc" aria-label={scene.title ?? scene.sheet.label ?? "World codes"}>
  {#if readings.length === 0}
    <p class="empty">No bodies were read through {scene.sheet.label || scene.sheet.id}.</p>
  {:else if tier === "text"}
    {#if showList}
      <ul class="text">{#each textList(scene) as t, i (i)}<li>{t}</li>{/each}</ul>
    {:else if current}
      <ul class="text">{#each textSheet(scene, current) as t, i (i)}<li>{t}</li>{/each}</ul>
    {/if}
  {:else if showList}
    <table class="list">
      <caption>{scene.title ?? scene.sheet.label ?? scene.sheet.id}</caption>
      <thead>
        <tr>
          <th scope="col">Body</th>
          {#each layoutIds as id (id)}<th scope="col">{id.toUpperCase()}</th>{/each}
          {#each groupCols as g (g.id)}<th scope="col">{g.label || g.id}</th>{/each}
        </tr>
      </thead>
      <tbody>
        {#each list as b (b.address)}
          <tr>
            <th scope="row">
              <button type="button" class="pick" onclick={() => pick(b.address)} aria-label="Open the sheet for {b.name}">{b.name}</button>
              {#if b.disagreements}<span class="differs" title="Hand-set digits that differ from physics">≠ {b.disagreements}</span>{/if}
            </th>
            {#each layoutIds as id (id)}<td class="code">{b.lines.find((l) => l.id === id)?.text ?? ""}</td>{/each}
            {#each groupCols as g (g.id)}<td>{b.groups.find((x) => x.id === g.id)?.codes.join(" ") ?? ""}</td>{/each}
          </tr>
        {/each}
      </tbody>
    </table>
  {:else if reading && current}
    <header>
      {#if readings.length > 1}
        {#if mode === "auto"}
          <button type="button" class="back" onclick={() => pick(null)}>All bodies</button>
        {:else}
          <label class="body-pick">Body
            <select value={current} onchange={(e) => pick((e.currentTarget as HTMLSelectElement).value)}>
              {#each readings as r (r.address)}<option value={r.address}>{name(r.address)}</option>{/each}
            </select>
          </label>
        {/if}
      {/if}
      <h3>{name(current)}</h3>
      <p class="sheet-name">{scene.sheet.label || scene.sheet.id}</p>
    </header>

    <dl class="lines">
      {#each lines as l (l.id)}
        <div class="line"><dt>{l.label}</dt><dd class="code">{l.text}</dd></div>
      {/each}
    </dl>

    <table class="segments">
      <thead><tr><th scope="col">Digit</th><th scope="col">Code</th><th scope="col">Why</th></tr></thead>
      <tbody>
        {#each rows as s (s.id)}
          <tr class:disagree={s.disagrees} class="k-{s.kind}">
            <th scope="row">{s.label}</th>
            <td class="code sym">
              {s.symbol}
              {#if s.kind === "authored"}<span class="mark" title="Set by hand">set</span>{/if}
              {#if s.disagrees}<span class="mark differs" title="Physics says {s.physics}">≠ {s.physics}</span>{/if}
            </td>
            <td class="why">{s.why}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    {#each groups as g (g.id)}
      <div class="group">
        <h4 id="wc-g-{g.id}">{g.label}</h4>
        <ul class="chips" aria-labelledby="wc-g-{g.id}">
          {#each g.chips as c (c.code)}
            {@const key = `${g.id}:${c.code}`}
            <li>
              <button type="button" class="chip" title={c.title} aria-expanded={open === key} onclick={() => (open = open === key ? null : key)}>
                <span class="code">{c.code}</span>{#if c.words}<span class="words">{c.words}</span>{/if}
              </button>
            </li>
          {/each}
        </ul>
        {#each g.chips as c (c.code)}
          {#if open === `${g.id}:${c.code}`}<p class="detail" role="note">{c.title}</p>{/if}
        {/each}
      </div>
    {/each}
  {/if}
</section>

<style>
  .wc {
    font-family: var(--sse-font-body);
    color: var(--sse-ink);
    background: var(--sse-ground);
    padding: 12px 16px;
  }
  .code {
    font-family: var(--sse-font-mono);
  }
  header h3 {
    font-family: var(--sse-font-display);
    margin: 4px 0 0;
  }
  .sheet-name,
  .empty {
    color: var(--sse-ink-muted);
    margin: 2px 0 10px;
  }
  .lines {
    display: flex;
    flex-wrap: wrap;
    gap: 8px 28px;
    margin: 0 0 14px;
  }
  .line dt {
    color: var(--sse-ink-muted);
    font-size: 0.8em;
    letter-spacing: var(--sse-label-tracking, 0.06em);
  }
  .line dd {
    margin: 0;
    font-size: 2em;
    letter-spacing: 0.08em;
  }
  table {
    border-collapse: collapse;
    width: 100%;
  }
  th,
  td {
    text-align: left;
    vertical-align: top;
    padding: 4px 8px;
    border-bottom: 1px solid var(--sse-rule);
  }
  thead th,
  caption {
    color: var(--sse-ink-muted);
    font-weight: normal;
    font-size: 0.85em;
    text-align: left;
  }
  .sym {
    font-size: 1.2em;
    white-space: nowrap;
  }
  .why {
    color: var(--sse-ink);
  }
  .mark {
    font-family: var(--sse-font-body);
    font-size: 0.7em;
    margin-left: 4px;
    padding: 0 4px;
    border: 1px solid var(--sse-line);
    color: var(--sse-ink-muted);
  }
  .differs {
    border-style: dashed;
    border-color: var(--sse-alert, var(--sse-accent));
    color: var(--sse-alert, var(--sse-accent));
    font-family: var(--sse-font-mono);
  }
  tr.disagree > * {
    background: color-mix(in srgb, var(--sse-alert, var(--sse-accent)) 10%, transparent);
  }
  tr.k-unclassified .sym {
    color: var(--sse-ink-muted);
  }
  .group h4 {
    margin: 14px 0 4px;
    font-size: 0.85em;
    color: var(--sse-ink-muted);
    font-weight: normal;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    list-style: none;
    margin: 0;
    padding: 0;
  }
  button {
    font: inherit;
    color: var(--sse-ink);
    background: var(--sse-ground-raised, var(--sse-ground));
    border: 1px solid var(--sse-line);
    cursor: pointer;
  }
  button:focus-visible,
  select:focus-visible {
    outline: 2px solid var(--sse-focus, var(--sse-accent));
    outline-offset: 1px;
  }
  .chip {
    padding: 2px 8px;
    border-radius: 999px;
  }
  .chip[aria-expanded="true"] {
    border-color: var(--sse-accent);
  }
  .chip .words {
    margin-left: 6px;
    color: var(--sse-ink-muted);
  }
  .detail {
    margin: 6px 0 0;
    color: var(--sse-ink-muted);
    font-size: 0.9em;
  }
  .pick {
    border: none;
    background: none;
    padding: 0;
    text-decoration: underline;
    text-align: left;
  }
  .back {
    padding: 2px 8px;
  }
  .body-pick select {
    font: inherit;
    color: var(--sse-ink);
    background: var(--sse-ground);
    border: 1px solid var(--sse-line);
    margin-left: 6px;
  }
  .text {
    margin: 0;
    padding-left: 1.2em;
  }
  @media (max-width: 520px) {
    .segments thead {
      display: none;
    }
    .segments tr {
      display: grid;
      grid-template-columns: auto 1fr;
      border-bottom: 1px solid var(--sse-rule);
    }
    .segments th,
    .segments td {
      border: none;
    }
    .segments .why {
      grid-column: 1 / -1;
    }
    .list {
      display: block;
      overflow-x: auto;
    }
  }
</style>
