// Which presets were fired tonight, and when (PB3, design 4.1: a quiet tick and the table time). Session only: not
// kept with the campaign, gone on reload, never a lock (nothing is greyed out or skipped).
const fired = $state<Record<string, string>>({});

export function noteFired(id: string): void {
  fired[id] = new Date().toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" });
}
export function firedAt(id: string): string | undefined {
  return fired[id];
}

// The "just now" strip (PB3, design 4.4 and 4.6): what was sent live tonight, newest first, kept to the last few, so a
// send that worked can be kept as a preset after the fact (Keep) or sent again (Again). Session only.
export interface JustSent {
  id: number;
  shown: import("../../../ui-contract/src/index.ts").Shown;
  dest: import("../../../ui-contract/src/index.ts").Dest;
  at: string;
}
const recent = $state<JustSent[]>([]);
let n = 0;
export function noteSent(shown: JustSent["shown"], dest: JustSent["dest"]): void {
  if ("off" in shown) return;
  const top = recent[0];
  if (top && top.dest === dest && JSON.stringify(top.shown) === JSON.stringify(shown)) return;
  recent.unshift({ id: ++n, shown: JSON.parse(JSON.stringify(shown)) as JustSent["shown"], dest, at: new Date().toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" }) });
  recent.splice(6);
}
export function justSent(): readonly JustSent[] {
  return recent;
}
