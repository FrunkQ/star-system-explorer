# Trailer capture rig

Films scripted shots of the GM shell as video clips for the SSE4 trailers. Every shot is a small script, so any
shot can be re-filmed after the interface changes: fix the script (or `tests/ui/nav.mjs`, which it shares with the
browser tests) and run it again.

```
npm run build                                   # dist/, the same build main deploys to v4.starsystemx.com
node scripts/trailer/capture.mjs --out trailer-out            # every shot
node scripts/trailer/capture.mjs --only 05-look-ahead,11-galaxy
node scripts/trailer/capture.mjs --base https://v4.starsystemx.com/   # film the live site instead
node scripts/trailer/capture.mjs --gpu --headed                       # on a machine with a real GPU (WebGPU shots)
```

Options: `--fps 30` (output rate), `--scale 1` (2 films at 2x pixels for a 4K master; slower).

Each shot writes `<id>.mp4` (H.264, constant rate), `<id>.png` (the last frame) and `<id>.json` (title, feature,
which trailer it serves, commit or site filmed, time, real frame rate, pass or fail, page errors).

How it works: a fresh browser context per shot, so every shot starts from the campaign the site opens with
(Kestrel, fixture data) and fixed seeds; Chromium's screencast gives frames at their real paint times, and
ffmpeg turns them into constant-rate video. Headless browsers draw no pointer, so `director.mjs` paints a soft
pointer and eases it between targets, with a press ripple on clicks. A shot that cannot find what it films fails
loudly: that is how a shot learns it needs a re-shoot.

Limits, honestly:
- No GPU in the cloud sandbox. WebGPU views (Body globe, System 3D) film blank there, so those shots fall back to
  the canvas tier; the hero takes need `--gpu` on a real GPU.
- Real-time capture: the frame rate is what the browser painted (about 15 to 40 fps in the sandbox). Fine for UI;
  heavy scenes want a faster machine.

The shot list, storyboards and filmed clips live in the project folder `sse4/trailer/` (not in the repo).

## Adding a shot

Add `shots/NN-name.mjs`:

```js
export default {
  id: "NN-name", title: "What the viewer sees", feature: "Feature (package ids)", trailers: ["features", "intro"],
  path: "",                 // optional, e.g. "players.html"
  async setup(page, d, nav) {},   // off camera
  async run(page, d, nav) {       // on camera: d.glide, d.click, d.drag, d.type, d.wheel, d.hold
  },
};
```

Not part of CI and not part of `scripts/check.sh`.
