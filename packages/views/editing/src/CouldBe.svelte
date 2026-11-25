<!--
  "Could be…" (ED10 part 1, editing design section 11C): one line under a world's handles, what it is not yet and could
  be ("have a field of its own · be locked to its star"). Pressing one works out the recipe on a scratch what-if (Reach
  it for each handle the record names, nothing written) and lists the ways, the smallest move first, each with its own
  "Do this": one change, one Undo. Suggestion, never nagging: closed until pressed.
-->
<script lang="ts">
  import { aboutFor, couldBeNow, handleValue, recipeFor, say, type CouldBe, type Depth, type FigureKind, type Handle, type Json, type RecipeStep, type ViewContext } from "../../../ui-contract/src/index.ts";

  let {
    ctx,
    system,
    address,
    facts,
    results,
    cls,
    depth,
    label,
    commit,
  }: {
    ctx: ViewContext;
    system: string;
    address: string;
    facts: Record<string, Json>;
    results: Record<string, Json>;
    cls: string;
    depth: Depth;
    /** The world's name, for the Curious sentence ("Mars could have a field of its own"). */
    label: string;
    commit: (h: Handle, si: number, how: { source: "computed" }, shown?: string) => Promise<string | null>;
  } = $props();

  // A suggestion the GM stopped for this world stays stopped, with the campaign (ED10 part 2): suggest, never nag.
  const quietKey = (c: CouldBe) => `${address}#${c.id}`;
  const offered = $derived(couldBeNow(results).filter((c) => !(ctx.workspace.could_quiet ?? []).includes(quietKey(c))));
  function quiet(c: CouldBe) {
    ctx.workspace.could_quiet = [...(ctx.workspace.could_quiet ?? []), quietKey(c)];
    open = null;
    steps = null;
  }
  let open = $state<CouldBe | null>(null);
  let steps = $state.raw<RecipeStep[] | null>(null);
  let working = $state(false);
  let said = $state<string | null>(null);
  const name = (h: Handle) => (depth === "curious" ? h.names.curious : h.names.science);
  const sayIt = (h: Handle, si: number) => say(Math.abs(si), h.part.kind as FigureKind, aboutFor(h, cls), { depth }).text;
  async function show(c: CouldBe) {
    if (open?.id === c.id) return void ((open = null), (steps = null));
    open = c;
    steps = null;
    said = null;
    working = true;
    try {
      steps = await recipeFor(ctx.session, c, { system, address, facts, at_yr: ctx.workspace.now_yr });
    } finally {
      working = false;
    }
  }
  async function doIt(s: RecipeStep) {
    const c = open!;
    said = await commit(s.handle, s.si, { source: "computed" }, `${sayIt(s.handle, s.si)}, to ${c.label}`);
    if (!said) ((open = null), (steps = null));
  }
</script>

{#if offered.length}
  <div class="could" data-could-be>
    {#if depth === "curious"}<span class="lead">{label} could</span>{:else}<span class="lead">Could be</span>{/if}
    {#each offered as c, i (c.id)}
      {#if i}<span class="dot" aria-hidden="true">{depth === "curious" ? (i === offered.length - 1 ? "or" : ",") : "·"}</span>{/if}
      <button type="button" class="opt" class:open={open?.id === c.id} aria-expanded={open?.id === c.id} data-could={c.id} onclick={() => show(c)}>{c.label}</button>
    {/each}
  </div>
  {#if open}
    <div class="recipe" aria-live="polite" data-recipe={open.id}>
      {#if working}<p>Working out what would {open.label}, on a scratch copy…</p>
      {:else if steps && steps.length}
        {#each steps as s, i (s.handle.id)}
          <p class="step" data-step={s.handle.id}>
            {#if i}<span class="or">or</span>{/if}
            <b>{name(s.handle)}</b> to {sayIt(s.handle, s.si)} {#if handleValue(s.handle, facts) !== null}<span class="from">(now {sayIt(s.handle, handleValue(s.handle, facts)!)})</span>{/if}
            <button type="button" class="do" data-do={s.handle.id} onclick={() => doIt(s)}>Do this</button>
          </p>
        {/each}
      {:else if steps}<p>Nothing within eight times any of its handles would {open.label}.</p>{/if}
      {#if said}<p class="problem">{said}</p>{/if}
      {#if !working}<p class="quiet"><button type="button" class="link" data-could-quiet={open.id} onclick={() => quiet(open!)}>Stop suggesting this for {label}</button></p>{/if}
    </div>
  {/if}
{/if}

<style>
  .quiet {
    margin: 4px 0 0;
  }
  .link {
    padding: 0;
    border: 0;
    background: none;
    color: var(--sse-ink-muted);
    font: inherit;
    font-size: 11px;
    text-decoration: underline;
    cursor: pointer;
  }
  .could {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px;
    padding: 4px 18px 6px;
    font-size: 12px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .lead {
    font-weight: 600;
    margin-right: 2px;
  }
  .dot {
    color: var(--sse-ink-muted);
  }
  .opt {
    padding: 0;
    border: 0;
    border-bottom: var(--sse-line) dotted var(--sse-ink-muted);
    background: none;
    color: var(--sse-ink);
    font: inherit;
    cursor: pointer;
  }
  .opt.open {
    color: var(--sse-accent-ink);
    border-bottom-color: var(--sse-accent-ink);
  }
  .recipe {
    padding: 4px 18px 8px;
    font-size: 12px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .recipe p {
    margin: 3px 0;
  }
  .or,
  .from {
    color: var(--sse-ink-muted);
  }
  .do {
    margin-left: 6px;
    min-height: 26px;
    padding: 1px 10px;
    border: var(--sse-line) solid var(--sse-accent);
    border-radius: 13px;
    background: var(--sse-ground-raised);
    color: var(--sse-accent-ink);
    font: inherit;
    cursor: pointer;
  }
  .problem {
    color: var(--sse-accent-ink);
  }
</style>
