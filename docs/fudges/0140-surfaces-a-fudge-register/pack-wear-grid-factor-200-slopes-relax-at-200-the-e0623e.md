| Where | Instead of the physics | Why (as the code or log says) | Real physics to replace it | Status |
|---|---|---|---|---|
| pack wear `grid_factor` 200 | slopes relax at 200× the ground's own creep rate (block `sse.surface.creep_diffusivity`): the unresolved hillslopes, slides and infill of a cell kilometres across | "set so wet ground under 1 bar gives the 10 m²/yr the wear was calibrated with"; airless ground relaxes about 10,000× slower, so small bodies keep their relief | a scale-dependent effective diffusivity derived from relief spectra, or a sub-grid hillslope model | documented |
