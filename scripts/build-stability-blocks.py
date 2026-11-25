#!/usr/bin/env python3
"""Write packs/base-physics/formulas/stability.json: the orbital stability
checks (package ST1) as formula blocks, each a visible Equation with its
source, its fitted numbers as Model numbers and worked examples worked out
here by the same formula.

Run: python3 scripts/build-stability-blocks.py
"""
import json
import math
import os

G = 6.6743e-11
HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "..", "packs", "base-physics", "formulas", "stability.json")

# Round figures for the worked examples (the blocks read nothing typed
# here; the sheet's own values never appear as typed copies).
M_SUN = 2.0e30
M_EARTH = 6.0e24
M_JUP = 1.9e27
AU = 1.5e11

GLADMAN = "Gladman, B. 1993, Dynamics of systems of two close planets, Icarus 106, 247. doi:10.1006/icar.1993.1169"
CHAMBERS = "Chambers, J. E., Wetherill, G. W. and Boss, A. P. 1996, The stability of multi-planet systems, Icarus 119, 261. doi:10.1006/icar.1996.0019"
PU_WU = "Pu, B. and Wu, Y. 2015, Spacing of Kepler planets: sculpting by dynamical instability, ApJ 807, 44: Kepler multis sit at about 10 mutual Hill radii and closer systems do not last. doi:10.1088/0004-637X/807/1/44"
LASKAR97 = "Laskar, J. 1997, Large scale chaos and the spacing of the inner planets, A&A 317, L75: the angular momentum deficit"
LP17 = "Laskar, J. and Petit, A. C. 2017, AMD-stability and the classification of planetary systems, A&A 605, A72: eqs. 26 to 39 (collision condition, relative AMD C/Λ', critical AMD), eq. 58 (β). doi:10.1051/0004-6361/201630022"
PLB17 = "Petit, A. C., Laskar, J. and Boué, G. 2017, AMD-stability in the presence of first-order mean motion resonances, A&A 607, A35: eq. 29 (r), eq. 89 (g), eq. 97 (critical AMD where resonances overlap), eqs. 100 and 101 (where it applies). doi:10.1051/0004-6361/201731196"
DWY06 = "Domingos, R. C., Winter, O. C. and Yokoyama, T. 2006, Stable satellites around extrasolar giant planets, MNRAS 373, 1227: critical semi-major axes for prograde and retrograde moons. doi:10.1111/j.1365-2966.2006.11104.x"
KOZAI = "Kozai, Y. 1962, Secular perturbations of asteroids with high inclination and eccentricity, AJ 67, 591. doi:10.1086/108790"
LIDOV = "Lidov, M. L. 1962, The evolution of orbits of artificial satellites of planets under the action of gravitational perturbations of external bodies, Planet. Space Sci. 9, 719. doi:10.1016/0032-0633(62)90129-0"
KEM98 = "Kiseleva, L. G., Eggleton, P. P. and Mikkola, S. 1998, Tidal friction in triple stars, MNRAS 300, 292: the Kozai cycle time. doi:10.1046/j.1365-8711.1998.01903.x"
MD99 = "Murray, C. D. and Dermott, S. F. 1999, Solar System Dynamics, Cambridge University Press: section 8.7, the width of a mean motion resonance (eq. 8.76 and its large-eccentricity limit; Table 8.5 for the strengths alpha f_d). doi:10.1017/CBO9781139174817"
DM81 = "Dermott, S. F. and Murray, C. D. 1981, The dynamics of tadpole and horseshoe orbits I. Theory, Icarus 48, 1: tadpoles within (8 mu / 3)^(1/2) of the shared orbit, horseshoes out to about the Hill scale. doi:10.1016/0019-1035(81)90147-0"
WISDOM80 = "Wisdom, J. 1980, The resonance overlap criterion and the onset of stochastic behavior in the restricted three-body problem, AJ 85, 1122: first-order resonances overlap within 1.3 mu^(2/7) of a planet's orbit. doi:10.1086/112778"
MW12 = "Mustill, A. J. and Wyatt, M. C. 2012, A general model of planetary chaotic zones, MNRAS 419, 3074: for stretched orbits the overlap zone is 1.8 (mu e)^(1/5). doi:10.1111/j.1365-2966.2011.19951.x"
HW99 = "Holman, M. J. and Wiegert, P. A. 1999, Long-term stability of planets in binary systems, AJ 117, 621: eqs. 3 (around one star) and 4 (around both), fitted for 0.1 <= mu <= 0.9 and 0 <= e <= 0.8. doi:10.1086/300695"


def model(v, grade, src):
    return {"value": v, "grade": grade, "source": src}


def inp(kind, unit, name, valid=None):
    d = {"kind": kind, "unit": unit, "name": name}
    if valid is not None:
        d["valid"] = valid
    return d


VERDICT = inp("orbit.stability_verdict", "1", "")


def verdict(name):
    d = dict(VERDICT)
    d["name"] = name
    return d


# ------------------------------------------------------------ Hill spacing


def hill(a1, e1, m1, a2, e2, m2, M, lasting=10.0):
    rh = (m1 + m2) / (3 * M)
    rh = rh ** (1 / 3) * (a1 + a2) / 2
    spacing = (a2 - a1) / rh
    gap = (a2 * (1 - e2) - a1 * (1 + e1)) / rh
    if gap <= 0:
        v = 3
    elif spacing < 2 * math.sqrt(3):
        v = 2
    elif spacing < lasting:
        v = 1
    else:
        v = 0
    return {"rh": rh, "spacing": spacing, "gap": gap, "verdict": v}


def hill_atom():
    exs = []
    for args in [
        (AU, 0.0167, M_EARTH, 1.523679 * AU, 0.0934, 0.6417e24, M_SUN),
        (0.1 * AU, 0.0, 5 * M_EARTH, 0.105 * AU, 0.0, 5 * M_EARTH, M_SUN),
        (5.2 * AU, 0.3, M_JUP, 6.0 * AU, 0.3, M_JUP, M_SUN),
    ]:
        r = hill(*args)
        exs.append({"a1": args[0], "e1": args[1], "m1": args[2], "a2": args[3], "e2": args[4], "m2": args[5], "M": args[6],
                    "rh": r["rh"], "spacing": r["spacing"], "gap": r["gap"], "verdict": r["verdict"]})
    return {
        "id": "sse.stability.hill_spacing",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "inner_semi_major_axis", "letter": "a1"},
            {"port": "inner_eccentricity", "letter": "e1"},
            {"port": "inner_mass", "letter": "m1"},
            {"port": "outer_semi_major_axis", "letter": "a2"},
            {"port": "outer_eccentricity", "letter": "e2"},
            {"port": "outer_mass", "letter": "m2"},
            {"port": "host_mass", "letter": "M"},
        ],
        "outputs": [
            {"port": "mutual_hill_radius", "letter": "rh"},
            {"port": "spacing", "letter": "spacing"},
            {"port": "closest_gap", "letter": "gap"},
            {"port": "verdict", "letter": "verdict"},
        ],
        "models": {"lasting": model(10.0, "fitted", PU_WU)},
        "positive": [
            {"port": "inner_semi_major_axis", "says": "inner semi-major axis"},
            {"port": "inner_mass", "says": "inner mass"},
            {"port": "outer_semi_major_axis", "says": "outer semi-major axis"},
            {"port": "outer_mass", "says": "outer mass"},
            {"port": "host_mass", "says": "host mass"},
        ],
        "references": [GLADMAN, CHAMBERS, PU_WU],
        "shown": "Δ = (a₂ − a₁) / R_H,  R_H = ((m₁ + m₂) / 3M)^⅓ (a₁ + a₂)/2",
        "equation": {
            "name": "How many shared gravity bubbles apart two neighbours are",
            "formula": "rh = cbrt((m1 + m2) / (3 * M)) * (a1 + a2) / 2\n"
                       "spacing = (a2 - a1) / rh\n"
                       "gap = (a2 * (1 - e2) - a1 * (1 + e1)) / rh\n"
                       "verdict = if gap <= 0 then 3 else if spacing < 2 * sqrt(3) then 2 else if spacing < lasting then 1 else 0",
            "inputs": {
                "a1": inp("orbit.semi_major_axis", "m", "size of the inner orbit"),
                "e1": inp("orbit.eccentricity", "1", "how stretched the inner orbit is", [0, 0.99]),
                "m1": inp("body.mass", "kg", "mass of the inner world"),
                "a2": inp("orbit.semi_major_axis", "m", "size of the outer orbit"),
                "e2": inp("orbit.eccentricity", "1", "how stretched the outer orbit is", [0, 0.99]),
                "m2": inp("body.mass", "kg", "mass of the outer world"),
                "M": inp("orbit.host_mass", "kg", "mass of what both circle"),
                "lasting": inp("model.hill_spacing_lasting", "1", "spacing, in shared bubbles, that systems keep for billions of years"),
            },
            "outputs": {
                "rh": inp("orbit.mutual_hill_radius", "m", "radius of the pair's shared gravity bubble"),
                "spacing": inp("orbit.hill_spacing", "1", "gap between the orbits, in shared bubbles"),
                "gap": inp("orbit.hill_gap", "1", "inner's farthest point to outer's nearest, in shared bubbles"),
                "verdict": verdict("0 lasts, 1 may scatter over long times, 2 too close to last, 3 the orbits cross"),
            },
            "source": {
                "text": "Gladman 1993: two planets on near-circular orbits can never meet when more than 2√3 mutual Hill radii apart. Chambers, Wetherill and Boss 1996, and Pu and Wu 2015: more than two planets need about 10 to last billions of years",
                "doi": "10.1006/icar.1993.1169",
                "where": "Gladman 1993 eq. 30 (Hill stability); Pu and Wu 2015 section 4",
            },
            "explain": {
                "curious": "Two worlds can share a star only if they keep far enough apart. Measured in the size of their shared gravity bubble, fewer than about 3.5 means they will meet; about 10 or more and they can last for billions of years.",
                "science": "Mutual Hill radius R_H = ((m₁+m₂)/3M)^⅓ (a₁+a₂)/2 and spacing Δ = (a₂−a₁)/R_H. Δ < 2√3 is Hill-unstable for two planets (Gladman 1993); multi-planet systems go unstable on times growing with Δ (Chambers et al. 1996), and Δ ≈ 10 survives Gyr (Pu and Wu 2015). The closest gap uses the inner apoapsis and the outer periapsis.",
            },
            "examples": exs,
        },
    }


# ------------------------------------------------------- angular momentum deficit


def amd_term(a, e, i, m, M):
    return m * math.sqrt(G * M * a) * (1 - math.sqrt(1 - e * e) * math.cos(i))


def amd_term_atom():
    exs = []
    for a, e, i, m in [(AU, 0.0167, 0.0, M_EARTH), (5.2 * AU, 0.0484, 0.0227, M_JUP), (0.387 * AU, 0.2056, 0.1222, 3.301e23)]:
        exs.append({"a": a, "e": e, "i": i, "m": m, "M": M_SUN, "amd": amd_term(a, e, i, m, M_SUN), "tolerance": 1e-6})
    return {
        "id": "sse.stability.amd_term",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "semi_major_axis", "letter": "a"},
            {"port": "eccentricity", "letter": "e"},
            {"port": "inclination", "letter": "i"},
            {"port": "body_mass", "letter": "m"},
            {"port": "host_mass", "letter": "M"},
        ],
        "outputs": [{"port": "amd", "letter": "amd"}],
        "constants": {"G": "constants/G"},
        "positive": [
            {"port": "semi_major_axis", "says": "semi-major axis"},
            {"port": "body_mass", "says": "body mass"},
            {"port": "host_mass", "says": "host mass"},
        ],
        "checks": [{"port": "eccentricity", "min": 0.0, "below": 1.0, "says": "eccentricity {} is not an ellipse"}],
        "references": [LASKAR97, LP17],
        "shown": "AMD = m √(G M a) (1 − √(1 − e²) cos i)",
        "equation": {
            "name": "Spin a world's orbit has lost by being stretched and tilted",
            "formula": "amd = m * sqrt(G * M * a) * (1 - sqrt(1 - e * e) * cos(i))",
            "inputs": {
                "G": inp("constant.G", "m3 kg-1 s-2", "gravitational constant"),
                "a": inp("orbit.semi_major_axis", "m", "size of the orbit"),
                "e": inp("orbit.eccentricity", "1", "how stretched the orbit is", [0, 0.99]),
                "i": inp("orbit.inclination", "1", "tilt of the orbit, radians"),
                "m": inp("body.mass", "kg", "mass of the world"),
                "M": inp("orbit.host_mass", "kg", "mass of what it circles"),
            },
            "outputs": {"amd": inp("orbit.angular_momentum_deficit", "kg m2 s-1", "spin lost against a flat circle")},
            "source": {
                "text": "Laskar 1997; Laskar and Petit 2017: the angular momentum deficit, the spin a system of orbits lacks against flat circles. Planets trade it, but its total is kept, so it caps how stretched any orbit can become",
                "doi": "10.1051/0004-6361/201630022",
                "where": "Laskar and Petit 2017 eq. 2",
            },
            "explain": {
                "curious": "A circular, flat orbit carries the most spin an orbit of that size can. Stretching or tilting it costs some. Worlds pass this cost between them, but the total stays the same, so it limits how wild any one orbit can get.",
                "science": "Λ (1 − √(1−e²) cos i), Λ = m √(GMa) (light planet). The sum over planets is conserved by the secular dynamics (Laskar 1997).",
            },
            "examples": exs,
        },
    }


def collision_step(e, a1, a2, m1, m2):
    al = a1 / a2
    ga = m1 / m2
    d = al * (1 - e * e) + ga * ga * e * e
    f = al * e + ga * e / math.sqrt(d) - 1 + al
    df = al + ga * al / (d * math.sqrt(d))
    return min(max(e - f / df, 0.0), 1.0)


STEPS = 16


def collision_root(a1, a2, m1, m2):
    e = 0.0
    for _ in range(STEPS):
        e = collision_step(e, a1, a2, m1, m2)
    return e


def collision_step_atom():
    exs = []
    for e, a1, a2, m1, m2 in [(0.0, 0.723, 1.0, 4.9, 6.0), (0.1, 0.5, 1.0, 1.0, 1.0), (0.3, 0.9, 1.0, 1.0, 300.0)]:
        exs.append({"e": e, "a1": a1 * AU, "a2": a2 * AU, "m1": m1 * 1e24, "m2": m2 * 1e24, "next": collision_step(e, a1, a2, m1, m2)})
    return {
        "id": "sse.stability.amd_collision_step",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "eccentricity_guess", "letter": "e"},
            {"port": "inner_semi_major_axis", "letter": "a1"},
            {"port": "outer_semi_major_axis", "letter": "a2"},
            {"port": "inner_mass", "letter": "m1"},
            {"port": "outer_mass", "letter": "m2"},
        ],
        "outputs": [{"port": "eccentricity_better", "letter": "next"}],
        "positive": [
            {"port": "inner_semi_major_axis", "says": "inner semi-major axis"},
            {"port": "outer_semi_major_axis", "says": "outer semi-major axis"},
            {"port": "inner_mass", "says": "inner mass"},
            {"port": "outer_mass", "says": "outer mass"},
        ],
        "references": [LP17],
        "shown": "e ← e − F/F′,  F = αe + γe/√(α(1−e²)+γ²e²) − 1 + α",
        "equation": {
            "name": "One step closer to the cheapest way two orbits can touch",
            "formula": "al = a1 / a2\n"
                       "ga = m1 / m2\n"
                       "d = al * (1 - e * e) + ga * ga * e * e\n"
                       "f = al * e + ga * e / sqrt(d) - 1 + al\n"
                       "df = al + ga * al / (d * sqrt(d))\n"
                       "next = clamp(e - f / df, 0, 1)",
            "inputs": {
                "e": inp("orbit.eccentricity", "1", "the inner orbit's stretch so far", [0, 1]),
                "a1": inp("orbit.semi_major_axis", "m", "size of the inner orbit"),
                "a2": inp("orbit.semi_major_axis", "m", "size of the outer orbit"),
                "m1": inp("body.mass", "kg", "mass of the inner world"),
                "m2": inp("body.mass", "kg", "mass of the outer world"),
            },
            "outputs": {"next": inp("orbit.eccentricity", "1", "a better guess")},
            "source": {
                "text": "Laskar and Petit 2017: the inner eccentricity at which two orbits first touch for the least deficit is the single root in [0, 1] of F(e) = 0; F rises with e, so Newton steps from e = 0 close in on it",
                "doi": "10.1051/0004-6361/201630022",
                "where": "eq. 35 (the collision equation), eqs. 36 and 37 (one root)",
            },
            "explain": {
                "curious": "To find the cheapest way for two orbits to touch, the app guesses how stretched the inner one must be and improves the guess, the same step each time, until it stops moving.",
                "science": "Newton's method on Laskar and Petit's eq. 35, F′(e) = α + γα/D^{3/2} with D = α(1−e²)+γ²e². Sixteen steps from e = 0 reach the root to about 1e−15 for 0.01 ≤ α ≤ 0.99 and 1e−4 ≤ γ ≤ 1e4.",
            },
            "examples": exs,
        },
    }


R_MMR = 0.80199


def amd_critical(e, a1, a2, m1, m2, M, amd, r=R_MMR):
    al = a1 / a2
    ga = m1 / m2
    crel = amd / (m2 * math.sqrt(G * M * a2))
    ep = 1 - al - al * e
    ccoll = ga * math.sqrt(al) * (1 - math.sqrt(1 - e * e)) + 1 - math.sqrt(1 - ep * ep)
    eps = (m1 + m2) / M
    g = 3 ** 4 * (1 - al) ** 5 / (2 ** 9 * r * eps) - 32 * r * eps / (9 * (1 - al) ** 2)
    edge = 4 / 3 * (2 * r * eps) ** (1 / 4) + math.sqrt(2 * r * eps) / 4
    if 1 - al < edge:
        crit = g * g / 2 * ga * math.sqrt(al) / (1 + ga * math.sqrt(al)) if g > 0 else 0.0
    else:
        crit = ccoll
    v = 0 if crel < crit else 1
    return {"crel": crel, "ccoll": ccoll, "crit": crit, "verdict": v}


def amd_critical_atom():
    exs = []
    planets = [(0.723, 0.0068, 0.0592, 4.9e24), (1.0, 0.0167, 0.0, 6.0e24)]
    amd = sum(amd_term(a * AU, e, i, m, M_SUN) for a, e, i, m in planets)
    for (a1, m1, a2, m2, M, A) in [
        (0.723 * AU, 4.9e24, 1.0 * AU, 6.0e24, M_SUN, amd),
        (5.2 * AU, M_JUP, 5.6 * AU, M_JUP, M_SUN, 1e40),
        (1.0 * AU, M_EARTH, 1.5 * AU, M_EARTH, M_SUN, 1e30),
    ]:
        e = collision_root(a1, a2, m1, m2)
        r = amd_critical(e, a1, a2, m1, m2, M, A)
        exs.append({"e": e, "a1": a1, "a2": a2, "m1": m1, "m2": m2, "M": M, "amd": A,
                    "crel": r["crel"], "ccoll": r["ccoll"], "crit": r["crit"], "verdict": r["verdict"], "tolerance": 1e-6})
    return {
        "id": "sse.stability.amd_critical",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "collision_eccentricity", "letter": "e"},
            {"port": "inner_semi_major_axis", "letter": "a1"},
            {"port": "outer_semi_major_axis", "letter": "a2"},
            {"port": "inner_mass", "letter": "m1"},
            {"port": "outer_mass", "letter": "m2"},
            {"port": "host_mass", "letter": "M"},
            {"port": "system_amd", "letter": "amd"},
        ],
        "outputs": [
            {"port": "relative_amd", "letter": "crel"},
            {"port": "collision_amd", "letter": "ccoll"},
            {"port": "critical_amd", "letter": "crit"},
            {"port": "verdict", "letter": "verdict"},
        ],
        "constants": {"G": "constants/G"},
        "models": {"r": model(R_MMR, "derived", "Petit, Laskar and Boué 2017 eq. 29: r = (K1(2/3) + 2 K0(2/3))/π")},
        "positive": [
            {"port": "inner_semi_major_axis", "says": "inner semi-major axis"},
            {"port": "outer_semi_major_axis", "says": "outer semi-major axis"},
            {"port": "inner_mass", "says": "inner mass"},
            {"port": "outer_mass", "says": "outer mass"},
            {"port": "host_mass", "says": "host mass"},
        ],
        "checks": [{"port": "outer_semi_major_axis", "above": "inner_semi_major_axis", "says": "the outer orbit {} must be wider than the inner"}],
        "references": [LP17, PLB17],
        "shown": "β = C/(Λ′ C_c) < 1",
        "equation": {
            "name": "Whether two neighbours have enough spare stretch to collide",
            "formula": "al = a1 / a2\n"
                       "gs = m1 / m2 * sqrt(al)\n"
                       "crel = amd / (m2 * sqrt(G * M * a2))\n"
                       "ep = 1 - al - al * e\n"
                       "ccoll = gs * (1 - sqrt(1 - e * e)) + 1 - sqrt(1 - ep * ep)\n"
                       "re = r * (m1 + m2) / M\n"
                       "g = 3 ^ 4 * (1 - al) ^ 5 / (2 ^ 9 * re) - 32 * re / (9 * (1 - al) ^ 2)\n"
                       "edge = 4 / 3 * (2 * re) ^ (1 / 4) + sqrt(2 * re) / 4\n"
                       "crit = if 1 - al < edge then (if g > 0 then g * g / 2 * gs / (1 + gs) else 0) else ccoll\n"
                       "verdict = if crel < crit then 0 else 1",
            "inputs": {
                "G": inp("constant.G", "m3 kg-1 s-2", "gravitational constant"),
                "e": inp("orbit.eccentricity", "1", "inner stretch at which the orbits first touch", [0, 1]),
                "a1": inp("orbit.semi_major_axis", "m", "size of the inner orbit"),
                "a2": inp("orbit.semi_major_axis", "m", "size of the outer orbit"),
                "m1": inp("body.mass", "kg", "mass of the inner world"),
                "m2": inp("body.mass", "kg", "mass of the outer world"),
                "M": inp("orbit.host_mass", "kg", "mass of what both circle"),
                "amd": inp("orbit.angular_momentum_deficit", "kg m2 s-1", "the whole family's spin deficit"),
                "r": inp("model.mmr_width", "1", "width factor of first-order resonances"),
            },
            "outputs": {
                "crel": inp("orbit.relative_amd", "1", "the family's deficit against the outer world's circular spin"),
                "ccoll": inp("orbit.relative_amd", "1", "least deficit at which the orbits can touch"),
                "crit": inp("orbit.relative_amd", "1", "least deficit at which they can touch or their resonances overlap"),
                "verdict": verdict("0 can never collide, 1 has the spare stretch to collide in time"),
            },
            "source": {
                "text": "Laskar and Petit 2017: a pair can never collide if the system's deficit, against the outer planet's circular momentum, is below the least that lets the orbits touch. Petit, Laskar and Boué 2017: very close pairs use instead the deficit at which first-order resonances overlap, zero where even circular orbits overlap",
                "doi": "10.1051/0004-6361/201630022",
                "where": "Laskar and Petit 2017 eqs. 28, 29 and 39; Petit, Laskar and Boué 2017 eqs. 89, 97, 100 and 101",
            },
            "explain": {
                "curious": "Every family of worlds has a fixed amount of spare stretch to share. If there is too little for two neighbours ever to swing across each other's path, they can never collide. If there is enough, they might one day, though many such families last for billions of years. Our own Solar System has enough, because of the giant planets.",
                "science": "β = C/(Λ′ C_c) with Λ′ = m′√(GMa′); C_c from the collision root e_c (eq. 35), e′_c = 1 − α − αe_c. For 1 − α < 1.50ε^{1/4} + 0.32ε^{1/2} the critical value is the resonance-overlap one, (g²/2) γ√α/(1+γ√α). β > 1 is AMD-unstable: collisions are possible, not certain; the Solar System as a whole is AMD-unstable (Laskar and Petit 2017 section 5.4.1).",
            },
            "examples": exs,
        },
    }


# ----------------------------------------------------------------- moons

DWY = {"p0": 0.4895, "p1": 1.0305, "p2": 0.2738, "q0": 0.9309, "q1": 1.0764, "q2": 0.9812}


def moon_limit(ap, ep, mp, M, am, em, im, c=DWY):
    rh = ap * (mp / (3 * M)) ** (1 / 3)
    if math.cos(im) >= 0:
        lim = c["p0"] * rh * (1 - c["p1"] * ep - c["p2"] * em)
    else:
        lim = c["q0"] * rh * (1 - c["q1"] * ep - c["q2"] * em)
    return {"limit": lim, "verdict": 0 if am < lim else 2}


def moon_atom():
    exs = []
    for args in [
        (AU, 0.0167, M_EARTH, M_SUN, 3.8e8, 0.0549, 0.09),
        (5.2 * AU, 0.0484, M_JUP, M_SUN, 2.4e10, 0.2, 2.6),
        (0.05 * AU, 0.0, M_JUP, M_SUN, 3e9, 0.0, 0.0),
    ]:
        r = moon_limit(*args)
        exs.append({"ap": args[0], "ep": args[1], "mp": args[2], "M": args[3], "am": args[4], "em": args[5], "im": args[6],
                    "limit": r["limit"], "verdict": r["verdict"]})
    models = {k: model(v, "fitted", "Domingos, Winter and Yokoyama 2006 eqs. 5 and 6") for k, v in DWY.items()}
    return {
        "id": "sse.stability.moon_limit",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "planet_semi_major_axis", "letter": "ap"},
            {"port": "planet_eccentricity", "letter": "ep"},
            {"port": "planet_mass", "letter": "mp"},
            {"port": "star_mass", "letter": "M"},
            {"port": "moon_semi_major_axis", "letter": "am"},
            {"port": "moon_eccentricity", "letter": "em"},
            {"port": "moon_inclination", "letter": "im"},
        ],
        "outputs": [{"port": "limit", "letter": "limit"}, {"port": "verdict", "letter": "verdict"}],
        "models": models,
        "positive": [
            {"port": "planet_semi_major_axis", "says": "planet semi-major axis"},
            {"port": "planet_mass", "says": "planet mass"},
            {"port": "star_mass", "says": "star mass"},
            {"port": "moon_semi_major_axis", "says": "moon semi-major axis"},
        ],
        "references": [DWY06],
        "shown": "a_crit = 0.4895 R_H (1 − 1.0305 e_p − 0.2738 e_s)  (prograde)",
        "equation": {
            "name": "How far out a moon can circle before its star takes it",
            "formula": "rh = ap * cbrt(mp / (3 * M))\n"
                       "limit = if cos(im) >= 0 then p0 * rh * (1 - p1 * ep - p2 * em) else q0 * rh * (1 - q1 * ep - q2 * em)\n"
                       "verdict = if am < limit then 0 else 2",
            "inputs": {
                "ap": inp("orbit.semi_major_axis", "m", "size of the planet's orbit"),
                "ep": inp("orbit.eccentricity", "1", "how stretched the planet's orbit is", [0, 0.5]),
                "mp": inp("body.mass", "kg", "mass of the planet"),
                "M": inp("orbit.host_mass", "kg", "mass of the star"),
                "am": inp("orbit.semi_major_axis", "m", "size of the moon's orbit"),
                "em": inp("orbit.eccentricity", "1", "how stretched the moon's orbit is", [0, 0.5]),
                "im": inp("orbit.inclination", "1", "tilt of the moon's orbit, radians; past a right angle it goes backwards"),
                "p0": inp("model.moon_limit_prograde", "1", "forward moons: share of the gravity bubble"),
                "p1": inp("model.moon_limit_prograde_ep", "1", "forward moons: loss per planet stretch"),
                "p2": inp("model.moon_limit_prograde_es", "1", "forward moons: loss per moon stretch"),
                "q0": inp("model.moon_limit_retrograde", "1", "backward moons: share of the gravity bubble"),
                "q1": inp("model.moon_limit_retrograde_ep", "1", "backward moons: loss per planet stretch"),
                "q2": inp("model.moon_limit_retrograde_es", "1", "backward moons: loss per moon stretch"),
            },
            "outputs": {
                "limit": inp("orbit.moon_limit", "m", "widest orbit a moon keeps"),
                "verdict": verdict("0 the moon stays, 2 the star pulls it away"),
            },
            "source": {
                "text": "Domingos, Winter and Yokoyama 2006: from numerical runs, the widest stable moon orbit as a share of the planet's Hill radius, for moons going the planet's way and the other way",
                "doi": "10.1111/j.1365-2966.2006.11104.x",
                "where": "eqs. 5 (prograde) and 6 (retrograde)",
            },
            "explain": {
                "curious": "A moon has to stay well inside its planet's gravity bubble: about half of it for a moon going the planet's way, nearly all of it for one going backwards. Stretched orbits shrink the safe zone.",
                "science": "a_crit = 0.4895 R_H(1 − 1.0305e_p − 0.2738e_s) prograde, 0.9309 R_H(1 − 1.0764e_p − 0.9812e_s) retrograde, R_H = a_p(m_p/3M)^⅓.",
            },
            "examples": exs,
        },
    }


# ----------------------------------------------------------------- Kozai


def kozai(i1, n1, i2, n2, pin, pout, eo, m12, m3):
    c = math.cos(i1) * math.cos(i2) + math.sin(i1) * math.sin(i2) * math.cos(n1 - n2)
    imut = math.atan2(math.sqrt(max(0.0, 1 - c * c)), c)
    emax = math.sqrt(1 - 5 / 3 * c * c) if c * c < 3 / 5 else 0.0
    t = 2 / (3 * math.pi) * pout * pout / pin * (m12 + m3) / m3 * (1 - eo * eo) * math.sqrt(1 - eo * eo)
    return {"imut": imut, "emax": emax, "t": t, "verdict": 1 if c * c < 3 / 5 else 0}


def kozai_atom():
    yr = 3.2e7
    exs = []
    for args in [
        (0.0, 0.0, math.radians(60), 0.0, yr, 300 * yr, 0.2, M_SUN + M_JUP, 0.5 * M_SUN),
        (math.radians(5), 1.0, math.radians(10), 2.0, yr, 12 * yr, 0.05, M_SUN + M_EARTH, M_JUP),
    ]:
        r = kozai(*args)
        exs.append({"i1": args[0], "n1": args[1], "i2": args[2], "n2": args[3], "pin": args[4], "pout": args[5], "eo": args[6],
                    "m12": args[7], "m3": args[8], "imut": r["imut"], "emax": r["emax"], "t": r["t"], "verdict": r["verdict"]})
    return {
        "id": "sse.stability.kozai",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "inner_inclination", "letter": "i1"},
            {"port": "inner_ascending_node", "letter": "n1"},
            {"port": "outer_inclination", "letter": "i2"},
            {"port": "outer_ascending_node", "letter": "n2"},
            {"port": "inner_period", "letter": "pin"},
            {"port": "outer_period", "letter": "pout"},
            {"port": "outer_eccentricity", "letter": "eo"},
            {"port": "inner_pair_mass", "letter": "m12"},
            {"port": "perturber_mass", "letter": "m3"},
        ],
        "outputs": [
            {"port": "mutual_inclination", "letter": "imut"},
            {"port": "largest_eccentricity", "letter": "emax"},
            {"port": "cycle_time", "letter": "t"},
            {"port": "verdict", "letter": "verdict"},
        ],
        "positive": [
            {"port": "inner_period", "says": "inner period"},
            {"port": "outer_period", "says": "outer period"},
            {"port": "inner_pair_mass", "says": "inner pair mass"},
            {"port": "perturber_mass", "says": "perturber mass"},
        ],
        "checks": [{"port": "outer_eccentricity", "min": 0.0, "below": 1.0, "says": "outer eccentricity {} is not an ellipse"}],
        "references": [KOZAI, LIDOV, KEM98],
        "shown": "e_max = √(1 − (5/3) cos² i),  t ≈ (2/3π) P_out²/P_in (M/m₃)(1 − e_out²)^{3/2}",
        "equation": {
            "name": "How far a tilted outer pull can stretch an inner orbit",
            "formula": "c = cos(i1) * cos(i2) + sin(i1) * sin(i2) * cos(n1 - n2)\n"
                       "imut = atan2(sqrt(max(0, 1 - c * c)), c)\n"
                       "emax = if c * c < 3 / 5 then sqrt(1 - 5 / 3 * c * c) else 0\n"
                       "t = 2 / (3 * pi) * pout * pout / pin * (m12 + m3) / m3 * (1 - eo * eo) * sqrt(1 - eo * eo)\n"
                       "verdict = if c * c < 3 / 5 then 1 else 0",
            "inputs": {
                "i1": inp("orbit.inclination", "1", "tilt of the inner orbit, radians"),
                "n1": inp("orbit.ascending_node", "1", "where the inner orbit crosses up, radians"),
                "i2": inp("orbit.inclination", "1", "tilt of the outer orbit, radians"),
                "n2": inp("orbit.ascending_node", "1", "where the outer orbit crosses up, radians"),
                "pin": inp("orbit.period", "s", "the inner year"),
                "pout": inp("orbit.period", "s", "the outer year"),
                "eo": inp("orbit.eccentricity", "1", "how stretched the outer orbit is", [0, 0.99]),
                "m12": inp("orbit.total_mass", "kg", "mass of the inner pair"),
                "m3": inp("body.mass", "kg", "mass of the outer body"),
            },
            "outputs": {
                "imut": inp("orbit.mutual_inclination", "1", "angle between the two orbits, radians"),
                "emax": inp("orbit.eccentricity", "1", "the most stretched the inner orbit gets, starting round"),
                "t": inp("orbit.kozai_time", "s", "time for one swing"),
                "verdict": verdict("0 no swings, 1 the inner orbit swings between round and stretched"),
            },
            "source": {
                "text": "Kozai 1962 and Lidov 1962: a distant outer body tilted more than 39.2 degrees (below 140.8) trades the inner orbit's tilt for stretch, up to e = √(1 − (5/3)cos² i) from a round start. Kiseleva, Eggleton and Mikkola 1998: the time for one swing",
                "doi": "10.1086/108790",
                "where": "Kozai 1962 section 4; Kiseleva, Eggleton and Mikkola 1998 eq. 1",
            },
            "explain": {
                "curious": "If something heavy circles far outside an orbit, and the two orbits are tilted steeply against each other, the inner orbit slowly swings between round and very stretched, again and again.",
                "science": "Quadrupole test-particle limit, from a round start: e_max = √(1 − (5/3)cos² i_mut) for cos² i_mut < 3/5. Strongest when the outer body dominates; general relativity, tides or a third body's own precession can quench it, and comparable masses or a close outer body need the octupole terms (not here).",
            },
            "examples": exs,
        },
    }


# ------------------------------------------------------------- resonance

STRENGTH = 1.5
OVERLAP = 1.3
OVERLAP_E = 1.8


def resonance(pin, pout, p, q, m1, m2, M, ecc, f=STRENGTH, c=OVERLAP, ce=OVERLAP_E):
    k = p - q
    ratio = pout / pin
    offset = abs(ratio * q / p - 1)
    mu = (m1 + m2) / M
    da = (mu / 3) ** (1 / 3) if k < 0.5 else math.sqrt(16 / 3 * f * mu * ecc ** k)
    width = 1.5 * da
    sep = ratio ** (2 / 3) - 1
    zone = max(c * mu ** (2 / 7), ce * (mu * ecc) ** (1 / 5))
    held = offset < width and (k < 0.5 or sep >= zone)
    return {"offset": offset, "width": width, "zone": zone, "verdict": 1 if held else 0}


def resonance_atom():
    yr = 3.2e7
    exs = []
    for args in [
        # Neptune and Pluto: 3:2.
        (164.8 * yr, 247.9 * yr, 3, 2, 1.0e26, 1.3e22, M_SUN, 0.25),
        # Jupiter and a Trojan: one shared orbit.
        (11.86 * yr, 11.90 * yr, 1, 1, M_JUP, 1.0e17, M_SUN, 0.1),
        # Earth and Mars: near 2:1, but not in it.
        (1.0 * yr, 1.881 * yr, 2, 1, M_EARTH, 6.4e23, M_SUN, 0.093),
        # A stretched orbit 20% outside Jupiter's: inside 4:3's width, but
        # the resonances overlap there and nothing is held.
        (11.86 * yr, 15.47 * yr, 4, 3, M_JUP, 1.0e17, M_SUN, 0.25),
    ]:
        r = resonance(*args)
        exs.append({"pin": args[0], "pout": args[1], "p": args[2], "q": args[3], "m1": args[4], "m2": args[5],
                    "M": args[6], "ecc": args[7], "offset": r["offset"], "width": r["width"], "zone": r["zone"],
                    "verdict": r["verdict"]})
    return {
        "id": "sse.stability.resonance",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "inner_period", "letter": "pin"},
            {"port": "outer_period", "letter": "pout"},
            {"port": "inner_turns", "letter": "p"},
            {"port": "outer_turns", "letter": "q"},
            {"port": "inner_mass", "letter": "m1"},
            {"port": "outer_mass", "letter": "m2"},
            {"port": "host_mass", "letter": "M"},
            {"port": "eccentricity", "letter": "ecc"},
        ],
        "outputs": [
            {"port": "offset", "letter": "offset"},
            {"port": "width", "letter": "width"},
            {"port": "chaotic_zone", "letter": "zone"},
            {"port": "verdict", "letter": "verdict"},
        ],
        "models": {
            "f": model(STRENGTH, "assumed", MD99),
            "c": model(OVERLAP, "fitted", WISDOM80),
            "ce": model(OVERLAP_E, "fitted", MW12),
        },
        "positive": [
            {"port": "inner_period", "says": "inner period"},
            {"port": "outer_period", "says": "outer period"},
            {"port": "inner_turns", "says": "inner turns"},
            {"port": "outer_turns", "says": "outer turns"},
            {"port": "host_mass", "says": "host mass"},
        ],
        "checks": [{"port": "eccentricity", "min": 0.0, "below": 1.0, "says": "eccentricity {} is not an ellipse"}],
        "references": [MD99, DM81, WISDOM80, MW12],
        "shown": "|P₂/P₁ · q/p − 1| < (3/2) δa/a,  δa/a = (16/3 f μ e^(p−q))^½,  or (μ/3)^⅓ for a shared orbit",
        "equation": {
            "name": "Whether two neighbours sit inside a resonance",
            "formula": "k = p - q\n"
                       "ratio = pout / pin\n"
                       "offset = abs(ratio * q / p - 1)\n"
                       "mu = (m1 + m2) / M\n"
                       "da = if k < 1 / 2 then cbrt(mu / 3) else sqrt(16 / 3 * f * mu * ecc ^ k)\n"
                       "width = 3 / 2 * da\n"
                       "sep = ratio ^ (2 / 3) - 1\n"
                       "zone = max(c * mu ^ (2 / 7), ce * (mu * ecc) ^ (1 / 5))\n"
                       "verdict = if offset >= width then 0 else if k < 1 / 2 then 1 else if sep < zone then 0 else 1",
            "inputs": {
                "pin": inp("orbit.period", "s", "the inner year"),
                "pout": inp("orbit.period", "s", "the outer year"),
                "p": inp("orbit.resonance_turns", "1", "turns the inner one makes in one cycle of the resonance"),
                "q": inp("orbit.resonance_turns", "1", "turns the outer one makes in the same time"),
                "m1": inp("body.mass", "kg", "mass of the inner world"),
                "m2": inp("body.mass", "kg", "mass of the outer world"),
                "M": inp("orbit.host_mass", "kg", "mass of what both circle"),
                "ecc": inp("orbit.eccentricity", "1", "the more stretched of the two orbits", [0, 0.99]),
                "f": inp("model.resonance_strength", "1", "strength of the resonance's pull (alpha f_d)"),
                "c": inp("model.resonance_overlap", "1", "overlap zone of round orbits, per mu^(2/7)"),
                "ce": inp("model.resonance_overlap_stretched", "1", "overlap zone of stretched orbits, per (mu e)^(1/5)"),
            },
            "outputs": {
                "offset": inp("orbit.resonance_offset", "1", "how far the period ratio is from p:q, as a fraction"),
                "width": inp("orbit.resonance_width", "1", "how far it may be and still be held, as a fraction"),
                "zone": inp("orbit.chaotic_zone", "1", "orbit gap, as a fraction, inside which resonances overlap and hold nothing"),
                "verdict": verdict("0 not in this resonance, 1 inside it: the resonance can keep them from meeting"),
            },
            "source": {
                "text": "Murray and Dermott 1999: a p:q resonance of order k = p − q holds orbits within δa/a = (16/3 · f μ e^k)^½ of exact (the large-eccentricity limit of eq. 8.76; f is alpha f_d, 0.75 to 2 for the first-order resonances of Table 8.5). Dermott and Murray 1981: one shared orbit holds tadpoles and horseshoes out to about the Hill scale (μ/3)^⅓. Kepler's third law turns δa/a into 3/2 of that in the period ratio. Wisdom 1980 and Mustill and Wyatt 2012: closer than 1.3 μ^(2/7), or 1.8 (μ e)^(1/5) for stretched orbits, neighbouring resonances overlap and the motion is chaotic, so nothing is held there (a shared orbit excepted)",
                "doi": "10.1017/CBO9781139174817",
                "where": "Murray and Dermott 1999 section 8.7, eq. 8.76; Dermott and Murray 1981 section 4; Wisdom 1980 eq. 29; Mustill and Wyatt 2012 section 3",
            },
            "explain": {
                "curious": "Two worlds whose years fit a simple ratio, like Pluto's three turns for every two of Neptune's, can be held apart by timing: whenever they line up, they are far apart. That is how Pluto crosses Neptune's orbit and never meets it.",
                "science": "Pendulum width of a p:q mean motion resonance in the large-eccentricity limit, with f a typical alpha f_d. Being inside the width is necessary, not sufficient: the resonant angle must librate, which needs where each body is along its orbit, so the verdict says 'can', and stays worth watching. At very round orbits first-order widths tend to a floor of order μ^⅔ that this leaves out, so it can miss a resonance there, never invent one. Inside the overlap zone the resonances blur into chaos and hold nothing.",
            },
            "examples": exs,
        },
    }


# --------------------------------------------------------------- binaries

HW_S = {"s0": 0.464, "s1": -0.380, "s2": -0.631, "s3": 0.586, "s4": 0.150, "s5": -0.198}
HW_P = {"p0": 1.60, "p1": 5.10, "p2": -2.22, "p3": 4.12, "p4": -4.27, "p5": -5.09, "p6": 4.61}


def hw_s(ab, eb, mh, mc, ap, c=HW_S):
    mu = mc / (mh + mc)
    lim = ab * (c["s0"] + c["s1"] * mu + c["s2"] * eb + c["s3"] * mu * eb + c["s4"] * eb * eb + c["s5"] * mu * eb * eb)
    return {"mu": mu, "limit": lim, "verdict": 0 if ap < lim else 2}


def hw_p(ab, eb, m1, m2, ap, c=HW_P):
    mu = min(m1, m2) / (m1 + m2)
    lim = ab * (c["p0"] + c["p1"] * eb + c["p2"] * eb * eb + c["p3"] * mu + c["p4"] * eb * mu + c["p5"] * mu * mu + c["p6"] * eb * eb * mu * mu)
    return {"mu": mu, "limit": lim, "verdict": 0 if ap > lim else 2}


def hw_s_atom():
    exs = []
    # Alpha Centauri A and B, a planet at 1 au round A.
    for args in [(23.4 * AU, 0.52, 1.079 * M_SUN, 0.909 * M_SUN, AU), (20 * AU, 0.0, M_SUN, M_SUN, 8 * AU)]:
        r = hw_s(*args)
        exs.append({"ab": args[0], "eb": args[1], "mh": args[2], "mc": args[3], "ap": args[4], "mu": r["mu"], "limit": r["limit"], "verdict": r["verdict"]})
    models = {k: model(v, "fitted", "Holman and Wiegert 1999 eq. 3") for k, v in HW_S.items()}
    return {
        "id": "sse.stability.binary_limit_one_star",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "binary_semi_major_axis", "letter": "ab"},
            {"port": "binary_eccentricity", "letter": "eb"},
            {"port": "host_star_mass", "letter": "mh"},
            {"port": "companion_mass", "letter": "mc"},
            {"port": "planet_semi_major_axis", "letter": "ap"},
        ],
        "outputs": [{"port": "mass_share", "letter": "mu"}, {"port": "limit", "letter": "limit"}, {"port": "verdict", "letter": "verdict"}],
        "models": models,
        "positive": [
            {"port": "binary_semi_major_axis", "says": "binary semi-major axis"},
            {"port": "host_star_mass", "says": "host star mass"},
            {"port": "companion_mass", "says": "companion mass"},
            {"port": "planet_semi_major_axis", "says": "planet semi-major axis"},
        ],
        "references": [HW99],
        "shown": "a_c/a_b = 0.464 − 0.380μ − 0.631e + 0.586μe + 0.150e² − 0.198μe²",
        "equation": {
            "name": "Widest orbit round one star of a pair",
            "formula": "mu = mc / (mh + mc)\n"
                       "limit = ab * (s0 + s1 * mu + s2 * eb + s3 * mu * eb + s4 * eb * eb + s5 * mu * eb * eb)\n"
                       "verdict = if ap < limit then 0 else 2",
            "inputs": {
                "ab": inp("orbit.semi_major_axis", "m", "size of the two stars' orbit"),
                "eb": inp("orbit.eccentricity", "1", "how stretched the stars' orbit is", [0, 0.8]),
                "mh": inp("body.mass", "kg", "mass of the planet's own star"),
                "mc": inp("body.mass", "kg", "mass of the other star"),
                "ap": inp("orbit.semi_major_axis", "m", "size of the planet's orbit"),
                "s0": inp("model.hw_s0", "1", "fit constant"),
                "s1": inp("model.hw_s1", "1", "fit, per mass share"),
                "s2": inp("model.hw_s2", "1", "fit, per stretch"),
                "s3": inp("model.hw_s3", "1", "fit, per mass share and stretch"),
                "s4": inp("model.hw_s4", "1", "fit, per stretch squared"),
                "s5": inp("model.hw_s5", "1", "fit, per mass share and stretch squared"),
            },
            "outputs": {
                "mu": inp("orbit.mass_share", "1", "the other star's share of the pair's mass"),
                "limit": inp("orbit.binary_limit", "m", "widest orbit that lasts"),
                "verdict": verdict("0 the planet lasts, 2 the other star throws it out"),
            },
            "source": {
                "text": "Holman and Wiegert 1999: from runs of 10,000 binary periods, the widest lasting orbit round one star of a pair, fitted against the mass share and the pair's stretch",
                "doi": "10.1086/300695",
                "where": "eq. 3 (fitted for 0.1 <= mu <= 0.9 and 0 <= e <= 0.8)",
            },
            "explain": {
                "curious": "A planet circling one star of a pair has to stay well inside the pair's own orbit, often within a third of it, or the other star pulls it away.",
                "science": "Holman and Wiegert's fit to the critical semi-major axis for S-type orbits, initially circular and in the binary plane, with μ the companion's mass share.",
            },
            "examples": exs,
        },
    }


def hw_p_atom():
    exs = []
    # Kepler-16 (AB): planet at 0.7048 au round a 0.2243 au pair.
    for args in [(0.22431 * AU, 0.15944, 0.6897 * M_SUN, 0.20255 * M_SUN, 0.7048 * AU), (AU, 0.5, M_SUN, M_SUN, 2 * AU)]:
        r = hw_p(*args)
        exs.append({"ab": args[0], "eb": args[1], "m1": args[2], "m2": args[3], "ap": args[4], "mu": r["mu"], "limit": r["limit"], "verdict": r["verdict"]})
    models = {k: model(v, "fitted", "Holman and Wiegert 1999 eq. 4") for k, v in HW_P.items()}
    return {
        "id": "sse.stability.binary_limit_both_stars",
        "version": "1.0.0",
        "cost": "trivial",
        "inputs": [
            {"port": "binary_semi_major_axis", "letter": "ab"},
            {"port": "binary_eccentricity", "letter": "eb"},
            {"port": "first_star_mass", "letter": "m1"},
            {"port": "second_star_mass", "letter": "m2"},
            {"port": "planet_semi_major_axis", "letter": "ap"},
        ],
        "outputs": [{"port": "mass_share", "letter": "mu"}, {"port": "limit", "letter": "limit"}, {"port": "verdict", "letter": "verdict"}],
        "models": models,
        "positive": [
            {"port": "binary_semi_major_axis", "says": "binary semi-major axis"},
            {"port": "first_star_mass", "says": "first star mass"},
            {"port": "second_star_mass", "says": "second star mass"},
            {"port": "planet_semi_major_axis", "says": "planet semi-major axis"},
        ],
        "references": [HW99],
        "shown": "a_c/a_b = 1.60 + 5.10e − 2.22e² + 4.12μ − 4.27eμ − 5.09μ² + 4.61e²μ²",
        "equation": {
            "name": "Nearest orbit round both stars of a pair",
            "formula": "mu = min(m1, m2) / (m1 + m2)\n"
                       "limit = ab * (p0 + p1 * eb + p2 * eb * eb + p3 * mu + p4 * eb * mu + p5 * mu * mu + p6 * eb * eb * mu * mu)\n"
                       "verdict = if ap > limit then 0 else 2",
            "inputs": {
                "ab": inp("orbit.semi_major_axis", "m", "size of the two stars' orbit"),
                "eb": inp("orbit.eccentricity", "1", "how stretched the stars' orbit is", [0, 0.8]),
                "m1": inp("body.mass", "kg", "mass of one star"),
                "m2": inp("body.mass", "kg", "mass of the other star"),
                "ap": inp("orbit.semi_major_axis", "m", "size of the planet's orbit"),
                "p0": inp("model.hw_p0", "1", "fit constant"),
                "p1": inp("model.hw_p1", "1", "fit, per stretch"),
                "p2": inp("model.hw_p2", "1", "fit, per stretch squared"),
                "p3": inp("model.hw_p3", "1", "fit, per mass share"),
                "p4": inp("model.hw_p4", "1", "fit, per stretch and mass share"),
                "p5": inp("model.hw_p5", "1", "fit, per mass share squared"),
                "p6": inp("model.hw_p6", "1", "fit, per both squared"),
            },
            "outputs": {
                "mu": inp("orbit.mass_share", "1", "the lighter star's share of the pair's mass"),
                "limit": inp("orbit.binary_limit", "m", "nearest orbit that lasts"),
                "verdict": verdict("0 the planet lasts, 2 the pair throws it out"),
            },
            "source": {
                "text": "Holman and Wiegert 1999: from runs of 10,000 binary periods, the nearest lasting orbit round both stars of a pair, fitted against the mass share and the pair's stretch",
                "doi": "10.1086/300695",
                "where": "eq. 4 (fitted for 0.1 <= mu <= 0.5 and 0 <= e <= 0.7)",
            },
            "explain": {
                "curious": "A planet circling both stars of a close pair has to keep well clear of them, usually more than twice as far out as the stars are apart.",
                "science": "Holman and Wiegert's fit to the critical semi-major axis for P-type (circumbinary) orbits, with μ the lighter star's mass share. Kepler-16 b sits just outside it.",
            },
            "examples": exs,
        },
    }


def main():
    doc = {
        "format": "sse.formulas/1",
        "family": "sse.stability",
        "about": "Orbital stability checks (package ST1, ruling of 23 Nov): each check a visible formula block with its source, its fitted numbers as Model numbers, so the card's verdict shows its working.",
        "atoms": [hill_atom(), amd_term_atom(), collision_step_atom(), amd_critical_atom(), moon_atom(), kozai_atom(), hw_s_atom(), hw_p_atom(), resonance_atom()],
    }
    text = json.dumps(doc, indent=1, ensure_ascii=False) + "\n"
    old = open(OUT).read() if os.path.exists(OUT) else None
    if old != text:
        open(OUT, "w").write(text)
        print("wrote", os.path.relpath(OUT))
    else:
        print("stability blocks are current")


if __name__ == "__main__":
    main()
