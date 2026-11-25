# Results 00: from Alex's Consensus pass on the visual polish note (24 Nov 2026)

Scored from handover/design/SSE4-visual-polish-citation-check.md (PR 464), items 1, 2, 3, 5, 6 and 17. Only papers on
the master list are scored here; the check's other findings are about claims, not papers on the list.

cox1954 | confirmed | Cox, C. and Munk, W. 1954, J. Opt. Soc. Am. 44(11), 838-850, doi 10.1364/JOSA.44.000838 | upwind slope variance 3.16e-3 W (clean surface) | secondary sources x2, primary PDF unread; slick-surface formulas stay from memory (VP check item 1)
hillaire2020 | confirmed | Hillaire, S. 2020, Computer Graphics Forum 39(4), 13-22 (EGSR 2020), doi 10.1111/cgf.14050 | multi-scattering LUT sky model | primary (VP check item 2)
jendersie2023 | confirmed | Jendersie, J. and d'Eon, E. 2023, ACM SIGGRAPH 2023 Talks, doi 10.1145/3587421.3595409 | approximate Mie phase (Draine 2003 form) | primary; the VP check found the fit used outside its valid range in the note; ACM article number open (item 3)
zeipel1924 | confirmed | von Zeipel, H. 1924, MNRAS 84(9), 665-684, doi 10.1093/mnras/84.9.665, and the companion MNRAS 84, 684-701 | T_eff ∝ g^0.25 for radiative envelopes | cite both parts; a fixed 0.25 is wrong for fast rotators (Espinosa Lara and Rieutord 2011: exponent depends on rotation; Altair 0.19, Vega 0.23) (item 5)
lucy1967 | confirmed | Lucy, L. B. 1967, Zeitschrift für Astrophysik 65, 89 | β = 0.08 for convective envelopes (0.32 in the flux convention) | secondary (item 5)
lara2011 | confirmed | Espinosa Lara, F. and Rieutord, M. 2011, A&A 533, A43, doi 10.1051/0004-6361/201117252 | gravity darkening exponent depends on rotation, 0.25 only for slow rotators | primary (item 5)
rhines1975 | says something else | Rhines, P. B. 1975, J. Fluid Mech. 69(3), 417-443, doi 10.1017/S0022112075001504 | L = π√(2U/β), β = 2Ω cos φ / a | π√(2U/β) is one jet's width, not the spacing of eastward jets, and it fits Jupiter and Saturn within a factor 2 only with U the eddy speed (10-40 m/s), not the 100 m/s jet speed; check which U each place uses (item 6)
bridson2007 | confirmed | Bridson, R., Hourihan, J. and Nordenstam, M. 2007, ACM Trans. Graph. 26(3), 46 (SIGGRAPH 2007), doi 10.1145/1276377.1276435 | divergence-free noise as the curl of a potential | "Houriham" in the ACM record is a typo (item 17)
rendering2015 | confirmed | Schneider, A. 2015, 2017, 2022, 2023, Nubis talks, Advances in Real-Time Rendering in Games, ACM SIGGRAPH Courses (2022 doi 10.1145/3532720.3546903) | volumetric cloud method | course talks, not peer-reviewed; the list's key "rendering2015" stands for these talks (item 17)
