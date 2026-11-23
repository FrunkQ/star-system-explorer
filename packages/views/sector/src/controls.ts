import { flatChart, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/** The Sector map's view controls (V10a, rule 12): a flat chart, and its HUD switch strips the labels and overlays and leaves the grid. */
export const SECTOR_CONTROLS: ViewControlsSpec = { ...flatChart("The sector map is a flat hex grid"), hud: { chart: true } };
