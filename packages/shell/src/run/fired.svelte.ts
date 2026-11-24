// Which presets were fired tonight, and when (PB3, design 4.1: a quiet tick and the table time). Session only: not
// kept with the campaign, gone on reload, never a lock (nothing is greyed out or skipped).
const fired = $state<Record<string, string>>({});

export function noteFired(id: string): void {
  fired[id] = new Date().toLocaleTimeString("en-GB", { hour: "2-digit", minute: "2-digit" });
}
export function firedAt(id: string): string | undefined {
  return fired[id];
}
