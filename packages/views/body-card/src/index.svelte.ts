import { mount, unmount } from "svelte";
import type { ViewModule } from "../../../ui-contract/src/index.ts";
import BodyCard from "./BodyCard.svelte";

export const bodyCard: ViewModule = {
  id: "sse.view.body-card",
  version: "0.1.0",
  tiers: ["text", "canvas"],
  inputs: [{ event_kinds: ["authored.set"] }],
  params: [
    { name: "columns", type: "integer", default: 2, range: [1, 3], meaning: "Columns of fields on the card.", promoted: true },
    { name: "prose", type: "boolean", default: true, meaning: "Show the subject's prose description." },
    { name: "vocab", type: "string", default: "documents/vocab.plain.json", meaning: "Vocabulary the players' card speaks in, when a lens shows a body as players see it." },
    { name: "stamps", type: "boolean", default: true, meaning: "Show evidence stamps (the lens can also hide them). Honesty stamps always show on the GM view." },
  ],
  intents: [{ kind: "set_value", meaning: "Set a field of the selected subject at the ruler's time." }],
  mount(el, ctx, scene) {
    const props = $state({ ctx, scene, params: ctx.params });
    const c = mount(BodyCard, { target: el, props });
    return {
      update(s) {
        props.scene = s;
      },
      dispose() {
        void unmount(c);
      },
    };
  },
};
