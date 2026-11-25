// Aurora shells on the GPU (VP-AU5 part 1): each line of each driver as a thin glowing shell at its own height over
// the globe, from aurora.ts's plan. The glow rings the magnetic poles at the oval's colatitude with its full width as
// the half-peak width, or covers the whole night side for a diffuse glow. Emission is light added, never covering:
// each surface of a shell gives the radiance of its half of the layer, and a line of sight through a thin shell is
// longer by 1 / cos of its angle to the shell, up to the chord at the limb, so the colours stack and brighten at the
// edge of the world as real aurorae do from orbit. Daylight drowns them: a shell point in sunlight adds nothing.
// The globe is unit-sized at the origin (rendering rule 3); shells carry the globe's turn.
import { AdditiveBlending, DoubleSide, Group, Matrix4, Mesh, SphereGeometry, Vector3 } from "three";
import { MeshBasicNodeMaterial } from "three/webgpu";
import { abs, acos, clamp, dot, exp, float, max, min, normalize, positionLocal, positionWorld, select, smoothstep, uniform, vec3, vec4 } from "three/tsl";
import type { AuroraDraw } from "./aurora.ts";

/** Most shells drawn at once (two hemispheres share one shell). */
export const AURORA_SHELLS_MAX = 8;

export interface AuroraGpu {
  group: Group;
  /** Set the shells for this frame: the plan under its exposure, the light's direction and the globe's turn. */
  set(d: AuroraDraw | null, light: [number, number, number], turn: Matrix4): void;
  dispose(): void;
}

export function makeAurora(): AuroraGpu {
  const group = new Group();
  group.renderOrder = 3;
  const geo = new SphereGeometry(1, 192, 96);
  const shells: { mesh: Mesh; mat: MeshBasicNodeMaterial; set(l: AuroraDraw["layers"][number], axis: AuroraDraw["axis"]): void }[] = [];
  const uL = uniform(new Vector3(0, 0, 1));
  for (let i = 0; i < AURORA_SHELLS_MAX; i++) {
    const uAxis = uniform(new Vector3(0, 0, 1));
    const uColat = uniform(0.3);
    const uWidth = uniform(0.08);
    const uDiffuse = uniform(0);
    const uRgb = uniform(new Vector3(0, 0, 0));
    const uChord = uniform(30);
    const mat = new MeshBasicNodeMaterial({ side: DoubleSide, transparent: true, depthWrite: false });
    mat.blending = AdditiveBlending;
    const p = normalize(positionLocal);
    // Angle from the nearer magnetic pole: ovals ring both.
    const c = acos(clamp(abs(dot(p, uAxis)), 0, 1));
    const sigma = uWidth.div(2.3548);
    const t = c.sub(uColat).div(sigma);
    const ring = select(uDiffuse.greaterThan(0.5), float(1), exp(t.mul(t).mul(-0.5)));
    // Daylight: the shell point's sun, faded over a few degrees of the terminator.
    const nW = normalize(positionWorld);
    const night = smoothstep(float(0.05), float(-0.12), dot(nW, uL));
    // Path through the layer: 1 / cos of the view angle (the camera looks along −z), capped at the limb's chord.
    const cosv = max(abs(nW.z), float(1e-4));
    const path = min(float(1).div(cosv), uChord);
    // Each of the two surfaces gives its half of the layer's column.
    const k = ring.mul(night).mul(path).mul(0.5);
    mat.colorNode = vec4(vec3(uRgb).mul(k), 1);
    const mesh = new Mesh(geo, mat);
    mesh.matrixAutoUpdate = false;
    mesh.visible = false;
    mesh.renderOrder = 3;
    group.add(mesh);
    shells.push({
      mesh,
      mat,
      set(l, axis) {
        uAxis.value.set(axis[0], axis[1], axis[2]);
        uColat.value = l.colatitude_rad;
        uWidth.value = l.width_rad;
        uDiffuse.value = l.diffuse ? 1 : 0;
        uRgb.value.set(l.rgb[0], l.rgb[1], l.rgb[2]);
        // Tangent chord through a shell of thickness t at radius r over t, per surface: √(2r/t).
        uChord.value = Math.sqrt((2 * l.radius) / l.thickness);
      },
    });
  }
  return {
    group,
    set(d, light, turn) {
      uL.value.set(light[0], light[1], light[2]);
      shells.forEach((s, i) => {
        const l = d?.layers[i];
        s.mesh.visible = !!l;
        if (!l || !d) return;
        s.set(l, d.axis);
        s.mesh.matrix.copy(turn).multiply(new Matrix4().makeScale(l.radius, l.radius, l.radius));
        s.mesh.matrixWorldNeedsUpdate = true;
      });
    },
    dispose() {
      geo.dispose();
      for (const s of shells) s.mat.dispose();
    },
  };
}
