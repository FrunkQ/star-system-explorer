<!-- The place line (top-level design 2.1; navigation section, owned by the "object selector and menus" build): the
     title of the chart room, from the largest scale shown to the one in view, with the picker beside the place's name.
     App holds where the GM is (the place, the trail, recent places, what each view says is below it); this draws it
     and turns a pick into a move through the callbacks. -->
<script lang="ts">
  import { Picker } from "../../../chart-grammar/src/index.ts";
  import PlaceNotes from "./PlaceNotes.svelte";
  import { stepperLine, stepIn, stepOut, isPlaceScale, PLACE_WORDS, SUBJECT_KIND_ORDER, subjectItems, type ElsewhereItem, type PickItem, type Place, type PlaceNote, type PlaceScale, type PlaceStep, type RailScale, type SceneSubject, type ViewSession, campaignAddress } from "../../../ui-contract/src/index.ts";

  let {
    railScale,
    place,
    steps,
    title,
    baseTitle,
    subjects,
    selection,
    selName,
    adoptedSystems,
    viewing,
    notes,
    recentElsewhere,
    goPlace,
    goTo,
    toScale,
    switchSystem,
    session = null,
    systemAddress = "",
    stepper = false,
  }: {
    railScale: RailScale;
    place: Place | null;
    steps: PlaceStep[];
    /** The system in view's title. */
    title: string;
    /** The campaign's own system's title (the shipped sample, or what it was renamed to). */
    baseTitle: string;
    subjects: readonly SceneSubject[];
    selection: string | null;
    selName: string | null;
    adoptedSystems: readonly { system: string; title: string }[];
    viewing: { system: string } | null;
    /** What each view above a system says it is and holds (sse:place). */
    notes: Partial<Record<PlaceScale, PlaceNote>>;
    recentElsewhere: readonly Place[];
    goPlace: (p: Place) => void;
    /** Go to a body by address, or "@system" for the whole system. */
    goTo: (v: string) => void;
    toScale: (s: PlaceScale) => void;
    /** Show another system of the campaign ("" for the campaign's own). */
    switchSystem: (system: string) => void;
    /** The record, for the place's description and notes (V3 gap 42); absent: no notes button. */
    session?: Pick<ViewSession, "record" | "addNote"> | null;
    /** The system in view's address ("u:kestrel/sys:1"); the campaign's notes live at its root. */
    systemAddress?: string;
    /** The Run desk's form (three desks design 3.2): every level drawn, Out and In at the end, Page Up and Page Down. */
    stepper?: boolean;
  } = $props();

  // The stepper (TM7): every level, the trail's below it pencilled, the rest as the level's word; Out and In.
  const line = $derived(stepper ? stepperLine(steps) : steps.map((x) => ({ ...x, unvisited: false })));
  const outTo = $derived(stepper ? stepOut(steps) : null);
  const inTo = $derived(stepper && place ? stepIn(steps, place, selection) : null);
  const goIn = () => inTo && goPlace(inTo);
  const goOut = () => outTo && goPlace(outTo);
  function stepKeys(e: KeyboardEvent) {
    if (!stepper || e.defaultPrevented || e.altKey || e.ctrlKey || e.metaKey) return;
    const t = e.target as HTMLElement | null;
    if (t?.closest?.("input, textarea, select, [contenteditable=true], [role=slider], dialog")) return;
    if (e.key === "PageUp" && outTo) (e.preventDefault(), goOut());
    else if (e.key === "PageDown" && inTo) (e.preventDefault(), goIn());
  }

  let narrow = $state(typeof matchMedia === "function" && matchMedia("(max-width: 600px)").matches);
  $effect(() => {
    const m = matchMedia("(max-width: 600px)");
    const on = () => (narrow = m.matches);
    m.addEventListener("change", on);
    return () => m.removeEventListener("change", on);
  });
  // Notes (V3 gap 42): at a system or a world, the system's; above the system, the whole campaign's.
  let notesOpen = $state(false);
  let notesBtn = $state<HTMLButtonElement | null>(null);
  const notesFor = $derived(
    !session?.record || !systemAddress
      ? null
      : railScale === "system" || railScale === "world"
        ? { address: systemAddress, label: title, kicker: "System" }
        : { address: campaignAddress(systemAddress), label: "This campaign", kicker: "Every system in it" },
  );

  /** The steps above a system in a word each, for a narrower bar. */
  const SHORT_WORDS: Partial<Record<PlaceScale, string>> = { web: "Web", cluster: "Nearby" };
  /** The step below this one when it is a scale of its own (the cosmic web → the galaxies nearby → the galaxy). */
  const downScale = $derived<PlaceScale | null>(railScale === "web" ? "cluster" : railScale === "cluster" ? "galaxy" : null);
  /** What the view in front says is nested below it, when the next scale down can open on it. */
  const belowNote = $derived(downScale ? notes[railScale as PlaceScale] : undefined);
  const systemTitle = (sys: string) => (sys ? (adoptedSystems.find((x) => x.system === sys)?.title ?? sys.split("/").pop()!) : baseTitle);
  /** A place in words, for the Recent list. */
  function placeWords(p: Place): string {
    if (p.scale === "world") return `${p.label ?? "A world"} · ${systemTitle(p.system)}`;
    if (p.scale === "system") return systemTitle(p.system);
    return p.label ?? PLACE_WORDS[p.scale];
  }
  function goBelow(v: string) {
    if (v === "notes:") return void (notesOpen = true);
    if (v.startsWith("at:")) {
      // A body in another system of the campaign, found by searching: that system, then the body.
      const [sys, world] = v.slice(3).split("|", 2) as [string, string];
      return goPlace({ scale: "world", system: sys, world });
    }
    if (v.startsWith("scale:") && isPlaceScale(v.slice(6))) return toScale(v.slice(6) as PlaceScale);
    if (v.startsWith("sys:")) return goPlace({ scale: "system", system: v.slice(4) });
    if (v.startsWith("up:")) {
      const u = steps.find((x) => x.scale === v.slice(3) && !x.current && !x.below);
      return void (u && goPlace(u.place));
    }
    if (v.startsWith("child:")) {
      const c = belowNote?.children?.[Number(v.slice(6))];
      return void (c && downScale && goPlace({ scale: downScale, system: viewing?.system ?? "", id: c.id }));
    }
    if (v.startsWith("recent:")) {
      const r = recentElsewhere[Number(v.slice(7))];
      return void (r && goPlace(r));
    }
    goTo(v);
  }
  // The picker beside the place's name (24 Nov): the level above first, then this level and what is in it, then the
  // other ways about (systems in the campaign, recent places). One component; the content changes with the scale.
  const atBodies = $derived(railScale === "system" || railScale === "world");
  const pickSystems = $derived([{ id: "sys:", label: baseTitle }, ...adoptedSystems.map((a) => ({ id: `sys:${a.system}`, label: a.title }))]);
  const pickItems = $derived.by<PickItem[]>(() => {
    if (atBodies) return subjectItems(subjects);
    // The galaxy holds the campaign's systems; the web and the galaxies nearby hold what their view names below them.
    if (railScale === "galaxy") return pickSystems.map((x) => ({ ...x, kinds: [] }));
    return [
      ...(downScale ? [{ id: `scale:${downScale}`, label: PLACE_WORDS[downScale], kinds: [] }] : []),
      ...(downScale ? (belowNote?.children ?? []).map((c, j) => ({ id: `child:${j}`, label: c.name, kinds: [] })) : []),
    ];
  });
  const pickSummary = $derived.by(() => {
    if (atBodies) return `${subjects.length} ${subjects.length === 1 ? "body" : "bodies"}`;
    if (railScale === "galaxy") return `${pickSystems.length} ${pickSystems.length === 1 ? "system" : "systems"}`;
    const n = belowNote?.children?.length ?? 0;
    return n ? `${n} below` : "Below";
  });
  // Search covers the whole campaign (V3 gap 37): its systems by name, and the bodies of every system already read
  // this session (kept here as the GM moves about; nothing heavy is run to find them). This level's own come first.
  let seen = $state<Record<string, { title: string; items: PickItem[] }>>({});
  $effect(() => {
    const key = viewing?.system ?? "";
    const items = subjectItems(subjects);
    const t = title;
    if (seen[key]?.items.length === items.length && seen[key]?.title === t) return;
    seen = { ...seen, [key]: { title: t, items } };
  });
  const pickElsewhere = $derived.by<ElsewhereItem[]>(() => {
    const here = viewing?.system ?? "";
    const out: ElsewhereItem[] = [];
    for (const sys of pickSystems) {
      const key = sys.id.slice(4);
      if (!(atBodies && key === here)) out.push({ id: sys.id, label: sys.label, kinds: ["Systems"], where: "a system in this campaign" });
    }
    for (const [key, v] of Object.entries(seen)) {
      if (atBodies && key === here) continue;
      for (const i of v.items) out.push({ ...i, id: `at:${key}|${i.id}`, where: `in ${v.title}` });
    }
    return out;
  });
  // Light up a tag (V3 gap 41): everything a "#tag" search found is marked wherever it is drawn. One general rule, no
  // view edits: anything standing for an object carries data-address (the chart's bodies, the picker's rows), so a
  // style keyed on those addresses marks them. It stays until turned off, here or from the chip on the line.
  let lit = $state<{ tag: string; ids: string[] } | null>(null);
  const litCss = $derived(
    lit?.ids.length
      ? `${lit.ids.map((a) => `[data-address="${CSS.escape(a)}"]:not([data-pick-row])`).join(",\n")} { filter: drop-shadow(0 0 3px var(--sse-accent)) drop-shadow(0 0 1px var(--sse-accent)); }`
      : "",
  );
  const pickAbove = $derived(steps.filter((x) => !x.current && !x.below).reverse().map((u) => ({ id: `up:${u.scale}`, label: u.label, caption: PLACE_WORDS[u.scale] === u.label ? undefined : PLACE_WORDS[u.scale] })));
  const pickSections = $derived([
    ...(atBodies && adoptedSystems.length ? [{ label: "Other systems in this campaign", entries: pickSystems.filter((x) => x.id !== `sys:${viewing?.system ?? ""}`) }] : []),
    ...(!atBodies && railScale !== "galaxy" ? [{ label: "Systems in this campaign", entries: pickSystems }] : []),
    { label: "Recent", entries: recentElsewhere.map((r, j) => ({ id: `recent:${j}`, label: placeWords(r) })) },
    // On a phone the line has no room for Notes beside the name, so it is a row here.
    ...(narrow && notesFor ? [{ label: "Notes", entries: [{ id: "notes:", label: `Description and notes`, caption: notesFor.label }] }] : []),
  ]);
</script>

<svelte:head>{#if litCss}{@html `<style data-lit-tag>${litCss}</style>`}{/if}</svelte:head>

<svelte:window onkeydown={stepKeys} />

<nav class="place" class:stepper aria-label="Place" data-scale={railScale} data-stepper={stepper ? "" : undefined}>
  {#each place ? line : [] as s, i (s.scale)}
    {@const parent = line[i + 1]?.current ?? false}
    {#if s.current && s.scale === "system" && adoptedSystems.length}
      <!-- The system's name is its own switcher when the campaign holds more than one: press the name, pick another. -->
      <label class="title-switch" title="Switch to another system in this campaign">
        <h1 class="title" data-place={s.scale}>{s.label}</h1>
        <select aria-label="System on the chart" value={viewing?.system ?? ""} onchange={(e) => switchSystem(e.currentTarget.value)} data-viewing={viewing?.system ?? ""}>
          <option value="">{baseTitle}</option>
          {#each adoptedSystems as a (a.system)}<option value={a.system}>{a.title}</option>{/each}
        </select>
      </label>
    {:else if s.current}
      <h1 class="title" data-place={s.scale}>{s.scale === "world" ? (selName ?? "A world") : s.label}</h1>
    {/if}
    {#if s.current}
      <!-- The picker: the level above, this level and what is in it, the systems in the campaign, and recent places. -->
      <span class="goto" data-below={railScale}>
        <Picker
          summary={pickSummary}
          label={atBodies ? "Go to a body" : "Go down"}
          items={pickItems}
          selected={atBodies ? selection : null}
          kindOrder={SUBJECT_KIND_ORDER}
          above={pickAbove}
          top={atBodies ? { id: "@system", label: "The whole system", caption: title } : undefined}
          sections={pickSections}
          placeholder={atBodies ? "Find a body…" : "Find…"}
          heading={!atBodies && belowNote?.children?.length ? `In ${belowNote.name}` : undefined}
          elsewhere={pickElsewhere}
          lit={lit?.tag ?? null}
          onlight={(tag, ids) => (lit = tag ? { tag, ids } : null)}
          onpick={goBelow}
        />
      </span>
      {#if lit}
        <button type="button" class="lit-chip" onclick={() => (lit = null)} title="Turn off the marks for {lit.tag}" data-lit-chip>{lit.tag} <span aria-hidden="true">×</span></button>
      {/if}
      {#if notesFor}
        <button type="button" class="notes-btn" bind:this={notesBtn} aria-expanded={notesOpen} onclick={() => (notesOpen = !notesOpen)} title="Description and notes for {notesFor.label === 'This campaign' ? 'this campaign' : notesFor.label}" data-place-notes-open aria-label="Notes">Notes</button>
        {#if notesOpen}<PlaceNotes session={session!} address={notesFor.address} label={notesFor.label} kicker={notesFor.kicker} from={notesBtn} onclose={() => (notesOpen = false)} />{/if}
      {/if}
    {:else if s.unvisited || !s.place}
      <button type="button" class="step below unvisited" data-place-step={s.scale} data-unvisited="" disabled={!inTo} onclick={goIn} title={inTo ? `In towards ${s.label}` : `Select something to go into`}>{s.label}</button>
    {:else}
      {@const to = s.place}
      <button type="button" class="step" class:up={parent} class:below={s.below} class:far={!s.below && !parent} data-place-step={s.scale} data-go={s.scale === "system" ? "system" : undefined} onclick={() => goPlace(to)} title={s.below ? `Back down to ${s.label}` : `Up to ${s.label}${parent ? " (Esc)" : ""}`}>{#if parent}<span class="up-arrow" aria-hidden="true">‹ </span>{/if}<span class="up-name"><span class="long">{s.label}</span>{#if SHORT_WORDS[s.scale] && s.label === PLACE_WORDS[s.scale]}<span class="short" aria-hidden="true">{SHORT_WORDS[s.scale]}</span>{/if}</span></button>
    {/if}
    {#if i < line.length - 1}<span class="sep" class:up-sep={parent} class:far={!s.below && !parent && !s.current} class:below={s.below} aria-hidden="true">›</span>{/if}
  {:else}
    <h1 class="title" data-place="system">{title}</h1>
  {/each}
  {#if stepper && place}
    <span class="outin" role="group" aria-label="Out and In">
      <button type="button" class="oi" data-step-out disabled={!outTo} onclick={goOut} title="Out one level (Page Up)">Out</button>
      <button type="button" class="oi" data-step-in disabled={!inTo} onclick={goIn} title={inTo ? "In one level (Page Down)" : "Select something to go into"}>In</button>
    </span>
  {/if}
</nav>

<style>
  .lit-chip {
    padding: 1px 6px;
    border: var(--sse-line) solid var(--sse-accent);
    background: transparent;
    color: var(--sse-accent);
    font: 12px var(--sse-font-mono);
    cursor: pointer;
  }
  .lit-chip:focus-visible {
    outline: 2px solid var(--sse-accent);
    outline-offset: 1px;
  }
  .notes-btn {
    padding: 2px 2px;
    border: 0;
    border-bottom: 1px dotted var(--sse-rule);
    background: transparent;
    color: var(--sse-ink-muted);
    font: 12px var(--sse-font-body);
    cursor: pointer;
  }
  .notes-btn:hover,
  .notes-btn[aria-expanded="true"] {
    color: var(--sse-ink);
    border-bottom-color: var(--sse-accent);
  }
  /* On a narrow bar the place's name needs the room: Notes is a row of the picker there instead. */
  @media (max-width: 600px) {
    .notes-btn {
      display: none;
    }
  }
  .notes-btn:focus-visible {
    outline: 2px solid var(--sse-accent);
    outline-offset: 1px;
  }
  .place {
    display: flex;
    align-items: baseline;
    gap: 8px;
    min-width: 0;
    /* A long line gives way inside its own space; it never runs under the desk switch. */
    overflow: hidden;
  }
  .title {
    margin: 0;
    min-width: 0;
    font-family: var(--sse-font-display);
    font-size: 26px;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .place > :not(.title) {
    flex: none;
  }
  .place .up {
    font: inherit;
    font-family: var(--sse-font-display);
    font-size: 17px;
    color: var(--sse-accent);
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    white-space: nowrap;
  }
  .place .up:hover {
    text-decoration: underline;
  }
  .place .title {
    flex: 0 0.1 auto;
  }
  .place .unvisited {
    color: var(--sse-ink-muted);
    font-style: italic;
  }
  .place .unvisited:disabled {
    cursor: default;
  }
  .outin {
    display: inline-flex;
    gap: 4px;
    margin-left: 6px;
    align-self: center;
  }
  .oi {
    height: 26px;
    padding: 0 10px;
    font: inherit;
    font-size: 12px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: transparent;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .oi:disabled {
    opacity: 0.4;
    cursor: default;
  }
  .place .step {
    flex: 0 8 auto;
    min-width: 2.5em;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .place .sep {
    flex: none;
  }
  .place .goto {
    flex: 0 4 auto;
    min-width: 3em;
  }
  .place .step:not(.up) {
    font: inherit;
    font-size: 13px;
    color: var(--sse-ink-muted);
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    white-space: nowrap;
  }
  .place .step:not(.up):hover {
    color: var(--sse-accent);
    text-decoration: underline;
  }
  .place .step.below {
    font-style: italic;
    flex: 0 20 auto;
    min-width: 1.5em;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .place:not([data-scale="system"], [data-scale="world"]) .title {
    flex: 0 1 auto;
    min-width: 8em;
  }
  .sep.below {
    opacity: 0.6;
  }
  .goto {
    display: inline-flex;
    min-width: 0;
  }
  .place .short {
    display: none;
  }
  @media (max-width: 1600px) {
    .place .step.far:has(.short) .long {
      display: none;
    }
    .place .step.far .short {
      display: inline;
    }
  }
  @media (max-width: 1180px) {
    .place .step.far,
    .sep.far {
      display: none;
    }
  }
  .sep {
    color: var(--sse-ink-muted);
  }
  .up-arrow {
    display: none;
  }
  .title-switch {
    position: relative;
    display: inline-flex;
    min-width: 0;
    flex: 0 0.1 auto;
    cursor: pointer;
  }
  .title-switch .title {
    text-decoration: underline dotted var(--sse-rule);
    text-underline-offset: 6px;
  }
  .title-switch select {
    position: absolute;
    inset: 0;
    width: 100%;
    opacity: 0;
    cursor: pointer;
  }
  @media (max-width: 760px) {
    .title {
      font-size: 22px;
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    .place {
      flex: 1 1 auto;
      overflow: hidden;
    }
    .place .title {
      flex: 0 1 auto;
    }
    .place .step {
      flex: none;
      min-width: 0;
    }
    .place .up {
      font-size: 22px;
      line-height: 1;
    }
    .up-arrow {
      display: inline;
    }
    .up-name,
    .up-sep {
      display: none;
    }
    .place .step.below,
    .sep.below {
      display: none;
    }
    .place .goto {
      flex: none;
      min-width: 0;
      align-self: center;
    }
    .place {
      min-width: 64px;
    }
    /* On Run the place line is its own row, the stepper (design section 8): every step drawn, scrolling sideways, Out
       and In thumb-sized and held at the row's two ends. */
    .place.stepper {
      align-items: center;
      gap: 6px;
      overflow-x: auto;
      overflow-y: hidden;
      scrollbar-width: none;
      padding: 0 0 2px;
    }
    .place.stepper .step,
    .place.stepper .step.below,
    .place.stepper .step.far,
    .place.stepper .sep,
    .place.stepper .sep.below,
    .place.stepper .sep.far {
      display: inline-flex;
      align-items: center;
      flex: none;
      min-height: 44px;
      font-size: 14px;
    }
    .place.stepper .up {
      font-size: 14px;
    }
    .place.stepper .up-name {
      display: inline;
    }
    .place.stepper .up-sep {
      display: inline-flex;
    }
    .place.stepper .up-arrow {
      display: none;
    }
    .place.stepper .title {
      flex: none;
      overflow: visible;
      font-size: 20px;
    }
    .place.stepper .outin {
      display: contents;
    }
    .place.stepper .oi {
      position: sticky;
      z-index: 1;
      flex: none;
      min-width: 56px;
      height: 44px;
      font-size: 14px;
      background: var(--sse-ground);
    }
    /* Held over the steps, so a disabled button is drawn muted rather than see-through. */
    .place.stepper .oi:disabled {
      opacity: 1;
      color: var(--sse-ink-muted);
      border-color: var(--sse-rule);
    }
    .place.stepper [data-step-out] {
      order: -1;
      left: 0;
    }
    .place.stepper [data-step-in] {
      order: 1;
      right: 0;
    }
  }
</style>
