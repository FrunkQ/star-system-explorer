| Where | Instead of the physics | Why | Real physics to replace it | Status |
|---|---|---|---|---|
| `renderer/src/gpu-light-path.ts` camera | The camera is the kernel's far viewer: each pixel's ray is read from infinity, the picture drawn as from 2,000 gravitational radii (the field of view sets the zoom) | Matches the kernel's traced points exactly; a close camera is XO6 | Start each ray at the camera's radius (XO6) | planned shortcut, XO4 |
