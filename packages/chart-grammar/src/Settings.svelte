<!--
  Promoted controls (H8.9): plain controls for the few settings a document
  promotes, drawn from the view's parameter schema. A toggle for a yes/no, a
  slider for a number with a range, a picker for a choice, a box for text.
  Everything else stays with Orac and the raw document.
-->
<script lang="ts">
  import type { Json, ParamSpec } from "../../ui-contract/src/index.ts";
  let { specs, values, onchange, disabled = false, idPrefix = "set" }: { specs: ParamSpec[]; values: Record<string, Json>; onchange: (name: string, value: Json) => void; disabled?: boolean; idPrefix?: string } = $props();
  const label = (s: ParamSpec) => s.name.replace(/_/g, " ").replace(/^./, (c) => c.toUpperCase());
  const val = (s: ParamSpec) => values[s.name] ?? s.default;
</script>

<div class="settings" role="group" aria-label="Settings">
  {#each specs as s (s.name)}
    {@const id = `${idPrefix}-${s.name}`}
    <div class="row" data-setting={s.name}>
      {#if s.type === "boolean"}
        <label class="tog" for={id}>
          <input {id} type="checkbox" checked={val(s) === true} {disabled} onchange={(e) => onchange(s.name, e.currentTarget.checked)} />
          <span>{label(s)}</span>
        </label>
      {:else if (s.type === "number" || s.type === "integer") && s.range}
        <label for={id}>{label(s)}</label>
        <div class="slide">
          <input {id} type="range" min={s.range[0]} max={s.range[1]} step={s.type === "integer" ? 1 : (s.range[1] - s.range[0]) / 100} value={Number(val(s))} {disabled} onchange={(e) => onchange(s.name, s.type === "integer" ? Math.round(Number(e.currentTarget.value)) : Number(Number(e.currentTarget.value).toPrecision(3)))} />
          <output for={id}>{String(val(s))}{s.unit ? ` ${s.unit}` : ""}</output>
        </div>
      {:else if s.type === "enum"}
        <label for={id}>{label(s)}</label>
        <select {id} value={String(val(s))} {disabled} onchange={(e) => onchange(s.name, e.currentTarget.value)}>
          {#each s.options ?? [] as o (o)}<option value={o}>{o}</option>{/each}
        </select>
      {:else if s.type === "list"}
        <label for={id}>{label(s)}</label>
        <input {id} type="text" value={(val(s) as Json[]).join(", ")} {disabled} onchange={(e) => onchange(s.name, e.currentTarget.value.split(",").map((x) => x.trim()).filter(Boolean).map((x) => (/^-?\d+(\.\d+)?$/.test(x) ? Number(x) : x)))} />
      {:else}
        <label for={id}>{label(s)}</label>
        <input {id} type={s.type === "string" ? "text" : "number"} value={String(val(s))} {disabled} onchange={(e) => onchange(s.name, s.type === "string" ? e.currentTarget.value : Number(e.currentTarget.value))} />
      {/if}
      <p class="m">{s.meaning}{#if s.cost}<span class="cost"> · costs {s.cost} when on</span>{/if}</p>
    </div>
  {/each}
</div>

<style>
  .settings {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .row {
    display: grid;
    grid-template-columns: minmax(0, 120px) minmax(0, 1fr);
    gap: 2px 10px;
    align-items: center;
  }
  .row:has(.tog) {
    grid-template-columns: minmax(0, 1fr);
  }
  label {
    font-size: 13px;
    color: var(--sse-ink);
  }
  .tog {
    display: flex;
    align-items: center;
    gap: 8px;
    min-height: 32px;
  }
  .tog input {
    width: 18px;
    height: 18px;
    accent-color: var(--sse-accent);
  }
  .slide {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .slide input {
    flex: 1;
    min-width: 0;
    accent-color: var(--sse-accent);
  }
  output {
    min-width: 4.5em;
    font-family: var(--sse-font-mono);
    font-size: 12px;
    text-align: right;
  }
  select,
  input[type="text"],
  input[type="number"] {
    min-height: 34px;
    min-width: 0;
    padding: 0 8px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
  }
  .m {
    grid-column: 1 / -1;
    margin: 0;
    font-size: 11.5px;
    color: var(--sse-ink-muted);
  }
  .cost {
    color: var(--sse-accent-ink);
  }
</style>
