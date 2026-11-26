// The look the class gallery's pre-drawn globes were drawn with (V14i part 2): a fingerprint of everything that changes
// how a class's sample world looks in True (the renderer, the kernel's surface, light and world crates, the physics
// pack's measured colours, the shipped atlases). scripts/build-class-pictures.mjs stamps the set with it and the shell
// build (packages/shell/vite.config.ts) carries the current one, so a gallery whose set was drawn with another look
// draws live instead, and `node scripts/build-class-pictures.mjs --check` says the set needs drawing again.
// Line endings are read as "\n", so a Windows checkout gives the same fingerprint.
import { createHash } from "node:crypto";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

export const LOOK_INPUTS = ["packages/renderer/src", "crates/sse-surface/src", "crates/sse-light/src", "crates/sse-world/src", "packs/base-physics/pack.json", "packs/base-atlas/pack.json", "packs/sse3-classic/pack.json"];

function files(root, p) {
  const at = join(root, p);
  if (!statSync(at, { throwIfNoEntry: false })) return [];
  if (statSync(at).isFile()) return [at];
  return readdirSync(at, { recursive: true })
    .map((f) => join(at, String(f)))
    .filter((f) => statSync(f).isFile() && /\.(ts|rs|json|wgsl|glsl)$/.test(f));
}

/** Twelve hex digits over the look's inputs under `root` (the repository). */
export function lookFingerprint(root) {
  const h = createHash("sha256");
  const all = LOOK_INPUTS.flatMap((p) => files(root, p)).map((f) => [relative(root, f).split("\\").join("/"), f]);
  all.sort((a, b) => (a[0] < b[0] ? -1 : a[0] > b[0] ? 1 : 0));
  for (const [name, f] of all) h.update(name + "\n" + readFileSync(f, "utf8").replace(/\r\n/g, "\n") + "\n");
  return h.digest("hex").slice(0, 12);
}

if (import.meta.url === `file://${process.argv[1]}`) console.log(lookFingerprint(process.cwd()));
