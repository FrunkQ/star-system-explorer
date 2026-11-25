<!--
  "Type it all in" (ED13, editing design section 3, "The sheet"): every input this body has as one plain table, by
  handle, each with its figure now and a box to type in any unit of its kind; Tab moves down it. A paste of several
  lines (a column from a spreadsheet, or rows naming the input then the value) spreads down from the box it lands in.
  Before anything is written it says what it understood and what it could not read; "Set them" applies all of it as
  one change, one Undo, optionally marked catalogued with its source (the evidence kind the kernel keeps for Sol).
-->
<script lang="ts">
  import { aboutFor, handleValue, readSheet, readTable, say, sheetRows, spreadPaste, type Depth, type FigureKind, type Handle, type Json, type SheetBody, type SheetRow } from "../../../ui-contract/src/index.ts";

  let {
    handles,
    facts,
    cls,
    depth,
    editable,
    apply,
    bodies = [],
    rowsFor,
    applyTable,
    onclose,
  }: {
    handles: readonly Handle[];
    facts: Record<string, Json>;
    cls: string;
    depth: Depth;
    editable: boolean;
    apply: (edits: { field: string; value: number; before: Json }[], display: string, before_display: string, catalogued: string | null) => Promise<string | null>;
    /** The system sheet (ED13a): every body here, its rows, and one change across them all. */
    bodies?: readonly SheetBody[];
    rowsFor?: (b: SheetBody) => readonly SheetRow[];
    applyTable?: (edits: { address: string; field: string; value: number; before: Json }[], display: string, before_display: string, catalogued: string | null) => Promise<string | null>;
    onclose: () => void;
  } = $props();
  let whole = $state(false);
  let table = $state("");
  const tread = $derived(whole && rowsFor ? readTable(table, rowsFor, bodies, depth) : null);
  async function setTable() {
    if (!tread?.edits.length || !applyTable || busy) return;
    if (catalogued && !source.trim()) return void (problem = "Say where the figures come from, or untick Catalogued.");
    busy = true;
    const n = new Set(tread.edits.map((e) => e.address)).size;
    const words = (k: "before" | "after") => tread.understood.slice(0, 6).map((u) => `${u.body} ${u.label} ${u[k]}`).join(", ") + (tread.understood.length > 6 ? ", …" : "");
    problem = await applyTable(tread.edits, `${tread.edits.length} figures for ${n} bodies (${words("after")})`, words("before"), catalogued ? source.trim() : null);
    busy = false;
    if (!problem) table = "";
  }

  const rows = $derived(sheetRows(handles, depth));
  let typed = $state<Record<string, string>>({});
  const read = $derived(readSheet(rows, typed, facts, cls, depth));
  let catalogued = $state(false);
  let source = $state("");
  let problem = $state<string | null>(null);
  let busy = $state(false);

  const now = (h: Handle) => {
    const v = handleValue(h, facts);
    return v === null ? "not set" : say(Math.abs(v), h.part.kind as FigureKind, aboutFor(h, cls), { depth, ...(h.part.reading ? { reading: h.part.reading } : {}) }).text;
  };
  function paste(e: ClipboardEvent, i: number) {
    const text = e.clipboardData?.getData("text/plain") ?? "";
    if (!/\n|\t/.test(text.trim())) return;
    e.preventDefault();
    typed = { ...typed, ...spreadPaste(rows, text, i) };
  }
  async function set() {
    if (!read.edits.length || busy) return;
    if (catalogued && !source.trim()) return void (problem = "Say where the figures come from, or untick Catalogued.");
    busy = true;
    const words = (k: "before" | "after") => read.understood.map((u) => `${u.label} ${u[k]}`).join(", ");
    problem = await apply(read.edits, words("after"), words("before"), catalogued ? source.trim() : null);
    busy = false;
    if (!problem) typed = {};
  }
</script>

<div class="sheet" data-sheet>
  <div class="top">
    <h4>Type it all in</h4>
    <button type="button" class="x" aria-label="Close the sheet" onclick={onclose}>×</button>
  </div>
  {#if applyTable && rowsFor && bodies.length > 1}
    <div class="which" role="group" aria-label="Which sheet">
      <button type="button" aria-pressed={!whole} class:on={!whole} onclick={() => (whole = false)} data-sheet-one>This body</button>
      <button type="button" aria-pressed={whole} class:on={whole} onclick={() => (whole = true)} data-sheet-whole>Every body here</button>
    </div>
  {/if}
  {#if whole}
    <p class="hint">Paste a table: a first line naming the columns ("Name, Orbit (AU), Mass, Day"), then a line per body, named as on the chart. A unit in brackets is the unit of that column's bare numbers.</p>
    <textarea rows="6" bind:value={table} disabled={!editable} aria-label="A table for every body here" data-sheet-table placeholder={"Name\tOrbit (AU)\tMass\nMars\t1.524\t0.107 M⊕"}></textarea>
    {#if tread && (tread.understood.length || tread.problems.length)}
      <div class="read" aria-live="polite" data-sheet-read>
        {#if tread.columns.length}<p class="cols">Columns: {tread.columns.map((c) => `${c.heading} → ${c.row ? (rows.find((r) => r.id === c.row)?.label ?? c.row) : "left out"}`).join(" · ")}</p>{/if}
        {#each tread.understood as u, i (i)}<p data-table-understood={u.body}><b>{u.body}</b> {u.label} {u.before} → {u.after}</p>{/each}
        {#each tread.problems as p, i (i)}<p class="problem" data-table-problem>{p.where}: {p.why}</p>{/each}
      </div>
    {/if}
  {:else}
  <p class="hint">Any unit of the kind. Paste a column, or rows naming each input then its value, into the first box it fills.</p>
  <table>
    <tbody>
      {#each rows as r, i (r.id)}
        <tr data-sheet-row={r.id}>
          <th scope="row">{r.label}</th>
          <td class="now">{now(r.handle)}</td>
          <td><input aria-label={r.label} disabled={!editable} bind:value={typed[r.id]} onpaste={(e) => paste(e, i)} data-sheet-entry={r.id} /></td>
        </tr>
      {/each}
    </tbody>
  </table>
  {#if read.understood.length || read.problems.length}
    <div class="read" aria-live="polite" data-sheet-read>
      {#each read.understood as u (u.row)}<p data-sheet-understood={u.row}><b>{u.label}</b> {u.before} → {u.after}</p>{/each}
      {#each read.problems as p (p.row)}<p class="problem" data-sheet-problem={p.row}><b>{p.label}</b>: {p.why}</p>{/each}
    </div>
  {/if}
  {/if}
  <div class="foot">
    <label><input type="checkbox" bind:checked={catalogued} disabled={!editable} data-sheet-catalogued /> Catalogued, from</label>
    <input class="source" placeholder="NASA fact sheet, 2024" bind:value={source} disabled={!editable || !catalogued} data-sheet-source />
    {#if whole}
      <button type="button" class="go" disabled={!editable || busy || !tread?.edits.length} onclick={setTable} data-sheet-set>{tread?.edits.length ? `Set ${tread.edits.length} as one change` : "Set them"}</button>
    {:else}
      <button type="button" class="go" disabled={!editable || busy || !read.edits.length} onclick={set} data-sheet-set>{read.edits.length ? `Set ${read.edits.length} as one change` : "Set them"}</button>
    {/if}
  </div>
  {#if problem}<p class="problem" role="status">{problem}</p>{/if}
</div>

<style>
  .sheet {
    padding: 8px 18px 10px;
    background: var(--sse-ground);
    border-bottom: var(--sse-line) solid var(--sse-rule);
    min-width: 0;
  }
  .top {
    display: flex;
    align-items: baseline;
  }
  h4 {
    margin: 0;
    font-size: 13px;
    font-weight: 600;
  }
  .x {
    margin-left: auto;
    border: 0;
    background: none;
    color: var(--sse-ink-muted);
    font-size: 16px;
    cursor: pointer;
  }
  .hint {
    margin: 2px 0 6px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }
  th {
    text-align: left;
    font-weight: normal;
    color: var(--sse-ink-muted);
    padding: 2px 6px 2px 0;
  }
  .now {
    font-family: var(--sse-font-mono);
    padding: 2px 6px;
    overflow-wrap: anywhere;
  }
  td input,
  .source {
    width: 100%;
    min-width: 6em;
    min-height: 26px;
    padding: 1px 6px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    border-radius: 3px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-family: var(--sse-font-mono);
    box-sizing: border-box;
  }
  .which {
    display: flex;
    gap: 4px;
    margin: 4px 0;
  }
  .which button {
    min-height: 28px;
    padding: 2px 10px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .which button.on {
    border-color: var(--sse-accent);
    color: var(--sse-accent-ink);
  }
  textarea {
    width: 100%;
    box-sizing: border-box;
    padding: 4px 6px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-mono);
    font-size: 12px;
  }
  .cols {
    color: var(--sse-ink-muted);
  }
  .read {
    margin: 6px 0;
    font-size: 12px;
  }
  .read p {
    margin: 2px 0;
  }
  .problem {
    color: var(--sse-accent-ink);
    font-size: 12px;
  }
  .foot {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    font-size: 12px;
  }
  .source {
    flex: 1;
    width: auto;
  }
  .go {
    min-height: 30px;
    padding: 2px 12px;
    border: var(--sse-line) solid var(--sse-accent);
    border-radius: 15px;
    background: var(--sse-ground-raised);
    color: var(--sse-accent-ink);
    font: inherit;
    cursor: pointer;
  }
  .go:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
