<!--
  Player screen host (V1, live since U5). Mounts the player view stack from a
  screen document onto the kernel's scene for the "everyone" audience, through
  the screens runtime (V3). The document is loaded when the page opens:
  `?screen=<file>` names one in documents/, `?screen=composed` takes the one
  the composer handed over on this device, and the Bridge is the default.
  Joined to a GM (`?join=<room>`, V4) the page runs its own kernel for the
  table's seat, replays the GM's events and stays in step by state hash; when
  the GM goes quiet it shows the last state read-only under a banner. Opened
  on its own, the page hosts a local kernel with the campaign's stack and
  recorded facts, and its Scanned control stands in for the GM's button.
-->
<script lang="ts">
  import { checkPlayerScene, checkScreen, enterFacts, gmSeat, markerFor, observeOf, playerSubjectOf, subjectName, type AuthoredFacts, type CheckedScreen, type PlayerScene, type ScreenDoc, type Vocab, type ViewSession } from "../../ui-contract/src/index.ts";
  import { openBrowserSession } from "../../ui-contract/src/host-browser.ts";
  import { PlayerReplica, audienceOf, broadcastHub, relayHub, sharedTableSeat, type Hub, type OwnRequest, type ReplicaStatus } from "../../ui-contract/src/index.ts";
  import { APP_VERSION } from "./share.svelte.ts";
  import { focusOf } from "../../views/player-stack/src/pick.ts";
  import stackDoc from "./documents/stack.json";
  import factsDoc from "./documents/kestrel.player-facts.json";
  import { PLAYER_STACK, ScreenRuntime } from "../../views/player-stack/src/index.ts";
  import { themeCss } from "../../skins/chart-room/src/index.ts";
  import { ALL_THEMES, PLAYER_THEMES, filterLayers } from "../../skins/player/src/index.ts";
  import { FilterChain, Segmented } from "../../chart-grammar/src/index.ts";
  import vocabDoc from "./documents/vocab.plain.json";
  import { SHIPPED } from "./symbols.svelte.ts";
  import { allStyles } from "./styles.svelte.ts";
  import guideVocabDoc from "./documents/vocab.guide.json";
  import { loadPref, savePref } from "./prefs.ts";

  // A theme names its vocabulary layer (H8.4); the screen's is the fallback.
  const VOCABS: Record<string, Vocab> = { plain: vocabDoc as unknown as Vocab, guide: guideVocabDoc as unknown as Vocab };
  // The screen document, loaded at runtime and checked against the view stack and the campaign's data views.
  let screenDoc = $state<ScreenDoc | null>(null);
  let checked = $state<{ ok: true; screen: CheckedScreen } | { ok: false; errors: string[] } | null>(null);
  // V10: a seat names the screens it opens; `?screen=` picks one, else the seat's first.
  let seatScreen: string | null = null;
  async function loadScreen(): Promise<ScreenDoc> {
    const asked = new URLSearchParams(location.search).get("screen") ?? seatScreen;
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
  // Joined through a seat's link, the scene is built for that seat's audience (and lens, if it has one).
  let audience = $state(AUDIENCE);
  let lens = $state<string | null>(null);
  let seatLabel = $state("");
  let mayAsk = $state(true);
  let session: ViewSession | null = null;
  let scene = $state<PlayerScene | null>(null);
  let sceneHash = $state("");
  let sceneErrors = $state<string[]>([]);
  async function refresh() {
    const view = lens ?? screenDoc?.data_view;
    const r = await session!.scene(facts.scope, { audience, ...(view ? { data_view: view } : {}) });
    if (!r.ok) return void (sceneErrors = [r.message]);
    const bad = checkPlayerScene(r.scene);
    sceneErrors = bad ? [bad] : [];
    scene = r.scene;
    sceneHash = r.hash;
  }
  async function useScreen(s: ViewSession) {
    const doc = await loadScreen();
    const views = observeOf(s.stack().base)?.setup.views.map((v) => v.id);
    checked = checkScreen(doc, PLAYER_STACK, views, { registry: allStyles(), tier: s.device()?.tier ?? "standard" });
    screenDoc = doc;
  }

  // Joined to a game master's room, or standing alone.
  const room = new URLSearchParams(location.search).get("join");
  const joined = room !== null && /^[a-z0-9]{4,40}$/.test(room);
  let link = $state<ReplicaStatus>({ state: "joining" });
  let stateHash = $state("");
  // How this screen reached the room: the site's relay (other devices) or this browser.
  let via = $state<"relay" | "browser" | "">("");
  async function roomHub(id: string): Promise<Hub> {
    const want = new URLSearchParams(location.search).get("via");
    if (want !== "browser") {
      const r = relayHub(document.baseURI, room!, id, { timeoutMs: 3000 });
      if (await r.opened) return (via = "relay"), r;
      r.close();
    }
    via = "browser";
    return broadcastHub(room!, id);
  }
  // V5: asking the GM. A request becomes an event only if the GM approves it.
  let replicaRef: PlayerReplica | null = null;
  let mine = $state<OwnRequest[]>([]);
  let ask = $state("");
  let askError = $state("");
  async function askName() {
    if (!replicaRef || !target || !ask.trim()) return;
    const r = await replicaRef.request({ kind: "set_value", address: target.address, field: "name", value: ask.trim(), at_yr: facts.at_yr });
    askError = r.ok ? "" : r.reason;
    if (r.ok) ask = "";
  }
  const seatId = new URLSearchParams(location.search).get("seat") ?? "table";
  if (joined) {
    const id = `player-${Math.random().toString(36).slice(2, 10)}`;
    void roomHub(id).then((hub) => {
      const replica = new PlayerReplica(
        hub,
        (m) => {
          const seat = m.seats.find((x) => x.id === seatId) ?? (seatId === "table" ? sharedTableSeat() : null);
          if (!seat) {
            sceneErrors = [`This link's seat (${seatId}) is not in the game master's session any more. Ask for a new link.`];
            return Promise.reject(new Error(sceneErrors[0]));
          }
          audience = audienceOf(seat);
          lens = seat.lens ?? null;
          seatLabel = seat.label;
          mayAsk = seat.may_send.includes("set_value");
          seatScreen = seat.screens?.[0] ?? null;
          return openBrowserSession({ base: document.baseURI, seat, universe: m.universe, packs: m.packs });
        },
        { kernel: "", app: APP_VERSION, now: () => performance.now() },
      );
      replicaRef = replica;
      replica.onRequests = (r) => (mine = [...r]);
      replica.onStatus = (st) => {
        link = st;
        if (st.state === "live" || st.state === "offline") stateHash = st.state_hash;
      };
      replica.onSession = (s) => {
        session = s;
        void useScreen(s).then(refresh).catch((e) => (sceneErrors = [String(e)]));
      };
      replica.onUpdate = () => {
        if (session && screenDoc) void refresh();
      };
      replica.join();
      // Ask again until the GM answers (the GM's page may open after this one).
      const ask = setInterval(() => (link.state === "joining" ? replica.join() : clearInterval(ask)), 1500);
      setInterval(() => replica.tick(), 1000);
    });
  } else {
    Promise.all(stackDoc.packs.map(async (p) => (await fetch(new URL(p, document.baseURI))).json()))
      .then((packs) => openBrowserSession({ base: document.baseURI, seat: gmSeat(), universe: facts.universe, packs }))
      .then(async (s) => {
        const bad = await enterFacts(s, facts);
        if (bad) throw new Error(`recorded facts refused: ${JSON.stringify(bad)}`);
        session = s;
        await useScreen(s);
        await refresh();
        stateHash = await s.stateHash();
      })
      .catch((e) => (sceneErrors = [String(e)]));
  }

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
    {#if !screenDoc?.data_view && !joined}
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
    {#if joined && link.state !== "live"}
      <div class="banner" role="status" aria-label="Game master" data-link={link.state}>
        {#if link.state === "joining"}Joining the game master…
        {:else if link.state === "resync"}Catching up with the game master ({link.reason})…
        {:else if link.state === "offline"}<strong>The game master is away.</strong> This is the last state they shared, read only. It updates when they are back.
        {:else if link.state === "reload"}<strong>{link.reason}</strong> <button type="button" onclick={() => location.reload()}>Reload</button>{/if}
      </div>
    {/if}
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
  {#if joined && scene && mayAsk}
    <section class="ask" aria-label="Ask the game master">
      <form onsubmit={(e) => (e.preventDefault(), askName())}>
        <label>Suggest a name for {target ? subjectName(target) : "the body in focus"} <input type="text" bind:value={ask} maxlength="40" disabled={link.state !== "live" || !target} /></label>
        <button type="submit" disabled={link.state !== "live" || !target || !ask.trim()}>Ask the game master</button>
      </form>
      {#if askError}<p class="err">{askError}</p>{/if}
      {#if mine.length}
        <ul>{#each mine as r (r.id)}<li data-own-request={r.status}>{r.intent.kind === "set_value" ? `"${r.intent.value}"` : r.intent.kind}: {r.status === "waiting" ? "waiting for the game master" : r.status}{r.reason ? ` (${r.reason})` : ""}</li>{/each}</ul>
      {/if}
    </section>
  {/if}
  <footer class="status" data-state-hash={stateHash} data-seat={joined ? seatId : undefined} data-audience={audience} data-link={joined ? link.state : "local"} data-via={via}>Scene: live from {joined ? "this screen's kernel, in step with the game master" : "the kernel"} {via ? ` (via ${via === "relay" ? "the site's relay" : "this browser"})` : ""} for {seatLabel && seatId !== "table" ? `the seat ${seatLabel}, ` : ""}the {audience} audience{sceneHash ? ` · scene ${sceneHash.slice(0, 10)}` : ""}{stateHash ? ` · state ${stateHash.slice(0, 10)}` : ""} {joined ? "" : screenDoc?.data_view ? ` · fixed to the ${screenDoc.data_view} view by the screen document` : ` · the Scanned control stands in for the GM's button and taps it for ${target ? subjectName(target) : "the body in focus"}`}</footer>
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
    grid-template-rows: auto 1fr auto auto;
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
  .ask {
    padding: 8px 24px 12px;
    font-size: 13px;
  }
  .ask form {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
  }
  .ask input,
  .ask button {
    min-height: 36px;
    font: inherit;
  }
  .ask ul {
    margin: 6px 0 0;
    padding-left: 18px;
  }
  .ask .err {
    margin: 4px 0 0;
    color: var(--sse-alert, currentColor);
  }
  .banner {
    flex-basis: 100%;
    padding: 8px 12px;
    border: var(--sse-line) solid var(--sse-ink);
    border-left-width: 4px;
    border-radius: var(--sse-radius);
    background: var(--sse-ground-raised);
    font-size: 14px;
  }
  .banner button {
    margin-left: 8px;
    min-height: 32px;
    font: inherit;
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
