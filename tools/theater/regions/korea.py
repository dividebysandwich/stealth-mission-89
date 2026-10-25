"""Geography of the Korea theater (1996): North Korea, the northern edge of South Korea, the
Chinese borderlands of Liaoning and Jilin along the Yalu and the Tumen, and Russia's Primorye
round Vladivostok and Nakhodka.

The theater squeezes 124.3-133.1 E, 37.5-44.2 N (about 740 by 740 km) into the 512 km of a
theater (squeeze 1.45). Coordinates here are (lon, lat) degrees.

Sides: South Korea, with its north-western islands, is the friendly side; the US carriers sail
the Sea of Japan. North Korea is the enemy, and in this scenario so are its Chinese and Soviet
neighbours (their airbases and SAM sites are hostile). The front is the DMZ along the Military
Demarcation Line from the Han estuary to the east coast below Kosong: a cold war turned tense,
with low intensity, hottest on the old invasion corridors of Kaesong-Munsan and Chorwon.
"""

PROJECTION = dict(lon0=124.3, lat0=37.5, lat_ref=40.8, squeeze=1.45)

LIBRARY = "korea"
LAND_RUBBLE = 73

DATE = (6, 25, 1996)
GRID = ("KO", 0, 0)
SQUADRONS = 7

ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("mig17", "none", "base"),
    ("mig21", "aa2", "base"),
    ("mig21", "aa2", "escort"),
    ("mig23", "aa2", "patrol"),
    ("mig23", "aa2", "base"),
    ("mig29", "aa10t", "base"),
    ("mig29", "aa10t", "escort"),
    ("su27", "aa10t", "base"),
    ("f4e_blue", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
]

# Radar types (units.ron ids): the fixed SAM sites, the mobile ones and the patrol boats.
SAM_RADARS = ["sa2", "sa5", "sa8", "sa10", "sa12"]
MOBILE_SAM_RADARS = ["sa8", "sa13"]
BOAT_RADARS = ["san4", "san5", "san6"]
SLAG_HEAP_TOWNS = set()

# Models this theater builds besides the water, forests and mountains (see models.py).
EXTRA_MODELS = []

# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea.

LANDS = [
    # The continent: the Korean peninsula's south coast (outside the theater), the east coast
    # north past the DMZ, Wonsan's Yonghung Bay, Hamhung, the Hamgyong coast to Chongjin and
    # Rajin, the Tumen's mouth, Primorye round Peter the Great Gulf (Amur Bay, Vladivostok,
    # Ussuri Bay, Nakhodka), coarsely on up the Tatar Strait, round the far north and west, down
    # past the Bohai, the Liaodong peninsula, Korea Bay at the Yalu, and the west coast of North
    # Korea south past the Taedong's mouth, Changsan Cape, Ongjin and Haeju Bay to the Han
    # estuary and Gyeonggi Bay.
    [
        # South coast and the east coast of South Korea
        (126.40, 34.40), (127.50, 34.60), (128.50, 34.85), (129.05, 35.05), (129.40, 35.45), (129.45, 35.95),
        (129.55, 36.05), (129.42, 36.40), (129.40, 36.80), (129.35, 37.10), (129.25, 37.35), (129.17, 37.45),
        (129.10, 37.55), (129.00, 37.68), (128.91, 37.78), (128.80, 37.91), (128.69, 38.04), (128.62, 38.18),
        (128.55, 38.33), (128.46, 38.48), (128.38, 38.60),
        # North Korea's east coast: Kosong, Tongchon, Wonsan with the Kalma peninsula, Yonghung Bay
        # and the Hodo peninsula
        (128.31, 38.74), (128.20, 38.85), (128.06, 38.94), (127.95, 38.99), (127.82, 39.03), (127.68, 39.06),
        (127.58, 39.10), (127.60, 39.17), (127.57, 39.24), (127.50, 39.21), (127.44, 39.18), (127.38, 39.22),
        (127.33, 39.30), (127.32, 39.40), (127.40, 39.46), (127.50, 39.43), (127.56, 39.31), (127.62, 39.33),
        (127.60, 39.45), (127.56, 39.56), (127.58, 39.68),
        # Hamhung's coast at Hungnam, Sinpo, Tanchon, Kimchaek, Musudan, Orang, Chongjin, Rajin
        (127.67, 39.79), (127.82, 39.85), (127.97, 39.94), (128.18, 40.00), (128.38, 40.10), (128.65, 40.29),
        (128.92, 40.41), (129.12, 40.57), (129.25, 40.64), (129.45, 40.79), (129.62, 40.86), (129.73, 40.90),
        (129.75, 41.08), (129.82, 41.25), (129.82, 41.40), (129.74, 41.55), (129.78, 41.70), (129.85, 41.76),
        (129.97, 41.85), (130.07, 41.96), (130.17, 42.10), (130.32, 42.19), (130.46, 42.25), (130.62, 42.28),
        # The Tumen's mouth and Russia's coast: Posyet, Slavyanka, Amur Bay, Vladivostok's
        # peninsula, Ussuri Bay, Nakhodka, Cape Povorotny
        (130.70, 42.29), (130.76, 42.45), (130.80, 42.58), (130.92, 42.65), (131.10, 42.60), (131.20, 42.56),
        (131.25, 42.68), (131.40, 42.80), (131.45, 42.95), (131.55, 43.10), (131.75, 43.25), (131.83, 43.36),
        (131.97, 43.30), (131.93, 43.22), (131.88, 43.17), (131.86, 43.10), (131.89, 43.05), (131.96, 43.06), (132.05, 43.15), (132.10, 43.25),
        (132.25, 43.33), (132.36, 43.27), (132.36, 43.13), (132.45, 43.05), (132.55, 42.86), (132.75, 42.76),
        (132.85, 42.76), (132.90, 42.86), (132.96, 42.82), (133.03, 42.67), (133.25, 42.75), (133.60, 42.85),
        (134.20, 43.00), (135.20, 43.60), (136.00, 44.40), (137.40, 45.50), (138.50, 47.00), (140.30, 48.50),
        (140.50, 50.00),
        # Far round the north and west, then the Bohai and the Liaodong peninsula
        (140.50, 56.00), (110.00, 56.00), (110.00, 37.00), (118.80, 37.50), (117.80, 38.00), (117.70, 38.70),
        (118.00, 39.10), (119.00, 39.30), (119.60, 39.90), (120.50, 40.30), (121.10, 40.85), (121.80, 40.85),
        (122.20, 40.55), (122.00, 40.20), (121.80, 39.75), (121.50, 39.40), (121.15, 39.00), (121.25, 38.80),
        (121.70, 38.95), (122.10, 39.15), (122.50, 39.40), (122.90, 39.55), (123.20, 39.70), (123.60, 39.75),
        (123.90, 39.80), (124.15, 39.85), (124.28, 39.88),
        # North Korea's west coast: the Yalu's mouth, Cholsan, the Chongchon's mouth, the coast down
        # to the Taedong, Changsan Cape, Ongjin, Haeju Bay, Yonan to the Han estuary
        (124.40, 39.85), (124.55, 39.76), (124.65, 39.62), (124.85, 39.70), (124.95, 39.60), (125.10, 39.56),
        (125.22, 39.58), (125.35, 39.57), (125.30, 39.45), (125.18, 39.30), (125.12, 39.12), (125.02, 38.97),
        (125.05, 38.82), (125.12, 38.74), (125.19, 38.70), (125.22, 38.66), (125.10, 38.65), (125.03, 38.55),
        (124.95, 38.40), (124.85, 38.25), (124.73, 38.13), (124.88, 38.08), (124.97, 38.02), (124.95, 37.92),
        (125.05, 37.80), (125.15, 37.76), (125.25, 37.85), (125.35, 37.89), (125.42, 37.82), (125.50, 37.75),
        (125.56, 37.85), (125.62, 37.96), (125.72, 38.01), (125.80, 37.97), (125.88, 37.90), (125.95, 37.80),
        (126.10, 37.84), (126.22, 37.88), (126.35, 37.85), (126.50, 37.82), (126.63, 37.79),
        # The Han estuary's south shore, Gimpo, Incheon and Gyeonggi Bay, then the west coast south
        (126.66, 37.74), (126.57, 37.72), (126.56, 37.62), (126.62, 37.52), (126.60, 37.40), (126.65, 37.25),
        (126.75, 37.10), (126.85, 36.90), (126.35, 36.95), (126.15, 36.75), (126.50, 36.40), (126.70, 36.00),
        (126.45, 35.60), (126.50, 35.00), (126.30, 34.70),
    ],
    # Ganghwa Island in the Han estuary, Baengnyeong and Yeonpyeong off the North's coast,
    # Sinmi Island off Cholsan, Russky Island off Vladivostok, Ulleung Island in the Sea of Japan.
    [(126.38, 37.80), (126.51, 37.78), (126.53, 37.65), (126.45, 37.59), (126.36, 37.65)],
    [(124.61, 37.95), (124.70, 37.99), (124.75, 37.96), (124.68, 37.91)],
    [(125.66, 37.66), (125.71, 37.69), (125.74, 37.66), (125.69, 37.63)],
    [(124.80, 39.53), (124.90, 39.60), (124.97, 39.55), (124.88, 39.48)],
    [(131.70, 43.00), (131.80, 43.02), (131.91, 43.015), (131.92, 42.97), (131.84, 42.93), (131.74, 42.94)],
    [(130.80, 37.48), (130.87, 37.54), (130.93, 37.50), (130.86, 37.45)],
]

# Reservoirs and lakes.
LAKES = [
    # Soyang Lake above Chunchon, the Chosin (Changjin) Reservoir, the Supung Reservoir on the
    # Yalu, the Huanren Reservoir on the Hun, Songhua Lake above Jilin, Paektu's crater lake.
    [(127.82, 37.95), (127.90, 37.98), (128.02, 38.02), (128.08, 37.98), (127.95, 37.93), (127.85, 37.92)],
    [(127.20, 40.30), (127.27, 40.33), (127.28, 40.45), (127.25, 40.55), (127.20, 40.50), (127.21, 40.38)],
    [(124.93, 40.47), (125.05, 40.55), (125.20, 40.62), (125.38, 40.72), (125.40, 40.68), (125.22, 40.57),
     (125.07, 40.49), (124.96, 40.44)],
    [(125.30, 41.20), (125.42, 41.27), (125.55, 41.38), (125.58, 41.34), (125.45, 41.22), (125.33, 41.17)],
    [(126.60, 43.62), (126.70, 43.70), (126.85, 43.68), (126.98, 43.55), (126.92, 43.50), (126.75, 43.60)],
    [(128.04, 42.02), (128.08, 42.03), (128.10, 42.00), (128.06, 41.98)],
]

# Rivers: polylines of (lon, lat, width in theater km).
RIVERS = [
    # Yalu (Amnok) from Paektu past Hyesan, Junggang, Manpo, the Supung dam and Sinuiju to its
    # estuary
    [(128.08, 41.98, 0.04), (128.15, 41.62, 0.06), (128.18, 41.42, 0.08), (127.85, 41.43, 0.09),
     (127.55, 41.48, 0.1), (127.25, 41.60, 0.1), (126.90, 41.77, 0.12), (126.65, 41.60, 0.12),
     (126.45, 41.35, 0.12), (126.27, 41.16, 0.13), (126.05, 41.00, 0.13), (125.75, 40.88, 0.14),
     (125.45, 40.75, 0.14), (125.25, 40.60, 0.14), (124.95, 40.45, 0.15), (124.72, 40.30, 0.16),
     (124.50, 40.18, 0.18), (124.39, 40.115, 0.2), (124.33, 40.03, 0.4), (124.30, 39.95, 0.8),
     (124.29, 39.87, 1.5)],
    # Hun, through Tonghua and the Huanren Reservoir to the Yalu
    [(126.20, 42.15, 0.05), (125.94, 41.75, 0.06), (125.60, 41.45, 0.07), (125.45, 41.27, 0.08),
     (125.30, 41.00, 0.08), (125.25, 40.75, 0.08), (125.30, 40.67, 0.08)],
    # Chongchon past Huichon, Kujang, Kaechon and Anju to the Yellow Sea
    [(126.75, 40.40, 0.05), (126.28, 40.15, 0.07), (126.10, 39.90, 0.08), (125.90, 39.72, 0.09),
     (125.67, 39.60, 0.1), (125.50, 39.57, 0.2), (125.34, 39.565, 0.5)],
    # Taedong past Tokchon, Sunchon and Pyongyang to Nampo and its estuary
    [(126.75, 40.05, 0.05), (126.30, 39.73, 0.07), (126.05, 39.48, 0.08), (125.90, 39.25, 0.1),
     (125.76, 39.02, 0.12), (125.62, 38.90, 0.13), (125.50, 38.78, 0.15), (125.40, 38.715, 0.3),
     (125.28, 38.69, 0.6), (125.20, 38.68, 1.0)],
    # Chaeryong through the Hwanghae plain to the Taedong
    [(125.55, 38.20, 0.04), (125.62, 38.40, 0.05), (125.60, 38.60, 0.06), (125.55, 38.75, 0.06)],
    # Yesong from Pyongsan past Kaesong to the Han estuary
    [(126.55, 38.70, 0.04), (126.42, 38.40, 0.05), (126.40, 38.10, 0.06), (126.38, 37.86, 0.1)],
    # Imjin across the DMZ past Yeoncheon and Munsan to the Han
    [(127.35, 39.00, 0.04), (127.18, 38.70, 0.05), (127.05, 38.40, 0.06), (127.07, 38.10, 0.07),
     (126.95, 37.96, 0.08), (126.80, 37.90, 0.1), (126.68, 37.765, 0.15)],
    # Han from Paldang through Seoul to the estuary; the Bukhan past Chunchon to it
    [(127.30, 37.52, 0.12), (127.10, 37.53, 0.14), (126.98, 37.515, 0.15), (126.85, 37.57, 0.16),
     (126.75, 37.65, 0.2), (126.68, 37.73, 0.4), (126.63, 37.77, 0.8)],
    [(127.75, 38.40, 0.05), (127.80, 38.10, 0.06), (127.70, 37.87, 0.07), (127.50, 37.80, 0.07),
     (127.38, 37.65, 0.08), (127.30, 37.52, 0.08)],
    # Tumen from Paektu past Musan, Hoeryong, Tumen city and Hunchun to the sea
    [(128.25, 42.05, 0.04), (128.75, 42.12, 0.05), (129.20, 42.25, 0.06), (129.55, 42.35, 0.07),
     (129.75, 42.47, 0.08), (129.85, 42.70, 0.08), (129.88, 42.95, 0.09), (130.05, 42.90, 0.1),
     (130.20, 42.70, 0.1), (130.35, 42.60, 0.11), (130.55, 42.42, 0.12), (130.68, 42.30, 0.15)],
    # Second Songhua from the Changbai Mountains to Songhua Lake and Jilin
    [(127.95, 42.20, 0.05), (127.50, 42.50, 0.06), (127.15, 42.80, 0.07), (126.95, 43.15, 0.08),
     (126.95, 43.52, 0.1), (126.62, 43.64, 0.1), (126.55, 43.85, 0.12), (126.45, 44.30, 0.12)],
    # Razdolnaya past Ussuriysk to Amur Bay
    [(131.40, 44.10, 0.05), (131.80, 43.85, 0.06), (131.95, 43.75, 0.06), (131.90, 43.55, 0.07),
     (131.83, 43.36, 0.1)],
]

# ---------------------------------------------------------------------------------------------
# Sides

# The Military Demarcation Line from the Han estuary past Panmunjom, north of Yeoncheon and
# Chorwon, over the Iron Triangle, the Punchbowl and down to the east coast below Kosong:
# (lon, lat, intensity). Hottest on the Kaesong-Munsan and Chorwon corridors.
FRONT = [
    (126.66, 37.77, 0.3), (126.68, 37.87, 0.4), (126.68, 37.96, 0.45), (126.80, 37.99, 0.45),
    (126.95, 38.05, 0.4), (127.05, 38.17, 0.35), (127.20, 38.26, 0.4), (127.35, 38.30, 0.45),
    (127.50, 38.31, 0.4), (127.65, 38.30, 0.3), (127.80, 38.28, 0.25), (127.95, 38.30, 0.25),
    (128.10, 38.32, 0.25), (128.20, 38.43, 0.25), (128.30, 38.54, 0.25), (128.38, 38.62, 0.25),
]

# The friendly side: South Korea south of the line, its islands off the North's coast below the
# Northern Limit Line, and the seas south of it (the carriers' waters).
BLUE = (
    [(123.0, 33.0), (123.0, 38.05), (124.85, 38.03), (125.05, 37.73), (125.50, 37.72), (125.62, 37.76),
     (125.95, 37.76), (126.20, 37.82), (126.50, 37.80)]
    + [(lon, lat) for lon, lat, _ in FRONT]
    + [(128.60, 38.70), (130.5, 38.70), (134.0, 37.0), (134.0, 33.0)]
)

# Neutral places: none, the Chinese and Soviet borderlands are on the enemy's side.
NEUTRAL = set()

# ---------------------------------------------------------------------------------------------
# Places

# Towns: (name, lon, lat, kind 0 village / 1 town / 2 city, radius in theater metres, ruined
# share, in the world name table).
TOWNS = [
    # South Korea
    ("Seoul", 126.98, 37.56, 2, 3500, 0.0, True),
    ("Chunchon", 127.73, 37.87, 1, 1400, 0.0, True),
    ("Uijongbu", 127.05, 37.74, 1, 1100, 0.0, False),
    ("Dongducheon", 127.06, 37.91, 0, 800, 0.0, False),
    ("Munsan", 126.77, 37.85, 0, 700, 0.0, False),
    ("Pocheon", 127.20, 37.89, 0, 700, 0.0, False),
    ("Cheorwon", 127.31, 38.15, 0, 600, 0.0, False),
    ("Gapyeong", 127.51, 37.83, 0, 600, 0.0, False),
    ("Hongcheon", 127.89, 37.69, 0, 700, 0.0, False),
    ("Hwacheon", 127.71, 38.11, 0, 600, 0.0, False),
    ("Inje", 128.17, 38.07, 0, 600, 0.0, False),
    ("Sokcho", 128.57, 38.20, 1, 900, 0.0, False),
    ("Gangneung", 128.88, 37.76, 1, 1200, 0.0, False),
    ("Ganghwa", 126.48, 37.74, 0, 600, 0.0, False),
    # North Korea: the west
    ("Pyongyang", 125.75, 39.03, 2, 3300, 0.0, True),
    ("Sunan", 125.68, 39.19, 0, 700, 0.0, True),
    ("Nampo", 125.40, 38.75, 2, 1800, 0.0, True),
    ("Kaesong", 126.55, 37.97, 2, 1700, 0.0, True),
    ("Haeju", 125.72, 38.05, 1, 1400, 0.0, True),
    ("Sariwon", 125.76, 38.51, 1, 1400, 0.0, False),
    ("Songnim", 125.63, 38.75, 1, 1000, 0.0, False),
    ("Namchong Am", 126.45, 38.33, 0, 700, 0.0, True),
    ("Pyongsong", 125.92, 39.25, 1, 1300, 0.0, False),
    ("Sunchon", 125.93, 39.43, 1, 1100, 0.0, False),
    ("Anju", 125.67, 39.63, 1, 1100, 0.0, False),
    ("Kaechon", 125.90, 39.70, 1, 1000, 0.0, False),
    ("Chongju", 125.32, 39.69, 1, 900, 0.0, False),
    ("Taechon", 125.42, 39.94, 0, 700, 0.0, True),
    ("Kusong", 125.25, 39.98, 1, 900, 0.0, False),
    ("Sinuiju", 124.42, 40.08, 2, 1700, 0.0, True),
    ("Huichon", 126.28, 40.18, 1, 900, 0.0, False),
    ("Tokchon", 126.30, 39.76, 1, 900, 0.0, False),
    ("Kanggye", 126.60, 40.97, 1, 1300, 0.0, True),
    ("Manpo", 126.30, 41.14, 1, 900, 0.0, False),
    ("Pyonggang", 127.30, 38.42, 0, 700, 0.0, False),
    # North Korea: the east
    ("Wonsan", 127.43, 39.15, 2, 1700, 0.0, True),
    ("Kosong", 128.30, 38.72, 0, 700, 0.0, False),
    ("Hamhung", 127.53, 39.92, 2, 2200, 0.0, True),
    ("Hungnam", 127.62, 39.84, 1, 1200, 0.0, False),
    ("Sinpo", 128.18, 40.04, 1, 900, 0.0, False),
    ("Tanchon", 128.90, 40.46, 1, 1000, 0.0, False),
    ("Kimchaek", 129.19, 40.68, 1, 1300, 0.0, False),
    ("Kilju", 129.33, 40.96, 1, 900, 0.0, False),
    ("Yong-an", 129.20, 41.05, 0, 700, 0.0, True),
    ("Chongjin", 129.76, 41.80, 2, 2000, 0.0, True),
    ("Rajin", 130.28, 42.25, 1, 1000, 0.0, False),
    ("Hoeryong", 129.74, 42.43, 1, 1000, 0.0, True),
    ("Musan", 129.21, 42.23, 1, 900, 0.0, False),
    ("Hyesan", 128.17, 41.39, 1, 1300, 0.0, True),
    ("Changjin", 127.33, 40.40, 0, 600, 0.0, False),
    # China
    ("Dandong", 124.37, 40.14, 2, 1800, 0.0, True),
    ("Kuandian", 124.78, 40.73, 0, 700, 0.0, False),
    ("Ji'an", 126.18, 41.12, 1, 800, 0.0, False),
    ("Tonghua", 125.94, 41.73, 2, 1600, 0.0, True),
    ("Hunjiang", 126.42, 41.94, 1, 1300, 0.0, True),
    ("Linjiang", 126.92, 41.81, 0, 700, 0.0, False),
    ("Changbai", 128.20, 41.42, 0, 700, 0.0, True),
    ("Fusong", 127.27, 42.33, 1, 800, 0.0, True),
    ("Huanren", 125.36, 41.27, 0, 700, 0.0, False),
    ("Meihekou", 125.68, 42.53, 1, 1000, 0.0, False),
    ("Panshi", 126.06, 42.94, 1, 800, 0.0, False),
    ("Huadian", 126.75, 42.97, 1, 900, 0.0, False),
    ("Jilin", 126.55, 43.84, 2, 2600, 0.0, True),
    ("Changchun", 125.32, 43.88, 2, 3200, 0.0, True),
    ("Siping", 124.37, 43.17, 1, 1400, 0.0, False),
    ("Dunhua", 128.23, 43.37, 1, 1100, 0.0, False),
    ("Antu", 128.90, 43.11, 0, 700, 0.0, False),
    ("Yanji", 129.50, 42.89, 2, 1600, 0.0, True),
    ("Longjing", 129.42, 42.77, 1, 900, 0.0, False),
    ("Tumen", 129.84, 42.97, 1, 900, 0.0, False),
    ("Wangqing", 129.77, 43.32, 0, 700, 0.0, False),
    ("Diyingou", 129.62, 43.50, 0, 600, 0.0, True),
    ("Hunchun", 130.36, 42.86, 1, 900, 0.0, False),
    # Russia
    ("Vladivostok", 131.90, 43.12, 2, 2800, 0.0, True),
    ("Artyom", 132.18, 43.36, 1, 1100, 0.0, False),
    ("Ussuriysk", 131.95, 43.80, 2, 1600, 0.0, True),
    ("Bolshoy Kamen", 132.40, 43.12, 0, 700, 0.0, False),
    ("Nakhodka", 132.88, 42.82, 2, 1600, 0.0, True),
    ("Partizansk", 133.10, 43.13, 1, 900, 0.0, False),
    ("Slavyanka", 131.38, 42.86, 0, 700, 0.0, False),
    ("Kraskino", 130.80, 42.71, 0, 600, 0.0, False),
    ("Poltavka", 131.36, 44.03, 0, 600, 0.0, True),
]

# Airbases: (name, lon, lat, friendly). The runway runs north-south.
AIRBASES = [
    # South Korea: Camp Page at Chunchon.
    ("Chunchon", 127.74, 37.89, True),
    # North Korea: Mirim at Pyongyang, Sunan, Onchon (Nampo), Sinuiju, Taechon, Kalma (Wonsan),
    # Sondok (Hamhung), Orang (Chongjin).
    ("Pyongyang", 125.86, 39.00, False),
    ("Sunan", 125.67, 39.22, False),
    ("Nampo", 125.23, 38.92, False),
    ("Sinuiju", 124.41, 40.05, False),
    ("Taechon", 125.49, 39.90, False),
    ("Wonsan", 127.49, 39.17, False),
    ("Hamhung", 127.47, 39.75, False),
    ("Chongjin", 129.65, 41.43, False),
    # China: Tonghua, Hunjiang (Baishan), Changchun's Dafangshen, Yanji, Diyingou.
    ("Tonghua", 125.70, 42.25, False),
    ("Hunjiang", 126.42, 41.90, False),
    ("Changchun", 125.22, 43.90, False),
    ("Yanji", 129.45, 42.88, False),
    ("Diyingou", 129.55, 43.46, False),
    # Russia: Knevichi (Vladivostok), Nakhodka, Poltavka.
    ("Vladivostok", 132.15, 43.40, False),
    ("Nakhodka", 132.85, 42.93, False),
    ("Poltavka", 131.40, 44.05, False),
]

# The US carrier group in the Sea of Japan, off the DMZ's east end.
CARRIER = [("USS Nimitz", 130.15, 38.50), ("USS America", 131.25, 40.58)]

# The theater's own site layouts within a 2 km cell, from its library's kinds:
# (metres east, metres north, kind).
TEMPLATES = {
    # A Scud brigade: its headquarters, three launchers on their pads, the missile depots.
    "scud_base": [(0, 0, "headquarters"), (-600, -600, "ss_missile"), (600, 600, "ss_missile"), (-550, 600, "ss_missile"),
                  (-700, 0, "depot"), (700, -100, "depot"), (100, -700, "supply_dump"), (650, -650, "depot")],
    # A corps or army headquarters with its stores and a tank company.
    "hq_ko": [(0, 0, "headquarters"), (-400, 0, "depot"), (400, 150, "supply_dump"), (-300, -550, "t_72_tanks"),
              (0, -600, "t_72_tanks"), (300, -550, "t_72_tanks")],
    # A supply depot: depots in a row, the dumps and its fuel.
    "depot_ko": [(-350, 0, "depot"), (0, 0, "depot"), (350, 0, "depot"), (-300, -450, "supply_dump"),
                 (300, -450, "supply_dump"), (0, 500, "tank_farm")],
    # Oil storage at a refinery or a port.
    "tankfarm_ko": [(-150, -150, "tank_farm"), (150, -150, "tank_farm"), (-150, 150, "tank_farm"), (150, 150, "tank_farm"),
                    (550, 0, "depot")],
    # Armour along a road behind the DMZ, with its supply dump.
    "armor_ko": [(-700 + 180 * k, -500 + 120 * k, "t_72_tanks") for k in range(6)] + [(450, 350, "supply_dump")],
    # A lone missile launcher with its stores.
    "ss_ko": [(0, 0, "ss_missile"), (-250, 200, "depot"), (250, -300, "supply_dump")],
}

# Enemy target sites: (template, lon, lat, name).
SITES = [
    # The two Scud brigades of the north-east
    ("scud_base", 129.15, 41.02, "Yong-an"), ("scud_base", 129.62, 42.36, "Hoeryong"),
    # Missile bases behind the DMZ
    ("ss_ko", 126.05, 38.58, "Sakkanmol"), ("ss_ko", 127.15, 38.70, "Chiha-ri"),
    # Headquarters: the General Staff at Pyongyang, II Corps at Kaesong, the wartime command at
    # Kanggye, VII Corps at Hamhung
    ("hq_ko", 125.82, 39.10, "Pyongyang"), ("hq_ko", 126.50, 38.03, "Kaesong"), ("hq_ko", 126.55, 40.92, "Kanggye"),
    ("hq_ko", 127.45, 39.98, "Hamhung"),
    # Supply depots
    ("depot_ko", 125.82, 38.47, "Sariwon"), ("depot_ko", 127.33, 38.45, "Pyonggang"), ("depot_ko", 125.73, 39.58, "Anju"),
    ("depot_ko", 125.30, 40.02, "Kusong"), ("depot_ko", 126.35, 41.10, "Manpo"), ("depot_ko", 127.30, 39.12, "Wonsan"),
    ("depot_ko", 128.12, 40.07, "Sinpo"), ("depot_ko", 129.68, 41.73, "Chongjin"), ("depot_ko", 128.12, 41.36, "Hyesan"),
    ("depot_ko", 125.98, 41.78, "Tonghua"), ("depot_ko", 129.55, 42.92, "Yanji"), ("depot_ko", 131.92, 43.75, "Ussuriysk"),
    # Oil: Nampo's port, the Ponghwa refinery by Sinuiju, Hungnam, Chongjin, the Sungri
    # refinery at Sonbong, Nakhodka's oil port
    ("tankfarm_ko", 125.45, 38.77, "Nampo"), ("tankfarm_ko", 124.58, 40.08, "Sinuiju"), ("tankfarm_ko", 127.60, 39.87, "Hungnam"),
    ("tankfarm_ko", 129.72, 41.83, "Chongjin"), ("tankfarm_ko", 130.38, 42.31, "Rajin"), ("tankfarm_ko", 132.86, 42.89, "Nakhodka"),
    # Armour massed behind the DMZ and on the roads south from Pyongyang
    ("armor_ko", 126.58, 38.06, "Kaesong"), ("armor_ko", 126.80, 38.10, "Kaesong"), ("armor_ko", 126.98, 38.20, "Namchong Am"),
    ("armor_ko", 127.20, 38.36, "Pyonggang"), ("armor_ko", 127.45, 38.40, "Pyonggang"), ("armor_ko", 127.72, 38.40, "Pyonggang"),
    ("armor_ko", 127.98, 38.44, "Kosong"), ("armor_ko", 128.22, 38.64, "Kosong"), ("armor_ko", 125.90, 38.62, "Sariwon"),
    ("armor_ko", 125.95, 38.95, "Pyongyang"),
]

# Cities of the enemy side with a SAM site at their edge.
CITY_SAMS = ["Kaesong", "Haeju", "Sariwon", "Kanggye", "Hyesan", "Kimchaek", "Ussuriysk", "Jilin"]

# Enemy missile boats: off Kosong, off Tanchon, off Rajin, in Peter the Great Gulf and off
# Ussuri Bay: (lon, lat).
BOATS = [(128.60, 38.85), (129.45, 39.95), (130.25, 41.80), (131.60, 42.70), (132.30, 42.70), (131.95, 42.30)]

# Other ships: (kind, lon, lat, radar or None). The Nimitz's destroyers, the second carrier
# group (USS America) further north-east, merchantmen in the shipping lanes and armed fishing
# boats off the North's coasts.
SHIPS = [
    ("destroyer", 130.08, 38.46, None), ("destroyer", 130.23, 38.53, None), ("destroyer", 130.12, 38.58, None),
    ("destroyer", 130.21, 38.43, None), ("destroyer", 130.05, 38.54, None),
    ("destroyer", 131.18, 40.54, None), ("destroyer", 131.33, 40.61, None), ("destroyer", 131.22, 40.66, None),
    ("destroyer", 131.31, 40.51, None), ("destroyer", 131.15, 40.62, None),
    ("ship", 124.45, 38.40, None), ("ship", 129.60, 38.49, None), ("ship", 128.30, 39.20, None),
    ("ship", 128.00, 39.40, None), ("ship", 129.05, 39.74, None), ("ship", 130.70, 39.74, None),
    ("ship", 131.80, 40.16, None), ("ship", 130.70, 41.00, None), ("ship", 130.10, 41.30, None),
    ("ship", 132.35, 42.62, None),
    ("armed_fishing_boat", 132.77, 37.76, None), ("armed_fishing_boat", 124.50, 38.60, None),
    ("armed_fishing_boat", 131.10, 38.60, None), ("armed_fishing_boat", 128.10, 39.15, None),
    ("armed_fishing_boat", 128.36, 39.85, None), ("armed_fishing_boat", 128.91, 40.26, None),
    ("armed_fishing_boat", 129.46, 40.62, None), ("armed_fishing_boat", 130.00, 41.05, None),
    ("armed_fishing_boat", 132.20, 41.50, None), ("armed_fishing_boat", 130.40, 41.85, None),
    ("armed_fishing_boat", 131.10, 42.35, None), ("armed_fishing_boat", 131.00, 42.48, None),
]

# Mountain ranges: (lon, lat, length theater km, width theater km, height m, bearing of the
# long axis in degrees from north).
MOUNTAINS = [
    # Paektu and the Hamgyong range along the north-east
    (128.08, 41.99, 20, 16, 2700, 0), (128.00, 40.75, 32, 12, 2200, 55), (128.55, 41.15, 32, 12, 2500, 50),
    (129.05, 41.55, 30, 12, 2200, 35), (129.35, 42.00, 24, 10, 1800, 20),
    # The Nangnim and Pujollyong spine down the middle, the Myohyang range, the Kangnam range
    # along the Yalu
    (127.10, 40.60, 30, 12, 2100, 10), (126.95, 40.05, 30, 12, 1900, 20), (126.25, 40.00, 22, 10, 1900, 45),
    (125.70, 40.65, 30, 10, 1500, 60), (126.40, 41.02, 30, 10, 1600, 60),
    # The Taebaek range down the east coast: the Masingryong, Kumgang and Sorak
    (127.40, 39.25, 24, 10, 1300, 150), (127.80, 38.85, 26, 10, 1400, 150), (128.10, 38.50, 26, 10, 1600, 160),
    (128.42, 38.05, 26, 10, 1700, 160),
    # The Changbai and Longgang mountains in China, the Zhangguangcai north of Yanji
    (128.45, 42.65, 30, 14, 1600, 40), (126.25, 42.20, 30, 12, 1200, 50), (128.90, 43.80, 30, 12, 1100, 30),
    # The southern Sikhote-Alin behind Nakhodka
    (133.00, 43.40, 30, 12, 1300, 35),
]

# Forest regions: (lon, lat, radius theater km, density 0..1).
FORESTS = [
    (127.05, 40.55, 40, 0.7), (128.50, 41.25, 40, 0.75), (128.10, 42.10, 30, 0.8), (126.20, 40.80, 35, 0.6),
    (126.60, 39.70, 25, 0.55), (127.85, 38.75, 30, 0.6), (128.35, 38.05, 22, 0.6), (127.55, 37.95, 20, 0.5),
    (127.85, 42.60, 40, 0.75), (126.50, 42.30, 35, 0.6), (129.20, 43.40, 35, 0.6), (130.30, 43.20, 25, 0.5),
    (133.00, 43.50, 30, 0.7), (126.45, 38.55, 20, 0.35), (125.60, 41.40, 25, 0.5), (129.40, 42.10, 25, 0.6),
]

GREEN = True


# ---------------------------------------------------------------------------------------------
# Models of this theater's library the layouts use, and its airfield's aprons.

KINDS = {"houses": 26, "houses2": 26, "buildings": 26, "carrier": 75, "carrier_object": 75, "patrol_boat": 53,
         "tank": 98}
APRONS = [(-500, -78, -180, 180), (78, 500, -173, 155)]
