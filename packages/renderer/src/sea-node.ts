// The GPU side of sea.ts: the same optics as TSL nodes (WGSL on WebGPU, GLSL on WebGL2), kept beside the CPU
// versions so the two say one thing (sea.test.ts checks the CPU one).
import type { Node } from "three/webgpu";
import { float, max, select, sqrt } from "three/tsl";

/** Unpolarised Fresnel reflectance from above into a liquid of index n, at the cosine of incidence (sea.ts `fresnel`). */
export function fresnelNode(n: Node<"float">, cosI: Node<"float">): Node<"float"> {
  const c = max(cosI, float(0));
  const st2 = float(1).sub(c.mul(c)).div(n.mul(n));
  const ct = sqrt(max(float(1).sub(st2), float(0)));
  const rs = c.sub(n.mul(ct)).div(c.add(n.mul(ct)));
  const rp = n.mul(c).sub(ct).div(n.mul(c).add(ct));
  return select(st2.greaterThanEqual(1), float(1), rs.mul(rs).add(rp.mul(rp)).mul(0.5));
}
