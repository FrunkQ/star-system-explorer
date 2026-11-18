// Mounting is the same for every player view: props in a $state object, so
// update(scene) re-renders without remounting.
import { mount, unmount, type Component } from "svelte";
import type { PlayerContext, PlayerScene, ViewHandle } from "../../../ui-contract/src/index.ts";

type Props = { ctx: PlayerContext; scene: PlayerScene };

export function mountPlayer(C: Component<Props>, el: HTMLElement, ctx: PlayerContext, scene: PlayerScene): ViewHandle<PlayerScene> {
  const props = $state<Props>({ ctx, scene });
  const c = mount(C, { target: el, props });
  return {
    update(s) {
      props.scene = s;
    },
    dispose() {
      void unmount(c);
    },
  };
}
