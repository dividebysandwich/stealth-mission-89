"""Geography of the Desert Storm theater (January 1991): Iraq, occupied Kuwait, the north-east
of Saudi Arabia, the northern Persian Gulf with Bahrain and Qatar, and the edge of Iran.

The theater squeezes 38.8-53.5 E, 24.1-36.8 N (about 1410 by 1410 km) into the 512 km of a
theater. Coordinates here are (lon, lat) degrees. The projection is fitted to the original
theater's named places, which were laid out schematically (Iraq pulled east and squashed north
to south, the Saudi bases in a row along the bottom), so the fit is loose; Tabuk lies 80 km west
of the window and is replaced by Al Jouf.

Sides: Iraq and occupied Kuwait are the enemy; Saudi Arabia, the Gulf states, Syria, Jordan and
Turkey the coalition (friendly) side. Iran is neutral: its ground is not the enemy's and its
towns are neutral ground. The front is the Saudi border with Kuwait and Iraq, hottest along
Kuwait and the Wadi al-Batin, quieter in the western desert.
"""

PROJECTION = dict(lon0=38.8, lat0=24.1, lat_ref=30.5, squeeze=2.75)

LIBRARY = "desert_storm"
LAND_RUBBLE = 78

DATE = (1, 17, 1991)
GRID = ("KU", 0, 0)
SQUADRONS = 6

ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("f1", "aa6", "patrol"),
    ("mig21", "aa2", "escort"),
    ("mig21", "aa6t", "patrol"),
    ("mig29", "aa10t", "base"),
    ("mig17", "aa2", "base"),
    ("mig23", "aa2", "escort"),
    ("f15", "none", "friendly"),
    ("a10", "none", "friendly"),
    ("f15", "none", "friendly"),
    ("a10", "none", "friendly"),
]

# Radar types (units.ron ids): the fixed SAM sites (SA-2, SA-5), Iraq's mobile SAMs (SA-8,
# SA-13) and the missile boats (the original theater had no boat units).
SAM_RADARS = ["sa2", "sa5"]
MOBILE_SAM_RADARS = ["sa8", "sa13"]
BOAT_RADARS = ["san5"]
SLAG_HEAP_TOWNS = set()

# Models this theater builds besides the water and mountains (see models.py).
EXTRA_MODELS = []

# A desert: no forests, no stray woods, no green farmland.
SPARSE_FOREST = False
GREEN = False

# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea. The only sea is the Persian Gulf.

LANDS = [
    # The mainland: round the window, then the Iranian shore of the Gulf from the Strait of
    # Hormuz west to the Shatt al-Arab, and the Arabian shore from Fao past Kuwait, Saudi
    # Arabia, Qatar and the Emirates back to Musandam.
    [
        (30.0, 15.0), (30.0, 45.0), (60.0, 45.0), (60.0, 27.0),
        # Iran
        (57.00, 27.15), (56.30, 27.15), (56.00, 27.00), (55.50, 26.70), (54.90, 26.55), (54.50, 26.60),
        (54.00, 26.70), (53.50, 26.85), (53.00, 27.10), (52.60, 27.45), (52.30, 27.65), (51.93, 27.84),
        (51.55, 27.92), (51.30, 28.15), (51.15, 28.45), (51.05, 28.75), (50.88, 28.90), (50.82, 28.98),
        (50.85, 29.10), (50.75, 29.20), (50.62, 29.48), (50.52, 29.58), (50.30, 29.85), (50.17, 30.05),
        (50.00, 30.10), (49.70, 30.12), (49.45, 30.05),
        # Khor Musa up to Bandar Mahshahr, then the Abadan shore to the Shatt al-Arab
        (49.25, 30.25), (49.18, 30.45), (49.10, 30.20), (49.02, 29.98), (48.85, 29.95), (48.62, 29.92),
        # Iraq: the Fao peninsula along the Khor Abdullah to Umm Qasr
        (48.52, 29.91), (48.40, 29.95), (48.30, 29.98), (48.10, 30.02), (47.98, 30.06), (47.96, 30.03),
        # Kuwait: the Subiya shore behind Bubiyan, round Kuwait Bay, down the coast
        (47.93, 30.00), (47.88, 29.90), (47.84, 29.78), (47.85, 29.65), (47.95, 29.56), (48.00, 29.55),
        (47.85, 29.50), (47.75, 29.45), (47.68, 29.40), (47.62, 29.36), (47.70, 29.33), (47.85, 29.35),
        (47.93, 29.37), (47.98, 29.39), (48.01, 29.40), (48.10, 29.36), (48.12, 29.25), (48.14, 29.15),
        (48.16, 29.06), (48.18, 28.98), (48.25, 28.85), (48.33, 28.72), (48.40, 28.62), (48.45, 28.52),
        # Saudi Arabia: Khafji, Ras al-Mishab, Jubail, the Ras Tanura spit, Dammam and Khobar
        (48.52, 28.40), (48.60, 28.22), (48.62, 28.10), (48.75, 27.95), (48.88, 27.85), (48.95, 27.72),
        (49.10, 27.60), (49.25, 27.50), (49.45, 27.32), (49.58, 27.15), (49.67, 27.02), (49.85, 26.88),
        (50.00, 26.78), (50.10, 26.72), (50.16, 26.70), (50.10, 26.63), (50.02, 26.58), (50.00, 26.52),
        (50.10, 26.45), (50.21, 26.30), (50.22, 26.15), (50.15, 26.05), (50.10, 25.92), (50.18, 25.78),
        (50.22, 25.62), (50.35, 25.40), (50.50, 25.15), (50.65, 24.90), (50.80, 24.72),
        # Qatar: up the west coast past Dukhan to Ras Rakan, down past Ras Laffan and Doha
        (50.88, 24.78), (50.82, 25.10), (50.77, 25.45), (50.85, 25.72), (51.03, 25.98), (51.22, 26.15),
        (51.35, 26.05), (51.55, 25.92), (51.50, 25.68), (51.53, 25.45), (51.55, 25.28), (51.62, 25.15),
        (51.58, 24.95), (51.45, 24.70), (51.35, 24.62),
        # The Emirates' coast to Musandam
        (51.50, 24.35), (51.60, 24.25), (51.75, 24.10), (52.20, 24.00), (52.60, 24.15), (52.75, 24.15),
        (53.20, 24.10), (53.80, 24.15), (54.40, 24.45), (55.00, 24.95), (55.40, 25.30), (55.95, 25.75),
        (56.10, 26.05), (56.35, 26.40), (60.0, 23.0), (60.0, 15.0),
    ],
    # Bubiyan, Failaka, Bahrain, Kharg.
    [(47.95, 29.98), (48.08, 29.97), (48.20, 29.90), (48.35, 29.78), (48.30, 29.65), (48.15, 29.58),
     (48.02, 29.60), (47.93, 29.70), (47.92, 29.85)],
    [(48.27, 29.45), (48.37, 29.46), (48.42, 29.42), (48.35, 29.40), (48.27, 29.42)],
    [(50.45, 26.24), (50.62, 26.25), (50.65, 26.10), (50.62, 25.95), (50.55, 25.80), (50.48, 25.95),
     (50.45, 26.10)],
    [(50.29, 29.29), (50.34, 29.29), (50.35, 29.22), (50.31, 29.20), (50.28, 29.24)],
]

# Lakes, reservoirs and the marshes of the south (still wet in 1991).
LAKES = [
    # Lake Tharthar, Lake Habbaniyah, Lake Razzaza (Milh), the Hadithah reservoir
    [(43.30, 34.15), (43.45, 34.00), (43.55, 33.80), (43.50, 33.62), (43.30, 33.60), (43.12, 33.72),
     (43.10, 33.90), (43.20, 34.05)],
    [(43.35, 33.31), (43.50, 33.32), (43.62, 33.27), (43.60, 33.17), (43.45, 33.13), (43.33, 33.20)],
    [(43.50, 32.88), (43.72, 32.88), (43.90, 32.72), (43.85, 32.55), (43.70, 32.45), (43.52, 32.52),
     (43.46, 32.70)],
    [(42.34, 34.20), (42.20, 34.36), (42.05, 34.49), (41.92, 34.53), (41.96, 34.43), (42.14, 34.29),
     (42.29, 34.18)],
    # Dukan and Darbandikhan reservoirs in Kurdistan
    [(44.90, 36.05), (45.00, 36.10), (45.05, 36.02), (44.98, 35.95), (44.92, 35.97)],
    [(45.68, 35.12), (45.75, 35.15), (45.78, 35.08), (45.70, 35.05)],
    # Hammar marsh, the central (Qurna) marshes, the Hawizeh marsh on the Iranian border
    [(46.62, 30.92), (46.80, 30.95), (47.10, 30.97), (47.35, 30.90), (47.45, 30.72), (47.20, 30.65),
     (46.85, 30.70), (46.65, 30.78)],
    [(46.75, 31.10), (47.00, 31.45), (47.20, 31.40), (47.30, 31.12), (47.05, 31.04)],
    [(47.45, 31.25), (47.60, 31.70), (47.75, 31.85), (47.85, 31.70), (47.75, 31.35), (47.60, 31.10),
     (47.48, 31.10)],
]

# Rivers: polylines of (lon, lat, width in theater km).
RIVERS = [
    # Tigris from the Turkish border past Mosul, Baiji, Tikrit, Samarra, Baghdad, Kut and Amarah
    # to Qurna
    [(42.36, 37.10, 0.08), (42.70, 36.88, 0.08), (42.95, 36.62, 0.09), (43.13, 36.35, 0.1), (43.18, 36.20, 0.1),
     (43.27, 35.97, 0.1), (43.25, 35.80, 0.1), (43.25, 35.50, 0.1), (43.38, 35.18, 0.1), (43.49, 34.95, 0.1),
     (43.70, 34.60, 0.1), (43.86, 34.19, 0.1), (44.10, 33.98, 0.11), (44.25, 33.70, 0.11), (44.36, 33.40, 0.12),
     (44.40, 33.30, 0.12), (44.48, 33.22, 0.12), (44.58, 33.08, 0.12), (44.75, 32.95, 0.12), (45.05, 32.90, 0.12),
     (45.50, 32.60, 0.12), (45.82, 32.50, 0.12), (46.30, 32.42, 0.12), (46.68, 32.45, 0.12), (46.88, 32.10, 0.12),
     (47.14, 31.85, 0.12), (47.27, 31.52, 0.12), (47.35, 31.25, 0.12), (47.43, 31.01, 0.14)],
    # Shatt al-Arab from Qurna past Basra, Khorramshahr, Abadan and Fao to the Gulf
    [(47.43, 31.01, 0.15), (47.60, 30.80, 0.18), (47.78, 30.60, 0.2), (47.85, 30.52, 0.22), (47.95, 30.48, 0.22),
     (48.10, 30.45, 0.25), (48.20, 30.40, 0.25), (48.28, 30.33, 0.25), (48.40, 30.10, 0.3), (48.50, 29.98, 0.4),
     (48.57, 29.91, 0.6)],
    # Euphrates from Syria to the Hadithah reservoir, and on past Hit, Ramadi, Fallujah, Kufa,
    # Samawah and Nasiriyah into the Hammar marsh; out of the marsh to the Shatt at Qurna
    [(40.90, 34.45, 0.08), (41.10, 34.38, 0.08), (41.45, 34.40, 0.08), (41.75, 34.45, 0.08), (41.95, 34.50, 0.08)],
    [(42.33, 34.18, 0.08), (42.37, 34.10, 0.09), (42.55, 33.90, 0.09), (42.83, 33.64, 0.1), (43.10, 33.48, 0.1),
     (43.30, 33.44, 0.1), (43.55, 33.41, 0.1), (43.78, 33.37, 0.1), (43.95, 33.18, 0.1), (44.10, 33.00, 0.1),
     (44.29, 32.79, 0.1), (44.22, 32.54, 0.09), (44.37, 32.23, 0.09), (44.40, 32.03, 0.09), (44.55, 31.85, 0.09),
     (44.70, 31.55, 0.09), (45.00, 31.40, 0.09), (45.29, 31.32, 0.1), (45.60, 31.20, 0.1), (46.00, 31.08, 0.1),
     (46.26, 31.05, 0.1), (46.45, 30.96, 0.1), (46.65, 30.88, 0.1)],
    [(47.40, 30.85, 0.1), (47.43, 31.01, 0.12)],
    # Shatt al-Hillah through Hillah and Diwaniyah to the Euphrates below Samawah
    [(44.29, 32.79, 0.05), (44.43, 32.48, 0.05), (44.70, 32.20, 0.05), (44.93, 31.99, 0.05), (45.15, 31.60, 0.05),
     (45.35, 31.30, 0.05)],
    # Diyala from Darbandikhan to the Tigris below Baghdad
    [(45.75, 35.10, 0.05), (45.45, 34.65, 0.05), (45.15, 34.25, 0.05), (44.90, 34.05, 0.06), (44.65, 33.75, 0.06),
     (44.55, 33.45, 0.06), (44.50, 33.24, 0.06)],
    # Great Zab and Little Zab
    [(44.30, 36.75, 0.06), (43.95, 36.50, 0.06), (43.65, 36.20, 0.06), (43.40, 36.05, 0.06), (43.27, 35.98, 0.06)],
    [(45.00, 35.95, 0.05), (44.60, 35.85, 0.05), (44.13, 35.76, 0.05), (43.80, 35.45, 0.05), (43.50, 35.22, 0.05),
     (43.38, 35.18, 0.05)],
    # Karun past Ahvaz to Khorramshahr, Bahmanshir past Abadan, Karkheh into the Hawizeh marsh
    [(48.85, 32.05, 0.07), (48.68, 31.32, 0.08), (48.45, 30.80, 0.08), (48.25, 30.55, 0.09), (48.18, 30.44, 0.1)],
    [(48.30, 30.35, 0.1), (48.55, 30.15, 0.12), (48.75, 30.02, 0.15), (48.90, 29.96, 0.2)],
    [(48.20, 32.40, 0.05), (48.05, 31.95, 0.05), (47.85, 31.60, 0.05), (47.78, 31.55, 0.05)],
    # Khor al-Zubair from Umm Qasr towards Zubair
    [(47.94, 30.06, 0.5), (47.90, 30.18, 0.4), (47.86, 30.30, 0.25)],
]

# ---------------------------------------------------------------------------------------------
# Sides

# The Saudi border from the Jordanian tri-point along Iraq to the Kuwaiti tri-point at the Wadi
# al-Batin, then along Kuwait to the coast north of Khafji: (lon, lat, intensity). The ground
# war has not started; the air war has, and the Kuwaiti border is where the armies face each
# other.
FRONT = [
    (39.20, 32.15, 0.2), (40.40, 31.95, 0.25), (41.40, 31.40, 0.3), (42.10, 31.10, 0.35), (42.80, 30.35, 0.4),
    (43.50, 29.88, 0.5), (44.70, 29.20, 0.6), (45.40, 29.15, 0.7), (46.00, 29.10, 0.85), (46.55, 29.10, 1.0),
    (47.10, 28.80, 1.0), (47.70, 28.53, 1.0), (48.10, 28.54, 1.0), (48.43, 28.54, 1.0),
]

# Iraq's border with Iran from the Turkish tri-point down to the Shatt al-Arab and its mouth.
_IRAN_BORDER = [
    (44.79, 37.15), (44.86, 36.70), (45.30, 36.40), (45.70, 35.90), (46.05, 35.60), (46.15, 35.25), (45.95, 35.00),
    (45.55, 34.55), (45.50, 34.25), (45.80, 33.65), (46.05, 33.10), (46.60, 32.75), (47.15, 32.40), (47.50, 32.10),
    (47.85, 31.75), (47.70, 31.40), (47.68, 31.00), (48.03, 30.48), (48.15, 30.42), (48.28, 30.33), (48.40, 30.10),
    (48.50, 29.98), (48.57, 29.91),
]
# Iraqi and Kuwaiti waters, from the Shatt's mouth round the offshore terminals to the coast at
# the Saudi border.
_ENEMY_WATERS = [(48.90, 29.75), (49.00, 29.30), (48.90, 28.55)]
# Iraq's borders with Jordan, Syria and Turkey, from the Jordanian tri-point round to the
# Iranian one.
_WEST_NORTH_BORDER = [
    (38.79, 33.37), (41.00, 34.42), (41.25, 35.60), (41.30, 36.35), (42.00, 36.75), (42.36, 37.10), (43.00, 37.30),
    (43.80, 37.25), (44.30, 37.05), (44.79, 37.15),
]

# The friendly (and neutral) side: everything but Iraq and Kuwait. One outline: the world's box
# joined by a slit at the Turkish-Iranian tri-point to Iraq and Kuwait, traced the other way.
BLUE = (
    [(44.795, 45.0), (30.0, 45.0), (30.0, 15.0), (60.0, 15.0), (60.0, 45.0), (44.805, 45.0)]
    + _IRAN_BORDER + _ENEMY_WATERS
    + [(lon, lat) for lon, lat, _ in reversed(FRONT)]
    + _WEST_NORTH_BORDER
)

# Neutral places: Iran's towns.
NEUTRAL = {"Abadan", "Khorramshahr", "Ahvaz", "Bushehr", "Dezful", "Kermanshah", "Ilam", "Hamadan", "Khorramabad",
           "Shiraz", "Behbahan", "Vanak"}

# ---------------------------------------------------------------------------------------------
# Places

# Towns: (name, lon, lat, kind 0 village / 1 town / 2 city, radius in theater metres, ruined
# share, in the world name table).
TOWNS = [
    # Iraq: Baghdad and the north
    ("Baghdad", 44.37, 33.31, 2, 3500, 0.0, True),
    ("Mosul", 43.13, 36.34, 2, 2800, 0.0, True),
    ("Kirkuk", 44.39, 35.47, 2, 2200, 0.0, True),
    ("Irbil", 44.01, 36.19, 2, 2000, 0.0, True),
    ("Sulaymaniyah", 45.43, 35.56, 2, 1800, 0.0, False),
    ("Tall Afar", 42.45, 36.37, 1, 1100, 0.0, False),
    ("Qayyarah", 43.21, 35.80, 0, 700, 0.0, False),
    ("Sharqat", 43.22, 35.51, 0, 700, 0.0, False),
    ("Baiji", 43.49, 34.93, 1, 1000, 0.0, True),
    ("Tikrit", 43.68, 34.60, 1, 1200, 0.0, False),
    ("Samarra", 43.88, 34.20, 1, 1300, 0.0, True),
    ("Baqubah", 44.62, 33.76, 2, 1500, 0.0, False),
    ("Khanaqin", 45.38, 34.35, 1, 1000, 0.0, False),
    ("Salman Pak", 44.58, 33.10, 1, 900, 0.0, True),
    # Iraq: the Euphrates and the western desert
    ("Qaim", 41.05, 34.37, 1, 900, 0.0, True),
    ("Hadithah", 42.38, 34.13, 1, 1100, 0.0, True),
    ("Hit", 42.85, 33.66, 1, 900, 0.0, False),
    ("Ramadi", 43.30, 33.42, 2, 1600, 0.0, False),
    ("Habbaniyah", 43.57, 33.37, 1, 900, 0.0, True),
    ("Fallujah", 43.78, 33.35, 1, 1400, 0.0, False),
    ("Rutbah", 40.29, 33.04, 1, 1000, 0.0, False),
    ("Nukhayb", 42.28, 32.04, 0, 600, 0.0, False),
    ("Rahhaliyah", 43.41, 32.75, 0, 600, 0.0, True),
    ("Karbala", 44.02, 32.61, 2, 1700, 0.0, True),
    ("Musayyib", 44.29, 32.78, 0, 700, 0.0, False),
    ("Hillah", 44.41, 32.47, 2, 1700, 0.0, False),
    ("Al Najaf", 44.33, 32.03, 2, 1800, 0.0, True),
    ("Ad Diwaniyah", 44.91, 31.97, 1, 1400, 0.0, True),
    ("As Samawah", 45.29, 31.31, 1, 1300, 0.0, True),
    # Iraq: the Tigris below Baghdad and the south
    ("Kut", 45.82, 32.52, 2, 1600, 0.0, False),
    ("Al Kumayt", 46.87, 32.04, 0, 700, 0.0, True),
    ("Amarah", 47.15, 31.84, 2, 1600, 0.0, True),
    ("Nasiriyah", 46.26, 31.04, 2, 1700, 0.0, True),
    ("Suq ash Shuyukh", 46.47, 30.89, 0, 700, 0.0, False),
    ("Qurna", 47.45, 31.00, 1, 900, 0.0, False),
    ("Jalibah", 46.50, 30.60, 0, 600, 0.0, True),
    ("Basra", 47.81, 30.51, 2, 2800, 0.0, True),
    ("Zubair", 47.70, 30.39, 1, 1200, 0.0, False),
    ("Safwan", 47.72, 30.11, 1, 900, 0.0, True),
    ("Umm Qasr", 47.93, 30.04, 1, 900, 0.0, False),
    ("Fao", 48.47, 29.98, 1, 900, 0.0, False),
    # Kuwait
    ("Kuwait City", 47.98, 29.37, 2, 3000, 0.0, True),
    ("Jahra", 47.66, 29.33, 1, 1200, 0.0, False),
    ("Abdali", 47.70, 29.98, 0, 600, 0.0, False),
    ("Ahmadi", 48.08, 29.08, 1, 1200, 0.0, False),
    ("Wafra", 47.93, 28.58, 0, 600, 0.0, False),
    # Saudi Arabia
    ("Riyadh", 46.72, 24.69, 2, 3500, 0.0, True),
    ("Shaqra", 45.25, 25.25, 1, 900, 0.0, True),
    ("Majmaah", 45.35, 25.90, 1, 900, 0.0, False),
    ("Buraydah", 43.97, 26.33, 2, 1800, 0.0, True),
    ("Unayzah", 43.99, 26.08, 1, 1200, 0.0, False),
    ("Ha'il", 41.69, 27.52, 2, 1700, 0.0, True),
    ("Jubbah", 40.93, 28.02, 0, 700, 0.0, True),
    ("Sakaka", 40.21, 29.97, 1, 1100, 0.0, False),
    ("Ar'ar", 41.04, 30.97, 1, 1300, 0.0, True),
    ("Rafha", 43.49, 29.63, 1, 1000, 0.0, False),
    ("Hafar al Batin", 45.97, 28.43, 1, 1300, 0.0, True),
    ("Qaisumah", 46.13, 28.31, 0, 600, 0.0, False),
    ("Khafji", 48.49, 28.42, 1, 1000, 0.0, True),
    ("Nuayriyah", 48.42, 27.47, 0, 600, 0.0, False),
    ("Jubail", 49.62, 27.00, 1, 1300, 0.0, False),
    ("Ras Tanura", 50.08, 26.66, 0, 700, 0.0, False),
    ("Dammam", 50.08, 26.43, 2, 2200, 0.0, False),
    ("Dhahran", 50.12, 26.29, 1, 1200, 0.0, True),
    ("Hofuf", 49.59, 25.38, 2, 1800, 0.0, True),
    # Bahrain, Qatar, Syria
    ("Manama", 50.58, 26.21, 2, 1800, 0.0, False),
    ("Doha", 51.50, 25.29, 2, 2000, 0.0, False),
    ("Deir ez-Zor", 40.14, 35.33, 2, 1500, 0.0, False),
    # Iran (neutral)
    ("Abadan", 48.29, 30.36, 2, 2000, 0.0, False),
    ("Khorramshahr", 48.17, 30.44, 1, 1200, 0.0, False),
    ("Ahvaz", 48.71, 31.32, 2, 2200, 0.0, False),
    ("Dezful", 48.40, 32.38, 1, 1300, 0.0, False),
    ("Behbahan", 50.24, 30.60, 1, 1000, 0.0, False),
    ("Bushehr", 50.86, 28.96, 1, 1300, 0.0, False),
    ("Shiraz", 52.53, 29.61, 2, 2200, 0.0, False),
    ("Ilam", 46.42, 33.64, 1, 900, 0.0, False),
    ("Kermanshah", 47.07, 34.31, 2, 2000, 0.0, False),
    ("Khorramabad", 48.36, 33.49, 1, 1200, 0.0, False),
    ("Hamadan", 48.52, 34.80, 2, 1800, 0.0, False),
    ("Vanak", 49.53, 34.68, 0, 600, 0.0, True),
]

# Airbases: (name, lon, lat, friendly). The runway runs north-south. Iraq's are the real
# airfields of each place (Hadithah: Al Asad; Habbaniyah: Al Taqaddum; Baghdad: Al Rasheed;
# Basra: Shaibah; Kuwait City: Ali Al Salem); Rahhaliyah has no real airfield and gets one in
# the desert west of the village. Tabuk is outside the window: Al Jouf stands in for it.
AIRBASES = [
    ("Hadithah", 42.44, 33.79, False),
    ("Rahhaliyah", 43.30, 32.80, False),
    ("Habbaniyah", 43.60, 33.34, False),
    ("Baghdad", 44.49, 33.28, False),
    ("Amarah", 47.07, 31.80, False),
    ("Basra", 47.66, 30.42, False),
    ("Mosul", 43.12, 36.31, False),
    ("Kirkuk", 44.35, 35.47, False),
    ("Jalibah", 46.60, 30.58, False),
    ("Kuwait City", 47.52, 29.35, False),
    ("Al Jouf", 40.10, 29.79, True),
    ("Ha'il", 41.69, 27.44, True),
    ("Buraydah", 43.77, 26.30, True),
    ("Shaqra", 45.25, 25.22, True),
    ("Riyadh", 46.73, 24.71, True),
    ("Dhahran", 50.15, 26.27, True),
    ("Hofuf", 49.49, 25.29, True),
]

# The carrier group in the central Gulf, between Ras al-Mishab and Bushehr.
CARRIER = ("USS T. Roosevelt", 49.90, 28.10)

# The theater's own target layouts within a 2 km cell: (metres east, metres north, kind).
TEMPLATES = {
    # A Republican Guard (or heavy division) position: two rows of armour dug in, its depot.
    "guards": [(-600 + 200 * k, 250, "rep_guards") for k in range(6)]
              + [(-500 + 200 * k, -150, "rep_guards") for k in range(5)] + [(300, -600, "depot")],
    # A Scud battery: the fixed launcher, two mobile launchers out on the sand, its stores.
    "scud": [(0, 0, "scud_launcher"), (-550, 400, "mobile_scud"), (600, -350, "mobile_scud"), (-400, -500, "depot")],
    # A burning oil field: wellheads spread out, a gathering tank farm.
    "oil_field": [(-650, 600, "burning_oil_rig"), (-50, 750, "burning_oil_rig"), (550, 500, "burning_oil_rig"),
                  (-700, -50, "burning_oil_rig"), (700, -100, "burning_oil_rig"), (-400, -650, "burning_oil_rig"),
                  (450, -700, "burning_oil_rig"), (0, 0, "tank_farm")],
    # The export terminal ashore: the pumping manifold among its tank farms.
    "terminal": [(0, 0, "pumping_manifold"), (-450, 350, "tank_farm"), (450, 350, "tank_farm"), (0, -450, "tank_farm")],
    # A chemical weapons plant with its bunkers; a biological weapons plant with its own.
    "chem_works": [(0, 0, "chem_wpns_plant"), (-550, -500, "depot"), (550, -500, "depot"), (0, 600, "sa_missile")],
    "bio_works": [(0, 0, "bio_wpns_plant"), (-500, 500, "depot"), (500, -450, "depot")],
}

# Enemy target sites in Iraq and Kuwait: (template, lon, lat, name).
SITES = [
    # Corps and command headquarters
    ("hq", 44.40, 33.36, "Baghdad"), ("hq", 47.75, 30.47, "Basra"), ("hq", 47.95, 29.32, "Kuwait City"),
    ("hq", 47.60, 29.28, "Jahra"), ("hq", 46.18, 30.98, "Nasiriyah"), ("hq", 44.44, 35.50, "Kirkuk"),
    ("hq", 43.22, 36.38, "Mosul"), ("hq", 47.22, 31.90, "Amarah"), ("hq", 47.62, 30.30, "Zubair"),
    ("hq", 40.35, 33.08, "Rutbah"), ("hq", 44.25, 32.00, "Al Najaf"), ("hq", 44.98, 32.03, "Ad Diwaniyah"),
    ("hq", 46.95, 32.08, "Al Kumayt"), ("hq", 43.62, 34.64, "Tikrit"), ("hq", 47.85, 28.66, "Wafra"),
    # Surface-to-surface missiles (FROGs and Ababils) behind the Kuwaiti front and at the corps
    ("ss", 47.80, 28.72, "Wafra"), ("ss", 47.98, 28.98, "Ahmadi"), ("ss", 47.45, 29.22, "Jahra"),
    ("ss", 47.62, 29.92, "Abdali"), ("ss", 47.55, 30.55, "Basra"), ("ss", 46.35, 30.92, "Nasiriyah"),
    ("ss", 44.10, 31.90, "Al Najaf"), ("ss", 45.05, 31.90, "Ad Diwaniyah"), ("ss", 40.20, 33.12, "Rutbah"),
    ("ss", 43.80, 34.50, "Tikrit"),
    # Scud batteries: the western launch boxes by H-2 and H-3 and Al Qaim, "Scud alley" south
    # of the Euphrates, Basra, the Tigris and the north
    ("scud", 40.55, 33.30, "Rutbah"), ("scud", 39.85, 32.85, "Rutbah"), ("scud", 41.25, 34.25, "Qaim"),
    ("scud", 42.30, 32.15, "Nukhayb"), ("scud", 45.40, 31.20, "As Samawah"), ("scud", 44.45, 31.80, "Al Najaf"),
    ("scud", 44.80, 31.85, "Ad Diwaniyah"), ("scud", 47.58, 30.05, "Safwan"), ("scud", 46.75, 32.12, "Al Kumayt"),
    ("scud", 44.10, 36.08, "Irbil"), ("scud", 44.22, 33.52, "Baghdad"),
    # The Republican Guard in the strategic reserve north and west of Kuwait (Tawakalna,
    # Medina, Hammurabi, Al Faw, Nebuchadnezzar, Adnan, Baghdad divisions) and the heavy
    # divisions behind the Kuwaiti front
    ("guards", 47.20, 29.92, "Abdali"), ("guards", 47.30, 30.18, "Safwan"), ("guards", 47.55, 30.08, "Safwan"),
    ("guards", 47.82, 30.22, "Umm Qasr"), ("guards", 47.65, 30.42, "Zubair"), ("guards", 47.40, 30.42, "Basra"),
    ("guards", 46.85, 30.45, "Jalibah"), ("guards", 46.95, 29.35, "Jahra"), ("guards", 47.05, 29.58, "Abdali"),
    ("guards", 46.70, 29.62, "Abdali"), ("guards", 47.70, 28.78, "Wafra"), ("guards", 46.60, 29.95, "Jalibah"),
    # Supply depots
    ("depot", 44.27, 33.55, "Baghdad"), ("depot", 44.30, 35.38, "Kirkuk"), ("depot", 47.75, 30.62, "Basra"),
    ("depot", 46.15, 31.10, "Nasiriyah"), ("depot", 45.78, 32.43, "Kut"), ("depot", 47.62, 29.42, "Jahra"),
    ("depot", 47.95, 29.12, "Ahmadi"), ("depot", 45.20, 31.38, "As Samawah"), ("depot", 43.25, 33.38, "Ramadi"),
    ("depot", 43.25, 36.28, "Mosul"), ("depot", 43.66, 34.52, "Tikrit"), ("depot", 40.40, 33.00, "Rutbah"),
    # Tank farms by the oil fields, pipelines and refineries
    ("tankfarm", 44.30, 35.53, "Kirkuk"), ("tankfarm", 43.55, 34.98, "Baiji"), ("tankfarm", 47.68, 30.34, "Zubair"),
    ("tankfarm", 44.45, 33.24, "Baghdad"), ("tankfarm", 43.28, 35.83, "Qayyarah"), ("tankfarm", 42.42, 34.24, "Hadithah"),
    ("tankfarm", 46.20, 31.08, "Nasiriyah"), ("tankfarm", 48.02, 29.10, "Ahmadi"),
    # Oil fields: Kuwait's (Burgan, Magwa, Ahmadi, Minagish, Umm Gudair, Wafra, Raudhatain,
    # Sabriyah) and Rumaila on the border
    ("oil_field", 47.95, 28.93, "Ahmadi"), ("oil_field", 48.03, 29.00, "Ahmadi"), ("oil_field", 47.88, 29.02, "Ahmadi"),
    ("oil_field", 48.00, 29.15, "Ahmadi"), ("oil_field", 47.57, 29.05, "Jahra"), ("oil_field", 47.68, 28.86, "Wafra"),
    ("oil_field", 47.98, 28.63, "Wafra"), ("oil_field", 47.70, 29.83, "Abdali"), ("oil_field", 47.80, 29.80, "Abdali"),
    ("oil_field", 47.35, 30.40, "Zubair"), ("oil_field", 47.38, 30.15, "Safwan"),
    # Export terminal at Mina al Ahmadi feeding the Sea Island
    ("terminal", 48.10, 29.11, "Sea Island"),
    # Chemical and biological weapons
    ("chem_works", 44.52, 33.05, "Salman Pak"), ("chem_works", 43.45, 34.90, "Baiji"),
    ("chem_works", 43.70, 33.29, "Fallujah"),
    ("bio_works", 43.78, 34.12, "Samarra"), ("bio_works", 41.20, 34.32, "Qaim"),
    # Bridges over the Tigris, the Euphrates and the Shatt al-Arab
    ("bridge", 44.39, 33.33, "Baghdad"), ("bridge", 43.14, 36.34, "Mosul"), ("bridge", 46.26, 31.05, "Nasiriyah"),
    ("bridge", 45.29, 31.32, "As Samawah"), ("bridge", 45.82, 32.50, "Kut"), ("bridge", 47.43, 31.02, "Qurna"),
    ("bridge", 47.85, 30.53, "Basra"), ("bridge", 47.14, 31.85, "Amarah"), ("bridge", 43.78, 33.37, "Fallujah"),
    ("bridge", 42.37, 34.11, "Hadithah"), ("bridge", 43.30, 33.44, "Ramadi"),
]

# Single objects at their real places: (kind, lon, lat, name).
OBJECTS = [
    # Nuclear: Tuwaitha by Baghdad, the Al Jazirah plant by Mosul, the Ash Sharqat enrichment site
    ("nuclear_plant", 44.53, 33.20, "Baghdad"), ("nuclear_plant", 43.06, 36.42, "Mosul"),
    ("nuclear_plant", 43.30, 35.48, "Sharqat"),
    # Saddam's palaces in Baghdad and Tikrit; the air defence operations centre and the sector
    # operations centres of the KARI system (north at Kirkuk, south at Tallil, west at H-3)
    ("palace", 44.385, 33.305, "Baghdad"), ("palace", 43.665, 34.58, "Tikrit"),
    ("air_def_command", 44.35, 33.33, "Baghdad"), ("air_def_command", 44.30, 35.42, "Kirkuk"),
    ("air_def_command", 46.09, 30.94, "Nasiriyah"), ("air_def_command", 39.74, 32.93, "Rutbah"),
    # Refineries: Daura in Baghdad, Basra (Shuaiba), Baiji, Qayyarah, Hadithah (K-3), Kuwait's
    # Mina al Ahmadi and Mina Abdullah; Saudi Arabia's Ras Tanura
    ("oil_refinery", 44.42, 33.25, "Baghdad"), ("oil_refinery", 47.73, 30.45, "Basra"),
    ("oil_refinery", 43.52, 35.00, "Baiji"), ("oil_refinery", 43.24, 35.84, "Qayyarah"),
    ("oil_refinery", 42.38, 34.27, "Hadithah"), ("oil_refinery", 48.13, 29.05, "Sea Island"),
    ("oil_refinery", 48.15, 28.97, "Ahmadi"), ("oil_refinery", 50.12, 26.67, "Ras Tanura"),
    # Pipeline pumping stations: Zubair (PS-1) and Hadithah (K-3)
    ("pumping_manifold", 47.73, 30.33, "Zubair"), ("pumping_manifold", 42.34, 34.22, "Hadithah"),
    # Mosques and shrines
    ("mosque", 44.34, 33.38, "Baghdad"), ("mosque", 44.03, 32.62, "Karbala"), ("mosque", 44.31, 32.00, "Al Najaf"),
    ("mosque", 43.89, 34.20, "Samarra"), ("mosque", 47.98, 29.37, "Kuwait City"), ("mosque", 47.80, 30.50, "Basra"),
    ("mosque", 43.12, 36.34, "Mosul"), ("mosque", 46.71, 24.63, "Riyadh"), ("mosque", 47.17, 31.83, "Amarah"),
    ("mosque", 44.37, 32.04, "Al Najaf"),
    # Offshore: Kuwait's Sea Island terminal, Iraq's Mina al Bakr and Khor al Amaya terminals,
    # the Khafji and Safaniya fields, Zuluf, Marjan, Berri, Abu Safah, Iran's Nowruz and Soroosh
    ("offshore_oil_rig", 48.30, 29.10, "Sea Island"), ("offshore_oil_rig", 48.33, 29.06, "Sea Island"),
    ("offshore_oil_rig", 48.80, 29.68, "Mina al Bakr"), ("offshore_oil_rig", 48.81, 29.78, "Khor al Amaya"),
    ("offshore_oil_rig", 48.75, 28.40, "Khafji"), ("offshore_oil_rig", 48.85, 28.15, "Safaniya"),
    ("offshore_oil_rig", 48.95, 28.05, "Safaniya"), ("offshore_oil_rig", 48.82, 28.27, "Safaniya"),
    ("offshore_oil_rig", 49.30, 28.40, "Zuluf"), ("offshore_oil_rig", 49.60, 28.35, "Marjan"),
    ("offshore_oil_rig", 49.80, 27.25, "Berri"), ("offshore_oil_rig", 50.45, 26.75, "Abu Safah"),
    ("offshore_oil_rig", 49.40, 29.50, "Nowruz"), ("offshore_oil_rig", 49.55, 29.05, "Soroosh"),
]

# Enemy towns with a SAM site at their edge.
CITY_SAMS = ["Karbala", "Nasiriyah", "Samarra", "Tikrit", "Kut", "Sulaymaniyah", "Ramadi", "Hillah"]

# The original theater had no patrol boat units; Iraq's missile boats are in SHIPS.
BOATS = []

# Ships: (kind, lon, lat, radar or None). Iraqi missile boats off Bubiyan and Kuwait; the
# carrier's destroyers; tankers and freighters in the shipping lanes down the Gulf.
SHIPS = [
    ("missile_boat", 48.45, 29.65, "san5"), ("missile_boat", 48.60, 29.40, "san5"),
    ("missile_boat", 48.50, 29.20, "san5"), ("missile_boat", 48.62, 29.82, "san5"),
    ("destroyer", 49.75, 28.20, None), ("destroyer", 50.05, 28.15, None), ("destroyer", 49.95, 27.95, None),
    ("destroyer", 49.70, 27.98, None), ("destroyer", 49.55, 28.55, None),
    ("ship", 49.40, 28.70, None), ("ship", 50.20, 27.60, None), ("ship", 50.70, 27.30, None),
    ("ship", 51.20, 26.95, None), ("ship", 51.80, 26.70, None), ("ship", 52.40, 26.40, None),
    ("ship", 53.10, 26.10, None), ("ship", 50.40, 26.95, None), ("ship", 49.85, 27.45, None),
    ("ship", 51.00, 27.70, None),
]

# Mountain ranges: (lon, lat, length theater km, width theater km, height m, bearing of the
# long axis in degrees from north).
MOUNTAINS = [
    # The Kurdish ranges north-east of Irbil, Hawraman on the Iranian border
    (44.45, 36.50, 20, 7, 1900, 125), (45.05, 36.45, 26, 12, 3000, 130), (45.75, 35.55, 24, 10, 2600, 140),
    # The Zagros, from Kermanshah to Shiraz, with its front ranges along the Iraqi border
    (46.55, 34.85, 30, 16, 2800, 135), (47.25, 34.15, 30, 16, 2800, 135), (47.95, 33.55, 30, 16, 3500, 135),
    (48.70, 32.95, 30, 16, 3500, 135), (49.45, 32.35, 30, 16, 3800, 135), (50.25, 31.65, 30, 16, 3800, 135),
    (51.00, 30.95, 30, 16, 3600, 135), (51.75, 30.20, 30, 16, 3000, 135), (52.45, 29.50, 30, 16, 2800, 135),
    (53.10, 28.80, 28, 14, 2500, 130),
    (46.90, 33.30, 24, 8, 2000, 135), (47.75, 32.70, 22, 8, 1800, 135), (49.10, 31.70, 24, 8, 1500, 135),
    # Jebel Sinjar and the Hamrin hills in Iraq
    (41.85, 36.35, 22, 6, 1460, 90), 
    # Jabal Aja and Jabal Salma by Ha'il, the Tuwaiq escarpment west of Riyadh
    (41.45, 27.60, 24, 7, 1300, 40), 
    
]

# A desert: no forest regions.
FORESTS = []


# ---------------------------------------------------------------------------------------------
# Models of this theater's library the layouts use, and its airfield's aprons.

KINDS = {"houses": 40, "houses2": 40, "buildings": 40, "patrol_boat": 92, "land": 0, "carrier": 73,
         "carrier_object": 87}
APRONS = [(-500, -78, -680, -320), (78, 500, -164, 164)]
LAND_COLOUR = 54

# The empty open land: a hamlet here and there.
TEMPLATES["hamlet"] = [(0, 0, "houses"), (170, -120, "houses")]
TEMPLATES["lone_house"] = [(0, 0, "houses")]
FILLER = [{}, {}, {}, {(1, 2): "hamlet"}, {(2, 1): "lone_house"}]
