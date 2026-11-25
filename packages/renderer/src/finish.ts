// The shared finish (VP1, visual polish design 5.1): the one way every True view turns its scene into the picture on
// the screen. The scene is drawn in linear light into a half-float picture (the pass's default target), so a star's
// disc, a sun glint or a lava lake can be far brighter than white; the eye's scatter spreads that light by the CIE
// glare equation (finish-glare.ts); an exposure scales the light to the eye's; AgX tone-maps it to the screen (it
// keeps bright colours from turning garish), and a little noise below one step of 8-bit output ends banding in dark
// skies and limbs. Charts never come through here: their colours are the theme's, exact.
//
// Grade (design section 3): glare is a camera effect of the eye, on in True. Lens effects are separate passes, off.
import { AgXToneMapping, SRGBColorSpace, type Camera, type Scene } from "three";
import { RenderPipeline, type WebGPURenderer } from "three/webgpu";
import { float, fract, pass, renderOutput, screenCoordinate, sin, uniform, vec3, vec4, dot, vec2 } from "three/tsl";
import { gaussianBlur } from "three/addons/tsl/display/GaussianBlurNode.js";
import { glareLadder, glareTotal, KERNEL, rungScale } from "./finish-glare.ts";

export interface FinishFrame {
  /** Scales linear light before tone mapping: 1 shows a white surface in the star's daylight as white. */
  exposure: number;
  /** The angle one pixel spans, degrees (the view's vertical field over its height): sets the glare's size. */
  pixel_deg: number;
  /** The eye's glare; false for a view that draws its own (or none). */
  glare?: boolean;
}

export interface Finish {
  /** Draws `scene` from `camera` through the finish. Rebuilds its chain only when the glare's ladder changes. */
  render(scene: Scene, camera: Camera, f: FinishFrame): void;
  dispose(): void;
}

/** How many doubling blurs the glare uses at most (out to 2^5 = 32 times the first). */
export const GLARE_RUNGS = 6;
export function startFinish(renderer: WebGPURenderer): Finish {
  const pipe = new RenderPipeline(renderer);
  pipe.outputColorTransform = false;
  const uExposure = uniform(1);
  let built: { scene: Scene; camera: Camera; sig: string; dispose: () => void } | null = null;

  const build = (scene: Scene, camera: Camera, f: FinishFrame) => {
    const ladder = f.glare === false ? [] : glareLadder(f.pixel_deg, GLARE_RUNGS);
    const sig = ladder.map((r) => `${r.sigma_px}:${r.weight}`).join("|");
    if (built && built.scene === scene && built.camera === camera && built.sig === sig) return;
    built?.dispose();
    const scenePass = pass(scene, camera);
    const colour = scenePass.getTextureNode("output");
    // The sharp picture keeps what the eye does not scatter; each rung adds its share, blurred to its width.
    let light = colour.rgb.mul(float(1 - glareTotal(ladder)));
    const blurs: { dispose(): void }[] = [];
    for (const r of ladder) {
      const { scale, step } = rungScale(r.sigma_px);
      const b = gaussianBlur(colour, float(step), KERNEL, { resolutionScale: scale });
      blurs.push(b);
      light = light.add(b.rgb.mul(float(r.weight)));
    }
    const shown = renderOutput(vec4(light.mul(uExposure), 1), AgXToneMapping, SRGBColorSpace);
    // Interleaved gradient noise (Jimenez 2014), under half a step of 8-bit output either way.
    const n = fract(sin(dot(screenCoordinate.xy, vec2(12.9898, 78.233))).mul(43758.5453));
    pipe.outputNode = vec4(shown.rgb.add(vec3(n.sub(0.5).div(255))), 1);
    pipe.needsUpdate = true;
    built = { scene, camera, sig, dispose: () => (scenePass.dispose(), blurs.forEach((b) => b.dispose())) };
  };

  return {
    render(scene, camera, f) {
      build(scene, camera, f);
      uExposure.value = f.exposure;
      pipe.render();
    },
    dispose() {
      built?.dispose();
      built = null;
      pipe.dispose();
    },
  };
}
