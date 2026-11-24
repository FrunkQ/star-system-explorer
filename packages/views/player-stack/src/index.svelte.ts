// The first player views of the finite view stack (H8.2). Each is a
// ViewModule over K9 player scenes; none of them touches the kernel, and
// adding another is adding a module here or in any other package (H8.1).
import type { PlayerContext, PlayerScene, ViewModule } from "../../../ui-contract/src/index.ts";
import { mountPlayer } from "./mount.svelte.ts";
import { CARD_PARAMS, CHART_PARAMS, READOUT_PARAMS, TABLE_PARAMS, TEXT_PARAMS } from "./schemas.ts";
import PlayerChart from "./PlayerChart.svelte";
import PlayerCard from "./PlayerCard.svelte";
import PlayerTable from "./PlayerTable.svelte";
import PlayerText from "./PlayerText.svelte";
import PlayerReadout from "./PlayerReadout.svelte";
import { playerSystem3d } from "../../system-3d/src/index.svelte.ts";
import { playerSurface } from "../../surface/src/index.svelte.ts";

type PlayerView = ViewModule<PlayerScene, PlayerContext>;
const scenes = [{ addresses: [], quantity_kinds: [] }];

export const playerChart: PlayerView = {
  id: "sse.view.chart",
  version: "0.1.0",
  scene: "player",
  box: "fill",
  tiers: ["canvas", "text"],
  inputs: scenes,
  params: CHART_PARAMS,
  mount: (el, ctx, scene) => mountPlayer(PlayerChart, el, ctx, scene),
};

export const playerCard: PlayerView = {
  id: "sse.view.card",
  version: "0.1.0",
  scene: "player",
  tiers: ["text"],
  inputs: scenes,
  params: CARD_PARAMS,
  mount: (el, ctx, scene) => mountPlayer(PlayerCard, el, ctx, scene),
};

export const playerTable: PlayerView = {
  id: "sse.view.table",
  version: "0.1.0",
  scene: "player",
  tiers: ["text"],
  inputs: scenes,
  params: TABLE_PARAMS,
  mount: (el, ctx, scene) => mountPlayer(PlayerTable, el, ctx, scene),
};

export const playerText: PlayerView = {
  id: "sse.view.text",
  version: "0.1.0",
  scene: "player",
  tiers: ["text"],
  inputs: scenes,
  params: TEXT_PARAMS,
  mount: (el, ctx, scene) => mountPlayer(PlayerText, el, ctx, scene),
};

export const playerReadout: PlayerView = {
  id: "sse.view.readout",
  version: "0.1.0",
  scene: "player",
  tiers: ["text"],
  inputs: scenes,
  params: READOUT_PARAMS,
  mount: (el, ctx, scene) => mountPlayer(PlayerReadout, el, ctx, scene),
};

export const PLAYER_STACK: readonly PlayerView[] = [playerChart, playerCard, playerTable, playerText, playerReadout, playerSystem3d, playerSurface];
