<!--
  The handles row (ED1, editing design sections 1 to 3): a world's six true inputs (a star's two) as one short row of
  figures on the body card. Pressing one opens its strip under the row; one strip at a time; closed it takes no room.
  On the Build desk the strip moves the value and letting go is one set_value, one Undo; the card then lists what
  followed. On Explore the strip is live as a try (three desks TM4): the first change opens a what-if by itself
  (`tryFirst` in edit.ts), the stamp and dashed frame appear, and nothing reaches the campaign unless "Keep it" is
  pressed under the time ruler. Reach it, Could be and the sweep are Build's. A handle with a box in the sheet (Orbit,
  Spin: ED12) opens the box, its own strip on top.
  Values are the kernel's facts for the body (resolved_system), never the card's display strings.
-->
<script lang="ts">
  import { aboutFor, sheetRows, chosenReading, commitEdit, commitFields, deskChanges, deskKeeps, deskTries, fieldValue, handleFigure, handlesFor, handleSubject, handleValue, referenceDots, resolveWith, resultsOf, say, makeupOf, PART_WORDS, type MakeupPart, scaleClass, stripDomain, sweepAsk, sweepMarks, atlasPacks, yearsToSeconds, classNamesIn, whatFollowed, type Followed, type PartMark, type Handle, type Json, type SceneSubject, type SolSheet, type StripDot, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";
  import { untrack } from "svelte";
  import Strip from "./Strip.svelte";
  import Box from "./Box.svelte";
  import Reach from "./Reach.svelte";
  import AirBox from "./AirBox.svelte";
  import MakeupBox from "./MakeupBox.svelte";
  import Sheet from "./Sheet.svelte";
  import CouldBe from "./CouldBe.svelte";

  let { ctx, scene, subject }: { ctx: ViewContext; scene: SystemScene; subject: SceneSubject } = $props();

  const ws = $derived(ctx.workspace);
  const depth = $derived(ws.depth ?? "science");
  const on = $derived(handleSubject(subject?.kind ?? ""));
  const handles = $derived(on ? handlesFor(on) : []);
  // Build moves handles and keeps the change; Explore moves them as a try, on a what-if that opens by itself with the
  // first change (TM4), and nothing reaches the main line unless it is kept. Run shows no editing. The desk's record
  // answers both (desk.ts), not its name.
  const gm = $derived(ctx.session.seat.authority === "gm");
  const editable = $derived(gm && deskChanges(ws.desk));
  // Build's own tools on the handles (design section 6): Reach it, Could be, the sweep behind the strip, Type it all in.
  const keeps = $derived(editable && !!ws.desk && deskKeeps(ws.desk));
  // On Explore before the first change, the strip says that a change is a try.
  const trying = $derived(editable && !ws.branch && deskTries(ws.desk));

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

  // The sweep behind the strip (ED2, kernel K1): when a world's Orbit or Mass strip opens on the Build desk, the
  // kernel runs that input over the strip's range on a scratch branch it throws away, and the strip marks where the
  // world stops lasting and where a yes-or-no result it can reach flips ("turns freely from here"). Nothing is
  // written; it runs again only when the value moves.
  // The class atlas (packs/base-atlas): on the stack, or the one the shell ships. The Mass strip's sweep labels each step
  // with it, so the strip also marks where the world's class changes; the Makeup box takes its Kinds from it.
  const stackedAtlas = $derived(atlasPacks(ctx.session.stack().base));
  let shippedAtlas = $state.raw<unknown[]>([]);
  $effect(() => {
    if (stackedAtlas.length || shippedAtlas.length || on !== "world") return;
    void fetch("packs/base-atlas.json")
      .then((r) => (r.ok ? r.json() : null))
      .then((p) => p && (shippedAtlas = [p]))
      .catch(() => {});
  });
  const atlas = $derived(stackedAtlas.length ? stackedAtlas : shippedAtlas);
  const classNames = $derived(classNamesIn(atlas, "physical"));
  // The class regions read every part of the makeup, so only a world with its makeup set gets class marks; a change of
  // makeup makes the Mass strip's marks stale, so it is part of the sweep's key.
  const makeupSig = $derived(makeupOf(mine).map((g) => g.share).join(","));
  const sweepKey = (h: Handle, v: number | null) => `${h.id === "mass" ? makeupSig : ""}§${subject.address}|${h.id}|${v}`;
  let swept = $state.raw<{ key: string; marks: PartMark[]; why?: string } | null>(null);
  let sweeping = $state<{ key: string; done: number; total: number } | null>(null);
  let sweepRunning = $state<string | null>(null);
  const SWEPT = ["orbit", "mass"];
  $effect(() => {
    const id = open;
    const sys = scene.system;
    const sweep = ctx.session.sweep?.bind(ctx.session);
    if (!id || !SWEPT.includes(id) || on !== "world" || !keeps || ws.branch || !sys || scene.fixture || !sweep) return;
    const h = handles.find((x) => x.id === id);
    const v = h ? handleValue(h, mine) : null;
    if (!h || v === null || !(v > 0)) return;
    const key = sweepKey(h, v);
    const classes = h.id === "mass" && makeupSig && atlas.length ? { packs: atlas, scheme: "physical" } : undefined;
    // One sweep per value at a time: the kernel names its scratch branch after the body and field, so a second
    // sweep of the same started before the first ends would share (and lose) its branch.
    if (swept?.key === key || sweepRunning) return;
    const d = stripDomain(h, v, untrack(() => dotsFor(h.id)));
    sweepRunning = key;
    sweeping = { key, done: 0, total: 0 };
    void sweep(sweepAsk(h, { system: sys, address: subject.address, at_s: yearsToSeconds(untrack(() => ws.now_yr)) }, d.lo, d.hi, 12, classes), (done, total) => {
      sweeping = { key, done, total };
    })
      .then((r) => (r.ok ? { key, marks: sweepMarks(h, r.result, undefined, classNames) } : { key, marks: [], why: r.message }))
      // A sweep that throws (its scratch branch gone under it) must not hold the next one back.
      .catch((e: unknown) => ({ key, marks: [], why: e instanceof Error ? e.message : String(e) }))
      .then((done) => {
        sweepRunning = null;
        sweeping = null;
        swept = done;
      });
  });
  const sweptFor = (h: Handle) => {
    const v = handleValue(h, mine);
    return swept && swept.key === sweepKey(h, v) ? swept.marks : [];
  };

  // The held value's exact answer on a pause (design section 3, point 4): a scratch branch holds a what-if's own edits
  // and the held value, the kernel resolves it (BC1), and the branch goes. What would follow, in the card's own words.
  async function pause(h: Handle, si: number, at?: string): Promise<string | null> {
    const system = scene.system;
    if (!system) return null;
    const edits = Object.entries(ws.branch?.applied ?? {}).map(([k, v]) => {
      const i = k.lastIndexOf("#");
      return { address: k.slice(0, i), field: k.slice(i + 1), value: v.value as Json };
    });
    const r = await resolveWith(ctx.session, { system, at_yr: ws.now_yr, edits: [...edits, { address: at ?? subject.address, field: h.part.field, value: fieldValue(h, si) as Json }] });
    if (!r.ok) return null;
    const rows = whatFollowed(resultsOf(surfaces, subject.address), resultsOf((r.result as { surfaces?: unknown }).surfaces ?? null, subject.address), cls, { depth });
    return rows.length ? rows.slice(0, 3).map((f) => `${f.label} ${f.after}`).join(", ") : "nothing else this card watches would move";
  }

  // The system sheet's bodies (ED13a): every world and star here with the kernel's facts for it.
  const sheetBodies = $derived(
    scene.subjects
      .filter((s) => handleSubject(s.kind) !== null)
      .map((s) => {
        const f = seen[s.address] ?? {};
        return { address: s.address, label: s.label, facts: f, cls: scaleClass(s.kind, typeof f["body.mass"] === "number" ? (f["body.mass"] as number) : undefined) };
      }),
  );

  // The Makeup handle's figure (ED9): its two largest parts, or "Earth's" when none is set (the physics' own default).
  function makeupWords(f: Record<string, Json>): string {
    const m = makeupOf(f).filter((g) => g.share > 0).sort((a, b) => b.share - a.share);
    if (!m.length) return depth === "curious" ? "like Earth" : "Earth's (not set)";
    return m
      .slice(0, 2)
      .map((g) => `${PART_WORDS[g.gas as MakeupPart][depth === "curious" ? "curious" : "science"]} ${say(g.share, "share", { class: cls }, { depth }).text}`)
      .join(", ");
  }

  // The Air box's mix (ED8): several shares as one change, one Undo; the card then says what followed, as for a handle.
  async function commitMix(edits: { address?: string; field: string; value: number; before: Json }[], display: string, before_display: string, label = "Air", source: string | null = null): Promise<string | null> {
    const system = scene.system;
    const before = resultsOf(surfaces, subject.address);
    const r = await commitFields(ctx.session, ws, { address: subject.address, edits, display, before_display, ...(source ? { evidence: "catalogued" as const, source } : {}) });
    if (r.status !== "applied") return r.status === "pending" ? "Sent to the GM to approve." : r.status === "refused" ? r.reason : r.message;
    const again = await ctx.session.resolvedSystem(system, undefined, ws.branch?.id);
    if (again.ok) {
      facts = again.result.facts as Record<string, Record<string, Json>>;
      surfaces = (again.result as { surfaces?: unknown }).surfaces ?? null;
    }
    followed = { handle: label === "Air" ? "air" : label === "Makeup" || label === "Radius" ? "makeup" : "sheet", change: { label, before: before_display, after: display }, rest: again.ok ? whatFollowed(before, resultsOf(surfaces, subject.address), cls, { depth }) : null };
    return null;
  }

  async function commit(h: Handle, si: number, how: { source: "computed" | "typed"; sig?: number }, shown?: string, at?: string): Promise<string | null> {
    const id = h.id.split(".")[0]!;
    // Held now: the card rereads its scene after the change, and the subject is a new object by then. A pair's part
    // writes to the other member (`at`: "round each other" is the lighter one's orbit); what followed is still this body's.
    const address = at ?? subject.address;
    const system = scene.system;
    const before = resultsOf(surfaces, subject.address);
    const old = handleValue(h, seen[address] ?? {});
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
    followed = { handle: id, change, rest: again.ok ? whatFollowed(before, resultsOf(surfaces, subject.address), cls, { depth }) : null };
    return null;
  }
</script>

{#if on && !scene.fixture}
  <div class="handles" data-handles={on} role="toolbar" aria-label={editable ? "What you can set" : "What sets this world"}>
    {#each handles as h (h.id)}
      {@const fig = handleFigure(h, mine, cls, { depth, reading: pick(h) })}
      {@const waits = h.part.scale === "bar" && h.id !== "makeup"}
      {@const built = h.id === "makeup" ? makeupWords(mine) : null}
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
        <span class="value">{#if built}{built}{:else if fig}{fig.main.text}{#if fig.also}, {fig.also.text}{/if}{:else if waits}soon{:else}not set{/if}</span>
      </button>
    {/each}
    <!-- Typing every input in from a source is setting a game up: Build's. -->
    {#if keeps}
      <button type="button" class="handle sheet-open" class:open={open === "sheet"} aria-expanded={open === "sheet"} data-handle="sheet" onclick={() => ((open = open === "sheet" ? null : "sheet"), (followed = null))}>
        <span class="label">Every input</span><span class="value">Type it all in</span>
      </button>
    {/if}
  </div>
  {#if on === "world" && keeps && !ws.branch && scene.system}
    <CouldBe {ctx} system={scene.system} address={subject.address} facts={mine} results={resultsOf(surfaces, subject.address)} {cls} {depth} label={subject.label} commit={(h, si, how, shown) => commit(h, si, how, shown)} />
  {/if}
  <!-- On a phone the open strip docks across the full width above the time ruler, with the verdict over it (design
       section 15, "Phone"). -->
  <div class="dock" data-dock class:docked={open !== null || followed !== null}>
  {#if open === "sheet"}
    <Sheet {handles} facts={mine} {cls} {depth} {editable} apply={(edits, display, was, source) => commitMix(edits, display, was, "Typed in", source)} bodies={sheetBodies} rowsFor={(b) => sheetRows(handlesFor(handleSubject(scene.subjects.find((x) => x.address === b.address)?.kind ?? "") ?? "world"), depth)} applyTable={(edits, display, was, source) => commitMix(edits, display, was, "Typed in for the system", source)} onclose={() => (open = null)} />
  {:else if open}
    {@const h = handles.find((x) => x.id === open)!}
    {#if h.box}
      <Box {ctx} {scene} {subject} handle={h} facts={seen} {surfaces} {cls} {depth} {editable} reading={pick(h)} dots={dotsFor(h.id)} swept={sweptFor(h)} {commit} {pause} onclose={() => (open = null)} />
    {:else}
      {#if h.id === "makeup"}
        <MakeupBox {ctx} {atlas} at_s={yearsToSeconds(ws.now_yr)} handle={h} system={scene.system} address={subject.address} facts={mine} {cls} {depth} {editable} onbranch={!!ws.branch} {commitMix} />
      {:else}
      <Strip handle={h} reading={pick(h)} value={handleValue(h, mine)} dots={dotsFor(h.id)} marks={sweptFor(h)} {cls} {depth} {editable} {trying} onpause={(si) => pause(h, si)} oncommit={(si, how) => commit(h, si, how)} onclose={() => (open = null)} />
      {#if h.id === "air" && h.on === "world"}
        <AirBox {ctx} handle={h} facts={mine} {cls} {depth} {editable} {sol} {commitMix} />
      {/if}
      {/if}
      {#if h.reach?.length && keeps && !ws.branch && scene.system && handleValue(h, mine) !== null}
        <Reach {ctx} system={scene.system} address={subject.address} handle={h} value={handleValue(h, mine)!} results={resultsOf(surfaces, subject.address)} {cls} {depth} {commit} />
      {/if}
    {/if}
    {#if sweeping && sweeping.key.endsWith(`|${open}|${handleValue(h, mine)}`)}<p class="sweeping" role="status" data-sweeping>Running {h.names[depth === "curious" ? "curious" : "science"].toLowerCase()} across the strip{sweeping.total ? ` (${sweeping.done} of ${sweeping.total})` : ""}…</p>{/if}
    {#if swept?.why && swept.key.endsWith(`|${open}|${handleValue(h, mine)}`)}<p class="sweeping" role="status" data-sweep-why>The strip's marks could not be worked out: {swept.why}</p>{/if}
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
  .sweeping {
    margin: 2px 18px;
    font-size: 11px;
    color: var(--sse-ink-muted);
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
