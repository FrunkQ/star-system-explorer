// The windows the GM opened on this computer's displays (PB9), for this visit only: a window is the GM's to move,
// split or close while it stays open, and is forgotten when it is closed or the page reloads. Splitting reloads the
// window on the same display with the new split; each part then joins as a screen of its own.
import { displayLink, type Display, type Split } from "../../../ui-contract/src/index.ts";

export interface OpenWindow {
  key: string;
  name: string;
  role: string;
  split: Split;
  href: string;
  win: Window | null;
}

export const opened = $state<OpenWindow[]>([]);
let n = 0;

export function openDisplay(o: { display: Display | null; role: string; name: string; href: string }): void {
  const d = o.display;
  const where = d ? `left=${d.left},top=${d.top},width=${d.width},height=${d.height}` : "width=1280,height=720";
  const key = `sse4-display-${d?.id ?? `w${++n}`}`;
  const win = window.open(o.href, key, `popup,${where}`);
  const had = opened.findIndex((w) => w.key === key);
  const row: OpenWindow = { key, name: o.name, role: o.role, split: "whole", href: o.href, win };
  if (had >= 0) opened[had] = row;
  else opened.push(row);
}

export function resplit(key: string, split: Split): void {
  const w = opened.find((x) => x.key === key);
  if (!w) return;
  w.split = split;
  w.href = displayLink(w.href, w.name, split);
  if (w.win && !w.win.closed) w.win.location.href = w.href;
  else w.win = window.open(w.href, key, "popup");
}

export function closeDisplay(key: string): void {
  const i = opened.findIndex((x) => x.key === key);
  if (i < 0) return;
  opened[i]!.win?.close();
  opened.splice(i, 1);
}
