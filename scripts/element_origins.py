"""Where each element was made: origin shares and origin stories for the master data sheet.

Read by scripts/build-base-reference.py. Shares come from handover/packages/MP2-data/origins.csv
(one row per element; the basis codes are explained in handover/packages/MP2.md). Stories are
written here from the shares, in the three wordings (physics, plain, story); a handful of
elements get a hand-written story. Every story that rests on a debated share says so in a line
(design section 8.1: say what is settled and what is not).
"""
import csv

SOURCES_ORDER = ["big_bang", "cosmic_rays", "low_mass_stars", "massive_stars", "white_dwarfs", "neutron_star_mergers", "human_made"]

# One legend for the whole app (design section 8.1): id -> (physics, plain, story) names.
SOURCE_NAMES = {
    "big_bang": ("Big Bang nucleosynthesis", "the Big Bang", "the first three minutes of the universe"),
    "cosmic_rays": ("cosmic-ray spallation", "cosmic rays breaking heavier atoms", "fast particles shattering atoms in the space between the stars"),
    "low_mass_stars": ("asymptotic giant branch stars (the s-process and hot-bottom burning)", "dying low-mass stars", "old, swollen giant stars, the kind the Sun will one day become, puffing their outer layers away"),
    "massive_stars": ("massive stars and their core-collapse supernovae", "exploding massive stars", "giant stars that burned out in a few million years and blew themselves apart"),
    "white_dwarfs": ("Type Ia supernovae", "exploding white dwarfs", "dead stars that took on too much matter and detonated"),
    "neutron_star_mergers": ("the r-process (here given to neutron-star mergers)", "merging neutron stars", "two dead stars, each heavier than the Sun and smaller than a city, spiralling together"),
    "human_made": ("synthesis in reactors and accelerators", "people, in reactors and particle accelerators", "people, in reactors and accelerators, a few atoms at a time"),
}

BASIS_SRC = {
    "bbn": ["fields2020", "lodders2021"],
    "bbn+cr": ["fields2020", "kobayashi2020"],
    "cr": ["johnson-origins"],
    "kobayashi2020": ["kobayashi2020", "johnson-origins"],
    "weak s": ["bisterzo2014", "kobayashi2020"],
    "s/r split": ["bisterzo2014", "kobayashi2020", "kasen2017"],
    "r only": ["kasen2017", "kobayashi2020"],
    "decay": ["kasen2017", "nubase2020"],
    "made": ["nubase2020"],
}

# Hand-written stories for the elements most worth a sentence of their own.
SPECIAL = {
    "h": ("Hydrogen nuclei are protons left over from the first minutes after the Big Bang, when the universe cooled too fast to fuse most of them. Stars have been burning it ever since, so the Sun's share is slightly less than the primordial one.",
          "Almost every hydrogen atom was made in the Big Bang, 13.8 billion years ago. It is the oldest thing you will ever touch.",
          "Every drop of water holds atoms that are as old as time itself, left over from the first minutes of the universe."),
    "he": ("About nine tenths of the Sun's helium is primordial (Big Bang nucleosynthesis set a helium mass fraction near 0.247); the rest was fused in stars of earlier generations and returned to the gas.",
           "Most helium was made in the Big Bang. The rest was made in older stars, which turn hydrogen into helium as they shine.",
           "Most of it is as old as the universe; a little was forged inside stars that lived and died before the Sun was born."),
    "c": ("Carbon comes from helium burning (the triple-alpha reaction). About half of it was returned by asymptotic-giant-branch stars, the other half by the winds and supernovae of massive stars.",
          "Carbon is made when stars fuse helium. About half came from dying low-mass stars puffing it out, and half from massive stars.",
          "The carbon in every living thing was cooked from helium inside old stars and breathed out into space as they died."),
    "o": ("Oxygen is made by helium and carbon burning in massive stars and is thrown out when they explode as core-collapse supernovae. It is the third most common element in the cosmos.",
          "Nearly all oxygen was made inside massive stars and blown out when they exploded.",
          "Every breath is the ash of giant stars that exploded long before the Sun formed."),
    "fe": ("Iron is the end of fusion's energy road. About half of the Sun's iron came from Type Ia supernovae and half from core-collapse supernovae, mostly as nickel-56 that decayed to iron.",
           "Iron comes from two kinds of exploding star: white dwarfs that detonated and massive stars that collapsed. Each gave about half.",
           "The iron in blood and in planets' cores was made as radioactive nickel in exploding stars, and turned to iron over a few months."),
    "au": ("Gold is almost entirely an r-process element: about 94% of the Sun's gold came from rapid neutron capture, here given to neutron-star mergers, and about 6% from the s-process in giant stars. A merger making heavy elements has been seen once directly (GW170817 and its kilonova). Whether mergers make nearly all the gold, or rare supernovae such as collapsars add a share, is still argued.",
           "Almost all gold was made in about a second, when two dead stars, each heavier than the Sun and smaller than a city, spiralled together and merged. The wreck probably collapsed into a black hole. Scientists still argue whether some gold also comes from rare kinds of supernova.",
           "Every ring was forged in a collision of two dead stars, so dense a teaspoon weighed a billion tonnes. The gold was flung out, drifted for hundreds of millions of years, and was swept into the cloud that made this star."),
    "pt": ("Platinum is about 95% r-process. A merger making heavy elements has been seen once (GW170817); how much rare supernovae add is argued.",
           "Platinum was made almost entirely in merging neutron stars, like gold. Whether rare supernovae also make some is still argued.",
           "Like gold, it was born in the crash of two dead stars and scattered across the galaxy."),
    "eu": ("Europium is the classic r-process tracer: about 95% of the Sun's europium is r-process. Its rise in old stars is too early for mergers alone, which is why rare magneto-rotational supernovae are argued to add a share (Kobayashi et al. 2020).",
           "Europium was made almost entirely by the rapid process, mostly in merging neutron stars. It appears so early in the galaxy that some must also come from rare supernovae, which is still argued.",
           "Astronomers follow europium to trace the galaxy's history of star collisions."),
    "ba": ("Barium is about 85% s-process: slow neutron capture in the helium-burning shells of asymptotic-giant-branch stars, over thousands of years.",
           "Most barium was made slowly, over thousands of years, inside dying low-mass stars.",
           "It was built atom by atom in the deep layers of old giant stars, one neutron at a time."),
    "u": ("Uranium is made only by the r-process. It decays (uranium-238 half-life 4.47 billion years), so the uranium in a world is a clock: there was nearly twice as much when the Sun formed.",
          "Uranium was made in merging neutron stars and has been slowly decaying ever since. Scientists still argue whether rare supernovae also make some.",
          "Its atoms are slowly running down. The uranium in this world has been ticking since before the Sun was born."),
    "li": ("Lithium has three sources: Big Bang nucleosynthesis, cosmic-ray spallation and stars (giants and novae). The Big Bang should have made about three times more lithium than the oldest stars show; this cosmological lithium problem is unsolved.",
           "Some lithium was made in the Big Bang, some by cosmic rays, and most by stars. Oddly, old stars have less lithium than the Big Bang should have made, a puzzle nobody has solved.",
           "The lightest metal carries an unsolved mystery: the oldest stars have less of it than the birth of the universe should have left."),
    "be": ("Beryllium is made by cosmic-ray spallation: fast protons and alpha particles break carbon, nitrogen and oxygen nuclei in the interstellar gas.",
           "Beryllium is made when cosmic rays smash into heavier atoms in space and break them apart.",
           "It is made by collisions in the empty space between stars, never inside them: stars destroy it."),
    "tc": ("Technetium has no stable isotope; the longest-lived lasts a few million years, so any made before the Sun formed is gone. Its lines in some red giants were the first proof that stars make elements (Merrill 1952).",
           "Technetium has no stable form, so the Earth's original supply is long gone. People make it in reactors. Some giant stars show it in their light, which proved that stars make elements.",
           "None survives from the stars. What exists was made by people, though some giant stars are making it right now."),
}

DECAY = ("Present in nature only briefly, as a step in the decay of uranium and thorium, which were made by the r-process.",
         "It exists in nature only fleetingly, made as uranium and thorium decay.",
         "It flickers into being inside uranium ores and soon decays away.")


def load(path):
    out = {}
    with open(path, newline="") as f:
        for r in csv.DictReader(f):
            shares = {k: float(r[k]) for k in SOURCES_ORDER if float(r[k]) > 0}
            out[r["id"]] = {"shares": shares, "basis": r["basis"], "argued": r["argued"]}
    return out


def _list(names):
    names = list(names)
    if len(names) == 1:
        return names[0]
    return ", ".join(names[:-1]) + " and " + names[-1]


def _share_words(f):
    if f >= 0.95:
        return "almost all"
    if f >= 0.7:
        return "most"
    if f >= 0.55:
        return "a little over half"
    if f >= 0.45:
        return "about half"
    if f >= 0.3:
        return "about a third"
    if f >= 0.15:
        return "a fifth or so"
    return "a little"


def story(eid, name, o):
    if eid in SPECIAL:
        p, pl, st = SPECIAL[eid]
        return {"physics": p, "plain": pl, "story": st}
    shares = sorted(o["shares"].items(), key=lambda kv: -kv[1])
    if o["basis"].startswith("decay"):
        p, pl, st = DECAY
    elif shares[0][0] == "human_made":
        p = f"{name} has no isotope long-lived enough to survive from the stars; what exists was made by {SOURCE_NAMES['human_made'][0]}."
        pl = f"{name} is made by {SOURCE_NAMES['human_made'][1]}. None is left from the stars."
        st = f"It does not occur in nature: it was made by {SOURCE_NAMES['human_made'][2]}."
    else:
        parts_p = [f"{round(f * 100)}% from {SOURCE_NAMES[k][0]}" for k, f in shares]
        p = f"The Sun's {name.lower()}: " + _list(parts_p) + "."
        first, f0 = shares[0]
        pl = f"{_share_words(f0).capitalize()} of the {name.lower()} was made by {SOURCE_NAMES[first][1]}"
        if len(shares) > 1:
            pl += "; the rest came from " + _list(SOURCE_NAMES[k][1] for k, _ in shares[1:])
        pl += "."
        st = f"Its atoms were made by {SOURCE_NAMES[first][2]}"
        if len(shares) > 1:
            st += ", with a share from " + _list(SOURCE_NAMES[k][1] for k, _ in shares[1:])
        st += "."
    if o["argued"] and eid not in SPECIAL and not o["basis"].startswith("decay"):
        p += " Argued: " + o["argued"] + "."
        pl += " Scientists still argue about part of this."
    return {"physics": p, "plain": pl, "story": st}


def origin_value(eid, o):
    src = BASIS_SRC[o["basis"].split(":")[0]]
    row = {"shares": o["shares"], "src": src[0], "also": src[1:], "check": "authored",
           "note": "approximate shares of the Sun's birth-cloud abundance, rounded; MP7 replaces them with the galaxy model's own ledger"}
    if o["argued"]:
        row["argued"] = o["argued"]
    return row
