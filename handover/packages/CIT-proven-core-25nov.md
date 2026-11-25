# Citation corrections in the proven core's files (25 Nov 2026)

These come from Alex's Consensus runs, as scored by the citation thread (`sse4/citations/corrections-02-05.md` and `corrections-06-09.md` in the project files). Each item below changes only wording or a test bound. No physics changed.

| Paper | Where | Change |
|---|---|---|
| Kesler and Wilkinson 2008 | `sse-atoms/src/matter.rs` KW08, `formulas/composition.json`, `packs/base-surface`, sol_tests | 1.7e11 t is porphyry copper at all crustal depths. The 3.3 km count depth is now named as SSE4's choice, and the ore share is calibrated to the total within it. |
| Ojakangas and Stevenson 1989 | `sse-atoms/src/tests.rs` `ice_shell_conduction` | The paper's bound is 25 km, so the test asks for less than 25 km (Europa-like, 13 km). |
| Frank et al. 2014 | `flows/planet-heat.json` Mars case | Mars' volcanism ending 0.5 to 1 Gyr ago is an observed date whose source is still to cite. Frank et al. is credited for the heating model only. |
| McDonough and Sun 1995 | `flows/planet-heat.json` Earth case | The roughly 20 TW radiogenic total is labelled "from their abundances, a source for the total still to cite". |
| Blair et al. 2017 | `sse-surface/src/tubes.rs`, tubes tests | "A roof spans wider as gravity falls" is now marked as SSE4's 1/g inference from the paper. |
| Greeley 1971 | `sse-surface/src/tubes.rs` | The citation now reads The Moon 3, 289 (now Earth, Moon, and Planets), doi:10.1007/BF00561842, replacing Modern Geology 2, 207. |
| Park et al. 2016 | sol_tests `small_worlds_barely_separate` | Ceres is partly differentiated into rock and volatiles. The paper says nothing of metal sinking. |
| Konôpková et al. 2016 | `formulas/dynamo.json`, `sse-atoms/src/test/magnetic.rs` | About 30 W/m/K was measured on solid iron, not on the liquid alloy. |
| Zahnle et al. 2003 | `sse-surface/src/impacts.rs` | Europa's surface is around 50 Myr old. Schenk et al. 2004's figure of about 60 Myr is given as theirs and marked not checked. |
| Takai et al. 2008 | `sse-life/src/traits.rs`, `packs/base-life` | 122 °C holds under 20 MPa. The lower limit near 253 K is credited to Clarke et al. 2013. |

The golden was re-blessed together with SW1 in the same lane slot: the base-physics pack carries these texts.
