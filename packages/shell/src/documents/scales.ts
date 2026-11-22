// The scales above a system on the place line (U24): each is a screen document whose rail entry names its scale, so
// the rail at that scale lists it, like any other instrument. A pack or a later package adds more by the same route.
import type { ScreenDoc } from "../../../ui-contract/src/index.ts";
import web from "./scale-web.gm.json";
import cluster from "./scale-cluster.gm.json";
import galaxy from "./scale-galaxy.gm.json";

export const PLACE_SCREENS: Record<string, ScreenDoc> = {
  web: web as unknown as ScreenDoc,
  cluster: cluster as unknown as ScreenDoc,
  galaxy: galaxy as unknown as ScreenDoc,
};
