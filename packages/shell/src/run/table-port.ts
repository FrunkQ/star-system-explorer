// The table as the Switchboard sees it (PB2, ui-contract `TablePort`), made from the shell's share.
import type { Shown, TablePort } from "../../../ui-contract/src/index.ts";
import type { Share } from "../share.svelte.ts";

export function tablePort(share: Share, live: () => Shown): TablePort {
  return {
    get state() {
      const st = share.state;
      return {
        board: st.board,
        roles: st.plan.seats.filter((x) => x.authority !== "gm").map((x) => ({ id: x.id, label: x.id === "table" ? "Table" : x.label, sealed: x.sealed, audience: x.audience })),
        players: st.players,
        link: st.link,
        can_undo: st.can_undo,
        can_redo: st.can_redo,
        waiting: st.waiting.length,
      };
    },
    act: (f) => share.act(f),
    undo: () => share.undo(),
    redo: () => share.redo(),
    start: () => share.start(live()),
    links: (role) => share.links(role),
    live,
  };
}
