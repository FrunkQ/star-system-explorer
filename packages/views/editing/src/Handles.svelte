<!--
  The handles row (ED1, editing design sections 1 to 3): a world's six true inputs (a star's two) as one short row of
  figures on the body card. Pressing one opens its strip under the row; one strip at a time; closed it takes no room.
  On the Build desk the strip moves the value and letting go is one set_value, one Undo; the card then lists what
  followed. On Explore the strip opens read-only, as an explanation of where this world sits, with "Try changing it"
  (ED7), which opens an example what-if where it moves. A handle with a box in the sheet (Orbit, Spin: ED12) opens the
  box, its own strip on top.
  Values are the kernel's facts for the body (resolved_system), never the card's display strings.
-->
<script lang="ts">
  import { aboutFor, chosenReading, commitEdit, openWhatIf, fieldValue, handleFigure, handlesFor, handleSubject, handleValue, referenceDots, resultsOf, say, scaleClass, whatFollowed, type Followed, type Handle, type Json, type SceneSubject, type SolSheet, type StripDot, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import Strip from "./Strip.svelte";
  import Box from "./Box.svelte";

  let { ctx, scene, subject }: { ctx: ViewContext; scene: SystemScene; subject: SceneSubject } = $props();

  const ws = $derived(ctx.workspace);
  const depth = $derived(ws.depth ?? "science");
  const on = $derived(handleSubject(subject?.kind ?? ""));
  const handles = $derived(on ? handlesFor(on) : []);
  // Build moves handles; a what-if moves them on its branch (an example opened from Explore with "Try changing it",
  // ED7, onboarding section 12), and nothing reaches the main line unless it is kept. Run shows no editing.
  const gm = $derived(ctx.session.seat.authority === "gm");
  const editable = $derived(gm && ws.desk !== "run" && (ws.desk === "build" || !!ws.branch));
  const canTry = $derived(gm && ws.desk === "explore" && !ws.branch);

  // The kernel's facts and results for every body of this system, reread after each change.
  let facts = $state.raw<Record<string, Record<string, Json>>>({});
  let surfaces = $state.raw<unknown>(null);
  // On a what-if, the branch's own line (kernel BC1): the exact figures and results there, nothing committed (ED1 part 3).
  const branchId = $derived(ws.branch?.id);
  $effect(() => {
    const sys = scene.system;
    void ws.revision;
    const b = branchId;
    void (b ? ws.branch?.edits.length : 0);
    if (!sys || scene.fixture) return;
    let live = true;
    void ctx.session.resolvedSystem(sys, undefined, b).then((r) => {
      if (!live || !r.ok) return;
      facts = r.result.facts as Record<string, Record<string, Json>>;
      surfaces = (r.result as { surfaces?: unknown }).surfaces ?? null;
    });
    return () => (live = false);
  });
  // On a what-if, its own values over the line read (until the branch's own resolve has come back, and for a kernel
  // without BC1).
  const seen = $derived.by(() => {
    const b = ws.branch;
    if (!b) return facts;
    const out: Record<string, Record<string, Json>> = { ...facts };
    for (const [k, v] of Object.entries(b.applied)) {
      const i = k.lastIndexOf("#");
      const [a, f] = [k.slice(0, i), k.slice(i + 1)];
      out[a] = { ...(out[a] ?? {}), [f]: v.value as Json };
    }
    return out;
  });
  const mine = $derived(seen[subject?.address ?? ""] ?? {});
  let tryProblem = $state<string | null>(null);
  async function tryIt() {
    tryProblem = await openWhatIf(ctx.session, ws, "EXAMPLE · change anything");
  }
  // The reading chosen by pressing a unit anywhere (a card's figure, a strip's), per kind and scale class.
  const pick = (h: Handle) => chosenReading(ws.unit_choices, h.part.kind, aboutFor(h, cls));
  const cls = $derived(scaleClass(subject?.kind ?? "", typeof mine["body.mass"] === "number" ? (mine["body.mass"] as number) : undefined));

  // The catalogued Solar System, for the reference dots (packs/base-sol, handed over by the shell).
  let sol = $state.raw<SolSheet | null>(null);
  $effect(() => {
    if (sol || !ctx.solSheet) return;
    void ctx.solSheet().then((s) => (sol = s));
  });

  let open = $state<string | null>(null);
  let followed = $state<{ handle: string; change: Followed; rest: Followed[] | null } | null>(null);
  // Another body closes the strip; a reread of the same body (a new scene after a change or a desk switch) does not.
  let openFor: string | null = null;
  $effect(() => {
    const a = subject?.address ?? null;
    if (a === openFor) return;
    openFor = a;
    open = null;
    followed = null;
  });

  function dotsFor(id: string): StripDot[] {
    const h = handles.find((x) => x.id === id)!;
    const refs = depth === "engine" ? [] : referenceDots(h, sol, cls).filter((d) => d.label !== subject.label);
    // This system's other worlds at every depth: for an orbit, the ones round the same body.
    const here: StripDot[] = [];
    for (const s of scene.subjects) {
      if (s.address === subject.address || handleSubject(s.kind) !== on) continue;
      if (h.id === "orbit" && s.parent !== subject.parent) continue;
      const v = handleValue(h, seen[s.address] ?? {});
      if (v !== null && v !== 0) here.push({ label: s.label, si: v, from: "here" });
    }
    return [...refs.filter((r) => !here.some((x) => x.label === r.label)), ...here];
  }

  async function commit(h: Handle, si: number, how: { source: "computed" | "typed"; sig?: number }, shown?: string): Promise<string | null> {
    const id = h.id.split(".")[0]!;
    // Held now: the card rereads its scene after the change, and the subject is a new object by then.
    const address = subject.address;
    const system = scene.system;
    const before = resultsOf(surfaces, address);
    const old = handleValue(h, mine);
    // A signed value's sign is said in words (a day of −24 h spins backwards); a box part may name the change itself.
    const words = (v: number, t: string) => (h.part.signed && v < 0 ? `${t} backwards` : t);
    const sayIt = (v: number, extra = {}) => words(v, say(Math.abs(v), h.part.kind, aboutFor(h, cls), { depth, reading: pick(h) ?? h.part.reading, ...extra }).text);
    const display = shown ?? sayIt(si, how.source === "typed" ? { source: "typed" as const, sig: how.sig ?? 3 } : {});
    const r = await commitEdit(ctx.session, ws, {
      address,
      field: h.part.field,
      value: fieldValue(h, si),
      display,
      before: { value: old === null ? null : fieldValue(h, old), display: old === null ? "not set" : sayIt(old) },
    });
    if (r.status !== "applied") return r.status === "pending" ? "Sent to the GM to approve." : r.status === "refused" ? r.reason : r.message;
    const again = await ctx.session.resolvedSystem(system, undefined, ws.branch?.id);
    if (again.ok) {
      facts = again.result.facts as Record<string, Record<string, Json>>;
      surfaces = (again.result as { surfaces?: unknown }).surfaces ?? null;
    }
    const change = { label: depth === "curious" ? h.names.curious : h.names.science, before: shown !== undefined ? "" : old === null ? "not set" : sayIt(old), after: display };
    followed = { handle: id, change, rest: again.ok ? whatFollowed(before, resultsOf(surfaces, address), cls, { depth }) : null };
    return null;
  }
</script>

{#if on && !scene.fixture}
  <div class="handles" data-handles={on} role="toolbar" aria-label={editable ? "What you can set" : "What sets this world"}>
    {#each handles as h (h.id)}
      {@const fig = handleFigure(h, mine, cls, { depth, reading: pick(h) })}
      {@const waits = h.part.scale === "bar"}
      <button
        type="button"
        class="handle"
        class:open={open === h.id}
        class:set={!!fig}
        data-handle={h.id}
        aria-expanded={open === h.id}
        disabled={waits}
        title={waits ? `Coming: ${h.waits ?? "its own box"}` : undefined}
        onclick={() => {
          open = open === h.id ? null : h.id;
          followed = null;
        }}
      >
        <span class="label">{depth === "curious" ? h.names.curious : h.names.science}</span>
        <span class="value">{#if fig}{fig.main.text}{#if fig.also}, {fig.also.text}{/if}{:else if waits}soon{:else}not set{/if}</span>
      </button>
    {/each}
  </div>
  <!-- On a phone the open strip docks across the full width above the time ruler, with the verdict over it (design
       section 15, "Phone"). -->
  <div class="dock" data-dock class:docked={open !== null || followed !== null}>
  {#if open}
    {@const h = handles.find((x) => x.id === open)!}
    {#if h.box}
      <Box {ctx} {scene} {subject} handle={h} facts={seen} {surfaces} {cls} {depth} {editable} ontry={canTry ? tryIt : undefined} reading={pick(h)} dots={dotsFor(h.id)} {commit} onclose={() => (open = null)} />
    {:else}
      <Strip handle={h} reading={pick(h)} value={handleValue(h, mine)} dots={dotsFor(h.id)} {cls} {depth} {editable} ontry={canTry ? tryIt : undefined} oncommit={(si, how) => commit(h, si, how)} onclose={() => (open = null)} />
    {/if}
    {#if tryProblem}<p class="try-problem" role="status">{tryProblem}</p>{/if}
  {/if}
  {#if followed}
    <div class="followed" data-followed aria-live="polite">
      <p class="what"><strong>{followed.change.label}</strong> {followed.change.before ? `${followed.change.before} → ` : ""}{followed.change.after}. <span class="undo">{!ws.branch ? "Undo takes it back." : `On ${ws.branch.label} only: Keep it or Throw away under the time ruler.`}</span></p>
      {#if !followed.rest}
        <p class="sub">What followed could not be worked out on {ws.branch?.label ?? "the what-if"}; it shows once it is kept.</p>
      {:else if followed.rest.length}
        <p class="sub">What followed</p>
        <ul>
          {#each followed.rest as f (f.label)}<li data-followed-row={f.label}><span>{f.label}</span> {f.before} → {f.after}</li>{/each}
        </ul>
      {:else}
        <p class="sub">Nothing else this card watches moved.</p>
      {/if}
    </div>
  {/if}
  </div>
{/if}

<style>
  .dock {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }
  @media (max-width: 600px) {
    .dock.docked {
      position: sticky;
      /* Above the time ruler, which is sticky at the foot of a phone's page (the shell gives its height). */
      bottom: var(--sse-ruler-h, 0px);
      z-index: 3;
      max-height: max(160px, min(50vh, calc(100vh - var(--sse-ruler-h, 0px) - 96px)));
      overflow-y: auto;
      overscroll-behavior: contain;
      background: var(--sse-ground-raised);
      border-top: var(--sse-line) solid var(--sse-rule);
    }
    .dock .followed {
      order: -1;
    }
  }
  .handles {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 2px 4px;
    min-width: 0;
    padding: 8px 12px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .handle {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    min-width: 0;
    min-height: 40px;
    padding: 4px 8px;
    border: var(--sse-line) solid transparent;
    background: none;
    color: var(--sse-ink);
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: border-color var(--sse-motion-quick) var(--sse-ease);
  }
  .handle:hover:not(:disabled),
  .handle.open {
    border-color: var(--sse-ink-muted);
  }
  .handle.open {
    background: var(--sse-ground);
  }
  .handle:disabled {
    cursor: default;
    color: var(--sse-ink-faint);
  }
  .label {
    font-size: 11px;
    letter-spacing: 0.04em;
    color: var(--sse-ink-muted);
  }
  .value {
    font-family: var(--sse-font-mono);
    font-size: 13px;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    border-bottom: var(--sse-line) dotted currentColor;
  }
  .handle:disabled .value {
    border-bottom-color: transparent;
  }
  .try-problem {
    margin: 0;
    padding: 4px 18px;
    font-size: 12px;
    color: var(--sse-accent-ink);
  }
  .followed {
    padding: 8px 18px 10px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    font-size: 13px;
  }
  .followed p {
    margin: 0;
  }
  .what strong {
    font-weight: 600;
  }
  .undo {
    color: var(--sse-ink-muted);
  }
  .sub {
    margin-top: 6px !important;
    font-size: 11px;
    letter-spacing: 0.04em;
    color: var(--sse-ink-muted);
  }
  ul {
    list-style: none;
    margin: 2px 0 0;
    padding: 0;
    font-family: var(--sse-font-mono);
    font-size: 12px;
  }
  li span {
    font-family: var(--sse-font-body);
    color: var(--sse-ink-muted);
    margin-right: 6px;
  }
  @media (max-width: 420px) {
    .handles {
      grid-template-columns: repeat(2, minmax(0, 1fr));
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .handle {
      transition: none;
    }
  }
</style>
