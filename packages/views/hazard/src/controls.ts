import { flatChart, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/** The Hazard map's view controls (rule 12): the Profile is a strip drawn flat on a stretched distance scale; the
 * cut (the Map) and its 3D turn come with the Mag layer's tracer. HUD: off leaves the bands without names and marks. */
export const HAZARD_CONTROLS: ViewControlsSpec = { ...flatChart("The Profile is a strip: the turning Map comes later"), hud: { chart: true } };
