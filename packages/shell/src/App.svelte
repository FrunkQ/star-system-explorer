<!--
  The GM shell: a fixed, curated chart-room layout (H8.8) whose areas are
  filled from a screen document with view modules. The shell reaches the
  kernel only through the session, the same client API a player app uses.
-->
<script lang="ts">
  import { applyEdits, applySetup, type AtomInfo, type SifterDoc, enterFacts, observeOf, type AuthoredFacts, checkScreen, deriveScene, gmSeat, redoEdit, subjectOf, systemOrder, undoEdit, type CheckedScreen, type DerivationDoc, type ScreenDoc, type SystemScene, type ViewHandle, type ViewModule, type ViewSession } from "../../ui-contract/src/index.ts";
  import { openBrowserSession } from "../../ui-contract/src/host-browser.ts";
  import { createShare, type Share } from "./share.svelte.ts";
  import { clearStored, download, readStored, setAside, writeStored } from "./campaign-store.ts";
  import { checkCampaignSave, makeCampaignSave, saveFileName, savedEvents, type CampaignSave } from "../../ui-contract/src/index.ts";
  import SeatsPanel from "./SeatsPanel.svelte";
  import LensPanel from "./LensPanel.svelte";
  import SoundControl from "../../sound/src/SoundControl.svelte";
  import LightControl from "./LightControl.svelte";
  import MakeSystem from "./MakeSystem.svelte";
  import LookAhead from "./LookAhead.svelte";
  import { listedTitle, sceneAt, sceneFromResolved, type LookedAhead, type SystemSpec } from "../../ui-contract/src/index.ts";
  import lightDoc from "./documents/light.recipes.json";
  import { SYMBOLS, checkLightRecipe, describeLight, litFromSystem, paint, type BandRecipe, type Mark } from "../../ui-contract/src/index.ts";
  import AgentsPanel from "../../agents/src/AgentsPanel.svelte";
  import type { OracHost } from "../../agents/src/OracTab.svelte";
  import type { MinervaHost } from "../../agents/src/MinervaTab.svelte";
  import type { TarsHost } from "../../agents/src/TarsTab.svelte";
  import { whyPassage } from "../../agents/src/tars.ts";
  import storyStarters from "../../views/story-finder/src/starters.json";
  import { showReference } from "../../ui-contract/src/index.ts";
  import { oracMay } from "../../agents/src/orac.ts";
  import { atomIssueTarget, flowEntries, lensEntries, lensTarget, loadAtoms, physicsTarget } from "../../agents/src/orac-targets.ts";
  import { dataViewEntries, dataViewTarget, lookEntries, screenTarget, styleTarget, symbologyTarget } from "../../agents/src/orac-looks.ts";
  import { orbitScene, voicesFromSystem } from "../../ui-contract/src/index.ts";
  import { orbitBodies, phasesFrom, withPhases, yearsToSeconds } from "../../ui-contract/src/index.ts";
  import { compareScenes, dropWhatIf, keepWhatIf, openWhatIf } from "../../ui-contract/src/index.ts";
  import { allLenses, checkLens, defaultLens, hasTool, lensAudience, type Lens, type Vocab } from "../../ui-contract/src/index.ts";
  import { themeCss, THEMES } from "../../skins/chart-room/src/index.ts";
  import { ALL_THEMES } from "../../skins/player/src/index.ts";
  import { Segmented, ToolButton, formatYears } from "../../chart-grammar/src/index.ts";
  import workspaceDoc from "./documents/workspace.gm.json";
  import physicsDoc from "./documents/physics.gm.json";
  import packsDoc from "./documents/packs.gm.json";
  import screensDoc from "./documents/screens.gm.json";
  import dataDoc from "./documents/dataviews.gm.json";
  import symbolsDoc from "./documents/symbols.gm.json";
  import bodyDoc from "./documents/body.gm.json";
  import compareDoc from "./documents/compare.gm.json";
  import lifeDoc from "./documents/life.gm.json";
  import stylesScreenDoc from "./documents/styles.gm.json";
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
  import { createWorkspace } from "./workspace.svelte.ts";
  import { loadPref, savePref } from "./prefs.ts";
  import { createSymbols } from "./symbols.svelte.ts";
  import type { ChangeRequest, Json } from "../../ui-contract/src/index.ts";

  // World stacks (P4) ride on their subjects; until the kernel serves them they come from a fixture document.
  const worlds = new Map((worldsDoc as unknown as { links: { body: string; fixture: string }[] }).links.map((w) => [w.body, w.fixture]));
  const base = { ...(sceneDoc as unknown as SystemScene), subjects: (sceneDoc as unknown as SystemScene).subjects.map((x) => (worlds.has(x.address) ? { ...x, world_fixture: worlds.get(x.address) } : x)) };
  const derivations = derivationsDoc as unknown as DerivationDoc;
  const ws = createWorkspace(base.now_yr);
  // Orac is not connected yet (A2): its sample requests come from a fixture document.
  for (const r of (oracDoc as unknown as { requests: ChangeRequest[] }).requests) ws.fileRequest(r);
  let scene = $state<SystemScene>(base);

  // Each screen document is checked before anything mounts; the GM switches between them.
  const SCREENS: Record<string, ScreenDoc> = { system: workspaceDoc as unknown as ScreenDoc, physics: physicsDoc as unknown as ScreenDoc, packs: packsDoc as unknown as ScreenDoc, screens: screensDoc as unknown as ScreenDoc, data: dataDoc as unknown as ScreenDoc, symbols: symbolsDoc as unknown as ScreenDoc, body: bodyDoc as unknown as ScreenDoc, compare: compareDoc as unknown as ScreenDoc, life: lifeDoc as unknown as ScreenDoc, styles: stylesScreenDoc as unknown as ScreenDoc, ...Object.fromEntries(MORE_SCREENS.map((m) => [m.value, m.doc])) };
  const SCREEN_TABS = [
    { value: "system", label: "Chart", key: "P" },
    { value: "physics", label: "Physics", key: "P" },
    { value: "body", label: "Body" },
    { value: "compare", label: "Sizes" },
    { value: "life", label: "Life" },
    ...MORE_SCREENS.filter((m) => m.tab).map((m) => ({ value: m.value, label: m.label })),
    { value: "screens", label: "Screens" },
    { value: "data", label: "Player data" },
    { value: "symbols", label: "Symbols" },
    { value: "styles", label: "Styles" },
    { value: "packs", label: "Packs" },
  ];
  const checks = Object.fromEntries(Object.entries(SCREENS).map(([k, d]) => [k, checkScreen(d, REGISTRY)]));
  const badScreens = Object.values(checks).flatMap((c) => (c.ok ? [] : c.errors));
  const screen = $derived.by((): CheckedScreen | null => {
    const c = checks[ws.screen] ?? checks.system!;
    return c.ok && !badScreens.length ? c.screen : null;
  });
  const mountFor = (area: string) => screen?.mounts.find((m) => m.area === area);

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
  let status = $state({ kernel: "", tier: "", hash: "", stack: "", campaign: false });
  async function loadPacks(): Promise<unknown[]> {
    return Promise.all(stackDoc.packs.map(async (p) => (await fetch(new URL(p, document.baseURI))).json()));
  }

  // Sharing with players (V4): recording starts as the campaign opens, before any fact is entered.
  let share = $state<Share | null>(null);
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
      if (listed.ok) adoptedSystems = listed.systems.filter((x) => x.system !== base.system && x.source !== "facts").map((x) => ({ system: x.system, title: listedTitle(x) }));
      session = s;
      status = { kernel: await s.kernelVersion(), tier: s.device()?.tier ?? "unmeasured", hash: await s.stateHash(), stack: s.stack().hash, campaign: false };
    })
    .catch((e) => (failure = String(e)));

  // Keeping the campaign (U10): the kernel's export plus the workspace overlay, in this browser and as a file.
  const ROOT = `u:${base.universe.id}`;
  let keptNote = $state<string | null>(null);
  let savedAt = $state<string | null>(null);
  let saveProblem = $state<string | null>(null);
  async function reopenKept(s: ViewSession): Promise<Json | null> {
    const { save, problem } = readStored(ROOT);
    if (problem) {
      setAside();
      keptNote = `The campaign kept in this browser could not be reopened (${problem}); it is set aside and a fresh campaign opened.`;
      return null;
    }
    if (!save) return null;
    if (save.stack?.added.length) await s.send({ kind: "stack_packs", packs: save.stack.added });
    if (save.stack?.campaign) await s.send({ kind: "adopt_physics", pack: save.stack.campaign, requests: [] });
    const im = await s.importChronicle(save.export);
    if (!im.ok) {
      setAside();
      keptNote = `The campaign kept in this browser could not be reopened (${im.message}); it is set aside and a fresh campaign opened.`;
      if (save.stack) await s.send({ kind: "stack_packs", packs: [] }).then(() => s.send({ kind: "adopt_physics", pack: null, requests: [] }));
      return null;
    }
    ws.applied = save.workspace.applied;
    ws.undo = save.workspace.undo;
    ws.redo = save.workspace.redo;
    ws.now_yr = save.workspace.now_yr;
    ws.campaign_now_yr = save.workspace.campaign_now_yr ?? save.workspace.now_yr;
    ws.campaign_epoch_yr = save.workspace.campaign_epoch_yr ?? base.now_yr;
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
      workspace: JSON.parse(JSON.stringify({ applied: ws.applied, undo: ws.undo, redo: ws.redo, now_yr: ws.now_yr, campaign_now_yr: ws.campaign_now_yr, campaign_epoch_yr: ws.campaign_epoch_yr })) as CampaignSave["workspace"],
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
  let campaignDialog: HTMLDialogElement | undefined = $state();
  let offered = $state<{ save: CampaignSave | null; problems: string[]; name: string } | null>(null);
  let confirmFresh = $state(false);
  async function pickFile(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    input.value = "";
    if (!file) return;
    let doc: unknown = null;
    try {
      doc = JSON.parse(await file.text());
    } catch {
      doc = null;
    }
    const problems = checkCampaignSave(doc, ROOT);
    offered = { save: problems.length ? null : (doc as CampaignSave), problems, name: file.name };
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
  let lensId = $state(loadPref("lens") ?? "");
  const lenses = $derived.by(() => {
    void ws.revision;
    const s = session;
    const st = s?.stack();
    return allLenses(mine, st ? [...st.base, ...st.added] : [], st ? observeOf(st.base)?.setup : undefined, share?.state.plan);
  });
  const lens = $derived(lenses.find((l) => l.id === lensId) ?? lenses.find((l) => l.id === defaultLens(session?.device()?.tier))!);
  $effect(() => {
    ws.lens = lens;
  });
  function chooseLens(id: string) {
    if (id === "__make") return void ((lensOpen = true), lensDialog?.showModal());
    lensId = id;
    savePref("lens", id);
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
    const aud = lensAudience(l);
    const g = ++lensGen;
    if (!s || !aud) return void (ws.lensScene = null);
    s.scene((factsDoc as unknown as AuthoredFacts).scope, { audience: aud, ...(l.data ? { data_view: l.data } : {}) }).then((r) => {
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
  let lightId = $state(SYMBOLS);
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
  const shown = $derived(ahead && ahead.snapshots[aheadAt] ? sceneAt(timed, ahead, ahead.snapshots[aheadAt]!) : timed);
  const lit = $derived(describeLight(litFromSystem(shown)));
  const lightNames = $derived(new Map(shown.subjects.map((x) => [x.address, x.label])));
  const lightColours = $derived.by(() => {
    const r = LIGHT_RECIPES.find((x) => x.id === lightId);
    return r ? paint(r, lit.lights) : null;
  });
  const lightMark = (m: Mark, address: string): Mark => (lightColours ? { ...m, colour: lightColours.get(address) ?? null } : m);
  const soundScene = $derived(orbitScene(shown.system, voicesFromSystem(shown)));
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
        const read = sceneFromResolved(r.result, v.title);
        const now_yr = v.fresh ? read.now_yr : now;
        if (v.fresh) ((v.fresh = false), shiftCampaign(read.now_yr));
        viewingSpec = r.result.spec ?? null;
        mainScene = scene = { ...read, now_yr, span_yr: [read.span_yr[0], Math.max(read.span_yr[1], now_yr * 1.25)] };
      });
      return;
    }
    // A now moved by a kept look-ahead can pass the fixture's ruler: the ruler grows to keep it in view.
    const at = { ...base, now_yr: now, span_yr: [base.span_yr[0], Math.max(base.span_yr[1], now * 1.25)] as [number, number] };
    const main = deriveScene(s, applyEdits(at, applied), derivations);
    const what = b ? deriveScene(s, applyEdits(at, { ...applied, ...b.applied }), derivations) : main;
    Promise.all([main, what]).then(([m, w]) => {
      if (g !== generation) return;
      mainScene = m;
      scene = w;
    });
  });
  let mainScene = $state<SystemScene>(base);
  // V13 part 2: systems adopted into the campaign this session, and the one on the chart (null: the campaign's own).
  let adoptedSystems = $state<{ system: string; title: string }[]>([]);
  let viewing = $state<{ system: string; title: string; fresh: boolean } | null>(null);
  let viewingSpec = $state<SystemSpec | null>(null);
  function showSystem(system: string) {
    const a = adoptedSystems.find((x) => x.system === system);
    ws.select(null);
    if (a) ((viewing = { ...a, fresh: true }), (ws.mode = "view"));
    else ((viewing = null), (viewingSpec = null), shiftCampaign(base.now_yr));
    ws.revision++;
  }
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
  let makeDialog: HTMLDialogElement | undefined = $state();
  let makeOpen = $state(false);
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
        atomIssueTarget((u) => window.open(u, "_blank", "noopener")),
      ],
      catalogue: () => [...flowEntries(s, atoms), ...lensEntries(lenses, fields()), ...lookEntries({ views: [...REGISTRY, ...PLAYER_STACK] as never, styles: styleHost.registry(), symbologies: symbols.registry() }), ...dataViewEntries(observeOf(s.stack().base)?.setup ?? null)],
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

  function view(node: HTMLElement, m: { module: ViewModule; params: Record<string, Json> }) {
    const h = m.module.mount(node, { session: session!, workspace: ws, params: m.params, mark: (s) => lightMark(symbols.marker().mark(subjectOf(s)), s.address), symbology: symbols, styles: styleHost }, shown);
    handles.push(h);
    return {
      destroy: () => {
        handles.splice(handles.indexOf(h), 1);
        h.dispose();
      },
    };
  }

  let help: HTMLDialogElement | undefined = $state();
  // V10: the seats panel opens from a quiet link once the session is shared.
  let seatsDialog: HTMLDialogElement | undefined = $state();
  let seatsOpen = $state(false);
  // Tree order: the star, then each planet followed by its moons.
  const bodies = systemOrder(base);
  function cycle(by: number) {
    const i = bodies.findIndex((b) => b.address === ws.selection);
    ws.select(bodies[(i + by + bodies.length) % bodies.length]!.address);
  }

  function onkeydown(e: KeyboardEvent) {
    const t = e.target as HTMLElement;
    if (t.closest("input, textarea, select, dialog")) return;
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "z") {
      e.preventDefault();
      void undo(!e.shiftKey);
      return;
    }
    if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === "y") {
      e.preventDefault();
      void undo(false);
      return;
    }
    if (e.ctrlKey || e.metaKey || e.altKey) return;
    const keys: Record<string, () => void> = {
      e: () => (ws.mode = "edit"),
      p: () => (ws.screen = ws.screen === "physics" ? "system" : "physics"),
      v: () => (ws.mode = "view"),
      n: () => (theme = theme === "chart-room" ? "night-chart" : "chart-room"),
      t: () => document.querySelector<HTMLElement>("[role=slider]")?.focus(),
      c: () => document.querySelector<HTMLElement>(".area-chart [role=button][aria-pressed=true], .area-chart [role=button]")?.focus(),
      "]": () => cycle(1),
      "[": () => cycle(-1),
      l: () => {
        const i = lenses.findIndex((x) => x.id === lens.id);
        chooseLens(lenses[(i + 1) % lenses.length]!.id);
      },
      "?": () => help?.showModal(),
      Escape: () => ws.select(null),
    };
    const f = keys[e.key];
    if (f) {
      e.preventDefault();
      f();
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="app">
  <header class="bar">
    <h1 class="title">{scene.title}</h1>
    <div class="kicker">{scene.subtitle} · {formatYears(ws.now_yr)}</div>
    <div class="spacer"></div>
    {#if session}
      <label class="lens" title="Lens: what the chart and cards show (L)">
        <span class="lens-k">Lens</span>
        <select value={lens.id} onchange={(e) => { chooseLens(e.currentTarget.value); e.currentTarget.value = lens.id; }} aria-label="Lens" data-lens={lens.id}>
          {#each [["shipped", "For the job"], ["mine", "Your own"], ["pack", "From packs"], ["players", "As players see it"]] as [o, group] (o)}
            {@const ls = lenses.filter((x) => x.origin === o)}
            {#if ls.length}<optgroup label={group}>{#each ls as l (l.id)}<option value={l.id}>{l.label}</option>{/each}</optgroup>{/if}
          {/each}
          <option value="__make">{lens.origin === "mine" ? "Change this lens…" : "Make your own…"}</option>
        </select>
      </label>
    {/if}
    <div class="screens">
      <Segmented
        label="Screen"
        value={ws.screen}
        options={SCREEN_TABS}
        onchange={(v) => (ws.screen = v)}
      />
    </div>
    <!-- Phone: the same screens as a menu, since the tabs do not fit. -->
    <label class="screen-menu">
      <span class="sr">Screen</span>
      <select value={ws.screen} onchange={(e) => (ws.screen = e.currentTarget.value)} aria-label="Screen menu">
        {#each SCREEN_TABS as t (t.value)}<option value={t.value}>{t.label}</option>{/each}
      </select>
    </label>
    <Segmented
      label="Mode"
      value={ws.mode}
      options={[
        { value: "edit", label: "Edit", key: "E" },
        { value: "view", label: "View", key: "V" },
      ]}
      onchange={(v) => (ws.mode = v)}
    />
    <div class="themes">
      <Segmented
        label="Chart style"
        value={theme}
        options={[
          { value: "chart-room", label: "Chart room", key: "N" },
          { value: "night-chart", label: "Night chart", key: "N" },
        ]}
        onchange={(v) => (theme = v)}
      />
    </div>
    <button type="button" class="night compact" aria-label="Switch between Chart room and Night chart" onclick={() => (theme = theme === "chart-room" ? "night-chart" : "chart-room")}>
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" aria-hidden="true"><path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z" /></svg>
    </button>
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
    </main>
  {:else if !session}
    <main class="problem" aria-busy="true"><p>Opening the chart…</p></main>
  {:else}
    {#key ws.screen}
    <main class="stage" data-screen={ws.screen} data-lens={lens.id} data-branch={ws.branch ? "open" : ahead ? "ahead" : undefined}>
      {#if ws.branch}<div class="branch-stamp" aria-hidden="true">What if · not kept</div>{/if}
      <div class="area-chart">{#if mountFor("chart")}<div class="fill" use:view={mountFor("chart")!}></div>{/if}</div>
      <nav class="tools" aria-label="Tools">
        <ToolButton label="Select" icon="M5 3l14 8-6 2-2 6z" pressed={true} />
        {#if hasTool(ws.lens, "undo")}<ToolButton label="Undo" icon="M9 14L4 9l5-5M4 9h11a5 5 0 0 1 0 10h-3" keyhint="Control+Z" disabled={ws.undo.length === 0 || ws.mode !== "edit" || !!ws.branch} onclick={() => undo(true)} />
        <ToolButton label="Redo" icon="M15 14l5-5-5-5M20 9H9a5 5 0 0 0 0 10h3" keyhint="Control+Shift+Z" disabled={ws.redo.length === 0 || ws.mode !== "edit" || !!ws.branch} onclick={() => undo(false)} />{/if}
        <ToolButton label="Keyboard shortcuts" icon="M3 7h18v10H3zM7 11h1M11 11h1M15 11h2M7 14h10" keyhint="?" onclick={() => help?.showModal()} />
      </nav>
      {#if undoNote}<p class="undo-note" role="status">{undoNote}</p>{/if}
      <aside class="area-card" aria-label={ws.screen === "physics" ? "Change requests" : "Selected body"}>{#if mountFor("card")}<div use:view={mountFor("card")!}></div>{/if}</aside>
    </main>
    <footer class="area-ruler" aria-label="Time">{#if mountFor("ruler")}<div class="fill" use:view={mountFor("ruler")!}></div>{/if}</footer>
    {/key}
    {#if keptNote}<p class="kept-note" role="status" data-kept-note>{keptNote} <button type="button" class="linkbtn" onclick={() => (keptNote = null)}>OK</button></p>{/if}
    <div class="status" aria-label="Session">
      <span>Seat: {session.seat.label}</span>
      <span>kernel {status.kernel}</span>
      <span title="Measured capability tier of this device">tier {status.tier}</span>
      <span title="State hash: {status.hash}" data-state-hash={status.hash}>state {status.hash.slice(0, 10)}</span>
      <span title="Pack stack hash: {status.stack}" data-stack-hash={status.stack}>physics {status.campaign ? "campaign" : "base"} {status.stack.slice(0, 8)}</span>
      <span class="whatif" data-branch={ws.branch?.id}>
        {#if ws.branch}
          <b>{ws.branch.label}</b>
          <span data-whatif-edits={ws.branch.edits.length}>{ws.branch.edits.length} {ws.branch.edits.length === 1 ? "change" : "changes"}</span>
          {#if branchHash}<span title="What-if state hash: {branchHash}" data-branch-hash={branchHash}>what-if {branchHash.slice(0, 10)}</span>{/if}
          <button type="button" class="linkbtn" onclick={() => compareDialog?.showModal()}>Compare</button>
          <button type="button" class="linkbtn" onclick={() => whatIf("keep")} disabled={!ws.branch.edits.length}>Keep it</button>
          <button type="button" class="linkbtn" onclick={() => whatIf("drop")}>Throw away</button>
        {:else}
          {#if !ahead && !viewing}<button type="button" class="linkbtn quiet" onclick={() => whatIf("open")} title="Try changes on a branch; the chart's history does not move until you keep them">What if…</button>{/if}
          <LookAhead {session} {scene} given={viewing ? viewingSpec : null} writeBack={!!viewing} bind:ahead bind:at={aheadAt} onkept={(note, now_myr, h) => ((whatNote = note), now_myr !== null && (ws.now_yr = ws.campaign_now_yr = ws.campaign_epoch_yr = now_myr * 1e6), (status = { ...status, hash: h }), ws.revision++)} />
        {/if}
        {#if whatNote}<span role="status">{whatNote}</span>{/if}
      </span>
      {#if adoptedSystems.length}
        <label class="onchart">On the chart
          <select aria-label="System on the chart" value={viewing?.system ?? ""} onchange={(e) => showSystem(e.currentTarget.value)} data-viewing={viewing?.system ?? ""}>
            <option value="">{base.title}</option>
            {#each adoptedSystems as a (a.system)}<option value={a.system}>{a.title}</option>{/each}
          </select>
        </label>
      {/if}
      <button type="button" class="linkbtn quiet" onclick={() => ((makeOpen = true), makeDialog?.showModal())} title="Import a V3 save or grow a new system, and add it to the campaign">Make a system</button>
      <button type="button" class="linkbtn quiet" onclick={() => ((agentsOpen = true), agentsDialog?.showModal())} title="Tars, Orac and Minerva, on your own key">Agents</button>
      <span class="kept">
        <span data-kept={savedAt ?? ""} title={saveProblem ?? "The campaign is kept in this browser after every change"}>{saveProblem ? "Not kept" : savedWords}</span>
        <button type="button" class="linkbtn quiet" onclick={() => ((offered = null), (confirmFresh = false), campaignDialog?.showModal())} title="Save the campaign as a file, open one, or start fresh">Campaign…</button>
      </span>
      <LightControl recipes={LIGHT_RECIPES} lights={lit.lights} missing={lit.missing} names={lightNames} bind:value={lightId} key="light.gm" />
      <SoundControl scene={soundScene} deviceTier={status.tier} seed={base.universe.seed} key="sound.gm" />
      {#if share}
        <span class="share" aria-label="Players">
          {#if share.state.link}
            <a href={share.state.link} target="_blank" rel="noopener" data-players-link={share.state.link}>players' screen</a>
            <span data-players={share.state.players}>{share.state.players} {share.state.players === 1 ? "screen" : "screens"} · {share.state.seq} events</span>
            <button type="button" class="linkbtn quiet" onclick={() => ((seatsOpen = true), seatsDialog?.showModal())}>{share.state.plan.seats.length > 1 ? `${share.state.plan.seats.length} seats` : "More seats"}</button>
            {#if share.state.waiting.length}
              <span class="requests" role="group" aria-label="Players' requests">
                {#each share.state.waiting as r (r.from + r.id)}
                  <span class="req" data-request={r.id}>
                    {r.intent.kind === "set_value" ? `${r.intent.field} of ${scene?.subjects.find((x) => x.address === (r.intent as { address: string }).address)?.label ?? (r.intent as { address: string }).address} → ${JSON.stringify(r.intent.value)}` : r.intent.kind}
                    <button type="button" class="linkbtn" onclick={async () => (await share!.host.approve(r.id, r.from), ws.notePhysics())}>Approve</button>
                    <button type="button" class="linkbtn" onclick={() => share!.host.decline(r.id, r.from)}>Decline</button>
                  </span>
                {/each}
              </span>
            {/if}
          {:else}
            <button type="button" class="linkbtn" onclick={() => window.open(share!.start(), "_blank", "noopener")}>Share with players</button>
          {/if}
        </span>
      {/if}
    </div>
  {/if}
</div>

<dialog bind:this={seatsDialog} class="help seats-dialog" aria-label="Seats" onclose={() => (seatsOpen = !!seatsDialog?.open)}>
  {#if seatsOpen && share && session}
    <SeatsPanel {share} session={session} bodies={bodies.filter((b) => !b.parent || b.parent === bodies[0]?.address)} scope={(factsDoc as unknown as AuthoredFacts).scope} at_yr={(factsDoc as unknown as AuthoredFacts).at_yr} onchange={() => ws.notePhysics()} />
  {/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

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

<dialog bind:this={agentsDialog} class="help seats-dialog" aria-label="Agents" onclose={() => (agentsOpen = !!agentsDialog?.open)}>
  {#if agentsOpen}<AgentsPanel tars={tarsHost} orac={oracHost} minerva={minervaHost} />{/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<dialog bind:this={makeDialog} class="help make-dialog" aria-label="Make a system" onclose={() => (makeOpen = !!makeDialog?.open)}>
  {#if makeOpen && session}<MakeSystem {session} onadopted={(sys, note, hash) => (adopted(sys, note, hash), sys[0] && (makeDialog?.close(), showSystem(sys[0].system)))} context={{ mark: (s) => symbols.marker().mark(subjectOf(s)), symbology: symbols, styles: styleHost }} />{/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<dialog bind:this={campaignDialog} class="help seats-dialog" aria-label="Campaign">
  <h2>Campaign</h2>
  <p class="hint">{saveProblem ?? `The campaign is kept in this browser after every change (${savedWords.toLowerCase()}). Save it as a file to keep a copy, move it to another device or hand it on.`}</p>
  <p class="row"><button type="button" onclick={saveFile}>Save campaign</button></p>
  <h3>Open a campaign</h3>
  <p class="hint">A file from Save campaign. It replaces the campaign open now; save that first if you want to keep it. Players' screens reload into the opened campaign.</p>
  <label class="row">Campaign file <input type="file" accept=".json,application/json" onchange={pickFile} /></label>
  {#if offered}
    {#if offered.save}
      <p data-offered={offered.save.saved_at}><b>{offered.save.title}</b>, kept {new Date(offered.save.saved_at).toLocaleString("en-GB", { dateStyle: "medium", timeStyle: "short" })}: {savedEvents(offered.save)} events.</p>
      <p class="row"><button type="button" onclick={reopenOffered}>Open it</button></p>
    {:else}
      <p role="alert" data-offered-problem>{offered.name} cannot be opened here: {offered.problems.join("; ")}.</p>
    {/if}
  {/if}
  <h3>Start fresh</h3>
  {#if confirmFresh}
    <p role="alert">This forgets the campaign kept in this browser and opens {base.title} as shipped. Save campaign first to keep a copy.</p>
    <p class="row"><button type="button" onclick={startFresh}>Forget it and start fresh</button> <button type="button" onclick={() => (confirmFresh = false)}>Keep it</button></p>
  {:else}
    <p class="row"><button type="button" onclick={() => (confirmFresh = true)}>Start fresh…</button></p>
  {/if}
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

<dialog bind:this={help} class="help" aria-labelledby="help-title">
  <h2 id="help-title">Keyboard</h2>
  <dl>
    <dt>Tab</dt><dd>Move between the chart's bodies, the card and the ruler</dd>
    <dt>Arrows</dt><dd>On the chart, the next or previous body; on the ruler, 1 % of the span</dd>
    <dt>Enter</dt><dd>Select the focused body; on the card, ink the typed value</dd>
    <dt>Esc</dt><dd>Put a typed value back, or clear the selection</dd>
    <dt>[ and ]</dt><dd>Previous or next body</dd>
    <dt>C, T</dt><dd>Go to the chart, go to the time ruler</dd>
    <dt>Page Up, Page Down, Home, End</dt><dd>On the ruler, larger steps</dd>
    <dt>Drag a body, or Page Up, Page Down on it</dt><dd>Move its orbit out or in (edit mode)</dd>
    <dt>Ctrl+Z, Ctrl+Shift+Z</dt><dd>Undo, redo (each is a new correction on the chart's history)</dd>
    <dt>why?</dt><dd>On any value: where it came from, at three depths (Curious, Science, Engine); "Open the flow" puts its physics on the bench</dd>
    <dt>P</dt><dd>Chart or Physics: the flow editor and change requests</dd>
    <dt>E, V</dt><dd>Edit or view</dd>
    <dt>N</dt><dd>Chart room or Night chart</dd>
    <dt>L</dt><dd>Next lens: what the chart and cards show for the job in hand (Basic, Advanced, Building, Running, as players see it)</dd>
  </dl>
  <form method="dialog"><button type="submit">Close</button></form>
</dialog>

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
  .title {
    margin: 0;
    font-family: var(--sse-font-display);
    font-size: 26px;
    font-weight: 600;
  }
  .kicker {
    font-size: 12px;
    color: var(--sse-ink-muted);
    letter-spacing: 0.08em;
    text-transform: uppercase;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .spacer {
    flex: 1;
  }
  .compact,
  .screen-menu {
    display: none;
  }
  .sr {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
  }
  .branch-stamp {
    position: absolute;
    top: 14px;
    left: 50%;
    transform: translateX(-50%) rotate(-1deg);
    z-index: 3;
    padding: 2px 10px;
    border: var(--sse-line) solid var(--sse-accent);
    color: var(--sse-accent-ink);
    font-family: var(--sse-font-mono);
    font-size: 11px;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    background: var(--sse-ground);
    pointer-events: none;
  }
  .stage[data-branch] {
    outline: 2px dashed var(--sse-accent);
    outline-offset: -4px;
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
  .onchart {
    display: inline-flex;
    gap: 6px;
    align-items: center;
  }
  .onchart select {
    font: inherit;
    font-size: 12px;
    max-width: 14em;
    background: var(--sse-ground);
    color: var(--sse-ink);
    border: 1px solid var(--sse-rule);
  }
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
  .lens {
    display: flex;
    align-items: center;
    gap: 6px;
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
  .stage {
    position: relative;
    min-height: 0;
  }
  .area-chart,
  .fill {
    position: absolute;
    inset: 0;
  }
  .area-chart .fill,
  .area-ruler .fill {
    position: relative;
    height: 100%;
  }
  /* The cutaway sits between the tools and the card instead of under them. */
  .stage[data-screen="body"] .area-chart,
  .stage[data-screen="compare"] .area-chart,
  .stage[data-screen="life"] .area-chart {
    inset: 24px 428px 24px 96px;
    overflow: auto;
  }
  .tools {
    position: absolute;
    left: 24px;
    top: 24px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .undo-note {
    position: absolute;
    left: 88px;
    top: 24px;
    margin: 0;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .area-card {
    position: absolute;
    right: 24px;
    top: 24px;
    width: 380px;
    max-height: calc(100% - 110px);
    overflow: auto;
    padding: 0 6px 6px 0;
  }
  .area-ruler {
    border-top: var(--sse-line) solid var(--sse-ink);
    min-width: 0;
  }
  .share {
    margin-left: auto;
    display: inline-flex;
    gap: 10px;
    align-items: center;
  }
  .requests {
    display: inline-flex;
    gap: 12px;
    color: var(--sse-ink);
  }
  .req {
    display: inline-flex;
    gap: 6px;
    padding: 2px 6px;
    border: var(--sse-line) solid var(--sse-accent);
    border-radius: var(--sse-radius);
  }
  .share a,
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
    max-width: 520px;
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
  .quiet {
    color: var(--sse-ink-muted);
  }
  .help::backdrop {
    background: rgba(0, 0, 0, 0.25);
  }
  .help h2 {
    margin: 0 0 12px;
    font-family: var(--sse-font-display);
  }
  .help dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 16px;
    margin: 0 0 16px;
    font-size: 14px;
  }
  .help dt {
    font-family: var(--sse-font-mono);
    font-size: 13px;
  }
  .help dd {
    margin: 0;
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
    .title {
      font-size: 22px;
      min-width: 0;
      overflow: hidden;
      text-overflow: ellipsis;
      white-space: nowrap;
    }
    /* The header holds the lens, the screen menu, the mode switch and the night button; on a narrow phone the two menus
       give way (they shrink, their text clips) so the bar never runs past the screen edge. */
    .lens,
    .screen-menu {
      flex: 0 1 auto;
      min-width: 56px;
    }
    .lens select {
      width: 100%;
      max-width: 92px;
      height: 44px;
    }
    .lens-k,
    .kicker,
    .themes,
    .screens,
    .tools {
      display: none;
    }
    .screen-menu {
      display: block;
    }
    .screen-menu select {
      width: 100%;
      height: 44px;
      max-width: 110px;
      border: var(--sse-line) solid var(--sse-ink);
      background: transparent;
      color: var(--sse-ink);
      font-family: var(--sse-font-body);
    }
    .compact {
      display: flex;
      align-items: center;
      justify-content: center;
      width: 44px;
      height: 44px;
      border: var(--sse-line) solid var(--sse-ink);
      background: transparent;
      color: var(--sse-ink);
    }
    .stage {
      display: contents;
    }
    .area-chart {
      position: relative;
      height: 44vh;
      border-bottom: var(--sse-line) solid var(--sse-ink);
    }
    .area-chart .fill {
      position: absolute;
    }
    /* Tool screens (symbols, styles, packs...) scroll in a taller window; the flow editor and the Body view flow with the page. */
    .stage:not([data-screen="system"]) .area-chart {
      height: 72dvh;
    }
    .stage:not([data-screen="system"]) .area-chart .fill {
      overflow: auto;
    }
    .stage:is([data-screen="physics"], [data-screen="body"], [data-screen="compare"], [data-screen="life"]) .area-chart {
      height: auto;
    }
    .stage:is([data-screen="physics"], [data-screen="body"], [data-screen="compare"], [data-screen="life"]) .area-chart .fill {
      position: relative;
    }
    .stage[data-screen="body"] .area-chart,
    .stage[data-screen="compare"] .area-chart,
    .stage[data-screen="life"] .area-chart {
      inset: auto;
      padding: 12px 16px;
    }
    .undo-note {
    position: absolute;
    left: 88px;
    top: 24px;
    margin: 0;
    font-size: 13px;
    color: var(--sse-ink-muted);
  }
  .area-card {
      position: static;
      width: auto;
      max-height: none;
      overflow: visible;
      /* A card opening from its body (U12) slides a little sideways; it must never make the page scroll. */
      overflow-x: clip;
      margin: 16px 16px 24px;
      padding: 0 6px 6px 0;
    }
    .area-ruler {
      position: sticky;
      bottom: 0;
      z-index: 2;
      min-width: 0;
      overflow-x: clip;
    }
    .status {
      flex-wrap: wrap;
      gap: 4px 12px;
      padding: 6px 12px;
    }
  }
</style>
