// The first player views of the finite view stack (H8.2). Each is a
// ViewModule over K9 player scenes; none of them touches the kernel, and
// adding another is adding a module here or in any other package (H8.1).
import type { PlayerContext, PlayerScene, ViewModule } from "../../../ui-contract/src/index.ts";
import { mountPlayer } from "./mount.svelte.ts";
import PlayerChart from "./PlayerChart.svelte";
import PlayerCard from "./PlayerCard.svelte";
import PlayerTable from "./PlayerTable.svelte";
import PlayerText from "./PlayerText.svelte";
import PlayerReadout from "./PlayerReadout.svelte";

type PlayerView = ViewModule<PlayerScene, PlayerContext>;
const scenes = [{ addresses: [], quantity_kinds: [] }];

export const playerChart: PlayerView = {
  id: "sse.view.chart",
  version: "0.1.0",
  scene: "player",
  tiers: ["canvas", "text"],
  inputs: scenes,
  params: [{ name: "distance_field", type: "string", default: "orbit.semi_major_axis_m", meaning: "Field that places a body radially (SI metres)." }],
  mount: (el, ctx, scene) => mountPlayer(PlayerChart, el, ctx, scene),
};

export const playerCard: PlayerView = {
  id: "sse.view.card",
  version: "0.1.0",
  scene: "player",
  tiers: ["text"],
  inputs: scenes,
  params: [
    { name: "fields", type: "list", default: [], meaning: "Fields to list first, in order (array of field names)." },
    { name: "unknown", type: "boolean", default: true, meaning: "List facts the audience has not been given, with their placeholder.", promoted: true },
  ],
  mount: (el, ctx, scene) => mountPlayer(PlayerCard, el, ctx, scene),
};

export const playerTable: PlayerView = {
  id: "sse.view.table",
  version: "0.1.0",
  scene: "player",
  tiers: ["text"],
  inputs: scenes,
  params: [
    { name: "title", type: "string", default: "Bodies", meaning: "Caption.", promoted: true },
    { name: "columns", type: "list", default: ["size_class", "orbit.semi_major_axis_m", "mass_kg", "surface.temp_mean_k"], meaning: "Fields as columns (array of field names); Name is always first." },
    { name: "include_scope", type: "boolean", default: true, meaning: "Include the scope's own subject (the star of a system)." },
  ],
  mount: (el, ctx, scene) => mountPlayer(PlayerTable, el, ctx, scene),
};

export const playerText: PlayerView = {
  id: "sse.view.text",
  version: "0.1.0",
  scene: "player",
  tiers: ["text"],
  inputs: scenes,
  params: [],
  mount: (el, ctx, scene) => mountPlayer(PlayerText, el, ctx, scene),
};

export const playerReadout: PlayerView = {
  id: "sse.view.readout",
  version: "0.1.0",
  scene: "player",
  tiers: ["text"],
  inputs: scenes,
  params: [
    { name: "field", type: "string", default: "surface.temp_mean_k", meaning: "Field to show." },
    { name: "subject", type: "string", default: "selected", meaning: "An address, or \"selected\" for the screen's selection." },
    { name: "style", type: "enum", options: ["value", "gauge", "light"], default: "value", meaning: "Number, gauge, or a status light for a tag.", promoted: true },
    { name: "range", type: "list", default: [0, 1], meaning: "Gauge ends in display units ([min, max])." },
    { name: "tag", type: "string", default: "", meaning: "Tag a status light shows." },
    { name: "label", type: "string", default: "", meaning: "Caption; empty uses the vocabulary's label." },
  ],
  mount: (el, ctx, scene) => mountPlayer(PlayerReadout, el, ctx, scene),
};

export const PLAYER_STACK: readonly PlayerView[] = [playerChart, playerCard, playerTable, playerText, playerReadout];
