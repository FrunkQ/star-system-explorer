<!--
  The Makeup box (ED9 part 1, editing design section 11B, part 1 "What it is built from"): one bulk bar of the five
  materials formation uses (iron and metal, rock, carbon, ices, gas), by mass. Pressing a part opens its own strip, its
  share from none to all of the world: setting it trades it against the rest, which keep their proportions, so the
  makeup always sums to one. The change is every moved share of `makeup.<part>` as one set of fields (`commitFields`),
  one Undo. Kinds for makeup are the class atlas's named examples (Like Mercury, Like Ganymede), never typed here.
  The radius the science expects for this mass and makeup (kernel K2 and K3) is said in pencil under the bar, with the
  radius it is set at; "Set the radius to it" makes it so, one more change. The cutaway and the heat and redox strips
  are ED9 part 2.
-->
<script lang="ts">
  import { makeupEdits, makeupKinds, makeupOf, MAKEUP_FIELD, PART_WORDS, retradeMakeup, say, type Depth, type Handle, type Json, type MakeupPart, type MixGas, type ViewContext } from "../../../ui-contract/src/index.ts";
  import Strip from "./Strip.svelte";

  let {
    ctx,
    handle,
    system,
    address,
    facts,
    cls,
    depth,
    editable,
    onbranch = false,
    commitMix,
  }: {
    ctx: ViewContext;
    /** The Makeup handle (this box is all of it). */
    handle: Handle;
    system: string;
    address: string;
    /** The body's own facts. */
    facts: Record<string, Json>;
    cls: string;
    depth: Depth;
    editable: boolean;
    /** On a what-if: the expected radius is the main line's, so it is not said. */
    onbranch?: boolean;
    /** One change of several fields; resolves to null when applied, or the reason it was not. */
    commitMix: (edits: { field: string; value: number; before: Json }[], display: string, before_display: string, label?: string) => Promise<string | null>;
  } = $props();

  const mix = $derived(makeupOf(facts));
  const words = (p: string) => PART_WORDS[p as MakeupPart]?.[depth === "curious" ? "curious" : "science"] ?? p;
  const share = (x: number) => say(x, "share", { class: cls }, { depth }).text;
  const wordsOf = (m: readonly MixGas[]) =>
    m
      .filter((g) => g.share > 0)
      .sort((a, b) => b.share - a.share)
      .slice(0, 3)
      .map((g) => `${words(g.gas)} ${share(g.share)}`)
      .join(", ");

  // Kinds for makeup from the class atlas on the stack, or the atlas the shell ships when none is stacked.
  const stacked = $derived(makeupKinds(ctx.session.stack().base).length ? ctx.session.stack().base : null);
  let shipped = $state.raw<unknown[] | null>(null);
  $effect(() => {
    if (stacked || shipped) return;
    void fetch("packs/base-atlas.json")
      .then((r) => (r.ok ? r.json() : null))
      .then((p) => p && (shipped = [p]))
      .catch(() => {});
  });
  const kinds = $derived(makeupKinds(stacked ?? shipped ?? []));

  let open = $state<MakeupPart | null>(null);
  const partHandle = (p: MakeupPart): Handle => ({
    ...handle,
    id: `makeup.${p}`,
    names: { curious: `${words(p).charAt(0).toUpperCase()}${words(p).slice(1)}, share of the world`, science: `${PART_WORDS[p].science}, mass share` },
    part: { field: MAKEUP_FIELD + p, kind: "share", scale: "linear", range: [0, 1] },
    refs: [],
    box: undefined,
  });

  let said = $state<string | null>(null);
  async function setShare(p: MakeupPart, x: number): Promise<string | null> {
    const next = retradeMakeup(mix, p, x);
    const edits = makeupEdits(facts, next);
    if (!edits.length) return null;
    return commitMix(edits, `${words(p)} ${share(x)} (${wordsOf(next)})`, mix.length ? wordsOf(mix) : "not set", "Makeup");
  }
  async function useKind(id: string) {
    const k = kinds.find((x) => x.id === id);
    if (!k) return;
    const edits = makeupEdits(facts, k.makeup);
    if (!edits.length) return void (said = `It is already built like ${k.label.replace(/^Like /, "")}.`);
    said = await commitMix(edits, `${k.label}: ${wordsOf(k.makeup)}`, mix.length ? wordsOf(mix) : "not set", "Makeup");
  }

  // The radius the science expects (K2): reread after every change of the world's facts; nothing moves.
  let expected = $state<{ set: number | null; expects: number | null; why?: string } | null>(null);
  $effect(() => {
    void facts;
    if (onbranch || !ctx.session.expects) return void (expected = null);
    let live = true;
    void ctx.session.expects(system, address, ["body.radius"]).then((r) => {
      if (!live || !r.ok) return;
      const f = r.fields[0];
      expected = f ? { set: typeof f.set === "number" ? f.set : null, expects: typeof f.expects === "number" ? f.expects : null, ...(f.why ? { why: f.why } : {}) } : null;
    });
    return () => (live = false);
  });
  const radius = (m: number) => say(m, "length", { class: cls }, { depth }).text;
  // Within half a percent is the same radius (the surface's solid radius against the bulk one).
  const differs = $derived(!!expected && expected.expects !== null && (expected.set === null || Math.abs(expected.set / expected.expects - 1) > 0.005));
  async function useRadius() {
    if (!expected || expected.expects === null) return;
    said = await commitMix([{ field: "body.radius", value: expected.expects, before: expected.set }], radius(expected.expects), expected.set === null ? "not set" : radius(expected.set), "Radius");
  }
</script>

<div class="makeup" data-makeup-box>
  <p class="head">What it is built from, by mass</p>
  {#if mix.length}
    <div class="bar" role="group" aria-label="The world's makeup" data-makeup-bar>
      {#each mix.filter((g) => g.share > 0) as g (g.gas)}
        <button type="button" class="band {g.gas}" class:open={open === g.gas} style:flex-grow={g.share} title={`${words(g.gas)}: ${share(g.share)}`} aria-pressed={open === g.gas} data-part={g.gas} onclick={() => (open = open === g.gas ? null : (g.gas as MakeupPart))}>
          {#if g.share >= 0.1}<span>{words(g.gas)} {share(g.share)}</span>{/if}
        </button>
      {/each}
    </div>
  {:else}
    <p class="head note">No makeup is set: the physics takes Earth's (a third iron, two thirds rock).</p>
  {/if}
  {#if editable}
    <div class="more" role="group" aria-label="Change a part">
      <span class="lead">Change</span>
      {#each Object.keys(PART_WORDS) as p (p)}
        <button type="button" class="chip" class:open={open === p} aria-pressed={open === p} data-part-chip={p} onclick={() => (open = open === p ? null : (p as MakeupPart))}>{words(p)}</button>
      {/each}
    </div>
  {/if}
  {#if open}
    {@const g = mix.find((x) => x.gas === open)}
    <Strip handle={partHandle(open)} value={g?.share ?? 0} dots={[]} {cls} {depth} {editable} oncommit={(si) => setShare(open!, si)} onclose={() => (open = null)} />
  {/if}
  {#if expected && expected.expects !== null}
    <p class="pencil" data-makeup-radius>
      With this mass and makeup the science expects a radius of {radius(expected.expects)}{#if expected.set !== null && differs}; it is set at {radius(expected.set)}{:else if expected.set !== null}, as it is set{/if}.
      {#if editable && differs}<button type="button" class="chip" data-makeup-use-radius onclick={useRadius}>Set the radius to it</button>{/if}
    </p>
  {:else if expected?.why}
    <p class="pencil" data-makeup-radius>{expected.why === "left out, the world has no ground to work it out from" ? "A world mostly of gas has no radius worked out from its makeup yet." : expected.why}</p>
  {/if}
  {#if editable && kinds.length}
    <div class="more" role="group" aria-label="Start from">
      <span class="lead">Start from</span>
      {#each kinds as k (k.id)}<button type="button" class="chip" data-makeup-kind={k.id} title={`${k.label}, the class atlas's ${k.of}: ${wordsOf(k.makeup)}`} onclick={() => useKind(k.id)}>{k.label}</button>{/each}
    </div>
  {/if}
  {#if said}<p class="head note" role="status">{said}</p>{/if}
</div>

<style>
  .makeup {
    padding: 6px 18px 10px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground);
  }
  .head {
    margin: 0 0 6px;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .note {
    margin: 6px 0 0;
  }
  .pencil {
    margin: 8px 0 0;
    font-size: 12px;
    font-style: italic;
    color: var(--sse-ink-muted);
  }
  .more {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    margin-top: 8px;
    font-size: 12px;
  }
  .lead {
    color: var(--sse-ink-muted);
  }
  .bar {
    display: flex;
    min-width: 0;
    height: 30px;
    border: var(--sse-line) solid var(--sse-ink-muted);
  }
  .band {
    flex-basis: 0;
    min-width: 3px;
    padding: 0 4px;
    border: 0;
    border-right: var(--sse-line) solid var(--sse-ground);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-size: 11px;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    cursor: pointer;
  }
  .band:nth-child(2n) {
    background: var(--sse-rule);
  }
  .band.open,
  .chip.open {
    outline: 2px solid var(--sse-accent);
    outline-offset: -2px;
  }
  .chip {
    min-height: 26px;
    padding: 2px 8px;
    border: var(--sse-line) solid var(--sse-ink-muted);
    border-radius: 13px;
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
    font-size: 11px;
    font-style: normal;
    cursor: pointer;
  }
  .makeup :global([data-strip]) {
    margin: 6px -18px 0;
  }
</style>
