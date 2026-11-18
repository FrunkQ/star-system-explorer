<!--
  Table: sortable facts, one row per subject. Columns are fields named in the
  screen document; a column sorts by its number (unknowns last), else by text.
-->
<script lang="ts">
  import { describeFact, fact, subjectName, wordsFor, type PlayerContext, type PlayerScene } from "../../../ui-contract/src/index.ts";

  let { ctx, scene }: { ctx: PlayerContext; scene: PlayerScene } = $props();
  const columns = $derived(["name", ...((ctx.params.columns as string[]) ?? []).filter((c) => c !== "name")]);
  let sortBy = $state<string>("name");
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
            <button type="button" onclick={() => sort(c)}>{c === "name" ? "Name" : wordsFor(ctx.vocab, c).label}{sortBy === c ? (asc ? " ▲" : " ▼") : ""}</button>
          </th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each rows as r (r.s.address)}
        <tr class:sel={ctx.selection() === r.s.address}>
          <th scope="row"><button type="button" class="name" onclick={() => ctx.select(r.s.address)}>{subjectName(r.s)}</button></th>
          {#each r.cells.slice(1) as c, i (columns[i + 1])}
            <td class:unknown={!c || c.state === "unknown"}>{c ? c.text : "–"}</td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<style>
  .wrap {
    overflow-x: auto;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  table {
    width: 100%;
    border-collapse: collapse;
  }
  caption {
    text-align: left;
    padding: 10px 12px 6px;
    font-family: var(--sse-font-display);
    font-size: 18px;
  }
  th,
  td {
    padding: 6px 12px;
    border-top: var(--sse-line) solid var(--sse-rule);
    text-align: left;
    vertical-align: baseline;
    white-space: nowrap;
  }
  thead th {
    border-top: var(--sse-line) solid var(--sse-ink);
    border-bottom: var(--sse-line) solid var(--sse-ink);
  }
  thead button {
    border: 0;
    background: none;
    padding: 4px 0;
    color: var(--sse-ink-muted);
    font-family: var(--sse-font-body);
    font-size: 11px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    cursor: pointer;
  }
  td {
    font-family: var(--sse-font-mono);
    font-size: 13px;
  }
  td.unknown {
    font-family: var(--sse-font-display);
    font-style: italic;
    color: var(--sse-ink-muted);
  }
  .name {
    border: 0;
    background: none;
    padding: 4px 0;
    color: var(--sse-ink);
    font-family: var(--sse-font-display);
    font-size: 15px;
    cursor: pointer;
  }
  tr.sel .name {
    color: var(--sse-accent-ink);
    font-weight: 600;
  }
</style>
