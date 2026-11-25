"""Write packs/base-physics/formulas/surface.json: the SW1 space weathering blocks.
Run from the repository root: python3 scripts/build-weathering-blocks.py"""
import json, math
GRIER01="Grier, J. A., McEwen, A. S., Lucey, P. G., Milazzo, M. and Strom, R. G. 2001, Journal of Geophysical Research 106, 32847: the optical maturity of the ejecta of large rayed lunar craters; bright rays fade as their soil matures, over about a billion years. doi:10.1029/1999JE001160"
PN16="Pieters, C. M. and Noble, S. K. 2016, Journal of Geophysical Research: Planets 121, 1865: space weathering on airless bodies; solar wind ions and micrometeorites both make nanophase iron that darkens and reddens soil. doi:10.1002/2016JE005128"
HAPKE01="Hapke, B. 2001, Journal of Geophysical Research 106, 10039: space weathering from Mercury to the asteroid belt; nanophase iron absorbs most strongly at short wavelengths, so mature soil is darker and redder. doi:10.1029/2000JE001338"
LR11="Lucey, P. G. and Riner, M. A. 2011, Icarus 212, 451: the optical effects of small iron particles that darken but do not redden; reflectance of fresh and mature lunar soils. doi:10.1016/j.icarus.2011.01.009"
def inp(p,l): return {"port":p,"letter":l}
def par(p,l,d,r,desc): return {"param":p,"letter":l,"default":d,"range":r,"description":desc}
TAU=9.5e15; FW=0.5
def mat(t,w,o,tau=TAU,fw=FW):
    rate=fw*w*min(max(o,0),1)+(1-fw)
    return 1-math.exp(-max(t,0)*rate/tau)
K=(0.62,0.55,0.48)
def fac(M): return [k**min(max(M,0),1) for k in K]
atoms=[]
ex=[dict(t=t,w=w,o=o,M=mat(t,w,o)) for (t,w,o) in [(3.2e16,1,1),(3.2e15,1,1),(3.2e16,7,0),(0,1,1)]]
atoms.append({"id":"sse.surface.space_weathering","version":"1.0.0","cost":"trivial",
 "inputs":[inp("exposure_age","t"),inp("wind_flux_ratio","w"),inp("wind_reaches_ground","o")],
 "outputs":[inp("maturity","M")],
 "params":[par("tau_1au","tau",TAU,[1.0e13,1.0e17],"time for airless soil at 1 AU to come within 1/e of full maturity, with the wind and micrometeorites both acting (fitted so the Moon's bright rays fade over about a billion years, Grier et al. 2001)"),
           par("wind_share","fw",FW,[0.0,1.0],"share of the weathering done by the star's wind at 1 AU; the rest by micrometeorites, taken as the same at any distance (Pieters and Noble 2016)")],
 "references":[GRIER01,PN16],
 "shown":"M = 1 − e^(−t (f_w w o + 1 − f_w)/τ)",
 "equation":{"name":"How long bare ground has lain open to space",
  "formula":"rate = fw * w * clamp(o, 0, 1) + (1 - fw)\nM = 1 - exp(-max(t, 0) * rate / tau)",
  "inputs":{"t":{"kind":"surface.exposure_age","unit":"s","name":"time the ground has lain open"},
   "w":{"kind":"surface.wind_flux_ratio","unit":"1","name":"star's wind flux against 1 AU"},
   "o":{"kind":"surface.wind_reaches_ground","unit":"1","name":"wind reaches the ground (1) or is held off by a field (0)"},
   "tau":{"kind":"model.weathering_time","unit":"s","name":"weathering time at 1 AU"},
   "fw":{"kind":"model.weathering_wind_share","unit":"1","name":"wind's share of the weathering"}},
  "outputs":{"M":{"kind":"surface.maturity","unit":"1","name":"soil maturity, 0 fresh to 1 mature"}},
  "source":{"text":"Grier, J. A. et al. 2001, J. Geophys. Res. 106, 32847","doi":"10.1029/1999JE001160","where":"the fading of lunar crater rays with age"},
  "explain":{"curious":"On a world with no air, the star's wind and a rain of dust grains slowly cook the top of the soil, leaving tiny beads of iron that darken it. Fresh craters throw out bright, raw rock; over hundreds of millions of years it weathers dark, so the youngest craters and their rays stand out.",
   "science":"Solar wind ions and micrometeorite vapour deposit nanophase iron on soil grains. Maturity is taken to approach 1 exponentially with exposure time; the wind's part scales with its flux (the inverse square of distance) and is held off by a magnetosphere, the micrometeorites' part is taken as the same at any distance. Gardening is folded into the one fitted time."},
  "examples":ex}})
ex=[dict(M=M,fr=fac(M)[0],fg=fac(M)[1],fb=fac(M)[2]) for M in (0.0,0.5,1.0)]
atoms.append({"id":"sse.surface.weathered_reflectance","version":"1.0.0","cost":"trivial",
 "inputs":[inp("maturity","M")],
 "outputs":[inp("factor_650","fr"),inp("factor_550","fg"),inp("factor_450","fb")],
 "params":[par("mature_650","kr",K[0],[0.05,1.0],"mature over fresh reflectance of the same soil at 650 nm"),
           par("mature_550","kg",K[1],[0.05,1.0],"mature over fresh reflectance of the same soil at 550 nm"),
           par("mature_450","kb",K[2],[0.05,1.0],"mature over fresh reflectance of the same soil at 450 nm")],
 "references":[HAPKE01,LR11],
 "shown":"f(λ) = k(λ)^M",
 "equation":{"name":"How much darker and redder weathered soil looks",
  "formula":"m = clamp(M, 0, 1)\nfr = kr ^ m\nfg = kg ^ m\nfb = kb ^ m",
  "inputs":{"M":{"kind":"surface.maturity","unit":"1","name":"soil maturity"},
   "kr":{"kind":"model.mature_reflectance_650","unit":"1","name":"mature over fresh at 650 nm"},
   "kg":{"kind":"model.mature_reflectance_550","unit":"1","name":"mature over fresh at 550 nm"},
   "kb":{"kind":"model.mature_reflectance_450","unit":"1","name":"mature over fresh at 450 nm"}},
  "outputs":{"fr":{"kind":"surface.weathering_factor_650","unit":"1","name":"reflectance factor at 650 nm"},
   "fg":{"kind":"surface.weathering_factor_550","unit":"1","name":"reflectance factor at 550 nm"},
   "fb":{"kind":"surface.weathering_factor_450","unit":"1","name":"reflectance factor at 450 nm"}},
  "source":{"text":"Hapke, B. 2001, J. Geophys. Res. 106, 10039","doi":"10.1029/2000JE001338","where":"nanophase iron and the darkening and reddening of soil"},
  "explain":{"curious":"Weathered soil is darker in every colour, but loses most of its blue, so old ground on the Moon looks dark grey-brown while fresh crater ejecta look bright and bluish-grey.",
   "science":"Nanophase iron absorbs more strongly at short wavelengths, so the ratio of mature to fresh reflectance falls from red to blue. Each band's ratio at full maturity is a parameter; between fresh and mature the factor is taken as the ratio raised to the maturity, exactly 1 for fresh ground."},
  "examples":ex}})
p='packs/base-physics/formulas/surface.json'
d={"format":"sse.formulas/1","family":"sse.surface","about":"Physics blocks whose maths is a formula (ND3b): each keeps its id, ports and results, and its maths is the Equation below, with a name that says what happens.","atoms":atoms}
open(p,'w').write(json.dumps(d,indent=1,ensure_ascii=False)+"\n")
print("ok",[ (round(e.get('M',0),4)) for e in atoms[0]['equation']['examples']])
