<!--
  Player screen host (V1, live since U5). Mounts the player view stack from a
  screen document onto the kernel's scene for the "everyone" audience, through
  the screens runtime (V3). The document is loaded when the page opens:
  `?screen=<file>` names one in documents/, `?screen=composed` takes the one
  the composer handed over on this device, and the Bridge is the default.
  With no network yet (V4), the page hosts its own kernel with the campaign's
  stack and recorded facts, and its Scanned control stands in for the GM's
  button: it taps Scanned for the body in focus.
-->
<script lang="ts">
  import { checkPlayerScene, checkScreen, enterFacts, gmSeat, markerFor, observeOf, playerSubjectOf, subjectName, type AuthoredFacts, type CheckedScreen, type PlayerScene, type ScreenDoc, type Vocab, type ViewSession } from "../../ui-contract/src/index.ts";
  import { openBrowserSession } from "../../ui-contract/src/host-browser.ts";
  import { focusOf } from "../../views/player-stack/src/pick.ts";
  import stackDoc from "./documents/stack.json";
  import factsDoc from "./documents/kestrel.player-facts.json";
  import { PLAYER_STACK, ScreenRuntime } from "../../views/player-stack/src/index.ts";
  import { themeCss } from "../../skins/chart-room/src/index.ts";
  import { ALL_THEMES, PLAYER_THEMES, filterLayers } from "../../skins/player/src/index.ts";
  import { FilterChain, Segmented } from "../../chart-grammar/src/index.ts";
  import vocabDoc from "./documents/vocab.plain.json";
  import { SHIPPED } from "./symbols.svelte.ts";
  import guideVocabDoc from "./documents/vocab.guide.json";
  import { loadPref, savePref } from "./prefs.ts";

  // A theme names its vocabulary layer (H8.4); the screen's is the fallback.
  const VOCABS: Record<string, Vocab> = { plain: vocabDoc as unknown as Vocab, guide: guideVocabDoc as unknown as Vocab };
  // The screen document, loaded at runtime and checked against the view stack and the campaign's data views.
  const asked = new URLSearchParams(location.search).get("screen");
  let screenDoc = $state<ScreenDoc | null>(null);
  let checked = $state<{ ok: true; screen: CheckedScreen } | { ok: false; errors: string[] } | null>(null);
  async function loadScreen(): Promise<ScreenDoc> {
    if (asked === "composed") {
      const raw = loadPref("composed-screen");
      if (!raw) throw new Error("No screen has been handed over from the composer on this device.");
      return JSON.parse(raw) as ScreenDoc;
    }
    const file = asked && /^[a-z0-9._-]+\.json$/i.test(asked) ? asked : "players.screen.json";
    const r = await fetch(new URL(`documents/${file}`, document.baseURI));
    if (!r.ok) throw new Error(`No screen document ${file}.`);
    return (await r.json()) as ScreenDoc;
  }

  const facts = factsDoc as unknown as AuthoredFacts;
  const AUDIENCE = "everyone";
  let session: ViewSession | null = null;
  let scene = $state<PlayerScene | null>(null);
  let sceneHash = $state("");
  let sceneErrors = $state<string[]>([]);
  async function refresh() {
    const r = await session!.scene(facts.scope, { audience: AUDIENCE, ...(screenDoc?.data_view ? { data_view: screenDoc.data_view } : {}) });
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
      const doc = await loadScreen();
      const views = observeOf(s.stack().base)?.setup.views.map((v) => v.id);
      checked = checkScreen(doc, PLAYER_STACK, views);
      screenDoc = doc;
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
  const vocabId = $derived(themeDoc.vocab && VOCABS[themeDoc.vocab] ? themeDoc.vocab : screenDoc && screenDoc.vocab in VOCABS ? screenDoc.vocab : "plain");
  const vocab = $derived(VOCABS[vocabId]!);
  // Symbols are the player's choice, apart from the theme; the theme says what it can carry.
  const savedSym = loadPref("player-symbology");
  let symbology = $state(SHIPPED.some((d) => d.id === savedSym) ? savedSym! : SHIPPED[0]!.id);
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

  const mark = (x: PlayerScene["subjects"][number]) => marker.mark(playerSubjectOf(x));
  const star = $derived(scene ? scene.subjects.find((s) => s.address === scene!.scope) : undefined);
</script>

<div class="page" style={rootFx}>
  <header class="bar">
    <h1>{star ? subjectName(star) : "System"}</h1>
    <span class="kicker">{screenDoc?.label ?? "…"}{screenDoc?.data_view ? ` · seen through ${screenDoc.data_view}` : ""} · audience {scene?.audience ?? "…"}</span>
    <span class="spacer"></span>
    {#if !screenDoc?.data_view}
    <Segmented
      label="Sensor data"
      value={which}
      options={[
        { value: "before", label: "Long range" },
        { value: "after", label: "Scanned" },
      ]}
      onchange={(v) => setSensor(v)}
    />
    {/if}
    <label class="pick">Theme
      <select bind:value={theme} title={themeDoc.description}>{#each PLAYER_THEMES as t (t.id)}<option value={t.id}>{t.name}</option>{/each}</select>
    </label>
    <label class="pick">Symbols
      <select bind:value={symbology}>{#each SHIPPED as d (d.id)}<option value={d.id}>{d.name}</option>{/each}</select>
    </label>
    {#if themeDoc.filters?.length}<label class="pick"><input type="checkbox" bind:checked={effects} /> Screen effects</label>{/if}
  </header>
  {#if (checked && !checked.ok) || sceneErrors.length}
    <main class="problem" role="alert"><h2>This screen cannot be shown</h2><ul>{#each [...(checked && !checked.ok ? checked.errors : []), ...sceneErrors] as e (e)}<li>{e}</li>{/each}</ul></main>
  {:else if !scene || !checked?.ok}
    <main class="problem" aria-busy="true"><p>Waiting for the scene…</p></main>
  {:else}
    {#key vocabId}
      <main class="grid"><ScreenRuntime screen={checked.screen} {scene} {vocab} {mark} bind:selection /></main>
    {/key}
  {/if}
  <footer class="status">Scene: live from the kernel for the {AUDIENCE} audience{sceneHash ? ` · scene ${sceneHash.slice(0, 10)}` : ""} {screenDoc?.data_view ? ` · fixed to the ${screenDoc.data_view} view by the screen document` : ` · the Scanned control stands in for the GM's button and taps it for ${target ? subjectName(target) : "the body in focus"}`}</footer>
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
    padding: 20px 24px;
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
      padding: 12px;
    }
  }
</style>
