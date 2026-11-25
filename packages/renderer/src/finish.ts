// The shared finish (VP1, visual polish design 5.1): the one way every True view turns its scene into the picture on
// the screen. The scene is drawn in linear light into a half-float picture (the pass's default target), so a star's
// disc, a sun glint or a lava lake can be far brighter than white; the eye's scatter spreads that light by the CIE
// glare equation (finish-glare.ts); an exposure scales the light to the eye's; AgX tone-maps it to the screen (it
// keeps bright colours from turning garish), and a little noise below one step of 8-bit output ends banding in dark
// skies and limbs. Charts never come through here: their colours are the theme's, exact.
//
// Grade (design section 3): glare is a camera effect of the eye, on in True. Lens effects are separate passes, off.
//
// Quality (VP4, VP5; the tier controller true-run.ts sets these): the glare ladder's rung count and the size of its
// pictures are the tier knob (Light 3 rungs at quarter size, High 6 at half); a render scale below 1 draws the scene
// smaller and enlarges it by Catmull-Rom (enlarge.ts) rather than the screen's bilinear; and at rest the finish keeps
// the running mean of the jittered half-float scene (`accumulate`), so glare and exposure act on the averaged picture.
import { AgXToneMapping, HalfFloatType, RenderTarget, SRGBColorSpace, Vector2, type Camera, type Scene } from "three";
import { MeshBasicNodeMaterial, QuadMesh, RenderPipeline, type WebGPURenderer } from "three/webgpu";
import { fract, min, mix, renderOutput, screenCoordinate, screenUV, sin, texture, uniform, vec3, vec4, dot, vec2 } from "three/tsl";
import { gaussianBlur } from "three/addons/tsl/display/GaussianBlurNode.js";
import { glareLadder, glareTotal, KERNEL, rungScale } from "./finish-glare.ts";
import { enlargeNode } from "./enlarge.ts";

export interface FinishFrame {
  /** Scales linear light before tone mapping: 1 shows a white surface in the star's daylight as white. */
  exposure: number;
  /** The angle one pixel spans, degrees (the view's vertical field over its height): sets the glare's size. */
  pixel_deg: number;
  /** The eye's glare; false for a view that draws its own (or none). */
  glare?: boolean;
  /** Most rungs in the glare ladder (the tier's knob; default all six). Fewer rungs: the last takes the rest of the reach. */
  rungs?: number;
  /** The largest size of a glare rung's picture against the view (the tier's knob: a quarter on Light, half on High; default 1). */
  glare_scale?: number;
  /** Fraction of the view's pixels the scene is drawn at (default 1); below 1 it is enlarged by Catmull-Rom. */
  render_scale?: number;
  /**
   * Sharpen at rest (still.ts): the index of this frame in the running mean (0 starts a new mean). Absent: no
   * averaging. The caller jitters the camera; the finish only averages what it is given.
   */
  accumulate?: number;
}

export interface Finish {
  /** Draws `scene` from `camera` through the finish. Its chain is built once; the ladder and size are numbers set each frame. */
  render(scene: Scene, camera: Camera, f: FinishFrame): void;
  dispose(): void;
}

/** How many doubling blurs the glare uses at most (out to 2^5 = 32 times the first). */
export const GLARE_RUNGS = 6;
export function startFinish(renderer: WebGPURenderer): Finish {
  const pipe = new RenderPipeline(renderer);
  pipe.outputColorTransform = false;
  const uExposure = uniform(1);
  // The scene in linear light, and two pictures for the running mean (ping-pong).
  const half = { type: HalfFloatType, depthBuffer: true };
  const sceneRT = new RenderTarget(1, 1, half);
  const acc = [new RenderTarget(1, 1, { ...half, depthBuffer: false }), new RenderTarget(1, 1, { ...half, depthBuffer: false })];
  let accAt = 0;
  const uWeight = uniform(1);
  const prev = texture(acc[0]!.texture);
  const cur = texture(sceneRT.texture);
  const accMat = new MeshBasicNodeMaterial();
  accMat.colorNode = mix(prev.sample(screenUV), cur.sample(screenUV), uWeight);
  const accQuad = new QuadMesh(accMat);
  // What the finish shows: the scene, or the mean; its value is swapped each frame.
  const src = texture(sceneRT.texture);
  const uSrcSize = uniform(new Vector2(1, 1));
  const size = new Vector2();
  // Built once: six blur slots whose weight, reach and picture size are set each frame, and a switch for enlarging.
  // A tier step or rest changes only these numbers, never the shaders, so the picture never waits on a compile (a
  // rebuilt chain drew black for seconds on a slow WebGL2 device while its shaders compiled).
  const uEnlarge = uniform(0);
  const uSharp = uniform(1);
  const slots = Array.from({ length: GLARE_RUNGS }, () => {
    const weight = uniform(0);
    const reach = uniform(1);
    const blur = gaussianBlur(src, reach, KERNEL, { resolutionScale: 1 / 64 });
    return { weight, reach, blur };
  });
  // Every read is kept finite: a star's point can pass half-float's top (65504) and an infinity times a zero weight,
  // or through the enlarger's negative lobes, is NaN, which the blurs would spread over the whole picture as black.
  const TOP = 6e4;
  const read = (p: Parameters<typeof src.sample>[0]) => min(src.sample(p), vec4(TOP));
  // The sharp picture: enlarged by Catmull-Rom when drawn small (both read; the switch picks).
  const plain = read(screenUV);
  const sharp = mix(plain, enlargeNode(read, uSrcSize, screenUV), uEnlarge);
  // The sharp picture keeps what the eye does not scatter; each rung adds its share, blurred to its width.
  let light = sharp.rgb.mul(uSharp);
  for (const k of slots) light = light.add(min(k.blur.rgb, vec3(TOP)).mul(k.weight));
  const shown = renderOutput(vec4(light.mul(uExposure), 1), AgXToneMapping, SRGBColorSpace);
  // Interleaved gradient noise (Jimenez 2014), under half a step of 8-bit output either way.
  const n = fract(sin(dot(screenCoordinate.xy, vec2(12.9898, 78.233))).mul(43758.5453));
  pipe.outputNode = vec4(shown.rgb.add(vec3(n.sub(0.5).div(255))), 1);

  let ladderKey = "";
  /** Set the slots for this frame's ladder: `scale` is the drawing's size against the view. */
  const setLadder = (f: FinishFrame, scale: number) => {
    const rungs = Math.min(GLARE_RUNGS, Math.round(f.rungs ?? GLARE_RUNGS));
    const cap = Math.max(1 / 64, Math.min(1, f.glare_scale ?? 1));
    const key = `${f.glare !== false}|${rungs}|${cap}|${f.pixel_deg}|${scale}`;
    if (key === ladderKey) return;
    ladderKey = key;
    const ladder = f.glare === false || rungs <= 0 ? [] : glareLadder(f.pixel_deg, rungs);
    uSharp.value = 1 - glareTotal(ladder);
    uEnlarge.value = scale < 1 ? 1 : 0;
    slots.forEach((k, i) => {
      const r = ladder[i];
      if (!r) {
        // An unused slot: no share, and the smallest picture so it costs next to nothing.
        k.weight.value = 0;
        k.reach.value = 1;
        k.blur.resolutionScale = 1 / 64;
        return;
      }
      // The rung's width in the drawing's own pixels, and its picture's size against the drawing.
      const { scale: rs, step } = rungScale(r.sigma_px * scale);
      // A smaller rung picture than its blur asks for: the blur's samples spread further apart to keep its width.
      const use = Math.min(rs, cap / scale, 1);
      k.weight.value = r.weight;
      k.reach.value = (step * use) / rs;
      k.blur.resolutionScale = use;
    });
  };

  return {
    render(scene, camera, f) {
      const scale = Math.max(0.25, Math.min(1, f.render_scale ?? 1));
      renderer.getDrawingBufferSize(size);
      const w = Math.max(1, Math.round(size.x * scale));
      const h = Math.max(1, Math.round(size.y * scale));
      if (sceneRT.width !== w || sceneRT.height !== h) {
        sceneRT.setSize(w, h);
        for (const a of acc) a.setSize(w, h);
      }
      uSrcSize.value.set(w, h);
      setLadder(f, scale);
      const was = renderer.getRenderTarget();
      renderer.setRenderTarget(sceneRT);
      renderer.render(scene, camera);
      if (f.accumulate !== undefined && f.accumulate >= 0) {
        // Frame k of the mean weighs 1/(k+1); frame 0 starts it.
        const into = acc[1 - accAt]!;
        prev.value = acc[accAt]!.texture;
        uWeight.value = 1 / (Math.floor(f.accumulate) + 1);
        renderer.setRenderTarget(into);
        accQuad.render(renderer);
        accAt = 1 - accAt;
        src.value = into.texture;
      } else src.value = sceneRT.texture;
      renderer.setRenderTarget(was);
      uExposure.value = f.exposure;
      pipe.render();
    },
    dispose() {
      for (const k of slots) k.blur.dispose();
      pipe.dispose();
      sceneRT.dispose();
      for (const a of acc) a.dispose();
      accMat.dispose();
    },
  };
}
