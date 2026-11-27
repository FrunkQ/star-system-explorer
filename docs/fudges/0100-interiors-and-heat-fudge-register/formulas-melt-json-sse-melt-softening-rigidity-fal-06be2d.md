| Where | Instead of the physics | Why (as the code/log says) | Real physics to replace it | Status |
|---|---|---|---|---|
| formulas/melt.json `sse.melt.softening`: rigidity falls linearly to zero at lock-up, floored at `rigidity_floor` = 0.05 (HF4b.0) | A partly molten rock's rigidity is taken as μ₀ (1 − φ/φ_c) | The simplest law going to zero where the grains stop touching; the measured form was not to hand (paper search out until 1 Dec 2026) | Bierson and Nimmo 2016's measured rigidity law, once read at source | Stand-in (HF4b.0; nothing reads it until HF4b.2) |
