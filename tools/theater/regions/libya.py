"""Geography of the Libya theater (April 1986): the Libyan coast from the Tunisian border round
the Gulf of Sidra to the Jebel Akhdar, the Sirte Basin oil fields and the Fezzan down to Sabha,
with southern Tunisia, Malta, southern Sicily and the tip of the Peloponnese across the sea.

The theater squeezes 10.5-23.43 E, 26.7-37.66 N (about 1220 by 1220 km) into the 512 km of a
theater. Coordinates here are (lon, lat) degrees.

Sides: Libya is the enemy. Italy (Sicily, Lampedusa, Pantelleria) is the friendly side; Tunisia,
Malta and Greece are friendly ground too but their towns are neutral places. The front is the
Libyan border with Tunisia (quiet) and Qaddafi's "Line of Death", the 32 deg 30' N closing line
of the Gulf of Sidra, fought over in March 1986 (hot in the middle of the gulf).
"""

import math

# The south-west corner of the theater, the latitude the scale is taken at, and how many real
# km a theater km stands for.
PROJECTION = dict(lon0=10.5, lat0=26.7, lat_ref=32.2, squeeze=2.38)

# The object models (tools/theater/libraries), and the rubble a destroyed object leaves on land.
LIBRARY = "libya"
LAND_RUBBLE = 74

# Briefing date of the first mission (month, day, year), grid reference letters and the corner's
# offset, squadrons per enemy airbase at most, the training missions.
DATE = (4, 14, 1986)
GRID = ("TD", 6, 4)
SQUADRONS = 2
# The training range: the Libyan coast nearest the friendly bases, flown from Sigonella; the
# second target a patrol boat off Khoms (a target under every rule of engagement).
TRAINING = {"strike": ["Misratah", "boat 2", "Sigonella", "Sigonella"],
            "air": ["Tripoli/base", "boat 2", "Sigonella", "Sigonella"]}

# The air forces: (aircraft, its air-to-air missile, task), units.ron ids. The first is the
# special aircraft of the intercept missions; the last four are friendly (the carrier's Hornets).
ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("mig25", "aa6", "patrol"),
    ("mig23", "aa2", "escort"),
    ("mig29", "aa10t", "patrol"),
    ("mig25", "aa6t", "patrol"),
    ("mig23", "aa2", "base"),
    ("mig29", "aa10t", "escort"),
    ("f18d", "none", "friendly"),
    ("f18d", "none", "friendly"),
    ("f18", "none", "friendly"),
    ("f18", "none", "friendly"),
]

# Radar types (units.ron ids): the fixed SAM sites (SA-2, SA-5), the mobile ones (SA-8, SA-13)
# and the patrol boats (Nanuchka and Combattante missile boats).
SAM_RADARS = ["sa2", "sa5"]
MOBILE_SAM_RADARS = ["sa8", "sa13"]
BOAT_RADARS = ["san4", "san5"]
SLAG_HEAP_TOWNS = set()

# Models this theater builds besides the water, forests and mountains (see models.py).
EXTRA_MODELS = []

# A desert theater: no green farmland, no stray woods.
GREEN = False
SPARSE_FOREST = False

# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea.

LANDS = [
    # North Africa: Tunisia from Bizerte round Cap Bon and the Sahel, the Gulf of Gabes, the
    # Libyan coast of Tripolitania, the Gulf of Sidra, Cyrenaica to Tobruk, Egypt.
    [
        (5.0, 20.0), (5.0, 36.9), (8.60, 36.94), (9.20, 37.25), (9.87, 37.33), (10.25, 37.18), (10.18, 36.90),
        # Gulf of Tunis and Cap Bon
        (10.31, 36.81), (10.34, 36.73), (10.45, 36.72), (10.57, 36.82), (10.80, 36.95), (10.92, 37.03),
        (11.04, 37.07), (11.12, 36.92), (11.10, 36.85), (10.99, 36.76), (10.86, 36.57), (10.73, 36.45),
        (10.62, 36.40),
        # The Sahel: Sousse, Monastir, Mahdia, Ras Kaboudia, Sfax
        (10.50, 36.15), (10.51, 36.03), (10.64, 35.83), (10.83, 35.77), (10.95, 35.64), (11.07, 35.50),
        (11.10, 35.35), (11.16, 35.24), (11.05, 35.08), (10.90, 34.95), (10.78, 34.74), (10.60, 34.60),
        (10.50, 34.52), (10.25, 34.38), (10.07, 34.29), (10.00, 34.10),
        # Gulf of Gabes, the Bou Grara lagoon behind Djerba, Zarzis, the Biban coast
        (10.10, 33.89), (10.30, 33.70), (10.55, 33.58), (10.75, 33.48), (11.00, 33.55), (11.10, 33.60),
        (11.12, 33.50), (11.22, 33.35), (11.38, 33.22),
        # Tripolitania: Ras Ajdir, Zuwara, Sabratha, Zawiya, Tripoli, Khoms, Zliten, Misratah
        (11.57, 33.17), (11.80, 33.07), (12.08, 32.94), (12.30, 32.85), (12.49, 32.81), (12.73, 32.79),
        (13.00, 32.86), (13.19, 32.91), (13.28, 32.91), (13.35, 32.89), (13.55, 32.82), (13.80, 32.75),
        (14.05, 32.70), (14.26, 32.67), (14.45, 32.55), (14.57, 32.49), (14.85, 32.43), (15.09, 32.41),
        (15.21, 32.38),
        # The Gulf of Sidra: Tawurgha, Buwayrat, Sirt, Nofaliya, Es Sider, Ras Lanuf, Brega, Zuwaytina
        (15.25, 32.20), (15.32, 32.05), (15.40, 31.85), (15.55, 31.60), (15.72, 31.42), (16.00, 31.28),
        (16.30, 31.23), (16.59, 31.23), (16.85, 31.11), (17.10, 31.01), (17.40, 30.91), (17.83, 30.81),
        (18.07, 30.80), (18.35, 30.65), (18.53, 30.52), (18.80, 30.38), (19.20, 30.27), (19.45, 30.31),
        (19.58, 30.43), (19.75, 30.56), (19.95, 30.76), (20.14, 30.96), (20.05, 31.15), (19.97, 31.45),
        (19.95, 31.70), (20.00, 31.95), (20.03, 32.12),
        # Cyrenaica: Benghazi, Tocra, Tolmeita, Susa, Ras al Hilal, Derna, the Gulf of Bomba, Tobruk
        (20.15, 32.25), (20.40, 32.42), (20.57, 32.54), (20.80, 32.65), (20.95, 32.72), (21.30, 32.80),
        (21.65, 32.88), (21.97, 32.91), (22.20, 32.92), (22.40, 32.84), (22.64, 32.78), (22.90, 32.72),
        (23.10, 32.63), (23.12, 32.40), (23.30, 32.25), (23.60, 32.10), (23.96, 32.08), (24.50, 32.00),
        (25.15, 31.65), (27.0, 31.40), (29.9, 31.20), (32.0, 31.30), (35.0, 20.0),
    ],
    # Djerba and the Kerkennah islands.
    [(10.76, 33.75), (10.85, 33.90), (10.98, 33.88), (11.08, 33.78), (11.03, 33.65), (10.92, 33.62),
     (10.80, 33.68)],
    [(11.05, 34.65), (11.20, 34.75), (11.32, 34.84), (11.30, 34.77), (11.15, 34.64), (11.05, 34.61)],
    # Sicily (its northern half is outside the theater).
    [
        (12.50, 38.02), (12.43, 37.80), (12.59, 37.65), (12.66, 37.57), (12.83, 37.58), (13.08, 37.50),
        (13.30, 37.40), (13.52, 37.28), (13.75, 37.15), (13.94, 37.10), (14.25, 37.06), (14.43, 36.90),
        (14.55, 36.78), (14.85, 36.72), (15.13, 36.68), (15.10, 36.85), (15.17, 36.95), (15.29, 37.06),
        (15.22, 37.17), (15.22, 37.24), (15.15, 37.30), (15.10, 37.40), (15.09, 37.50), (15.17, 37.62),
        (15.25, 37.75), (15.29, 37.85), (15.45, 38.05), (15.57, 38.25), (15.20, 38.25), (14.50, 38.05),
        (13.80, 37.98), (13.35, 38.20), (12.70, 38.12),
    ],
    # Malta and Gozo, Lampedusa, Linosa, Pantelleria.
    [(14.33, 35.99), (14.38, 35.97), (14.45, 35.95), (14.52, 35.91), (14.57, 35.87), (14.57, 35.82),
     (14.51, 35.80), (14.43, 35.82), (14.36, 35.84), (14.33, 35.88), (14.32, 35.95)],
    [(14.19, 36.07), (14.26, 36.08), (14.34, 36.04), (14.30, 36.01), (14.22, 36.02), (14.18, 36.04)],
    [(12.51, 35.52), (12.58, 35.53), (12.63, 35.51), (12.62, 35.49), (12.55, 35.49), (12.51, 35.50)],
    [(12.85, 35.86), (12.88, 35.87), (12.89, 35.85), (12.86, 35.84)],
    [(11.93, 36.82), (11.97, 36.84), (12.04, 36.81), (12.06, 36.76), (12.02, 36.73), (11.95, 36.75),
     (11.92, 36.79)],
    # The southern Peloponnese (Messenia, the Mani, Cape Maleas), Kythira, Zakynthos.
    [
        (21.12, 37.94), (21.32, 37.65), (21.55, 37.50), (21.67, 37.25), (21.70, 36.91), (21.70, 36.82),
        (21.88, 36.72), (21.96, 36.80), (22.11, 37.02), (22.23, 36.89), (22.38, 36.67), (22.48, 36.38),
        (22.57, 36.76), (22.70, 36.78), (22.92, 36.52), (23.06, 36.51), (23.20, 36.44), (23.06, 36.69),
        (22.87, 37.17), (22.77, 37.40), (22.80, 37.57), (23.20, 37.40), (23.50, 37.60), (23.40, 38.30),
        (22.00, 38.30), (21.30, 38.20),
    ],
    [(22.92, 36.38), (23.02, 36.37), (23.10, 36.22), (22.99, 36.14), (22.93, 36.25)],
    [(20.62, 37.85), (20.95, 37.75), (20.82, 37.65), (20.72, 37.70), (20.62, 37.80)],
    # Crete (outside the theater, east of it).
    [(23.52, 35.30), (23.55, 35.60), (24.10, 35.60), (24.30, 35.55), (25.00, 35.43), (25.75, 35.33),
     (26.30, 35.28), (26.20, 35.00), (25.50, 34.98), (24.75, 34.93), (24.00, 35.25), (23.60, 35.20)],
]

# Lagoons and lakes: the Bahiret el Biban lagoon on the Tunisian border, Lake Lentini in Sicily.
# Libya's sebkhas are dry salt flats, not water.
LAKES = [
    [(11.15, 33.27), (11.25, 33.30), (11.38, 33.26), (11.40, 33.22), (11.28, 33.20), (11.17, 33.22)],
    [(14.92, 37.35), (14.98, 37.36), (15.00, 37.33), (14.94, 37.32)],
]

# Rivers: polylines of (lon, lat, width in theater km). Libya has no perennial river (its wadis
# are dry); only Sicily's.
RIVERS = [
    # Simeto through the plain of Catania
    [(14.75, 37.68, 0.04), (14.82, 37.55, 0.05), (14.88, 37.45, 0.05), (15.00, 37.40, 0.06), (15.08, 37.40, 0.07)],
    # Salso (Imera Meridionale) to Licata
    [(14.12, 37.55, 0.04), (14.10, 37.40, 0.04), (14.02, 37.25, 0.05), (13.96, 37.10, 0.06)],
    # Platani
    [(13.65, 37.62, 0.03), (13.45, 37.50, 0.04), (13.30, 37.40, 0.05)],
]

# ---------------------------------------------------------------------------------------------
# Sides

# The Libyan-Tunisian border from the Ghadames corner north to Ras Ajdir (quiet), then out to
# sea past Tripoli to the Line of Death across the Gulf of Sidra from Misratah to Benghazi.
FRONT = [
    (9.95, 30.25, 0.0), (10.25, 30.85, 0.0), (10.30, 31.45, 0.0), (10.75, 31.95, 0.0), (11.15, 32.40, 0.0),
    (11.45, 32.65, 0.0), (11.52, 33.00, 0.0), (11.57, 33.17, 0.0), (12.40, 33.40, 0.1), (13.20, 33.40, 0.2),
    (14.30, 33.00, 0.3), (15.25, 32.50, 0.5), (16.40, 32.50, 0.8), (17.50, 32.50, 1.0), (18.70, 32.50, 0.9),
    (19.90, 32.50, 0.6),
]

# The friendly side: Tunisia west of the border, Italy's and Malta's islands, the Peloponnese
# and Crete.
BLUE = [
    # Tunisia (and its sea up to 11.8 E)
    [(5.0, 28.0), (9.60, 30.10)] + [(lon, lat) for lon, lat, _ in FRONT[:8]] + [(11.80, 33.45), (11.80, 37.50),
                                                                                  (5.0, 37.50)],
    # Sicily, Malta, Lampedusa, Linosa, Pantelleria
    [(11.80, 35.20), (13.00, 35.20), (15.00, 35.50), (16.00, 36.40), (16.50, 38.50), (11.80, 38.50)],
    # The Peloponnese, Kythira, Zakynthos, Crete
    [(20.40, 37.40), (21.40, 36.40), (22.60, 35.90), (23.30, 34.70), (26.50, 34.70), (26.50, 38.50), (20.40, 38.50)],
]

# Neutral places: the player never flies from or to them, they are no one's targets.
NEUTRAL = {"Valletta", "Victoria", "Sfax", "Mahdia", "Monastir", "Kelibia", "Zarzis", "Houmt Souk",
           "Ben Gardane", "Kalamata", "Pylos"}

# ---------------------------------------------------------------------------------------------
# Places

# Towns: (name, lon, lat, kind 0 village / 1 town / 2 city, radius in theater metres, ruined
# share, in the world name table).
TOWNS = [
    # Tripolitania and the Jebel Nafusa
    ("Tripoli", 13.19, 32.88, 2, 3300, 0.0, True),
    ("Zawiya", 12.73, 32.76, 1, 1300, 0.0, True),
    ("Sabratha", 12.49, 32.78, 0, 700, 0.0, False),
    ("Zuwara", 12.08, 32.91, 1, 1000, 0.0, False),
    ("Surman", 12.57, 32.75, 0, 600, 0.0, False),
    ("Al Aziziyah", 13.02, 32.53, 0, 700, 0.0, False),
    ("Tarhunah", 13.63, 32.43, 1, 900, 0.0, False),
    ("Khoms", 14.26, 32.64, 1, 1100, 0.0, True),
    ("Zliten", 14.57, 32.46, 1, 1000, 0.0, False),
    ("Misratah", 15.09, 32.37, 2, 2000, 0.0, True),
    ("Gharyan", 13.02, 32.17, 1, 1100, 0.0, True),
    ("Yafran", 12.53, 32.06, 0, 700, 0.0, True),
    ("Zintan", 12.25, 31.93, 0, 700, 0.0, False),
    ("Jadu", 12.03, 31.96, 0, 600, 0.0, False),
    ("Nalut", 10.98, 31.87, 0, 700, 0.0, False),
    ("Mizdah", 12.98, 31.45, 0, 600, 0.0, False),
    ("Bani Walid", 13.98, 31.76, 1, 900, 0.0, False),
    ("Tawurgha", 15.10, 31.98, 0, 600, 0.0, False),
    ("Ash Shwayrif", 14.27, 29.98, 0, 600, 0.0, False),
    # The Gulf of Sidra and the Sirte Basin oil fields
    ("Sirt", 16.59, 31.20, 1, 1400, 0.0, True),
    ("Buwayrat al Hasun", 15.72, 31.38, 0, 600, 0.0, False),
    ("Nofaliya", 17.83, 30.77, 0, 600, 0.0, False),
    ("Bin Jawad", 18.06, 30.77, 0, 600, 0.0, False),
    ("Es Sider", 18.35, 30.62, 0, 700, 0.0, False),
    ("Ras Lanuf", 18.52, 30.48, 1, 1000, 0.0, True),
    ("Al Uqaylah", 19.20, 30.24, 0, 600, 0.0, False),
    ("Port Brega", 19.57, 30.40, 1, 1000, 0.0, True),
    ("Zuwaytina", 20.12, 30.93, 0, 700, 0.0, False),
    ("Ajdabiya", 20.22, 30.76, 1, 1300, 0.0, True),
    ("Maradah", 19.21, 29.23, 0, 700, 0.0, True),
    ("Raguba", 19.08, 29.07, 0, 600, 0.0, True),
    ("Zaltan", 19.77, 28.92, 0, 600, 0.0, False),
    ("Beda", 19.04, 28.43, 0, 600, 0.0, True),
    ("Awjila", 21.29, 29.11, 0, 700, 0.0, False),
    ("Jalo Oasis", 21.55, 29.03, 0, 800, 0.0, True),
    ("Gialo", 21.38, 28.71, 0, 600, 0.0, True),
    ("Amal", 21.12, 29.45, 0, 600, 0.0, True),
    ("Sarir", 22.50, 27.65, 0, 600, 0.0, True),
    ("Zillah", 17.58, 28.55, 0, 600, 0.0, False),
    # The Jufra oases and the Fezzan
    ("Hun", 15.95, 29.12, 1, 900, 0.0, True),
    ("Waddan", 16.13, 29.16, 0, 700, 0.0, False),
    ("Sokna", 15.79, 29.07, 0, 600, 0.0, False),
    ("Sabha", 14.43, 27.04, 2, 1600, 0.0, True),
    ("Brak", 14.27, 27.55, 0, 700, 0.0, False),
    ("Idri", 13.05, 27.46, 0, 600, 0.0, False),
    # Cyrenaica and the Jebel Akhdar
    ("Benghazi", 20.08, 32.11, 2, 2800, 0.0, True),
    ("Qaminis", 20.03, 31.66, 0, 600, 0.0, False),
    ("Al Abyar", 20.60, 32.19, 0, 600, 0.0, False),
    ("Tocra", 20.57, 32.52, 0, 700, 0.0, False),
    ("Al Marj", 20.83, 32.49, 1, 1100, 0.0, True),
    ("Tolmeita", 20.95, 32.70, 0, 500, 0.0, False),
    ("Al Bayda", 21.76, 32.76, 1, 1300, 0.0, True),
    ("Shahhat", 21.86, 32.82, 0, 700, 0.0, False),
    ("Susa", 21.96, 32.89, 0, 600, 0.0, False),
    ("Al Qubah", 22.25, 32.76, 0, 700, 0.0, False),
    ("Derna", 22.64, 32.75, 1, 1300, 0.0, True),
    ("Martuba", 22.75, 32.58, 0, 600, 0.0, False),
    ("Al Mukhayli", 22.27, 32.17, 0, 600, 0.0, True),
    # Tunisia
    ("Sfax", 10.74, 34.75, 2, 2000, 0.0, False),
    ("Mahdia", 11.05, 35.50, 1, 900, 0.0, False),
    ("Monastir", 10.81, 35.76, 1, 1000, 0.0, False),
    ("Kelibia", 11.08, 36.85, 0, 700, 0.0, False),
    ("Houmt Souk", 10.86, 33.87, 0, 800, 0.0, False),
    ("Zarzis", 11.09, 33.50, 1, 900, 0.0, False),
    ("Ben Gardane", 11.22, 33.14, 0, 800, 0.0, False),
    # Malta and the Italian islands
    ("Valletta", 14.51, 35.90, 2, 1800, 0.0, True),
    ("Victoria", 14.24, 36.04, 0, 600, 0.0, False),
    ("Lampedusa", 12.61, 35.505, 0, 500, 0.0, False),
    ("Pantelleria", 11.95, 36.81, 0, 600, 0.0, False),
    # Sicily
    ("Catania", 15.07, 37.51, 2, 2600, 0.0, True),
    ("Siracusa", 15.27, 37.07, 2, 1700, 0.0, True),
    ("Augusta", 15.20, 37.23, 1, 900, 0.0, False),
    ("Ragusa", 14.73, 36.93, 1, 1100, 0.0, False),
    ("Gela", 14.25, 37.07, 1, 1100, 0.0, False),
    ("Licata", 13.92, 37.11, 1, 800, 0.0, False),
    ("Agrigento", 13.58, 37.31, 1, 1200, 0.0, False),
    ("Sciacca", 13.08, 37.51, 1, 800, 0.0, False),
    ("Caltanissetta", 14.06, 37.49, 1, 1000, 0.0, False),
    # The Peloponnese
    ("Kalamata", 22.11, 37.04, 1, 1100, 0.0, False),
    ("Pylos", 21.72, 36.91, 0, 500, 0.0, False),
]

# Airbases: (name, lon, lat, side: True friendly, False enemy, None neutral). The runway runs
# north-south.
AIRBASES = [
    ("Tripoli", 13.276, 32.894, False),     # Okba Ben Nafi (Mitiga, the old Wheelus AFB)
    ("Port Brega", 19.576, 30.378, False),  # Marsa Brega
    ("Benghazi", 20.269, 32.097, False),    # Benina
    ("Sabha", 14.472, 27.017, False),
    ("Al Bayda", 21.964, 32.789, False),    # La Abraq
    ("Sigonella", 14.922, 37.402, True),    # NAS Sigonella, Sicily
    ("Hal Far", 14.527, 35.813, None),      # Malta
]

# Small strips: on the Jebel Nafusa by Yafran and the old desert field at El Mechili.
AIRSTRIPS = [
    ("Yafran", 12.55, 31.99, False),
    ("Al Mukhayli", 22.27, 32.14, False),
]

# The friendly carrier group in the central Mediterranean north of the Gulf of Sidra.
CARRIER = ("USS America", 18.08, 34.45)

# The theater's own site layouts within a 2 km cell: (metres east, metres north, kind). The
# library has no depots, houses or bridges; its special targets are the oil industry, the
# terrorist training camps, the SS missile brigades and the nuclear centre.
TEMPLATES = {
    # An army or Jamahiriya Guard headquarters with its barracks and air defence.
    "command": [(0, 0, "headquarters"), (-400, -150, "camp"), (450, 300, "tank_farm"), (0, 480, "mobile_sam"),
                (0, -480, "mobile_sam"), (-600, 450, "tank"), (-450, 520, "tank")],
    # An SS-1 Scud battery: three launchers, their fuel and a mobile SAM.
    "scud": [(0, 0, "ss_missile"), (160, -120, "ss_missile"), (-160, -120, "ss_missile"), (0, -500, "tank_farm"),
             (420, 320, "mobile_sam")],
    # A training camp.
    "camp": [(0, 0, "camp"), (380, 200, "camp"), (-400, -350, "tank"), (-250, -420, "tank")],
    # Armour along a desert track, with its air defence and camp.
    "armour": [(-700 + 180 * k, -500 + 120 * k, "tank") for k in range(6)] + [(400, 300, "mobile_sam"),
                                                                            (700, -300, "camp")],
    # An air defence battery: its radar, five launchers on a pentagon and three mobile SAMs.
    "battery": [(0, 0, "sam_radar")] + [(round(420 * math.sin(math.radians(72 * k))),
                                         round(420 * math.cos(math.radians(72 * k))),
                                         "missile") for k in range(5)]
               + [(-200, -150, "mobile_sam"), (200, -150, "mobile_sam"), (0, 220, "mobile_sam")],
    # A desert oil field: wells, gathering pump stations and the field's tank farms.
    "oilfield": [(-600, 600, "oil_rig"), (0, 650, "oil_rig"), (600, 600, "oil_rig"), (-650, 0, "oil_rig"),
                 (650, 0, "oil_rig"), (-600, -600, "oil_rig"), (600, -600, "oil_rig"), (0, 0, "pump_station"),
                 (-200, -350, "tank_farm"), (200, -350, "tank_farm")],
    # A coastal refinery with its export tank farms.
    "terminal": [(0, 0, "refinery"), (-450, 250, "tank_farm"), (-450, -150, "tank_farm"), (-150, -500, "tank_farm"),
                 (400, 350, "tank_farm"), (450, -350, "pump_station")],
    # An export terminal without a refinery.
    "oilport": [(-250, -250, "tank_farm"), (250, -250, "tank_farm"), (-250, 250, "tank_farm"), (250, 250, "tank_farm"),
                (650, 0, "pump_station"), (-650, 0, "pump_station")],
    # A fuel depot.
    "fuel": [(-150, -150, "tank_farm"), (150, -150, "tank_farm"), (-150, 150, "tank_farm"), (150, 150, "tank_farm"),
             (500, 0, "pump_station")],
}

# Enemy target sites: (template, lon, lat, name).
SITES = [
    # Headquarters and barracks: Bab al Aziziya in Tripoli and the Jamahiriya barracks in
    # Benghazi (the targets of 15 April 1986), the regional commands.
    ("command", 13.17, 32.86, "Bab al Aziziya"), ("command", 20.10, 32.08, "Benghazi"),
    ("command", 15.05, 32.33, "Misratah"), ("command", 16.62, 31.16, "Sirt"), ("command", 18.48, 30.45, "Ras Lanuf"),
    ("command", 19.62, 30.36, "Port Brega"), ("command", 19.24, 29.26, "Maradah"), ("command", 14.45, 27.08, "Sabha"),
    ("command", 21.72, 32.73, "Al Bayda"), ("command", 15.97, 29.08, "Hun"), ("command", 13.04, 32.14, "Gharyan"),
    # SS-1 Scud brigades; Zuwara's fired at Lampedusa on 15 April 1986.
    ("scud", 12.15, 32.85, "Zuwara"), ("scud", 13.62, 32.40, "Tarhunah"), ("scud", 15.00, 32.25, "Misratah"),
    ("scud", 16.50, 31.10, "Sirt"), ("scud", 18.30, 30.55, "Es Sider"), ("scud", 19.75, 30.30, "Port Brega"),
    ("scud", 19.30, 29.30, "Maradah"), ("scud", 20.40, 32.05, "Benghazi"), ("scud", 21.65, 32.65, "Al Bayda"),
    ("scud", 14.50, 27.15, "Sabha"),
    # Training camps: Sidi Bilal (raided in 1986), Ras al Hilal's naval commandos and others.
    ("camp", 13.03, 32.84, "Sidi Bilal"), ("camp", 22.15, 32.87, "Ras al Hilal"), ("camp", 13.70, 32.45, "Tarhunah"),
    ("camp", 14.55, 27.00, "Sabha"), ("camp", 19.15, 29.18, "Maradah"), ("camp", 20.60, 32.15, "Al Abyar"),
    # Armour: the southern army in the Fezzan (the war in Chad), the coast's brigades.
    ("armour", 14.30, 26.85, "Sabha"), ("armour", 14.70, 26.88, "Sabha"), ("armour", 14.25, 27.50, "Brak"),
    ("armour", 16.05, 29.05, "Waddan"), ("armour", 20.25, 30.70, "Ajdabiya"), ("armour", 20.90, 32.45, "Al Marj"),
    ("armour", 12.90, 32.25, "Gharyan"), ("armour", 16.70, 31.05, "Sirt"), ("armour", 12.70, 32.60, "Zawiya"),
    # Air defence batteries
    ("battery", 15.15, 32.30, "Misratah"), ("battery", 13.40, 32.80, "Tajura"), ("battery", 21.85, 32.70, "Al Bayda"),
    ("battery", 20.20, 32.00, "Benghazi"),
    ("msam", 12.75, 32.70, "Zawiya"), ("msam", 14.30, 32.58, "Khoms"), ("msam", 17.85, 30.70, "Nofaliya"),
    ("msam", 20.15, 30.80, "Ajdabiya"), ("msam", 22.60, 32.70, "Derna"), ("msam", 14.60, 32.40, "Zliten"),
    # The Sirte Basin oil fields
    ("oilfield", 19.04, 28.43, "Beda"), ("oilfield", 22.50, 27.65, "Sarir"), ("oilfield", 22.30, 27.80, "Sarir"),
    ("oilfield", 21.38, 28.71, "Gialo"), ("oilfield", 19.08, 29.07, "Raguba"), ("oilfield", 21.12, 29.45, "Amal"),
    ("oilfield", 19.77, 28.92, "Zaltan"), ("oilfield", 17.75, 29.48, "Dahra"), ("oilfield", 19.95, 28.55, "Waha"),
    ("oilfield", 19.90, 28.05, "Defa"), ("oilfield", 21.00, 29.00, "Intisar"), ("oilfield", 21.55, 29.25, "Nafoora"),
    ("oilfield", 22.10, 28.80, "Bu Attifel"),
    # Refineries and export terminals
    ("terminal", 18.55, 30.47, "Ras Lanuf"), ("terminal", 19.55, 30.40, "Port Brega"),
    ("terminal", 12.67, 32.76, "Zawiya"), ("oilport", 18.37, 30.61, "Es Sider"),
    ("oilport", 20.16, 30.90, "Zuwaytina"), ("fuel", 14.40, 27.00, "Sabha"), ("fuel", 13.10, 32.82, "Tripoli"),
]

# Single objects: the Tajura nuclear research centre and the planned nuclear plant on the Gulf
# of Sidra, the Bouri offshore field's platforms, the pipelines' pump stations.
OBJECTS = [
    ("nuclear_plant", 13.37, 32.82, "Tajura"),
    ("nuclear_plant", 16.80, 31.06, "Sirt"),
    ("oil_rig", 12.60, 33.88, "Bouri"), ("oil_rig", 12.66, 33.92, "Bouri"), ("oil_rig", 12.70, 33.86, "Bouri"),
    ("oil_rig", 12.63, 33.95, "Bouri"), ("oil_rig", 12.57, 33.93, "Bouri"), ("oil_rig", 12.74, 33.91, "Bouri"),
    # Pump stations: Zaltan-Brega, Raguba-Brega, Dahra-Es Sider, Amal-Ras Lanuf, Intisar-Zuwaytina, Sarir-Hariga
    ("pump_station", 19.70, 29.60, "Zaltan"), ("pump_station", 19.65, 30.05, "Zaltan"),
    ("pump_station", 19.25, 29.65, "Raguba"), ("pump_station", 17.95, 29.95, "Dahra"),
    ("pump_station", 18.20, 30.35, "Dahra"), ("pump_station", 20.40, 29.75, "Amal"),
    ("pump_station", 19.50, 30.10, "Amal"), ("pump_station", 20.50, 30.05, "Intisar"),
    ("pump_station", 22.75, 28.40, "Sarir"), ("pump_station", 23.10, 29.20, "Sarir"),
    ("pump_station", 21.20, 28.10, "Messla"), ("pump_station", 20.30, 29.10, "Gialo"),
]

# Cities of the enemy side with a SAM site at their edge (the airbases have their own).
CITY_SAMS = ["Misratah", "Sirt", "Ras Lanuf", "Maradah", "Ajdabiya", "Hun", "Derna", "Zawiya"]

# Patrol boats: (lon, lat): off Zuwara, Tripoli, Khoms, Misratah, in the Gulf of Sidra, off
# Benghazi and Derna.
BOATS = [(12.20, 33.15), (13.30, 33.15), (14.50, 32.90), (15.45, 32.55), (17.50, 31.50), (18.80, 31.00),
         (19.75, 32.30), (22.40, 33.10)]

# Other ships: tankers at the oil terminals and in the shipping lanes, the carrier's destroyers.
SHIPS = [
    ("oil_tanker", 18.40, 30.75, None), ("oil_tanker", 18.60, 30.62, None), ("oil_tanker", 19.55, 30.55, None),
    ("oil_tanker", 19.72, 30.58, None), ("oil_tanker", 19.98, 31.02, None), ("oil_tanker", 12.70, 32.92, None),
    ("oil_tanker", 13.15, 33.05, None), ("oil_tanker", 16.00, 33.80, None), ("oil_tanker", 19.00, 34.70, None),
    ("oil_tanker", 21.50, 34.20, None), ("oil_tanker", 14.20, 35.20, None), ("oil_tanker", 12.00, 35.40, None),
    ("destroyer", 18.15, 34.48, None), ("destroyer", 18.02, 34.50, None), ("destroyer", 18.10, 34.38, None),
    ("destroyer", 17.98, 34.40, None), ("destroyer", 18.20, 34.41, None),
]

# Mountain ranges: (lon, lat, length theater km, width theater km, height m, bearing of the
# long axis in degrees from north).
MOUNTAINS = [
    # The Jebel Nafusa escarpment from Nalut past Yafran to Gharyan and Tarhunah
    (11.25, 31.90, 28, 8, 700, 80), (12.00, 31.98, 30, 8, 900, 80), (12.75, 32.10, 28, 8, 850, 75),
    (13.40, 32.35, 22, 8, 500, 60),
    # The Jebel Akhdar from Al Marj to Derna
    (20.95, 32.50, 28, 14, 600, 70), (21.55, 32.66, 30, 14, 880, 80), (22.20, 32.70, 28, 12, 700, 85),
    # Jabal as Sawda, Jabal Hasawinah, the Haruj volcanic field
    (15.50, 28.90, 32, 14, 800, 110), (16.30, 28.70, 25, 12, 700, 110), (13.80, 28.30, 30, 12, 800, 80),
    (17.20, 27.80, 35, 28, 1000, 140), (17.80, 27.30, 35, 28, 1200, 140),
    # Sicily: the Iblei, the Erei, the Sicani, Etna (its summit just outside the theater's edge)
    (14.85, 37.10, 22, 14, 986, 40), (14.35, 37.50, 20, 12, 900, 20), (13.35, 37.60, 20, 10, 1500, 80),
    (15.00, 37.64, 18, 16, 3300, 0),
    # The Taygetos in the Peloponnese
    (22.35, 36.95, 24, 8, 2400, 170),
]

# Forest regions: (lon, lat, radius theater km, density 0..1): the juniper and maquis of the
# Jebel Akhdar.
FORESTS = [(21.65, 32.68, 9, 0.3), (21.05, 32.55, 6, 0.2)]


# ---------------------------------------------------------------------------------------------
# Models of this theater's library the layouts use, and its airfield's aprons.

KINDS = {"houses": 39, "houses2": 39, "buildings": 39, "patrol_boat": 54, "land": 0, "sea": 29, "carrier": 63,
         "carrier_object": 87, "airstrip": 84}
APRONS = [(-500, -78, -219, 31), (78, 500, 250, 500)]
LAND_COLOUR = 54

# The empty open land: a hamlet here and there.
TEMPLATES["hamlet"] = [(0, 0, "houses"), (170, -120, "houses")]
TEMPLATES["lone_house"] = [(0, 0, "houses")]
FILLER = [{}, {}, {}, {(1, 2): "hamlet"}, {(2, 1): "lone_house"}]
