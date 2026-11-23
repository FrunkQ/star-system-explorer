import { WHY_WIREFRAME_LATER, type SeenFrom, type ViewControlsSpec } from "../../../ui-contract/src/index.ts";

/**
 * The Body globe's view controls (rule 12). Chart is the survey relief recipe, True the true-colour recipe (a recipe
 * chosen by hand in Engine depth still wins). The globe is always round, so 3D is Chart. Seen from: the pole, a
 * tilted view, the equator, or turned by hand.
 */
/** Scale: the globe is one world, the same either way; Real draws the cutaway below it to scale (Readable spreads it
 * on a log scale). Band: Visible is the globe; Mag replaces it with the field's cut (magnetic fields design 8 and 9: Mag replaces the
 * picture, never overlays it). */
export const BODY_CONTROLS: ViewControlsSpec = { styles: { chart: true, "3d": "The globe is already 3D: Chart draws it", true: true, wireframe: WHY_WIREFRAME_LATER }, scale: { readable: true, real: true }, bands: { visible: true, infrared: "The globe is drawn in visible light for now", ultraviolet: "The globe is drawn in visible light for now", xray: "The globe is drawn in visible light for now", radio: "The globe is drawn in visible light for now", mag: true } };
/** The latitude each fixed view looks down from. */
export const GLOBE_LAT: Record<Exclude<SeenFrom, "free">, number> = { above: 90, tilted: 30, edge: 0 };
