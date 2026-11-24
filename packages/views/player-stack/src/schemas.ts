// Parameter schemas of the player views (H8.9), apart from the components so
// tools and tests read them without a renderer. Every setting is described
// for Orac; a few are promoted as plain controls.
import type { ParamSpec, PartControl } from "../../../ui-contract/src/index.ts";

export const CHART_PARAMS: ParamSpec[] = [
  { name: "distance_field", type: "string", default: "orbit.semi_major_axis_m", group: "content", meaning: "Field that places a body radially (SI metres)." },
  { name: "labels", type: "boolean", default: true, group: "look", meaning: "Write each body's name beside it.", promoted: true },
  { name: "label_size", type: "number", default: 15, range: [9, 24], unit: "px", group: "look", meaning: "Size of body names." },
  { name: "mark_scale", type: "number", default: 1, range: [0.5, 2.5], group: "look", meaning: "Scale of the body marks (the symbology still decides their look).", promoted: true },
  { name: "show_scale", type: "boolean", default: true, group: "look", meaning: "Note the log scale's range in the corner." },
  { name: "show_unplaced", type: "boolean", default: true, group: "content", meaning: "List bodies whose position the audience does not have beside the chart." },
  { name: "animate", type: "boolean", default: false, cost: "medium", group: "behaviour", meaning: "Bodies drift slowly round their orbits (redraws every frame; off under reduced motion).", promoted: true },
];

export const CARD_PARAMS: ParamSpec[] = [
  { name: "fields", type: "list", default: [], group: "content", meaning: "Fields to list first, in order (array of field names)." },
  { name: "only_fields", type: "boolean", default: false, group: "content", meaning: "Show only the fields listed in `fields`." },
  { name: "unknown", type: "boolean", default: true, group: "content", meaning: "List facts the audience has not been given, with their placeholder.", promoted: true },
  { name: "show_tags", type: "boolean", default: true, group: "content", meaning: "Show the subject's tags as chips.", promoted: true },
  { name: "show_moons", type: "boolean", default: true, group: "content", meaning: "List the subject's moons as links." },
  { name: "show_view", type: "boolean", default: true, group: "look", meaning: "Name the data view the card is seen through, in its cap." },
  { name: "density", type: "enum", options: ["normal", "compact"], default: "normal", group: "look", meaning: "Row spacing.", promoted: true },
];

export const TABLE_PARAMS: ParamSpec[] = [
  { name: "title", type: "string", default: "Bodies", group: "content", meaning: "Caption above the table.", promoted: true },
  { name: "columns", type: "list", default: ["size_class", "orbit.semi_major_axis_m", "mass_kg", "surface.temp_mean_k"], group: "content", meaning: "Fields as columns (array of field names); Name is always first." },
  { name: "include_scope", type: "boolean", default: true, group: "content", meaning: "Include the scope's own subject (the star of a system).", promoted: true },
  { name: "sort_by", type: "string", default: "name", group: "behaviour", meaning: "Column the table is first sorted by (\"name\" or a field)." },
  { name: "sortable", type: "boolean", default: true, group: "behaviour", meaning: "Column headings sort the table when pressed.", promoted: true },
  { name: "max_rows", type: "integer", default: 100, range: [1, 500], group: "content", meaning: "Most rows shown." },
  { name: "empty_text", type: "string", default: "–", group: "look", meaning: "What a cell shows when the audience has no value." },
];

export const TEXT_PARAMS: ParamSpec[] = [
  { name: "max_sentences", type: "integer", default: 0, range: [0, 40], group: "content", meaning: "Most sentences of prose; 0 for all.", promoted: true },
  { name: "show_heading", type: "boolean", default: true, group: "look", meaning: "Name the subject and its kind above the prose.", promoted: true },
  { name: "show_story", type: "boolean", default: true, group: "content", meaning: "Quote story facts (rumours, notes) the audience has." },
  { name: "size", type: "enum", options: ["small", "normal", "large"], default: "normal", group: "look", meaning: "Size of the prose.", promoted: true },
];

export const READOUT_PARAMS: ParamSpec[] = [
  { name: "field", type: "string", default: "surface.temp_mean_k", group: "content", meaning: "Field to show.", promoted: true },
  { name: "subject", type: "string", default: "selected", meaning: "An address, or \"selected\" for the screen's selection." },
  { name: "style", type: "enum", options: ["value", "gauge", "light"], default: "value", meaning: "Number, gauge, or a status light for a tag.", promoted: true },
  { name: "range", type: "list", default: [0, 1], meaning: "Gauge ends in display units ([min, max])." },
  { name: "tag", type: "string", default: "", meaning: "Tag a status light shows." },
  { name: "label", type: "string", default: "", group: "look", meaning: "Caption; empty uses the vocabulary's label.", promoted: true },
];

// The parts' own controls (AT13): what a layout can switch on or off. Text and readouts have none: they are pictures.
export const CHART_CONTROLS: PartControl[] = [{ id: "select", words: "Pick a world on the chart (the screen's other parts follow it)", kind: "button", default: true }];

export const CARD_CONTROLS: PartControl[] = [{ id: "open", words: "Open a moon from its link on the card", kind: "link", default: true }];

export const TABLE_CONTROLS: PartControl[] = [
  { id: "sort", words: "Sort by a column, pressing its heading", kind: "button", default: true },
  { id: "select", words: "Pick a world by its name in the table", kind: "link", default: true },
];

// A picture or a web page on a layout (PB13 part 2b): what a box holds when it is not a place. Neither reads the scene.
/** A web address a box may show: https anywhere, or http on this computer (an OBS or camera page). */
export function addressOk(a: string): boolean {
  return /^https:\/\/[^\s]+$/i.test(a) || /^http:\/\/(localhost|127\.0\.0\.1)(:\d+)?(\/[^\s]*)?$/i.test(a);
}
/** A picture a box may show: a web address as above, a file beside the shell (a pack's), or the GM's own as data. */
export function pictureOk(a: string): boolean {
  return addressOk(a) || /^(packs|documents)\/[a-z0-9._/-]+\.(png|jpe?g|webp|gif|svg)$/i.test(a) || /^data:image\/(png|jpeg|webp|gif);base64,[a-z0-9+/=]+$/i.test(a);
}
/** The GM's own picture travels with the layout to every screen, so it is kept small. */
export const PICTURE_DATA_MAX = 400_000;

export const PICTURE_PARAMS: ParamSpec[] = [
  { name: "src", type: "string", default: "", group: "content", meaning: "The picture: a web address, a pack's file, or the GM's own (kept small, as data).", promoted: true },
  { name: "caption", type: "string", default: "", group: "content", meaning: "Words under the picture.", promoted: true },
  { name: "fit", type: "enum", options: ["contain", "cover"], default: "contain", group: "look", meaning: "Show all of it (contain) or fill the box (cover)." },
];

export const LINK_PARAMS: ParamSpec[] = [
  { name: "address", type: "string", default: "", group: "content", meaning: "The web page: https, or http on this computer (an OBS or camera page).", promoted: true },
  { name: "title", type: "string", default: "", group: "content", meaning: "What the page is, for people who cannot see it." },
];

export const LINK_CONTROLS: PartControl[] = [{ id: "use", words: "Use the page (press, scroll, type in it)", kind: "switch", default: false }];
