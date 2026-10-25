"""Geography of the Middle East theater (1990): Syria, Lebanon, Jordan and northern Iraq with
Israel, Cyprus, the Turkish south-east, the tip of Sinai and the north of Saudi Arabia.

The theater squeezes 32.25-45.2 E, 28.0-38.8 N (about 1200 by 1200 km) into the 512 km of a
theater. Coordinates here are (lon, lat) degrees.

The original theater was drawn turned about 53 degrees anticlockwise (Turkey to the north-west,
Iraq to the north-east, Saudi Arabia to the south-east); an unrotated projection cannot keep
that, so the window is the one that holds all of the theater's named places at their real
positions with a margin, at the original's scale.

Sides: Israel with the territories it held in 1990 (the West Bank, Gaza, the Golan), Turkey,
Cyprus with the British base at Akrotiri, and neutral Egypt and Saudi Arabia (their towns are
neutral ground) are the friendly side; Syria, Lebanon, Jordan and Iraq the enemy. There is no
shooting war in October 1990 (the Gulf crisis is still a standoff): the front line marks
Israel's borders, at a low intensity along the Lebanese border (the fighting in the south
Lebanon security zone) and 0 elsewhere.
"""

PROJECTION = dict(lon0=32.25, lat0=28.0, lat_ref=33.4, squeeze=2.35)

LIBRARY = "middle_east"
LAND_RUBBLE = 78

DATE = (10, 6, 1990)
GRID = ("ES", 0, 0)
SQUADRONS = 5

ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("mig31", "aa6", "patrol"),
    ("f5", "aa2", "escort"),
    ("mig31", "aa6t", "patrol"),
    ("mig29", "aa10t", "base"),
    ("f5", "aa2", "base"),
    ("mig23", "aa2", "escort"),
    ("f18d", "none", "friendly"),
    ("f18d", "none", "friendly"),
    ("f18", "none", "friendly"),
    ("f18", "none", "friendly"),
]

# Radar types (units.ron ids): the fixed SAM sites (SA-2 and SA-5), the mobile ones (SA-8 and
# SA-13, both in Syrian and Iraqi service) and the patrol boats (the theater has none).
SAM_RADARS = ["sa2", "sa5"]
MOBILE_SAM_RADARS = ["sa8", "sa13"]
BOAT_RADARS = []
SLAG_HEAP_TOWNS = set()

# Models this theater builds besides the water, forests and mountains (see models.py).
EXTRA_MODELS = []

# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea.

LANDS = [
    # The continent: from the Red Sea up the Gulf of Suez and back, round Sinai and up the Gulf
    # of Aqaba and back, down the Saudi coast, far round the east and north, then the south
    # coast of Turkey eastwards, the Levant coast south, Sinai's north coast and the Nile delta.
    [
        (20.0, 24.0), (34.40, 25.60), (34.28, 26.10), (33.95, 26.60), (33.82, 27.25), (33.55, 27.80),
        (33.10, 28.36), (32.85, 28.75), (32.65, 29.12), (32.40, 29.50), (32.35, 29.65), (32.45, 29.88),
        (32.55, 29.97), (32.60, 29.90), (32.70, 29.60), (32.88, 29.35), (33.10, 29.05), (33.18, 28.90),
        (33.40, 28.55), (33.62, 28.24), (33.85, 28.00), (34.10, 27.80), (34.25, 27.73), (34.33, 27.88),
        (34.42, 28.15), (34.52, 28.50), (34.63, 28.80), (34.66, 28.97), (34.75, 29.20), (34.89, 29.49),
        (34.97, 29.55), (35.00, 29.52), (35.02, 29.40), (34.96, 29.30), (34.88, 29.10), (34.80, 28.80),
        (34.75, 28.40), (34.62, 27.98), (34.85, 28.08), (35.25, 27.95), (35.69, 27.35), (36.45, 26.23),
        (37.50, 24.50), (55.0, 24.0), (55.0, 46.0), (26.0, 46.0), (26.0, 38.5), (27.3, 37.0),
        (28.00, 36.70), (29.60, 36.20), (30.45, 36.30), (30.60, 36.75), (30.70, 36.88), (31.30, 36.80),
        (31.75, 36.65), (32.00, 36.54), (32.30, 36.27), (32.55, 36.10), (32.80, 36.02), (33.05, 36.10),
        (33.32, 36.14), (33.55, 36.17), (33.70, 36.20), (33.88, 36.32), (33.98, 36.28), (34.10, 36.40),
        (34.30, 36.58), (34.63, 36.79), (34.90, 36.73), (35.10, 36.66), (35.38, 36.56), (35.55, 36.57),
        (35.79, 36.77), (35.95, 36.88), (36.05, 36.93), (36.20, 36.80), (36.17, 36.59), (35.98, 36.50),
        (35.88, 36.41), (35.80, 36.28), (35.93, 36.12), (35.96, 36.04), (35.92, 35.93), (35.83, 35.85),
        (35.78, 35.70), (35.77, 35.52), (35.87, 35.40), (35.92, 35.36), (35.93, 35.18), (35.92, 35.00),
        (35.87, 34.89), (35.94, 34.72), (35.97, 34.64), (35.90, 34.53), (35.81, 34.46), (35.72, 34.33),
        (35.65, 34.25), (35.64, 34.12), (35.62, 33.98), (35.55, 33.90), (35.47, 33.90), (35.48, 33.83),
        (35.45, 33.70), (35.37, 33.56), (35.27, 33.42), (35.20, 33.27), (35.10, 33.09), (35.09, 33.01),
        (35.07, 32.93), (35.08, 32.87), (35.04, 32.82), (34.96, 32.83), (34.94, 32.70), (34.89, 32.50),
        (34.85, 32.33), (34.77, 32.10), (34.72, 31.95), (34.64, 31.80), (34.55, 31.67), (34.45, 31.52),
        (34.33, 31.40), (34.24, 31.30), (34.00, 31.20), (33.80, 31.13), (33.45, 31.12), (33.10, 31.22),
        (32.80, 31.21), (32.60, 31.08), (32.45, 31.15), (32.30, 31.27), (31.80, 31.52), (31.00, 31.60),
        (30.00, 31.30), (29.90, 31.20), (25.0, 31.60), (20.0, 31.0),
    ],
    # Cyprus, from Cape Andreas along the Karpas and the south coast to Cape Arnauti and back
    # along the north coast.
    [
        (34.59, 35.69), (34.40, 35.53), (34.10, 35.40), (33.93, 35.28), (33.95, 35.12), (34.08, 34.96),
        (33.85, 34.98), (33.63, 34.91), (33.62, 34.82), (33.30, 34.71), (33.04, 34.67), (33.05, 34.56),
        (32.94, 34.57), (32.88, 34.62), (32.75, 34.65), (32.40, 34.72), (32.32, 34.90), (32.28, 35.10),
        (32.42, 35.04), (32.70, 35.17), (32.88, 35.18), (32.92, 35.40), (33.32, 35.34), (33.70, 35.37),
        (34.05, 35.47), (34.35, 35.60),
    ],
]

# Lakes, lagoons and reservoirs.
LAKES = [
    # The Dead Sea (with the evaporation ponds of its southern basin), the Sea of Galilee
    [(35.47, 31.77), (35.56, 31.77), (35.59, 31.65), (35.57, 31.40), (35.53, 31.25), (35.50, 31.10),
     (35.42, 31.12), (35.40, 31.30), (35.46, 31.48), (35.42, 31.62)],
    [(35.52, 32.89), (35.62, 32.87), (35.65, 32.75), (35.58, 32.70), (35.53, 32.75)],
    # Lake Bardawil on the Sinai coast, the Great Bitter Lake on the Suez Canal
    [(32.75, 31.15), (33.10, 31.17), (33.40, 31.08), (33.30, 31.03), (33.00, 31.04), (32.80, 31.07)],
    [(32.30, 30.42), (32.42, 30.38), (32.45, 30.30), (32.35, 30.27), (32.28, 30.35)],
    # Syria: Lake Assad behind the Tabqa dam, Lake Qattinah by Homs
    [(38.57, 35.87), (38.45, 35.92), (38.25, 35.98), (38.10, 36.02), (38.02, 36.10), (38.10, 36.14),
     (38.30, 36.06), (38.50, 35.98), (38.59, 35.91)],
    [(36.54, 34.62), (36.61, 34.63), (36.62, 34.68), (36.56, 34.69)],
    # Iraq: the Qadisiyah reservoir behind the Haditha dam, Lake Tharthar, Lake Habbaniyah,
    # Lake Razzaza, the Mosul and Dukan reservoirs
    [(42.35, 34.18), (42.25, 34.25), (42.10, 34.35), (41.98, 34.42), (42.05, 34.46), (42.20, 34.38),
     (42.37, 34.24)],
    [(43.35, 34.18), (43.20, 34.05), (43.05, 33.90), (43.05, 33.75), (43.20, 33.65), (43.45, 33.68),
     (43.55, 33.80), (43.50, 34.00)],
    [(43.45, 33.29), (43.57, 33.27), (43.62, 33.18), (43.53, 33.12), (43.42, 33.17)],
    [(43.70, 32.88), (43.95, 32.80), (44.00, 32.68), (43.85, 32.55), (43.65, 32.62), (43.58, 32.75)],
    [(42.82, 36.63), (42.75, 36.72), (42.62, 36.82), (42.55, 36.90), (42.62, 36.93), (42.72, 36.83),
     (42.86, 36.70)],
    [(44.95, 35.92), (45.02, 35.97), (45.00, 36.10), (44.92, 36.13), (44.90, 36.02)],
    # Turkey: Lake Van, the Ataturk reservoir (filling since January 1990), Lake Hazar, the
    # Keban reservoir at the top edge
    [(42.30, 38.50), (42.55, 38.40), (42.90, 38.32), (43.15, 38.35), (43.35, 38.48), (43.40, 38.70),
     (43.30, 38.90), (43.35, 39.03), (43.10, 39.00), (42.85, 38.85), (42.60, 38.75), (42.35, 38.62)],
    [(38.32, 37.48), (38.45, 37.55), (38.65, 37.65), (38.85, 37.80), (38.95, 37.92), (38.85, 37.95),
     (38.60, 37.80), (38.35, 37.65), (38.25, 37.55)],
    [(39.25, 38.47), (39.42, 38.46), (39.50, 38.50), (39.35, 38.51)],
    [(38.75, 38.80), (39.00, 38.83), (39.30, 38.79), (39.50, 38.88), (39.50, 39.00), (39.00, 38.95),
     (38.75, 38.90)],
]

# Rivers: polylines of (lon, lat, width in theater km).
RIVERS = [
    # Euphrates from Keban past the Ataturk dam, Birecik, Lake Assad, Raqqa, Deir ez-Zor,
    # Al Qaim, Haditha, Hit, Ramadi and Fallujah, then south past Hillah
    [(38.90, 38.80, 0.08), (38.75, 38.55, 0.1), (38.60, 38.30, 0.1), (38.70, 38.05, 0.1), (38.95, 37.92, 0.1),
     (38.32, 37.48, 0.12), (38.05, 37.20, 0.14), (37.98, 37.03, 0.15), (38.01, 36.82, 0.15), (38.05, 36.55, 0.15),
     (38.02, 36.10, 0.15), (38.57, 35.87, 0.15), (38.80, 35.92, 0.16), (39.01, 35.94, 0.16), (39.40, 35.80, 0.16),
     (39.80, 35.55, 0.16), (40.14, 35.33, 0.17), (40.45, 35.02, 0.17), (40.70, 34.75, 0.17), (40.92, 34.45, 0.17),
     (41.10, 34.38, 0.17), (41.50, 34.42, 0.17), (41.98, 34.42, 0.17), (42.36, 34.16, 0.17), (42.60, 33.90, 0.17),
     (42.83, 33.64, 0.18), (43.05, 33.48, 0.18), (43.30, 33.43, 0.18), (43.55, 33.36, 0.18), (43.78, 33.36, 0.18),
     (44.05, 33.05, 0.18), (44.23, 32.78, 0.18), (44.43, 32.48, 0.16), (44.60, 32.10, 0.16), (45.00, 31.80, 0.16),
     (45.40, 31.40, 0.16)],
    # Tigris from Diyarbakir past Cizre, the Mosul dam, Mosul, Baiji, Tikrit, Samarra and Baghdad
    [(40.22, 37.91, 0.08), (40.90, 37.80, 0.1), (41.50, 37.85, 0.11), (41.95, 37.60, 0.12), (42.19, 37.32, 0.13),
     (42.36, 37.11, 0.14), (42.60, 36.95, 0.14), (42.82, 36.63, 0.15), (43.00, 36.45, 0.15), (43.14, 36.34, 0.16),
     (43.22, 36.20, 0.16), (43.29, 35.80, 0.16), (43.40, 35.30, 0.17), (43.49, 34.93, 0.17), (43.69, 34.60, 0.17),
     (43.88, 34.20, 0.18), (44.10, 33.80, 0.18), (44.30, 33.50, 0.18), (44.37, 33.32, 0.18), (44.45, 33.28, 0.18),
     (44.55, 33.18, 0.18), (44.95, 32.85, 0.18), (45.40, 32.55, 0.18)],
    # Khabur from Ras al-Ayn past Hasakah to the Euphrates, the Balikh
    [(40.07, 36.85, 0.04), (40.40, 36.60, 0.05), (40.75, 36.50, 0.05), (40.85, 36.10, 0.06), (40.70, 35.70, 0.06),
     (40.43, 35.15, 0.06)],
    [(39.00, 36.70, 0.04), (39.05, 36.30, 0.04), (39.07, 35.95, 0.05)],
    # Great Zab, Little Zab and Diyala to the Tigris
    [(44.10, 36.85, 0.08), (43.70, 36.30, 0.09), (43.35, 36.00, 0.1)],
    [(44.92, 35.92, 0.06), (44.55, 35.85, 0.07), (44.15, 35.76, 0.07), (43.80, 35.45, 0.08), (43.40, 35.25, 0.08)],
    [(45.10, 34.60, 0.07), (44.85, 34.20, 0.08), (44.65, 33.75, 0.08), (44.50, 33.25, 0.08)],
    # Orontes from the Bekaa past Lake Qattinah, Homs, Hama, the Ghab and Antakya to the sea
    [(36.35, 34.35, 0.04), (36.54, 34.62, 0.04), (36.70, 34.78, 0.05), (36.75, 35.13, 0.05), (36.55, 35.30, 0.05),
     (36.38, 35.55, 0.05), (36.32, 35.81, 0.05), (36.45, 36.05, 0.05), (36.16, 36.20, 0.06), (35.96, 36.04, 0.07)],
    # Litani, Jordan and Yarmouk
    [(36.10, 33.85, 0.03), (35.75, 33.45, 0.03), (35.55, 33.34, 0.03), (35.25, 33.34, 0.04)],
    [(35.62, 33.10, 0.03), (35.60, 32.89, 0.03), (35.58, 32.70, 0.03), (35.57, 32.30, 0.04), (35.55, 31.78, 0.04)],
    [(36.05, 32.75, 0.03), (35.75, 32.70, 0.03), (35.58, 32.68, 0.03)],
    # Seyhan through Adana and Ceyhan to the Gulf of Iskenderun
    [(35.40, 37.30, 0.06), (35.32, 37.00, 0.07), (35.10, 36.85, 0.08), (34.92, 36.74, 0.08)],
    [(36.40, 37.40, 0.06), (36.00, 37.10, 0.07), (35.80, 36.90, 0.07), (35.62, 36.62, 0.08)],
    # The Suez Canal
    [(32.31, 31.24, 0.12), (32.30, 30.85, 0.12), (32.28, 30.58, 0.12), (32.30, 30.40, 0.12), (32.50, 30.15, 0.12),
     (32.56, 29.97, 0.12)],
]

# ---------------------------------------------------------------------------------------------
# Sides

# Israel's borders: the Lebanese border from Rosh HaNikra (the south Lebanon security zone
# beyond it), the Golan ceasefire line, the Yarmouk, the Jordan, the Dead Sea and the Arava
# down to Aqaba: (lon, lat, intensity).
FRONT = [
    (35.10, 33.09, 0.2), (35.20, 33.08, 0.2), (35.37, 33.06, 0.2), (35.50, 33.09, 0.2), (35.55, 33.26, 0.2),
    (35.62, 33.28, 0.2), (35.75, 33.33, 0.1), (35.82, 33.25, 0.0), (35.80, 33.13, 0.0), (35.85, 32.98, 0.0),
    (35.84, 32.82, 0.0), (35.75, 32.72, 0.0), (35.58, 32.65, 0.0), (35.55, 32.40, 0.0), (35.55, 32.10, 0.0),
    (35.53, 31.80, 0.0), (35.48, 31.40, 0.0), (35.40, 31.05, 0.0), (35.30, 30.80, 0.0), (35.18, 30.40, 0.0),
    (35.08, 30.00, 0.0), (35.00, 29.55, 0.0),
]

BLUE = [
    # Israel with the West Bank, Gaza and the Golan, Egypt and Saudi Arabia: Israel's border
    # down to Aqaba, the borders of Saudi Arabia with Jordan and Iraq, round the south and back
    # over the sea.
    [(lon, lat) for lon, lat, _ in FRONT]
    + [(34.96, 29.36), (36.07, 29.19), (36.50, 29.50), (36.76, 29.87), (37.50, 30.00), (37.67, 30.34),
       (37.00, 31.50), (38.00, 31.83), (39.20, 32.15), (40.41, 31.95), (41.44, 31.37), (42.08, 31.11),
       (42.86, 30.50), (44.72, 29.20), (46.50, 29.10), (46.5, 20.0), (25.0, 20.0), (25.0, 33.0),
       (34.00, 33.20)],
    # Turkey: its borders with Syria from the coast below Samandag (Hatay is Turkish), with Iraq
    # and with Iran, then round the north and back over the sea.
    [(35.92, 35.93), (36.00, 35.88), (36.15, 35.85), (36.38, 36.18), (36.66, 36.22), (36.66, 36.83),
     (37.10, 36.65), (37.50, 36.65), (38.00, 36.83), (38.50, 36.70), (38.95, 36.69), (39.50, 36.72),
     (40.07, 36.84), (40.60, 36.95), (41.22, 37.07), (41.80, 37.10), (42.36, 37.11), (42.78, 37.33),
     (43.30, 37.32), (44.00, 37.30), (44.25, 37.15), (44.79, 37.15), (44.60, 37.75), (44.40, 38.40),
     (44.30, 39.00), (44.50, 40.00), (45.0, 46.0), (26.0, 46.0), (26.0, 35.5), (35.50, 35.95)],
    # Cyprus with the British sovereign bases.
    [(32.0, 34.4), (34.8, 34.4), (34.8, 35.9), (32.0, 35.9)],
]

# Neutral places: the player never flies from or to them, they are no one's targets.
NEUTRAL = {"Tabuk", "Sakakah", "Arar", "Turayf", "Al Qurayyat", "Haql", "El Arish", "Suez", "Nicosia"}

# ---------------------------------------------------------------------------------------------
# Places

# Towns: (name, lon, lat, kind 0 village / 1 town / 2 city, radius in theater metres, ruined
# share, in the world name table).
TOWNS = [
    # Syria
    ("Damascus", 36.29, 33.51, 2, 3400, 0.0, True),
    ("Halab", 37.16, 36.20, 2, 3200, 0.0, True),
    ("Homs", 36.72, 34.73, 2, 2200, 0.0, True),
    ("Hamah", 36.76, 35.13, 2, 1800, 0.0, True),
    ("Al Ladhiqiyah", 35.79, 35.52, 2, 1800, 0.0, True),
    ("Tartus", 35.89, 34.89, 1, 1300, 0.0, True),
    ("Tadmur", 38.27, 34.56, 1, 1000, 0.0, True),
    ("Dayr As Zawr", 40.14, 35.34, 2, 1600, 0.0, True),
    ("Ar Raqqah", 39.01, 35.95, 1, 1400, 0.0, True),
    ("Al Hasakah", 40.76, 36.50, 1, 1200, 0.0, False),
    ("Al Qamishli", 41.22, 37.05, 1, 1200, 0.0, False),
    ("Idlib", 36.63, 35.93, 1, 1100, 0.0, False),
    ("Dar'a", 36.10, 32.62, 1, 1000, 0.0, False),
    ("As Suwayda", 36.57, 32.71, 1, 900, 0.0, False),
    ("Al Bukamal", 40.93, 34.45, 0, 800, 0.0, False),
    ("Al Mayadin", 40.46, 35.02, 0, 800, 0.0, False),
    ("Al Qunaytirah", 35.82, 33.13, 0, 600, 0.7, False),
    # Lebanon
    ("Beirut", 35.50, 33.89, 2, 2600, 0.0, True),
    ("Tripoli", 35.84, 34.44, 2, 1600, 0.0, False),
    ("Sidon", 35.38, 33.56, 1, 1100, 0.0, False),
    ("Tyre", 35.21, 33.27, 1, 900, 0.0, False),
    ("Baalbek", 36.21, 34.01, 1, 900, 0.0, False),
    ("Zahle", 35.90, 33.85, 1, 900, 0.0, False),
    # Jordan
    ("Amman", 35.93, 31.95, 2, 3000, 0.0, True),
    ("Az Zarqa", 36.09, 32.07, 2, 1700, 0.0, False),
    ("Irbid", 35.85, 32.55, 2, 1500, 0.0, False),
    ("Al Mafraq", 36.21, 32.34, 1, 900, 0.0, False),
    ("Ma'an", 35.73, 30.20, 1, 1000, 0.0, True),
    ("Al Aqabah", 35.01, 29.53, 1, 1000, 0.0, False),
    # Iraq
    ("Baghdad", 44.37, 33.31, 2, 3500, 0.0, True),
    ("Mosul", 43.13, 36.34, 2, 2600, 0.0, True),
    ("Kirkuk", 44.39, 35.47, 2, 2000, 0.0, True),
    ("Arbil", 44.01, 36.19, 2, 1800, 0.0, True),
    ("Tikrit", 43.68, 34.61, 1, 1000, 0.0, False),
    ("Samarra", 43.87, 34.20, 1, 1100, 0.0, False),
    ("Ar Ramadi", 43.30, 33.42, 1, 1300, 0.0, True),
    ("Al Fallujah", 43.78, 33.35, 1, 1200, 0.0, False),
    ("Habbaniyah", 43.57, 33.37, 0, 700, 0.0, True),
    ("Hit", 42.84, 33.64, 0, 800, 0.0, False),
    ("Haditha", 42.37, 34.13, 0, 800, 0.0, False),
    ("Al Qaim", 41.09, 34.38, 0, 800, 0.0, False),
    ("Ar Rutbah", 40.28, 33.04, 0, 800, 0.0, False),
    ("Baiji", 43.50, 34.93, 1, 900, 0.0, False),
    ("Dahuk", 42.99, 36.87, 1, 1000, 0.0, False),
    ("Baqubah", 44.66, 33.75, 1, 1100, 0.0, False),
    ("Karbala", 44.02, 32.62, 2, 1600, 0.0, False),
    ("Al Hillah", 44.44, 32.48, 2, 1600, 0.0, False),
    ("An Najaf", 44.33, 32.00, 2, 1600, 0.0, False),
    # Israel and the territories
    ("Jerusalem", 35.21, 31.78, 2, 2200, 0.0, True),
    ("Tel Aviv", 34.78, 32.08, 2, 2800, 0.0, True),
    ("Haifa", 35.00, 32.80, 2, 1900, 0.0, True),
    ("Beersheba", 34.79, 31.25, 1, 1300, 0.0, False),
    ("Lod", 34.89, 31.95, 1, 900, 0.0, True),
    ("Eilat", 34.95, 29.56, 1, 900, 0.0, False),
    ("Ashdod", 34.66, 31.80, 1, 1100, 0.0, False),
    ("Nazareth", 35.30, 32.70, 1, 900, 0.0, False),
    ("Tiberias", 35.52, 32.79, 0, 700, 0.0, False),
    ("Gaza", 34.46, 31.50, 2, 1500, 0.0, False),
    ("Nablus", 35.26, 32.22, 1, 1000, 0.0, False),
    # Cyprus
    ("Nicosia", 33.37, 35.17, 2, 1700, 0.0, False),
    ("Limassol", 33.04, 34.69, 1, 1300, 0.0, False),
    ("Larnaca", 33.62, 34.92, 1, 1000, 0.0, False),
    # Turkey
    ("Malatya", 38.31, 38.35, 2, 1600, 0.0, True),
    ("Adana", 35.33, 37.00, 2, 2500, 0.0, True),
    ("Gaziantep", 37.38, 37.07, 2, 2000, 0.0, False),
    ("Diyarbakir", 40.23, 37.91, 2, 1800, 0.0, False),
    ("Sanliurfa", 38.79, 37.16, 2, 1500, 0.0, False),
    ("Mersin", 34.63, 36.81, 2, 1700, 0.0, False),
    ("Iskenderun", 36.17, 36.58, 1, 1300, 0.0, False),
    ("Antakya", 36.17, 36.20, 1, 1200, 0.0, False),
    ("Elazig", 39.22, 38.67, 1, 1300, 0.0, False),
    ("Van", 43.38, 38.50, 1, 1100, 0.0, False),
    # Egypt and Saudi Arabia
    ("Suez", 32.53, 29.98, 2, 1500, 0.0, False),
    ("El Arish", 33.80, 31.12, 1, 1000, 0.0, False),
    ("Tabuk", 36.57, 28.38, 1, 1600, 0.0, True),
    ("Haql", 34.96, 29.29, 0, 600, 0.0, False),
    ("Al Qurayyat", 37.34, 31.33, 0, 800, 0.0, False),
    ("Turayf", 38.66, 31.67, 0, 700, 0.0, False),
    ("Arar", 41.04, 30.98, 1, 1000, 0.0, False),
    ("Sakakah", 40.21, 29.97, 1, 1000, 0.0, False),
]

# Airbases: (name, lon, lat, side: True friendly, False enemy, None neutral). The runway runs
# north-south.
AIRBASES = [
    # Syria: Aleppo's Nayrab, Shayrat for Homs, Palmyra, Deir ez-Zor, Mezzeh for Damascus, the
    # airfield at Jableh for Latakia
    ("Halab", 37.22, 36.18, False),
    ("Homs", 36.91, 34.49, False),
    ("Tadmur", 38.32, 34.56, False),
    ("Dayr As Zawr", 40.19, 35.29, False),
    ("Damascus", 36.22, 33.48, False),
    ("Al Ladhiqiyah", 35.95, 35.40, False),
    # Iraq: Mosul, Kirkuk, Al Taqaddum by Habbaniyah, Al Rasheed in Baghdad, H3 in the west
    ("Mosul", 43.15, 36.31, False),
    ("Kirkuk", 44.35, 35.47, False),
    ("Habbaniyah", 43.60, 33.34, False),
    ("Baghdad", 44.49, 33.28, False),
    ("H3", 39.75, 32.93, False),
    # Jordan: Marka for Amman, Ma'an
    ("Amman", 35.99, 31.97, False),
    ("Ma'an", 35.78, 30.17, False),
    # Israel, Cyprus and Turkey
    ("Ramat David", 35.18, 32.67, True),
    ("Tel Nof", 34.82, 31.84, True),
    ("Lod", 34.89, 32.01, True),
    ("Hatzerim", 34.66, 31.23, True),
    ("Akrotiri", 32.99, 34.59, True),
    ("Malatya", 38.09, 38.44, True),
    # Saudi Arabia: King Faisal air base
    ("Tabuk", 36.62, 28.37, None),
]

# The friendly carrier group in the Levantine basin, between Cyprus and the coast of Israel.
CARRIER = ("USS Eisenhower", 34.30, 33.35)

# Theater layouts within a 2 km cell, from the kinds the library has (no houses, tanks or
# mobile SAMs here): headquarters with their stores, ammunition and supply depots, oil
# installations, Scud brigades with their launchers.
TEMPLATES = {
    "hq": [(0, 0, "headquarters"), (-380, 0, "depot"), (380, -250, "depot"), (-450, 500, "tank_farm")],
    "depot": [(-300, 0, "depot"), (0, 0, "depot"), (300, 0, "depot"), (-150, -350, "depot"), (150, -350, "depot"),
              (0, 450, "tank_farm")],
    "tankfarm": [(-150, -150, "tank_farm"), (150, -150, "tank_farm"), (-150, 150, "tank_farm"), (150, 150, "tank_farm"),
                 (500, 0, "tank_farm"), (-550, -400, "depot")],
    "scud": [(0, 0, "ss_missile"), (300, 250, "ss_missile"), (-300, 250, "ss_missile"), (300, -250, "ss_missile"),
             (-300, -250, "ss_missile"), (0, 600, "depot"), (-450, 600, "depot")],
}

# Enemy target sites: (template, lon, lat, name).
SITES = [
    # Headquarters: Syrian GHQ at Qatana and the forces in Lebanon at Anjar, the corps at Homs and
    # Aleppo; the Iraqi ministry in Baghdad, the corps at Kirkuk and Mosul; Jordan's at Zarqa
    ("hq", 36.08, 33.43, "Damascus"), ("hq", 35.93, 33.73, "Zahle"), ("hq", 36.80, 34.70, "Homs"),
    ("hq", 37.30, 36.15, "Halab"), ("hq", 44.33, 33.36, "Baghdad"), ("hq", 44.25, 35.50, "Kirkuk"),
    ("hq", 43.08, 36.40, "Mosul"), ("hq", 36.05, 32.02, "Az Zarqa"),
    # Depots: Al Kiswah, Masyaf, Al Safir; Taji, Al Muthanna, Salman Pak, the Fallujah chemical
    # plants, Al Atheer, Al Qaim's phosphate (uranium) plant
    ("depot", 36.23, 33.36, "Damascus"), ("depot", 36.35, 35.06, "Hamah"), ("depot", 37.42, 36.05, "Halab"),
    ("depot", 44.25, 33.53, "Baghdad"), ("depot", 43.85, 33.80, "Samarra"), ("depot", 44.53, 33.11, "Baghdad"),
    ("depot", 43.65, 33.40, "Al Fallujah"), ("depot", 44.10, 32.80, "Al Hillah"), ("depot", 41.15, 34.32, "Al Qaim"),
    # Oil: the refineries at Homs, Baniyas, Baiji, Daura (Baghdad) and Zarqa, the Kirkuk field,
    # the Qayyarah and Ain Zalah fields, the Rumeilan field in north-east Syria, the K2 and T4
    # pumping stations
    ("tankfarm", 36.66, 34.76, "Homs"), ("tankfarm", 35.96, 35.22, "Tartus"), ("tankfarm", 43.45, 34.92, "Baiji"),
    ("tankfarm", 44.40, 33.26, "Baghdad"), ("tankfarm", 36.14, 32.10, "Az Zarqa"), ("tankfarm", 44.33, 35.53, "Kirkuk"),
    ("tankfarm", 43.25, 35.83, "Mosul"), ("tankfarm", 42.60, 36.73, "Dahuk"), ("tankfarm", 42.05, 36.85, "Al Qamishli"),
    ("tankfarm", 43.55, 35.00, "Baiji"), ("tankfarm", 37.63, 34.52, "Tadmur"),
    # Scud brigades: Al Qutayfah north of Damascus, Hama; the western Scud box at H2 and
    # Rutbah, Al Qaim, Taji
    ("scud", 36.60, 33.74, "Damascus"), ("scud", 36.85, 35.25, "Hamah"), ("scud", 40.30, 33.15, "Ar Rutbah"),
    ("scud", 40.95, 34.20, "Al Qaim"), ("scud", 44.15, 33.60, "Baghdad"),
    # Bridges over the Euphrates, the Tigris and the Orontes
    ("bridge", 39.01, 35.94, "Ar Raqqah"), ("bridge", 40.15, 35.33, "Dayr As Zawr"), ("bridge", 40.92, 34.45, "Al Bukamal"),
    ("bridge", 42.36, 34.16, "Haditha"), ("bridge", 42.83, 33.64, "Hit"), ("bridge", 43.30, 33.43, "Ar Ramadi"),
    ("bridge", 43.78, 33.36, "Al Fallujah"), ("bridge", 44.23, 32.78, "Karbala"), ("bridge", 44.43, 32.48, "Al Hillah"),
    ("bridge", 43.14, 36.34, "Mosul"), ("bridge", 43.69, 34.60, "Tikrit"), ("bridge", 43.88, 34.20, "Samarra"),
    ("bridge", 44.37, 33.32, "Baghdad"), ("bridge", 44.45, 33.28, "Baghdad"), ("bridge", 36.75, 35.13, "Hamah"),
    ("bridge", 36.32, 35.81, "Idlib"),
]

# Single objects: (kind, lon, lat, name). Iraq's nuclear sites at Tuwaitha (Osirak) and Al
# Jazirah by Mosul; the training camps of the PFLP-GC and others at Naameh south of Beirut,
# Janta in the Bekaa and Ain al-Saheb outside Damascus.
OBJECTS = [
    ("nuclear_plant", 44.53, 33.21, "Baghdad"),
    ("nuclear_plant", 43.04, 36.26, "Mosul"),
    ("terrorist_camp", 35.48, 33.73, "Naameh"),
    ("terrorist_camp", 36.10, 33.97, "Janta"),
    ("terrorist_camp", 36.14, 33.63, "Ain al-Saheb"),
]

# Cities of the enemy side with a SAM site at their edge.
CITY_SAMS = ["Beirut", "Hamah", "Tartus", "Ar Raqqah", "Al Hasakah", "Arbil", "Tikrit", "Ar Ramadi", "Az Zarqa"]

# Patrol boats: the theater has none.
BOATS = []

# Ships at sea: (kind, lon, lat, radar). The carrier's destroyers, merchant ships on the
# lanes to Haifa, Ashdod, Beirut, Tartus, Latakia, Iskenderun, Mersin and Limassol, and in the
# Gulfs of Aqaba (Iraq's supply port) and Suez.
SHIPS = [
    ("destroyer", 34.37, 33.40, None), ("destroyer", 34.22, 33.38, None), ("destroyer", 34.30, 33.27, None),
    ("destroyer", 34.31, 33.45, None), ("destroyer", 34.40, 33.30, None), ("destroyer", 34.20, 33.30, None),
    ("destroyer", 32.70, 34.35, None), ("destroyer", 34.60, 32.70, None),
    ("ship", 34.70, 32.95, None), ("ship", 34.40, 32.40, None), ("ship", 34.45, 31.85, None), ("ship", 35.20, 33.95, None),
    ("ship", 35.60, 34.60, None), ("ship", 35.55, 35.20, None), ("ship", 35.55, 35.55, None), ("ship", 35.90, 36.55, None),
    ("ship", 34.65, 36.55, None), ("ship", 33.20, 34.50, None), ("ship", 33.90, 34.30, None), ("ship", 33.50, 35.80, None),
    ("ship", 34.72, 28.70, None), ("ship", 34.88, 29.30, None), ("ship", 32.65, 29.55, None), ("ship", 33.20, 28.60, None),
]

# Mountain ranges: (lon, lat, length theater km, width theater km, height m, bearing of the
# long axis in degrees from north).
MOUNTAINS = [
    # Mount Lebanon, the Anti-Lebanon and Mount Hermon, the Jabal Ansariyah, the Amanos
    (35.75, 33.75, 26, 9, 2200, 30), (35.98, 34.20, 26, 9, 3000, 30), (35.95, 33.40, 22, 9, 2800, 35),
    (36.35, 33.85, 28, 9, 2400, 35), (36.30, 36.60, 28, 9, 2200, 25),
    # The Taurus along the north-west, the Anti-Taurus, the south-eastern Taurus and Hakkari
    (32.60, 36.80, 34, 14, 2500, 70), (33.90, 37.10, 34, 14, 3000, 65), (35.10, 37.60, 34, 14, 3700, 55),
    (36.70, 38.10, 34, 14, 3000, 45), (39.80, 38.35, 34, 12, 2500, 80), (41.50, 38.20, 34, 14, 2800, 90),
    (43.50, 37.55, 34, 14, 3800, 100),
    # The Kurdish ranges along the Iranian border, Jabal Sinjar
    (44.60, 36.65, 30, 12, 3000, 135), (45.05, 35.95, 26, 10, 2600, 140), 
    # Troodos, the Palmyrides, Jabal al-Arab, the Edom heights, the Sinai and Hejaz mountains
    (32.85, 34.95, 26, 12, 1950, 95), 
    (35.60, 30.60, 32, 10, 1700, 15), (33.95, 28.60, 30, 14, 2600, 135), (35.35, 28.60, 32, 12, 2500, 150),
]

# Forest regions: (lon, lat, radius theater km, density 0..1): the pine forests of the coastal
# mountains, the Amanos, Troodos, Carmel and the Taurus, the oak of the Kurdish hills.
FORESTS = [
    (36.10, 35.45, 12, 0.35), (36.30, 36.60, 12, 0.4), (35.85, 34.00, 8, 0.25), (32.85, 34.95, 12, 0.5),
    (35.05, 32.72, 4, 0.3), (33.90, 37.00, 20, 0.35), (32.60, 36.75, 18, 0.35), (36.80, 37.80, 12, 0.3),
    (43.70, 37.10, 14, 0.25),
]

SPARSE_FOREST = False
GREEN = False


# ---------------------------------------------------------------------------------------------
# Models of this theater's library the layouts use, and its airfield's aprons.

KINDS = {"houses": 40, "houses2": 40, "buildings": 40, "land": 0, "carrier": 73, "carrier_object": 87}
APRONS = [(-500, -78, -680, -320), (78, 500, -180, 180)]
LAND_COLOUR = 54

# The empty open land: a hamlet here and there.
TEMPLATES["hamlet"] = [(0, 0, "houses"), (170, -120, "houses")]
TEMPLATES["lone_house"] = [(0, 0, "houses")]
FILLER = [{}, {}, {}, {(1, 2): "hamlet"}, {(2, 1): "lone_house"}]
