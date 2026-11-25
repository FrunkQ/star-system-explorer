// The aurora from the ground (VP-AU7): the same shells as the globe's (gpu-aurora.ts), seen from inside, on a dome
// round the eye. For each direction the shader follows the ray from the eye out to each shell (in globe radii, the
// ground at 1, the site at the top of the world, z up), reads the ring at that point's magnetic colatitude, and adds
// the line's light times the path through the layer (its thickness over the cosine there, up to the tangent chord),
// dimmed by the air between (the Land screen's own air mass) and drowned where the shell point is in sunlight. So an
// oval far poleward shows as an arc low over the poleward horizon, one overhead fills the sky with curtains of light,
// and nothing shows where the oval is below the horizon. Light is in the Land screen's units (per unit of the star's
// light at the top of the air), times its exposure.
import { AdditiveBlending, BackSide, Mesh, SphereGeometry, Vector3 } from "three";
import { MeshBasicNodeMaterial, type Node } from "three/webgpu";
import { Fn, abs, acos, clamp, dot, exp, float, max, min, normalize, positionWorld, select, smoothstep, sqrt, uniform, vec3, vec4 } from "three/tsl";
import type { AuroraSky } from "./aurora.ts";

/** Shells the ground view draws at once. */
export const AURORA_SKY_SHELLS = 4;

export interface AuroraSkyGpu {
  mesh: Mesh;
  /**
   * Set for this frame: the sky plan, the eye's height over the ground in globe radii, the sun (site frame), the air
   * (optical depth straight up at the eye, R/H for the air mass) and the view's exposure.
   */
  set(a: AuroraSky | null, o: { eye: number; sun: [number, number, number]; tauEye: [number, number, number]; k: number; exposure: number }): void;
  dispose(): void;
}

export function makeAuroraSky(radius = 2.95e6): AuroraSkyGpu {
  const uAxis = uniform(new Vector3(0, 0, 1));
  const uSun = uniform(new Vector3(0, 0, 1));
  const uTau = uniform(new Vector3(0, 0, 0));
  const uK = uniform(1000);
  const uEye = uniform(1);
  const uExp = uniform(1);
  const L = Array.from({ length: AURORA_SKY_SHELLS }, () => ({ r: uniform(1.02), t: uniform(0.005), c: uniform(0.3), w: uniform(0.08), diffuse: uniform(0), rgb: uniform(new Vector3(0, 0, 0)) }));
  const mat = new MeshBasicNodeMaterial({ side: BackSide, transparent: true, depthWrite: false });
  mat.blending = AdditiveBlending;
  const airMass = (mu: Node<"float">) => sqrt(uK.mul(mu).mul(uK.mul(mu)).add(uK.mul(2)).add(1)).sub(uK.mul(mu));
  mat.colorNode = Fn(() => {
    const d = normalize(positionWorld);
    let sum: Node<"vec3"> = vec3(0, 0, 0);
    for (const l of L) {
      // The ray from (0, 0, eye) along d meets the shell of radius r where s = −b + √(b² − c).
      const b = uEye.mul(d.z);
      const c = uEye.mul(uEye).sub(l.r.mul(l.r));
      const s = b.negate().add(sqrt(max(b.mul(b).sub(c), float(0))));
      const q = vec3(d.x.mul(s), d.y.mul(s), uEye.add(d.z.mul(s)));
      const n = normalize(q);
      const cosn = max(abs(dot(n, d)), float(1e-4));
      const path = min(float(1).div(cosn), sqrt(l.r.mul(2).div(l.t)).mul(2));
      const colat = acos(clamp(abs(dot(n, uAxis)), 0, 1));
      const tt = colat.sub(l.c).div(l.w.div(2.3548));
      const ring = select(l.diffuse.greaterThan(0.5), float(1), exp(tt.mul(tt).mul(-0.5)));
      const night = smoothstep(float(0.05), float(-0.12), dot(n, uSun));
      sum = sum.add(vec3(l.rgb).mul(ring.mul(path).mul(night)));
    }
    // The air between the eye and the light, and nothing below the horizon.
    const T = exp(uTau.mul(airMass(max(d.z, float(0.02)))).negate());
    const above = select(d.z.lessThan(0), float(0), float(1));
    return vec4(sum.mul(T).mul(uExp).mul(above), 1);
  })();
  const mesh = new Mesh(new SphereGeometry(radius, 96, 48), mat);
  mesh.renderOrder = 999;
  mesh.frustumCulled = false;
  mesh.visible = false;
  return {
    mesh,
    set(a, o) {
      mesh.visible = !!a && a.layers.length > 0;
      if (!a) return;
      uAxis.value.set(a.axis[0], a.axis[1], a.axis[2]);
      uSun.value.set(o.sun[0], o.sun[1], o.sun[2]);
      uTau.value.set(o.tauEye[0], o.tauEye[1], o.tauEye[2]);
      uK.value = o.k;
      uEye.value = o.eye;
      uExp.value = o.exposure;
      L.forEach((u, i) => {
        const l = a.layers[i];
        if (!l) return void u.rgb.value.set(0, 0, 0);
        u.r.value = l.radius;
        u.t.value = l.thickness;
        u.c.value = l.colatitude_rad;
        u.w.value = l.width_rad;
        u.diffuse.value = l.diffuse ? 1 : 0;
        u.rgb.value.set(l.rgb[0], l.rgb[1], l.rgb[2]);
      });
    },
    dispose() {
      mesh.geometry.dispose();
      mat.dispose();
    },
  };
}
