// What a view shows when nothing is chosen, and other small shared rules.
import { parentOf, type PlayerScene, type PlayerSubject } from "../../../ui-contract/src/index.ts";

/** The selected subject, else the first body below the scope, else the scope itself. */
export function focusOf(scene: PlayerScene, selected: string | null): PlayerSubject | undefined {
  return (
    scene.subjects.find((s) => s.address === selected) ??
    scene.subjects.find((s) => s.address !== scene.scope && parentOf(s.address, scene.scope) === null) ??
    scene.subjects[0]
  );
}

/** Bodies directly in the scope (planets, belts), not their moons and not the scope's own subject. */
export function topLevel(scene: PlayerScene): PlayerSubject[] {
  return scene.subjects.filter((s) => s.address !== scene.scope && parentOf(s.address, scene.scope) === null);
}

export function childrenOf(scene: PlayerScene, address: string): PlayerSubject[] {
  return scene.subjects.filter((s) => parentOf(s.address, scene.scope) === address);
}
