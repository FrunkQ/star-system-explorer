| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|
| `renderer/src/hole-sky.ts` stars | Each star is spread over the 3 × 3 texels round it (a Gaussian keeping its light) in a 1,024-texel patch round the hole and a 1,024 by 512 whole sky; lensing then carries the radiance through the bent ray | Stars squeezed near the shadow still land on a pixel; the light total is kept (tested to 2%) | Stars as points traced per pixel at a higher tier | presentation, XO4 |
