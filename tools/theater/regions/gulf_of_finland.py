"""Geography of the Gulf of Finland theater (2027): southern Finland, Estonia, the north of
Latvia, and the Russian north-west round St Petersburg, Novgorod and Pskov.

The theater squeezes 22.0-34.3 E, 56.6-62.8 N (about 690 km square) into the 512 km of a
theater. Coordinates here are (lon, lat) degrees.

Sides: Finland, Estonia and Latvia (NATO) are the friendly side, Russia the enemy. The war has
just broken out: the front line is the border, the Finnish-Russian one from Ilomantsi down to
the coast at Vaalimaa, across the gulf to the Narva, through Lake Peipus and on along the
Estonian and Latvian borders. Fighting flares on the Karelian Isthmus, at Narva and Pechory.
"""

import os
import re

PROJECTION = dict(lon0=22.0, lat0=56.6, lat_ref=59.7, squeeze=1.35)

LIBRARY = "central_europe"
LAND_RUBBLE = 102

DATE = (4, 15, 2027)
GRID = ("VL", 0, 0)
SQUADRONS = 4

EXTRA_MODELS = ["npp", "ruins", "rubble"]

SAM_RADARS = ["sa10", "sa21", "sa17", "sa22"]
MOBILE_SAM_RADARS = ["sa17", "sa15"]
BOAT_RADARS = ["san7", "san5"]

CITY_SAMS = ["St Petersburg", "Pskov", "Novgorod", "Vyborg", "Kronstadt"]

# The air forces: (aircraft, its air-to-air missile, task). Finnish Hornets and allied F-16s
# on the friendly side.
ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("su34", "aa8", "base"),
    ("su35", "aa10", "patrol"),
    ("su27", "aa10t", "patrol"),
    ("a50", "aa10", "patrol_wide"),
    ("mig31", "aa9", "patrol"),
    ("mig31", "aa9", "escort"),
    ("su35", "aa10t", "escort"),
    ("su34", "aa10t", "base"),
    ("f18", "none", "friendly"),
    ("f18", "none", "friendly"),
    ("f16", "none", "friendly"),
    ("f16", "none", "friendly"),
]


# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea.

LANDS = [
    # The mainland, coarse far outside the theater: the southern Baltic coast, Kaliningrad and
    # the Lithuanian and Latvian coasts, the Gulf of Riga, Estonia's west and north coasts, the
    # Russian coast to the Neva, the Karelian Isthmus and Vyborg Bay, the Finnish coast west to
    # Hanko and Turku, up the Gulf of Bothnia and down the Swedish coast.
    [
        (14.2, 53.9), (16.0, 54.3), (17.5, 54.75), (18.5, 54.8), (18.6, 54.45), (19.4, 54.35), (19.9, 54.6),
        (19.95, 54.95), (20.6, 54.95), (21.2, 55.25), (21.08, 55.7), (21.05, 56.1), (21.0, 56.55),
        (21.2, 57.0), (21.45, 57.3), (21.8, 57.55), (22.25, 57.65), (22.6, 57.76),
        # The Gulf of Riga: Kolka, Riga, the Vidzeme coast, Pärnu bay
        (22.65, 57.55), (23.0, 57.3), (23.4, 57.05), (23.8, 56.98), (24.1, 57.03), (24.35, 57.2),
        (24.4, 57.6), (24.35, 57.9), (24.45, 58.08), (24.52, 58.36), (24.2, 58.35), (23.8, 58.28),
        (23.65, 58.45), (23.5, 58.58),
        # Estonia's west coast past Haapsalu, the north coast past Paldiski, Tallinn and Kunda
        (23.6, 58.75), (23.48, 58.95), (23.4, 59.15), (23.6, 59.25), (23.95, 59.3), (24.0, 59.38),
        (24.3, 59.42), (24.55, 59.45), (24.75, 59.48), (24.85, 59.58), (25.05, 59.5), (25.4, 59.55),
        (25.65, 59.62), (25.95, 59.6), (26.3, 59.55), (26.55, 59.55), (26.9, 59.48), (27.3, 59.45),
        (27.75, 59.42), (28.04, 59.46),
        # The Russian coast: Luga Bay, Koporye Bay, Sosnovy Bor, Lomonosov, the Neva
        (28.12, 59.6), (28.05, 59.78), (28.25, 59.8), (28.42, 59.68), (28.6, 59.72), (28.75, 59.82),
        (28.95, 59.8), (29.1, 59.88), (29.4, 59.97), (29.77, 59.92), (30.05, 59.86), (30.2, 59.88),
        (30.24, 59.97),
        # The Neva Bay's north shore, the Karelian Isthmus, Vyborg Bay
        (30.15, 60.0), (29.95, 60.1), (29.7, 60.2), (29.4, 60.17), (29.0, 60.2), (28.75, 60.3),
        (28.62, 60.4), (28.75, 60.55), (28.78, 60.7), (28.68, 60.7), (28.55, 60.6), (28.35, 60.52),
        # The Finnish coast: Vaalimaa, Hamina, Kotka, Loviisa, Porvoo, Helsinki, Porkkala, Hanko
        (28.0, 60.52), (27.75, 60.5), (27.5, 60.52), (27.2, 60.53), (26.95, 60.45), (26.5, 60.42),
        (26.25, 60.42), (25.65, 60.33), (25.2, 60.2), (24.95, 60.15), (24.65, 60.12), (24.45, 60.0),
        (24.1, 60.02), (23.6, 59.95), (23.2, 59.85), (22.95, 59.82), (22.85, 59.95), (22.5, 60.05),
        (22.3, 60.25), (22.2, 60.42),
        # The west coast and the Gulf of Bothnia
        (21.7, 60.5), (21.4, 60.75), (21.45, 61.1), (21.5, 61.5), (21.3, 62.0), (21.2, 62.4), (21.5, 63.0),
        (22.3, 63.5), (23.5, 64.0), (24.5, 64.7), (25.3, 65.0), (25.2, 65.4), (24.6, 65.7), (24.15, 65.85),
        # Sweden
        (23.2, 65.8), (22.2, 65.6), (21.4, 65.0), (20.9, 64.4), (20.3, 63.8), (19.4, 63.5), (18.6, 63.2),
        (17.6, 62.5), (17.3, 61.7), (17.2, 61.0), (17.2, 60.65), (17.9, 60.4), (18.6, 60.25), (18.9, 59.9),
        (18.7, 59.6), (18.3, 59.35), (17.6, 58.9), (16.8, 58.6), (16.6, 57.9), (16.5, 57.2), (16.3, 56.65),
        (15.9, 56.2), (14.7, 56.1), (14.2, 55.4), (13.0, 55.4), (12.9, 55.9), (12.5, 56.3), (11.0, 58.0),
        (10.0, 59.0), (5.0, 59.0), (5.0, 72.0), (60.0, 72.0), (60.0, 45.0), (10.0, 45.0), (10.0, 53.9),
    ],
    # Saaremaa, Muhu, Hiiumaa, Vormsi
    [(21.8, 58.45), (22.1, 58.6), (22.6, 58.62), (23.0, 58.58), (23.05, 58.4), (22.75, 58.25), (22.5, 58.22),
     (22.25, 58.1), (22.1, 57.92), (22.0, 58.05), (22.05, 58.25), (21.9, 58.3)],
    [(23.1, 58.55), (23.35, 58.65), (23.4, 58.55), (23.2, 58.5)],
    [(22.05, 58.95), (22.45, 59.05), (22.9, 58.98), (23.0, 58.85), (22.75, 58.75), (22.5, 58.7), (22.2, 58.8)],
    [(23.05, 59.0), (23.3, 59.05), (23.25, 58.95)],
    # Kotlin (Kronstadt) and Gogland in the gulf
    [(29.62, 59.99), (29.75, 60.03), (29.83, 60.0), (29.7, 59.975)],
    [(26.95, 60.03), (27.0, 60.12), (27.06, 60.05)],
    # Åland, Gotland, Öland
    [(19.6, 60.1), (19.75, 60.4), (20.2, 60.35), (20.3, 60.1), (19.95, 59.98)],
    [(18.15, 57.1), (18.3, 57.6), (18.75, 57.95), (19.1, 57.8), (18.8, 57.4), (18.5, 56.92)],
    [(16.4, 56.2), (16.95, 57.35), (17.1, 57.3), (16.6, 56.2)],
]

LAKES = [
    # Ladoga
    [(31.0, 59.98), (31.5, 60.08), (32.0, 60.1), (32.35, 60.15), (32.7, 60.35), (32.9, 60.55), (32.75, 60.85),
     (32.4, 61.05), (32.0, 61.3), (31.6, 61.55), (31.1, 61.68), (30.7, 61.7), (30.3, 61.55), (30.2, 61.25),
     (30.18, 61.08), (30.3, 60.8), (30.5, 60.5), (30.65, 60.25), (30.85, 60.05)],
    # Onega (its western shore at the theater's edge)
    [(35.3, 61.0), (35.7, 61.1), (36.4, 61.3), (36.5, 61.9), (35.8, 62.4), (35.0, 62.8), (34.5, 62.4),
     (34.45, 61.85), (34.9, 61.4)],
    # Peipus, Lämmijärv and Pskov lake
    [(27.05, 58.92), (27.4, 59.0), (27.75, 58.98), (28.0, 58.88), (27.9, 58.6), (27.65, 58.38), (27.6, 58.2),
     (27.8, 58.05), (28.0, 57.95), (28.12, 57.86), (27.95, 57.84), (27.75, 57.92), (27.55, 58.1),
     (27.45, 58.25), (27.15, 58.42), (26.98, 58.68), (26.98, 58.85)],
    # Ilmen
    [(31.05, 58.3), (31.25, 58.43), (31.6, 58.38), (31.75, 58.25), (31.5, 58.1), (31.2, 58.12)],
    # Saimaa round Lappeenranta, and Haukivesi below Savonlinna
    [(28.0, 61.13), (28.35, 61.11), (28.6, 61.22), (28.55, 61.35), (28.2, 61.36), (27.95, 61.25)],
    [(28.3, 61.55), (28.7, 61.6), (28.78, 61.8), (28.5, 61.82), (28.25, 61.7)],
    # Päijänne
    [(25.45, 61.05), (25.6, 61.25), (25.75, 61.6), (25.8, 62.0), (25.7, 62.2), (25.58, 62.15), (25.55, 61.8),
     (25.4, 61.4), (25.35, 61.15)],
    # Võrtsjärv
    [(25.95, 58.4), (26.1, 58.35), (26.15, 58.2), (26.05, 58.05), (25.9, 58.15)],
]

RIVERS = [
    # Neva from Shlisselburg to St Petersburg
    [(31.0, 59.96, 0.35), (30.75, 59.85, 0.35), (30.55, 59.83, 0.35), (30.45, 59.88, 0.35), (30.35, 59.95, 0.4),
     (30.22, 59.95, 0.5)],
    # Narva from Peipus through the Narva reservoir to Narva-Jõesuu
    [(28.0, 58.97, 0.15), (28.05, 59.1, 0.15), (28.0, 59.2, 1.0), (28.08, 59.27, 0.8), (28.205, 59.37, 0.15),
     (28.12, 59.42, 0.15), (28.04, 59.47, 0.2)],
    # Volkhov from Ilmen past Novgorod, Kirishi and Volkhov to Ladoga
    [(31.3, 58.42, 0.15), (31.27, 58.52, 0.15), (31.6, 58.9, 0.15), (32.0, 59.45, 0.15), (32.33, 59.92, 0.15),
     (32.32, 60.08, 0.2)],
    # Svir from Onega to Ladoga
    [(35.5, 61.0, 0.2), (34.8, 61.0, 0.2), (34.17, 60.91, 0.25), (33.55, 60.73, 0.2), (33.2, 60.6, 0.2),
     (32.88, 60.52, 0.25)],
    # Luga
    [(30.5, 58.5, 0.08), (29.85, 58.73, 0.1), (29.2, 59.05, 0.1), (28.65, 59.35, 0.1), (28.45, 59.55, 0.12),
     (28.42, 59.67, 0.15)],
    # Velikaya past Ostrov and Pskov
    [(28.5, 56.6, 0.1), (28.35, 57.35, 0.12), (28.33, 57.82, 0.12), (28.1, 57.86, 0.15)],
    # Kymijoki through Kouvola to Kotka
    [(26.25, 61.05, 0.1), (26.6, 60.9, 0.1), (26.75, 60.75, 0.1), (26.85, 60.55, 0.1), (26.92, 60.46, 0.12)],
    # Vuoksi from Imatra to Ladoga
    [(28.6, 61.22, 0.15), (28.8, 61.17, 0.15), (29.2, 61.1, 0.15), (29.6, 60.95, 0.15), (29.85, 61.0, 0.15),
     (30.18, 61.03, 0.2)],
    # Msta to Ilmen
    [(34.3, 58.0, 0.08), (33.92, 58.39, 0.08), (33.0, 58.55, 0.08), (32.2, 58.5, 0.08), (31.7, 58.36, 0.08)],
    # Daugava to Riga
    [(26.5, 56.25, 0.25), (25.85, 56.5, 0.25), (25.2, 56.65, 0.3), (24.6, 56.8, 0.3), (24.1, 56.95, 0.35),
     (24.03, 57.04, 0.4)],
    # Emajõgi from Võrtsjärv past Tartu to Peipus
    [(26.12, 58.38, 0.06), (26.72, 58.38, 0.07), (27.0, 58.36, 0.08), (27.3, 58.36, 0.08)],
]


# ---------------------------------------------------------------------------------------------
# Sides

# Front line, north to south: (lon, lat, intensity). The hop across the gulf is quiet.
FRONT = [
    (31.45, 62.9, 0.1), (31.0, 62.6, 0.1), (30.6, 62.4, 0.15), (30.15, 62.2, 0.15), (29.95, 61.95, 0.15),
    (29.65, 61.65, 0.2), (29.3, 61.35, 0.25), (28.95, 61.18, 0.35), (28.7, 61.05, 0.4), (28.45, 60.9, 0.45),
    (28.1, 60.72, 0.5), (27.82, 60.56, 0.5), (27.75, 60.48, 0.0), (27.9, 60.0, 0.0),
    (28.04, 59.46, 0.5), (28.205, 59.375, 0.5), (28.06, 59.2, 0.4), (28.0, 59.0, 0.1), (27.65, 58.85, 0.1),
    (27.5, 58.5, 0.1), (27.55, 58.15, 0.1), (27.7, 57.95, 0.3), (27.62, 57.85, 0.35), (27.75, 57.75, 0.35),
    (27.55, 57.6, 0.25), (27.35, 57.52, 0.2), (27.7, 57.3, 0.15), (27.85, 57.1, 0.15), (28.15, 56.85, 0.15),
    (28.2, 56.55, 0.1),
]

# Finland, Estonia, Latvia and the open Baltic west of the front.
BLUE = (
    [(15.0, 50.0), (15.0, 70.0), (29.0, 70.0), (29.6, 66.0), (30.1, 64.5), (30.6, 63.6)]
    + [(lon, lat) for lon, lat, _ in FRONT]
    + [(28.4, 56.0), (28.5, 50.0)]
)


# ---------------------------------------------------------------------------------------------
# Places

TOWNS = [
    # Finland
    ("Helsinki", 24.94, 60.19, 2, 3000, 0.0, True),
    ("Espoo", 24.66, 60.21, 2, 1800, 0.0, False),
    ("Tampere", 23.76, 61.50, 2, 2200, 0.0, True),
    ("Turku", 22.27, 60.46, 2, 2000, 0.0, True),
    ("Lahti", 25.66, 60.98, 2, 1600, 0.0, True),
    ("Jyvaskyla", 25.75, 62.24, 2, 1500, 0.0, False),
    ("Kotka", 26.93, 60.48, 1, 1200, 0.0, True),
    ("Hamina", 27.20, 60.57, 1, 900, 0.05, False),
    ("Kouvola", 26.70, 60.87, 1, 1200, 0.0, True),
    ("Lappeenranta", 28.19, 61.06, 1, 1400, 0.05, True),
    ("Imatra", 28.76, 61.18, 1, 1100, 0.1, True),
    ("Mikkeli", 27.27, 61.69, 1, 1100, 0.0, True),
    ("Savonlinna", 28.88, 61.87, 1, 900, 0.0, False),
    ("Joensuu", 29.76, 62.60, 1, 1200, 0.0, True),
    ("Porvoo", 25.66, 60.40, 1, 900, 0.0, False),
    ("Loviisa", 26.23, 60.46, 0, 700, 0.0, False),
    ("Hanko", 22.97, 59.84, 0, 700, 0.0, False),
    ("Hameenlinna", 24.46, 61.00, 1, 1100, 0.0, False),
    ("Riihimaki", 24.77, 60.74, 1, 900, 0.0, False),
    ("Salo", 23.13, 60.39, 1, 900, 0.0, False),
    ("Heinola", 26.03, 61.20, 0, 700, 0.0, False),
    ("Parikkala", 29.50, 61.55, 0, 500, 0.1, False),
    # Estonia
    ("Tallinn", 24.75, 59.43, 2, 2600, 0.0, True),
    ("Tartu", 26.72, 58.38, 2, 1600, 0.0, True),
    ("Narva", 28.19, 59.38, 1, 1400, 0.15, True),
    ("Parnu", 24.50, 58.39, 1, 1200, 0.0, True),
    ("Kohtla-Jarve", 27.27, 59.40, 1, 1100, 0.05, False),
    ("Sillamae", 27.76, 59.40, 0, 800, 0.1, False),
    ("Johvi", 27.41, 59.36, 0, 700, 0.0, False),
    ("Rakvere", 26.36, 59.35, 1, 900, 0.0, True),
    ("Viljandi", 25.59, 58.36, 1, 900, 0.0, False),
    ("Voru", 27.00, 57.83, 0, 800, 0.0, False),
    ("Valga", 26.05, 57.78, 0, 800, 0.0, False),
    ("Paide", 25.56, 58.89, 0, 600, 0.0, False),
    ("Haapsalu", 23.54, 58.94, 0, 700, 0.0, False),
    ("Kuressaare", 22.48, 58.25, 0, 800, 0.0, False),
    ("Paldiski", 24.06, 59.34, 0, 600, 0.0, False),
    ("Rapla", 24.79, 59.00, 0, 600, 0.0, False),
    # Latvia
    ("Riga", 24.11, 56.95, 2, 3000, 0.0, True),
    ("Valmiera", 25.43, 57.54, 1, 900, 0.0, False),
    ("Cesis", 25.27, 57.31, 0, 700, 0.0, False),
    ("Sigulda", 24.86, 57.15, 0, 600, 0.0, False),
    ("Ogre", 24.60, 56.82, 0, 700, 0.0, False),
    ("Gulbene", 26.75, 57.18, 0, 600, 0.0, False),
    ("Aluksne", 27.05, 57.42, 0, 600, 0.05, False),
    ("Balvi", 27.27, 57.13, 0, 600, 0.05, False),
    ("Smiltene", 25.90, 57.42, 0, 500, 0.0, False),
    # Russia: St Petersburg and the Leningrad region
    ("St Petersburg", 30.32, 59.94, 2, 5000, 0.0, True),
    ("Kronstadt", 29.77, 60.00, 1, 900, 0.0, True),
    ("Vyborg", 28.75, 60.72, 1, 1400, 0.05, True),
    ("Sestroretsk", 29.97, 60.10, 0, 800, 0.0, False),
    ("Zelenogorsk", 29.70, 60.20, 0, 700, 0.0, False),
    ("Primorsk", 28.61, 60.37, 0, 600, 0.0, False),
    ("Lomonosov", 29.77, 59.90, 0, 700, 0.0, False),
    ("Sosnovy Bor", 29.10, 59.89, 1, 900, 0.0, True),
    ("Ust-Luga", 28.42, 59.65, 0, 600, 0.0, False),
    ("Kingisepp", 28.60, 59.37, 1, 900, 0.0, True),
    ("Ivangorod", 28.22, 59.37, 0, 700, 0.1, False),
    ("Slantsy", 28.09, 59.12, 0, 700, 0.0, False),
    ("Gatchina", 30.13, 59.57, 1, 1100, 0.0, True),
    ("Pushkin", 30.40, 59.72, 1, 1000, 0.0, False),
    ("Kolpino", 30.60, 59.75, 1, 900, 0.0, False),
    ("Tosno", 30.88, 59.54, 0, 700, 0.0, False),
    ("Shlisselburg", 31.03, 59.94, 0, 600, 0.0, False),
    ("Luga", 29.85, 58.74, 1, 900, 0.0, True),
    ("Kirishi", 32.02, 59.45, 1, 900, 0.0, True),
    ("Volkhov", 32.34, 59.92, 1, 900, 0.0, False),
    ("Tikhvin", 33.60, 59.64, 1, 900, 0.0, True),
    ("Lodeynoye Pole", 33.55, 60.73, 0, 700, 0.0, True),
    ("Podporozhye", 34.17, 60.91, 0, 600, 0.0, False),
    ("Priozersk", 30.13, 61.04, 0, 700, 0.0, False),
    ("Svetogorsk", 28.88, 61.11, 0, 700, 0.1, False),
    ("Kamennogorsk", 29.13, 60.95, 0, 500, 0.0, False),
    ("Sortavala", 30.70, 61.71, 0, 700, 0.0, True),
    ("Olonets", 32.96, 60.98, 0, 600, 0.0, False),
    ("Pitkyaranta", 31.57, 61.57, 0, 600, 0.0, False),
    ("Petrozavodsk", 34.35, 61.78, 2, 1800, 0.0, False),
    # Russia: Pskov and Novgorod
    ("Pskov", 28.33, 57.82, 2, 1800, 0.0, True),
    ("Pechory", 27.61, 57.81, 0, 600, 0.15, False),
    ("Ostrov", 28.35, 57.35, 0, 800, 0.0, True),
    ("Porkhov", 29.56, 57.77, 0, 600, 0.0, False),
    ("Dno", 29.97, 57.83, 0, 600, 0.0, False),
    ("Novgorod", 31.27, 58.53, 2, 1900, 0.0, True),
    ("Chudovo", 31.68, 59.12, 0, 600, 0.0, False),
    ("Staraya Russa", 31.36, 57.99, 1, 900, 0.0, False),
    ("Borovichi", 33.92, 58.39, 1, 900, 0.0, False),
    ("Valday", 33.24, 57.98, 0, 600, 0.0, False),
]

# Airbases: (name, lon, lat, friendly).
AIRBASES = [
    ("Helsinki", 24.96, 60.32, True),
    ("Tampere", 23.60, 61.42, True),
    ("Utti", 26.94, 60.90, True),
    ("Lappeenranta", 28.14, 61.04, True),
    ("Mikkeli", 27.20, 61.69, True),
    ("Turku", 22.28, 60.51, True),
    ("Halli", 24.79, 61.86, True),
    ("Jyvaskyla", 25.68, 62.40, True),
    ("Amari", 23.97, 59.26, True),
    ("Tallinn", 24.83, 59.41, True),
    ("Tartu", 26.69, 58.31, True),
    ("Parnu", 24.47, 58.42, True),
    ("Lielvarde", 24.85, 56.77, True),
    ("Levashovo", 30.20, 60.09, False),
    ("Pushkin", 30.35, 59.69, False),
    ("Siverskiy", 30.04, 59.36, False),
    ("Vyborg", 28.85, 60.55, False),
    ("Pskov", 28.39, 57.78, False),
    ("Ostrov", 28.33, 57.30, False),
    ("Novgorod", 31.37, 58.62, False),
    ("Lodeynoye Pole", 33.58, 60.71, False),
    ("Besovets", 34.16, 61.88, False),
]

# Nuclear power plants: the Leningrad plant at Sosnovy Bor (RBMK and VVER-1200), Loviisa.
NPPS = [
    ("Leningrad NPP", 29.05, 59.82, "kursk"),
    ("Loviisa NPP", 26.32, 60.42, "south"),
]

# Attack drone (Geran) launch sites.
LAUNCH_SITES = [
    ("Pskov", 28.6, 57.6),
    ("Luga", 29.6, 58.55),
    ("Lodeynoye Pole", 33.3, 60.6),
]

SITES = [
    ("hq", 30.55, 60.05, "St Petersburg"), ("hq", 28.45, 57.9, "Pskov"), ("hq", 31.15, 58.62, "Novgorod"),
    ("hq", 28.95, 60.78, "Vyborg"),
    ("depot", 29.95, 58.7, "Luga"), ("depot", 28.72, 59.33, "Kingisepp"), ("depot", 30.05, 59.53, "Gatchina"),
    ("depot", 28.45, 57.42, "Ostrov"), ("depot", 33.5, 59.6, "Tikhvin"), ("depot", 29.15, 60.9, "Kamennogorsk"),
    ("tankfarm", 32.08, 59.42, "Kirishi"), ("tankfarm", 28.45, 59.62, "Ust-Luga"),
    ("tankfarm", 30.45, 59.85, "St Petersburg"), ("tankfarm", 28.68, 60.38, "Primorsk"),
    ("armor", 28.32, 59.33, "Ivangorod"), ("armor", 29.3, 60.55, "Vyborg"), ("armor", 27.78, 57.85, "Pechory"),
    ("armor", 28.98, 61.05, "Svetogorsk"), ("armor", 28.2, 57.2, "Ostrov"),
    ("ss", 29.75, 58.66, "Luga"), ("ss", 28.42, 57.22, "Ostrov"),
    ("oth", 30.62, 60.28, "Lekhtusi"),
    ("bridge", 28.2, 59.375, "Narva"), ("bridge", 32.3, 59.9, "Volkhov"), ("bridge", 28.62, 59.37, "Kingisepp"),
    ("bridge", 30.95, 59.92, "Shlisselburg"),
    ("msam", 29.6, 59.92, "Kronstadt"), ("msam", 28.6, 60.65, "Vyborg"), ("msam", 28.15, 57.7, "Pskov"),
    ("msam", 28.75, 59.45, "Kingisepp"), ("msam", 31.0, 58.65, "Novgorod"),
    ("pow", 31.9, 59.1, "Chudovo"), ("pow", 29.6, 57.72, "Porkhov"),
]

# Patrol boats: (lon, lat), off Kronstadt, Ust-Luga and Vyborg Bay.
BOATS = [(29.55, 60.02), (29.9, 59.97), (28.35, 59.78), (28.5, 60.35), (29.2, 60.05)]

# Forest regions: (lon, lat, radius theater km, density 0..1). Boreal everywhere.
FORESTS = [
    # Finland
    (23.5, 61.0, 25, 0.65), (25.0, 61.3, 30, 0.7), (26.5, 61.5, 30, 0.7), (27.8, 62.0, 30, 0.75),
    (29.5, 62.4, 30, 0.75), (24.5, 62.2, 25, 0.7), (26.2, 62.4, 25, 0.75), (23.4, 60.6, 15, 0.5),
    (25.8, 60.7, 15, 0.5), (29.3, 61.4, 15, 0.7), (22.8, 61.5, 20, 0.6),
    # The Karelian Isthmus and Ladoga Karelia
    (29.5, 60.6, 25, 0.7), (30.0, 60.9, 15, 0.65), (31.5, 62.2, 25, 0.75), (33.0, 61.4, 30, 0.75),
    # The Leningrad, Novgorod and Pskov regions
    (33.5, 60.2, 30, 0.75), (32.5, 59.0, 30, 0.7), (31.0, 59.2, 25, 0.65), (29.5, 59.2, 25, 0.6),
    (30.5, 58.5, 30, 0.65), (33.5, 58.8, 30, 0.65), (29.0, 58.2, 25, 0.6), (32.5, 57.8, 25, 0.6),
    (33.6, 57.2, 25, 0.55), (30.3, 57.2, 25, 0.55),
    # Estonia
    (25.5, 59.1, 20, 0.55), (26.5, 58.8, 20, 0.6), (27.6, 59.15, 12, 0.6), (23.9, 58.75, 15, 0.5),
    (25.0, 58.2, 18, 0.55), (26.8, 57.75, 15, 0.5), (22.6, 58.4, 10, 0.45), (22.7, 58.9, 8, 0.5),
    # Latvia
    (26.0, 57.0, 20, 0.55), (24.5, 57.4, 15, 0.5), (26.8, 57.5, 15, 0.55),
]


# ---------------------------------------------------------------------------------------------
# Palette


def palette(root):
    """Central Europe's palettes with a darker, cooler (boreal) green ground."""
    src = open(os.path.join(root, "assets", "palettes", "tod_central_europe.ron")).read()
    colour = re.compile(r"\((\d+), (\d+), (\d+)\)")
    starts = [src.index(k) for k in ("day:", "dusk:", "night:")] + [len(src)]
    blocks = [src[starts[k]:starts[k + 1]] for k in range(3)]

    def fix(block):
        cols = [tuple(int(v) for v in c) for c in colour.findall(block)]
        out = list(cols)
        for i in range(0x70, 0x80):
            r, g, b = cols[i]
            out[i] = (round(r * 0.85), round(g * 0.94), min(63, round(b * 1.08 + 1)))
        it = iter(out)
        return colour.sub(lambda m: "(%d, %d, %d)" % next(it), block)

    text = src[:starts[0]] + "".join(fix(b) for b in blocks)
    open(os.path.join(root, "assets", "palettes", "tod_gulf_of_finland.ron"), "w").write(text)
