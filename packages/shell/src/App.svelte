<!--
  The GM shell: a fixed, curated chart-room layout (H8.8) whose areas are
  filled from a screen document with view modules. The shell reaches the
  kernel only through the session, the same client API a player app uses.
-->
<script lang="ts">
  import { tick, untrack } from "svelte";
  import { stackProvides, railOf, scalesOf, type RailScale, DEPTHS, isDepth, type Depth, cellHalo, readViewChoice, type ViewChoice, nextTrail, parsePlaceHash, placeHash, placeRank, placeSteps, readRecent, rememberPlace, samePlace, PLACE_EVENT, PLACE_SCALES, PLACE_WORDS, type Place, type PlaceNote, type PlaceScale, type Shown } from "../../ui-contract/src/index.ts";
  import { PLACE_SCREENS } from "./documents/scales.ts";
  import { applyEdits, applySetup, type AtomInfo, type SifterDoc, enterFacts, observeOf, type AuthoredFacts, checkScreen, deriveScene, gmSeat, redoEdit, subjectOf, systemOrder, undoEdit, type CheckedScreen, type DerivationDoc, type ScreenDoc, type SystemScene, type ViewHandle, type ViewModule, type ViewSession } from "../../ui-contract/src/index.ts";
  import { openBrowserSession } from "../../ui-contract/src/host-browser.ts";
  import { createShare, type Share } from "./share.svelte.ts";
  import { clearStored, download, readStored, setAside, writeStored } from "./campaign-store.ts";
  import YourData from "./YourData.svelte";
  import { downloadText, opening as markOpening, restoreSetAside, setAsideText, unfinishedOpen, watchProblems, type DiagnosticState } from "./your-data.ts";
  import { checkCampaignSave, makeCampaignSave, saveFileName, savedEvents, type CampaignSave } from "../../ui-contract/src/index.ts";
  import { CONVERTER_URL, checkPackDoc, creditWords, fetchExplorers, previewStack, reportWords, sniffFile, stackPacks as addPacks, v3SaveText, type ExplorersListing, type PackDoc, type Sniffed, type StackPreview } from "../../ui-contract/src/index.ts";
  import TableStatus from "./run/TableStatus.svelte";
  import LensPanel from "./LensPanel.svelte";
  import SoundControl from "../../sound/src/SoundControl.svelte";
  import LightControl from "./LightControl.svelte";
  import MakeSystem from "./MakeSystem.svelte";
  import AddHere from "./AddHere.svelte";
  import StartSheet, { type StartAction } from "./StartSheet.svelte";
  import HelpDoor, { markNewsSeen, unseenNews, type HelpPage } from "./HelpDoor.svelte";
  import startDoc from "./documents/start.lines.json";
  import LookAhead from "./LookAhead.svelte";
  import { readSol, solBatches, solSourceText, SOL_TITLE, type SolSheet } from "../../ui-contract/src/index.ts";
  import { listedTitle, namesInLayer, relabel, sceneAt, sceneFromResolved, type LookedAhead, type SystemSpec } from "../../ui-contract/src/index.ts";
  import lightDoc from "./documents/light.recipes.json";
  import { SYMBOLS, checkLightRecipe, describeLight, lightForBand, litFromSystem, paint, readLight, recipeBand, type ViewBand, type BandRecipe, type Mark } from "../../ui-contract/src/index.ts";
  import AgentsPanel from "../../agents/src/AgentsPanel.svelte";
  import type { OracHost } from "../../agents/src/OracTab.svelte";
  import type { MinervaHost } from "../../agents/src/MinervaTab.svelte";
  import type { TarsHost } from "../../agents/src/TarsTab.svelte";
  import { whyPassage } from "../../agents/src/tars.ts";
  import storyStarters from "../../views/story-finder/src/starters.json";
  import { showReference } from "../../ui-contract/src/index.ts";
  import { oracMay } from "../../agents/src/orac.ts";
  import { atomIssueTarget, flowEntries, lensEntries, lensTarget, loadAtoms, physicsTarget } from "../../agents/src/orac-targets.ts";
  import { programTarget, ruleEntries, sifterEntries, sifterTarget } from "../../agents/src/orac-rules.ts";
  import { blockEntries, boxTarget, tableTarget } from "../../agents/src/orac-blocks.ts";
  import { handbookEntries, sheetEntries, substanceTarget } from "../../agents/src/orac-sheet.ts";
  import { templateTarget, triggerEntries, triggerTarget } from "../../agents/src/orac-triggers.ts";
  import { placeEntries, revealTarget } from "../../agents/src/orac-known.ts";
  import { dataViewEntries, dataViewTarget, lookEntries, screenTarget, styleTarget, symbologyTarget } from "../../agents/src/orac-looks.ts";
  import { orbitScene, voicesFromSystem, bookmarksFrom, layoutsFrom, makeBookmark, playbookFrom, type Bookmark, type Whereabouts } from "../../ui-contract/src/index.ts";
  import { orbitBodies, phasesFrom, withPhases, yearsToSeconds } from "../../ui-contract/src/index.ts";
  import { compareScenes, dropWhatIf, keepWhatIf, openWhatIf } from "../../ui-contract/src/index.ts";
  import { readStarFacts, withStarFacts, type StarFacts } from "../../ui-contract/src/index.ts";
  import { AU_M, placeBodies as placePasted, withdrawBodies, copyClip, missingPacks, placeClip, primaryOf, readClip, clipWords, type Clip } from "../../ui-contract/src/index.ts";
  import { EVERYONE, allLenses, checkLens, defaultLens, hasTool, lensAudience, type Lens, type Vocab } from "../../ui-contract/src/index.ts";
  import { actionById, keyWords, menuFor, type ActionTarget } from "../../ui-contract/src/index.ts";
  import ObjectMenu from "./ObjectMenu.svelte";
  import { watchObjectMenu } from "./object-menu.ts";
  import { themeCss, THEMES } from "../../skins/chart-room/src/index.ts";
  import { ALL_THEMES } from "../../skins/player/src/index.ts";
  import { Segmented, ToolButton, ViewControls, formatYears } from "../../chart-grammar/src/index.ts";
  import PlaceLine from "./nav/PlaceLine.svelte";
  import Rail from "./nav/Rail.svelte";
  import Stage from "./chart/Stage.svelte";
  import workspaceDoc from "./documents/workspace.gm.json";
  import physicsDoc from "./documents/physics.gm.json";
  import packsDoc from "./documents/packs.gm.json";
  import referenceDoc from "./documents/reference.gm.json";
  import screensDoc from "./documents/screens.gm.json";
  import dataDoc from "./documents/dataviews.gm.json";
  import symbolsDoc from "./documents/symbols.gm.json";
  import bodyDoc from "./documents/body.gm.json";
  import compareDoc from "./documents/compare.gm.json";
  import { POSTCARD_SCREENS } from "./postcards/screens.postcards.svelte.ts";
  import { visit, GO_BOOKMARK_EVENT, PASTE_CLIP_EVENT } from "./postcards/visits.svelte.ts";
  import { checkPostcard, looksLikePostcard, readPostcard } from "../../ui-contract/src/index.ts";
  import lifeDoc from "./documents/life.gm.json";
  import stylesScreenDoc from "./documents/styles.gm.json";
  import routeDoc from "./documents/route.gm.json";
  import windowDoc from "./documents/window.gm.json";
  import storiesDoc from "./documents/stories.gm.json";
  import cutawayDoc from "./documents/cutaway.gm.json";
  import { createStyles } from "./styles.svelte.ts";
  import { PLAYER_STACK } from "../../views/player-stack/src/index.ts";
  import oracDoc from "./documents/orac.requests.json";
  import sceneDoc from "./documents/kestrel.scene.json";
  import worldsDoc from "./documents/kestrel.worlds.json";
  import derivationsDoc from "./documents/derivations.base.json";
  import stackDoc from "./documents/stack.json";
  import factsDoc from "./documents/kestrel.player-facts.json";
  import plainVocab from "./documents/vocab.plain.json";
  import { REGISTRY } from "./registry.ts";
  import { MORE_SCREENS } from "./screens.more.ts";
  import { GAME_SCREENS } from "./screens.game.ts";
  import { RENDER_SCREENS } from "./screens.renderer.ts";
  import { RUN_SCREENS } from "./run/screens.run.ts";
  import Bookmarks from "./run/Bookmarks.svelte";
  import OnAir from "./run/OnAir.svelte";
  import { tablePort } from "./run/table-port.ts";
  import { createWorkspace } from "./workspace.svelte.ts";
  import { loadPref, savePref } from "./prefs.ts";
  import { createSymbols } from "./symbols.svelte.ts";
  import PerfReadout from "./PerfReadout.svelte";
  import { perf } from "../../ui-contract/src/index.ts";
  import type { ChangeRequest, Json } from "../../ui-contract/src/index.ts";

  // World stacks (P4) ride on their subjects; until the kernel serves them they come from a fixture document.
  const worlds = new Map((worldsDoc as unknown as { links: { body: string; fixture: string }[] }).links.map((w) => [w.body, w.fixture]));
  const base = { ...(sceneDoc as unknown as SystemScene), subjects: (sceneDoc as unknown as SystemScene).subjects.map((x) => (worlds.has(x.address) ? { ...x, world_fixture: worlds.get(x.address) } : x)) };
  const derivations = derivationsDoc as unknown as DerivationDoc;
  const ws = createWorkspace(base.now_yr);
  // The standard view controls (rule 12): the viewer's view style and "seen from", kept on this device for every
  // diagram; a view turned by hand becomes Free.
  ws.view = readViewChoice(loadPref("view"));
  function setView(c: ViewChoice) {
    ws.view = c;
    savePref("view", JSON.stringify(c));
  }
  ws.turned = () => {
    if (ws.view?.seen !== "free") setView({ ...ws.view!, seen: "free" });
  };
  // A view's own press may set the choice for the viewer (the size comparison's Gallery opens in True).
  ws.pickView = (c) => setView(c);
  // Orac is not connected yet (A2): its sample requests come from a fixture document.
  for (const r of (oracDoc as unknown as { requests: ChangeRequest[] }).requests) ws.fileRequest(r);
  let scene = $state<SystemScene>(base);

  // Each screen document is checked before anything mounts; the GM switches between them.
  const SCREENS: Record<string, ScreenDoc> = { system: workspaceDoc as unknown as ScreenDoc, physics: physicsDoc as unknown as ScreenDoc, packs: packsDoc as unknown as ScreenDoc, reference: referenceDoc as unknown as ScreenDoc, screens: screensDoc as unknown as ScreenDoc, data: dataDoc as unknown as ScreenDoc, symbols: symbolsDoc as unknown as ScreenDoc, body: bodyDoc as unknown as ScreenDoc, compare: compareDoc as unknown as ScreenDoc, life: lifeDoc as unknown as ScreenDoc, styles: stylesScreenDoc as unknown as ScreenDoc, route: routeDoc as unknown as ScreenDoc, window: windowDoc as unknown as ScreenDoc, stories: storiesDoc as unknown as ScreenDoc, cutaway: cutawayDoc as unknown as ScreenDoc, ...Object.fromEntries(MORE_SCREENS.map((m) => [m.value, m.doc])), ...Object.fromEntries(GAME_SCREENS.map((m) => [m.value, m.doc])), ...Object.fromEntries(RENDER_SCREENS.map((m) => [m.value, m.doc])), ...Object.fromEntries(RUN_SCREENS.map((m) => [m.value, m.doc])), ...Object.fromEntries(POSTCARD_SCREENS.map((m) => [m.value, m.doc])), ...PLACE_SCREENS };
  const checks = Object.fromEntries(Object.entries(SCREENS).map(([k, d]) => [k, checkScreen(d, REGISTRY)]));
  const badScreens = Object.values(checks).flatMap((c) => (c.ok ? [] : c.errors));
  const screen = $derived.by((): CheckedScreen | null => {
    const c = checks[ws.screen] ?? checks.system!;
    return c.ok && !badScreens.length ? c.screen : null;
  });
  const mountFor = (area: string) => screen?.mounts.find((m) => m.area === area);
  /** What the chart's view draws in each view style, when it is a diagram (rule 12). */
  const viewSpec = $derived(mountFor("chart")?.module.controls);
  // A view that opens on its own scale (the size comparison at Real) gets it each time it is opened; the viewer's own
  // scale comes back when they leave.
  let keptScale: ViewChoice["scale"] | null = null;
  $effect(() => {
    const opens = viewSpec?.opens?.scale;
    untrack(() => {
      if (opens && ws.view && keptScale === null) {
        keptScale = ws.view.scale ?? "readable";
        ws.view = { ...ws.view, scale: opens };
      } else if (!opens && keptScale !== null && ws.view) {
        setView({ ...ws.view, scale: keptScale });
        keptScale = null;
      }
    });
  });
  /** The place in view, as the table would be shown it: on the Land screen, with the landing the GM is flying. */
  let lastLanding = $state.raw<Json | null>(null);
  const LAND_SCREEN = "players.land.screen.json";
  // A send made on the Land screen before its first landing arrived waits for it, then goes again with it.
  let landingAwaited = false;
  const here = (): Shown => {
    landingAwaited = ws.screen === "surface" && !lastLanding;
    const focus = ws.selection ? { focus: ws.selection } : {};
    return ws.screen === "surface" && lastLanding ? { scope: scene.system, title: `${selName ?? scene.title}, landing`, screen: LAND_SCREEN, landing: lastLanding, ...focus } : { scope: scene.system, title: scene.title, ...focus };
  };
  // Driving a read-only screen (PB3b): as the GM moves, the screen follows. Never while on the Switchboard itself.
  $effect(() => {
    if (!share?.state.board.driving || ws.screen === "switchboard") return;
    const live = here();
    untrack(() => share?.follow(live));
  });
  // The kicker gives way whole when the bar is too full to say anything with it ("IN…" helped nobody).
  let kickerW = $state(999);

  // The instrument rail (top-level design 2.2) at the scale in view: system, world, or the Workshop.
  let railScale = $state<RailScale>("system");
  // A rail entry that `needs` something (a game pack's code sheets) shows only while a stacked pack gives it.
  const stackHas = $derived.by(() => {
    void ws.revision;
    const st = session?.stack();
    const packs = st ? [...st.base, ...(st.campaign ? [st.campaign] : [])] : [];
    return (need: string) => stackProvides(packs, need);
  });
  const rail = $derived(railOf(SCREENS, railScale, stackHas));
  // A screen opened from elsewhere (a why?'s "Open the flow", a key) takes the rail to a scale that lists it.
  $effect(() => {
    const on = scalesOf(SCREENS[ws.screen] ?? {});
    untrack(() => {
      if (on.length && !on.includes(railScale)) railScale = on[0]!;
    });
  });
  function go(id: string) {
    ws.screen = id;
  }
  /** Down a step: the world selected (or the one given), on a world instrument. */
  function toWorld(address?: string) {
    if (address) ws.select(address);
    railScale = "world";
    if (!scalesOf(SCREENS[ws.screen] ?? {}).includes("world")) ws.screen = "body";
  }
  /** Up to the whole system, on a system instrument. */
  function toSystem() {
    if (cellHalo(ws.selection)) ws.select(null);
    railScale = "system";
    if (!scalesOf(SCREENS[ws.screen] ?? {}).includes("system")) ws.screen = "system";
  }
  function toWorkshop() {
    railScale = "workshop";
  }
  /** Out to a scale above the system (U24): the galaxy, the galaxies nearby, the cosmic web, on that scale's first instrument. */
  function toScale(scale: PlaceScale) {
    if (scale === "system") return toSystem();
    if (scale === "world") return toWorld();
    railScale = scale;
    const first = railOf(SCREENS, scale)[0]?.lines[0]?.screen;
    if (first && !scalesOf(SCREENS[ws.screen] ?? {}).includes(scale)) ws.screen = first;
  }

  function goTo(v: string) {
    if (v === "@system") return toSystem();
    if (v) toWorld(v);
  }
  // Bookmarks (PB2b): where the GM is and how they look, and back again exactly as it was left.
  // The instrument a bookmark keeps: the Switchboard is where bookmarks are used, never where they point.
  let lastInstrument = "system";
  $effect(() => {
    if (ws.screen !== "switchboard") lastInstrument = ws.screen;
  });
  const instrumentNow = () => (ws.screen === "switchboard" ? lastInstrument : ws.screen);
  // The camera goes on the bookmark only while the view that keeps it is in front (PC1).
  const cameraNow = () => (ws.camera && ws.screen !== "switchboard" ? ws.camera : instrumentNow() === lastCameraScreen ? lastCamera : null);
  let lastCamera: Whereabouts["pose"] = null;
  let lastCameraScreen = "";
  $effect(() => {
    if (ws.screen !== "switchboard") {
      lastCamera = ws.camera ?? null;
      lastCameraScreen = ws.screen;
    }
  });
  const whereabouts = (): Whereabouts => ({ scope: scene.system, now_yr: ws.now_yr, instrument: instrumentNow(), scale: railScale, ...(ws.view ? { view: ws.view } : {}), selected: ws.selection, pose: cameraNow() });
  function markHere(): Bookmark {
    const words = SCREENS[instrumentNow()]?.label ?? instrumentNow();
    return makeBookmark(ws.bookmarks ?? [], whereabouts(), railScale === "world" || railScale === "system" ? (selName ?? scene.title) : scene.title, String(words));
  }
  ws.markHere = markHere;
  function goBookmark(b: Bookmark) {
    ws.select(b.selected);
    if (b.scale) railScale = b.scale as RailScale;
    ws.screen = b.instrument;
    if (b.at_yr !== null) ws.now_yr = b.at_yr;
    if (b.view) setView({ ...b.view, framed: (ws.view?.framed ?? 0) + 1 });
    ws.aim = b.pose ?? null;
  }
  // Postcards (PC3, PC4): the wall's Go there and Keep this system, and a postcard opened from outside as a visit.
  $effect(() => {
    const go = (e: Event) => {
      const r = checkPostcard({ schema: "sse.postcard/1", id: "go", name: "go", bookmark: (e as CustomEvent<unknown>).detail, world: { how: "here" } });
      if (r.ok) goBookmark(r.postcard.bookmark);
    };
    const paste = (e: Event) => {
      const t = (e as CustomEvent<unknown>).detail;
      if (typeof t !== "string") return;
      if (desk !== "build") chooseDesk("build");
      pasteText(t);
    };
    window.addEventListener(GO_BOOKMARK_EVENT, go);
    window.addEventListener(PASTE_CLIP_EVENT, paste);
    return () => {
      window.removeEventListener(GO_BOOKMARK_EVENT, go);
      window.removeEventListener(PASTE_CLIP_EVENT, paste);
    };
  });
  function openPostcard(text: string): boolean {
    const r = readPostcard(text);
    if (!r.ok) {
      clipNote = r.why;
      return false;
    }
    visit(r.postcard);
    ws.screen = "postcards";
    return true;
  }
  // A postcard link (`#postcard=…`) opens as a visit once the session is up; the address bar then drops it.
  const linkedPostcard = typeof location === "undefined" || !/[#&]postcard=/.test(location.hash) ? null : location.hash;
  let postcardTaken = false;
  $effect(() => {
    if (!session || postcardTaken || !linkedPostcard) return;
    postcardTaken = true;
    untrack(() => {
      if (openPostcard(linkedPostcard)) history.replaceState(history.state, "", location.pathname + location.search);
    });
  });
  let moreOpen = $state(false);
  // The performance readout (PF1): off by default, remembered per device.
  let perfOn = $state(loadPref("perf.readout") === "on");
  // At once, so a readout left on also times the start-up calls.
  perf.setEnabled(loadPref("perf.readout") === "on");
  let moreBtn = $state<HTMLElement | null>(null);
  $effect(() => {
    perf.setEnabled(perfOn);
    savePref("perf.readout", perfOn ? "on" : "off");
  });

  let theme = $state(loadPref("theme") === "night-chart" ? "night-chart" : "chart-room");
  // Symbols are chosen apart from the theme (H8.4A); the theme only says what it can carry.
  const previews = ALL_THEMES.map((t) => ({ id: t.id, name: t.name, style: themeCss(t.id, ALL_THEMES), capability: t }));
  const symbols = createSymbols((workspaceDoc as unknown as ScreenDoc).symbology, () => THEMES.find((t) => t.id === theme) ?? THEMES[0]!, () => previews);
  $effect(() => {
    const root = document.documentElement;
    root.setAttribute("style", themeCss(theme));
    root.style.colorScheme = THEMES.find((t) => t.id === theme)?.mode ?? "light";
    root.dataset.theme = theme;
    savePref("theme", theme);
  });

  let session = $state<ViewSession | null>(null);
  // Style documents (H8.9): checked for this device's measured tier.
  const styleHost = createStyles(() => [...REGISTRY, ...PLAYER_STACK] as never, () => session?.device()?.tier ?? "standard");
  let failure = $state<string | null>(null);
  watchProblems();
  let status = $state({ kernel: "", tier: "", hash: "", stack: "", campaign: false });
  async function loadPacks(): Promise<unknown[]> {
    const packs = await Promise.all(stackDoc.packs.map(async (p) => (await fetch(new URL(p, document.baseURI))).json()));
    perf.stamp("startup.packs", performance.now());
    return packs;
  }

  // Sharing with players (V4): recording starts as the campaign opens, before any fact is entered.
  let share = $state<Share | null>(null);
  // The party's ship (run-through): the table always knows where it is, so every change goes to the players' screens.
  // It never moves their view: that is the GM's "Send this view".
  $effect(() => {
    const p = ws.party ? ($state.snapshot(ws.party) as unknown as Json) : null;
    if (share) share.host.setState({ party: p });
  });
  // The layouts the GM made (PB13) go to the players' screens, so a role or a send can name one.
  $effect(() => {
    const l = $state.snapshot(ws.layouts ?? []) as unknown as Json[];
    if (share) share.host.setState({ layouts: l });
  });
  // The table for the Switchboard (PB2): the share, with the live view as its first source.
  $effect(() => {
    ws.table = share && session
      ? tablePort(share, () => here(), {
          session,
          bodies: () => bodies.filter((b) => !b.parent || b.parent === bodies[0]?.address),
          scope: () => (factsDoc as unknown as AuthoredFacts).scope,
          at_yr: () => (factsDoc as unknown as AuthoredFacts).at_yr,
          onchange: () => (ws.revision++, ws.notePhysics()),
        })
      : undefined;
  });
  let opened: unknown[] = [];
  loadPacks()
    .then((packs) => ((opened = packs), openBrowserSession({ base: document.baseURI, seat: gmSeat(), universe: base.universe, packs })))
    .then(async (s) => {
      // U10: the campaign kept in this browser reopens first; players then join from it.
      const reopened = await reopenKept(s);
      const packs = [...opened, ...s.stack().added, ...(s.stack().campaign ? [s.stack().campaign] : [])];
      share = await createShare(s, { universe: base.universe, packs, ...(reopened ? { opening_export: reopened } : {}), clock_yr: ws.campaign_now_yr ?? ws.now_yr, views: () => observeOf(s.stack().base)?.setup.views.map((v) => v.id) ?? [], scope: (factsDoc as unknown as AuthoredFacts).scope });
      // The player-facing facts the GM has recorded, entered when a fresh campaign opens (a reopened one has them).
      if (!reopened) {
        const bad = await enterFacts(s, factsDoc as unknown as AuthoredFacts);
        if (bad) throw new Error(`recorded facts refused: ${JSON.stringify(bad)}`);
      }
      scene = await deriveScene(s, { ...base, now_yr: ws.now_yr }, derivations);
      // Systems already adopted into this universe (kept by the kernel across reloads) go on the "On the chart" list.
      const listed = await s.listSystems();
      if (listed.ok) adoptedSystems = listed.systems.filter((x) => x.system !== base.system && (x.source !== "facts" || x.system === `u:${base.universe.id}/sys:sol`)).map((x) => ({ system: x.system, title: listedTitle(x) }));
      session = s;
      status = { kernel: await s.kernelVersion(), tier: s.device()?.tier ?? "unmeasured", hash: await s.stateHash(), stack: s.stack().hash, campaign: false };
      perf.stamp("startup.first_chart", performance.now());
      // Opened from an Explorers item page: the link is read once, then leaves the address bar.
      const fromExplorers = new URLSearchParams(location.search).get("explorers");
      if (fromExplorers) {
        const u = new URL(location.href);
        u.searchParams.delete("explorers");
        history.replaceState(history.state, "", u.pathname + u.search + u.hash);
        void openFromExplorers(fromExplorers);
      }
    })
    .catch((e) => (failure = String(e)));

  // Keeping the campaign (U10): the kernel's export plus the workspace overlay, in this browser and as a file.
  const ROOT = `u:${base.universe.id}`;
  let keptNote = $state<string | null>(null);
  let savedAt = $state<string | null>(null);
  let saveProblem = $state<string | null>(null);
  // A kept campaign that failed to open is set aside, never lost, and the recovery note offers it back (V3 gap 27).
  let recovery = $state(false);
  async function reopenKept(s: ViewSession): Promise<Json | null> {
    const stuck = unfinishedOpen();
    if (stuck) {
      setAside();
      markOpening(false);
      recovery = true;
      keptNote = `Last time, opening the campaign kept in this browser did not finish (started ${new Date(stuck).toLocaleString("en-GB", { dateStyle: "medium", timeStyle: "short" })}). It is set aside, not lost, and a fresh campaign given.`;
      return null;
    }
    const { save, problem } = readStored(ROOT);
    if (problem) {
      setAside();
      recovery = true;
      keptNote = `The campaign kept in this browser could not be reopened (${problem}). It is set aside, not lost, and a fresh campaign given.`;
      return null;
    }
    if (!save) return null;
    markOpening(true);
    try {
      return await reopenSave(s, save);
    } finally {
      markOpening(false);
    }
  }
  async function reopenSave(s: ViewSession, save: CampaignSave): Promise<Json | null> {
    if (save.stack?.added.length) await s.send({ kind: "stack_packs", packs: save.stack.added });
    if (save.stack?.campaign) await s.send({ kind: "adopt_physics", pack: save.stack.campaign, requests: [] });
    const im = await s.importChronicle(save.export);
    if (!im.ok) {
      setAside();
      recovery = true;
      keptNote = `The campaign kept in this browser could not be reopened (${im.message}). It is set aside, not lost, and a fresh campaign given.`;
      if (save.stack) await s.send({ kind: "stack_packs", packs: [] }).then(() => s.send({ kind: "adopt_physics", pack: null, requests: [] }));
      return null;
    }
    ws.applied = save.workspace.applied;
    ws.undo = save.workspace.undo;
    ws.redo = save.workspace.redo;
    ws.now_yr = save.workspace.now_yr;
    ws.campaign_now_yr = save.workspace.campaign_now_yr ?? save.workspace.now_yr;
    ws.campaign_epoch_yr = save.workspace.campaign_epoch_yr ?? base.now_yr;
    ws.overlay_anchors = save.workspace.overlay_anchors ?? {};
    ws.unit_choices = save.workspace.unit_choices ?? {};
    ws.party = save.workspace.party ?? null;
    ws.bookmarks = bookmarksFrom(save.workspace.bookmarks);
    ws.playbook = playbookFrom(save.workspace.playbook);
    ws.layouts = layoutsFrom(save.workspace.layouts);
    savedAt = save.saved_at;
    const events = savedEvents(save);
    keptNote = `Reopened the campaign kept ${new Date(save.saved_at).toLocaleString("en-GB", { dateStyle: "medium", timeStyle: "short" })}: ${events} ${events === 1 ? "event" : "events"}.`;
    if (!im.stack_matches) keptNote += " The physics loaded now differs from the physics it was kept with: results may differ.";
    if (im.state_hash !== save.state_hash) keptNote += " Its state differs from when it was kept.";
    return save.export;
  }
  async function currentSave(): Promise<CampaignSave | null> {
    const s = session;
    if (!s) return null;
    const ex = await s.exportChronicle();
    if (!ex.ok) return null;
    const st = s.stack();
    return makeCampaignSave({
      title: base.title,
      root: ROOT,
      state_hash: await s.stateHash(),
      export: ex.export,
      workspace: JSON.parse(JSON.stringify({ applied: ws.applied, undo: ws.undo, redo: ws.redo, now_yr: ws.now_yr, campaign_now_yr: ws.campaign_now_yr, campaign_epoch_yr: ws.campaign_epoch_yr, overlay_anchors: ws.overlay_anchors ?? {}, unit_choices: ws.unit_choices ?? {}, party: ws.party ?? null, bookmarks: ws.bookmarks ?? [], playbook: ws.playbook, layouts: ws.layouts ?? [] })) as CampaignSave["workspace"],
      saved_at: new Date().toISOString(),
      ...(st.added.length || st.campaign ? { stack: { added: st.added as unknown as Json[], campaign: (st.campaign ?? null) as unknown as Json } } : {}),
    });
  }
  // Autosave: after each change of state (or of the ruler's time, or the undo stack), once things settle.
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    void status.hash;
    void status.stack;
    void ws.now_yr;
    void ws.campaign_now_yr;
    void ws.undo.length;
    void ws.redo.length;
    void ws.bookmarks;
    void ws.playbook;
    void ws.layouts;
    if (!session || !status.hash) return;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(async () => {
      const save = await currentSave();
      if (!save) return;
      const bad = writeStored(save);
      saveProblem = bad ? `Not kept in this browser: ${bad}. Use Save campaign to download it.` : null;
      if (!bad) savedAt = save.saved_at;
    }, 600);
  });
  async function saveFile() {
    const save = await currentSave();
    if (save) download(save, saveFileName(save.title, save.saved_at));
  }
  async function diagnosticState(): Promise<DiagnosticState> {
    const save = session ? await currentSave() : null;
    return {
      kernel: status.kernel,
      tier: status.tier,
      state_hash: status.hash,
      stack_hash: status.stack,
      campaign: { title: base.title, events: save ? savedEvents(save) : null, saved_at: savedAt },
      notes: [failure, keptNote, saveProblem].filter((x): x is string => !!x),
    };
  }
  let campaignDialog: HTMLDialogElement | undefined = $state();
  // Each opening reads Your data afresh (a save may have been set aside since the page loaded).
  let campaignOpens = $state(0);
  function openCampaign(pick = false) {
    campaignOpens++;
    campaignDialog?.showModal();
    if (pick) queueMicrotask(() => campaignDialog?.querySelector<HTMLInputElement>("[data-open-file]")?.click());
  }
  let offered = $state<{ save: CampaignSave | null; problems: string[]; name: string } | null>(null);
  let confirmFresh = $state(false);
  // The one Open door (V3 gap 28): any file, named by what is inside it, with the one way to open it.
  let given = $state<{ name: string; sniffed: Sniffed; text: string; bytes: Uint8Array; from?: ExplorersListing } | null>(null);
  let makeFile = $state<{ text: string; name: string } | null>(null);
  let packCheck = $state<StackPreview | null>(null);
  let packNote = $state<string | null>(null);
  async function offerFile(name: string, bytes: Uint8Array, from?: ExplorersListing) {
    const text = new TextDecoder().decode(bytes);
    const sniffed = sniffFile(name, text, bytes);
    given = { name, sniffed, text, bytes, ...(from ? { from } : {}) };
    offered = null;
    packCheck = null;
    packNote = null;
    if (sniffed.kind === "v4-campaign") {
      const problems = checkCampaignSave(sniffed.doc, ROOT);
      offered = { save: problems.length ? null : (sniffed.doc as CampaignSave), problems, name };
    }
    if (sniffed.kind === "pack" && session) {
      const info = session.stack();
      const bad = checkPackDoc(sniffed.doc, [...info.base, ...(info.campaign ? [info.campaign] : [])].map((p) => p.id));
      packCheck = bad ? { ok: false, problems: [bad], changes: [] } : await previewStack(session, scene, [...(info.added as PackDoc[]), sniffed.doc as PackDoc], { report: (sniffed.doc as PackDoc).id });
    }
  }
  async function pickFile(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (file) await offerFile(file.name, new Uint8Array(await file.arrayBuffer()));
  }
  async function openOpened() {
    const o = given;
    if (!o) return;
    const k = o.sniffed.kind;
    if (k === "v4-campaign") return reopenOffered();
    if (k === "v4-postcard") {
      campaignDialog?.close();
      return void openPostcard(o.text);
    }
    if (k === "v4-clip") {
      campaignDialog?.close();
      if (desk !== "build") chooseDesk("build");
      return pasteText(o.text);
    }
    if (k === "v3-save") {
      let text: string;
      try {
        text = await v3SaveText(o.bytes);
      } catch (e) {
        return void (given = { ...o, sniffed: { kind: "unknown", what: `That zip could not be read: ${(e as Error).message}.` } });
      }
      campaignDialog?.close();
      makeFile = { text, name: o.name };
      return openMake("import");
    }
    if (k === "pack" && session && packCheck?.ok) {
      const r = await addPacks(session, ws, $state.snapshot([...(session.stack().added as PackDoc[]), o.sniffed.doc as PackDoc]) as PackDoc[]);
      packNote = r.status === "applied" ? "Added. Every value it touches is worked out again and labelled." : r.status === "refused" ? r.reason : "Not added.";
    }
  }
  // Drop a file anywhere on the chart room: the same door.
  async function dropFile(e: DragEvent) {
    const f = e.dataTransfer?.files?.[0];
    if (!f) return;
    e.preventDefault();
    await offerFile(f.name, new Uint8Array(await f.arrayBuffer()));
    openCampaign();
  }
  // ?explorers=<an Explorers /api/v4/<slug> URL>: the item page's Open in SSE4 button. The bytes are checked against
  // the listing before anything is offered, and the link leaves the address bar once read.
  let explorersNote = $state<string | null>(null);
  async function openFromExplorers(raw: string) {
    const r = await fetchExplorers(raw);
    if (!r.ok) return void (explorersNote = r.why);
    const name = r.listing.pack ? `${r.listing.pack.id}-${r.listing.pack.version}.json` : `${r.listing.source.title || "campaign"}.json`;
    await offerFile(name, r.bytes, r.listing);
    openCampaign();
  }
  function reopenOffered() {
    if (!offered?.save) return;
    const bad = writeStored(offered.save);
    if (bad) return void (offered = { ...offered, problems: [`it could not be kept to reopen: ${bad}`], save: null });
    location.reload();
  }
  function startFresh() {
    clearStored();
    clearTimeout(saveTimer);
    // No autosave between here and the reload.
    session = null;
    location.reload();
  }
  const savedWords = $derived(savedAt ? `Kept ${new Date(savedAt).toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" })}` : "Not kept yet");

  // Lenses (U9, H8.6): what the GM sees for the job in hand. Switching never changes data, time or selection.
  function loadMine(): Lens[] {
    try {
      const raw = JSON.parse(loadPref("lenses") ?? "[]") as unknown[];
      return raw.filter((l) => checkLens(l).length === 0).map((l) => ({ ...(l as Lens), origin: "mine" as const }));
    } catch {
      return [];
    }
  }
  let mine = $state<Lens[]>(loadMine());
  // Desks (top-level design 2.3): what the GM is doing. The desk arms the verbs (Build edits, Explore and Run do not) and
  // picks the lens it opens with; "See as" changes the lens for this desk only, remembered on this device.
  type Desk = "explore" | "build" | "run";
  const DESKS: { value: Desk; label: string; key: string; hint: string }[] = [
    { value: "explore", label: "Explore", key: "V", hint: "Look around: nothing changes by accident (V)" },
    { value: "build", label: "Build", key: "E", hint: "Change things: values edit, bodies drag, undo and redo (E)" },
    { value: "run", label: "Run", key: "R", hint: "At the table: what players know, their screens and requests (R)" },
  ];
  const DESK_MODE: Record<Desk, "edit" | "view"> = { explore: "view", build: "edit", run: "view" };
  const deskLensDefault = (d: Desk) => (d === "build" ? "building" : d === "run" ? "running" : defaultLens(session?.device()?.tier));
  const storedDesk = loadPref("desk");
  let desk = $state<Desk>(storedDesk === "build" || storedDesk === "run" ? storedDesk : "explore");
  let deskLens = $state<Partial<Record<Desk, string>>>(((): Partial<Record<Desk, string>> => {
    try {
      return JSON.parse(loadPref("desk-lens") ?? "{}") as Partial<Record<Desk, string>>;
    } catch {
      return {};
    }
  })());
  const lensId = $derived(deskLens[desk] ?? deskLensDefault(desk));
  function chooseDesk(d: Desk) {
    desk = d;
    savePref("desk", d);
  }
  // Depth (2.4): how much every card, number and why? says. A new visitor starts at Curious; remembered on this device.
  const storedDepth = loadPref("depth");
  let depth = $state<Depth>(isDepth(storedDepth) ? storedDepth : "curious");
  function chooseDepth(d: Depth) {
    depth = d;
    savePref("depth", d);
  }
  $effect(() => {
    ws.depth = depth;
  });
  // The desk decides: a system read back from the universe (grown, adopted, the Solar System) is edited in place too,
  // each typed value going to the fact the universe holds (SceneFact.kernel), so Build means build everywhere.
  $effect(() => {
    // The Workshop changes the tool, not the universe, so its screens are always armed.
    ws.mode = railScale === "workshop" ? "edit" : DESK_MODE[desk];
  });
  // Secrets (SV2): the desk decides whether the "Known to" eyes show; a share's connected players hold reveals.
  $effect(() => {
    ws.desk = desk;
  });
  $effect(() => {
    ws.players_online = share?.state.players ?? 0;
    ws.sendLanding = (show) => {
      lastLanding = show;
      const sh = share?.state.shown;
      if (landingAwaited && share && ws.screen === "surface" && sh && "scope" in sh && sh.scope === scene.system && !sh.landing) return void share.show(here());
      landingAwaited = false;
      if (share && sh && "landing" in sh && sh.landing) share.show({ ...(JSON.parse(JSON.stringify(sh)) as typeof sh), landing: show });
    };
    ws.shared = !!share?.state.link;
  });
  const lenses = $derived.by(() => {
    void ws.revision;
    const s = session;
    const st = s?.stack();
    return allLenses(mine, st ? [...st.base, ...st.added] : [], st ? observeOf(st.base)?.setup : undefined, share?.state.plan);
  });
  const lens = $derived(lenses.find((l) => l.id === lensId) ?? lenses.find((l) => l.id === deskLensDefault(desk)) ?? lenses.find((l) => l.id === defaultLens(session?.device()?.tier))!);
  $effect(() => {
    ws.lens = lens;
  });
  function chooseLens(id: string) {
    if (id === "__make") return void ((lensOpen = true), lensDialog?.showModal());
    // Choosing the desk's own lens clears the override, so the desk follows its default again.
    const next = { ...deskLens };
    if (id === deskLensDefault(desk)) delete next[desk];
    else next[desk] = id;
    deskLens = next;
    savePref("desk-lens", JSON.stringify(next));
  }
  function saveLens(l: Lens) {
    mine = [...mine.filter((x) => x.id !== l.id), l];
    savePref("lenses", JSON.stringify($state.snapshot(mine)));
    chooseLens(l.id);
  }
  function deleteLens(id: string) {
    mine = mine.filter((x) => x.id !== id);
    savePref("lenses", JSON.stringify($state.snapshot(mine)));
    chooseLens(defaultLens(session?.device()?.tier));
    lensDialog?.close();
  }
  let lensDialog: HTMLDialogElement | undefined = $state();
  let lensOpen = $state(false);
  // The scene a lens looks through: what one audience knows, or every body through one data view.
  let lensGen = 0;
  $effect(() => {
    void ws.revision;
    const l = lens;
    const s = session;
    // Pencil and ink (track SV): on the Build and Run desks the GM's chart always knows what the table knows, so an
    // unknown body is drawn in pencil whatever the lens; a player lens looks through its own audience.
    const aud = lensAudience(l) ?? (desk === "build" || desk === "run" ? EVERYONE : null);
    const g = ++lensGen;
    // The system in view, not the shipped sample's: a lens on the Solar System looks at the Solar System.
    const sys = scene.system || (factsDoc as unknown as AuthoredFacts).scope;
    if (!s || !aud) return void (ws.lensScene = null);
    s.scene(sys, { audience: aud, ...(l.data ? { data_view: l.data } : {}) }).then((r) => {
      if (g === lensGen) ws.lensScene = r.ok ? r.scene : null;
    });
  });
  // The fields a card rule can list: per kind of body, and all of them for "any body".
  const lensFields = $derived.by(() => {
    const list = (subs: typeof scene.subjects) => {
      const seen = new Map<string, string>();
      for (const sub of subs) for (const f of sub.facts) if (!seen.has(f.field)) seen.set(f.field, f.label);
      // Two fields may share a label (a star's mass and a planet's): the second says which it is.
      const used = new Set<string>();
      return [...seen].map(([id, label]) => {
        const l = used.has(label) ? `${label} (${id})` : label;
        used.add(label);
        return { id, label: l };
      });
    };
    const out: Record<string, { id: string; label: string }[]> = { "*": list(scene.subjects) };
    for (const k of new Set(scene.subjects.map((x) => x.kind))) out[k] = list(scene.subjects.filter((x) => x.kind === k));
    return out;
  });
  // N1: the sound bed for the frame in view (off until the GM turns it on; never touches the session).
  // Light (O1): which band recipe paints the marks. Presentation only, on this device.
  const LIGHT_RECIPES = (lightDoc.recipes as unknown as BandRecipe[]).filter((r) => !checkLightRecipe(r).length);
  let lightId = $state(readLight(loadPref("light.gm"), LIGHT_RECIPES.map((r) => r.id)));
  // Band (one set of controls, Alex 23 Nov): chosen in the view controls; the Light panel paints within it. A choice
  // kept before the Band switch takes its band from the light it kept.
  if (ws.view && !ws.view.band) ws.view = { ...ws.view, band: recipeBand(LIGHT_RECIPES.find((r) => r.id === lightId)) };
  const band = $derived<ViewBand>(ws.view?.band ?? "visible");
  const lastLight: Partial<Record<ViewBand, string>> = {};
  $effect(() => {
    const b = band;
    const now = untrack(() => lightId);
    const from = recipeBand(LIGHT_RECIPES.find((r) => r.id === now));
    if (from === b) return;
    lastLight[from] = now;
    lightId = lightForBand(LIGHT_RECIPES, b, now, lastLight);
  });
  // U4 part 2: with a look-ahead open, the views show the snapshot the scrubber is on.
  let ahead = $state<LookedAhead | null>(null);
  let aheadAt = $state(0);
  // The device's tier on the page, so basic devices drop motion (U12).
  $effect(() => {
    if (status.tier) document.documentElement.dataset.tier = status.tier;
  });
  // U11 part 2: every body where it is on the day being viewed, from the kernel's orbits_at (two-body Kepler, local
  // to each host). Views draw from the scene, so the phases go onto it and the chart, 3D view and sound move together.
  // A fixture scene sends its own elements, with its drawn phases as their places at the campaign's start.
  let phases = $state<Map<string, number> | null>(null);
  // Play asks ten times a second: one call at a time, and only the latest time waits.
  let orbitsFor = "";
  let orbitsBusy = false;
  let orbitsWant: { s: ViewSession; sc: SystemScene; at: bigint; epoch: bigint; key: string } | null = null;
  function pumpOrbits() {
    if (orbitsBusy || !orbitsWant) return;
    const w = orbitsWant;
    orbitsWant = null;
    orbitsBusy = true;
    w.s.orbitsAt(w.sc.system, [w.at], orbitBodies(w.sc, w.epoch)).then((r) => {
      orbitsBusy = false;
      if (r.ok && orbitsFor === w.key) phases = phasesFrom(r.bodies);
      pumpOrbits();
    });
  }
  $effect(() => {
    const s = session;
    const sc = scene;
    const at = yearsToSeconds(ws.now_yr);
    const epoch = yearsToSeconds(ws.campaign_epoch_yr ?? base.now_yr);
    if (!s) return;
    const key = `${sc.system}|${sc.fixture ? "f" : "k"}`;
    if (key !== orbitsFor) ((orbitsFor = key), (phases = null));
    orbitsWant = { s, sc, at, epoch, key };
    pumpOrbits();
  });
  // A pack's calendar (kernel calendar), read again whenever the pack stack changes.
  $effect(() => {
    const s = session;
    void status.stack;
    if (!s) return;
    s.calendar().then((r) => (ws.pack_calendar = r.ok ? (r.calendar ?? null) : null));
  });
  const timed = $derived(withPhases(scene, phases));
  // Names on the chart (NM6): a lens may label the chart by another name layer (designation, local); names only.
  let layerLabels = $state.raw<Map<string, string>>(new Map());
  $effect(() => {
    void ws.revision;
    const layer = lens.names ?? "";
    const s = session;
    const addrs = scene.subjects.map((x) => x.address);
    if (!s || !layer || layer === "common") return void (layerLabels = new Map());
    void namesInLayer(s, addrs, layer).then((m) => {
      if ((lens.names ?? "") === layer) layerLabels = m;
    });
  });
  const shown = $derived(relabel(ahead && ahead.snapshots[aheadAt] ? sceneAt(timed, ahead, ahead.snapshots[aheadAt]!) : timed, layerLabels));
  const lit = $derived(describeLight(litFromSystem(shown)));
  const lightNames = $derived(new Map(shown.subjects.map((x) => [x.address, x.label])));
  const lightColours = $derived.by(() => {
    const r = LIGHT_RECIPES.find((x) => x.id === lightId);
    return r ? paint(r, lit.lights) : null;
  });
  // Stars that show what they do (V3 gap 19): each star's power and activity, read once per system, for its mark.
  let starFacts = $state<Map<string, StarFacts>>(new Map());
  const starKey = $derived(`${scene.system}|${scene.subjects.filter((x) => x.kind === "star").map((x) => x.address).join(",")}`);
  $effect(() => {
    const key = starKey;
    const sys = scene.system;
    const stars = untrack(() => scene.subjects.filter((x) => x.kind === "star").map((x) => x.address));
    if (!session || !sys) return;
    let live = true;
    void readStarFacts(session, sys, stars).then((f) => {
      if (live && key === starKey) starFacts = f;
    });
    return () => {
      live = false;
    };
  });
  const lightMark = (m: Mark, address: string): Mark => (lightColours ? { ...m, colour: lightColours.get(address) ?? null } : m);
  const soundScene = $derived(orbitScene(shown.system, voicesFromSystem(shown)));
  const selName = $derived(scene.subjects.find((x) => x.address === ws.selection)?.label ?? null);
  // The place line's list: every body in tree order, moons set in under their planets.
  const placeBodies = $derived.by(() => {
    const order = systemOrder(scene);
    const depth = (a: string | undefined): number => {
      let d = 0;
      for (let p = scene.subjects.find((x) => x.address === a)?.parent; p; p = scene.subjects.find((x) => x.address === p)?.parent) d++;
      return d;
    };
    return order.map((b) => ({ address: b.address, label: b.label, depth: Math.max(0, depth(b.address) - 1) }));
  });
  const lensKinds = $derived([...new Set(scene.subjects.filter((x) => x.facts.length).map((x) => x.kind))]);

  // The campaign's clock travels to players (U11): its now, not whatever time the GM is viewing.
  $effect(() => {
    const yr = ws.campaign_now_yr ?? ws.now_yr;
    if (share?.state.link) share.setClock(yr);
  });
  // Physics changes over thousands of years, not hours: scenes re-derive when the viewed time moves by a kyr or more,
  // so stepping and playing through days does not call the kernel's derivations at every frame.
  const deriveNow = $derived(Math.abs(ws.now_yr - base.now_yr) < 1 ? base.now_yr : Math.round(ws.now_yr / 1000) * 1000);
  /** Move the campaign's clock to another system's present, keeping the time elapsed in the campaign. */
  function shiftCampaign(present: number) {
    const elapsed = (ws.campaign_now_yr ?? ws.now_yr) - (ws.campaign_epoch_yr ?? ws.now_yr);
    ws.campaign_epoch_yr = present;
    ws.campaign_now_yr = present + elapsed;
    ws.now_yr = ws.campaign_now_yr;
  }

  // Scenes come from the kernel's derivations over the authored values; any
  // edit or a move of the ruler re-derives (K9 moves this into the kernel).
  let generation = 0;
  $effect(() => {
    void ws.revision;
    const now = deriveNow;
    const applied = { ...ws.applied };
    const s = session;
    if (!s) return;
    const b = ws.branch ? { id: ws.branch.id, applied: { ...ws.branch.applied } } : null;
    const g = ++generation;
    s.stateHash().then((h) => (status = { ...status, hash: h, stack: s.stack().hash, campaign: s.stack().campaign !== null }));
    // U4: with a what-if open, the chart shows the branch; the main line is kept for comparing.
    if (b) s.branchStateHash(b.id).then((h) => g === generation && (branchHash = h));
    else branchHash = "";
    // V13 part 2: a system adopted into the campaign is read back from the universe, not derived from the fixture.
    const v = viewing;
    if (v) {
      s.resolvedSystem(v.system).then((r) => {
        if (g !== generation) return;
        if (!r.ok) return void (whatNote = `Could not read ${v.title} back: ${r.message}`);
        const read0 = sceneFromResolved(r.result, v.title);
        // The Solar System's sheet marks a few values unverified: those bodies carry the mark on the GM's view.
        const marks = v.system === SOL ? solUnverified : null;
        const ids = v.system === SOL ? solIds : null;
        const read = marks?.size || ids?.size ? { ...read0, subjects: read0.subjects.map((x) => ({ ...x, ...(marks?.has(x.address) ? { stamps: [...x.stamps, "unverified"] } : {}), ...(ids?.has(x.address) ? { world_fixture: ids.get(x.address) } : {}) })) } : read0;
        const now_yr = v.fresh ? read.now_yr : now;
        if (v.fresh) ((v.fresh = false), shiftCampaign(read.now_yr));
        viewingSpec = r.result.spec ?? null;
        mainScene = scene = { ...read, now_yr, span_yr: [read.span_yr[0], Math.max(read.span_yr[1], now_yr * 1.25)] };
      });
      return;
    }
    // A now moved by a kept look-ahead can pass the fixture's ruler: the ruler grows to keep it in view.
    const at = { ...base, now_yr: now, span_yr: [base.span_yr[0], Math.max(base.span_yr[1], now * 1.25)] as [number, number] };
    // Bodies pasted or added by hand into the campaign's own system live in the universe, not the fixture: they are read back from it
    // and set in beside the fixture's (a withdrawn one, by Undo or Cut, has left the read-back).
    const added = pastedSubjects(s, at);
    const main = added.then((x) => deriveScene(s, applyEdits(x, applied), derivations));
    const what = b ? added.then((x) => deriveScene(s, applyEdits(x, { ...applied, ...b.applied }), derivations)) : main;
    Promise.all([main, what]).then(([m, w]) => {
      if (g !== generation) return;
      mainScene = m;
      scene = w;
    });
  });
  async function pastedSubjects(s: ViewSession, at: SystemScene): Promise<SystemScene> {
    const r = await s.resolvedSystem(at.system);
    if (!r.ok) return at;
    const have = new Set(at.subjects.map((x) => x.address));
    const read = sceneFromResolved(r.result).subjects.filter((x) => !have.has(x.address));
    if (!read.length) return at;
    // Read back alone, a pasted star would be taken for the system's primary: its host and orbit come from its facts.
    const placed = read.map((x) => {
      const f = r.result.facts[x.address] ?? {};
      const host = typeof f["orbit.host"] === "string" ? (f["orbit.host"] as string) : x.parent;
      const sma = f["orbit.semi_major_axis"];
      const orbit = x.orbit ?? (typeof sma === "number" && !x.band_au ? { a_au: sma / AU_M, phase_deg: 0 } : undefined);
      return { ...x, subtitle: `Added by hand · ${x.kind}`, stamps: x.stamps.filter((t) => t !== "imported"), ...(host ? { parent: host } : {}), ...(orbit ? { orbit } : {}) };
    });
    return { ...at, subjects: [...at.subjects, ...placed] };
  }
  let mainScene = $state<SystemScene>(base);
  // V13 part 2: systems adopted into the campaign this session, and the one on the chart (null: the campaign's own).
  let adoptedSystems = $state<{ system: string; title: string }[]>([]);
  let viewing = $state<{ system: string; title: string; fresh: boolean } | null>(null);
  let viewingSpec = $state<SystemSpec | null>(null);
  function showSystem(system: string) {
    const a = adoptedSystems.find((x) => x.system === system);
    ws.select(null);
    if (a) viewing = { ...a, fresh: true };
    else ((viewing = null), (viewingSpec = null), shiftCampaign(base.now_yr));
    ws.revision++;
  }
  // The place line at every scale (U24, top-level design 2.1): a breadcrumb up, the list of what is nested below for
  // going down, the trail back to where the GM was, recent places, and the place as a deep link in the address bar.
  // Above a system, the scale's own view says which place is in view and what is nested below it (an "sse:place" event,
  // ui-contract place-event.ts): the latest word from each scale names its step and fills the Below list.
  let notes = $state<Partial<Record<PlaceScale, PlaceNote>>>({});
  const UPPER: readonly PlaceScale[] = ["web", "cluster", "galaxy"];
  const noteId = (scale: RailScale) => (UPPER.includes(scale as PlaceScale) ? notes[scale as PlaceScale]?.id : undefined);
  const place = $derived<Place | null>(
    railScale === "workshop"
      ? null
      : { scale: railScale, system: viewing?.system ?? "", ...(railScale === "world" && ws.selection ? { world: ws.selection } : {}), ...(noteId(railScale) ? { id: noteId(railScale) } : {}), ...(ws.screen !== firstScreen(railScale) ? { screen: ws.screen } : {}) },
  );
  // A place to open a scale's view on (a link, a step, a child picked in Below), passed as its `place` param; a new one
  // mounts the view afresh. Without one, a scale's view opens on the place it last named.
  let opening = $state<Partial<Record<PlaceScale, string>>>({});
  let openings = $state(0);
  const chartMount = $derived.by(() => {
    const m = mountFor("chart");
    // Back at a scale, its view opens where the GM last was there.
    const id = opening[railScale as PlaceScale] ?? noteId(railScale);
    return m && id && m.module.params.some((x) => x.name === "place") ? { ...m, params: { ...m.params, place: id } } : m;
  });
  $effect(() => {
    const hear = (e: Event) => {
      const d = (e as CustomEvent<PlaceNote>).detail;
      if (!d || !UPPER.includes(d.scale) || typeof d.id !== "string") return;
      // Only news changes anything: a view saying the same again must not stir the line (or the view) again.
      if (JSON.stringify(untrack(() => notes[d.scale])) !== JSON.stringify(d)) notes = { ...untrack(() => notes), [d.scale]: d };
      // Once the view is there, it is no longer being opened on it: a later visit opens wherever the GM goes next.
      if (untrack(() => opening[d.scale]) === d.id) opening = Object.fromEntries(Object.entries(untrack(() => opening)).filter(([k]) => k !== d.scale));
    };
    window.addEventListener(PLACE_EVENT, hear);
    return () => window.removeEventListener(PLACE_EVENT, hear);
  });
  /** The instrument a scale opens on: the first on its rail. */
  function firstScreen(scale: RailScale): string | undefined {
    return scale === "world" ? "body" : scale === "system" ? "system" : railOf(SCREENS, scale)[0]?.lines[0]?.screen;
  }
  // The place a link names, read before anything writes the address bar.
  const linkedPlace = typeof location === "undefined" ? null : parsePlaceHash(location.hash);
  let trail = $state<Place | null>(null);
  let linked = false;
  let recent = $state<Place[]>(readRecent(loadPref("places")));
  const placeName = (a: string | undefined | null) => (a ? (scene.subjects.find((x) => x.address === a)?.label ?? null) : null);
  const steps = $derived(
    place
      ? placeSteps(place, trail, { system: scene.title, world: placeName(railScale === "world" ? ws.selection : trail?.world), above: Object.fromEntries(UPPER.flatMap((k) => (notes[k] ? [[k, { id: notes[k]!.id, name: notes[k]!.name }]] : []))) })
      : [],
  );
  $effect(() => {
    const p = place;
    if (!p) return;
    untrack(() => {
      trail = nextTrail(trail, p);
      const label = p.scale === "world" ? placeName(p.world) : p.id ? (notes[p.scale]?.name ?? null) : null;
      // Above a system, a place is remembered once its view has said which one it is.
      if (p.scale === "system" || label) {
        recent = rememberPlace(recent, label ? { ...p, label } : p);
        savePref("places", JSON.stringify(recent));
      }
      // The address bar follows: a new place is a step in the browser's history; another instrument at the same place is not.
      if ((p.scale === "world" && !p.world) || !linked) return;
      const h = placeHash(p);
      if (typeof location === "undefined" || location.hash === h || startOpen) return;
      const was = parsePlaceHash(location.hash);
      // The view naming its place (an id arriving for the same scale) refines this step rather than making a new one.
      const refined = was && was.scale === p.scale && was.system === p.system && !was.id && !!p.id;
      if (was && (samePlace(was, p) || refined)) history.replaceState(history.state, "", h);
      else history.pushState(history.state, "", h);
    });
  });
  /** Go to a place: its system on the chart, its scale, its world and its instrument. */
  function goPlace(p: Place) {
    if (p.system !== (viewing?.system ?? "")) {
      if (p.system && !adoptedSystems.some((x) => x.system === p.system)) return;
      showSystem(p.system);
    }
    if (p.scale === "world") toWorld(p.world);
    else {
      // A place above a system that its view is not on yet: open the view on it.
      if (p.id && p.id !== notes[p.scale]?.id) {
        opening = { ...opening, [p.scale]: p.id };
        if (railScale === p.scale) openings++;
      }
      toScale(p.scale);
    }
    if (p.screen && SCREENS[p.screen] && scalesOf(SCREENS[p.screen]!).includes(p.scale)) ws.screen = p.screen;
    // A landing site in the link (Voyages' "Land here"): the Land screen picks it up once it has read the world.
    if (p.site && p.world) ws.landing_site = { body: p.world, lat_deg: p.site.lat_deg, lon_deg: p.site.lon_deg };
  }
  /** Up one scale, from above the system (Esc); at a world, up to its system. */
  function upAStep() {
    const i = placeRank(railScale as PlaceScale);
    if (i > 0 && railScale !== "system") toScale(PLACE_SCALES[i - 1]!);
  }
  const recentElsewhere = $derived(place ? recent.filter((r) => !samePlace(r, place)) : recent);
  // The address bar's Back and Forward, and a pasted link, move through places.
  function fromHash() {
    const p = parsePlaceHash(location.hash);
    if (p && session && !(place && samePlace(place, p) && (!p.screen || p.screen === ws.screen))) goPlace(p);
  }
  // A chart can send the GM to another scale (the galaxies nearby's "Chart this galaxy"): a bubbling "sse:explore" event
  // naming the section. Inside the Explore screen its own tabs answer it; on the place line it is a scale.
  const SECTION_SCALE: Record<string, PlaceScale> = { foam: "web", cluster: "cluster", galaxy: "galaxy" };
  $effect(() => {
    const go = (e: Event) => {
      const to = SECTION_SCALE[(e as CustomEvent<{ section?: string }>).detail?.section ?? ""];
      if (to && ws.screen !== "explore") toScale(to);
    };
    window.addEventListener("sse:explore", go);
    return () => window.removeEventListener("sse:explore", go);
  });
  $effect(() => {
    if (!session || linked) return;
    linked = true;
    untrack(() => {
      const p = linkedPlace;
      if (p && (p.scale !== "system" || p.system || p.screen)) goPlace(p);
    });
  });
  function adopted(systems: { system: string; title: string }[], note: string, hash: string) {
    adoptedSystems = [...adoptedSystems.filter((x) => !systems.some((y) => y.system === x.system)), ...systems];
    whatNote = note;
    status = { ...status, hash };
    ws.revision++;
  }
  let branchHash = $state("");
  let whatNote = $state<string | null>(null);
  let compareDialog: HTMLDialogElement | undefined = $state();
  // A1: agents, on the GM's own key. The panel reads only what is on the chart.
  let agentsDialog: HTMLDialogElement | undefined = $state();
  let agentsOpen = $state(false);
  // Describe on a card (V3 gap 23): the subject handed to Minerva, until the Agents panel closes.
  let describing = $state<string | null>(null);
  function describe(address: string) {
    describing = address;
    agentsOpen = true;
    agentsDialog?.showModal();
  }
  // "Ask Orac to create it" (editing screens, Alex 24 Nov): opens Orac with the request written in; nothing is drafted until the GM presses Draft.
  let asking = $state<string | null>(null);
  function askOrac(request: string) {
    asking = request;
    agentsOpen = true;
    agentsDialog?.showModal();
  }
  let makeDialog: HTMLDialogElement | undefined = $state();
  let makeOpen = $state(false);
  let makeStart = $state<"import" | "generate">("import");
  function openMake(tab: "import" | "generate") {
    if (!makeDialog?.open && tab === "generate") makeFile = null;
    makeStart = tab;
    makeOpen = true;
    makeDialog?.showModal();
  }

  // The start sheet (top-level design 3.1, step 3): what the app opens on, unless this device asked to skip it or the
  // address ends in #chart. Features the sheet may name once they are built ("sol", "tours", "find", ...): none yet.
  // Features the start sheet's lines and verbs may name: the Solar System once its sheet is in the build.
  const STARTED: ReadonlySet<string> = new Set(["sol"]);
  // The Solar System door (U21 part 2): SSE4's own Sol from packs/base-sol, entered once as catalogued facts, one batch
  // per source, then opened like any system in the campaign. Later opens go straight to it.
  const SOL = `u:${base.universe.id}/sys:sol`;
  let solUnverified = $state<ReadonlySet<string> | null>(null);
  // Each Sol body's sheet id ("earth", "europa"), by address: the name the kernel's world stacks go by, so a catalogued
  // world opens in the cutaway when the kernel has its stack (and says it has none otherwise).
  let solIds = $state<ReadonlyMap<string, string>>(new Map());
  // The same sheet for views (reference worlds on the editing strips), read once and kept.
  let solForViews: Promise<SolSheet | null> | null = null;
  const solSheetForViews = () =>
    (solForViews ??= fetch(new URL(stackDoc.catalogues.sol, document.baseURI))
      .then((r) => (r.ok ? r.json() : null))
      .then((p) => (p ? readSol(p) : null))
      .catch(() => null));
  async function solSheet() {
    const sheet = readSol(await (await fetch(new URL(stackDoc.catalogues.sol, document.baseURI))).json());
    if (!sheet) throw new Error("the Solar System sheet could not be read");
    const laid = solBatches(sheet, SOL);
    solUnverified = new Set(laid.unverified.map((k) => k.split("#")[0]!));
    solIds = new Map(Object.entries(laid.addresses).map(([id, a]) => [a, id]));
    return sheet;
  }
  // Opened from the system list after a reload, the sheet is read for its marks.
  $effect(() => {
    if (viewing?.system === SOL && !solUnverified) solSheet().then(() => ws.revision++, () => {});
  });
  async function openSol() {
    const s = session;
    if (!s) return;
    if (!adoptedSystems.some((x) => x.system === SOL)) {
      whatNote = "Entering the Solar System from its catalogue…";
      try {
        const sheet = await solSheet();
        for (const b of solBatches(sheet, SOL).batches) {
          const r = await s.send({ kind: "set_values", edits: b.edits, at_yr: 0, evidence: "catalogued", source: solSourceText(sheet, b.source) });
          if (r.status !== "applied") throw new Error(r.status === "refused" ? r.reason : r.status === "error" ? r.message : "sent to the game master");
        }
      } catch (e) {
        return void (whatNote = `Could not open the Solar System: ${e instanceof Error ? e.message : String(e)}`);
      }
      adoptedSystems = [...adoptedSystems, { system: SOL, title: SOL_TITLE }];
      whatNote = "The Solar System, as measured.";
    }
    // The door opens on the chart, whatever screen was in front.
    railScale = "system";
    ws.screen = "system";
    showSystem(SOL);
  }
  let startOpen = $state(typeof location !== "undefined" && !parsePlaceHash(location.hash) && !/[#&]postcard=/.test(location.hash) && loadPref("start") !== "skip");
  const keptSave = readStored(`u:${base.universe.id}`).save;
  const carryOn = keptSave ? { title: keptSave.title, when: new Date(keptSave.saved_at).toLocaleString("en-GB", { dateStyle: "medium", timeStyle: "short" }) } : null;
  /** A world for the world instruments when none is selected: the selected body's world, else the first planet. */
  function someWorld(): string | undefined {
    const sel = scene.subjects.find((x) => x.address === ws.selection);
    if (sel && sel.kind !== "star") return sel.address;
    return systemOrder(scene).find((x) => x.kind === "planet")?.address;
  }
  function fromStart(a: StartAction) {
    startOpen = false;
    queueMicrotask(() => {
      if ("door" in a) {
        if (a.door === "sol") void openSol();
        else if (a.door === "grow") openMake("generate");
        else if (a.door === "bring") openCampaign();
      } else if ("desk" in a) chooseDesk(a.desk);
      else if ("screen" in a) (toSystem(), go(a.screen));
      else if ("world" in a) (toWorld(someWorld()), (ws.screen = a.world));
      else if ("dialog" in a) {
        if (a.dialog === "ask" && a.orac) askOrac(a.orac);
        else if (a.dialog === "ask") ((agentsOpen = true), agentsDialog?.showModal());
        else if (a.dialog === "grow") openMake("generate");
        else if (a.dialog === "bring") openCampaign();
        else if (a.dialog === "share" && share) window.open(share.start(here()), "_blank", "noopener");
      } else if ("control" in a) document.querySelector<HTMLButtonElement>(a.control === "light" ? ".light .toggle" : ".sound .toggle")?.click();
      else if ("link" in a) window.open(a.link, "_blank", "noopener");
    });
  }
  const agentContext = $derived.by(() => {
    const sub = scene.subjects.find((x) => x.address === ws.selection);
    const head = `${scene.title}: ${scene.subtitle}.`;
    return sub ? `${head}\nSelected: ${sub.label}${sub.subtitle ? ` (${sub.subtitle})` : ""}.\n${sub.facts.map((f) => `${f.label}: ${f.display} (${f.source})`).join("\n")}` : head;
  });
  // The kernel's atoms (parameter ranges and units) for Orac's catalogue, read once a session is open.
  let atoms = $state.raw<readonly AtomInfo[]>([]);
  $effect(() => {
    const s = session;
    if (s && oracMay(s.seat)) loadAtoms(s).then((a) => (atoms = a));
  });
  /** An accepted Orac screen goes to the players page on this device, as the composer's Show to players does. */
  function openOracScreen(d: ScreenDoc) {
    try {
      localStorage.setItem("sse4.composed-screen", JSON.stringify(d));
      const page = "players.html?screen=composed";
      window.open(new URL(page, document.baseURI).href, "_blank", "noopener");
    } catch {
      /* A browser that keeps nothing: the composer's download is the way. */
    }
  }
  // A2: Orac's targets go through the GM's own paths (the kernel's checks, the Physics screen's queue, the lens list).
  const oracHost = $derived.by((): OracHost | undefined => {
    const s = session;
    if (!s || !oracMay(s.seat)) return undefined;
    const places = () => scene.subjects.map((x) => ({ address: x.address, name: x.label }));
    const fields = () => Object.fromEntries(Object.entries(lensFields).filter(([k]) => k !== "*").map(([k, v]) => [k, v.map((f) => f.id)]));
    return {
      targets: () => [
        physicsTarget(s, ws),
        lensTarget({ taken: () => lenses.map((l) => l.id), fields, save: saveLens }),
        styleTarget({ registry: styleHost.registry, views: styleHost.views, tier: styleHost.tier, keep: styleHost.keep }),
        screenTarget({ views: () => PLAYER_STACK, taken: () => [], dataViews: () => observeOf(s.stack().base)?.setup.views.map((v) => v.id), styles: () => ({ registry: styleHost.registry(), tier: styleHost.tier() }), themes: () => ALL_THEMES.map((t) => t.id), symbologies: () => symbols.registry().map((d) => d.id), open: openOracScreen }),
        symbologyTarget({ registry: symbols.registry, keep: symbols.keep }),
        dataViewTarget({ setup: () => observeOf(s.stack().base)?.setup ?? null, apply: async (x) => {
          const r = await applySetup(s, ws, x);
          return r.status === "applied" ? null : r.status === "refused" ? r.reason : `not applied (${r.status})`;
        } }),
        programTarget(s, ws),
        sifterTarget(s, ws, (factsDoc as unknown as AuthoredFacts).scope),
        boxTarget(s, ws),
        tableTarget(s, ws),
        substanceTarget(s, ws),
        triggerTarget(s, ws),
        templateTarget(s, ws),
        revealTarget(s, ws, places),
        atomIssueTarget((u) => window.open(u, "_blank", "noopener")),
      ],
      catalogue: () => [...flowEntries(s, atoms), ...lensEntries(lenses, fields()), ...lookEntries({ views: [...REGISTRY, ...PLAYER_STACK] as never, styles: styleHost.registry(), symbologies: symbols.registry() }), ...dataViewEntries(observeOf(s.stack().base)?.setup ?? null), ...ruleEntries(s), ...sifterEntries(s, STORY_STARTERS), ...blockEntries(s), ...sheetEntries(s), ...triggerEntries(s), ...placeEntries(places()), ...handbookEntries()],
      here: () => [agentContext, ws.bench ? `Flow on the bench: ${ws.bench.flow}${ws.bench.node ? `, node ${ws.bench.node}` : ""}` : "", `Lens in use: ${lens.label}`].filter(Boolean).join("\n"),
    };
  });
  // A4: Tars reads SSE's help passages and the kernel's "why?" for the selected body's derived values.
  const tarsHost = $derived.by((): TarsHost => {
    const s = session;
    const sub = scene.subjects.find((x) => x.address === ws.selection);
    return {
      here: () => agentContext,
      why: async () => {
        if (!s || !sub) return [];
        const derived = sub.facts.filter((f) => f.why).slice(0, 8);
        const out = await Promise.all(
          derived.map(async (f) => {
            const r = await s.explain(f.why!, f.why!.kind);
            if (!r.ok) return null;
            const x = r.explanation;
            const refs = x.references.map(showReference);
            return whyPassage({ subject: sub.label, field: f.field, label: f.label, display: f.display, curious: x.lenses.curious, science: x.lenses.science, validity: x.validity, references: refs });
          }),
        );
        return out.filter((p) => p !== null);
      },
    };
  });
  const STORY_STARTERS = storyStarters.sifters as unknown as SifterDoc[];
  // A3: Minerva reads one audience's scene (K9) and nothing else; the GM's scene is used only to check her text.
  const minervaHost = $derived.by((): MinervaHost | undefined => {
    const s = session;
    if (!s || !oracMay(s.seat)) return undefined;
    const scope = (factsDoc as unknown as AuthoredFacts).scope;
    return {
      audiences: () => (share?.state.plan.audiences ?? [{ id: "everyone", label: "Everyone" }]).map((a) => ({ id: a.id, label: a.label })),
      scene: async (audience) => {
        const r = await s.scene(scope, audience ? { audience } : {});
        return r.ok ? r.scene : null;
      },
      vocab: plainVocab as unknown as Vocab,
      title: `${scene.title}: ${scene.subtitle}`,
      // The stack's sifters and the story finder's starters, over what that audience is shown.
      sift: async (audience) => {
        const r = await s.sift({ scope, audience, sifters: STORY_STARTERS });
        return r.ok ? r.result : null;
      },
      // A kept description is a narrative note in the record, for the audience it was written for (the GM sees every note).
      keep: async (address, text, audience) => {
        if (!s.addNote) return "This session cannot write the record.";
        const r = await s.addNote({ address, text, audience: audience === "everyone" ? "everyone" : [audience], tags: ["description"], narrative: true, at_s: yearsToSeconds(ws.now_yr) });
        return r.ok ? null : r.message;
      },
    };
  });
  const differences = $derived(ws.branch ? compareScenes(mainScene, scene, ws.branch.applied) : []);
  async function whatIf(action: "open" | "keep" | "drop") {
    if (!session) return;
    whatNote = null;
    if (action === "open") whatNote = await openWhatIf(session, ws, `What if, from ${formatYears(ws.now_yr)}`);
    else if (action === "drop") await dropWhatIf(session, ws);
    else {
      const n = ws.branch?.edits.length ?? 0;
      const bad = await keepWhatIf(session, ws);
      whatNote = bad ?? `Kept: ${n} ${n === 1 ? "change is" : "changes are"} now on the chart's history.`;
    }
    compareDialog?.close();
  }

  const handles: ViewHandle[] = [];
  $effect(() => {
    const s = shown;
    for (const h of handles) h.update(s);
  });

  let undoNote = $state<string | null>(null);
  async function undo(back: boolean) {
    if (!session) return;
    const r = await (back ? undoEdit(session, ws) : redoEdit(session, ws));
    undoNote = r === null ? (back ? "Nothing to undo." : "Nothing to redo.") : r.status === "applied" ? null : "Could not change it back.";
  }

  // Copy and paste (top-level design 8.2). Copy takes the selected body and everything under it as a clip on the
  // clipboard; Paste reads one on the Build desk, shows where it would land, and places it as one change (the kernel's
  // batch edit), so one Undo takes the whole paste back. Cut is Copy, then the body is withdrawn, as one change too.
  let clipNote = $state<string | null>(null);
  let clipDialog: HTMLDialogElement | undefined = $state();
  let pasted = $state<{ clip: Clip; words: string; host: string; hostKind: string; hostLabel: string; a_au?: number; missing: { id: string; version: string; have?: string }[]; placed: { address: string; label: string; kind: string }[] } | null>(null);
  const clipArmed = $derived(desk === "build" && railScale !== "workshop" && hasTool(ws.lens, "clip"));
  const stackPacks = () => (session?.stack().base ?? []).map((p) => ({ id: p.id, version: p.version }));
  function copyText(): { text: string; words: string; left: string[] } | null {
    const a = ws.selection;
    if (!a) return null;
    const c = copyClip(scene, a, { packs: stackPacks() });
    return c ? { text: c.text, words: c.words, left: c.left } : null;
  }
  function copiedNote(c: { words: string; left: string[] }) {
    clipNote = `Copied ${c.words}. Paste it on the Build desk of any campaign${c.left.length ? `; ${c.left.slice(0, 3).join(", ").toLowerCase()}${c.left.length > 3 ? " and more" : ""} will be worked out again where it lands` : ""}.`;
  }
  async function copyBody() {
    const c = copyText();
    if (!c) return void (clipNote = "Select a body to copy.");
    try {
      await navigator.clipboard.writeText(c.text);
      copiedNote(c);
    } catch {
      // No clipboard permission (an insecure page, or a refused prompt): the copy event carries it instead.
      copyFallback = c;
      if (!document.execCommand("copy")) clipNote = "The browser would not let the chart room use the clipboard. Select the body and press Ctrl+C.";
    }
  }
  let copyFallback: { text: string; words: string; left: string[] } | null = null;
  function oncopy(e: ClipboardEvent) {
    const t = e.target as HTMLElement;
    if (!copyFallback && (!clipArmed || t.closest?.("input, textarea, select, dialog") || (window.getSelection()?.toString() ?? ""))) return;
    const c = copyFallback ?? copyText();
    copyFallback = null;
    if (!c || !e.clipboardData) return;
    e.clipboardData.setData("text/plain", c.text);
    e.preventDefault();
    copiedNote(c);
  }
  function oncut(e: ClipboardEvent) {
    const t = e.target as HTMLElement;
    if (!clipArmed || !cuttable || t.closest?.("input, textarea, select, dialog") || (window.getSelection()?.toString() ?? "")) return;
    const c = copyText();
    if (!c || !e.clipboardData) return;
    e.clipboardData.setData("text/plain", c.text);
    e.preventDefault();
    const a = ws.selection!;
    void withdrawBodies(session!, ws, { roots: [a], words: c.words }).then((r) => {
      if (r.status !== "applied") return void (clipNote = `Could not cut it: ${r.status === "refused" ? r.reason : r.status === "error" ? r.message : "sent to the game master"}.`);
      ws.select(null);
      clipNote = `Cut ${c.words}. Paste it anywhere on the Build desk; Undo puts it back.`;
    });
  }
  function onpaste(e: ClipboardEvent) {
    const t = e.target as HTMLElement;
    if (t.closest?.("input, textarea, select, dialog")) return;
    const text = e.clipboardData?.getData("text/plain") ?? "";
    if (looksLikePostcard(text)) {
      e.preventDefault();
      return void openPostcard(text);
    }
    const r = readClip(text);
    if (!r.ok && !r.v3 && desk !== "build") return;
    e.preventDefault();
    pasteText(text);
  }
  async function pasteFromMenu() {
    moreOpen = false;
    try {
      pasteText(await navigator.clipboard.readText());
    } catch {
      clipNote = "The browser would not let the chart room read the clipboard. Press Ctrl+V on the chart instead.";
    }
  }
  function pasteText(text: string) {
    const r = readClip(text);
    if (desk === "explore") return void (clipNote = "Paste works on the Build desk. Press E to switch to Build.");
    if (desk === "run") return void (clipNote = "On the Run desk only ships and stations from stock can be pasted. Press E to paste on the Build desk.");
    if (!r.ok) return void (clipNote = r.why);
    const sel = scene.subjects.find((x) => x.address === ws.selection);
    const primary = primaryOf(scene);
    // What it orbits: a moon goes round the selected planet (or the planet of a selected moon); anything else goes
    // round the selected star, else the system's primary. The clip's own distance is kept.
    const rootKind = r.clip.nodes[0]!.kind;
    const planetOf = (x: typeof sel) => (x?.kind === "planet" ? x : x?.kind === "moon" ? scene.subjects.find((y) => y.address === x.parent) : undefined);
    const host = rootKind === "moon" ? (planetOf(sel) ?? primary) : sel?.kind === "star" ? sel : primary;
    if (!host) return void (clipNote = "There is no star here to paste around.");
    const a = r.clip.events.find((x) => x.node === r.clip.root && x.field === "orbit.semi_major_axis")?.value;
    const p = placeClip(r.clip, { system: scene.system, host: host.address, hostKind: host.kind, taken: scene.subjects.map((x) => x.address) });
    pasted = { clip: r.clip, words: clipWords(r.clip), host: host.address, hostKind: host.kind, hostLabel: host.label, ...(typeof a === "number" ? { a_au: a / AU_M } : {}), missing: missingPacks(r.clip, stackPacks()), placed: p.placed };
    clipNote = null;
    clipDialog?.showModal();
  }

  // An address once used keeps its history, withdrawn or not, so a paste never takes one with any.
  async function placement(clip: Clip, host: { address: string; kind: string }) {
    const taken = scene.subjects.map((x) => x.address);
    for (let i = 0; i < 64; i++) {
      const p = placeClip(clip, { system: scene.system, host: host.address, hostKind: host.kind, taken });
      const root = p.placed[0]!.address;
      if (!session || !(await session.history(root)).length) return p;
      taken.push(root);
    }
    return null;
  }
  let placing = $state(false);
  // Add here (V3 gap 2): one world or moon by hand, round the selected star or planet (else the primary).
  let addDialog: HTMLDialogElement | undefined = $state();
  let addFor = $state<{ address: string; label: string; kind: string; mass_sun: number | null } | null>(null);
  const addOrbits = $derived(addFor ? scene.subjects.filter((x) => (addFor!.kind === "star" ? x.kind === "planet" : x.parent === addFor!.address) && x.orbit).map((x) => x.orbit!.a_au) : []);
  function openAdd() {
    const sel = scene.subjects.find((x) => x.address === ws.selection);
    const h = sel && (sel.kind === "star" || sel.kind === "planet") ? sel : primaryOf(scene);
    if (!h) return void (clipNote = "There is no star here to add round.");
    const m = h.facts.find((f) => f.field === "mass_sun")?.value;
    addFor = { address: h.address, label: h.label, kind: h.kind, mass_sun: typeof m === "number" ? m : null };
    addDialog?.showModal();
  }
  async function addClip(clip: Clip, words: string): Promise<string | null> {
    if (!session || !addFor) return "The chart is not ready.";
    const p = await placement(clip, { address: addFor.address, kind: addFor.kind });
    if (!p) return "Could not find a free place for it.";
    const roots = p.placed.filter((x) => x.parent === addFor!.address).map((x) => x.address);
    const r = await placePasted(session, ws, { edits: p.edits, roots, words });
    if (r.status !== "applied") return `Could not add it: ${r.status === "refused" ? r.reason : r.status === "error" ? r.message : "sent to the game master"}.`;
    addDialog?.close();
    clipNote = `Added ${words}, orbiting ${addFor.label}. Undo takes it back.`;
    ws.select(roots[0]!);
    return null;
  }
  async function placePaste() {
    const p0 = pasted;
    if (!session || !p0 || placing) return;
    placing = true;
    try {
      const p = await placement(p0.clip, { address: p0.host, kind: p0.hostKind });
      if (!p) return void (clipNote = "Could not find a free place for it.");
      const roots = p.placed.filter((x) => x.parent === p0.host).map((x) => x.address);
      const r = await placePasted(session, ws, { edits: p.edits, roots, words: p0.words });
      clipDialog?.close();
      if (r.status !== "applied") return void (clipNote = `Could not paste it: ${r.status === "refused" ? r.reason : r.status === "error" ? r.message : "sent to the game master"}.`);
      clipNote = `Pasted ${p0.words}, orbiting ${p0.hostLabel}. Undo takes it back.`;
      ws.select(roots[0]!);
    } finally {
      placing = false;
    }
  }
  // Bodies in the universe rather than the fixture (pasted ones) can be cut; the sample's own are drawn from its file.
  const baseAddrs = new Set(base.subjects.map((x) => x.address));
  const cuttable = $derived(!viewing && !!ws.selection && !baseAddrs.has(ws.selection) && scene.subjects.some((x) => x.address === ws.selection));
  async function cutBody() {
    if (!session || !cuttable) return void (clipNote = "Only a pasted body can be cut for now; the sample's own bodies come from its file.");
    const a = ws.selection!;
    const c = copyText();
    if (!c) return;
    try {
      await navigator.clipboard.writeText(c.text);
    } catch {
      return void (clipNote = "The browser would not let the chart room use the clipboard, so nothing was cut.");
    }
    const r = await withdrawBodies(session, ws, { roots: [a], words: c.words });
    if (r.status !== "applied") return void (clipNote = `Could not cut it: ${r.status === "refused" ? r.reason : r.status === "error" ? r.message : "sent to the game master"}.`);
    ws.select(null);
    clipNote = `Cut ${c.words}. Paste it anywhere on the Build desk; Undo puts it back.`;
  }

  // The menu on any object (V3 gap 38): right-click, or press and hold, on anything carrying `data-address`. Its items
  // are the one action list (ui-contract actions.ts) for the desk in front; each runs the same function as the tool,
  // the key or the More entry that does it, on the object the menu was opened on.
  let objMenu = $state<{ target: ActionTarget; at: { x: number; y: number }; from: Element | null; kicker: string } | null>(null);
  $effect(() =>
    watchObjectMenu(window, (ask, from) => {
      const sub = scene.subjects.find((x) => x.address === ask.address);
      if (!sub || railScale === "workshop") return;
      const host = sub.parent ? scene.subjects.find((x) => x.address === sub.parent) : sub.orbit ? primaryOf(scene) : undefined;
      ws.select(sub.address);
      objMenu = { target: { address: sub.address, kind: sub.kind, label: sub.label }, at: { x: ask.x, y: ask.y }, from, kicker: `${sub.kind.replace(/_/g, " ")}${host ? ` · orbits ${host.label}` : ""}` };
    }),
  );
  const objMenuItems = $derived(
    objMenu ? menuFor(objMenu.target, { desk, sharing: !!share?.state.link, canDescribe: !!minervaHost, canCut: !viewing && !baseAddrs.has(objMenu.target.address), branch: !!ws.branch }) : null,
  );
  // More's edit group says what the object menu says: the same words and keys, from the one list.
  const moreWords = (id: string) => actionById(id)!.label({ address: ws.selection ?? "", kind: "", label: selName ?? "" });
  const moreKey = (id: string) => keyWords(actionById(id)!.key!);
  async function runAction(id: string, address: string) {
    ws.select(address);
    if (id === "open") return toWorld(address);
    if (id === "describe") return describe(address);
    if (id === "copy") return copyBody();
    if (id === "paste") return pasteFromMenu();
    if (id === "add") return openAdd();
    if (id === "cut") return cutBody();
    if (id === "voyage") return go("voyages");
    if (id === "land") return (toWorld(address), go("surface"));
    if (id === "link") {
      const url = `${location.origin}${location.pathname}${location.search}${placeHash({ scale: "world", system: viewing?.system ?? "", world: address })}`;
      try {
        await navigator.clipboard.writeText(url);
        clipNote = `Copied a link to ${placeName(address) ?? "it"}. It opens the chart room on it.`;
      } catch {
        clipNote = `The browser would not let the chart room use the clipboard. The link is ${url}`;
      }
      return;
    }
    if (id === "send" && share) {
      toWorld(address);
      await tick();
      share.show(here());
      clipNote = `The players now see ${placeName(address) ?? "it"}, as far as they know it.`;
    }
  }

  function view(node: HTMLElement, m: { module: ViewModule; params: Record<string, Json> }) {
    const h = m.module.mount(node, { session: session!, workspace: ws, params: m.params, mark: (s) => lightMark(symbols.marker().mark(withStarFacts(subjectOf(s), starFacts.get(s.address))), s.address), symbology: symbols, styles: styleHost, solSheet: solSheetForViews, ...(minervaHost ? { describe } : {}) }, shown);
    handles.push(h);
    return {
      destroy: () => {
        handles.splice(handles.indexOf(h), 1);
        h.dispose();
      },
    };
  }

  let help: HTMLDialogElement | undefined = $state();
  // The Help door (V3 gap 31): What's new, Getting started, Keyboard. The dot marks news this device has not seen.
  let helpPage = $state<HelpPage>("new");
  let newsDot = $state(unseenNews());
  function openHelp(page: HelpPage) {
    helpPage = page;
    help?.showModal();
    if (page === "new") (markNewsSeen(), (newsDot = false));
  }
  // V10: the seats panel opens from a quiet link once the session is shared.
  // Tree order: the star, then each planet followed by its moons.
  const bodies = systemOrder(base);
  function cycle(by: number) {
    const i = bodies.findIndex((b) => b.address === ws.selection);
    ws.select(bodies[(i + by + bodies.length) % bodies.length]!.address);
  }

  function onkeydown(e: KeyboardEvent) {
    // The start sheet takes no chart keys; Escape leaves it for the chart.
    if (startOpen) {
      if (e.key === "Escape") ((startOpen = false), e.preventDefault());
      return;
    }
    const t = e.target as HTMLElement;
    if (t.closest("input, textarea, select, dialog")) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
      e.preventDefault();
      void undo(!e.shiftKey);
      return;
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "o") {
      e.preventDefault();
      given = null;
      offered = null;
      openCampaign(true);
      return;
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "y") {
      e.preventDefault();
      void undo(false);
      return;
    }
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const keys: Record<string, () => void> = {
      e: () => chooseDesk("build"),
      a: () => clipArmed && openAdd(),
      r: () => chooseDesk("run"),
      p: () => (ws.screen = ws.screen === "physics" ? "system" : "physics"),
      v: () => chooseDesk("explore"),
      d: () => chooseDepth(DEPTHS[(DEPTHS.findIndex((x) => x.value === depth) + 1) % DEPTHS.length]!.value),
      n: () => (theme = theme === "chart-room" ? "night-chart" : "chart-room"),
      t: () => document.querySelector<HTMLElement>("[role=slider]")?.focus(),
      c: () => document.querySelector<HTMLElement>(".area-chart [role=button][aria-pressed=true], .area-chart [role=button]")?.focus(),
      "]": () => cycle(1),
      "[": () => cycle(-1),
      l: () => {
        const i = lenses.findIndex((x) => x.id === lens.id);
        chooseLens(lenses[(i + 1) % lenses.length]!.id);
      },
      b: () => document.querySelector<HTMLElement>("[data-bookmarks] > button")?.click(),
      "?": () => openHelp("keys"),
      // Esc clears the selection, then goes up a step.
      Escape: () => (ws.selection ? ws.select(null) : railScale === "world" || railScale === "workshop" ? toSystem() : upAStep()),
    };
    const f = keys[e.key];
    if (f) {
      e.preventDefault();
      f();
    }
  }
</script>

<svelte:window {onkeydown} {oncopy} {oncut} {onpaste} ondragover={(e) => e.dataTransfer?.types.includes("Files") && e.preventDefault()} ondrop={dropFile} onpopstate={fromHash} onhashchange={fromHash} onpointerdown={(e) => moreOpen && !(e.target as HTMLElement).closest(".more") && (moreOpen = false)} />

{#if startOpen}<StartSheet doc={startDoc as never} built={STARTED} {carryOn} sample={base.title === "Kestrel" ? "Kestrel Reach" : base.title} ready={!!session} onopen={fromStart} />{/if}
<div class="app" inert={startOpen} data-depth={depth} data-state-hash={status.hash} data-stack-hash={status.stack} data-physics={status.campaign ? "campaign" : "base"}>
  <header class="bar">
    <!-- Where (top-level design 2.1): the place line is the title, from the largest scale shown to the one in view. -->
    <PlaceLine
      {railScale}
      {place}
      {steps}
      title={scene.title}
      baseTitle={base.title}
      session={session ?? null}
      systemAddress={scene.system}
      subjects={scene.subjects}
      selection={ws.selection}
      {selName}
      {adoptedSystems}
      {viewing}
      {notes}
      {recentElsewhere}
      {goPlace}
      {goTo}
      {toScale}
      switchSystem={(sys) => (showSystem(sys), toSystem())}
    />
    <!-- The system's subtitle belongs to the system; above it the line itself says where. -->
    {#if railScale === "system" || railScale === "world" || railScale === "workshop"}<div class="kicker" class:gone={kickerW < 140} bind:clientWidth={kickerW} title="{scene.subtitle} · {formatYears(ws.now_yr)}">{scene.subtitle} · {formatYears(ws.now_yr)}</div>{/if}
    <div class="spacer"></div>
    <!-- What (2.3): the desk. Explore looks, Build changes, Run is the table. -->
    <Segmented label="Desk" value={desk} options={DESKS} onchange={chooseDesk} slide />
    {#if share && desk === "run"}<OnAir {share} onswitchboard={() => go("switchboard")} bar />{/if}
    {#if session}
      <!-- The desk's own small menu: see the same place through another lens, for this desk only. -->
      <label class="lens" title="See as: what the chart and cards show at this desk (L)">
        <span class="lens-k">See as</span>
        <select value={lens.id} onchange={(e) => { chooseLens(e.currentTarget.value); e.currentTarget.value = lens.id; }} aria-label="Lens" data-lens={lens.id}>
          {#each [["shipped", "For the job"], ["mine", "Your own"], ["pack", "From packs"], ["players", "As players see it"]] as [o, group] (o)}
            {@const ls = lenses.filter((x) => x.origin === o)}
            {#if ls.length}<optgroup label={group}>{#each ls as l (l.id)}<option value={l.id}>{l.label}{l.id === deskLensDefault(desk) ? " (this desk)" : ""}</option>{/each}</optgroup>{/if}
          {/each}
          <option value="__make">{lens.origin === "mine" ? "Change this lens…" : "Make your own…"}</option>
        </select>
      </label>
    {/if}
    <!-- How much (2.4): the depth. On a phone it lives in More. -->
    <div class="depth-bar"><Segmented label="Depth" value={depth} options={DEPTHS} onchange={chooseDepth} slide /></div>
    <!-- Ask (2.5): the agents, promoted from the status line. Tars needs no key. -->
    <button type="button" class="barbtn ask" onclick={() => ((agentsOpen = true), agentsDialog?.showModal())} title="Ask Tars, Orac or Minerva: what can I do here?">Ask</button>
    <div class="more">
      <button type="button" class="barbtn" bind:this={moreBtn} aria-expanded={moreOpen} aria-controls="more-menu" onclick={() => (moreOpen = !moreOpen)}>More{#if newsDot}<span class="news-dot" aria-hidden="true" title="Something new in Help"></span>{/if}</button>
      {#if moreOpen}
        <div class="menu" id="more-menu" role="group" aria-label="More">
          <button type="button" class="ask-more" onclick={() => ((moreOpen = false), (agentsOpen = true), agentsDialog?.showModal())}>Ask Tars, Orac or Minerva</button>
          <div class="depth-more"><span class="depth-k">Depth</span><Segmented label="Depth in More" value={depth} options={DEPTHS} onchange={chooseDepth} /></div>
          {#if clipArmed}
            <!-- The edit group reads its words and keys from the one action list (ui-contract actions.ts), as the object menu does. -->
            <button type="button" data-more-copy disabled={!ws.selection} onclick={() => ((moreOpen = false), void copyBody())}>{moreWords("copy")} {selName ?? "a body"} <kbd>{moreKey("copy")}</kbd></button>
            {#if cuttable}<button type="button" data-more-cut onclick={() => ((moreOpen = false), void cutBody())}>{moreWords("cut")} {selName} <kbd>{moreKey("cut")}</kbd></button>{/if}
            <button type="button" data-more-paste onclick={pasteFromMenu}>{moreWords("paste")} <kbd>{moreKey("paste")}</kbd></button>
            <button type="button" data-more-add onclick={() => ((moreOpen = false), openAdd())}>{moreWords("add")} <kbd>{moreKey("add")}</kbd></button>
          {/if}
          <button type="button" onclick={() => ((moreOpen = false), toWorkshop())}>Workshop</button>
          <button type="button" data-more-open onclick={() => ((moreOpen = false), (offered = null), (given = null), (confirmFresh = false), openCampaign(true))}>Open a file… <kbd>Ctrl+O</kbd></button>
          <button type="button" onclick={() => ((moreOpen = false), (offered = null), (confirmFresh = false), openCampaign())}>Campaign… <small>save, open, your data</small></button>
          <button type="button" onclick={() => ((moreOpen = false), openMake("import"))} title="Import a V3 save or grow a new system, and add it to the campaign">Make a system…</button>
          <button type="button" data-more-start onclick={() => ((moreOpen = false), (startOpen = true))}>Start sheet</button>
          <button type="button" aria-pressed={theme === "night-chart"} onclick={() => ((moreOpen = false), (theme = theme === "chart-room" ? "night-chart" : "chart-room"))}>Night chart <kbd>N</kbd></button>
          <button type="button" data-more-help onclick={() => ((moreOpen = false), openHelp(newsDot ? "new" : "start"))}>Help and what's new{#if newsDot}<span class="news-dot" aria-hidden="true" title="Something new"></span>{/if} <kbd>?</kbd></button>
          <button type="button" data-more-perf aria-pressed={perfOn} onclick={() => ((moreOpen = false), (perfOn = !perfOn))} title="Time every chunk of the app on this device, for the minimum spec">Performance readout <small>{perfOn ? "on" : "off"}</small></button>
        </div>
      {/if}
    </div>
  </header>

  {#if !screen}
    <main class="problem">
      <h2>The workspace document is not valid</h2>
      <ul>{#each badScreens as e (e)}<li>{e}</li>{/each}</ul>
    </main>
  {:else if failure}
    <main class="problem" role="alert">
      <h2>The kernel did not start</h2>
      <p>{failure}</p>
      <p>Nothing kept in this browser has been changed. <button type="button" class="linkbtn" onclick={() => location.reload()}>Try again</button> · <button type="button" class="linkbtn" data-start-without onclick={() => (setAside(), location.reload())}>Start without the kept campaign</button> (it is set aside, not lost) · <button type="button" class="linkbtn" onclick={async () => downloadText(JSON.stringify({ kind: "sse4-diagnostic", failure, problems: [] }, null, 2), "sse4-diagnostic.json")}>Diagnostic file</button></p>
    </main>
  {:else if !session}
    <main class="problem" aria-busy="true"><p>Opening the chart…</p></main>
  {:else}
    <div class="middle">
      <!-- How (2.2): every way of looking at the place in view, in words, built from the screens' own rail entries. -->
      <Rail
        {railScale}
        {rail}
        screen={ws.screen}
        title={scene.title}
        {selName}
        backWords={trail?.scale === "world" && placeName(trail.world) ? placeName(trail.world)! : scene.title}
        back={() => goPlace(trail && trail.system === (viewing?.system ?? "") && placeRank(trail.scale) > placeRank("galaxy") ? trail : { scale: "system", system: viewing?.system ?? "" })}
        {go}
        toWorld={() => toWorld()}
        {toSystem}
        toGalaxy={() => toScale("galaxy")}
        {toWorkshop}
      />
      {#key ws.screen}
      <Stage
        {ws}
        {desk}
        lensId={lens.id}
        ahead={!!ahead}
        {viewSpec}
        {setView}
        {chartMount}
        cardMount={mountFor("card")}
        {openings}
        {view}
        {clipArmed}
        {cuttable}
        {undoNote}
        {clipNote}
        {undo}
        {copyBody}
        {openAdd}
        {cutBody}
        openKeys={() => openHelp("keys")}
      >
        {#snippet extraTools()}<Bookmarks list={ws.bookmarks ?? []} scope={scene.system} mark={markHere} where={whereabouts} go={goBookmark} onchange={(next) => (ws.bookmarks = next)} />{/snippet}
      </Stage>
      {/key}
    </div>
    <footer class="area-ruler" aria-label="Time">
      {#key ws.screen}<div class="ruler-view">{#if mountFor("ruler")}<div class="fill" use:view={mountFor("ruler")!}></div>{/if}</div>{/key}
      <!-- What if and Look ahead live on the ruler (2.7): both are about time. Light and Sound sit with them, as modifiers of any instrument. -->
      <div class="ruler-side" aria-label="Time tools" role="group">
        <span class="whatif" data-branch={ws.branch?.id}>
          {#if ws.branch}
            <b>{ws.branch.label}</b>
            <span data-whatif-edits={ws.branch.edits.length}>{ws.branch.edits.length} {ws.branch.edits.length === 1 ? "change" : "changes"}</span>
            {#if branchHash}<span class="engine" title="What-if state hash: {branchHash}" data-branch-hash={branchHash}>what-if {branchHash.slice(0, 10)}</span>{/if}
            <button type="button" class="linkbtn" onclick={() => compareDialog?.showModal()}>Compare</button>
            <button type="button" class="linkbtn" onclick={() => whatIf("keep")} disabled={!ws.branch.edits.length}>Keep it</button>
            <button type="button" class="linkbtn" onclick={() => whatIf("drop")}>Throw away</button>
          {:else}
            {#if !ahead && !viewing}<button type="button" class="linkbtn quiet" onclick={() => whatIf("open")} title="Try changes on a branch; the chart's history does not move until you keep them">What if…</button>{/if}
            <LookAhead {session} {scene} given={viewing ? viewingSpec : null} writeBack={!!viewing} bind:ahead bind:at={aheadAt} onkept={(note, now_myr, h) => ((whatNote = note), now_myr !== null && (ws.now_yr = ws.campaign_now_yr = ws.campaign_epoch_yr = now_myr * 1e6), (status = { ...status, hash: h }), ws.revision++)} />
          {/if}
          {#if whatNote}<span class="note" role="status" title={whatNote}>{whatNote}</span>{/if}
        </span>
        <span class="strip">
          <LightControl recipes={LIGHT_RECIPES} lights={lit.lights} missing={lit.missing} names={lightNames} {band} bind:value={lightId} key="light.gm" />
          <SoundControl scene={soundScene} deviceTier={status.tier} seed={base.universe.seed} key="sound.gm" />
        </span>
      </div>
    </footer>
    {#if explorersNote}<p class="kept-note" role="alert" data-explorers-note>{explorersNote} <button type="button" class="linkbtn" onclick={() => (explorersNote = null)}>OK</button></p>{/if}
    {#if keptNote}<p class="kept-note" role="status" data-kept-note>{keptNote}{#if recovery && setAsideText()}{" "}<button type="button" class="linkbtn" data-recover-try onclick={() => restoreSetAside() && location.reload()}>Try again</button> · <button type="button" class="linkbtn" onclick={() => downloadText(setAsideText() ?? "", "sse4-set-aside-campaign.json")}>Download it</button> ·{/if} <button type="button" class="linkbtn" onclick={() => (keptNote = null)}>{recovery ? "Carry on fresh" : "OK"}</button></p>{/if}
    <div class="status" aria-label="Session">
      <span>Seat: {session.seat.label}</span>
      <span class="kept">
        <span data-kept={savedAt ?? ""} title={saveProblem ?? "The campaign is kept in this browser after every change"}>{saveProblem ? "Not kept" : savedWords}</span>
      </span>
      {#if share}
        <TableStatus
          {share}
          {session}
          system={scene.system}
          title={scene.title}
          subjects={scene.subjects}
          landingUnsent={ws.screen === "surface" && !!lastLanding}
          focus={ws.selection ?? undefined}
          {here}
          onswitchboard={() => go("switchboard")}
          chip={desk !== "run"}
          party={ws.party ?? null}
          now_yr={ws.campaign_now_yr ?? ws.now_yr}
          onchange={() => (ws.revision++, ws.notePhysics())}
        />
      {/if}
      <!-- The engine's own numbers by depth (2.4): Science adds the device tier, Engine the kernel and the hashes. -->
      {#if depth !== "curious"}
        <span class="engine-line">
          {#if depth === "engine"}<span>kernel {status.kernel}</span>{/if}
          <span title="Measured capability tier of this device">tier {status.tier}</span>
          {#if depth === "engine"}
            <span title="State hash: {status.hash}" data-engine="state">state {status.hash.slice(0, 10)}</span>
            <span title="Pack stack hash: {status.stack}" data-engine="physics">physics {status.campaign ? "campaign" : "base"} {status.stack.slice(0, 8)}</span>
          {/if}
        </span>
      {/if}
    </div>
  {/if}
</div>

{#if perfOn}
  <PerfReadout kernel={status.kernel} edge={session?.device()} from={() => moreBtn} onclose={() => (perfOn = false)} />
{/if}

<dialog bind:this={lensDialog} class="help seats-dialog" aria-label="Lens editor" onclose={() => (lensOpen = !!lensDialog?.open)}>
  {#if lensOpen && session}
    <LensPanel from={lens} {lenses} fieldsByKind={lensFields} kinds={lensKinds} onsave={saveLens} ondelete={deleteLens} />
  {/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<dialog bind:this={compareDialog} class="help seats-dialog" aria-label="Compare">
  <h2>Control and changed</h2>
  {#if ws.branch}
    <p class="hint">The main line against <b>{ws.branch.label}</b>, at {formatYears(ws.now_yr)}. Set: changed on the branch. Follows: derived from what was set.</p>
    {#if differences.length}
      <table class="diff">
        <thead><tr><th scope="col">Body</th><th scope="col">Value</th><th scope="col">Main line</th><th scope="col">What if</th><th scope="col"></th></tr></thead>
        <tbody>
          {#each differences as d (d.address + d.field)}
            <tr data-diff={d.field}><th scope="row">{d.label}</th><td>{d.field_label}</td><td>{d.before}</td><td>{d.after}</td><td class="how">{d.how === "set" ? "set" : "follows"}</td></tr>
          {/each}
        </tbody>
      </table>
    {:else}
      <p>No differences yet. Edit on the chart or a card; the edits go to the what-if.</p>
    {/if}
    <p class="row"><button type="button" onclick={() => whatIf("keep")} disabled={!ws.branch.edits.length}>Keep it</button> <button type="button" onclick={() => whatIf("drop")}>Throw away</button></p>
  {/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<dialog bind:this={agentsDialog} class="help seats-dialog" aria-label="Agents" onclose={() => ((agentsOpen = !!agentsDialog?.open), agentsOpen || ((describing = null), (asking = null)))}>
  {#if agentsOpen}<AgentsPanel tars={tarsHost} orac={oracHost} minerva={minervaHost} describe={describing} ask={asking} />{/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<dialog bind:this={makeDialog} class="help make-dialog" aria-label="Make a system" onclose={() => (makeOpen = !!makeDialog?.open)}>
  {#if makeOpen && session}<MakeSystem {session} start={makeStart} file={makeFile} view={ws.view} onview={setView} onadopted={(sys, note, hash) => (adopted(sys, note, hash), sys[0] && (makeDialog?.close(), showSystem(sys[0].system)))} context={{ mark: (s) => symbols.marker().mark(subjectOf(s)), symbology: symbols, styles: styleHost }} />{/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<dialog bind:this={campaignDialog} class="help seats-dialog" aria-label="Campaign">
  <h2>Campaign</h2>
  <p class="hint">{saveProblem ?? `The campaign is kept in this browser after every change (${savedWords.toLowerCase()}). Save it as a file to keep a copy, move it to another device or hand it on.`}</p>
  <p class="row"><button type="button" onclick={saveFile}>Save campaign</button></p>
  <h3>Open a file</h3>
  <p class="hint">Any file: a saved campaign, a clip, a pack, a Star System Explorer 3 save, or a Universe Sandbox or SpaceEngine file. SSE4 looks inside and says what it is. You can also drop a file anywhere on the chart room.</p>
  <label class="row">File <input type="file" data-open-file onchange={pickFile} /></label>
  {#if given}
    <div class="opened" data-opened={given.sniffed.kind}>
      {#if given.from}<p class="hint" data-from-explorers>From Explorers: <b>{given.from.source.title}</b>{creditWords(given.from.source.creditChain) ? `. ${creditWords(given.from.source.creditChain)}` : ""}. Its bytes match the listing.</p>{/if}
      <p><b>{given.name}</b>: {given.sniffed.what}</p>
      {#if given.sniffed.kind === "v4-campaign"}
        {#if offered?.save}
          <p data-offered={offered.save.saved_at}><b>{offered.save.title}</b>, kept {new Date(offered.save.saved_at).toLocaleString("en-GB", { dateStyle: "medium", timeStyle: "short" })}: {savedEvents(offered.save)} events. Save the campaign open now first if you want to keep it; players' screens reload into the opened one.</p>
          <p class="row"><button type="button" onclick={openOpened}>Open it</button></p>
        {:else if offered}
          <p role="alert" data-offered-problem>{offered.name} cannot be opened here: {offered.problems.join("; ")}.</p>
        {/if}
      {:else if given.sniffed.kind === "v4-postcard"}
        <p class="row"><button type="button" data-open-postcard onclick={openOpened}>Open the postcard</button></p>
      {:else if given.sniffed.kind === "v4-clip"}
        <p class="row"><button type="button" onclick={openOpened}>Paste it on the chart</button></p>
      {:else if given.sniffed.kind === "v3-save"}
        <p class="row"><button type="button" data-bring-v3 onclick={openOpened}>Bring it in</button></p>
      {:else if given.sniffed.kind === "pack"}
        {#if !packCheck}<p class="hint">Checking it with the kernel…</p>
        {:else if packCheck.ok}
          <p data-pack-check>The kernel accepts it. It would change {packCheck.changes.length} {packCheck.changes.length === 1 ? "value" : "values"} on this chart{packCheck.changes.length ? `, such as ${packCheck.changes.slice(0, 3).map((c) => c.label).join(", ")}` : ""}.</p>
          {#if packCheck.report}<p data-pack-report={packCheck.report.pack}>It brings {reportWords(packCheck.report)}{packCheck.report.sources.length ? `, citing ${packCheck.report.sources.map((x) => x.text).slice(0, 2).join(" and ")}${packCheck.report.sources.length > 2 ? " and more" : ""}` : ""}.</p>{/if}
          {#if packNote}<p role="status">{packNote}</p>{:else}<p class="row"><button type="button" onclick={openOpened}>Add it to this campaign</button></p>{/if}
        {:else}<p role="alert" data-pack-problem>It cannot be added: {packCheck.problems.join("; ")}.</p>{/if}
      {:else if given.sniffed.kind === "other-app"}
        <p class="row"><a class="linkbtn" href={CONVERTER_URL} target="_blank" rel="noopener">Open the converter</a></p>
      {/if}
    </div>
  {/if}
  <h3>Start fresh</h3>
  {#if confirmFresh}
    <p role="alert">This forgets the campaign kept in this browser and opens {base.title} as shipped. Save campaign first to keep a copy.</p>
    <p class="row"><button type="button" onclick={startFresh}>Forget it and start fresh</button> <button type="button" onclick={() => (confirmFresh = false)}>Keep it</button></p>
  {:else}
    <p class="row"><button type="button" onclick={() => (confirmFresh = true)}>Start fresh…</button></p>
  {/if}
  {#key campaignOpens}<YourData onbackup={saveFile} report={diagnosticState} />{/key}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<dialog bind:this={addDialog} class="help seats-dialog" aria-labelledby="add-title" data-add onclose={() => !addDialog?.open && (addFor = null)}>
  {#if addFor && session}<AddHere {session} system={scene.system} host={addFor} orbits={addOrbits} packs={stackPacks()} onadd={addClip} />{/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<dialog bind:this={clipDialog} class="help seats-dialog" aria-labelledby="paste-title" data-paste onclose={() => !clipDialog?.open && (pasted = null)}>
  {#if pasted}
    <h2 id="paste-title">Paste {pasted.words}</h2>
    <p>{pasted.placed.length === 1 ? "It" : `The ${pasted.placed.length} bodies`} would join {scene.title}, orbiting <b>{pasted.hostLabel}</b>{pasted.a_au !== undefined ? ` at ${Number(pasted.a_au.toPrecision(3))} AU, as copied` : ""}, at {formatYears(ws.now_yr)} on the ruler.</p>
    {#if pasted.clip.source.site !== "sse4" || pasted.clip.source.creditChain.length}<p class="hint">From {pasted.clip.source.site}{pasted.clip.source.creditChain.length ? `, made by ${pasted.clip.source.creditChain.join(", then ")}` : ""}.</p>{/if}
    {#if pasted.missing.length}
      <p data-paste-missing>It was made with {pasted.missing.map((m) => (m.have ? `${m.id} ${m.version} (this campaign has ${m.have})` : `${m.id} ${m.version}`)).join(", ")}, not stacked here as made. Values {pasted.missing.length === 1 ? "that pack sets" : "those packs set"} would be stamped IMPOSSIBLE until {pasted.missing.length === 1 ? "it is" : "they are"} stacked.</p>
    {/if}
    {#if depth === "engine"}<ul class="engine" data-depth="engine">{#each pasted.placed as b (b.address)}<li>{b.label} ({b.kind}) at {b.address}</li>{/each}</ul>{/if}
    <p class="hint">It goes in as one change: one Undo takes the whole paste back.</p>
  {/if}
  <form method="dialog" class="paste-actions">
    <button type="button" class="primary" data-paste-place disabled={placing || !!ws.branch} onclick={placePaste}>{placing ? "Pasting…" : "Paste here"}</button>
    <button type="submit">Close</button>
  </form>
  {#if ws.branch}<p class="hint">A what-if is open: keep it or throw it away, then paste.</p>{/if}
</dialog>

<dialog bind:this={help} class="help" aria-labelledby="help-title" onclose={() => (newsDot = unseenNews())}>
  <HelpDoor bind:page={helpPage} onstart={() => (help?.close(), (startOpen = true))} onask={() => (help?.close(), (agentsOpen = true), agentsDialog?.showModal())} onsol={() => (help?.close(), void openSol())} />
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

{#if objMenu && objMenuItems}{@const om = objMenu}<ObjectMenu target={om.target} menu={objMenuItems} at={om.at} from={om.from} kicker={om.kicker} onpick={(id) => void runAction(id, om.target.address)} ondesk={chooseDesk} onclose={() => (objMenu = null)} />{/if}

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
  /* Tactility (handover/interface-style.md): every control answers a touch at once, quietly. */
  :global(button, select, [role="button"]) {
    transition:
    background-color var(--sse-motion-quick) var(--sse-ease),
    color var(--sse-motion-quick) var(--sse-ease),
    box-shadow var(--sse-motion-quick) var(--sse-ease),
    transform var(--sse-motion-quick) var(--sse-ease);
  }
  :global(button:not(:disabled):active) {
    transform: translateY(1px);
  }
  /* Basic-tier devices drop motion too (interface-style rule 8). */
  :global([data-tier="basic"] *) {
    --sse-motion-quick: 0ms !important;
    --sse-motion: 0ms !important;
  }
  @media (prefers-reduced-motion: reduce) {
    :global(*) {
      --sse-motion-quick: 0ms !important;
      --sse-motion: 0ms !important;
    }
  }
  .app {
    display: grid;
    grid-template-rows: 64px minmax(0, 1fr) 96px 28px;
    height: 100vh;
    height: 100dvh;
    overflow: hidden;
  }
  .bar {
    display: flex;
    align-items: center;
    gap: 20px;
    padding: 0 24px;
    border-bottom: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    min-width: 0;
  }
  /* The steps above the place in view are quiet; the ones below it are the way back to where the GM was. */
  /* The name keeps its room: the steps above it give way first, then the list beside it. */
  /* Above a system the name is the chart's own ("The cosmic web, 200 Mpc across"): it gives way only after the trail. */
  /* Narrower: the far steps say it in a word; narrower still, only the step above shows. */
  .barbtn {
    height: 32px;
    padding: 0 14px;
    white-space: nowrap;
    border: var(--sse-line) solid var(--sse-ink);
    background: transparent;
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
    cursor: pointer;
  }
  .barbtn:hover,
  .barbtn[aria-expanded="true"] {
    background: var(--sse-ink);
    color: var(--sse-on-ink);
  }
  .more {
    position: relative;
  }
  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 6px);
    z-index: 20;
    display: flex;
    flex-direction: column;
    min-width: 200px;
    padding: 6px 0;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground-raised);
    box-shadow: var(--sse-card-shadow);
    animation: menu-in var(--sse-motion) var(--sse-ease);
    transform-origin: top right;
  }
  @keyframes menu-in {
    from {
      opacity: 0;
      transform: translateY(-4px) scale(0.97);
    }
  }
  .menu button {
    display: flex;
    justify-content: space-between;
    gap: 16px;
    text-align: left;
    font: inherit;
    font-size: 14px;
    padding: 8px 16px;
    border: 0;
    background: none;
    color: var(--sse-ink);
    cursor: pointer;
  }
  .menu button:hover {
    background: var(--sse-ground);
  }
  .menu button[aria-pressed="true"] {
    color: var(--sse-accent-ink);
  }
  .menu .ask-more {
    display: none;
  }
  .menu small {
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .menu kbd {
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
  }
  .middle {
    display: grid;
    grid-template-columns: 172px minmax(0, 1fr);
    min-height: 0;
  }
  /* When the bar is full the kicker gives way first, then the place's name ends in "…"; nothing overlaps. */
  .kicker {
    min-width: 0;
    /* Only the room the place line leaves. */
    flex: 1 1 0;
    font-size: 12px;
    color: var(--sse-ink-muted);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .kicker.gone {
    visibility: hidden;
  }
  .spacer {
    flex: 1;
  }
  .whatif {
    display: inline-flex;
    gap: 10px;
    align-items: center;
  }
  .kept {
    display: inline-flex;
    gap: 8px;
    align-items: center;
  }
  .kept-note {
    position: fixed;
    left: 24px;
    bottom: 40px;
    z-index: 5;
    margin: 0;
    max-width: min(560px, calc(100vw - 48px));
    padding: 8px 12px;
    font-size: 13px;
    background: var(--sse-ground);
    border: var(--sse-line) solid var(--sse-accent);
    color: var(--sse-ink);
  }
  /* The name, pressed, opens the list of the campaign's systems (the list is the name's own, laid over it unseen). */
  .whatif b {
    color: var(--sse-accent-ink);
    font-weight: normal;
  }
  .help .hint {
    color: var(--sse-ink-muted);
    font-size: 12px;
  }
  .diff {
    border-collapse: collapse;
    font-size: 13px;
    margin: 8px 0;
  }
  .diff th,
  .diff td {
    text-align: left;
    padding: 4px 12px 4px 0;
    border-bottom: var(--sse-line) solid var(--sse-rule);
    font-weight: normal;
  }
  .diff td:nth-child(3),
  .diff td:nth-child(4) {
    font-family: var(--sse-font-mono);
  }
  .diff .how {
    color: var(--sse-ink-muted);
    font-size: 11px;
  }
  /* Depth sits in the bar on a wide screen and in More otherwise (design 2.4 and board 4). */
  .depth-bar :global(.seg button) {
    height: 32px;
    padding: 0 10px;
    font-size: 12px;
  }
  .depth-more {
    display: none;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 16px;
    border-bottom: var(--sse-line) solid var(--sse-rule);
  }
  .depth-more .depth-k {
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .menu .depth-more :global(.seg button) {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    height: 36px;
    padding: 0 10px;
    font-size: 12px;
  }
  @media (max-width: 1279px) {
    .depth-bar {
      display: none;
    }
    .depth-more {
      display: flex;
    }
  }
  .lens {
    display: flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .lens select {
    height: 32px;
    max-width: 180px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ground);
    color: var(--sse-ink);
    font-family: var(--sse-font-body);
    font-size: 13px;
  }
  .fill {
    position: absolute;
    inset: 0;
  }
  .area-ruler .fill {
    position: relative;
    height: 100%;
  }
  /* The cutaway sits between the tools and the card instead of under them. */
  /* A screen with no card is a page: its one view scrolls in the stage. */
  /* The view controls (rule 12): always at the top of the stage, beside the tools, on every diagram. */
  /* Below the view controls however many rows they wrap to (their measured height). */
  .area-ruler {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto;
    border-top: var(--sse-line) solid var(--sse-ink);
    min-width: 0;
  }
  .ruler-view {
    min-width: 0;
  }
  .ruler-side {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 6px;
    max-width: 340px;
    padding: 6px 18px;
    border-left: var(--sse-line) solid var(--sse-rule);
    font-size: 12px;
    color: var(--sse-ink-muted);
  }
  .ruler-side .whatif {
    flex-wrap: wrap;
    gap: 4px 12px;
  }
  .ruler-side .note {
    flex-basis: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--sse-ink);
  }
  .strip {
    display: inline-flex;
    flex-wrap: wrap;
    gap: 4px 14px;
    align-items: center;
  }
  .engine-line {
    margin-left: auto;
    display: inline-flex;
    gap: 14px;
    opacity: 0.75;
  }
  .linkbtn {
    color: var(--sse-accent);
    font: inherit;
    background: none;
    border: 0;
    padding: 0;
    cursor: pointer;
    text-decoration: underline;
  }
  .status {
    display: flex;
    gap: 16px;
    align-items: center;
    padding: 0 24px;
    border-top: var(--sse-line) solid var(--sse-rule);
    background: var(--sse-ground-raised);
    font-family: var(--sse-font-mono);
    font-size: 11px;
    color: var(--sse-ink-muted);
    white-space: nowrap;
    overflow: hidden;
  }
  .problem {
    padding: 48px 24px;
    max-width: 640px;
  }
  .problem h2 {
    font-family: var(--sse-font-display);
  }
  .help {
    max-width: 600px;
    width: calc(100vw - 32px);
    box-sizing: border-box;
    border: var(--sse-line) solid var(--sse-ink);
    box-shadow: var(--sse-card-shadow);
    background: var(--sse-ground-raised);
    color: var(--sse-ink);
    padding: 20px 24px;
  }
  .seats-dialog {
    max-width: 640px;
    width: calc(100vw - 32px);
  }
  .make-dialog {
    box-sizing: border-box;
    max-width: 1040px;
    width: calc(100vw - 32px);
    max-height: calc(100vh - 32px);
  }
  .make-dialog form[method="dialog"] {
    margin-top: 12px;
  }
  .seats-dialog form[method="dialog"] {
    margin-top: 16px;
  }
  .paste-actions {
    display: flex;
    gap: 10px;
  }
  .paste-actions .primary {
    background: var(--sse-ink);
    color: var(--sse-ground);
    border: var(--sse-line) solid var(--sse-ink);
    padding: 6px 14px;
    font: inherit;
    cursor: pointer;
  }
  .paste-actions .primary:disabled {
    opacity: 0.5;
    cursor: progress;
  }
  .quiet {
    color: var(--sse-ink-muted);
  }
  .news-dot {
    display: inline-block;
    width: 7px;
    height: 7px;
    margin-left: 6px;
    border-radius: 50%;
    background: var(--sse-accent);
    vertical-align: middle;
  }
  .help::backdrop {
    background: rgba(0, 0, 0, 0.25);
  }
  .help button {
    height: 40px;
    padding: 0 16px;
    border: var(--sse-line) solid var(--sse-ink);
    background: var(--sse-ink);
    color: var(--sse-on-ink);
    font-family: var(--sse-font-body);
  }
  /* Phone: the same card and ruler without the wide chart (H8.5). */
  @media (max-width: 760px) {
    .app {
      grid-template-rows: 56px auto auto auto auto;
      height: auto;
      min-height: 100dvh;
      overflow: visible;
    }
    .bar {
      gap: 10px;
      padding: 0 12px;
    }
    /* The header holds the place line, the lens, the mode switch, Ask and More; the place line gives way first. */
    /* The name gives way first on a phone (the desktop rule keeps it whole and shrinks the far steps). */
    .lens {
      flex: 0 1 auto;
      min-width: 56px;
    }
    .lens select {
      width: 100%;
      max-width: 72px;
      height: 44px;
    }
    /* The narrowest phones keep the place, the desk and More; "See as" waits for a wider screen. */
    @media (max-width: 370px) {
      .lens {
        display: none;
      }
    }
    /* The desk switch stays whole on a phone (design 4), only tighter. */
    .bar :global(.seg button) {
      height: 44px;
      padding: 0 9px;
      font-size: 12px;
    }
    .barbtn {
      height: 44px;
      padding: 0 10px;
    }
    /* Ask moves into More on a phone, so the place line keeps room for its name. */
    .barbtn.ask {
      display: none;
    }
    .menu .ask-more {
      display: flex;
    }
    .lens-k,
    .kicker {
      display: none;
    }
    /* The rail becomes one row of chips that scrolls sideways (design 4). */
    .middle {
      display: contents;
    }
    /* The row scrolls; nothing in it is squeezed (a squeezed group once ran "Life" into "Route to here"). */
    /* The view controls sit over the chart they set, one row, labels dropped. */
    /* Tool screens (symbols, styles, packs...) scroll in a taller window; the flow editor and the Body view flow with the page. */
    .area-ruler {
      position: sticky;
      bottom: 0;
      z-index: 2;
      min-width: 0;
      overflow-x: clip;
      grid-template-columns: minmax(0, 1fr);
      background: var(--sse-ground);
    }
    /* The ruler takes the height it needs on a phone, so it never runs over the time tools below it. */
    .area-ruler .fill {
      height: auto;
    }
    .ruler-side {
      max-width: none;
      flex-direction: row;
      flex-wrap: wrap;
      padding: 4px 12px 6px;
      border-left: 0;
      border-top: 1px solid var(--sse-rule);
    }
    .status {
      flex-wrap: wrap;
      gap: 4px 12px;
      padding: 6px 12px;
    }
  }
</style>
