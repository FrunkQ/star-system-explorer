<!--
  Player screen host (V1, live since U5). Mounts the player view stack from a
  screen document onto the kernel's scene for the "everyone" audience. With no
  network yet (V4), the page hosts its own kernel with the campaign's stack and
  recorded facts, and its Scanned control stands in for the GM's button: it
  taps Scanned for the body in focus. V3 turns this into the screens runtime.
-->
<script lang="ts">
  import { checkPlayerScene, checkScreen, enterFacts, gmSeat, markerFor, playerSubjectOf, subjectName, type AuthoredFacts, type PlayerContext, type PlayerScene, type ScreenDoc, type Vocab, type ViewHandle, type ViewSession } from "../../ui-contract/src/index.ts";
  import { openBrowserSession } from "../../ui-contract/src/host-browser.ts";
  import { focusOf } from "../../views/player-stack/src/pick.ts";
  import stackDoc from "./documents/stack.json";
  import factsDoc from "./documents/kestrel.player-facts.json";
  import { PLAYER_STACK } from "../../views/player-stack/src/index.ts";
  import { themeCss } from "../../skins/chart-room/src/index.ts";
  import { ALL_THEMES, PLAYER_THEMES, filterLayers } from "../../skins/player/src/index.ts";
  import { FilterChain, Segmented } from "../../chart-grammar/src/index.ts";
  import screenDoc from "./documents/players.screen.json";
  import vocabDoc from "./documents/vocab.plain.json";
  import { SHIPPED } from "./symbols.svelte.ts";
  import guideVocabDoc from "./documents/vocab.guide.json";
  import { loadPref, savePref } from "./prefs.ts";

  // A theme names its vocabulary layer (H8.4); the screen's is the fallback.
  const VOCABS: Record<string, Vocab> = { plain: vocabDoc as unknown as Vocab, guide: guideVocabDoc as unknown as Vocab };
  const checked = checkScreen(screenDoc as unknown as ScreenDoc, PLAYER_STACK);

  const facts = factsDoc as unknown as AuthoredFacts;
  const AUDIENCE = "everyone";
  let session: ViewSession | null = null;
  let scene = $state<PlayerScene | null>(null);
  let sceneHash = $state("");
  let sceneErrors = $state<string[]>([]);
  async function refresh() {
    const r = await session!.scene(facts.scope, { audience: AUDIENCE });
    if (!r.ok) return void (sceneErrors = [r.message]);
    const bad = checkPlayerScene(r.scene);
    sceneErrors = bad ? [bad] : [];
    scene = r.scene;
    sceneHash = r.hash;
  }
  Promise.all(stackDoc.packs.map(async (p) => (await fetch(new URL(p, document.baseURI))).json()))
    .then((packs) => openBrowserSession({ base: document.baseURI, seat: gmSeat(), universe: facts.universe, packs }))
    .then(async (s) => {
      const bad = await enterFacts(s, facts);
      if (bad) throw new Error(`recorded facts refused: ${JSON.stringify(bad)}`);
      session = s;
      await refresh();
    })
    .catch((e) => (sceneErrors = [String(e)]));

  let selection = $state<string | null>(null);
  // The Scanned control acts on the body in focus; the kernel says which view each tap left in force.
  const target = $derived(scene ? focusOf(scene, selection) : undefined);
  let scanned = $state<Record<string, boolean>>({});
  const which = $derived(target && scanned[target.address] ? "after" : "before");
  let busy = false;
  async function setSensor(v: "before" | "after") {
    if (!session || !target || busy || (v === "after") === !!scanned[target.address]) return;
    busy = true;
    const address = target.address;
    const r = await session.send({ kind: "scan_toggle", address, audience: AUDIENCE, at_yr: facts.at_yr });
    if (r.status === "applied") scanned = { ...scanned, [address]: !scanned[address] };
    else sceneErrors = [JSON.stringify(r)];
    await refresh();
    busy = false;
  }
  const saved = loadPref("player-theme");
  let theme = $state(PLAYER_THEMES.some((t) => t.id === saved) ? saved! : "chart");
  const themeDoc = $derived(PLAYER_THEMES.find((t) => t.id === theme)!);
  const vocabId = $derived(themeDoc.vocab && VOCABS[themeDoc.vocab] ? themeDoc.vocab : screenDoc.vocab in VOCABS ? screenDoc.vocab : "plain");
  const vocab = $derived(VOCABS[vocabId]!);
  // Symbols are the player's choice, apart from the theme; the theme says what it can carry.
  const savedSym = loadPref("player-symbology");
  let symbology = $state(SHIPPED.some((d) => d.id === savedSym) ? savedSym! : SHIPPED.some((d) => d.id === screenDoc.symbology) ? screenDoc.symbology : SHIPPED[0]!.id);
  const marker = $derived(markerFor(symbology, SHIPPED, themeDoc));
  $effect(() => savePref("player-symbology", symbology));
  // Screen effects follow the viewer: no motion when asked, none at all for more contrast or when switched off.
  const media = (q: string) => typeof matchMedia === "function" && matchMedia(q).matches;
  let effects = $state(loadPref("effects") !== "off");
  const layers = $derived(filterLayers(themeDoc.filters, { reducedMotion: media("(prefers-reduced-motion: reduce)"), off: !effects || media("(prefers-contrast: more)") }));
  const rootFx = $derived(layers.map((l) => l.root).join(" "));
  $effect(() => {
    const root = document.documentElement;
    root.setAttribute("style", themeCss(theme, ALL_THEMES));
    root.style.colorScheme = themeDoc.mode;
    root.dataset.theme = theme;
    savePref("player-theme", theme);
    savePref("effects", effects ? "on" : "off");
  });

  const handles: ViewHandle<PlayerScene>[] = [];
  $effect(() => {
    const s = scene;
    if (s) for (const h of handles) h.update(s);
  });

  function view(node: HTMLElement, m: { module: (typeof PLAYER_STACK)[number]; params: PlayerContext["params"] }) {
    const ctx: PlayerContext = { params: m.params, vocab, selection: () => selection, select: (a) => (selection = a), mark: (s) => marker.mark(playerSubjectOf(s)) };
    const h = m.module.mount(node, ctx, scene!);
    handles.push(h);
    return { destroy: () => (handles.splice(handles.indexOf(h), 1), h.dispose()) };
  }
  const star = $derived(scene ? scene.subjects.find((s) => s.address === scene!.scope) : undefined);
</script>

<div class="page" style={rootFx}>
  <header class="bar">
    <h1>{star ? subjectName(star) : "System"}</h1>
    <span class="kicker">{screenDoc.label} · audience {scene?.audience ?? "…"}</span>
    <span class="spacer"></span>
    <Segmented
      label="Sensor data"
      value={which}
      options={[
        { value: "before", label: "Long range" },
        { value: "after", label: "Scanned" },
      ]}
      onchange={(v) => setSensor(v)}
    />
    <label class="pick">Theme
      <select bind:value={theme} title={themeDoc.description}>{#each PLAYER_THEMES as t (t.id)}<option value={t.id}>{t.name}</option>{/each}</select>
    </label>
    <label class="pick">Symbols
      <select bind:value={symbology}>{#each SHIPPED as d (d.id)}<option value={d.id}>{d.name}</option>{/each}</select>
    </label>
    {#if themeDoc.filters?.length}<label class="pick"><input type="checkbox" bind:checked={effects} /> Screen effects</label>{/if}
  </header>
  {#if !checked.ok || sceneErrors.length}
    <main class="problem" role="alert"><h2>This screen cannot be shown</h2><ul>{#each [...(checked.ok ? [] : checked.errors), ...sceneErrors] as e (e)}<li>{e}</li>{/each}</ul></main>
  {:else if !scene}
    <main class="problem" aria-busy="true"><p>Waiting for the scene…</p></main>
  {:else}
    {#key vocabId}
      <main class="grid">
        {#each checked.screen.mounts as m (m.area)}
          <section class="area area-{m.area}" data-view={m.module.id}><div class="fill" use:view={m}></div></section>
        {/each}
      </main>
    {/key}
  {/if}
  <footer class="status">Scene: live from the kernel for the {AUDIENCE} audience{sceneHash ? ` · scene ${sceneHash.slice(0, 10)}` : ""} · the Scanned control stands in for the GM's button and taps it for {target ? subjectName(target) : "the body in focus"}</footer>
</div>
<FilterChain {layers} />

<style>
  :global(html, body) {
    margin: 0;
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
  }
  :global(*:focus-visible) {
    outline: 2px solid var(--sse-focus);
    outline-offset: 2px;
  }
  .page {
    min-height: 100dvh;
    display: grid;
    grid-template-rows: auto 1fr auto;
  }
  .bar {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 20px;
    padding: 12px 24px;
    border-bottom: var(--sse-line) solid var(--sse-ink);
  }
  h1 {
    margin: 0;
    font-family: var(--sse-font-display);
    font-size: 26px;
    font-weight: 600;
  }
  .kicker {
    font-size: 12px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--sse-ink-muted);
  }
  .spacer {
    flex: 1;
  }
  .pick {
    display: flex;
    gap: 8px;
    align-items: center;
    min-height: 40px;
    font-size: 13px;
  }
  .pick select {
    min-height: 40px;
    padding: 0 8px;
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    font: inherit;
  }
  .grid {
    display: grid;
    gap: 20px;
    padding: 20px 24px;
    grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr) minmax(0, 1fr);
    grid-template-areas:
      "a a b"
      "a a b"
      "c d b"
      "e e e"
      "f f f";
    align-items: start;
  }
  .area-a { grid-area: a; height: 460px; overflow: hidden; border: var(--sse-frame-width) solid var(--sse-frame-line); border-radius: var(--sse-frame-radius); box-shadow: var(--sse-frame-shadow); }
  .area-b { grid-area: b; }
  .area-c { grid-area: c; }
  .area-d { grid-area: d; }
  .area-e { grid-area: e; }
  .area-f { grid-area: f; }
  .area-a .fill {
    height: 100%;
  }
  .status {
    padding: 8px 24px;
    border-top: var(--sse-line) solid var(--sse-rule);
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .problem {
    padding: 24px;
  }
  @media (max-width: 760px) {
    .bar {
      padding: 10px 12px;
    }
    .grid {
      grid-template-columns: minmax(0, 1fr);
      grid-template-areas: "a" "b" "c" "d" "e" "f";
      padding: 12px;
      gap: 14px;
    }
    .area-a {
      height: 320px;
    }
  }
</style>
