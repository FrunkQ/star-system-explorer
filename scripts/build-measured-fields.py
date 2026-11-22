#!/usr/bin/env python3
"""MF0: the measured-fields fixture, tests/fixtures/magnetic/measured-fields.json.

Every value below was read from the source named beside it (22 Nov 2026,
MF0; handover/packages/MF0.md lists how). Worked-out values say so. SI units.
Run with no arguments to write the fixture, or --check to fail if it is stale.
Python standard library only.
"""
import json
import os
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
OUT = os.path.join(ROOT, "tests", "fixtures", "magnetic", "measured-fields.json")
NS="https://nssdc.gsfc.nasa.gov/planetary/factsheet/%sfact.html"
def dip(name,g,r_km,tilt,off,model):
    b=g*1e-4; r=r_km*1e3
    return {"id":name.lower(),"name":name,"kind":"planet","field":"global dipole",
      "dipole_surface_equatorial_t":b,"reference_radius_m":r,
      "moment_a_m2":float("%.4g"%(b*r**3*1e7)),"moment_note":"worked out: m = 4π B0 R³ / μ0 = B0 R³ × 1e7 (SI)",
      "tilt_deg":tilt,"offset_radii":off,"model":model,
      "src":{"what":"NASA NSSDCA %s Fact Sheet, Magnetospheric Parameters"%name,"url":NS%name.lower()},"grade":"measured"}
bodies=[
 dip("Mercury",0.002,2440,0.0,0.17,"MESSENGER 2010"),
 {"id":"venus","name":"Venus","kind":"planet","field":"none (no global dipole)","moment_a_m2":0.0,"src":{"what":"NASA NSSDCA Venus Fact Sheet: no magnetospheric parameters listed","url":NS%"venus"},"grade":"measured","note":"the fact sheet lists no magnetosphere; Venus has only an induced magnetosphere"},
 dict(dip("Earth",0.306,6378,9.35,0.076,"GSFC-1283"),tilt_note="worked out: 90° − 80.65°, the geocentric dipole pole latitude the fact sheet gives (WMM2020)"),
 {"id":"mars","name":"Mars","kind":"planet","field":"crustal only","moment_a_m2":0.0,"crustal_max_t_at_100km":1.6e-6,"largest_source_moment_a_m2":1.3e17,
  "src":{"what":"Acuña, M. H. et al. 1999, Science 284, 790: global distribution of crustal magnetization discovered by the Mars Global Surveyor MAG/ER experiment","doi":"10.1126/science.284.5415.790"},"also":{"what":"NASA NSSDCA Mars Fact Sheet: no magnetospheric parameters listed","url":NS%"mars"},"grade":"measured"},
 dip("Jupiter",4.30,71398,9.4,0.119,"GSFC-O6"),
 dip("Saturn",0.215,60330,0.0,0.038,"GSFC-Z3"),
 dip("Uranus",0.228,25600,58.6,0.352,"GSFC-O3"),
 dip("Neptune",0.142,24765,46.9,0.485,"GSFC-O8"),
 {"id":"ganymede","name":"Ganymede","kind":"moon","field":"intrinsic dipole","dipole_surface_equatorial_t":719e-9,"tilt_deg":176,
  "src":{"what":"Kivelson, M. G., Khurana, K. K. and Volwerk, M. 2002, Icarus 157, 507: the permanent and inductive magnetic moments of Ganymede","doi":"10.1006/icar.2002.6834"},"grade":"measured"},
 {"id":"europa","name":"Europa","kind":"moon","field":"induced by Jupiter's rotating field","induced_dipole_surface_t":126e-9,"induced_ideal_t":104e-9,
  "src":{"what":"Kivelson, M. G. et al. 2000, Science 289, 1340: Galileo magnetometer measurements, a stronger case for a subsurface ocean at Europa (Table 1, E26 flyby)","doi":"10.1126/science.289.5483.1340"},"grade":"measured","note":"fitted internal dipole My = −126 nT; an ideal induced dipole for a perfect conductor gives −104 nT; moments stated in field units at the surface"},
 {"id":"callisto","name":"Callisto","kind":"moon","field":"induced by Jupiter's rotating field","induced_moment_a_m2":2.22e18,"response_fraction":[0.7,1.0],
  "src":{"what":"Zimmer, C., Khurana, K. K. and Kivelson, M. G. 2000, Icarus 147, 329: subsurface oceans on Europa and Callisto, constraints from Galileo magnetometer observations","doi":"10.1006/icar.2000.6456"},"also":{"what":"Liuzzo, L. et al. 2017, JGR Space Physics, quoting Zimmer et al. 2000 for the C3 flyby moment","doi":"10.1002/2017JA024303"},"grade":"measured","check":"secondary","note":"values read from works quoting Zimmer et al. 2000, not from the paper itself"},
 {"id":"moon","name":"Moon","kind":"moon","field":"crustal only","moment_a_m2":0.0,"crustal_max_surface_t":718e-9,
  "src":{"what":"Tsunakawa, H. et al. 2015, JGR Planets 120, 1160: surface vector mapping of magnetic anomalies over the Moon using Kaguya and Lunar Prospector observations","doi":"10.1002/2014JE004785"},"grade":"measured","note":"largest anomaly near the Crisium antipode"},
 {"id":"sun","name":"Sun","kind":"star","field":"cycling dynamo","polar_field_at_minimum_t":[5e-4,1e-3],"umbral_typical_t":0.3,"umbral_record_t":0.625,
  "src":{"what":"Petrie, G. J. D. 2015, Living Rev. Solar Phys. 12, 5: solar magnetism in the polar regions","doi":"10.1007/lrsp-2015-5"},
  "also":{"what":"Okamoto, T. J. and Sakurai, T. 2018, ApJL 852, L16: super-strong magnetic field in sunspots (6250 G; typical umbra about 3000 G)","doi":"10.3847/2041-8213/aaa3d8"},"grade":"measured"},
]
R22={"what":"Reiners, A., Shulyak, D. et al. 2022, A&A 662, A41: magnetism, rotation, and nonthermal emission in cool stars, average magnetic field measurements in 292 M dwarfs (Table B.1, VizieR J/A+A/662/A41)","doi":"10.1051/0004-6361/202243251","url":"https://vizier.cfa.harvard.edu/viz-bin/asu-tsv?-source=J/A%2BA/662/A41/tableb1"}
def cool(i,n,b,per,tau,sp):
    return {"id":i,"name":n,"kind":"star","spectral_type":sp,"field":"mean surface field","mean_surface_t":b*0.1,"rotation_d":per,"convective_turnover_d":tau,
            "rossby":float("%.2g"%(per/tau)),"rossby_note":"worked out: Per / tau from the table","src":R22,"grade":"measured"}
bodies+= [
 cool("wx-uma","WX UMa (GJ 412 B)",6.88,0.8,398,"M6.0 V"),
 cool("yz-cmi","YZ CMi (GJ 285)",4.54,2.8,116,"M4.5"),
 cool("ad-leo","AD Leo (GJ 388)",3.57,2.2,80,"M3.5e V"),
 cool("ev-lac","EV Lac (GJ 873)",4.32,4.3,110,"M3.5"),
 cool("barnards-star","Barnard's Star (GJ 699)",0.43,148.6,207,"M4.0 V"),
 cool("hd-165222","HD 165222 (GJ 701)",0.27,127.8,67,"M0 V"),
 {"id":"hd-215441","name":"HD 215441 (Babcock's star)","kind":"star","spectral_type":"Ap","field":"fossil","mean_surface_t":3.44,"dipole_polar_t":6.7,"dipole_polar_note":"dipole term of a dipole + quadrupole (−55 kG) + octupole (+30 kG) fit",
  "src":{"what":"Babcock, H. W. 1960, ApJ 132, 521: the 34-kilogauss magnetic field of HD 215441","doi":"10.1086/146960"},"also":{"what":"Landstreet, J. D. et al. 1989, ApJ 344, 876: magnetic field models of HD 215441","doi":"10.1086/167855"},"grade":"measured"},
 {"id":"hd-191612","name":"HD 191612","kind":"star","spectral_type":"O6–8 f?p","field":"fossil","dipole_polar_t":0.15,"dipole_polar_note":"about −1.5 kG",
  "src":{"what":"Donati, J.-F. et al. 2006, MNRAS 365, L6: the magnetic field and wind confinement of HD 191612","doi":"10.1111/j.1745-3933.2005.00115.x"},"grade":"measured"},
 {"id":"wd-2047-372","name":"WD 2047+372","kind":"white_dwarf","field":"fossil (weak)","mean_surface_t":5.69,
  "src":{"what":"Landstreet, J. D., Bagnulo, S., Martin, A. and Valyavin, G. 2016, A&A 591, A80: discovery of an extremely weak magnetic field in the white dwarf LTT 16093 = WD 2047+372","doi":"10.1051/0004-6361/201628488"},"grade":"measured","note":"56.9 kG"},
 {"id":"grw-70-8247","name":"Grw +70°8247","kind":"white_dwarf","field":"fossil (strong)","field_range_t":[1.0e4,3.2e4],
  "src":{"what":"Ferrario, L., de Martino, D. and Gänsicke, B. T. 2015, Space Sci. Rev. 191, 111: magnetic white dwarfs (section 2.1)","url":"https://arxiv.org/abs/1504.08072"},"also":{"what":"Bagnulo, S. and Landstreet, J. D. 2019, MNRAS: about 400 MG","doi":"10.1093/mnras/stz1121"},"grade":"measured","note":"100 to 320 MG; models differ"},
 {"id":"pg-1031-234","name":"PG 1031+234","kind":"white_dwarf","field":"fossil (strongest class)","field_min_t":5.0e4,
  "src":{"what":"Schmidt, G. D. et al. 1986, ApJ 309, 218: a magnetic white dwarf with a field over 500 MG","doi":"10.1086/164593"},"grade":"measured","note":"lower bound from the paper's title"},
]
ATNF={"what":"ATNF Pulsar Catalogue (Manchester, R. N. et al. 2005, AJ 129, 1993), VizieR copy B/psr/psr; BSURF = 3.2e19 (P Pdot)^½ G","doi":"10.1086/428488","url":"https://vizier.cfa.harvard.edu/viz-bin/asu-tsv?-source=B/psr/psr"}
def psr(i,n,p,pd,b):
    return {"id":i,"name":n,"kind":"pulsar","field":"spin-down dipole","period_s":p,"period_dot":pd,"surface_dipole_t":b*1e-4,"src":ATNF,"grade":"computed","note":"field inferred from spin-down, the standard magnetic-dipole estimate"}
bodies+=[
 psr("crab","Crab pulsar (B0531+21)",0.0333924123,4.20972e-13,3.79e12),
 psr("vela","Vela pulsar (B0833−45)",0.089328385024,1.25008e-13,3.38e12),
 psr("j0437-4715","PSR J0437−4715",0.00575745193671,5.729215e-20,5.81e8),
 {"id":"sgr-1806-20","name":"SGR 1806−20","kind":"magnetar","field":"spin-down dipole","period_s":7.55,"surface_dipole_t":2.0e11,
  "src":{"what":"Kaspi, V. M. and Beloborodov, A. M. 2017, ARA&A 55, 261: magnetars (Table 1, from the McGill catalogue, Olausen and Kaspi 2014)","doi":"10.1146/annurev-astro-081915-023329"},"grade":"computed","note":"2.0e15 G"},
]
out={"about":"Measured magnetic fields for MF tests (MF0). Every value was read from the source named, 22 Nov 2026; worked-out values say so. SI units (tesla, A m², metres). Grade 'computed' means the field is inferred from spin-down, not measured directly.","bodies":bodies}
text = json.dumps(out, indent=1, ensure_ascii=False, allow_nan=False) + "\n"
if "--check" in sys.argv:
    cur = open(OUT, encoding="utf-8").read() if os.path.exists(OUT) else ""
    if cur != text:
        print("measured-fields.json is stale: run scripts/build-measured-fields.py")
        sys.exit(1)
    print("measured-fields.json is current (%d bodies)" % len(bodies))
else:
    with open(OUT, "w", encoding="utf-8", newline="\n") as fh:
        fh.write(text)
    print("wrote %s (%d bodies)" % (os.path.relpath(OUT, ROOT), len(bodies)))
