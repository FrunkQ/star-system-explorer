// Packing a campaign postcard for sending (design section 4 rule 1): the world goes as a clip of the bodies in frame
// and what they orbit, through what the maker's seat can see. A GM's campaign that has been shared goes "as the table
// sees it"; before that, as the GM sees it. Notes, secrets and Known-to records are never in a clip.
import { clipThroughSeat, copyClip, goesInOrigin, type PostcardOrigin, envelope, primaryOf, subtree, within, type Bookmark, type Postcard, type SystemScene, type ViewContext } from "../../../ui-contract/src/index.ts";

export async function packForSending(ctx: ViewContext, scene: SystemScene, b: Bookmark & { at_yr: number }, opts: { packs: readonly { id: string; version: string }[]; kernel: string; credit: readonly string[]; asTable: boolean }): Promise<Postcard | string> {
  if (b.scope !== scene.system) return "Open this postcard's system to send it.";
  const primary = primaryOf(scene);
  if (!primary) return "There is no star here to send with it.";
  const about = b.pose?.about ?? b.selected ?? primary.address;
  const tree = subtree(scene, primary.address);
  const copied = copyClip(scene, primary.address, { packs: opts.packs, credit: [...opts.credit] });
  if (!copied) return "This system could not be copied.";
  const addrOf = new Map(tree.map((s, i) => [`n${i}`, s.address]));
  // In frame: the body itself, everything under it, and what it orbits up to the star.
  const ancestors = new Set<string>();
  for (let s = scene.subjects.find((x) => x.address === about); s; s = s.parent ? scene.subjects.find((x) => x.address === s!.parent) : undefined) ancestors.add(s.address);
  ancestors.add(primary.address);
  let seen: { bodies: Set<string>; facts: Map<string, Set<string>> } | null = null;
  if (opts.asTable) {
    const r = await ctx.session.scene(scene.system, { audience: "everyone" });
    if (!r.ok) return `What the table knows could not be read: ${r.message}`;
    const facts = new Map(r.scene.subjects.map((s) => [s.address, new Set(s.facts.filter((f) => f.value !== undefined).map((f) => f.field))]));
    seen = { bodies: new Set(facts.keys()), facts };
  }
  const clip = clipThroughSeat(copied.clip, {
    body: (n) => {
      const a = addrOf.get(n.id) ?? "";
      return (ancestors.has(a) || within(a, about) || scene.subjects.find((x) => x.address === a)?.parent === about) && (!seen || seen.bodies.has(a));
    },
    fact: (n, field) => !seen || field === "name" || !!seen.facts.get(addrOf.get(n.id) ?? "")?.has(field),
  });
  if (!clip) return "The table does not know this place, so there is nothing to send.";
  // The camera's body as a node of the clip, so a visit can find it wherever the clip is laid out.
  // The star's age goes too: without it the worlds cannot be worked out again where the postcard is opened.
  const rs = await ctx.session.resolvedSystem(scene.system);
  const age = rs.ok ? rs.result.facts[scene.system]?.["system.age"] : undefined;
  const node = [...addrOf].find(([id, a]) => a === about && clip.nodes.some((n) => n.id === id))?.[0];
  // Where it was made, with every authored fact the bodies that go hold (and the system's own), so a visit opens the
  // same world, ground and all. Only bodies the seat sees go (the clip is already through the seat); notes and words
  // for the GM never do (goesInOrigin).
  const nodes: Record<string, string> = {};
  for (const n of clip.nodes) {
    const a = addrOf.get(n.id);
    if (a) nodes[n.id] = a;
  }
  const facts: PostcardOrigin["facts"] = [];
  if (rs.ok) {
    for (const a of [scene.system, ...Object.values(nodes)]) {
      const ev = rs.result.evidence?.[a] ?? {};
      for (const [field, value] of Object.entries(rs.result.facts[a] ?? {})) {
        const e = ev[field]?.evidence ?? "";
        if (!e.startsWith("authored") || !goesInOrigin(field) || value === undefined) continue;
        facts.push({ address: a, field, value, ...(e === "authored-impossible" ? { impossible: true } : {}) });
      }
    }
  }
  const origin: PostcardOrigin = { universe: { id: ctx.session.universe.id, seed: ctx.session.universe.seed }, system: scene.system, nodes, facts };
  return envelope(b, { how: "clip", clip, packs: clip.packs, made_with: opts.kernel, ...(node ? { about: node } : {}), ...(typeof age === "number" && age > 0 ? { age_s: age } : {}), ...(facts.length ? { origin } : {}) }, { name: b.name, credit: opts.credit });
}
