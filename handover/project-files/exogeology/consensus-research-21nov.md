# Planetary geology rules for a procedural planet generator

Nov 21, 2026 · @Alex Clark

## How to read this

Every number below carries one of three statuses, so you can decide what to hard-code and what to check first.

- **READ** means I retrieved the abstract plus a relevant section or the full text in this session, through Consensus full-text excerpts or the open-access copy. **FOUND** means abstract or title only.
- **(derived)** marks a number I computed from a cited relation, such as evaluating a polynomial or a flow law. The script inputs are stated next to it.
- **(recalled)** marks a value I know from the literature but did not re-check against the paper's text in this session. Verify these before relying on them.

Units are SI unless stated; 1 µbar = 0.1 Pa. Each topic lists its sources by author and year; full citations and DOIs are in the source table at the end. A DOI appears only where I confirmed it against a Crossref, OpenAlex, Semantic Scholar or arXiv record, because the Consensus results on this plan do not return DOIs.

## 1. Volatile ice cycles (N2, CH4, CO, CO2)

A volatile-covered surface sits at its frost point: surface pressure equals the vapour pressure at the ice temperature, and a global energy balance sets that temperature. Pressure therefore responds exponentially to insolation.

### Vapour pressure

Fray and Schmitt (2009) fit each ice with ln P\[bar\] = A0 + A1/T + A2/T^2 + ... (T in K). I read the coefficients from the reproduction in Lisse et al. (2021, Table 2), not from the paywalled original.

| Ice (phase) | Valid T (K) | A0 | A1 | A2 | A3 | A4 | A5 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| N2 (alpha) | below 35.61 | 12.40 | -807.4 | -3926 | 6.297e4 | -4.633e5 | 1.325e6 |
| N2 (beta) | 35.61 to 63.14 | 8.514 | -458.4 | -1.987e4 | 4.800e5 | -4.524e6 | 0 |
| CO (alpha) | below 61.55 | 10.43 | -721.3 | -1.074e4 | 2.341e5 | -2.392e6 | 9.478e6 |
| CO (beta) | 61.55 to 68.1 | 10.25 | -748.2 | -5843 | 3.939e4 | 0 | 0 |
| CH4 | below 90.69 | 10.51 | -1110 | -4341 | 1.035e5 | -7.910e5 | 0 |
| CO2 | 40 to 194.7 | 14.76 | -2571 | -7.781e4 | 4.325e6 | -1.207e8 | 1.350e9 |
| CO2 | 194.7 to 216.58 | 18.61 | -4154 | 1.041e5 | 0 | 0 | 0 |

The phase boundaries come from a second reproduction (an Astronomy & Astrophysics 2023 appendix table) that also contains three misprints in the N2 rows, so do not copy from it. The pressure unit (bar) is confirmed numerically: the fits return 0.1215, 0.1538, 0.1187 and 5.20 bar at the N2, CO, CH4 and CO2 triple points (derived).

A two-constant form is cheaper in a browser. These are my least-squares fits to the polynomials (derived), ln P\[Pa\] = a - b/T:

| Ice | T range (K) | a | b (K) | Max error | Latent heat (J/kg) |
| --- | --- | --- | --- | --- | --- |
| N2 (alpha) | 22 to 35.6 | 25.116 | 920.0 | 1% | 2.7e5 at 30 K |
| N2 (beta) | 36 to 63 | 22.505 | 824.8 | 4% | 2.5e5 at 37 K |
| CO | 25 to 61.5 | 23.698 | 948.4 | 5% | 2.8e5 at 37 K |
| CH4 | 30 to 90 | 22.427 | 1181.9 | 3% | 6.1e5 at 40 K |
| CO2 | 100 to 194 | 27.919 | 3181.7 | 7% | 6.0e5 at 148 K |

Spot values from the polynomial (derived), in Pa:

| T (K) | N2 | CO | CH4 |
| --- | --- | --- | --- |
| 30 | 3.9e-3 | 3.6e-4 | 4.4e-8 |
| 37 | 1.2 | 0.15 | 7.3e-5 |
| 40 | 6.6 | 1.0 | 8.0e-4 |
| 45 | 66 | 14 | 0.021 |
| 50 | 415 | 115 | 0.30 |
| 60 | 6250 | 2560 | 15 |

CO2 gives 4.1 Pa at 120 K, 186 Pa at 140 K, 633 Pa at 148 K and 3130 Pa at 160 K (derived). Volatility runs N2 > CO > CH4, far above CO2, with N2 about 10^4 times CH4 near 37 to 40 K.

### Energy balance and pressure

```latex
\epsilon \sigma T^4 = (1-A)\,S\cos i + L\,\dot{m} + F_{cond}
```

Here S is the solar flux at the body, i the incidence angle, m-dot the condensation rate (negative for sublimation) and F\_cond the heat conducted from below. The Pluto and Triton volatile transport models treat the atmosphere as transparent and keep only insolation, thermal emission, latent heat and subsurface conduction (Bertrand et al. 2018, model section).

- **Global buffer.** When the atmosphere can carry latent heat round the body, all N2 ice shares one temperature and p = p\_vap(T\_ice). Lewis et al. (2020) pair Pluto's 11.5 µbar with 36.93 ± 0.10 K; the beta-N2 fit above returns 1.15 Pa at that temperature (derived).
- **Local rate.** Each ice patch then condenses or sublimes at the rate that closes its own balance at T\_ice.
- **Sensitivity (derived).** d ln p / d ln T = b/T, about 22 for N2 at 37 K. A 4% change in absorbed flux moves T by 1% and pressure by about 25%.
- **Calibrated parameters.** N2 ice Bond albedo 0.72 to 0.73, emissivity 0.8 and subsurface thermal inertia 700 to 900 J s^-1/2 m^-2 K^-1 reproduce Pluto's occultation record (Meza et al. 2019). Lewis et al. (2020) instead measure bolometric albedo 0.83 ± 0.11 and infer emissivity 0.47 to 0.72.

### Where the ice ends up

- **Obliquity decides poles or equator.** Running one model with each body's orbit, Bertrand et al. (2024) find obliquity is the main driver: Pluto's high obliquity sends N2 to low latitudes, and Triton's poles act as cold traps. The usual threshold is about 54 degrees of obliquity, above which the poles receive more annual sunlight than the equator (recalled; standard insolation geometry, not from the fetched text).
- **Pluto's bands.** Over one obliquity cycle of about 2.8 Myr, latitudes 25°S to 30°N of Sputnik Planitia gain N2 and 30°N to 50°N lose it. N2 is not stable at the poles; equatorial depressions keep thick deposits for tens of Myr (Bertrand et al. 2018).
- **Topography.** N2 accumulates in the deepest low-latitude basin, here 4 km deep and 1000 km wide (Bertrand and Forget 2016). The frost point rises about 0.09 K per km of depth on Pluto, for a 17.7 km scale height (derived).
- **Species.** CO follows N2 and stays trapped in N2 ice. CH4 forms its own CH4-rich deposits holding 3 to 5% N2, plus seasonal frosts at mid and high latitudes (Bertrand et al. 2018; Bertrand and Forget 2016).
- **Thermal inertia.** High subsurface inertia stores summer heat and keeps Pluto's winter pole above the N2 frost point, which limits polar condensation (Meza et al. 2019).
- **Triton.** Best fits need an N2 reservoir thicker than 200 m, bedrock albedo 0.6 to 0.7 and bedrock thermal inertia above 500 SI. A permanent northern cap survives if internal heat flow stays below 3 mW m^-2 and vanishes in some runs at 30 mW m^-2 (Bertrand et al. 2022).

### Sizes, rates and lifetimes

- **Pluto pressure.** It rose by a factor of 2.5 to 3 between 1988 and 2015, to 10 to 11 µbar (Meza et al. 2019). Over millions of years it stays between milli-pascals and pascals, so liquid N2 never flows on the surface (Bertrand et al. 2018).
- **Pluto ice budget.** About 1 km of N2 ice left the northern edge of Sputnik Planitia in 2 Myr, or 0.5 mm per Earth year (derived). Daily sublimation reaches 4.2e-7 kg m^-2 s^-1 in the north (Morison et al. 2021, quoting the transport model).
- **Pluto solution families.** Young (2013) finds three: permanent northern volatiles with high thermal inertia, hemispheric exchange with a pressure plateau, and exchange with early atmospheric collapse.
- **Triton pressure.** 14.1 ± 0.4 µbar in 2017 and 14.54 ± 0.47 µbar in 2022; models peak at 15 to 19 µbar near 2000 to 2010. The extreme southern solstice recurs about every 650 years with the subsolar point near 50°S (Sicardy et al. 2024).
- **Atmospheric mixing on Triton.** N2 and CO redistribute within about 1 s of model time; CH4 takes 1e7 s (Bertrand et al. 2022).
- **Mars.** The frost point is 148 K at 633 Pa (derived). InSight's annual mean is 717.7 Pa with 0.03e15 kg of atmosphere per Pa, a mean atmospheric mass of 23.5e15 kg, and 27e15 kg of CO2 in atmosphere plus seasonal frost (Corchete 2025). Frost at 55° to 70° latitude drives most of the pressure cycle (Wood and Paige 1992). Local topography amplifies the cycle by up to 2.15 in Hellas (Mendoza et al. 2020).

### Where the literature disagrees

- **Triton's 1990s surge.** Occultations in 1995 to 1997 suggested a strong pressure rise; transport models give only a moderate one, and the two data sets appear mutually inconsistent with the model (Marques Oliveira et al. 2022). Models also predicted a decline from 2005 that the 2022 occultation did not show (Sicardy et al. 2024).
- **N2 emissivity.** 0.8 from model fitting against 0.47 to 0.72 from the New Horizons energy balance. Low emissivity implies a long constant-pressure period as Pluto recedes (Lewis et al. 2020).
- **Atmospheric collapse.** Young (2013) could not rule out early collapse; Bertrand et al. (2018) find the pressure never drops below milli-pascals.
- **Fit accuracy.** The N2 fit is 3% low at the triple point (12.15 kPa against 12.52 kPa measured; derived, measured value recalled).

**Sources.** Fray and Schmitt 2009 (FOUND; coefficients READ via Lisse et al. 2021); Bertrand and Forget 2016 (FOUND); Bertrand et al. 2018 (READ); Young 2013 (FOUND); Meza et al. 2019 (READ); Lewis et al. 2020 (FOUND); Bertrand et al. 2022 Triton (READ); Bertrand et al. 2024 (READ); Sicardy et al. 2024 (READ); Marques Oliveira et al. 2022 (READ); Wood and Paige 1992 (FOUND); Corchete 2025 (READ); Mendoza et al. 2020 (READ).

## 2. Solid-state convection in N2 ice (Sputnik Planitia)

An N2 ice layer thicker than roughly 0.5 to 1 km convects under a few mW m^-2 of heat flow. Its cells are several times wider than the layer is deep, and the surface is renewed in about 0.5 to 1 Myr.

### Rule

For Newtonian creep the Rayleigh number at the basal viscosity is:

```latex
Ra = \frac{\rho\, g\, \alpha\, \Delta T\, D^{3}}{\kappa\, \eta_b}
```

McKinnon et al. (2016) use the power-law generalisation, Ra = rho g alpha A^(1/n) dT D^((2+n)/n) / (kappa^(1/n) exp(E\*/(n R T\_b))). The PDF text of that expression was partly garbled in extraction, so check it against the paper before coding it.

- **Critical Ra, rigid base and free top, small viscosity contrast.** 1101 for n = 1 and about 200 for n = 2.2. With rigid top and bottom: 2038 and 310.
- **With a viscosity contrast.** Ra\_cr = 1100 exp(theta/1.78) for n = 1 and 200 exp(theta/3.87) for n = 2.2, where theta is the log viscosity contrast across the layer (my reading of the symbol).
- **Convective regime.** Sluggish lid, where the surface moves but more slowly than the interior, holds while the viscosity contrast stays below about 1e4 (n = 1) or 1e6 (n = 2.2). Above that the lid stagnates and the surface is not renewed.

### Constants used for N2 ice on Pluto

| Quantity | Value | Source |
| --- | --- | --- |
| Thermal expansivity | 2e-3 K^-1 | McKinnon et al. 2016 |
| Thermal conductivity | 0.2 W m^-1 K^-1 | McKinnon et al. 2016 |
| Thermal diffusivity | 1.33e-7 m^2 s^-1 | McKinnon et al. 2016 |
| Density | 1000 - 2.14 (T - 36 K) kg m^-3 | McKinnon et al. 2016 |
| Gravity | 0.617 m s^-2 | McKinnon et al. 2016 |
| Surface and maximum basal temperature | 36 K and 63 K (N2 melting) | McKinnon et al. 2016 |
| Heat flow and conductive gradient | about 3 mW m^-2, 15 K km^-1 | McKinnon et al. 2016 |
| Power-law creep | n = 2.2 ± 0.2, E\* = 3.5 kJ mol^-1, A = 3.73e-12 Pa^-2.2 s^-1 | Yamashita et al. 2010 via McKinnon |
| Diffusion creep | n = 1, E\* = 8.6 kJ mol^-1, A = 1.52e-7 (d / 1 mm)^-2 (T / 50 K)^-1 Pa^-1 s^-1 | McKinnon et al. 2016 |
| Measured viscosity | about 2e8 Pa s at 56 K and 100 kPa | Moore et al. 2016 |
| Viscosity contrast across the layer | about 150 (power law) up to 2e5 (diffusion creep) | McKinnon et al. 2016 |

### Conditions

- **Onset thickness.** The abstract gives a threshold of about 1 km at present-day heat flow; the main text gives under 1 km, and about 500 m for chondritic heat flow (McKinnon et al. 2016). Umurhan et al. (2017) put the limit of laminar, non-convecting flow at 300 to 1000 m, and at 1.2 km for 2 to 3 mm grains.
- **Which worlds.** A low-viscosity volatile ice (N2, CO, CO2, possibly CH4) must be abundant, on a body massive enough to keep it. Morison et al. (2021) name Triton, Eris and Makemake as candidates.

### Sizes, rates and lifetimes

- **Cell width against depth.** At onset the wavelength is 2.34 D. Isoviscous cells have aspect ratio near 1, which would need a 10 to 20 km layer. Sluggish-lid cells have wavelength 4 to 6 D with a rigid base and up to 12 D with a free-slip base (McKinnon et al. 2016).
- **Observed.** Cells are 10 to 40 km across, giving D of 3 to 6 km (McKinnon et al. 2016). Trowbridge et al. (2016) read 20 to 30 km polygons as about 10 km of ice.
- **Speed and age.** A few cm per year and 500,000 years of renewal (McKinnon et al. 2016); 1.5 cm per year and about 1 Myr (Trowbridge et al. 2016). Both sit under the 10 Myr crater-retention limit.
- **Relief.** Cell centres stand tens of metres above the edges (Trowbridge et al. 2016); 100 to 150 m in the larger cells (Jain et al. 2026). Heat flow over upwellings is nearly twice the mean and near zero over downwellings.
- **Dimensional scales.** Velocity scales as kappa/D, heat flow as k dT/D and stress as eta\_b kappa/D^2 (McKinnon et al. 2016).

### Where the literature disagrees

- **What drives it.** McKinnon and Trowbridge heat the layer from below. Morison et al. (2021) drive it by sublimation cooling at the top, with a basal flux near 0.3 mW m^-2, or the usual 2 to 3 mW m^-2 if the viscosity contrast is about 3000 instead of 100. A third proposal uses internal heating (Vilella and Deschamps 2017, as cited by Jain et al. 2026).
- **Topography test.** Morison et al. argue that bottom heating raises the polygon edges, while Sputnik Planitia has flat polygons bounded by troughs. Jain et al. (2026) answer that bottom heating with a realistic power-law rheology does give domes over upwellings, and that the small isolated cells at the margins are localised plumes with about 40 m of relief.
- **Layer thickness.** 3 to 6 km against about 10 km, depending on the assumed cell aspect ratio.

**Sources.** McKinnon et al. 2016 (READ, full text); Trowbridge et al. 2016 (FOUND); Morison et al. 2021 (READ); Jain et al. 2026 (READ); Moore et al. 2016 (READ); Umurhan et al. 2017 (READ, full text).

## 3. Glacial flow of N2 and CH4 ice

N2 ice flows as a Glen-type fluid with exponent near 2.1 to 2.2 and a very small activation energy. A 100 to 200 m glacier on a 1° to 10° slope moves metres to kilometres per year, then thins and stalls.

### Flow law

Umurhan et al. (2017) fit the Yamashita et al. (2010) data for annealed polycrystalline N2 as strain rate = A(T) sigma^n:

| Parameter | Value |
| --- | --- |
| A(T) | A45 exp(Ta/45 - Ta/T), with A45 = 0.005 s^-1 MPa^-n |
| Activation temperature Ta | 422 K (Ea = 3.5 kJ mol^-1) |
| Exponent n(T) | 2.1 + 0.0155 (T - 45), so 2.1 at 45 K and 2.27 at 56 K |
| Viscosity | 1 / (n A sigma^(n-1)); 1.6e10 Pa s at 10 kPa and 45 K |

From that law (derived): 4.9e10 Pa s at 38.5 K and 10 kPa; 1.2e9 Pa s at 45 K and 100 kPa; 2.6e8 Pa s at 56 K and 100 kPa, which matches the measured 2e8 Pa s in topic 2.

### Mass flux for a thin glacier

```latex
q_{glen} = \frac{A\,(\rho g H)^{n}\,H^{2}}{n+2}\,S^{\,n-1}, \qquad q_{dry} = g_Q \exp\!\left[\frac{H/H_a}{1+H/H_{\Delta T}}\right] q_{glen}
```

The flux vector is q times the surface gradient, and S is the magnitude of that gradient. The exponential corrects for ice warming with depth: H\_a = Ts^2 / (Ta T0z) = 175 m and H\_dT = Ts / T0z = 1925 m, for surface temperature Ts = 38.5 K and gradient T0z = 20 K km^-1 (4 mW m^-2 through 0.2 W m^-1 K^-1). The factor g\_Q is 0.5. A basally wet glacier adds a sliding term, H u\_sl0 / S for H above a melt thickness H\_m, with H\_m = 100 m and u\_sl0 = 0.2 km per year in the paper's runs.

Surface speed of an isothermal slab, u = 2A/(n+1) (rho g sin(slope))^n H^(n+1), evaluated with the law above at 38.5 K and g = 0.62 m s^-2 (derived; the slab formula is the standard shallow-ice result, not taken from the paper):

| Thickness (m) | Slope (degrees) | Surface speed (m per Earth year) |
| --- | --- | --- |
| 50 | 1 | 0.3 |
| 100 | 1 | 2.5 |
| 200 | 1 | 20 |
| 500 | 0.5 | 80 |
| 200 | 10 | 2000 |

### Conditions

- **Laminar regime.** The glacier formulation holds for N2 layers thinner than 300 to 1000 m; thicker layers convect (topic 2).
- **Basal melting.** Base temperature is Ts + H T0z. It reaches the 63.1 K triple point at about 1.2 km for Ts = 38.5 K, and at a few hundred metres if the surface is at 55 to 57 K. The paper's wet-base thresholds vary by section, from 100 to 400 m.
- **Strength.** N2 fails brittly at up to 9 MPa and CH4 at 10 MPa, in tests at 5 to 56 K and 5 to 77 K and strain rates of 1e-4 to 1e-2 s^-1. Neither supports relief of several kilometres, so water ice must underlie such terrain (Yamashita et al. 2010).
- **CH4 and CO2.** Solid CH4 creeps with exponent 1.9 ± 0.2 and an effective viscosity of 1e11 Pa s at 0.1 MPa and 77 K. Solid CO2 deforms brittly at 77 K (Yamashita et al. 1997).

### Sizes, rates and lifetimes

- **Channel drainage.** A 50 km channel at 10° slope holding 200 m of ice has an e-folding time of about 50 years. It thins to about 75 m, the next e-fold takes about 5000 years, and ice drains below 50 m within 100 to 500 years (Umurhan et al. 2017).
- **Recent flow.** Wavy dark bands on the northern shore of Sputnik Planitia may record a shoreward flow event within the last few hundred years. Dark lobes on the eastern shore fit a basally wet glacier descending from the pitted uplands (Umurhan et al. 2017).
- **Ice-sheet balance.** The 1 km lost by sublimation at the northern edge in 2 Myr is replaced by viscous flow, and the sheet is now at its minimum extent north and south (Bertrand et al. 2018).
- **Ancient liquid.** Channel-like landforms are attributed to liquid N2 at the base of thick ancient glaciers, because surface pressure never allowed surface liquid (Bertrand et al. 2018). Stern et al. (2026) propose that basal melt still reaches the surface in northern Sputnik Planitia.

### Where the literature disagrees

- **Rheology.** Viscosity estimates span 1e8 to 1e12 Pa s at 1e5 Pa (Bertrand et al. 2018). The experiments did not report grain size, and the alpha phase and N2-CH4-CO mixtures are unmeasured.
- **Activation temperature.** Umurhan et al. use 422 K in the main text and 1030 K in their own appendix Rayleigh-number formula.

**Sources.** Umurhan et al. 2017 (READ, full text); Yamashita et al. 2010 (FOUND); Yamashita et al. 1997 (FOUND); Bertrand et al. 2018 (READ); Stern et al. 2026 (READ).

## 4. Penitentes and bladed terrain

Penitentes need ice held close to its vapour-pressure curve under strong sunlight, with enough gas above it to sustain a vapour gradient. Spacing follows the thickness of that vapour layer, from centimetres on Earth to kilometres on Pluto; on airless Europa the mechanism is disputed.

### Rule

- **Instability.** Troughs trap light and ablate faster than peaks. Self-illumination alone is scale-free; vapour diffusion and heat conduction select the wavelength, which is set by aerodynamic mixing of vapour above the surface (Claudin et al. 2015).
- **Classic conditions.** Low humidity, strong insolation and low temperature. Hand et al. (2019) combine these into one test: the ice must sit near its vapour-pressure curve. Too far above it, liquid processes dominate; too far below, there is too little vapour to hold a temperature difference across the surface.
- **Laboratory check.** Millimetre-scale penitentes form between 50 and 2000 Pa at -15 to -2 °C, only close to the equilibrium vapour pressure. Away from it the ice sublimates into a smooth crater. Spacing agrees with the Claudin model (Berisford et al. 2024).
- **Shape.** Hobley et al. (2018) take a depth to spacing ratio of 2:1 from experiments under steady single-angle light; field values are lower.
- **Terrestrial mechanism.** Warren (2022) holds that Andean penitentes need melting troughs beside dry sublimating peaks, exploiting the 8.5-fold latent heat difference, and that they lower albedo by about 0.3.

### Cases

| Body | Ice | Spacing | Relief | Where | Growth |
| --- | --- | --- | --- | --- | --- |
| Earth | H2O snow | bowls tens of cm across | spires up to several m | high subtropical mountains; blades east-west, tilted to the noon sun | not given in the sources read |
| Pluto, Tartarus Dorsa | CH4-rich | 3000 to 5000 m | about 500 m | 220° to 250°E, 0° to 20°N | 1 cm per orbit, only at high atmospheric pressure; tens of Myr |
| Europa (predicted) | H2O | 7.5 m maximum; tens of cm minimum | up to 15 m | latitudes below 23° | over the mean surface age; millions of years |
| Laboratory | H2O | millimetres | millimetres | 50 to 2000 Pa | within an experiment |

- **Pluto detail.** Modelled blades reproduce the three observed ridge orientations (north-south, east-west, northeast-southwest) and start from relief of no more than a few tens of metres (Moores et al. 2017). An earlier description gives ridges several hundred metres high and 5 to 10 km apart, with V-shaped valleys and Y-junctions, and notes that N2 ice is too soft to hold them (Moore et al. 2016 abstract).
- **Europa condition.** Blades are predicted where sublimation outpaces erosion by space weathering, which holds in the equatorial belt (Hobley et al. 2018).

### Where the literature disagrees

- **Europa: against.** Water molecules there have a mean free path near 1e5 m, so the surface is in free-molecular flow with no diffusive boundary layer. Hand et al. (2019) argue that sublimation is then uniform, and that Hobley's 2:1 ratio comes from the Earth regime while the depth comes from vacuum kinetics.
- **Europa: reply.** Hobley et al. (2019) concede the name and call the features morphological analogues. They keep the prediction on radiative grounds: sunlight focused into troughs, helped by dark lag, as on Callisto and Pluto. They restate 7.5 m as a maximum spacing.
- **Europa: albedo.** Warren (2022) adds that Europa's visible albedo near 0.7 argues against such roughness.
- **Pluto: origin of the deposit.** Bertrand et al. (2020) show CH4 condensing on equatorial summits from methane-enriched air a few kilometres up, which may supply the bladed deposits; Moores et al. treat the blades as erosional.

**Sources.** Moores et al. 2017 (FOUND); Hobley et al. 2018 (FOUND); Hand et al. 2019 (READ, body text truncated); Hobley et al. 2019 reply (READ); Claudin et al. 2015 (FOUND); Berisford et al. 2024 (FOUND); Warren 2022 (FOUND); Moore et al. 2016 conference abstract (FOUND); Bertrand et al. 2020 (FOUND).

## 5. Sublimation pits and degradation

Sublimation cuts pits whose walls retreat at a rate set by the ice's volatility at the local temperature: about 0.4 mm per year for N2 on Pluto and 1 to 3 m per year for CO2 on Mars. Where the ice holds a refractory fraction, the residue armours the surface and leaves knobs, pinnacles or hollows.

### Cases

| Feature | Volatile | Size | Rate | Age or lifetime |
| --- | --- | --- | --- | --- |
| Pluto, Sputnik Planitia pits | N2 | radii up to several hundred m; length to width 2 to 4 | radius grows 3.6e-4 m per year (+2.1e-4, -0.6e-4) | cell-edge surfaces 4.2e5 to 8.9e5 years |
| Mars, residual south cap ("Swiss cheese") | CO2 | scarps 1 to 12 m high in a 1 to 10 m slab | walls retreat 1 to 3 m per year | oldest units over 125 Mars years; some pits predate 1765 |
| Callisto, pinnacle terrain | CO2 and H2O | pinnacle ice up to 64 m thick | modelled over 4.5 Gyr | ancient, still degrading |
| Mercury, hollows | sulfur or carbon (disputed) | up to 19,110 mapped | growth over 1000 times earlier estimates | maximum model age 103 kyr (+200, -96) |
| Comet 67P, pits | cometary ices | heterogeneity in the top few hundred metres | walls retreat slowly by sublimation | fresh surfaces are ragged and pitted, evolved ones smooth |

### Rules and conditions

- **Pluto pits.** Pits are small near convection-cell centres and larger toward the edges; radius grows by 2.1e-3 to 5.9e-3 m per metre of distance, a lower bound set by image resolution. Dividing by the growth rate gives surface speeds of 1.5 to 17.9 cm per year (Buhler and Ingersoll 2018). Their long axes run roughly north-south, and the lost mass recondenses elsewhere on Pluto (Stern et al. 2021). Pit formation follows the annual sublimation cycle (Bertrand et al. 2018).
- **Mars CO2 slab.** Steep scarps erode in summer because they absorb more sunlight; winter CO2 deposition smooths them into ramps. The interplay yields quasi-circular pits, heart-shaped pits, linear troughs or moats (Buhler et al. 2017). Backwasting starts from scarp-parallel fractures that form about once per Mars year in the thicker unit (Thomas et al. 2009).
- **Mars triggers.** Moat formation times cluster after the planet-encircling dust events of Mars years 9, 12 and 28 (Thomas et al. 2023). The whole residual cap is under 3% of the atmosphere's mass (Thomas et al. 2009).
- **Callisto.** Ice sublimates from an ice-cemented crust, dark residue mass-wastes downslope, and the vapour refreezes on cold-trap peaks and crater rims. A crust with 10% CO2 reproduces the observed degradation; 33% H2O reproduces the pinnacles. Collapse limits pinnacle height (White et al. 2016). The degradation implies an ice more volatile than H2O (Moore et al. 1999).
- **Mercury.** Hollows are larger toward the equator and the hot-pole longitudes and favour equator-facing slopes, pointing to insolation and micrometeoroid bombardment as drivers (Bickel et al. 2025). Only small hollows occur in the coldest terrain. Sharp, bright hollows outnumber softened ones: 1545 Stage 1, 1111 Stage 2 and 10 Stage 3 (Deutsch et al. 2025). At least 1266 km^3 of volatiles has been lost (Wang et al. 2020).
- **Comets.** Pits on 67P form by sinkhole collapse, possibly with outbursts, then widen by sublimation of the walls (Vincent et al. 2015).
- **Ablation waves.** Carpy et al. (2023) give three scaling laws linking the wavelength, migration speed and growth time of sublimation waves to pressure, temperature, friction velocity and ablation rate, with charts for N2 and CO2 atmospheres. I did not extract the coefficients.

### Where the literature disagrees

- **N2 ice strength on Pluto.** Pits several hundred metres across survive only if surface viscosity is 1e16 to 1e17 Pa s (Buhler and Ingersoll 2018). Laboratory N2 flow laws give 1e8 to 1e12 Pa s (topic 3).
- **Mercury's volatile.** Sulfur is favoured by a survey of 57 candidate phases, with a fumarole-cycling model (Phillips et al. 2021). Carbon is favoured on spectral grounds (Wang et al. 2020). Sulfides were the earlier candidate.
- **Hollow ages.** The 103 kyr maximum implies much faster growth than earlier estimates (Wang et al. 2020).

### Named papers I did not retrieve

Howard et al. (2017), Moore et al. (2017), Blewett et al. (2011) and Byrne and Ingersoll (2003) appear in the source table with confirmed DOIs, but Consensus did not return their abstracts in my searches. Two values I hold from them are unverified: Mercury hollows span tens of metres to several kilometres with depths of tens of metres (recalled), and comet 67P pits are tens to a few hundred metres across (recalled).

**Sources.** Buhler and Ingersoll 2018 (FOUND); Stern et al. 2021 (FOUND); Thomas et al. 2009 (FOUND); Thomas et al. 2023 (FOUND); Buhler et al. 2017 (FOUND); Betz et al. 2009 conference abstract (FOUND); White et al. 2016 (FOUND); Moore et al. 1999 (FOUND); Deutsch et al. 2025 (FOUND); Bickel et al. 2025 (FOUND); Wang et al. 2020 (FOUND); Phillips et al. 2021 (FOUND); Vincent et al. 2015 (FOUND); Carpy et al. 2023 (FOUND).

## 6. Solid-state greenhouse jets and geysers

Sunlight that passes through a sealed, translucent slab of volatile ice heats its base. A few kelvin of warming is enough: on Triton under 4 K drives 8 km plumes, and on Mars basal CO2 gas lifts and ruptures the seasonal slab every spring.

### Rule and energy threshold

- **Triton.** A rise of less than 4 K above the ambient 38 ± 3 K is more than adequate to lift plumes to 8 km (Soderblom et al. 1990). A 1 K rise roughly doubles the N2 vapour pressure (Duxbury and Brown 1997); the topic 1 fit gives a factor of 1.8 per kelvin at 38 K (derived).
- **Mars.** The seasonal cap anneals into an impermeable, translucent CO2 slab that sublimates from its base. Gas pressure builds, levitates the slab and ruptures it. High-velocity vents carry sand-sized grains to the surface as dark spots and fans, and the converging gas erodes radial channels (Kieffer et al. 2006).
- **Mars lift-off pressure (derived).** Gas must exceed the slab's weight, about 6 kPa per metre of slab for a CO2 ice density of 1600 kg m^-3 (density recalled) and g = 3.71 m s^-2.
- **Sediment transport.** Experiments show CO2 sublimation is an efficient mover of grains at present Mars pressure, and that the Shields parameter governs the branching pattern (Mc Keown et al. 2021).

### Where they occur

- **Mars.** Spiders occur only in the "cryptic" terrain, the transparent part of the southern seasonal cap, and only on the polar layered deposits, whose surface is weak and easily eroded (Piqueux et al. 2003). Spots and fans stay at CO2 ice temperature well into summer, so they are grains lying on ice, not bare ground (Kieffer et al. 2006).
- **Triton.** Plumes were seen on the southern cap near the subsolar latitude of the 1989 flyby. Dark fans across the cap are taken as deposits of earlier plumes (Hofgartner et al. 2022).

### Sizes, rates and lifetimes

| Quantity | Triton plumes (Soderblom et al. 1990) |
| --- | --- |
| Number active in 1989 | at least four reported; two well documented |
| Column height | about 8 km |
| Downwind cloud length | over 100 km |
| Column radius | several tens of metres to 1 km |
| Dark particle flux | up to 10 kg s^-1 (or twice that as N2 ice) |
| N2 gas flux | several hundred kg s^-1 or more |
| Duration | a year or more per eruption |
| Ice sublimed per eruption | about 0.1 km^3 |

### Where the literature disagrees

- **Triton's energy source.** Five hypotheses exist; three eruptive ones remain plausible. One is the solar-powered solid-state greenhouse, and two draw on internal heat. Hofgartner et al. (2022) argue that the plume and fan locations are not strong evidence for the solar model, and propose five remote-sensing tests.
- **Internal-heat variant.** Convection in a thick N2 cap can bring ice 1 K warmer than the surface close to it, and venting that ice into the 15 µbar atmosphere reaches 8 km (Duxbury and Brown 1997).
- **Mars spider troughs.** Kieffer's model erodes them by gas flow under the slab. Mc Keown et al. (2024) reproduce spider-like crack networks from CO2 sublimating inside the regolith, without surface scouring.

Kirk et al. (1990), the energy storage model the request names, has a confirmed DOI but was not returned by Consensus; nothing above depends on it.

**Sources.** Kieffer et al. 2006 (FOUND); Piqueux et al. 2003 (FOUND); Mc Keown et al. 2021 (FOUND); Mc Keown et al. 2024 (FOUND); Soderblom et al. 1990 (FOUND); Hofgartner et al. 2022 (FOUND); Duxbury and Brown 1997 (FOUND).

## 7. Cryovolcanism

Liquid water is denser than ice, so it reaches the surface only with help: ocean or reservoir overpressure from freezing, exsolving gas, or a low-density brine or slurry. The resulting edifices are kilometres tall and relax away in 0.1 to 1 Gyr where the crust is ice-rich.

### Rule

```latex
\Delta P_{erupt} \ge (\rho_w - \rho_i)\, g\, H
```

This is the hydrostatic condition for water to rise through a shell of thickness H (derived; standard, not taken from a fetched paper). Without overpressure, water in an open crack stops about 10% of the shell thickness below the surface. With a density difference of 80 kg m^-3 it gives 1.0 MPa for Europa (10 km shell), 0.18 MPa for Enceladus (20 km) and 5 MPa per 100 km of shell on Pluto (derived).

- **Freezing oceans.** As a shell thickens, the expansion of water to ice pressurises the ocean, balanced by compression of the water and elastic stretching of the shell. On Europa, freezing 1 to 10 km of ice breaks the shell in tension, yet the pressure is too low to erupt water. On small moons such as Enceladus the pressure can be large enough (Manga and Wang 2007).
- **Crack penetration.** With a 3 MPa tensile strength, cracks do not cross Europa's shell. On Enceladus they do when the mean shell is under 15 km and tidal dissipation varies by more than 15% over eccentricity cycles of about 100 Myr (Rudolph et al. 2022).
- **Shallow reservoirs.** A freezing brine pocket pressurises itself, but warm ice walls creep and relieve the pressure. On Europa the transition lies at 4 to 8 km depth depending on reservoir radius; shallow lens-shaped pockets are the best eruption sources (Lesage et al. 2022).
- **Brine delivery on Ceres.** Chloride or ammonia brines stay liquid to the surface if their fracture propagates at 1e-5 m s^-1 or faster (Quick et al. 2019).

### Edifices

| Feature | Height | Width | Other |
| --- | --- | --- | --- |
| Wright Mons, Pluto | 4 to 5 km | 150 km | volume 2.4e4 km^3; central depression 40 to 50 km wide and 4 km deep |
| Piccard Mons, Pluto | 7 km | 225 km | larger, U-shaped depression |
| Coleman Mons, Pluto | 1.5 km (1 to 2.5) | 25 km | basal yield strength 6e4 Pa |
| Pluto hummocks | a few hundred m to 1 km | 6 to 12 km most common | flank slopes 3° to 5° |
| Ahuna Mons, Ceres | 4 km | not given in the abstracts | over 40% ice; flows at 10 to 500 m per Myr |
| Cerealia Dome, Occator | not given | not given | lava kinematic viscosity 1e6 to 1e8 m^2 s^-1; relaxed in 2.5 to 273 days |

### Conditions

- **Pluto.** The constructs are mainly water ice, with CH4 only as a thin atmospheric coating; N2 ice cannot hold the relief. Total erupted volume exceeds 1e4 km^3 from several sources. There are no collapse terraces, flow directions or vents, and the depression is as deep as the edifice is tall, so it is neither a caldera nor a shield (Singer et al. 2022).
- **Pluto's heat problem.** Radiogenic heat flow is 5 mW m^-2 or less, tidal heating ended within 100 Myr of Charon's formation, and any ocean lies 100 to 200 km down. The authors suggest stored heat released late, possibly from beneath a clathrate layer (Singer et al. 2022).
- **Ceres.** Ahuna Mons sits above a mantle uplift read as a plume, and its shape fits a slurry of brine and solid particles (Ruesch et al. 2019). Occator overlies a 200 by 200 km negative gravity anomaly; a freezing reservoir there could feed brines to the surface for tens to hundreds of millions of years (Quick et al. 2019).
- **Occator faculae.** Brine effusion emplaced them in an impact-driven hydrothermal system. The central faculae drew on an impact melt chamber plus a deeper reservoir; the thinner, more ballistic Vinalia Faculae drew on the deep, gas-richer reservoir alone (Scully et al. 2020).

### Frequency and lifetimes

- **Ceres.** Only one sharp dome exists because older ones relax beyond recognition in 1e8 to 1e9 years (Sori et al. 2017). Ahuna Mons itself was extruded recently on crater-count ages (Ruesch et al. 2016).
- **Pluto.** The upper age limit is 1 to 2 Ga, and the terrain could be much younger; crater statistics are weak (Singer et al. 2022).

### Where the literature disagrees

- **Can overpressure erupt water on Enceladus?** Manga and Wang (2007) say yes. Rudolph et al. (2022) find ocean overpressure never extrudes water, and that decompression boiling sustains the eruptions.
- **What Ahuna Mons is made of.** A salt and water-ice cryomagma (Ruesch et al. 2016) or a mud-bearing slurry from a convecting mantle, unlike icy-satellite cryovolcanism (Ruesch et al. 2019).
- **Source of Occator's brines.** A deep cryomagma chamber (Quick et al. 2019) against a mainly impact-generated melt chamber (Scully et al. 2020).

Fagents (2003) is in the source table with a confirmed DOI, but Consensus did not return its abstract and I took nothing from it.

**Sources.** Manga and Wang 2007 (FOUND); Rudolph et al. 2022 (FOUND); Lesage et al. 2022 (FOUND); Singer et al. 2022 (READ, full text); Sori et al. 2017 (FOUND); Ruesch et al. 2016 (FOUND); Ruesch et al. 2019 (FOUND); Quick et al. 2019 (FOUND); Scully et al. 2020 (FOUND).

## 8. Ice-shell tectonics

Daily tides open and steer cracks in thin shells that float on oceans. A freezing ocean stretches the surface by about 1%. Buoyant instabilities form diapirs spaced a few layer-thicknesses apart, and warm ice flattens craters in 10 Myr to 1 Gyr.

### Tidal cracks

- **Cycloids, Europa.** A tensile crack starts when tidal stress reaches the ice strength, curves as the daily stress field rotates, and stalls when stress drops. A few hours later it restarts in a new direction, so each orbit adds one arc. The model needs ice tensile strength below 40 kPa and a thick liquid layer under the shell (Hoppa et al. 1999).
- **Failure threshold.** Most cusps form at 50 to 90% of the local peak tidal stress, which points to fatigue. A small obliquity with a precessing spin pole fits the global cycloid population better than eccentricity tides alone (Rhoden et al. 2021).
- **Tiger stripes, Enceladus.** Each part of each rift spends about half of every orbit in tension, when it opens and erupts; shear along the rift adds heat. The shell behaves as a thin elastic layer a few tens of kilometres thick (Hurford et al. 2007).
- **Sustained eruption.** Slots about 1 m wide, flushed by ocean water, stay open through the tidal cycle. Plume flux lags peak extension by about 1 radian, and the long-term power settles near 1e10 W (Kite and Rubin 2016).
- **Geometry.** Four parallel fissures 130 km long and 35 km apart (Abdulah et al. 2026).

### Extension from a freezing ocean

```latex
\epsilon_{area} \approx 2\left(\frac{\rho_w}{\rho_i}-1\right)\frac{h}{R}
```

Freezing a water layer of thickness h on a body of radius R raises the surface by 8.7% of h (derived; simple volume balance, not from a fetched paper). Charon's chasmata record a global areal extension of order 1%, consistent with an ancient ocean that froze (Beyer et al. 2017). The relation gives about 35 km of frozen water for 1% on Charon (derived).

### Diapirs and coronae

| Feature | Spacing or size | Layer and conditions | Source |
| --- | --- | --- | --- |
| Triton cantaloupe cells | mean separation 47 km | compositional overturn in a crust about 20 km thick, viscosity up to 1e22 Pa s | Schenk and Jackson 1993 |
| Triton cavi | basins 1300 to 2050 km^2, 300 to 500 m deep | relief on Triton stays within 1 km | Cioria et al. 2024; Schenk et al. 2021 |
| Miranda coronae | three, each about 200 km across on a 236 km body | outer belts of tilt-block normal faults; elastic thickness 2 km, gradient 8 to 20 K km^-1, heat flow 24 to 60 mW m^-2 | Pappalardo et al. 1997, via Hammond and Barr 2014 |
| Miranda, Inverness Corona | brittle shell 2.5 to 3.8 km | extension over a rising diapir, or shell thickening after a resonance with Umbriel | Leonard et al. 2023 |

The Triton figures give a cell spacing of about 2.4 times the layer thickness (derived).

### Viscous relaxation of craters

- **Ganymede.** Craters 10 km and larger need 40 to 50 mW m^-2 sustained for about 1 Gyr, or pulses near 100 mW m^-2 lasting 10 to 100 Myr. Radiogenic heat alone is insufficient, and 4 km craters resist relaxation even at 150 mW m^-2 (Bland et al. 2017).
- **Large craters.** Craters 40 km and larger on Ganymede and Callisto reach their observed shallow depths in 200 Myr to 1 Gyr under radiogenic heating, for pure ice at 120 K. Callisto, at 130 K, needs dirtier or coarser ice. Post-relaxation depth should increase with latitude (Bland et al. 2023).
- **Dome craters.** Pit craters turn into dome craters within 10 Myr, helped by leftover impact heat under the pit (Caussi et al. 2024).
- **Scaling.** The half-space relaxation time is 4 pi eta / (rho g lambda), so long wavelengths flatten first and rims outlast floors (recalled; textbook relation).

### Where the literature disagrees

- **Charon's chasmata.** Ocean freezing (Beyer et al. 2017) against expansion and contraction episodes from compaction, differentiation and geochemical reactions, with no ocean needed (Malamud et al. 2017). Chen et al. (2026) add an earlier despinning phase with 1% compression, an elastic shell at least 30 to 36 km thick and an initial 14.3 h rotation.
- **Cantaloupe terrain.** A one-time compositional overturn is hard to square with Triton's young surface. Thermal convection reproduces deformable surfaces for yield stresses of 0.05 to 0.5 MPa (Golabek et al. 2018 conference abstract). Explosion-crater origins have also been proposed (Cioria et al. 2024).
- **Miranda's coronae.** Upwelling diapirs or convection during tidal heating, against sinking silicate bodies that contract the surface (Hammond and Barr 2014 abstract).
- **What paces Enceladus's jets.** Normal opening (Hurford et al. 2007), turbulent flow in slots (Kite and Rubin 2016), or strike-slip motion peaking twice per orbit (Berne et al. 2024).

I did not retrieve a source on Ariel's chasmata, or Porco et al. (2006) beyond its title. Cycloid arc lengths and crack speeds are not in the abstract I read.

**Sources.** Hoppa et al. 1999 (FOUND); Rhoden et al. 2021 (FOUND); Hurford et al. 2007 (FOUND); Kite and Rubin 2016 (FOUND); Berne et al. 2024 (FOUND); Abdulah et al. 2026 (FOUND); Beyer et al. 2017 (FOUND); Malamud et al. 2017 (FOUND); Chen et al. 2026 (FOUND); Schenk and Jackson 1993 (FOUND); Schenk et al. 2021 (FOUND); Cioria et al. 2024 conference abstract (FOUND); Hammond and Barr 2014 conference abstract (FOUND); Leonard et al. 2023 (FOUND); Bland et al. 2017 (FOUND); Bland et al. 2023 (FOUND); Caussi et al. 2024 (FOUND).

## 9. Tholins and surface colour

Red-brown colour on cold worlds comes from photochemical organics. They arrive as haze falling from a methane-bearing atmosphere, or form in place from volatiles cold-trapped at a winter pole. Dark material can then trigger a thermal runaway that segregates ice from dirt.

### Haze fallout (Pluto)

- **Haze structure.** Haze extends to at least 200 km altitude in about 20 layers. Particles near the surface behave as 0.5 µm spheres, with fractal aggregates higher up. Extinction is greater in the north (Cheng et al. 2017).
- **Deposition.** Fallout is enough to change surface optical properties on seasonal time scales (Cheng et al. 2017), and it settles quickly on geological ones (Grundy et al. 2018).
- **Colour follows surface age.** Haze simply accumulates on the old, inactive Cthulhu region. On Sputnik Planitia, convection and the sublimation cycle refresh the surface and hide it (Grundy et al. 2018).
- **Albedo values.** Single-scattering albedo at 492 nm is about 0.98 and spectrally neutral on volatile-ice regions, and about 0.16 on the dark equatorial belt, the reddest terrain (Protopapa et al. 2020).

### Cold-trapped poles (Charon)

- **Mechanism.** Methane escaping Pluto freezes onto Charon's winter pole, which stays very cold through long seasons at high obliquity. Radiation converts it to heavier, less volatile molecules that survive the next sunrise (Grundy et al. 2016).
- **Latitude pattern.** Refractory production increases with latitude. The ratio of condensing methane to interplanetary Lyman-alpha photons sets the products: ethane at the poles, more complex refractories at mid-latitudes (Raut et al. 2022).
- **Reddening rate.** Solid ethane reddens within hours at 60 K, Charon's peak subsolar temperature, but needs about 100 years at 45 K (Gimar et al. 2026).

### Thermal runaway (Iapetus)

- **Mechanism.** Dark exogenic dust lands on the leading hemisphere, warms it, and drives water ice to sublimate and recondense on colder bright terrain. The feedback explains the sharp, complex boundary and a trailing side about 10 times brighter (Spencer and Denk 2010).
- **Condition.** It works on Iapetus alone among Saturn's moons, because slow rotation gives unusually high daytime temperatures for a given albedo (Spencer and Denk 2010).
- **Scale.** The dark blanket is metres thick or less. Equator-facing crater walls are dark and pole-facing ones bright (Denk et al. 2010).

### Where the literature disagrees

- **One colourant or several?** Grundy et al. (2018) could not match both Lowell Regio and Cthulhu with one haze particle, and propose chemistry after landing or uneven supply. Protopapa et al. (2020) fit all of Pluto's colours with a single Titan-like tholin.
- **What reddens Charon's pole.** Lyman-alpha photolysis mainly makes colourless ethane. Raut et al. (2022) invoke later solar-wind radiolysis; Gimar et al. (2026) add thermal clustering of ethane.

**Sources.** Cheng et al. 2017 (FOUND); Grundy et al. 2018 (FOUND); Protopapa et al. 2020 (FOUND); Grundy et al. 2016 (FOUND); Raut et al. 2022 (FOUND); Gimar et al. 2026 (FOUND); Spencer and Denk 2010 (FOUND); Denk et al. 2010 (FOUND).

## 10. Why small bodies become round

A body rounds when the stress from its own gravity exceeds the strength of its material. Observed thresholds are a mean radius of about 200 km for icy bodies and 300 km for rocky ones, and the threshold scales as the square root of strength divided by density.

### Rule

```latex
R_{pot} \approx B\,\sqrt{\frac{\sigma_y}{G\,\rho^{2}}}
```

Lineweaver and Norman (2010) call this the "potato radius". Their simple criterion has B = sqrt(2/pi) = 0.80; their refined differential-pressure version has B = sqrt(1/0.44) = 1.5, quoted as 240 km for a yield strength of 10 MPa and a density of 2.5 g cm^-3. Evaluating the refined form gives 233 km (derived), which confirms my reading of exponents that were garbled in extraction.

### Values

| Quantity | Icy bodies | Rocky bodies |
| --- | --- | --- |
| Observed potato radius | about 200 km | about 300 km |
| Strength inferred from that radius | 0.4 to 3 MPa | 4 to 30 MPa |
| Mean density used | 1.0 g cm^-3 (0.5 to 1.5) | 2.5 g cm^-3 (1.5 to 3.5) |
| Round examples (mean radius) | Mimas 198 km, Miranda 235 km, Enceladus 251 km | Ceres 475 km |
| Irregular or borderline examples | Hyperion 140 km, Janus 95 km | Pallas 275 km, Vesta 265 km, Juno 115 km |

Earlier thresholds quoted in the paper are 150 km for icy satellites and 400 km for bodies with stony-meteorite strength.

### Dependence on strength and temperature

- **Shape is set early.** Bodies were weaker during their hot formative period and stiffen as they cool, so a body can keep a round shape that its present strength would not produce.
- **Ice was weakened more.** The rock-to-ice strength ratio implied by the two thresholds is about 14, against about 2 at present temperatures.
- **Caveat from the authors.** Yield stress changes by only a factor of a few between 1000 K and 0 K, which sits awkwardly with that factor of 14.
- **Other controls.** Spin flattens bodies, and large impacts can undo roundness, as may have happened to Vesta.

### Where the literature disagrees

- **Density scaling fails alone.** At equal strength the formula predicts an icy threshold 2.5 times larger than the rocky one; the observed ratio is 200 to 300 km, the other way round.
- **Against the IAU proxy.** The 200 km icy threshold is well below the 419 km radius implied by the IAU's absolute-magnitude limit, and would add many trans-Neptunian dwarf planets.
- **Status of the source.** This is a conference-proceedings paper with an arXiv copy and no DOI, not a journal article. I did not retrieve a later peer-reviewed treatment.

**Sources.** Lineweaver and Norman 2010 (READ, full text, arXiv:1004.1091).

## 11. Exoplanet surfaces

Most exotic surface types have a single gate the generator can test: substellar temperature for lava worlds, water inventory for night-side ice, mass and radius for hycean worlds, an element ratio for carbon worlds, and forced eccentricity for tidal volcanism. Plate tectonics on super-Earths has no agreed rule.

### Lava worlds

```latex
T_{ss} = T_*\,(1-\alpha)^{1/4}\sqrt{\frac{r_*}{2a}}\;4^{1/4}, \qquad \cos\theta_p \approx \left(\frac{T_{lo}}{T_{ss}}\right)^{4}
```

T\_ss is the substellar temperature for albedo alpha (0.1 in the paper), and theta\_p is the angular radius of the magma pool. T\_lo is 1673 K, where the melt locks up at about 40% melt fraction (Kite et al. 2016). The pool edge sits at 61°, 76° and 84° for T\_ss of 2000, 2400 and 3000 K (derived). Nguyen et al. (2020) place the K2-141b shore at 79° for 3000 K.

| Quantity | Value | Source |
| --- | --- | --- |
| Rock-vapour pressure, bulk silicate Earth, 1400 to 2800 K | log10 P\[bar\] = k1 exp(k2 T); fresh melt k1 = -34.7, k2 = -0.00112; half evaporated -60.1, -0.00122; over 90% evaporated -60.2, -0.00101 | Kite et al. 2016 |
| Na vapour pressure | P\[Pa\] = 10^9.6 exp(-38000/T) | Nguyen et al. 2020 |
| SiO vapour pressure | P\[Pa\] = 6.16e13 exp(-69400/T) | Nguyen et al. 2020 |
| SiO2 vapour pressure | P\[Pa\] = 1.07e12 exp(-66600/T) | Nguyen et al. 2020 |
| Substellar pressure at 3000 K | Na 13.8 kPa; SiO 5.25 kPa; SiO2 240 Pa | Nguyen et al. 2020 |
| Peak wind speed | Na 2.3 km/s; SiO 2.2 km/s; SiO2 1.75 km/s | Nguyen et al. 2020 |
| Angle where the atmosphere collapses | Na 102°; SiO 92°; SiO2 89° | Nguyen et al. 2020 |
| Magma return flow needed | 1e-4 m/s for SiO or SiO2; 2 to 3 cm/s for Na | Nguyen et al. 2020 |
| Pool depth | order 10 km; about 50 km at a mean of 2400 K | Kite et al. 2016 |
| Pool overturn time | 1e8 to 1e9 s; magma currents about 0.02 m/s | Kite et al. 2016 |
| Antistellar surface temperature | about 50 K without an atmosphere | Kite et al. 2016 |

The bulk-silicate fit gives 20 Pa at 2000 K, 440 Pa at 2400 K and 6.2 kPa at 3000 K (derived).

- **Evaporation order.** Na, then K, then Fe, then Si and Mg, then Ca, then Al.
- **Composition gate.** Below a substellar temperature of about 2400 K, pool circulation outpaces evaporation and the surface keeps the bulk composition. Above it, fractional evaporation changes the surface (Kite et al. 2016).
- **Two outcomes above the gate.** An FeO-rich mantle yields a stable, buoyant CaO and Al2O3 lag with an atmosphere of about 1 Pa or less, possibly a high albedo. An FeO-poor mantle yields a patchy, time-variable surface under a thicker atmosphere.
- **Night side.** Vapour evaporates near the substellar point and condenses beyond about 35° from it. Magma carries about 1e4 times less heat than the star supplies, so it does not warm the night side (Kite et al. 2016).

### Tidally locked rocky planets

- **Hemispheric tectonics.** With a day to night surface contrast of about 1000 K on LHS 3844b, mantle models find three regimes with a mobile surface. In two, downwellings gather on one hemisphere and upwellings on the other, in either sense (Meier et al. 2021).
- **Night-side ice.** A planet with less than about a quarter of Earth's ocean mass can lock most of its surface water on the night side as ice that flows back only slowly, leaving the day side dry (Menou 2013).
- **Bistable land planets.** Inside the inner edge of the habitable zone, a dry planet can sit in a runaway greenhouse or in a collapsed state with its water in permanent cold traps. A thick enough ice cap flows under gravity and can hold liquid water at its edge or base (Leconte et al. 2013).
- **Wind pattern.** Circulation switches from super-rotation to substellar-to-antistellar flow when the equatorial Rossby deformation radius far exceeds the planet's radius (Leconte et al. 2013).

### Hycean worlds

Ocean planets under hydrogen-rich atmospheres can be as large as 2.6 Earth radii at 10 Earth masses, or 2.3 at 5. Their habitable zone reaches equilibrium temperatures of about 500 K around late M dwarfs and has no outer limit. "Dark" hycean worlds are habitable only on the permanent night side; "cold" ones receive negligible starlight (Madhusudhan et al. 2021). Later papers dispute whether such oceans survive; I did not retrieve them.

### High-pressure ice sea floors

- **Melt at the boundaries.** A partially molten layer always exists where the high-pressure ice meets the ocean. A second can exist at the rock interface at low Rayleigh number, probably only early on, carrying salts and argon-40 into the ocean (Kalousová and Sotin 2018).
- **Heat from below.** Mantle heat can melt the ice layer from beneath, sometimes episodically, depending on layer thickness, planet mass and surface temperature. High sea-floor pressure may also suppress volcanism (Noack et al. 2016).
- **Chemistry lock-in.** With 10 to 1000 times Earth's water, sea-floor pressure curtails carbon exchange with the mantle. The planet keeps the ocean chemistry of its first 1% of history, and many stay habitable for over 1 Gyr with CO2 pressures of 0.2 to 20 bar (Kite and Ford 2018).

### Carbon-rich planets

If (Mg + 2Si + Fe + 2C)/O exceeds 1, the excess carbon sits in the mantle as diamond. Diamond is about three orders of magnitude more viscous and more thermally conductive than silicate, so convection becomes sluggish or stagnant. The result covers planets of 0.1 to 2 Earth masses (Unterborn et al. 2014).

### Strongly tidally heated Earths

LP 791-18 d has a radius of 1.03 ± 0.04 and a mass of 0.9 (+0.5, -0.4) in Earth units, at 300 to 400 K. A 7.1 Earth-mass neighbour keeps its orbit eccentric, so tides heat the interior continuously and probably drive strong volcanism; water may condense on the night side (Peterson et al. 2023).

### Plate tectonics on super-Earths

- **For.** As mass rises, the shear stress available to move plates rises and plates thin, so subduction becomes easier. Super-Earths up to 10 Earth masses should have plate tectonics even if dry (Valencia et al. 2007).
- **Against.** Larger radius lowers the ratio of driving to resisting stress, because gravity strengthens faults more than it strengthens buoyancy. Large Earths tend to an episodic or stagnant lid (O'Neill and Lenardic 2007).
- **Third view.** Korenaga (2010) asks whether size matters at all; the abstract was not available to me, so I report only the title.
- **Other axes.** Composition (Unterborn et al. 2014) and day to night temperature contrast (Meier et al. 2021) each change the regime independently of size.

**Sources.** Kite et al. 2016 (READ, full text); Nguyen et al. 2020 (READ, full text); Meier et al. 2021 (FOUND); Menou 2013 (FOUND); Leconte et al. 2013 (FOUND); Madhusudhan et al. 2021 (FOUND); Journaux et al. 2020 (FOUND); Kalousová and Sotin 2018 (FOUND); Noack et al. 2016 (FOUND); Kite and Ford 2018 (FOUND); Unterborn et al. 2014 (FOUND); Peterson et al. 2023 (FOUND); Valencia et al. 2007 (FOUND); O'Neill and Lenardic 2007 (FOUND); Korenaga 2010 (FOUND, title only).

## 12. Results since 2020 the generator would otherwise miss

Several recent results overturn a default the earlier topics would lead you to code. Each row gives the finding and the rule it changes.

| Finding | Key numbers | What it changes | Source |
| --- | --- | --- | --- |
| Climate can drive convection in a volatile ice sheet | basal flux of 0.3 mW m^-2 suffices | cellular plains do not require interior heat | Morison et al. 2021 |
| Obliquity alone separates Pluto-like from Triton-like surfaces | same inventory, 20 Myr runs | derive ice distribution from spin and orbit before interior history | Bertrand et al. 2024 |
| Methane frost caps equatorial peaks without adiabatic cooling | methane-enriched air a few km up | bright CH4 summits inside dark belts | Bertrand et al. 2020 |
| Kilometre-high water-ice volcanic constructs on a cold dwarf planet | over 1e4 km^3; heat flow 5 mW m^-2 or less | allow late cryovolcanism with no tidal source | Singer et al. 2022 |
| Surface methane on Eris and Makemake may be made inside | D/H of 2.5e-4 and 2.9e-4; cores above about 150 °C | surface volatiles can track interior heating | Grundy et al. 2024; Glein et al. 2024 |
| A heavily cratered moon hides a young ocean | Mimas: ocean 20 to 30 km down, under 25 Myr old | an old surface does not exclude an ocean | Lainey et al. 2024 |
| Strong tidal heating need not make a magma ocean | Io: k2 = 0.125 ± 0.047 | tidally heated worlds can keep mostly solid mantles | Park et al. 2025 |
| A lava world holds a volatile atmosphere | 55 Cancri e: 1.95 Earth radii, 8.8 Earth masses, about 2000 K; CO2 or CO | rock-vapour-only atmospheres are not the only lava-world case | Hu et al. 2024 |
| A temperate M-dwarf rocky planet looks airless | TRAPPIST-1 b: over 0.1 bar excluded at 3 sigma; albedo 0.02 ± 0.11 | dark bare rock is a live default near M dwarfs | Ih et al. 2023 |
| Enceladus eruptions run on boiling, not overpressure | cracks cross shells under 15 km | separate crack formation from eruption drive | Rudolph et al. 2022 |
| Mercury hollows are very young and widespread | up to 19,110 hollows; maximum age about 100 kyr | hollows as an active process on hot volatile-bearing crust | Bickel et al. 2025; Wang et al. 2020 |
| Central domes grow from pit craters | within 10 Myr, using leftover impact heat | crater form on icy moons depends on age and heat flow | Caussi et al. 2024 |
| Charon's polar red needs more than photolysis | ethane reddens in hours at 60 K, a century at 45 K | colour depends on peak summer temperature | Raut et al. 2022; Gimar et al. 2026 |
| Sublimation waves have scaling laws | wavelength, speed and growth time against pressure, temperature and wind | a bedform class distinct from dunes and penitentes | Carpy et al. 2023 |

### Disputes opened since 2020

- **Eris and Makemake methane.** Mousis et al. (2025) argue the same D/H ratios fit primordial methane, with no internal source needed.
- **Europa penitentes.** See topic 4; the critique and reply both appeared after the prediction.
- **Sputnik Planitia.** Jain et al. (2026) defend bottom heating, and Stern et al. (2026) propose liquid N2 reaching the surface (topics 2 and 3).

### Leads I did not retrieve

These exist to my knowledge but were not searched or verified in this session, so they are absent from the source table: methane-ice dunes on Pluto (Telfer et al. 2018), possible oceans in the large Uranian moons (Castillo-Rogez et al. 2023), a changed volcanic vent on Venus in Magellan images (Herrick and Hensley 2023), and the dispute over whether K2-18 b is a hycean world.

**Sources.** Morison et al. 2021 (READ); Bertrand et al. 2024 (READ); Bertrand et al. 2020 (FOUND); Singer et al. 2022 (READ); Grundy et al. 2024 (FOUND); Glein et al. 2024 (FOUND); Mousis et al. 2025 (FOUND); Lainey et al. 2024 (FOUND); Park et al. 2025 (FOUND); Hu et al. 2024 (FOUND); Ih et al. 2023 (FOUND); other rows as listed in their own topics.

## Source table

The two tables list 119 sources: 21 READ and 98 FOUND, of which 9 are title only. "DOI not found" means I could not confirm one in this session; those rows link to the Consensus record instead. Years follow the journal issue where it differs from online publication.

### Topics 1 to 6

| Topic | Author | Year | Title | Journal | DOI or identifier | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | Fray, Schmitt | 2009 | Sublimation of ices of astrophysical interest: A bibliographic review | Planetary and Space Science | 10.1016/j.pss.2009.09.011 | FOUND |
| 1 | Lisse et al. | 2021 | On the Origin and Thermal Stability of Arrokoth's and Pluto's Ices | arXiv preprint; journal not confirmed | arXiv:2009.02277 | READ (Table 2) |
| 1 | Bertrand, Forget | 2016 | Observed glacier and volatile distribution on Pluto from atmosphere-topography processes | Nature | 10.1038/nature19337 | FOUND |
| 1 | Bertrand et al. | 2018 | The nitrogen cycles on Pluto over seasonal and astronomical timescales | Icarus | 10.1016/j.icarus.2018.03.012 | READ |
| 1 | Young | 2013 | Pluto's seasons: new predictions for New Horizons | Astrophysical Journal Letters | 10.1088/2041-8205/766/2/L22 | FOUND |
| 1 | Meza et al. | 2019 | Lower atmosphere and pressure evolution on Pluto from ground-based stellar occultations, 1988-2016 | Astronomy & Astrophysics | 10.1051/0004-6361/201834281 | READ |
| 1 | Lewis et al. | 2020 | Distribution and energy balance of Pluto's nitrogen ice, as seen by New Horizons in 2015 | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:73440ab1-b7a3-418a-96d4-fe6a7848db0a/?utm_source=claude_desktop) | FOUND |
| 1 | Bertrand et al. | 2022 | Volatile transport modeling on Triton with new observational constraints | Icarus | 10.1016/j.icarus.2021.114764 | READ |
| 1 | Bertrand et al. | 2024 | How obliquity has differently shaped Pluto's and Triton's landscapes and climates | PNAS | 10.1073/pnas.2408226121 | READ |
| 1 | Sicardy et al. | 2024 | Constraints on the evolution of the Triton atmosphere from occultations: 1989-2022 | Astronomy & Astrophysics | 10.1051/0004-6361/202348756 | READ |
| 1 | Marques Oliveira et al. | 2022 | Constraints on the structure and seasonal variations of Triton's atmosphere from the 5 October 2017 stellar occultation and previous observations | Astronomy & Astrophysics | DOI not found; [Consensus record](https://consensus.app/papers/details/w:2f1738a7-3ffc-4ea7-8412-3ab6572da04f/?utm_source=claude_desktop) | READ |
| 1 | Wood, Paige | 1992 | Modeling the Martian seasonal CO2 cycle 1. Fitting the Viking Lander pressure curves | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:4a678c2b-5767-42b2-a9c1-6f469a5b49ed/?utm_source=claude_desktop) | FOUND |
| 1 | Corchete | 2025 | CO2 polar mass on Mars determined from InSight pressure data | Frontiers in Astronomy and Space Sciences | DOI not found; [Consensus record](https://consensus.app/papers/details/w:e159039d-c4a5-4d65-a32b-4a61e33fdde4/?utm_source=claude_desktop) | READ |
| 1 | Mendoza et al. | 2020 | Thermodynamic simulation of the seasonal cycle of temperature, pressure and ice caps on Mars | Atmósfera | DOI not found; [Consensus record](https://consensus.app/papers/details/w:f54e26b0-5940-4fb3-9f2f-50e43075bde7/?utm_source=claude_desktop) | READ |
| 2 | McKinnon et al. | 2016 | Convection in a volatile nitrogen-ice-rich layer drives Pluto's geological vigour | Nature | 10.1038/nature18289 | READ |
| 2 | Trowbridge et al. | 2016 | Vigorous convection as the explanation for Pluto's polygonal terrain | Nature | 10.1038/nature18016 | FOUND |
| 2 | Morison et al. | 2021 | Sublimation-driven convection in Sputnik Planitia on Pluto | Nature | 10.1038/s41586-021-04095-w | READ |
| 2 | Jain et al. | 2026 | Localized Convection in Sputnik Planitia, Pluto | Geophysical Research Letters | 10.1029/2026GL121638 | READ |
| 2 | Moore et al. | 2016 | The geology of Pluto and Charon through the eyes of New Horizons | Science | 10.1126/science.aad7055 | READ |
| 3 | Umurhan et al. | 2017 | Modeling glacial flow on and onto Pluto's Sputnik Planitia | Icarus | 10.1016/j.icarus.2017.01.017 | READ |
| 3 | Yamashita et al. | 2010 | Experimental study on the rheological properties of polycrystalline solid nitrogen and methane: Implications for tectonic processes on Triton | Icarus | 10.1016/j.icarus.2009.11.032 | FOUND |
| 3 | Yamashita et al. | 1997 | Viscoelastic properties of polycrystalline solid methane and carbon dioxide | Geophysical Research Letters | DOI not found; [Consensus record](https://consensus.app/papers/details/w:2220fa55-bf47-485c-8a44-0e544b06a7b5/?utm_source=claude_desktop) | FOUND |
| 3 | Stern et al. | 2026 | Evidence for Possible N2 Basal Flow beneath Pluto's Northern Sputnik Planitia | Planetary Science Journal | DOI not found; [Consensus record](https://consensus.app/papers/details/w:39b20017-a749-430c-bf5a-4eacc9bf2ed8/?utm_source=claude_desktop) | READ |
| 4 | Moores et al. | 2017 | Penitentes as the origin of the bladed terrain of Tartarus Dorsa on Pluto | Nature | 10.1038/nature20779 | FOUND |
| 4 | Hobley et al. | 2018 | Formation of metre-scale bladed roughness on Europa's surface by ablation of ice | Nature Geoscience | 10.1038/s41561-018-0235-0 | FOUND |
| 4 | Hand et al. | 2019 | Penitente formation is unlikely on Europa | Nature Geoscience | 10.1038/s41561-019-0496-2 | READ |
| 4 | Hobley et al. | 2019 | Reply to: Penitente formation is unlikely on Europa | Nature Geoscience | 10.1038/s41561-019-0497-1 | READ |
| 4 | Claudin et al. | 2015 | Physical processes causing the formation of penitentes | Physical Review E | 10.1103/PhysRevE.92.033015 | FOUND |
| 4 | Berisford et al. | 2024 | Laboratory formation of micro-penitentes at temperatures and pressures relevant to Earth and other worlds | Journal of Glaciology | DOI not found; [Consensus record](https://consensus.app/papers/details/w:f4927abe-6dbb-4192-8a97-93e0c9677ae3/?utm_source=claude_desktop) | FOUND |
| 4 | Warren | 2022 | Snow spikes (penitentes) in the dry Andes, but not on Europa: a defense of Lliboutry's classic paper | Annals of Glaciology | DOI not found; [Consensus record](https://consensus.app/papers/details/w:928ab120-8b0c-4902-8514-43cd7779d2bd/?utm_source=claude_desktop) | FOUND |
| 4 | Moore et al. | 2016 | Bladed Terrain on Pluto: Possible Origins and Evolutions | AGU Fall Meeting abstract | DOI not found; [Consensus record](https://consensus.app/papers/details/w:406ccfdc-daeb-4c3f-9b14-f444d9b0b43c/?utm_source=claude_desktop) | FOUND |
| 4 | Bertrand et al. | 2020 | Equatorial mountains on Pluto are covered by methane frosts resulting from a unique atmospheric process | Nature Communications | DOI not found; [Consensus record](https://consensus.app/papers/details/w:836ba91c-4093-4ac9-bdaf-5b36bd62491a/?utm_source=claude_desktop) | FOUND |
| 5 | Buhler, Ingersoll | 2018 | Sublimation pit distribution indicates convection cell surface velocities of about 10 cm per year in Sputnik Planitia, Pluto | Icarus | 10.1016/j.icarus.2017.09.018 | FOUND |
| 5 | Stern et al. | 2021 | New Investigations of Dark-floored Pits In the Volatile Ice of Sputnik Planitia on Pluto | Astronomical Journal | DOI not found; [Consensus record](https://consensus.app/papers/details/w:8a078913-92c3-46d2-8902-429f84948fde/?utm_source=claude_desktop) | FOUND |
| 5 | Thomas et al. | 2009 | Residual south polar cap of Mars: Stratigraphy, history, and implications of recent changes | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:5e529fb6-0f82-4f5a-8beb-8df390e13e7b/?utm_source=claude_desktop) | FOUND |
| 5 | Thomas et al. | 2023 | Moats in the residual south polar cap of Mars: Ages, formation, and evolution | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:7fbe3a91-6ac8-41b8-9a33-2732edb9d42c/?utm_source=claude_desktop) | FOUND |
| 5 | Buhler et al. | 2017 | How the martian residual south polar cap develops quasi-circular and heart-shaped pits, troughs, and moats | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:d8acd874-bd53-403b-bf1c-8eb750c09eea/?utm_source=claude_desktop) | FOUND |
| 5 | Betz et al. | 2009 | Determining the Heights and Distributions of Swiss Cheese Features on Mars South Polar Residual Cap using Photoclinometry | Lunar and Planetary Science Conference abstract | DOI not found; [Consensus record](https://consensus.app/papers/details/w:2e05ddc1-6189-4584-88d9-03fac9af9fff/?utm_source=claude_desktop) | FOUND |
| 5 | White et al. | 2016 | Modeling of ice pinnacle formation on Callisto | JGR Planets | 10.1002/2015JE004846 | FOUND |
| 5 | Moore et al. | 1999 | Mass Movement and Landform Degradation on the Icy Galilean Satellites: Results of the Galileo Nominal Mission | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:93ac859c-e501-44dc-9159-26681f51e0d3/?utm_source=claude_desktop) | FOUND |
| 5 | Deutsch et al. | 2025 | Hollows on Mercury: Global Classification of Degradation States and Insight Into Hollow Evolution | JGR Planets | DOI not found; [Consensus record](https://consensus.app/papers/details/w:4d30110d-1d0f-4a35-a7bf-099e77718116/?utm_source=claude_desktop) | FOUND |
| 5 | Bickel et al. | 2025 | Hollows on Mercury: Creation and Analysis of a Global Reference Catalog With Deep Learning | JGR Machine Learning and Computation | DOI not found; [Consensus record](https://consensus.app/papers/details/w:2c504f06-aa64-4b56-9ced-3e268d8d5fe6/?utm_source=claude_desktop) | FOUND |
| 5 | Wang et al. | 2020 | Lost Volatiles During the Formation of Hollows on Mercury | JGR Planets | DOI not found; [Consensus record](https://consensus.app/papers/details/w:c37571f9-eb5f-4b5a-96a8-1a31f0e24316/?utm_source=claude_desktop) | FOUND |
| 5 | Phillips et al. | 2021 | The lifecycle of hollows on Mercury: An evaluation of candidate volatile phases and a novel model of formation | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:4a6dd20d-69d2-4e93-9cd9-abcda7b3363b/?utm_source=claude_desktop) | FOUND |
| 5 | Vincent et al. | 2015 | Large heterogeneities in comet 67P as revealed by active pits from sinkhole collapse | Nature | 10.1038/nature14564 | FOUND |
| 5 | Carpy et al. | 2023 | Scaling laws for ablation waves formed by ice sublimation and rock dissolution: applications to the Earth, Mars and Pluto | Frontiers in Astronomy and Space Sciences | DOI not found; [Consensus record](https://consensus.app/papers/details/w:e88439c0-9266-45a2-8268-0a2c65bcd26e/?utm_source=claude_desktop) | FOUND |
| 5 | Howard et al. | 2017 | Pluto: Pits and mantles on uplands north and east of Sputnik Planitia | Icarus | 10.1016/j.icarus.2017.02.027 | FOUND (title only) |
| 5 | Moore et al. | 2017 | Sublimation as a landform-shaping process on Pluto | Icarus | 10.1016/j.icarus.2016.08.025 | FOUND (title only) |
| 5 | Blewett et al. | 2011 | Hollows on Mercury: MESSENGER Evidence for Geologically Recent Volatile-Related Activity | Science | 10.1126/science.1211681 | FOUND (title only) |
| 5 | Byrne, Ingersoll | 2003 | A Sublimation Model for Martian South Polar Ice Features | Science | 10.1126/science.1080148 | FOUND (title only) |
| 6 | Kieffer et al. | 2006 | CO2 jets formed by sublimation beneath translucent slab ice in Mars' seasonal south polar ice cap | Nature | 10.1038/nature04945 | FOUND |
| 6 | Piqueux et al. | 2003 | Sublimation of Mars's southern seasonal CO2 ice cap and the formation of spiders | Journal of Geophysical Research | DOI not found; [Consensus record](https://consensus.app/papers/details/w:fab908c1-5c6b-4560-918a-5ff2c8f540d8/?utm_source=claude_desktop) | FOUND |
| 6 | Mc Keown et al. | 2021 | The formation of araneiforms by carbon dioxide venting and vigorous sublimation dynamics under martian atmospheric pressure | Scientific Reports | 10.1038/s41598-021-82763-7 | FOUND |
| 6 | Mc Keown et al. | 2024 | A Lab-scale Investigation of the Mars Kieffer Model | Planetary Science Journal | DOI not found; [Consensus record](https://consensus.app/papers/details/w:6df1aabd-2a58-4931-97dd-92f1e9cf8ca6/?utm_source=claude_desktop) | FOUND |
| 6 | Soderblom et al. | 1990 | Triton's Geyser-Like Plumes: Discovery and Basic Characterization | Science | 10.1126/science.250.4979.410 | FOUND |
| 6 | Kirk et al. | 1990 | Subsurface Energy Storage and Transport for Solar-Powered Geysers on Triton | Science | 10.1126/science.250.4979.424 | FOUND (title only) |
| 6 | Hofgartner et al. | 2022 | Hypotheses for Triton's plumes: New analyses and future remote sensing tests | Icarus | 10.1016/j.icarus.2021.114835 | FOUND |
| 6 | Duxbury, Brown | 1997 | The Role of an Internal Heat Source for the Eruptive Plumes on Triton | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:62886cc1-14e9-4898-96bd-b070105e92c0/?utm_source=claude_desktop) | FOUND |

### Topics 7 to 12

| Topic | Author | Year | Title | Journal | DOI or identifier | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 7 | Manga, Wang | 2007 | Pressurized oceans and the eruption of liquid water on Europa and Enceladus | Geophysical Research Letters | 10.1029/2007GL029297 | FOUND |
| 7 | Fagents | 2003 | Considerations for effusive cryovolcanism on Europa: The post-Galileo perspective | JGR Planets | 10.1029/2003JE002128 | FOUND (title only) |
| 7 | Rudolph et al. | 2022 | Cooling Crusts Create Concomitant Cryovolcanic Cracks | Geophysical Research Letters | 10.1029/2021GL094421 | FOUND |
| 7 | Lesage et al. | 2022 | Simulation of Freezing Cryomagma Reservoirs in Viscoelastic Ice Shells | Planetary Science Journal | DOI not found; [Consensus record](https://consensus.app/papers/details/w:5573049f-b4ad-43cc-b43d-a0f5508e8ce8/?utm_source=claude_desktop) | FOUND |
| 7 | Singer et al. | 2022 | Large-scale cryovolcanic resurfacing on Pluto | Nature Communications | 10.1038/s41467-022-29056-3 | READ |
| 7 | Sori et al. | 2017 | The vanishing cryovolcanoes of Ceres | Geophysical Research Letters | 10.1002/2016GL072319 | FOUND |
| 7 | Ruesch et al. | 2016 | Cryovolcanism on Ceres | Science | 10.1126/science.aaf4286 | FOUND |
| 7 | Ruesch et al. | 2019 | Slurry extrusion on Ceres from a convective mud-bearing mantle | Nature Geoscience | DOI not found; [Consensus record](https://consensus.app/papers/details/w:0f44de04-b3ca-4cb9-9405-fa6caefa2a6d/?utm_source=claude_desktop) | FOUND |
| 7 | Quick et al. | 2019 | A Possible Brine Reservoir Beneath Occator Crater: Thermal and Compositional Evolution and Formation of the Cerealia Dome and Vinalia Faculae | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:d505ce4e-a308-413c-942e-b954dc9f1043/?utm_source=claude_desktop) | FOUND |
| 7 | Scully et al. | 2020 | The varied sources of faculae-forming brines in Ceres' Occator crater emplaced via hydrothermal brine effusion | Nature Communications | DOI not found; [Consensus record](https://consensus.app/papers/details/w:c5f6a8b3-4c63-4ba1-8041-88f1e59fb224/?utm_source=claude_desktop) | FOUND |
| 8 | Hoppa et al. | 1999 | Formation of Cycloidal Features on Europa | Science | 10.1126/science.285.5435.1899 | FOUND |
| 8 | Rhoden et al. | 2021 | Obliquity, Precession, and Fracture Mechanics: Implications of Europa's Global Cycloid Population | JGR Planets | DOI not found; [Consensus record](https://consensus.app/papers/details/w:858b20ee-c456-4308-8607-9b340d72f453/?utm_source=claude_desktop) | FOUND |
| 8 | Hurford et al. | 2007 | Eruptions arising from tidally controlled periodic openings of rifts on Enceladus | Nature | 10.1038/nature05821 | FOUND |
| 8 | Porco et al. | 2006 | Cassini Observes the Active South Pole of Enceladus | Science | 10.1126/science.1123013 | FOUND (title only) |
| 8 | Kite, Rubin | 2016 | Sustained eruptions on Enceladus explained by turbulent dissipation in tiger stripes | PNAS | DOI not found; [Consensus record](https://consensus.app/papers/details/w:b5811aa7-8a0a-42fa-9222-e5d54581c32c/?utm_source=claude_desktop) | FOUND |
| 8 | Berne et al. | 2024 | Jet activity on Enceladus linked to tidally driven strike-slip motion along tiger stripes | Nature Geoscience | DOI not found; [Consensus record](https://consensus.app/papers/details/w:64eed95b-5e78-4729-bd0a-ba7ad7e3e484/?utm_source=claude_desktop) | FOUND |
| 8 | Abdulah et al. | 2026 | An Oceanic Mechanism for Geyser Formation on Enceladus | AGU Advances | DOI not found; [Consensus record](https://consensus.app/papers/details/w:7e3b1d66-1aff-4ec1-9b8f-169560522192/?utm_source=claude_desktop) | FOUND |
| 8 | Beyer et al. | 2017 | Charon tectonics | Icarus | 10.1016/j.icarus.2016.12.018 | FOUND |
| 8 | Malamud et al. | 2017 | The contraction/expansion history of Charon with implications for its planetary-scale tectonic belt | MNRAS | DOI not found; [Consensus record](https://consensus.app/papers/details/w:1e4f26ed-2321-4311-9eb1-e4648cbe0cde/?utm_source=claude_desktop) | FOUND |
| 8 | Chen et al. | 2026 | Early tidal despinning history recorded in the tectonics of Oz Terra, Charon | Nature Communications | DOI not found; [Consensus record](https://consensus.app/papers/details/w:c7ce40f0-c42b-4e68-a328-c1ee8222c683/?utm_source=claude_desktop) | FOUND |
| 8 | Schenk, Jackson | 1993 | Diapirism on Triton: A record of crustal layering and instability | Geology | DOI not found; [Consensus record](https://consensus.app/papers/details/w:8acd2346-5300-477a-9126-8fc060326f2d/?utm_source=claude_desktop) | FOUND |
| 8 | Schenk et al. | 2021 | Triton: Topography and Geology of a Probable Ocean World with Comparison to Pluto and Charon | Remote Sensing | DOI not found; [Consensus record](https://consensus.app/papers/details/w:b05e0e55-d7d6-4d8a-9f29-1ed30a9adbed/?utm_source=claude_desktop) | FOUND |
| 8 | Cioria et al. | 2024 | Geomorphological analysis with digital elevation model of Monad Regio, Triton | Conference abstract | DOI not found; [Consensus record](https://consensus.app/papers/details/w:7d8e63c3-444a-491d-9199-1e3af931c0bb/?utm_source=claude_desktop) | FOUND |
| 8 | Golabek et al. | 2018 | 3D numerical models of thermal convection inside Triton's icy shell | EGU General Assembly abstract | DOI not found; [Consensus record](https://consensus.app/papers/details/w:c97aded5-8542-49a4-94a4-ef4fbcd61f83/?utm_source=claude_desktop) | FOUND |
| 8 | Hammond, Barr | 2014 | Formation of Coronae on Miranda by Convection Driven Resurfacing | Lunar and Planetary Science Conference abstract | DOI not found; [Consensus record](https://consensus.app/papers/details/w:e8fd6520-6a4c-4557-9c76-9b977483ce11/?utm_source=claude_desktop) | FOUND |
| 8 | Pappalardo et al. | 1997 | Extensional tilt blocks on Miranda: Evidence for an upwelling origin of Arden Corona | JGR Planets | 10.1029/97JE00802 | FOUND (title only; values via Hammond and Barr) |
| 8 | Leonard et al. | 2023 | Unraveling the Geologic History of Miranda's Inverness Corona | Planetary Science Journal | DOI not found; [Consensus record](https://consensus.app/papers/details/w:0a4e5c3f-c164-48ed-94a3-dfd3386ea345/?utm_source=claude_desktop) | FOUND |
| 8 | Bland et al. | 2017 | Viscous relaxation of Ganymede's impact craters: Constraints on heat flux | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:b6d6f776-9eb5-4a31-8a39-fd5c3525a0bf/?utm_source=claude_desktop) | FOUND |
| 8 | Bland et al. | 2023 | The inevitability of large shallow craters on Callisto and Ganymede: Implications for crater depth-diameter trends | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:06526185-8afd-4087-825a-633bfffbc887/?utm_source=claude_desktop) | FOUND |
| 8 | Caussi et al. | 2024 | Dome Craters on Ganymede and Callisto May Form by Topographic Relaxation of Pit Craters Aided by Remnant Impact Heat | JGR Planets | DOI not found; [Consensus record](https://consensus.app/papers/details/w:11152a1f-a1a8-4449-9fb0-51cce6273f2a/?utm_source=claude_desktop) | FOUND |
| 9 | Cheng et al. | 2017 | Haze in Pluto's atmosphere | Icarus | 10.1016/j.icarus.2017.02.024 | FOUND |
| 9 | Grundy et al. | 2018 | Pluto's haze as a surface material | Icarus | 10.1016/j.icarus.2018.05.019 | FOUND |
| 9 | Protopapa et al. | 2020 | Disk-resolved Photometric Properties of Pluto and the Coloring Materials across its Surface | Astronomical Journal | DOI not found; [Consensus record](https://consensus.app/papers/details/w:09cc2696-d642-4aab-80d9-468203f1ff60/?utm_source=claude_desktop) | FOUND |
| 9 | Grundy et al. | 2016 | The formation of Charon's red poles from seasonally cold-trapped volatiles | Nature | 10.1038/nature19340 | FOUND |
| 9 | Raut et al. | 2022 | Charon's refractory factory | Science Advances | DOI not found; [Consensus record](https://consensus.app/papers/details/w:304d404d-db8b-4ee6-a956-432395e22fe5/?utm_source=claude_desktop) | FOUND |
| 9 | Gimar et al. | 2026 | Thermal clustering of solid ethane with potential relevance to Charon's Mordor region | Astronomy & Astrophysics | DOI not found; [Consensus record](https://consensus.app/papers/details/w:736d1afa-27ad-4211-b650-b0e9e2525715/?utm_source=claude_desktop) | FOUND |
| 9 | Spencer, Denk | 2010 | Formation of Iapetus' Extreme Albedo Dichotomy by Exogenically Triggered Thermal Ice Migration | Science | 10.1126/science.1177132 | FOUND |
| 9 | Denk et al. | 2010 | Iapetus: Unique Surface Properties and a Global Color Dichotomy from Cassini Imaging | Science | DOI not found; [Consensus record](https://consensus.app/papers/details/w:270713b1-8032-4b85-a2c9-3028b1ae4517/?utm_source=claude_desktop) | FOUND |
| 10 | Lineweaver, Norman | 2010 | The Potato Radius: a Lower Minimum Size for Dwarf Planets | Proceedings of the 9th Australian Space Science Conference | arXiv:1004.1091 (DOI not found) | READ |
| 11 | Kite et al. | 2016 | Atmosphere-interior exchange on hot, rocky exoplanets | Astrophysical Journal | 10.3847/0004-637X/828/2/80 | READ |
| 11 | Nguyen et al. | 2020 | Modelling the atmosphere of lava planet K2-141b: implications for low- and high-resolution spectroscopy | MNRAS | 10.1093/mnras/staa2487 | READ |
| 11 | Meier et al. | 2021 | Hemispheric Tectonics on Super-Earth LHS 3844b | Astrophysical Journal Letters | 10.3847/2041-8213/abe400 | FOUND |
| 11 | Menou | 2013 | Water-trapped worlds | Astrophysical Journal | 10.1088/0004-637X/774/1/51 | FOUND |
| 11 | Leconte et al. | 2013 | 3D climate modeling of close-in land planets: Circulation patterns, climate moist bistability, and habitability | Astronomy & Astrophysics | 10.1051/0004-6361/201321042 | FOUND |
| 11 | Madhusudhan et al. | 2021 | Habitability and Biosignatures of Hycean Worlds | Astrophysical Journal | 10.3847/1538-4357/abfd9c | FOUND |
| 11 | Journaux et al. | 2020 | Large Ocean Worlds with High-Pressure Ices | Space Science Reviews | DOI not found; [Consensus record](https://consensus.app/papers/details/w:8acb3410-50a3-4698-a1de-1e7910521d7b/?utm_source=claude_desktop) | FOUND |
| 11 | Kalousová, Sotin | 2018 | Melting in High-Pressure Ice Layers of Large Ocean Worlds: Implications for Volatiles Transport | Geophysical Research Letters | DOI not found; [Consensus record](https://consensus.app/papers/details/w:2c13fdc4-2c38-4812-a48c-9cf04ee9e036/?utm_source=claude_desktop) | FOUND |
| 11 | Noack et al. | 2016 | Water-rich planets: How habitable is a water layer deeper than on Earth? | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:68222d97-a63f-4074-bd4d-4cb5b155d168/?utm_source=claude_desktop) | FOUND |
| 11 | Kite, Ford | 2018 | Habitability of Exoplanet Waterworlds | Astrophysical Journal | DOI not found; [Consensus record](https://consensus.app/papers/details/w:c479a077-bba6-4fcb-a3f4-1bf531521cab/?utm_source=claude_desktop) | FOUND |
| 11 | Unterborn et al. | 2014 | The role of carbon in extrasolar planetary geodynamics and habitability | Astrophysical Journal | 10.1088/0004-637X/793/2/124 | FOUND |
| 11 | Peterson et al. | 2023 | A temperate Earth-sized planet with tidal heating transiting an M6 star | Nature | 10.1038/s41586-023-05934-8 | FOUND |
| 11 | Valencia et al. | 2007 | Inevitability of Plate Tectonics on Super-Earths | Astrophysical Journal | 10.1086/524012 | FOUND |
| 11 | O'Neill, Lenardic | 2007 | Geological consequences of super-sized Earths | Geophysical Research Letters | 10.1029/2007GL030598 | FOUND |
| 11 | Korenaga | 2010 | On the likelihood of plate tectonics on super-Earths: does size matter? | Astrophysical Journal Letters | 10.1088/2041-8205/725/1/L43 | FOUND (title only) |
| 12 | Grundy et al. | 2024 | Measurement of D/H and 13C/12C ratios in methane ice on Eris and Makemake: Evidence for internal activity | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:9270315c-d5a5-43f7-a044-3a02b18c7657/?utm_source=claude_desktop) | FOUND |
| 12 | Glein et al. | 2024 | Moderate D/H ratios in methane ice on Eris and Makemake as evidence of hydrothermal or metamorphic processes in their interiors: Geochemical analysis | Icarus | DOI not found; [Consensus record](https://consensus.app/papers/details/w:a2993141-bf59-4068-b5a1-fe24bdc0428a/?utm_source=claude_desktop) | FOUND |
| 12 | Mousis et al. | 2025 | Primordial Origin of Methane on Eris and Makemake Supported by D/H Ratios | Astrophysical Journal Letters | DOI not found; [Consensus record](https://consensus.app/papers/details/w:595cb9d0-fcd7-444c-a7ee-2d45176269a1/?utm_source=claude_desktop) | FOUND |
| 12 | Lainey et al. | 2024 | A recently formed ocean inside Saturn's moon Mimas | Nature | 10.1038/s41586-023-06975-9 | FOUND |
| 12 | Park et al. | 2025 | Io's tidal response precludes a shallow magma ocean | Nature | 10.1038/s41586-024-08442-5 | FOUND |
| 12 | Hu et al. | 2024 | A secondary atmosphere on the rocky exoplanet 55 Cancri e | Nature | 10.1038/s41586-024-07432-x | FOUND |
| 12 | Ih et al. | 2023 | Constraining the Thickness of the Atmosphere of TRAPPIST-1 b from its JWST Secondary Eclipse Observation | Astrophysical Journal Letters | 10.3847/2041-8213/ace03b | FOUND |
