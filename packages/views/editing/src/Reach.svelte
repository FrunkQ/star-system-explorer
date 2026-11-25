<!--
  Reach it (ED5, editing design section 8): ask for a result and the handle finds the value that gives it. "Gravity
  about 0.38 g. Reach [1 g] by changing Mass." The search runs on a scratch what-if the kernel resolves and throws away
  (`reachIt`); the answer is set as the handle's own change (one Undo) and the world stays wholly physical, no mark.
  When no value from an eighth to eight times the present one gets there, it says how close it can come. The results a
  handle can reach are listed in the handle sheet (`reach`), so a pack's handle reaches the same way. Main line only for
  now: a what-if's own Reach it waits on a branch inside a branch.
-->
<script lang="ts">
  import { aboutFor, fieldValue, handleValue, readFigure, reachIt, resultFrom, RESULTS, say, type Depth, type FigureKind, type Handle, type Json, type ViewContext } from "../../../ui-contract/src/index.ts";

  let {
    ctx,
    system,
    address,
    handle,
    value,
    results,
    cls,
    depth,
    commit,
  }: {
    ctx: ViewContext;
    system: string;
    address: string;
    handle: Handle;
    /** The handle's value now, in SI. */
    value: number;
    /** The kernel's results for the body now (`resultsOf`). */
    results: Record<string, Json>;
    cls: string;
    depth: Depth;
    commit: (h: Handle, si: number, how: { source: "computed" }, shown?: string) => Promise<string | null>;
  } = $props();

  const ws = $derived(ctx.workspace);
  const name = $derived(depth === "curious" ? handle.names.curious : handle.names.science);
  const rows = $derived((handle.reach ?? []).map((k) => RESULTS.find((r) => r.key === k)).filter((r): r is NonNullable<typeof r> => !!r && !!r.kind && typeof results[r.key] === "number"));
  const sayIt = (si: number, kind: string, reading?: string) => say(si, kind as FigureKind, { class: cls }, { depth, ...(reading ? { reading } : {}) }).text;
  const sayValue = (si: number) => say(Math.abs(si), handle.part.kind as FigureKind, aboutFor(handle, cls), { depth }).text;

  let asks = $state<Record<string, string>>({});
  let said = $state<{ key: string; text: string; problem?: boolean } | null>(null);
  let busy = $state(false);
  async function reach(key: string, kind: string, label: string) {
    said = null;
    const got = readFigure(asks[key] ?? "", kind as FigureKind);
    if (!got.ok) return void (said = { key, text: `Type a ${label.toLowerCase()} to reach, with its unit.`, problem: true });
    const [lo, hi] = [Math.abs(value) / 8, Math.abs(value) * 8];
    busy = true;
    try {
      const r = await reachIt(ctx.session, { system, address, field: handle.part.field, at_yr: ws.now_yr, lo: fieldValue(handle, lo), hi: fieldValue(handle, hi), log: true, target: got.si, read: resultFrom(address, key), within: 2e-3 });
      if (!r.ok) {
        said = r.closest
          ? { key, text: `No ${name.toLowerCase()} from ${sayValue(lo)} to ${sayValue(hi)} gets there: the nearest is ${sayIt(r.closest.got, kind)} at ${sayValue(toSI(r.closest.value))}.`, problem: true }
          : { key, text: `Could not reach it: ${r.why}.`, problem: true };
        return;
      }
      const si = toSI(r.value);
      const problem = await commit(handle, si, { source: "computed" }, `${sayValue(si)}, for ${label.toLowerCase()} ${sayIt(r.got, kind)}`);
      said = problem ? { key, text: problem, problem: true } : { key, text: `${name} set to ${sayValue(si)}: ${label.toLowerCase()} ${sayIt(r.got, kind)}.` };
    } finally {
      busy = false;
    }
  }
  // The search runs in the kernel field's own units; this turns one back into SI.
  const toSI = (v: number) => handleValue(handle, { [handle.part.field]: v }) ?? v;
</script>

{#each rows as r (r.key)}
  <form class="reach" data-reach={r.key} onsubmit={(ev) => (ev.preventDefault(), reach(r.key, r.kind!, r.label))}>
    <span>{r.label} {sayIt(results[r.key] as number, r.kind!, r.reading)}. Reach</span>
    <input aria-label={`${r.label} to reach`} data-reach-ask bind:value={asks[r.key]} placeholder={sayIt(results[r.key] as number, r.kind!, r.reading).replace(/^about /, "")} />
    <button type="submit" class="go" data-reach-go disabled={busy}>{busy ? "Finding…" : `by changing ${name}`}</button>
  </form>
  {#if said?.key === r.key}<p class="said" class:problem={said.problem} aria-live="polite" data-reach-said>{said.text}</p>{/if}
{/each}

<style>
  .reach {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    padding: 6px 18px 2px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  input {
    width: 7em;
    min-height: 30px;
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    border-radius: 4px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-family: var(--sse-font-mono);
  }
  .go {
    min-height: 30px;
    padding: 2px 10px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    border-radius: 15px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
  }
  .said {
    margin: 2px 18px 4px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .said.problem {
    color: var(--sse-accent-ink);
  }
</style>
