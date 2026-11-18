<!--
  Table: sortable facts, one row per subject. Columns are fields named in the
  screen document; a column sorts by its number (unknowns last), else by text.
-->
<script lang="ts">
  import { untrack } from "svelte";
  import { describeFact, fact, subjectName, wordsFor, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";

  let { ctx, scene }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const columns = $derived(["name", ...((ctx.params.columns as string[]) ?? []).filter((c) => c !== "name")]);
  let sortBy = $state<string>(untrack(() => String(ctx.params.sort_by ?? "name")));
  let asc = $state(true);

  const rows = $derived.by(() => {
    const rs = scene.subjects
      .filter((s) => ctx.params.include_scope !== false || s.address !== scene.scope)
      .map((s) => ({ s, cells: columns.map((c) => (c === "name" ? null : fact(s, c))).map((f) => (f ? describeFact(ctx.vocab, f) : null)) }));
    const i = columns.indexOf(sortBy);
    const key = (r: (typeof rs)[number]) => {
      if (sortBy === "name") return subjectName(r.s);
      const c = r.cells[i];
      return c && c.state !== "unknown" ? ("number" in c && c.number !== undefined ? c.number : c.text) : null;
    };
    return [...rs].sort((a, b) => {
      const x = key(a);
      const y = key(b);
      if (x === null && y === null) return 0;
      if (x === null) return 1;
      if (y === null) return -1;
      const o = typeof x === "number" && typeof y === "number" ? x - y : String(x).localeCompare(String(y));
      return asc ? o : -o;
    });
  });

  function sort(c: string) {
    if (sortBy === c) asc = !asc;
    else {
      sortBy = c;
      asc = true;
    }
  }
</script>

<div class="wrap">
  <table>
    <caption>{String(ctx.params.title ?? "Bodies")}</caption>
    <thead>
      <tr>
        {#each columns as c (c)}
          <th scope="col" aria-sort={sortBy === c ? (asc ? "ascending" : "descending") : "none"}>
            {#if ctx.params.sortable !== false}
              <button type="button" onclick={() => sort(c)}>{c === "name" ? "Name" : wordsFor(ctx.vocab, c).label}{sortBy === c ? (asc ? " ▲" : " ▼") : ""}</button>
            {:else}{c === "name" ? "Name" : wordsFor(ctx.vocab, c).label}{/if}
          </th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each rows.slice(0, Number(ctx.params.max_rows ?? 100)) as r (r.s.address)}
        <tr class:sel={ctx.selection() === r.s.address}>
          <th scope="row"><button type="button" class="name" onclick={() => ctx.select(r.s.address)}>{subjectName(r.s)}</button></th>
          {#each r.cells.slice(1) as c, i (columns[i + 1])}
            <td class:unknown={!c || c.state === "unknown"}>{c ? c.text : String(ctx.params.empty_text ?? "–")}</td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .wrap {
    overflow-x: auto;
    background: var(--sse-panel-ground);
    border: var(--sse-frame-width) solid var(--sse-frame-line);
    border-radius: var(--sse-frame-radius);
    box-shadow: var(--sse-frame-shadow);
    color: var(--sse-panel-ink);
    text-shadow: var(--sse-text-glow);
    font-family: var(--sse-font-body);
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  caption {
    text-align: left;
    padding: 10px 12px 6px;
    background: var(--sse-cap-ground);
    color: var(--sse-cap-ink);
    border-left: var(--sse-cap-end) solid var(--sse-cap-end-colour);
    font-family: var(--sse-title-font);
    font-style: var(--sse-title-style);
    text-transform: var(--sse-title-case);
    font-size: 18px;
  }
  th,
  td {
    padding: 6px 12px;
    border-top: var(--sse-line) var(--sse-row-rule-style) var(--sse-row-rule);
    text-align: left;
    vertical-align: baseline;
    white-space: nowrap;
  }
  thead th {
    border-top: var(--sse-line) solid var(--sse-cap-rule);
    border-bottom: var(--sse-line) solid var(--sse-cap-rule);
  }
  thead button {
    border: 0;
    background: none;
    padding: 4px 0;
    color: var(--sse-panel-muted);
    font-family: var(--sse-font-body);
    font-size: 11px;
    letter-spacing: var(--sse-label-tracking);
    text-transform: var(--sse-label-case);
    text-shadow: inherit;
    cursor: pointer;
  }
  td {
    font-family: var(--sse-value-font);
    color: var(--sse-value-ink);
    font-size: 13px;
  }
  td.unknown {
    font-family: var(--sse-title-font);
    font-style: var(--sse-unknown-style);
    color: var(--sse-panel-muted);
  }
  .name {
    border: 0;
    background: none;
    padding: 4px 0;
    color: var(--sse-panel-ink);
    font-family: var(--sse-title-font);
    text-shadow: inherit;
    font-size: 15px;
    cursor: pointer;
  }
  tr.sel .name {
    color: var(--sse-accent-ink);
    font-weight: 600;
  }
</style>
