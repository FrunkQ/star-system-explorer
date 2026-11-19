// System 3D (V6) as a player view of the finite stack. The GM's chart has
// the same component behind its 2D/3D switch (views/system-chart).
import type { ParamSpec, PlayerContext, PlayerScene, ViewModule } from "../../../ui-contract/src/index.ts";
import { mountPlayer } from "../../player-stack/src/mount.svelte.ts";
import PlayerSystem3D from "./PlayerSystem3D.svelte";

export const SYSTEM3D_PARAMS: ParamSpec[] = [
  { name: "distance_field", type: "string", default: "orbit.semi_major_axis_m", group: "content", meaning: "Field that places a body on its ring (SI metres)." },
  { name: "labels", type: "boolean", default: true, group: "look", meaning: "Write each body's name beside it.", promoted: true },
  { name: "tier", type: "enum", options: ["auto", "canvas"], default: "auto", cost_options: { auto: "medium", canvas: "light" }, group: "behaviour", meaning: "auto uses the GPU (WebGPU, else WebGL2; basic quality: flat-shaded spheres and orbit lines) and falls back to Canvas 2D; canvas always draws with Canvas 2D.", promoted: true },
];

export const playerSystem3d: ViewModule<PlayerScene, PlayerContext> = {
  id: "sse.view.system-3d",
  version: "0.1.0",
  scene: "player",
  box: "fill",
  tiers: ["gpu-basic", "canvas"],
  inputs: [{ addresses: [], quantity_kinds: [] }],
  params: SYSTEM3D_PARAMS,
  mount: (el, ctx, scene) => mountPlayer(PlayerSystem3D, el, ctx, scene),
};
