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
  import { PlayerReplica, SealedReplica, audienceOf, broadcastHub, relayHub, sharedTableSeat, type Hub, type OwnRequest, type ReplicaStatus, type Shown } from "../../ui-contract/src/index.ts";
  import { APP_VERSION } from "./share.svelte.ts";
  import { V3_URL } from "./alpha.ts";
  import SoundControl from "../../sound/src/SoundControl.svelte";
  import LightControl from "./LightControl.svelte";
  import lightDoc from "./documents/light.recipes.json";
  import { SYMBOLS, checkLightRecipe, describeLight, litFromPlayers, paint, type BandRecipe } from "../../ui-contract/src/index.ts";
  import { orbitScene, voicesFromPlayers } from "../../ui-contract/src/index.ts";
  import { focusOf } from "../../views/player-stack/src/pick.ts";
  import stackDoc from "./documents/stack.json";
  import { PLAYER_STACK, ScreenRuntime } from "../../views/player-stack/src/index.ts";
  import { themeCss } from "../../skins/chart-room/src/index.ts";
  import { ALL_THEMES, PLAYER_THEMES, filterLayers } from "../../skins/player/src/index.ts";
  import { FilterChain, Segmented } from "../../chart-grammar/src/index.ts";
  import vocabDoc from "./documents/vocab.plain.json";
  import { SHIPPED } from "./symbols.svelte.ts";
  import { allStyles } from "./styles.svelte.ts";
  import guideVocabDoc from "./documents/vocab.guide.json";
  import { loadPref, savePref } from "./prefs.ts";
  import PerfReadout from "./PerfReadout.svelte";
  import { perf } from "../../ui-contract/src/index.ts";
  import { EmbedBridge, embedAsked, embedMode, framingOrigin } from "../../ui-contract/src/index.ts";
  import { groundGrant, landingShowOf, type LandingGround, type PartyShip } from "../../ui-contract/src/index.ts";
  import { DEFAULT_JOIN, joinLine, joinWordsOf, type JoinWords, type SeatBrand } from "../../ui-contract/src/index.ts";

  // A theme names its vocabulary layer (H8.4); the screen's is the fallback.
  const VOCABS: Record<string, Vocab> = { plain: vocabDoc as unknown as Vocab, guide: guideVocabDoc as unknown as Vocab };
  // The screen document, loaded at runtime and checked against the view stack and the campaign's data views.
  let screenDoc = $state<ScreenDoc | null>(null);
  let checked = $state<{ ok: true; screen: CheckedScreen } | { ok: false; errors: string[] } | null>(null);
  // V10: a seat names the screens it opens; `?screen=` picks one, else the seat's first.
  let seatScreen: string | null = null;
  async function loadScreen(): Promise<ScreenDoc> {
    // What the GM sent names its own screen (a landing is watched on the Landing screen), else the link's, else the seat's.
    const sent = shown && "scope" in shown ? shown.screen : undefined;
    const asked = sent ?? new URLSearchParams(location.search).get("screen") ?? seatScreen;
    if (asked === "composed") {
      const raw = loadPref("composed-screen");
      if (!raw) throw new Error("No screen has been handed over from the composer on this device.");
      return JSON.parse(raw) as ScreenDoc;
    }
    // A layout the GM made (PB13) travels with the session; the shipped ones are files beside the shell.
    const made = asked?.startsWith("mine.") ? (replicaRef?.layouts ?? []).find((d) => (d as unknown as ScreenDoc).id === asked) : undefined;
    if (made) return structuredClone(made) as unknown as ScreenDoc;
    const file = asked && /^[a-z0-9._-]+\.json$/i.test(asked) ? asked : "players.screen.json";
    const r = await fetch(new URL(`documents/${file}`, document.baseURI));
    if (!r.ok) throw new Error(`No screen document ${file}.`);
    return (await r.json()) as ScreenDoc;
  }

  // The campaign's recorded facts, loaded only when this screen runs a kernel: a sealed seat's screen never fetches them.
  let facts: AuthoredFacts = { schema: "", universe: { id: "", seed: "" }, scope: "", at_yr: 0, facts: [] } as unknown as AuthoredFacts;
  const loadFacts = async () => (facts = (await import("./documents/kestrel.player-facts.json")).default as unknown as AuthoredFacts);
  const AUDIENCE = "everyone";
  // Joined through a seat's link, the scene is built for that seat's audience (and lens, if it has one).
  let audience = $state(AUDIENCE);
  let lens = $state<string | null>(null);
  let seatLabel = $state("");
  let mayAsk = $state(true);
  let session: ViewSession | null = null;
  let scene = $state<PlayerScene | null>(null);
  // What the game master has sent the table (the landing run-through): a place, or screens off. A screen that is not
  // joined, or a GM from before this, uses the screen's own campaign scope.
  let shown = $state<Shown | undefined>(undefined);
  // The party's ship (the landing run-through): the table always knows where it is, whatever screen is up.
  let party = $state.raw<PartyShip | null>(null);
  const partyOf = (j: unknown): PartyShip | null => (j && typeof j === "object" && typeof (j as PartyShip).name === "string" && typeof (j as PartyShip).at === "string" ? (j as PartyShip) : null);
  const scopeNow = () => (shown && "scope" in shown && shown.scope ? shown.scope : facts.scope);
  let sceneHash = $state("");
  let sceneErrors = $state<string[]>([]);
  async function refresh() {
    const view = lens ?? screenDoc?.data_view;
    if (shown && "off" in shown) return;
    const r = await session!.scene(scopeNow(), { audience, ...(view ? { data_view: view } : {}) });
    if (!r.ok) return void (sceneErrors = [r.message]);
    const bad = checkPlayerScene(r.scene);
    sceneErrors = bad ? [bad] : [];
    scene = r.scene;
    sceneHash = r.hash;
    await refreshPlaces(view);
  }
  // Boxes that hold a place (PB13 part 2): each place drawn for this screen's audience, as the main scene is.
  let places = $state.raw<Record<string, PlayerScene>>({});
  async function refreshPlaces(view: string | undefined) {
    const scopes = [...new Set((checked?.ok ? checked.screen.mounts : []).flatMap((m) => (m.place ? [m.place.scope] : [])))];
    if (!scopes.length) return void (Object.keys(places).length && (places = {}));
    const next: Record<string, PlayerScene> = {};
    for (const sc of scopes) {
      const r = await session!.scene(sc, { audience, ...(view ? { data_view: view } : {}) });
      if (r.ok) next[sc] = r.scene;
    }
    places = next;
  }
  async function useScreen(s: ViewSession) {
    await useScreenWith(observeOf(s.stack().base)?.setup.views.map((v) => v.id), s.device()?.tier ?? "standard");
  }
  // Two loads in flight (the role's layout named, then its document arriving): only the last one asked is used.
  let screenAsk = 0;
  async function useScreenWith(views: string[] | undefined, tier: string) {
    const ask = ++screenAsk;
    const doc = await loadScreen();
    if (ask !== screenAsk) return;
    checked = checkScreen(doc, PLAYER_STACK, views, { registry: allStyles(), tier: tier as "standard" });
    screenDoc = doc;
  }

  // Joined to a game master's room, or standing alone.
  // A sealed seat (V10 part 2) joins its own room with `?sealed=`: no kernel, no seed, only the GM's finished scenes.
  const sealedRoom = new URLSearchParams(location.search).get("sealed");
  const isSealed = sealedRoom !== null && /^[a-z0-9]{4,40}$/.test(sealedRoom);
  const room = isSealed ? sealedRoom : new URLSearchParams(location.search).get("join");
  const joined = room !== null && /^[a-z0-9]{4,40}$/.test(room);
  // The landing the table watches (the landing run-through): this seat's own kernel may read that one world's ground
  // while it lasts, and no longer (the grant ends with the landing). A sealed seat has no kernel and gets neither.
  const watching = $derived(isSealed ? null : landingShowOf(shown && "scope" in shown ? shown.landing : null));
  let sessionReady = $state(false);
  $effect(() => {
    const w = watching;
    if (!sessionReady || !session) return;
    // A landing's world, or the world an "In orbit" send names when the table knows it (LR4): that one map, no more.
    const sh = shown && "scope" in shown ? shown : null;
    const known = new Set((scene?.subjects ?? []).map((x) => x.address));
    session.grantLanding?.(w ? { system: w.system, body: w.body } : groundGrant(sh, (a) => known.has(a)));
  });
  const landingNow = () => watching;
  const shownNow = () => shown ?? null;
  const ground: LandingGround | undefined = isSealed
    ? undefined
    : {
        surfaceTiles: (system, body) => session!.surfaceTiles(system, body),
        surfaceTilesAt: (system, body, keys) => session!.surfaceTilesAt(system, body, keys),
        planDescent: (ask) => session!.planDescent(ask),
      };
  // Inside a virtual tabletop (V3 gap 26): `?embed=1`, and only for a tabletop the GM listed on this seat.
  const asksEmbed = embedAsked(location.search);
  const framing = framingOrigin(window, document.referrer);
  let embedList = $state<string[] | undefined>(undefined);
  let seatKnown = $state(!joined);
  // Branding and join words (V3 gap 25): the seat's brand, and the stacked packs' words while joining.
  let brand = $state<SeatBrand | null>(null);
  let words = $state<JoinWords>(DEFAULT_JOIN);
  // Quick overrides (V3 gap 25 part 2): the GM's live switches for this seat, as they change.
  let live = $state<Record<string, boolean> | undefined>(undefined);
  const say = (t: string) => joinLine(t, { seat: seatLabel, ...(brand ? { brand: brand.name } : {}) });
  const embed = $derived(seatKnown ? embedMode(asksEmbed, framing, embedList) : asksEmbed && framing !== "unframed" ? ({ mode: "waiting" } as const) : ({ mode: "page" } as const));
  // A clean output (PB8, design 9): `?clean=1`, no controls, a fixed 16:9 picture for a stream; `&ground=none` drops the
  // page's ground so the picture floats over the streamer's own scene. A screen like any other: it joins its role.
  const clean = new URLSearchParams(location.search).get("clean") === "1";
  const clearGround = clean && new URLSearchParams(location.search).get("ground") === "none";
  const bare = $derived(embed.mode !== "page" || clean);
  if (clearGround) document.body.classList.add("sse-clear-ground");
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
  let replicaRef: (Pick<PlayerReplica, "request"> & { layouts?: PlayerReplica["layouts"] }) | null = null;
  let mine = $state<OwnRequest[]>([]);
  let ask = $state("");
  let askError = $state("");
  async function askName() {
    if (!replicaRef || !target || !ask.trim()) return;
    const r = await replicaRef.request({ kind: "set_value", address: target.address, field: "name", value: ask.trim(), at_yr: link.state === "live" ? link.clock_yr : facts.at_yr });
    askError = r.ok ? "" : r.reason;
    if (r.ok) ask = "";
  }
  const seatId = new URLSearchParams(location.search).get("seat") ?? "table";
  // This screen (Switchboard, PB1): an id kept in this tab, so a reload comes back as the same screen, and a name the
  // GM sees on the board (`?name=`, else a short one from the id).
  const screenId = (() => {
    const key = `sse4.screen.${room ?? "alone"}`;
    try {
      const had = sessionStorage.getItem(key);
      if (had && /^[a-z0-9-]{4,40}$/.test(had)) return had;
      const made = `scr-${Math.random().toString(36).slice(2, 8)}`;
      sessionStorage.setItem(key, made);
      return made;
    } catch {
      return `scr-${Math.random().toString(36).slice(2, 8)}`;
    }
  })();
  const screenName = (new URLSearchParams(location.search).get("name") ?? "").slice(0, 40) || `Screen ${screenId.slice(4).toUpperCase()}`;
  const hello = { id: screenId, name: screenName, role: seatId, ...(new URLSearchParams(location.search).get("clean") === "1" ? { output: true } : {}) };
  // What this screen is shown: its own line if the GM gave it one, else its seat's, else the table's.
  const shownFor = (r: PlayerReplica) => r.shownFor(seatId, screenId);
  if (isSealed) {
    const id = `player-${Math.random().toString(36).slice(2, 10)}`;
    void roomHub(id).then((hub) => {
      const replica = new SealedReplica(hub, seatId, { app: APP_VERSION, now: () => performance.now() });
      replicaRef = replica;
      replica.onRequests = (r) => (mine = [...r]);
      replica.onStatus = (st) => (link = st);
      let first = true;
      replica.onScene = (sc, hash) => {
        const seat = replica.seat!;
        audience = audienceOf(seat);
        seatLabel = seat.label;
        mayAsk = seat.may_send.includes("set_value");
        embedList = seat.embed_origins;
        brand = seat.brand ?? null;
        live = seat.live;
        seatKnown = true;
        const bad = checkPlayerScene(sc);
        sceneErrors = bad ? [bad] : [];
        scene = sc;
        sceneHash = hash;
        stateHash = "";
        // Held by the GM: the card, and an empty scene behind it.
        const held = replica.held;
        shown = held === null ? undefined : held ? { off: true, card: held } : { off: true };
        if (first) {
          first = false;
          seatScreen = seat.screens?.[0] ?? null;
          void useScreenWith([...replica.dataViews], "standard").catch((e) => (sceneErrors = [String(e)]));
        }
      };
      replica.join();
      const again = setInterval(() => (link.state === "joining" ? replica.join() : clearInterval(again)), 1500);
      setInterval(() => replica.tick(), 1000);
    });
  } else if (joined) {
    const id = `player-${Math.random().toString(36).slice(2, 10)}`;
    void Promise.all([roomHub(id), loadFacts()]).then(([hub]) => {
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
          embedList = seat.embed_origins;
          brand = seat.brand ?? null;
          live = seat.live;
          words = joinWordsOf(m.packs);
          seatKnown = true;
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
      let layoutKey = "";
      replica.onSession = (s) => {
        session = s;
        sessionReady = true;
        shown = shownFor(replica);
        party = partyOf(replica.party());
        perfEdge = s.device();
        layoutKey = layoutNow();
        void useScreen(s).then(refresh).catch((e) => (sceneErrors = [String(e)]));
      };
      // The role's layout (PB7 role card, PB13): a change of it, or of the made layout it names, reloads the screen.
      const layoutNow = () => {
        const asked = (shown && "scope" in shown ? shown.screen : undefined) ?? seatScreen;
        return `${asked ?? ""}|${asked?.startsWith("mine.") ? JSON.stringify(replica.layouts.find((d) => (d as unknown as ScreenDoc).id === asked) ?? null) : ""}`;
      };
      replica.onUpdate = () => {
        const mine = replica.seats().find((x) => x.id === seatId);
        if (!new URLSearchParams(location.search).get("screen")) seatScreen = mine ? (mine.screens?.[0] ?? null) : seatScreen;
        if (JSON.stringify(mine?.live ?? null) !== JSON.stringify(live ?? null)) live = mine?.live;
        const ship = partyOf(replica.party());
        if (JSON.stringify(ship) !== JSON.stringify(party)) party = ship;
        const sent = shownFor(replica);
        if (JSON.stringify(sent ?? null) !== JSON.stringify(shown ?? null)) {
          const was = shown && "scope" in shown ? shown.screen : undefined;
          shown = sent;
          const now = sent && "scope" in sent ? sent.screen : undefined;
          // Another screen named: load it before the scene is built for it.
          if (was !== now && session) return void useScreen(session).then(refresh).catch((e) => (sceneErrors = [String(e)]));
        }
        const key = layoutNow();
        if (key !== layoutKey && session && screenDoc) {
          layoutKey = key;
          return void useScreen(session).then(refresh).catch((e) => (sceneErrors = [String(e)]));
        }
        if (session && screenDoc) void refresh();
      };
      replica.join(hello);
      // Ask again until the GM answers (the GM's page may open after this one).
      const ask = setInterval(() => (link.state === "joining" ? replica.join(hello) : clearInterval(ask)), 1500);
      setInterval(() => replica.tick(), 1000);
    });
  } else {
    Promise.all([loadFacts(), ...stackDoc.packs.map(async (p) => (await fetch(new URL(p, document.baseURI))).json())])
      .then(([, ...packs]) => openBrowserSession({ base: document.baseURI, seat: gmSeat(), universe: facts.universe, packs }))
      .then(async (s) => {
        const bad = await enterFacts(s, facts);
        if (bad) throw new Error(`recorded facts refused: ${JSON.stringify(bad)}`);
        session = s;
        perfEdge = s.device();
        await useScreen(s);
        await refresh();
        stateHash = await s.stateHash();
      })
      .catch((e) => (sceneErrors = [String(e)]));
  }

  let selection = $state<string | null>(null);
  // Read only (PB3b): the GM steers this screen; it watches and takes no input but sound.
  // Words sent with what is shown (PB6): a card over the picture until the GM takes them back or the player puts it away.
  const sayWords = $derived(shown && "scope" in shown && shown.say ? shown.say : "");
  let sayHidden = $state("");
  const readOnly = $derived(!!shown && "scope" in shown && shown.steer === "read");
  // A send that names a thing (a bookmark's selection, PB3) selects it here, once per send, if this screen's scene has it.
  let focusSent = "";
  $effect(() => {
    const f = shown && "scope" in shown ? (shown.focus ?? "") : "";
    const key = f ? `${f}|${shown && "scope" in shown ? shown.title : ""}` : "";
    if (key === focusSent) return;
    if (f && !scene?.subjects.some((x) => x.address === f)) return;
    focusSent = key;
    if (f) selection = f;
  });
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
  // The performance readout (PF1b): the same switch as the chart room's, for players' own devices.
  let perfOn = $state(loadPref("perf.readout") === "on");
  perf.setEnabled(loadPref("perf.readout") === "on");
  let perfEdge = $state<{ tier: string; unitsPerSecond: number } | undefined>(undefined);
  $effect(() => {
    perf.setEnabled(perfOn);
    savePref("perf.readout", perfOn ? "on" : "off");
  });
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

  // Light (O1): players see light only from facts their data view gives them.
  const LIGHT_RECIPES = (lightDoc.recipes as unknown as BandRecipe[]).filter((r) => !checkLightRecipe(r).length);
  let lightId = $state(SYMBOLS);
  const lit = $derived(scene ? describeLight(litFromPlayers(scene)) : { lights: [], missing: [] });
  const lightNames = $derived(new Map((scene?.subjects ?? []).map((x) => [x.address, subjectName(x)])));
  const lightColours = $derived.by(() => {
    const r = LIGHT_RECIPES.find((x) => x.id === lightId);
    return r ? paint(r, lit.lights) : null;
  });
  const mark = (x: PlayerScene["subjects"][number]) => {
    const m = marker.mark(playerSubjectOf(x));
    return lightColours ? { ...m, colour: lightColours.get(x.address) ?? null } : m;
  };
  // A place by the name the table knows it by: from this scene when it is here, else the last part of its address.
  function placeName(a: string): string {
    const hit = scene?.subjects.find((x) => x.address === a);
    if (hit) return subjectName(hit);
    const last = a.split("/").pop() ?? a;
    return last.charAt(0).toUpperCase() + last.slice(1).replace(/[-_]/g, " ");
  }
  const dayWords = (d: number) => (d < 2 ? `${Math.round(d * 24)} hours` : d < 400 ? `${Math.round(d)} days` : `${(d / 365.25).toFixed(1)} years`);
  const star = $derived(scene ? scene.subjects.find((s) => s.address === scene!.scope) : undefined);

  // The tabletop bridge: tell the allowed tabletop the view is ready and what is in focus; take its "focus this" asks for subjects this seat already sees.
  let bridge = $state<EmbedBridge | null>(null);
  $effect(() => {
    if (embed.mode !== "embedded") return;
    const b = new EmbedBridge(embed.origin, window.parent, (f) => {
      const h = (e: MessageEvent) => f({ origin: e.origin, source: e.source, data: e.data });
      addEventListener("message", h);
      return () => removeEventListener("message", h);
    });
    b.onAsk((a) => {
      if (scene?.subjects.some((x) => x.address === a.address)) selection = a.address;
    });
    b.send({ source: "sse4", v: 1, type: "ready", seat: seatLabel, audience });
    bridge = b;
    return () => {
      b.close();
      bridge = null;
    };
  });
  $effect(() => {
    const t = scene && selection ? scene.subjects.find((x) => x.address === selection) : undefined;
    bridge?.send({ source: "sse4", v: 1, type: "focus", address: t?.address ?? null, name: t ? subjectName(t) : null });
  });
</script>

<div class="page" class:bare class:clean class:clear={clearGround} style={rootFx} data-embed={embed.mode} data-clean={clean ? "" : undefined}>
  {#if !bare}
  <header class="bar">
    {#if brand}<span class="brand" data-brand>{#if brand.logo}<img src={brand.logo} alt="" />{/if}{brand.name}</span>{/if}
    <h1>{shown && "off" in shown ? "Stand by" : star ? subjectName(star) : "System"}</h1>
    <span class="kicker">{screenDoc?.label ?? "…"}{screenDoc?.data_view ? ` · seen through ${screenDoc.data_view}` : ""} · audience {scene?.audience ?? "…"}</span>
    {#if party}<span class="ship" data-party-ship={party.ship} data-party-at={party.at} title={party.voyage ? `Last voyage: ${placeName(party.voyage.from)} to ${placeName(party.voyage.to)}, ${dayWords(party.voyage.days)}` : undefined}>Aboard <strong>{party.name}</strong>, {party.at === scopeNow() ? "in this system" : `at ${placeName(party.at)}`}</span>{/if}
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
    <label class="pick" title="Time this device, for the minimum spec; nothing is sent"><input type="checkbox" bind:checked={perfOn} data-perf-switch /> Performance</label>
    {#if joined && link.state !== "live"}
      <div class="banner" role="status" aria-label="Game master" data-link={link.state}>
        {#if link.state === "joining"}{say(words.joining)}
        {:else if link.state === "resync"}Catching up with the game master ({link.reason})…
        {:else if link.state === "offline"}{say(words.away)}
        {:else if link.state === "reload"}<strong>{link.reason}</strong> <button type="button" onclick={() => location.reload()}>Reload</button>{/if}
      </div>
    {/if}
  </header>
  {/if}
  {#if embed.mode === "refused"}
    <main class="problem" role="alert"><h2>This view cannot be shown here</h2><p>{embed.reason}</p></main>
  {:else if shown && "off" in shown}
    {#if shown.card}
      <main class="problem off held" data-screens-off data-held={shown.card}><p class="card">{shown.card}</p></main>
    {:else}
      <main class="problem off" data-screens-off><p>The game master has the screens off for now.</p></main>
    {/if}
  {:else if (checked && !checked.ok) || sceneErrors.length}
    <main class="problem" role="alert"><h2>This screen cannot be shown</h2><ul>{#each [...(checked && !checked.ok ? checked.errors : []), ...sceneErrors] as e (e)}<li>{e}</li>{/each}</ul></main>
  {:else if !scene || !checked?.ok}
    <main class="problem" aria-busy="true"><p data-join-words>{say(words.waiting)}</p><p class="alpha" data-alpha>An alpha preview of what is coming. To use Star System Explorer for real today, use <a href={V3_URL} target="_blank" rel="noopener">version 3 ›</a></p></main>
  {:else}
    {#key vocabId}
      <main class="grid" class:read-only={readOnly} data-read-only={readOnly ? "" : undefined} inert={readOnly}><ScreenRuntime screen={checked.screen} {scene} {vocab} {mark} {live} landing={landingNow} {ground} shown={shownNow} still={clean} {places} bind:selection /></main>
      {#if readOnly && !clean}<p class="steered" data-steered>The game master is steering this screen</p>{/if}
    {/key}
    {#if sayWords && sayHidden !== sayWords}
      <!-- A Read aloud sent to screens (PB6, design 4.3): plain words on a card over the picture. The player may put it away; new words bring it back. -->
      <aside class="say-card" data-say role="status" aria-live="polite">
        <p>{sayWords}</p>
        {#if !clean}<button type="button" class="say-hide" data-say-hide onclick={() => (sayHidden = sayWords)}>Put away</button>{/if}
      </aside>
    {/if}
  {/if}
  {#if joined && scene && mayAsk && !bare && !watching && !readOnly && !(shown && "off" in shown)}
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
  {#if scene && !bare}<div class="soundbar"><LightControl recipes={LIGHT_RECIPES} lights={lit.lights} missing={lit.missing} names={lightNames} bind:value={lightId} key="light.players" /><SoundControl scene={orbitScene(scene.scope, voicesFromPlayers(scene))} key="sound.players" /></div>{/if}
  <footer class="status" hidden={bare} data-state-hash={stateHash} data-scene-hash={sceneHash} data-sealed={isSealed || undefined} data-seat={joined ? seatId : undefined} data-audience={audience} data-link={joined ? link.state : "local"} data-via={via}>Scene: live from {isSealed ? "the game master's machine (sealed seat: this screen holds no seed and runs no kernel)" : joined ? "this screen's kernel, in step with the game master" : "the kernel"} {via ? ` (via ${via === "relay" ? "the site's relay" : "this browser"})` : ""} for {seatLabel && seatId !== "table" ? `the seat ${seatLabel}, ` : ""}the {audience} audience{sceneHash ? ` · scene ${sceneHash.slice(0, 10)}` : ""}{stateHash ? ` · state ${stateHash.slice(0, 10)}` : ""} {joined ? "" : screenDoc?.data_view ? ` · fixed to the ${screenDoc.data_view} view by the screen document` : ` · the Scanned control stands in for the GM's button and taps it for ${target ? subjectName(target) : "the body in focus"}`}</footer>
</div>
{#if perfOn}
  <PerfReadout kernel="" edge={perfEdge} onclose={() => (perfOn = false)} />
{/if}
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
  .brand {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: var(--sse-font-display);
    font-weight: 600;
    color: var(--sse-ink);
  }
  .brand img {
    height: 24px;
    width: auto;
    max-width: 64px;
    object-fit: contain;
  }
  .page.bare {
    grid-template-rows: 1fr;
  }
  /* The clean output: a 16:9 picture, letterboxed in whatever window the stream gives it, no pointer. */
  .page.clean {
    width: min(100vw, calc(100vh * 16 / 9));
    height: min(100vh, calc(100vw * 9 / 16));
    min-height: 0;
    margin: auto;
    overflow: hidden;
    cursor: none;
  }
  .page.clear {
    background: transparent;
  }
  :global(body.sse-clear-ground),
  :global(html:has(body.sse-clear-ground)) {
    background: transparent;
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
  .ship {
    font-size: 13px;
    color: var(--sse-ink-muted);
    white-space: nowrap;
  }
  .ship strong {
    color: var(--sse-ink);
    font-weight: 600;
  }
  .alpha {
    margin-top: 18px;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .alpha a {
    color: var(--sse-accent);
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
  .soundbar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px 18px;
    padding: 6px 24px 0;
    font-family: var(--sse-font-body);
    font-size: 12px;
    color: var(--sse-ink-muted);
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
  /* A holding card (Switchboard): one line of words in the middle of the glass, arriving by a fade until looks exist. */
  .held {
    min-height: 60vh;
    display: grid;
    place-items: center;
    text-align: center;
  }
  .held .card {
    font-family: var(--sse-font-mono);
    font-size: clamp(16px, 2.4vw, 28px);
    letter-spacing: 0.04em;
    max-width: 36ch;
    margin: 0;
    animation: card-in var(--sse-motion) var(--sse-ease);
  }
  @keyframes card-in {
    from {
      opacity: 0;
    }
  }
  @media (prefers-reduced-motion: reduce) {
    .held .card {
      animation: none;
    }
  }
  @media (max-width: 760px) {
    .bar {
      padding: 10px 12px;
    }
    .grid {
      padding: 12px;
    }
  }
  .say-card {
    position: fixed;
    left: 50%;
    bottom: 64px;
    transform: translateX(-50%);
    width: min(640px, calc(100vw - 32px));
    margin: 0;
    padding: 16px 20px 12px;
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-ink);
    border-radius: var(--sse-radius);
    box-shadow: var(--sse-card-shadow);
    z-index: 5;
    animation: card-in var(--sse-motion) var(--sse-ease);
  }
  .say-card p {
    margin: 0 0 8px;
    font-family: var(--sse-font-display);
    font-size: 20px;
    line-height: 1.4;
    white-space: pre-wrap;
  }
  .say-hide {
    font: inherit;
    font-size: 12px;
    color: var(--sse-ink-muted);
    background: transparent;
    border: var(--sse-line) solid var(--sse-rule);
    border-radius: var(--sse-radius);
    padding: 2px 8px;
    cursor: pointer;
  }
  @media (prefers-reduced-motion: reduce) {
    .say-card {
      animation: none;
    }
  }
  .steered {
    position: fixed;
    right: 16px;
    bottom: 12px;
    margin: 0;
    padding: 3px 10px;
    font-size: 12px;
    letter-spacing: 0.04em;
    color: var(--sse-ink-muted);
    background: var(--sse-ground-raised);
    border: var(--sse-line) solid var(--sse-rule);
    border-radius: var(--sse-radius);
    pointer-events: none;
  }
</style>
