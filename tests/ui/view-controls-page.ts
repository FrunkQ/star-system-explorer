// The one page that says what each view switch does at every scale (docs/view-controls.md), made from the screens'
// own control specs so it can never drift from what the app offers. `node tests/ui/view-controls-page.ts --write`
// rewrites the page; view-controls-page.test.ts fails when it is out of date.
import { readFileSync, writeFileSync } from "node:fs";
import { BAND_PICKS, COMPOSITE, COMPOSITE_WORD, FOAM_CONTROLS, VOYAGE_CONTROLS, viewOptions, SCALES, SEEN_FROM, SEEN_WORDS, SCALE_WORDS, VIEW_BANDS, VIEW_BAND_WORDS, VIEW_STYLES, VIEW_STYLE_WORDS, type ViewControlsSpec } from "../../packages/ui-contract/src/index.ts";
import { CLUSTER_CONTROLS } from "../../packages/views/cluster-map/src/controls.ts";
import { BODY_CONTROLS } from "../../packages/views/body/src/controls.ts";
import { COMPARE_CONTROLS } from "../../packages/views/compare/src/controls.ts";
import { CUTAWAY_CONTROLS } from "../../packages/views/cutaway/src/controls.ts";
import { GALAXY_CONTROLS } from "../../packages/views/galaxy-chart/src/controls.ts";
import { SECTOR_CONTROLS } from "../../packages/views/sector/src/controls.ts";
import { SYSTEM_CHART_CONTROLS } from "../../packages/views/system-chart/src/controls.ts";
import { SURFACE_CONTROLS } from "../../packages/views/surface/src/controls.ts";
import { WINDOW_CONTROLS } from "../../packages/views/window/src/controls.ts";

/** Every screen with view controls, from the largest scale to the smallest. */
export const SCREENS: { scale: string; screen: string; spec: ViewControlsSpec }[] = [
  { scale: "Cosmic web", screen: "Cosmic web (foam)", spec: FOAM_CONTROLS },
  { scale: "Galaxies nearby", screen: "Galaxies nearby", spec: CLUSTER_CONTROLS },
  { scale: "Galaxy", screen: "Galaxy chart", spec: GALAXY_CONTROLS },
  { scale: "Local stars", screen: "Sector map", spec: SECTOR_CONTROLS },
  { scale: "Between stars", screen: "Voyages", spec: VOYAGE_CONTROLS },
  { scale: "System", screen: "System chart", spec: SYSTEM_CHART_CONTROLS },
  { scale: "Sky from a world", screen: "Window", spec: WINDOW_CONTROLS },
  { scale: "World", screen: "Body (globe)", spec: BODY_CONTROLS },
  { scale: "On the ground", screen: "Land (surface)", spec: SURFACE_CONTROLS },
  { scale: "Inside a world", screen: "Cutaway", spec: CUTAWAY_CONTROLS },
  { scale: "Side by side", screen: "Size comparison", spec: COMPARE_CONTROLS },
];

const MEANING = `## What each switch means, everywhere

- **View style** says how a place is drawn. **Chart** is the diagram: readable marks, names and lines on the chart room's paper. **3D** turns the same diagram so you can look at it from any side. **True** draws what is really there in light: surfaces, stars, gas, dust, against black space. **Wireframe** is coming with the view styles.
- **Seen from** sets where the camera sits in a view that turns: **Above**, **Tilted**, **Edge-on**, or **Free** (drag to turn, as you like). A view drawn flat is always seen from above.
- **Scale** is **Readable** (sizes and distances squeezed so everything can be read at once) or **Real** (everything in true proportion, so small worlds become points and gaps become huge).
- **HUD** lays names, orbit lines, rings and outlines over the picture. **Off** leaves a photograph; **On** is a heads-up display over it. A diagram always shows its labels, so HUD is only offered where a view can look like a photograph. Pack overlays (a hex grid, territory) draw only with HUD on.
- **Band** is the light the place is seen in: **Visible**, **Infrared**, **Ultraviolet**, **X-ray**, **Radio**, or **Mag** (the magnetic field), and last **Composite**: several bands laid into one picture, as observatory press images are, chosen from a tick list with presets (each band scaled to its own brightest; the legend lists every layer and its colour). A band other than Visible is mapped (it has no colour to see) and says how. A player's seat only gets the bands its ship's sensors carry, in Composite too; a phone lays at most two bands at once.

The rule (Alex, 23 Nov 2026): one set of controls, and similar levels of drawing at every scale, from the cosmic web to a lava tube. Where a switch is not offered yet, the app says why on the switch itself, and the same words are below.`;

export function viewControlsPage(): string {
  const reasons: string[] = [];
  const note = (why: string) => {
    let i = reasons.indexOf(why);
    if (i < 0) i = reasons.push(why) - 1;
    return `—${i + 1}`;
  };
  const cell = (ok: boolean, why?: string) => (ok ? "✓" : note(why ?? "not offered"));
  const rows = SCREENS.map(({ scale, screen, spec }) => {
    const o = viewOptions(spec, { style: "chart", seen: "above" });
    const styles = VIEW_STYLES.map((s) => {
      const opt = o.styles.find((x) => x.value === s)!;
      return cell(!opt.unavailable, opt.unavailable);
    });
    // Seen from, scale and HUD in the view's turnable style if it has one, else Chart.
    const turn = (["3d", "true"] as const).find((s) => !o.styles.find((x) => x.value === s)?.unavailable) ?? "chart";
    const t = viewOptions(spec, { style: turn, seen: "free" });
    const seen = SEEN_FROM.filter((s) => !t.seen.find((x) => x.value === s)!.unavailable).map((s) => SEEN_WORDS[s]);
    const scales = SCALES.filter((s) => !t.scales.find((x) => x.value === s)!.unavailable).map((s) => SCALE_WORDS[s]);
    const hudIn = VIEW_STYLES.filter((s) => spec.hud?.[s] === true).map((s) => VIEW_STYLE_WORDS[s]);
    const bands = BAND_PICKS.map((b) => {
      const opt = t.bands.find((x) => x.value === b)!;
      return opt.unavailable ? note(opt.unavailable) : "✓";
    });
    return `| ${scale} | ${screen} | ${styles.join(" | ")} | ${seen.join(", ")} | ${scales.join(", ")} | ${hudIn.length ? hudIn.join(", ") : "always labelled"} | ${bands.join(" | ")} |`;
  });
  const head = `| Scale | Screen | ${VIEW_STYLES.map((s) => VIEW_STYLE_WORDS[s]).join(" | ")} | Seen from | Scale | HUD in | ${BAND_PICKS.map((b) => (b === COMPOSITE ? COMPOSITE_WORD : VIEW_BAND_WORDS[b])).join(" | ")} |`;
  const rule = `|${" --- |".repeat(2 + VIEW_STYLES.length + 3 + BAND_PICKS.length)}`;
  return [
    "# The view controls at every scale",
    "",
    "Made from the screens' own control specs by `tests/ui/view-controls-page.ts`; do not edit by hand (`node tests/ui/view-controls-page.ts --write`).",
    "",
    MEANING,
    "",
    "## What each screen offers today",
    "",
    "✓ is offered. —n is not offered yet, for reason n below. Seen from, Scale and Band are as offered in the screen's turning style (3D or True) where it has one.",
    "",
    head,
    rule,
    ...rows,
    "| Down a lava tube | Caves | not built yet: the kernel's lava tubes (survey_caves) drawn underground come after the walk |",
    "",
    "### Why a switch is not offered",
    "",
    ...reasons.map((r, i) => `${i + 1}. ${r}.`.replace(/\.\.$/, ".")),
    "",
  ].join("\n");
}

const PATH = new URL("../../docs/view-controls.md", import.meta.url);
if (process.argv.includes("--write")) writeFileSync(PATH, viewControlsPage());
export const pageOnDisk = () => readFileSync(PATH, "utf8");
