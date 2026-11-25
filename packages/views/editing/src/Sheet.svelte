<!--
  "Type it all in" (ED13, editing design section 3, "The sheet"): every input this body has as one plain table, by
  handle, each with its figure now and a box to type in any unit of its kind; Tab moves down it. A paste of several
  lines (a column from a spreadsheet, or rows naming the input then the value) spreads down from the box it lands in.
  Before anything is written it says what it understood and what it could not read; "Set them" applies all of it as
  one change, one Undo, optionally marked catalogued with its source (the evidence kind the kernel keeps for Sol).
-->
<script lang="ts">
  import { aboutFor, handleValue, readSheet, say, sheetRows, spreadPaste, type Depth, type FigureKind, type Handle, type Json } from "../../../ui-contract/src/index.ts";

  let {
    handles,
    facts,
    cls,
    depth,
    editable,
    apply,
    onclose,
  }: {
    handles: readonly Handle[];
    facts: Record<string, Json>;
    cls: string;
    depth: Depth;
    editable: boolean;
    apply: (edits: { field: string; value: number; before: Json }[], display: string, before_display: string, catalogued: string | null) => Promise<string | null>;
    onclose: () => void;
  } = $props();

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
  <div class="foot">
    <label><input type="checkbox" bind:checked={catalogued} disabled={!editable} data-sheet-catalogued /> Catalogued, from</label>
    <input class="source" placeholder="NASA fact sheet, 2024" bind:value={source} disabled={!editable || !catalogued} data-sheet-source />
    <button type="button" class="go" disabled={!editable || busy || !read.edits.length} onclick={set} data-sheet-set>{read.edits.length ? `Set ${read.edits.length} as one change` : "Set them"}</button>
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
