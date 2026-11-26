"""Write packs/base-physics/formulas/surface.json: the SW1 space weathering and IG1 ice ground blocks.
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

# IG1: ice ground.
HUEBNER92="Huebner, W. F., Keady, J. J. and Lyon, S. P. 1992, Astrophysics and Space Science 195, 1: solar photo rates for planetary atmospheres and atmospheric pollutants; a water molecule at 1 AU is broken up by sunlight about once a day (rate about 1.2e-5 per second under the quiet Sun). doi:10.1007/BF00644558"
FS09="Fray, N. and Schmitt, B. 2009, Planetary and Space Science 57, 2053: sublimation of ices of astrophysical interest, a bibliographic review; the vapour curves the data sheet's Look up reads. doi:10.1016/j.pss.2009.09.011"
WATSON61="Watson, K., Murray, B. C. and Brown, H. 1961, Journal of Geophysical Research 66, 3033: the behaviour of volatiles on the lunar surface; open ice sublimes away on the warm Moon and survives only in cold traps. doi:10.1029/JZ066i009p03033"
SM99="Showman, A. P. and Malhotra, R. 1999, Science 286, 77: the Galilean satellites; their mean densities fall outward from Io's rock to Callisto's half ice and half rock. doi:10.1126/science.286.5437.77"
RHO_R=3500.0; RHO_I=930.0
def ice_share(rho,rr=RHO_R,ri=RHO_I):
    if rho<=0 or rr<=ri: return 0.0
    w=(1/rho-1/rr)/(1/ri-1/rr); return min(max(w,0.0),1.0)
ex=[dict(rho=r,rr=RHO_R,ri=RHO_I,w=ice_share(r)) for r in (3013.0,1936.0,3344.0,5514.0)]
atoms.append({"id":"sse.surface.ice_share_from_density","version":"1.0.0","cost":"trivial",
 "inputs":[inp("mean_density","rho"),inp("rock_density","rr"),inp("ice_density","ri")],
 "outputs":[inp("ice_share","w")],
 "params":[],
 "references":[SM99],
 "shown":"1/ρ = w/ρ_ice + (1 − w)/ρ_rock",
 "equation":{"name":"How much of a small world is ice",
  "formula":"w0 = (1 / rho - 1 / rr) / (1 / ri - 1 / rr)\nw = if rho <= 0 or rr <= ri then 0 else min(max(w0, 0), 1)",
  "inputs":{"rho":{"kind":"body.bulk_density","unit":"kg m-3","name":"world's mean density"},
   "rr":{"kind":"material.rock_density","unit":"kg m-3","name":"rock's density (Look up: surface rock)"},
   "ri":{"kind":"material.ice_density","unit":"kg m-3","name":"ice's density (Look up: surface ice)"}},
  "outputs":{"w":{"kind":"body.ice_share","unit":"1","name":"share of the mass that is ice"}},
  "source":{"text":"Showman, A. P. and Malhotra, R. 1999, Science 286, 77","doi":"10.1126/science.286.5437.77","where":"ice and rock shares of the Galilean moons from their densities"},
  "explain":{"curious":"A world lighter than rock for its size must be part ice. Europa is only a little lighter than rock, so it has a thin ice shell; Ganymede and Callisto are about half ice.",
   "science":"Rock and ice are mixed by volume at their low-pressure densities. Self-compression is left out, which is fair for moons and dwarf planets but makes the bigger icy moons come out somewhat drier than they are (Ganymede about 0.3 against about 0.45 from its moment of inertia)."},
  "examples":ex}})
R=8.31446261815324
# Water's lifetime in sunlight at 1 au, from the data sheet (scripts/photo_rows.py).
TAU_H2O=next(x for x in json.load(open('packs/base-reference/pack.json'))["provides"]["reference"]["substances"] if x["id"]=="h2o")["photochemistry"]["lifetime_1au"]["v"]
def loss(p,T,M,g,ve,d,t,ri=RHO_I,tau=TAU_H2O):
    if p<=0 or T<=0 or g<=0 or ri<=0: return 0.0
    flux=p*math.sqrt(M/(2*math.pi*R*T)); v2=2*R*T/M; hop=2*math.sqrt(v2)/g
    rate=1/(tau*d*d) if d>0 and tau>0 else 0.0
    ph=1-math.exp(-hop*rate); lam=ve*ve/v2; j=(1+lam)*math.exp(-lam)
    return flux*min(ph+j,1.0)*max(t,0)/ri
AU=1.495978707e11
ex=[dict(p=p,T=T,M=0.018,g=g,ve=ve,d=d*1.5e11,t=1.4e17,ri=RHO_I,tau=TAU_H2O,L=loss(p,T,0.018,g,ve,d*1.5e11/AU,1.4e17)) for (p,T,g,ve,d) in [(76.0,250.0,1.62,2380.0,1.0),(2.0e-6,130.0,1.24,2440.0,5.2),(1.0e-12,100.0,1.31,2025.0,5.2)]]
atoms.append({"id":"sse.surface.open_ice_loss","version":"1.0.0","cost":"trivial",
 "inputs":[inp("vapour_pressure","p"),inp("temperature","T"),inp("molar_mass","M"),inp("gravity","g"),inp("escape_speed","ve"),inp("star_distance","d"),inp("age","t"),inp("ice_density","ri"),inp("photo_lifetime","tau")],
 "outputs":[inp("ice_lost","L")],
 "params":[],
 "constants":{"R":"sheet/R_gas","au":"sheet/au"},
 "references":[FS09,HUEBNER92,WATSON61],
 "shown":"L = p √(M/2πRT) (1 − e^(−t_hop (au/d)²/τ) + (1+λ)e^(−λ)) t / ρ",
 "equation":{"name":"How much open ice an airless world loses",
  "formula":"flux = p * sqrt(M / (2 * pi * R * T))\nv2 = 2 * R * T / M\nhop = 2 * sqrt(v2) / g\nrate = if d > 0 and tau > 0 then (au / d) * (au / d) / tau else 0\nlam = ve * ve / v2\nshare = min(1 - exp(-hop * rate) + (1 + lam) * exp(-lam), 1)\nL = if p <= 0 or T <= 0 or g <= 0 or ri <= 0 then 0 else flux * share * max(t, 0) / ri",
  "inputs":{"p":{"kind":"material.vapour_pressure","unit":"Pa","name":"the ice's vapour pressure at the ground's temperature (Look up)"},
   "T":{"kind":"planet.surface_temperature","unit":"K","name":"ground temperature"},
   "M":{"kind":"material.molar_mass","unit":"kg mol-1","name":"molar mass of the ice"},
   "g":{"kind":"body.surface_gravity","unit":"m s-2","name":"surface gravity"},
   "ve":{"kind":"body.escape_velocity","unit":"m s-1","name":"escape speed"},
   "d":{"kind":"orbit.star_distance","unit":"m","name":"distance from the star"},
   "t":{"kind":"body.age","unit":"s","name":"time the ice has lain open"},
   "ri":{"kind":"material.ice_density","unit":"kg m-3","name":"ice's density"},
   "tau":{"kind":"material.photo_lifetime_1au","unit":"s","name":"how long a water molecule lasts in sunlight at 1 AU (Look up)"},
   "R":{"kind":"constant.R_gas","unit":"J mol-1 K-1","name":"molar gas constant"},
   "au":{"kind":"constant.au","unit":"m","name":"astronomical unit"}},
  "outputs":{"L":{"kind":"surface.ice_lost","unit":"m","name":"depth of open ice lost"}},
  "source":{"text":"Watson, K., Murray, B. C. and Brown, H. 1961, J. Geophys. Res. 66, 3033","doi":"10.1029/JZ066i009p03033","where":"why ice cannot lie open on the warm Moon"},
  "explain":{"curious":"Ice left out on an airless world slowly turns straight to vapour. Most of the vapour hops and lands again, but some is broken up by sunlight in flight or flies off into space. Near the Sun the Moon would lose a kilometres-thick ice sheet in a blink of geological time; at Jupiter, where ice is far colder, almost nothing is lost, so its moons keep icy ground.",
   "science":"Sublimation flux from the Hertz-Knudsen relation with a sticking coefficient of 1 and the vapour pressure from Fray and Schmitt's fits. Molecules move on ballistic hops of about 2v/g with v the most probable speed; per hop a share 1 − e^(−t_hop J) is photolysed (J the photo rate, one over the molecule's lifetime in sunlight read from the data sheet, falling as 1/d²) and a share (1 + λ)e^(−λ) escapes, λ = v_esc²/v² (Jeans). The rest lands again, so the net loss is the flux times that share. Sputtering and cold-trapping are left out."},
  "examples":ex}})
MOORE99="Moore, J. M. et al. 1999, Icarus 140, 294: mass movement and landform degradation on the icy Galilean satellites; sublimation leaves a dark lag of rock and dust over Callisto's ice, which shows bright in fresh craters. doi:10.1006/icar.1999.6132"
SPENCER87="Spencer, J. R. 1987, Icarus 69, 297: thermal segregation of water ice on the Galilean satellites; sublimed ice leaves a dark lag on the warm ground and frosts the cold. doi:10.1016/0019-1035(87)90107-8"
LAG=7.3e16
def lag(t,tau=LAG): return 0.0 if tau<=0 else 1-math.exp(-max(t,0)/tau)
ex=[dict(t=t,tau=LAG,ri=0.95,rd=0.0572,D=lag(t),R=(1-lag(t))*0.95+lag(t)*0.0572) for t in (1.4e17,6.3e16,1.9e15,0.0)]
atoms.append({"id":"sse.surface.ice_dust_lag","version":"1.0.0","cost":"trivial",
 "inputs":[inp("exposure_age","t"),inp("ice_reflectance","ri"),inp("dust_reflectance","rd")],
 "outputs":[inp("lag_share","D"),inp("ground_reflectance","R")],
 "params":[par("lag_time","tau",LAG,[1.0e15,1.0e18],"time for open ice ground to come within 1/e of a full dark lag (fitted to Callisto and Ganymede's old ground, Moore et al. 1999)")],
 "references":[MOORE99,SPENCER87],
 "shown":"D = 1 − e^(−t/τ), R = (1 − D) R_ice + D R_dust",
 "equation":{"name":"How old icy ground goes dark",
  "formula":"D = if tau <= 0 then 0 else 1 - exp(-max(t, 0) / tau)\nR = (1 - D) * ri + D * rd",
  "inputs":{"t":{"kind":"surface.exposure_age","unit":"s","name":"time the ground has lain open"},
   "ri":{"kind":"material.reflectance","unit":"1","name":"clean ice's reflectance (Look up)"},
   "rd":{"kind":"material.reflectance","unit":"1","name":"dark dust's reflectance (Look up)"},
   "tau":{"kind":"model.lag_time","unit":"s","name":"time to build a dark lag"}},
  "outputs":{"D":{"kind":"surface.lag_share","unit":"1","name":"share of the ground under a dark lag"},
   "R":{"kind":"surface.ground_reflectance","unit":"1","name":"reflectance of the ground"}},
  "source":{"text":"Spencer, J. R. 1987, Icarus 69, 297","doi":"10.1016/0019-1035(87)90107-8","where":"the dark lag left on icy ground"},
  "explain":{"curious":"Icy moons are not all white. Old icy ground slowly gathers a dark skin of dust and rock left behind as the ice turns to vapour and grains rain in from space. Young ground, like Europa's or a fresh crater's, is still clean bright ice; Callisto's ancient ground is dark.",
   "science":"The lag's areal share approaches 1 exponentially with exposure age, with one time fitted to Callisto and Ganymede; the ground's reflectance mixes clean ice and dark dust by area. Sublimation's dependence on temperature and the infall rate's on distance are folded into the one fitted time."},
  "examples":ex}})

p='packs/base-physics/formulas/surface.json'
d={"format":"sse.formulas/1","family":"sse.surface","about":"Physics blocks whose maths is a formula (ND3b): each keeps its id, ports and results, and its maths is the Equation below, with a name that says what happens.","atoms":atoms}
open(p,'w').write(json.dumps(d,indent=1,ensure_ascii=False)+"\n")
print("ok",[ (round(e.get('M',0),4)) for e in atoms[0]['equation']['examples']])
