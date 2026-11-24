<!-- The place line (top-level design 2.1; navigation section, owned by the "object selector and menus" build): the
     title of the chart room, from the largest scale shown to the one in view, with the picker beside the place's name.
     App holds where the GM is (the place, the trail, recent places, what each view says is below it); this draws it
     and turns a pick into a move through the callbacks. -->
<script lang="ts">
  import { Picker } from "../../../chart-grammar/src/index.ts";
  import { isPlaceScale, PLACE_WORDS, SUBJECT_KIND_ORDER, subjectItems, type PickItem, type Place, type PlaceNote, type PlaceScale, type PlaceStep, type RailScale, type SceneSubject } from "../../../ui-contract/src/index.ts";

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
  } = $props();

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
  const pickAbove = $derived(steps.filter((x) => !x.current && !x.below).reverse().map((u) => ({ id: `up:${u.scale}`, label: u.label, caption: PLACE_WORDS[u.scale] === u.label ? undefined : PLACE_WORDS[u.scale] })));
  const pickSections = $derived([
    ...(atBodies && adoptedSystems.length ? [{ label: "Other systems in this campaign", entries: pickSystems.filter((x) => x.id !== `sys:${viewing?.system ?? ""}`) }] : []),
    ...(!atBodies && railScale !== "galaxy" ? [{ label: "Systems in this campaign", entries: pickSystems }] : []),
    { label: "Recent", entries: recentElsewhere.map((r, j) => ({ id: `recent:${j}`, label: placeWords(r) })) },
  ]);
</script>

<nav class="place" aria-label="Place" data-scale={railScale}>
  {#each place ? steps : [] as s, i (s.scale)}
    {@const parent = steps[i + 1]?.current ?? false}
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
          onpick={goBelow}
        />
      </span>
    {:else}
      <button type="button" class="step" class:up={parent} class:below={s.below} class:far={!s.below && !parent} data-place-step={s.scale} data-go={s.scale === "system" ? "system" : undefined} onclick={() => goPlace(s.place)} title={s.below ? `Back down to ${s.label}` : `Up to ${s.label}${parent ? " (Esc)" : ""}`}>{#if parent}<span class="up-arrow" aria-hidden="true">‹ </span>{/if}<span class="up-name"><span class="long">{s.label}</span>{#if SHORT_WORDS[s.scale] && s.label === PLACE_WORDS[s.scale]}<span class="short" aria-hidden="true">{SHORT_WORDS[s.scale]}</span>{/if}</span></button>
    {/if}
    {#if i < steps.length - 1}<span class="sep" class:up-sep={parent} class:far={!s.below && !parent && !s.current} class:below={s.below} aria-hidden="true">›</span>{/if}
  {:else}
    <h1 class="title" data-place="system">{title}</h1>
  {/each}
</nav>

<style>
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
  }
</style>
