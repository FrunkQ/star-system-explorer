import { flatChart, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/** The Window's view controls (rule 12): the sky as a flat map for now. HUD: off leaves the lights alone, without the
 * ecliptic or names (the table below still names them). */
export const WINDOW_CONTROLS: ViewControlsSpec = {
  ...flatChart("The sky is drawn as a flat map for now"),
  hud: { chart: true },
  // The stars beyond are read in V and Ks (sky_from_here): Visible in the stars' actual colour, Infrared as Ks.
  bands: { visible: true, infrared: true },
};
