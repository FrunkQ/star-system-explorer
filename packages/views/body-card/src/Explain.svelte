<!--
  "Why?": the kernel's explanation of one derived value, in three lenses
  (curious, science, engine), with its inputs. A derived input opens its own
  explanation, so the chain can be followed back to what was set or resolved.
-->
<script lang="ts">
  import type { DeriveRef, Explanation, ViewSession } from "../../../ui-contract/src/index.ts";
  import { roundTo } from "../../../ui-contract/src/index.ts";
  import { Lenses, References, Stamp } from "../../../chart-grammar/src/index.ts";

  let {
    session,
    why,
    label,
    onclose,
    onopenflow,
    depth = "curious",
  }: { depth?: "curious" | "science" | "engine"; session: ViewSession; why: DeriveRef & { kind: string }; label: string; onclose: () => void; onopenflow?: (node: string) => void } = $props();

  let trail = $state<{ ref: string; label: string }[]>([]);
  // Opens at the app's depth (top-level design 2.4); its own tabs still move freely.
  // svelte-ignore state_referenced_locally
  let lens = $state<"curious" | "science" | "engine">(depth);
  let result = $state<{ ok: true; explanation: Explanation } | { ok: false; message: string } | null>(null);
  const kind = $derived(trail.length ? trail[trail.length - 1]!.ref : why.kind);

  $effect(() => {
    trail = [];
    void why;
  });
  $effect(() => {
    result = null;
    session.explain(why, kind).then((r) => (result = r));
  });

  const fmt = (v: number) => (Math.abs(v) >= 1e5 || (Math.abs(v) < 1e-3 && v !== 0) ? v.toExponential(3) : String(roundTo(v, 4)));
  const SI = ["m", "kg", "s", "A", "K", "mol", "cd"];
  const unit = (dim: number[]) =>
    dim
      .map((p, i) => (p === 0 ? "" : p === 1 ? SI[i] : `${SI[i]}^${p}`))
      .filter(Boolean)
      .join(" ");
  /** The campaign pack that publishes this flow, when it is not a base pack. */
  const changedBy = (flow: string | undefined) => {
    const p = flow ? session.flowDoc(flow)?.pack : undefined;
    return p && !session.stack().base.some((b) => b.id === p) ? p : null;
  };
  const pretty = (q: string) => q.replace(/[._]/g, " ");
</script>

<div class="explain" role="region" aria-label="Why {label}">
  <div class="top">
    <span class="k">Why</span>
    <nav aria-label="Explanation trail">
      <button type="button" class="crumb" onclick={() => (trail = [])} aria-current={trail.length === 0 ? "step" : undefined}>{label.toLowerCase()}</button>
      {#each trail as t, i (i)}
        <span aria-hidden="true">›</span>
        <button type="button" class="crumb" onclick={() => (trail = trail.slice(0, i + 1))} aria-current={i === trail.length - 1 ? "step" : undefined}>{t.label}</button>
      {/each}
    </nav>
    <button type="button" class="close" onclick={onclose} aria-label="Close the explanation">×</button>
  </div>
  {#if !result}
    <p class="muted">Asking the kernel…</p>
  {:else if !result.ok}
    <p class="muted">No explanation: {result.message}</p>
  {:else}
    {@const e = result.explanation}
    <p class="formula">{e.formula_display}</p>
    <p class="value"><span>{pretty(e.subject)}</span> = {fmt(e.value.value)} {unit(e.value.dim)}</p>
    <div class="stamps">
      {#if e.validity !== "in-range"}<Stamp kind="out-of-range" title={e.out_of_range.join("; ")} />{/if}
      {#if e.fictional}<Stamp kind="fictional" />{/if}
      {#if changedBy(e.publisher.flow)}<Stamp kind="changed-physics" title="Published by {changedBy(e.publisher.flow)}, not the base physics" />{/if}
    </div>
    <Lenses text={e.lenses} bind:lens />
    <table>
      <caption>Inputs</caption>
      <tbody>
        {#each e.inputs as i (i.port)}
          <tr>
            <th scope="row">{pretty(i.quantity)}</th>
            <td class="num">{fmt(i.value.value)} {unit(i.value.dim)}</td>
            <td>
              {#if i.explanation_ref}
                <button type="button" class="link" onclick={() => (trail = [...trail, { ref: i.explanation_ref!, label: pretty(i.quantity) }])}>why?</button>
              {:else}
                <span class="src">{i.source === "authored" ? "set" : i.source}</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
    {#if Object.keys(e.params).length}
      <p class="muted">Parameters: {Object.entries(e.params).map(([k, v]) => `${k} = ${v}`).join(", ")}</p>
    {/if}
    {#if onopenflow}
      <p class="flowlink"><button type="button" class="link" onclick={() => onopenflow(e.publisher.node ?? "")}>Open the flow</button> <span class="src">{e.publisher.flow} · node {e.publisher.node}</span></p>
    {/if}
    {#if e.references.length}
      <References refs={e.references} />
    {/if}
  {/if}
</div>

<style>
  .explain {
    grid-column: 1 / -1;
    padding: 12px 18px 14px;
    background: var(--sse-ground);
    border-bottom: var(--sse-line) solid var(--sse-ink);
    border-top: var(--sse-line) solid var(--sse-ink);
    font-size: 13px;
    line-height: 1.5;
  }
  .top {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  nav {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
    min-width: 0;
  }
  .k {
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-accent-ink);
  }
  .crumb,
  .link,
  .close {
    border: 0;
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    cursor: pointer;
    min-height: 28px;
  }
  .crumb {
    font-family: var(--sse-font-display);
    font-style: italic;
    font-size: 14px;
    padding: 0 2px;
  }
  .crumb[aria-current="step"] {
    text-decoration: underline;
    text-underline-offset: 3px;
  }
  .close {
    font-size: 18px;
    min-width: 32px;
  }
  .formula {
    margin: 8px 0 2px;
    font-family: var(--sse-font-display);
    font-size: 17px;
  }
  .value {
    margin: 0 0 6px;
    font-family: var(--sse-font-mono);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .stamps {
    display: flex;
    gap: 6px;
  }
  .flowlink {
    margin: 6px 0;
  }
  table {
    width: 100%;
    border-collapse: collapse;
    margin: 4px 0;
  }
  caption {
    text-align: left;
    font-size: 11px;
    letter-spacing: 0.1em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  th,
  td {
    padding: 3px 0;
    border-bottom: var(--sse-line) dotted var(--sse-rule);
    text-align: left;
    font-weight: 400;
    vertical-align: baseline;
  }
  .num {
    font-family: var(--sse-font-mono);
    font-size: 12px;
  }
  .link {
    color: var(--sse-accent-ink);
    font-family: var(--sse-font-display);
    font-style: italic;
  }
  .src {
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .muted {
    color: var(--sse-ink-muted);
    margin: 6px 0;
  }
</style>
