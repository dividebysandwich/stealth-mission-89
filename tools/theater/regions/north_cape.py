"""Geography of the North Cape theater (1985): northern Norway, Finnish and Swedish Lapland, the
top of the Gulf of Bothnia, and the Soviet Kola peninsula, Karelia's White Sea coast and
Arkhangelsk.

The theater squeezes 19.0-44.0 E, 63.92-73.08 N (about 1020 by 1020 km) into the 512 km of a
theater. Coordinates here are (lon, lat) degrees.

Sides: Norway, Finland and Sweden are the friendly side (the neutral Swedish airfields of Kiruna
and Lulea are neutral ground); the Soviet Union east of the Norwegian and Finnish borders is the
enemy, with the Northern Fleet's bases, sub pens and airfields round Murmansk and Severomorsk.
There is no shooting war: the front line only marks the border (intensity 0).
"""

# The south-west corner of the theater, the latitude the scale is taken at, and how many real
# km a theater km stands for.
PROJECTION = dict(lon0=19.0, lat0=63.92, lat_ref=68.5, squeeze=1.99)

LIBRARY = "north_cape"
LAND_RUBBLE = 66

# Briefing date of the first mission (month, day, year), grid reference letters and the corner's
# offset, squadrons per enemy airbase at most.
DATE = (3, 1, 1985)
GRID = ("WX", 0, 0)
SQUADRONS = 3

# The air forces: (aircraft, its air-to-air missile, task), units.ron ids. The first is the
# special aircraft of the intercept missions; the last four are friendly.
ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("yak38", "aa8", "base"),
    ("su27", "aa10t", "patrol"),
    ("mig29", "aa10", "patrol"),
    ("a50", "aa10", "patrol_wide"),
    ("mig31", "aa9", "patrol"),
    ("yak38", "aa8", "base"),
    ("su27", "aa10t", "escort"),
    ("mig25", "aa6t", "escort"),
    ("f18d", "none", "friendly"),
    ("f18d", "none", "friendly"),
    ("f18", "none", "friendly"),
    ("f4e_blue", "none", "friendly"),
]

# Radar types (units.ron ids): the fixed SAM sites (SA-5 and SA-10), the mobile ones (SA-8 and
# SA-11). The theater has no patrol boats (BOATS is empty); the frigates' SA-N-4 stands in.
SAM_RADARS = ["sa5", "sa10"]
MOBILE_SAM_RADARS = ["sa8", "sa11"]
BOAT_RADARS = ["san4"]
SLAG_HEAP_TOWNS = set()

# Models this theater builds besides the water, forests and mountains (see models.py).
EXTRA_MODELS = []

# Tundra on the coasts and fells, taiga in the south: no stray woods outside FORESTS, and the
# open land is bog, heath and rock, not farmland.
SPARSE_FOREST = False
GREEN = False

# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea; the Gulf of Bothnia is cut out of the
# mainland as a sea of its own.

LANDS = [
    # The mainland: the Norwegian coast from the far south-west round the Lyngen peninsula, the
    # Finnmark fjords, North Cape and Varanger; the Murman coast with Rybachy, the Motovsky Gulf
    # and Kola Bay; Svyatoy Nos, the Tersky coast and the Kandalaksha Gulf; Karelia's White Sea
    # coast, Onega Bay, the Onega peninsula, Dvina Bay, the Zimny coast, Mezen Bay and Kanin.
    [
        (5.0, 58.0), (5.0, 62.5), (9.0, 63.8), (12.0, 65.5), (13.5, 67.0), (15.0, 68.0), (16.5, 68.6),
        (17.5, 69.1), (18.3, 69.45), (18.9, 69.62), (19.30, 69.78),
        # Ullsfjord, the Lyngen peninsula, Lyngenfjord to Skibotn, Reisafjord
        (19.60, 69.82), (19.70, 69.60), (19.80, 69.55), (19.82, 69.80), (19.90, 70.02), (20.05, 70.08),
        (20.25, 69.98), (20.32, 69.80), (20.22, 69.55), (20.28, 69.40), (20.40, 69.45), (20.48, 69.62),
        (20.60, 69.82), (20.80, 69.93), (20.92, 69.86), (21.03, 69.80), (21.10, 69.87), (21.05, 69.98),
        # Kvaenangen, Bergsfjord, Oksfjord, Stjernsund, Altafjord to Alta, Vargsund, Kvalsund
        (21.25, 70.05), (21.45, 70.00), (21.70, 69.88), (21.90, 69.78), (22.05, 69.78), (22.00, 69.92),
        (21.85, 70.05), (21.70, 70.15), (21.85, 70.25), (22.10, 70.30), (22.35, 70.25), (22.55, 70.20),
        (22.75, 70.12), (22.95, 70.05), (23.10, 70.00), (23.27, 69.99), (23.40, 70.02), (23.42, 70.10),
        (23.55, 70.25), (23.70, 70.37),
        # Repparfjord, the coast to Magerosundet, Porsangerfjord to Lakselv and back to Svaerholt
        (24.00, 70.42), (24.30, 70.45), (24.50, 70.55), (24.75, 70.65), (25.00, 70.80), (25.30, 70.90),
        (25.60, 70.93), (25.85, 70.90), (25.65, 70.75), (25.35, 70.55), (25.15, 70.35), (25.00, 70.15),
        (24.90, 70.10), (25.05, 70.09), (25.20, 70.12), (25.50, 70.30), (25.80, 70.50), (26.00, 70.70),
        (26.30, 70.95), (26.60, 70.95),
        # Laksefjord, Nordkinn, Tanafjord, Berlevag, Batsfjord, the Varanger peninsula
        (26.80, 70.75), (26.80, 70.55), (26.90, 70.40), (27.10, 70.50), (27.25, 70.75), (27.35, 70.95),
        (27.65, 71.12), (27.85, 71.07), (28.20, 70.95), (28.30, 70.70), (28.25, 70.50), (28.25, 70.37),
        (28.45, 70.55), (28.60, 70.80), (28.85, 70.90), (29.10, 70.87), (29.45, 70.75), (29.75, 70.67),
        (30.10, 70.62), (30.60, 70.52), (31.00, 70.45), (31.15, 70.38),
        # Varangerfjord: Vardo, Vadso, Varangerbotn, Bugoynes, Kirkenes, Grense Jakobselv
        (31.05, 70.28), (30.70, 70.18), (30.30, 70.10), (29.95, 70.07), (29.75, 70.05), (29.40, 70.10),
        (29.00, 70.17), (28.75, 70.18), (28.55, 70.14), (28.70, 70.05), (29.10, 70.00), (29.65, 69.96),
        (29.90, 69.85), (30.00, 69.78), (30.05, 69.76), (30.25, 69.82), (30.55, 69.82), (30.85, 69.79),
        # Pechenga Bay, Rybachy and Sredny, the Motovsky Gulf, Zapadnaya Litsa, Ura Guba, Sayda
        (31.05, 69.75), (31.30, 69.66), (31.22, 69.58), (31.32, 69.59), (31.42, 69.70), (31.65, 69.73),
        (31.80, 69.82), (31.95, 69.95), (32.20, 69.95), (32.45, 69.92), (32.70, 69.88), (32.95, 69.85),
        (32.85, 69.75), (32.55, 69.68), (32.25, 69.62), (32.05, 69.56), (31.95, 69.50), (32.10, 69.45),
        (32.30, 69.43), (32.40, 69.42), (32.40, 69.33), (32.50, 69.40), (32.75, 69.42), (32.90, 69.36),
        (33.05, 69.33), (33.20, 69.31), (33.32, 69.29), (33.48, 69.30),
        # Kola Bay: the west shore past Polyarnyy to Murmansk and Kola, the east shore past
        # Severomorsk back to the mouth
        (33.48, 69.22), (33.40, 69.17), (33.30, 69.10), (33.22, 69.04), (33.05, 69.00), (32.99, 68.95),
        (32.97, 68.88), (33.03, 68.87), (33.05, 68.94), (33.08, 68.98), (33.20, 69.02), (33.36, 69.07),
        (33.47, 69.13), (33.57, 69.21), (33.65, 69.30),
        # The Murman coast: Teriberka, Dalniye Zelentsy, Kharlovka, Iokanga (Gremikha), Svyatoy Nos
        (33.80, 69.27), (34.00, 69.25), (34.30, 69.22), (34.60, 69.21), (34.85, 69.20), (35.05, 69.18),
        (35.15, 69.13), (35.30, 69.15), (35.60, 69.10), (36.05, 69.08), (36.50, 68.95), (37.00, 68.85),
        (37.35, 68.78), (37.70, 68.65), (38.10, 68.50), (38.50, 68.35), (38.90, 68.22), (39.20, 68.12),
        (39.45, 68.10), (39.60, 68.12), (39.80, 68.17), (39.95, 68.05),
        # The east coast to Cape Orlov and Ponoy, the Tersky coast, Umba, the Kandalaksha Gulf
        (40.20, 67.85), (40.55, 67.62), (40.90, 67.35), (41.20, 67.20), (41.15, 67.05), (40.95, 66.85),
        (40.70, 66.62), (40.45, 66.47), (39.95, 66.25), (39.50, 66.15), (39.00, 66.10), (38.50, 66.05),
        (38.00, 66.08), (37.50, 66.15), (37.00, 66.25), (36.60, 66.30), (36.10, 66.40), (35.60, 66.52),
        (35.00, 66.60), (34.50, 66.65), (34.10, 66.75), (33.70, 66.80), (33.30, 66.90), (33.00, 67.00),
        (32.70, 67.08), (32.45, 67.12), (32.40, 67.05), (32.55, 66.95), (32.75, 66.85),
        # Karelia: Kovda, Chupa, Gridino, Kem, Belomorsk, the Pomor coast to Onega
        (32.85, 66.72), (33.00, 66.55), (33.15, 66.35), (33.40, 66.20), (33.70, 66.05), (34.10, 65.92),
        (34.55, 65.85), (34.70, 65.55), (34.75, 65.25), (34.80, 65.00), (34.85, 64.75), (34.85, 64.55),
        (35.10, 64.40), (35.40, 64.25), (35.70, 64.18), (36.00, 64.05), (36.50, 63.95), (37.20, 63.85),
        (37.90, 63.88),
        # The Onega peninsula round its northern tip, the Letny coast past Nyonoksa to Severodvinsk,
        # the Dvina delta below Arkhangelsk
        (38.15, 63.98), (37.90, 64.10), (37.55, 64.25), (37.20, 64.40), (36.90, 64.55), (36.60, 64.72),
        (36.50, 64.85), (36.70, 65.00), (36.85, 65.12), (37.20, 65.10), (37.60, 65.00), (38.10, 64.88),
        (38.60, 64.78), (39.00, 64.70), (39.25, 64.63), (39.55, 64.58), (39.80, 64.58), (40.05, 64.63),
        (40.15, 64.68), (40.25, 64.75), (40.30, 64.85),
        # The Zimny coast to Cape Voronov, Mezen Bay, Kanin's west coast to Kanin Nos
        (40.20, 64.95), (40.05, 65.10), (39.90, 65.25), (39.80, 65.45), (40.00, 65.60), (40.30, 65.80),
        (40.70, 66.00), (41.20, 66.20), (41.70, 66.35), (42.20, 66.50), (42.50, 66.40), (42.80, 66.20),
        (43.20, 66.00), (43.60, 65.90), (44.10, 65.85), (44.25, 66.00), (44.10, 66.20), (43.70, 66.55),
        (43.30, 66.90), (43.40, 67.10), (43.70, 67.40), (43.80, 67.70), (43.95, 68.00), (43.85, 68.25),
        (43.60, 68.50), (43.27, 68.65),
        # Kanin's north coast and on east, coarse outside the theater
        (44.20, 68.55), (45.00, 68.52), (45.90, 68.45), (46.50, 68.10), (46.8, 67.7), (48.0, 67.6),
        (50.0, 68.0), (53.0, 68.5), (56.0, 68.6), (60.0, 69.0), (60.0, 50.0), (5.0, 50.0),
    ],
    # Norwegian islands: Vannoya, Arnoya, Soroya, Stjernoya, Seiland, Kvaloya (Hammerfest),
    # Rolvsoya and Ingoya, Mageroya (North Cape, Honningsvag).
    [(19.55, 70.15), (19.80, 70.25), (20.05, 70.20), (19.85, 70.08)],
    [(20.40, 70.10), (20.55, 70.20), (20.85, 70.15), (20.75, 70.05), (20.50, 70.03)],
    [(21.95, 70.60), (22.30, 70.75), (22.80, 70.72), (23.15, 70.62), (22.90, 70.52), (22.40, 70.50), (22.05, 70.52)],
    [(22.35, 70.30), (22.55, 70.37), (22.70, 70.30), (22.60, 70.25), (22.40, 70.25)],
    [(22.75, 70.35), (22.95, 70.44), (23.35, 70.45), (23.55, 70.38), (23.35, 70.26), (23.10, 70.18), (22.85, 70.22)],
    [(23.45, 70.62), (23.55, 70.78), (23.85, 70.80), (24.10, 70.68), (24.05, 70.52), (23.80, 70.46), (23.55, 70.48)],
    [(23.85, 71.00), (24.05, 71.10), (24.25, 71.02), (24.05, 70.93)],
    [(25.45, 71.05), (25.70, 71.17), (26.00, 71.15), (26.25, 71.05), (26.10, 70.96), (25.85, 70.95), (25.60, 70.97)],
    # Kildin off the Murman coast, the Solovetsky islands and Morzhovets in the White Sea.
    [(33.85, 69.33), (34.10, 69.36), (34.35, 69.31), (34.20, 69.27), (33.95, 69.28)],
    [(35.45, 65.10), (35.70, 65.20), (35.95, 65.12), (35.90, 64.98), (35.60, 64.95)],
    [(42.40, 66.75), (42.55, 66.82), (42.70, 66.76), (42.55, 66.68)],
]

# The Gulf of Bothnia's northern half: the Swedish coast past Skelleftea, Pitea and Lulea, the
# head of the gulf at Kalix, Haparanda and Kemi, the Finnish coast past Oulu and Raahe.
SEAS = [
    [
        (18.0, 60.0), (17.2, 61.0), (17.5, 62.5), (18.6, 63.2), (19.7, 63.5), (20.30, 63.80), (20.80, 64.05),
        (21.10, 64.30), (21.35, 64.55), (21.30, 64.80), (21.50, 65.00), (21.60, 65.25), (21.85, 65.40),
        (22.10, 65.50), (22.20, 65.56), (22.30, 65.62), (22.55, 65.62), (22.75, 65.70), (23.10, 65.75),
        (23.45, 65.78), (23.80, 65.78), (24.15, 65.80), (24.40, 65.72), (24.55, 65.70), (24.85, 65.55),
        (25.20, 65.40), (25.35, 65.20), (25.40, 65.02), (25.20, 64.85), (24.75, 64.75), (24.45, 64.65),
        (24.15, 64.45), (23.90, 64.25), (23.50, 64.05), (23.05, 63.85), (22.50, 63.55), (21.50, 63.0),
        (21.3, 62.0), (21.5, 61.0), (21.4, 60.5), (19.5, 60.0),
    ],
]

# Lakes and reservoirs.
LAKES = [
    # Lake Inari
    [(27.25, 69.05), (27.55, 69.20), (28.00, 69.22), (28.40, 69.10), (28.60, 68.95), (28.30, 68.85),
     (27.90, 68.80), (27.60, 68.78), (27.30, 68.85), (27.10, 68.95)],
    # Imandra from Monchegorsk past Apatity to its southern arm above Polyarnye Zori
    [(32.90, 67.92), (33.05, 67.88), (33.10, 67.75), (33.20, 67.65), (33.25, 67.55), (33.10, 67.48),
     (32.85, 67.45), (32.75, 67.30), (32.60, 67.22), (32.45, 67.25), (32.55, 67.38), (32.70, 67.50),
     (32.85, 67.60), (32.95, 67.72), (32.85, 67.85)],
    # Umbozero and Lovozero either side of the Lovozero massif
    [(34.15, 67.80), (34.35, 67.75), (34.45, 67.55), (34.35, 67.40), (34.20, 67.45), (34.15, 67.65)],
    [(35.00, 67.95), (35.20, 67.92), (35.30, 67.80), (35.15, 67.72), (35.00, 67.80)],
    # The Upper Tuloma reservoir
    [(31.80, 68.62), (31.50, 68.58), (31.15, 68.52), (30.90, 68.47), (30.80, 68.40), (31.00, 68.38),
     (31.30, 68.42), (31.60, 68.50), (31.85, 68.55)],
    # Kovdozero, Pyaozero, Topozero, the Kuyto lakes by Kalevala (Karelia)
    [(31.40, 66.90), (31.80, 66.95), (32.25, 66.85), (32.20, 66.75), (31.80, 66.78), (31.40, 66.75)],
    [(30.60, 66.15), (31.10, 66.22), (31.55, 66.10), (31.20, 66.00), (30.75, 66.02)],
    [(31.70, 65.85), (32.10, 65.82), (32.40, 65.65), (32.20, 65.50), (31.90, 65.55), (31.65, 65.65)],
    [(30.60, 65.05), (31.00, 65.25), (31.40, 65.12), (31.10, 64.95), (30.75, 64.95)],
    # The Lokka and Porttipahta reservoirs, Oulujarvi (Finland)
    [(27.40, 67.85), (27.70, 67.95), (27.95, 67.88), (27.80, 67.78), (27.50, 67.78)],
    [(26.45, 68.05), (26.65, 68.18), (26.80, 68.10), (26.60, 67.98)],
    [(26.95, 64.30), (27.30, 64.45), (27.75, 64.38), (27.55, 64.22), (27.15, 64.18)],
    # Tornetrask (Sweden)
    [(18.70, 68.40), (19.30, 68.40), (19.70, 68.35), (20.10, 68.30), (20.00, 68.22), (19.50, 68.25),
     (19.00, 68.30), (18.70, 68.33)],
]

# Rivers: polylines of (lon, lat, width in theater km).
RIVERS = [
    # Tana (the Norwegian-Finnish border) from Karigasniemi past Utsjoki to the Tanafjord
    [(25.85, 69.40, 0.08), (26.20, 69.60, 0.1), (26.65, 69.80, 0.1), (27.03, 69.91, 0.12), (27.60, 70.00, 0.12),
     (28.00, 70.10, 0.15), (28.15, 70.22, 0.2), (28.22, 70.33, 0.3), (28.25, 70.40, 0.4)],
    # Pasvik from Lake Inari to Kirkenes (the Norwegian-Soviet border)
    [(28.55, 68.98, 0.12), (28.75, 69.02, 0.12), (29.10, 69.20, 0.15), (29.35, 69.32, 0.15), (29.75, 69.40, 0.15),
     (30.05, 69.48, 0.15), (30.10, 69.62, 0.2), (30.10, 69.77, 0.3)],
    # Alta
    [(23.65, 69.40, 0.06), (23.45, 69.70, 0.07), (23.32, 69.92, 0.08), (23.33, 70.01, 0.1)],
    # Tuloma from its reservoir to Kola Bay, the Kola from the south
    [(31.85, 68.60, 0.12), (32.30, 68.72, 0.15), (32.70, 68.85, 0.15), (32.90, 68.90, 0.2), (33.00, 68.93, 0.25)],
    [(33.20, 68.30, 0.06), (33.15, 68.55, 0.07), (33.05, 68.78, 0.08), (33.00, 68.885, 0.1)],
    # Varzuga and Ponoy across the Kola peninsula
    [(35.80, 67.05, 0.05), (36.30, 66.80, 0.06), (36.75, 66.45, 0.07), (36.90, 66.22, 0.08)],
    [(36.80, 67.50, 0.06), (37.10, 67.42, 0.06), (38.50, 67.30, 0.07), (39.80, 67.15, 0.08), (40.80, 67.10, 0.1),
     (41.20, 67.09, 0.12)],
    # Kem to the White Sea
    [(31.50, 64.95, 0.08), (32.50, 64.85, 0.1), (33.50, 64.88, 0.1), (34.30, 64.96, 0.12), (34.85, 65.00, 0.15)],
    # Northern Dvina past Kholmogory, Novodvinsk and Arkhangelsk to its delta, and the Pinega
    [(42.30, 63.40, 0.3), (42.00, 63.80, 0.3), (41.80, 64.10, 0.35), (41.65, 64.22, 0.35), (41.20, 64.35, 0.4),
     (40.82, 64.45, 0.5), (40.48, 64.55, 0.7), (40.40, 64.62, 0.8), (40.20, 64.70, 1.2)],
    [(43.80, 64.75, 0.1), (43.40, 64.71, 0.1), (42.80, 64.60, 0.1), (42.20, 64.35, 0.12), (41.85, 64.12, 0.12)],
    # Kemijoki from Kemijarvi through Rovaniemi to Kemi, and the Ounasjoki from Hetta
    [(27.40, 66.68, 0.1), (26.80, 66.62, 0.1), (26.20, 66.55, 0.12), (25.72, 66.47, 0.15), (25.30, 66.20, 0.15),
     (24.90, 65.95, 0.15), (24.65, 65.68, 0.2)],
    [(23.70, 68.30, 0.06), (24.20, 67.95, 0.06), (24.80, 67.68, 0.07), (25.00, 67.45, 0.08), (25.40, 67.00, 0.09),
     (25.65, 66.60, 0.1), (25.72, 66.47, 0.1)],
    # Torne and Muonio (the Swedish-Finnish border) from Karesuando past Muonio and Pello to the gulf
    [(22.55, 68.40, 0.08), (23.20, 68.20, 0.1), (23.58, 67.95, 0.1), (23.60, 67.70, 0.1), (23.72, 67.35, 0.12),
     (23.80, 67.10, 0.12), (23.85, 66.75, 0.12), (23.68, 66.40, 0.13), (23.95, 66.10, 0.14), (24.15, 65.78, 0.15)],
    # Kalix and Lule (Sweden)
    [(21.20, 67.40, 0.07), (22.00, 67.10, 0.08), (22.70, 66.50, 0.09), (23.05, 66.10, 0.1), (23.25, 65.74, 0.12)],
    [(19.80, 66.58, 0.08), (20.50, 66.20, 0.1), (21.20, 65.95, 0.12), (21.65, 65.80, 0.12), (22.32, 65.57, 0.15)],
    # Oulujoki from Oulujarvi to Oulu
    [(26.95, 64.40, 0.1), (26.30, 64.75, 0.1), (25.80, 64.90, 0.12), (25.30, 64.96, 0.15)],
]

# ---------------------------------------------------------------------------------------------
# Sides

# The Soviet border from Grense Jakobselv up the Pasvik to the three-country cairn, then the
# Finnish-Soviet border south past Salla, Kuusamo and Vartius: (lon, lat, intensity).
FRONT = [(lon, lat, 0.0) for lon, lat in [
    (30.83, 69.79), (30.45, 69.70), (30.15, 69.66), (30.10, 69.60), (30.05, 69.45), (29.75, 69.38),
    (29.35, 69.30), (29.15, 69.17), (28.93, 69.05), (28.80, 68.88), (28.45, 68.55), (28.70, 68.20),
    (29.33, 68.07), (29.80, 67.75), (29.95, 67.50), (29.60, 67.20), (29.20, 66.95), (29.55, 66.60),
    (29.90, 66.25), (29.85, 66.00), (30.10, 65.70), (29.75, 65.30), (29.90, 64.90), (30.10, 64.70),
    (29.95, 64.45), (30.40, 64.10), (30.55, 63.70),
]]

# The friendly side: Norway, Finland and Sweden west of the border, with the Norwegian Sea and
# the Barents Sea off Norway.
BLUE = (
    [(5.0, 58.0), (30.55, 58.0)]
    + [(lon, lat) for lon, lat, _ in reversed(FRONT)]
    + [(31.4, 70.3), (31.3, 70.6), (30.5, 72.0), (30.5, 76.0), (5.0, 76.0)]
)

# Neutral places: the player never flies from or to them, they are no one's targets.
NEUTRAL = {"Kiruna", "Lulea"}

# ---------------------------------------------------------------------------------------------
# Places

# Towns: (name, lon, lat, kind 0 village / 1 town / 2 city, radius in theater metres, ruined
# share, in the world name table).
TOWNS = [
    # Norway
    ("Hammerfest", 23.68, 70.66, 1, 900, 0.0, True),
    ("Alta", 23.27, 69.96, 1, 1100, 0.0, True),
    ("Lakselv", 24.95, 70.05, 0, 600, 0.0, True),
    ("Kautokeino", 23.04, 69.01, 0, 600, 0.0, True),
    ("Karasjok", 25.51, 69.47, 0, 600, 0.0, False),
    ("Honningsvag", 25.97, 70.98, 0, 600, 0.0, False),
    ("Berlevag", 29.09, 70.85, 0, 500, 0.0, False),
    ("Tana Bru", 28.25, 70.20, 0, 500, 0.0, False),
    ("Vadso", 29.75, 70.08, 1, 800, 0.0, True),
    ("Vardo", 31.08, 70.37, 0, 700, 0.0, False),
    ("Kirkenes", 30.05, 69.72, 1, 900, 0.0, True),
    # Finland
    ("Rovaniemi", 25.73, 66.50, 1, 1500, 0.0, True),
    ("Oulu", 25.47, 65.01, 2, 2200, 0.0, False),
    ("Kemi", 24.56, 65.74, 1, 1100, 0.0, False),
    ("Tornio", 24.15, 65.85, 1, 900, 0.0, False),
    ("Sodankyla", 26.59, 67.42, 0, 700, 0.0, True),
    ("Ivalo", 27.54, 68.66, 0, 700, 0.0, False),
    ("Inari", 27.03, 68.91, 0, 500, 0.0, False),
    ("Utsjoki", 27.00, 69.88, 0, 500, 0.0, False),
    ("Kemijarvi", 27.42, 66.71, 1, 900, 0.0, False),
    ("Kittila", 24.91, 67.66, 0, 600, 0.0, False),
    ("Muonio", 23.68, 67.95, 0, 500, 0.0, False),
    ("Hetta", 23.66, 68.38, 0, 500, 0.0, False),
    ("Kolari", 23.79, 67.33, 0, 500, 0.0, False),
    ("Pello", 23.97, 66.77, 0, 500, 0.0, False),
    ("Salla", 28.67, 66.83, 0, 600, 0.0, False),
    ("Kuusamo", 29.18, 65.97, 1, 900, 0.0, False),
    # Sweden
    ("Kiruna", 20.23, 67.86, 1, 1100, 0.0, True),
    ("Gallivare", 20.66, 67.13, 1, 1000, 0.0, True),
    ("Lulea", 22.15, 65.58, 2, 1800, 0.0, True),
    ("Boden", 21.69, 65.83, 1, 1000, 0.0, False),
    ("Pitea", 21.48, 65.32, 1, 1000, 0.0, False),
    ("Kalix", 23.16, 65.85, 1, 800, 0.0, False),
    ("Haparanda", 24.14, 65.84, 0, 600, 0.0, False),
    ("Jokkmokk", 19.85, 66.62, 0, 600, 0.0, False),
    ("Pajala", 23.37, 67.21, 0, 600, 0.0, False),
    ("Overtornea", 23.62, 66.39, 0, 500, 0.0, False),
    ("Karesuando", 22.45, 68.45, 0, 500, 0.0, False),
    # The Soviet Union: Pechenga and Kola Bay
    ("Murmansk", 33.12, 68.97, 2, 3000, 0.0, True),
    ("Severomorsk", 33.42, 69.07, 1, 1200, 0.0, True),
    ("Polyarnyy", 33.42, 69.20, 1, 900, 0.0, True),
    ("Gadzhiyevo", 33.32, 69.26, 0, 600, 0.0, False),
    ("Zaozersk", 32.53, 69.37, 0, 600, 0.0, False),
    ("Pechenga", 31.22, 69.55, 0, 700, 0.0, True),
    ("Zapolyarnyy", 30.80, 69.42, 1, 900, 0.0, False),
    ("Nikel", 30.22, 69.41, 1, 900, 0.0, True),
    ("Kola", 33.06, 68.85, 1, 800, 0.0, False),
    ("Kildinstroy", 33.10, 68.80, 0, 600, 0.0, True),
    ("Murmashi", 32.81, 68.81, 0, 700, 0.0, False),
    ("Teriberka", 35.17, 69.11, 0, 500, 0.0, False),
    ("Ristikent", 31.30, 68.30, 0, 500, 0.0, True),
    # The Kola interior and the Kandalaksha Gulf
    ("Olenegorsk", 33.40, 68.14, 1, 1000, 0.0, True),
    ("Monchegorsk", 32.92, 67.94, 1, 1100, 0.0, True),
    ("Apatity", 33.40, 67.57, 1, 1100, 0.0, False),
    ("Kirovsk", 33.67, 67.61, 1, 900, 0.0, True),
    ("Revda", 34.52, 67.94, 0, 600, 0.0, False),
    ("Lovozero", 34.98, 68.00, 0, 600, 0.0, False),
    ("Polyarnye Zori", 32.50, 67.37, 1, 700, 0.0, False),
    ("Kovdor", 30.47, 67.56, 1, 800, 0.0, False),
    ("Kandalaksha", 32.41, 67.15, 1, 1100, 0.0, True),
    ("Alakurtti", 30.35, 66.96, 0, 600, 0.0, True),
    ("Umba", 34.33, 66.70, 0, 600, 0.0, False),
    ("Varzuga", 36.62, 66.40, 0, 500, 0.0, False),
    ("Krasnoshchelye", 37.05, 67.35, 0, 500, 0.0, False),
    ("Ostrovnoy", 39.47, 68.06, 0, 600, 0.0, False),
    ("Ponoy", 41.05, 67.05, 0, 500, 0.0, False),
    # Karelia
    ("Loukhi", 33.03, 66.07, 0, 600, 0.0, True),
    ("Chupa", 33.05, 66.27, 0, 500, 0.0, False),
    ("Kovda", 32.82, 66.70, 0, 500, 0.0, False),
    ("Kalevala", 31.18, 65.22, 0, 600, 0.0, False),
    ("Kem", 34.60, 64.93, 1, 900, 0.0, True),
    ("Belomorsk", 34.77, 64.52, 1, 900, 0.0, False),
    ("Solovetsky", 35.72, 65.03, 0, 500, 0.0, False),
    # Arkhangelsk
    ("Arkhangelsk", 40.54, 64.54, 2, 3000, 0.0, True),
    ("Severodvinsk", 39.83, 64.56, 2, 2000, 0.0, True),
    ("Novodvinsk", 40.85, 64.40, 1, 900, 0.0, False),
    ("Nyonoksa", 39.22, 64.62, 0, 500, 0.0, False),
    ("Kushkushara", 40.35, 64.96, 0, 500, 0.0, True),
    ("Kholmogory", 41.65, 64.20, 0, 600, 0.0, False),
    ("Pinega", 43.40, 64.67, 0, 600, 0.0, False),
]

# Airbases: (name, lon, lat, side: True friendly, False enemy, None neutral). The runway runs
# north-south.
AIRBASES = [
    # Norway and Sweden
    ("Lakselv", 24.97, 70.07, True),             # Banak
    ("Kautokeino", 23.03, 69.03, True),
    ("Gallivare", 20.81, 67.13, True),
    ("Kiruna", 20.34, 67.82, None),
    ("Lulea", 22.12, 65.54, None),               # Kallax
    # The Soviet Union
    ("Pechenga", 31.00, 69.40, False),           # Luostari
    ("Polyarnyy", 33.42, 69.03, False),          # Severomorsk-1, the Northern Fleet's air base
    ("Kilpyavr", 32.40, 69.10, False),
    ("Kildinstroy", 33.72, 68.87, False),        # Severomorsk-3
    ("Olenegorsk", 33.46, 68.15, False),         # Olenya
    ("Monchegorsk", 33.02, 67.99, False),
    ("Loukhi", 33.05, 66.06, False),
    ("Kem", 34.52, 64.92, False),
    ("Severodvinsk", 39.70, 64.52, False),
    ("Kushkushara", 40.38, 64.96, False),
]

# Enemy airstrips: (name, lon, lat, side).
AIRSTRIPS = [
    ("Krasnoshchelye", 37.10, 67.33, False),
    ("Teriberka", 35.05, 69.10, False),
]

# The friendly carrier group in the Barents Sea north of Nordkinn: (name, lon, lat).
CARRIER = ("USS Kennedy", 27.40, 72.15)

# Layouts of this theater's sites within a 2 km cell (metres east, metres north of the cell
# centre, library kind): its library has no tanks, houses, mobile launchers, camps' huts or
# bridges, so the headquarters, depots, fuel farms, the early warning radar, the mobile SAM
# batteries and the prison camps are built from what it has. An army headquarters stands with
# a pair of SS missiles (as the original theater had them).
TEMPLATES = {
    "hq": [(0, 0, "headquarters"), (-350, 0, "depot"), (300, -250, "ss_missile"), (450, -100, "ss_missile"),
           (-450, -450, "depot"), (500, 450, "tank_farm")],
    "depot": [(-300, 0, "depot"), (0, 0, "depot"), (300, 0, "depot"), (0, 450, "tank_farm"), (-400, -400, "depot")],
    "tankfarm": [(-150, -150, "tank_farm"), (150, -150, "tank_farm"), (-150, 150, "tank_farm"),
                 (150, 150, "tank_farm"), (550, 0, "depot")],
    "oth": [(0, 0, "lpar_radar"), (300, 250, "depot")],
    "msam": [(0, 0, "sam_radar"), (0, 450, "sa_missile"), (428, 139, "sa_missile"), (265, -364, "sa_missile"),
             (-265, -364, "sa_missile"), (-428, 139, "sa_missile")],
    "pow": [(0, 0, "camp"), (350, 120, "depot")],
}

# Enemy target sites: (template, lon, lat, name).
SITES = [
    # Army and fleet headquarters: the Northern Fleet at Severomorsk, the divisions at Pechenga,
    # on the Tuloma, at Alakurtti and Arkhangelsk.
    ("hq", 33.25, 68.93, "Murmansk"), ("hq", 33.50, 69.10, "Severomorsk"), ("hq", 30.95, 69.48, "Pechenga"),
    ("hq", 31.40, 68.28, "Ristikent"), ("hq", 30.42, 66.99, "Alakurtti"), ("hq", 40.62, 64.60, "Arkhangelsk"),
    # Supply depots along the Murmansk railway and the White Sea ports
    ("depot", 32.50, 67.20, "Kandalaksha"), ("depot", 33.15, 68.88, "Murmansk"), ("depot", 40.62, 64.46, "Arkhangelsk"),
    ("depot", 34.48, 64.88, "Kem"), ("depot", 30.88, 69.34, "Pechenga"), ("depot", 33.32, 68.10, "Olenegorsk"),
    ("depot", 34.65, 64.47, "Belomorsk"), ("depot", 39.95, 64.50, "Severodvinsk"), ("depot", 33.12, 66.03, "Loukhi"),
    ("depot", 34.25, 66.74, "Umba"),
    # Fuel farms of the fleet and the ports
    ("tankfarm", 33.28, 69.00, "Murmansk"), ("tankfarm", 33.55, 69.12, "Severomorsk"),
    ("tankfarm", 40.80, 64.56, "Arkhangelsk"), ("tankfarm", 32.30, 67.13, "Kandalaksha"),
    ("tankfarm", 39.62, 64.48, "Severodvinsk"), ("tankfarm", 34.42, 64.98, "Kem"),
    # The early warning radar north of Olenegorsk
    ("oth", 33.91, 68.11, "Kirovsk"),
    # Mobile SAM batteries round the fleet bases and the railway
    ("msam", 33.30, 69.15, "Polyarnyy"), ("msam", 32.70, 68.95, "Murmansk"), ("msam", 31.40, 69.50, "Pechenga"),
    ("msam", 32.20, 67.25, "Kandalaksha"), ("msam", 40.30, 64.45, "Arkhangelsk"), ("msam", 33.70, 67.95, "Olenegorsk"),
    # Prison camps
    ("pow", 41.95, 64.40, "Kholmogory"), ("pow", 32.70, 67.20, "Kandalaksha"), ("pow", 31.60, 68.70, "Ristikent"),
    ("pow", 30.70, 69.30, "Zapolyarnyy"), ("pow", 34.30, 69.08, "Severomorsk"),
]

# Single objects: the Northern Fleet's submarine pens on the shore.
OBJECTS = [
    ("sub_pen", 33.40, 69.075, "Severomorsk"),
    ("sub_pen", 33.44, 69.20, "Polyarnyy"),
    ("sub_pen", 33.33, 69.27, "Gadzhiyevo"),
    ("sub_pen", 32.52, 69.39, "Zaozersk"),
    ("sub_pen", 39.78, 64.57, "Severodvinsk"),
]

# Cities of the enemy side with a SAM site at their edge.
CITY_SAMS = ["Alakurtti", "Ristikent", "Nikel", "Murmansk", "Arkhangelsk", "Kandalaksha", "Severomorsk", "Apatity"]

# No patrol boats: the fleet's frigates are ships.
BOATS = []

# Ships at sea: (library kind, lon, lat, radar id or None).
SHIPS = [
    # Typhoon missile submarines off Zapadnaya Litsa, Kola Bay, Severodvinsk and in the
    # Kandalaksha Gulf
    ("typhoon", 32.50, 69.50, None), ("typhoon", 33.70, 69.42, None), ("typhoon", 39.40, 64.85, None),
    ("typhoon", 33.30, 66.70, None),
    # The carrier Kuznetsov in the White Sea with its Krivak escort
    ("cv_kuznetsov", 38.30, 65.55, "san6"),
    ("krivak", 38.00, 65.45, "san4"), ("krivak", 38.60, 65.72, "san4"), ("krivak", 38.75, 65.40, "san4"),
    ("krivak", 37.85, 65.70, "san4"),
    # Krivak frigates along the Murman coast, in the Gorlo and the White Sea
    ("krivak", 31.50, 70.05, "san4"), ("krivak", 32.60, 70.10, "san4"), ("krivak", 34.00, 69.55, "san4"),
    ("krivak", 35.60, 69.40, "san4"), ("krivak", 37.50, 68.95, "san4"), ("krivak", 39.20, 68.45, "san4"),
    ("krivak", 41.00, 67.80, "san4"), ("krivak", 40.90, 66.45, "san4"), ("krivak", 36.50, 65.90, "san4"),
    ("krivak", 33.80, 66.55, "san4"),
    # The friendly carrier's destroyers
    ("destroyer", 27.55, 72.15, None), ("destroyer", 27.25, 72.15, None), ("destroyer", 27.40, 72.25, None),
    ("destroyer", 27.40, 72.05, None), ("destroyer", 27.60, 72.22, None),
    # Cargo ships in the ports and the shipping lanes
    ("cargo_ship", 33.62, 69.35, None), ("cargo_ship", 35.00, 69.45, None), ("cargo_ship", 37.00, 69.20, None),
    ("cargo_ship", 32.80, 66.93, None), ("cargo_ship", 39.90, 64.75, None), ("cargo_ship", 37.00, 65.50, None),
    ("cargo_ship", 36.20, 64.40, None), ("cargo_ship", 34.95, 64.98, None), ("cargo_ship", 40.20, 66.00, None),
    ("cargo_ship", 22.60, 65.45, None), ("cargo_ship", 24.30, 65.60, None), ("cargo_ship", 25.00, 65.10, None),
    ("cargo_ship", 24.50, 71.20, None), ("cargo_ship", 30.40, 69.95, None),
]

# Mountain ranges: (lon, lat, length theater km, width theater km, height m, bearing of the
# long axis in degrees from north).
MOUNTAINS = [
    # Kola: the Khibiny, the Lovozero massif, the Monche and Chuna tundras, the Lapland reserve's
    # fells, the Kandalaksha hills, the Keivy ridge
    (33.70, 67.73, 22, 18, 1200, 90), (34.72, 67.83, 15, 10, 1120, 0), (32.45, 67.75, 20, 9, 1000, 20),
    (31.80, 67.95, 16, 8, 900, 40), (33.20, 67.05, 12, 6, 650, 90), (37.50, 67.60, 30, 8, 400, 100),
    # Finnish Lapland: Saariselka, Pallas-Ounastunturi, Yllas, Pyha-Luosto, Salla
    (27.60, 68.40, 18, 10, 718, 110), (24.10, 68.05, 22, 7, 821, 10), (24.20, 67.57, 10, 6, 718, 0),
    (27.00, 67.10, 14, 6, 540, 30), 
    # Norway: the Lyngen Alps, the Kvaenangen and Oksfjord ice caps, Seiland, Finnmarksvidda,
    # the Varanger peninsula, the Porsanger and Nordkinn plateaus
    (20.15, 69.65, 20, 7, 1800, 0), (22.20, 70.05, 18, 10, 1200, 40), (23.15, 70.36, 10, 7, 1000, 70),
    (24.50, 69.30, 30, 18, 600, 60), (26.00, 69.20, 25, 15, 550, 60), (29.50, 70.45, 24, 10, 630, 80),
    (26.50, 70.55, 15, 8, 500, 30),
    # Sweden: the Kebnekaise massif at the western edge
    (18.60, 67.90, 20, 14, 2100, 0),
]

# Forest regions: (lon, lat, radius theater km, density 0..1). Taiga up to about 68.5 N, birch
# and pine in the river valleys further north, bare tundra on the coasts.
FORESTS = [
    # Arkhangelsk and the Onega peninsula
    (41.00, 64.30, 30, 0.7), (42.50, 64.60, 30, 0.7), (39.00, 64.25, 22, 0.65), (37.50, 64.35, 18, 0.6),
    (41.50, 65.30, 25, 0.5), (43.00, 65.40, 25, 0.55),
    # Karelia
    (33.50, 65.30, 30, 0.65), (32.00, 65.00, 30, 0.65), (31.50, 66.00, 25, 0.6), (33.00, 66.40, 25, 0.6),
    (32.00, 66.80, 20, 0.55),
    # The southern Kola peninsula and the Tersky coast
    (32.80, 67.00, 18, 0.5), (31.50, 67.30, 22, 0.5), (30.80, 67.80, 20, 0.45), (35.50, 66.80, 25, 0.45),
    (32.50, 68.50, 15, 0.3),
    # Finland
    (27.50, 64.80, 30, 0.65), (26.00, 65.40, 25, 0.6), (28.50, 65.80, 25, 0.6), (26.50, 66.20, 28, 0.6),
    (28.00, 66.80, 25, 0.55), (25.00, 66.80, 25, 0.55), (26.50, 67.50, 25, 0.5), (24.50, 67.40, 20, 0.45),
    (27.50, 68.30, 20, 0.4), (28.80, 67.80, 20, 0.45),
    # Sweden
    (21.00, 65.70, 25, 0.6), (22.50, 66.30, 25, 0.6), (20.50, 66.80, 22, 0.55), (21.50, 67.30, 20, 0.45),
    (20.00, 65.30, 20, 0.6), (23.00, 67.20, 18, 0.5),
    # The Pasvik valley, the Alta valley and the birch woods round Karasjok and Inari
    (29.60, 69.30, 10, 0.45), (23.40, 69.85, 8, 0.4), (25.30, 69.40, 12, 0.3), (29.50, 68.80, 15, 0.4),
]


# ---------------------------------------------------------------------------------------------
# Models of this theater's library the layouts use, and its airfield's aprons.

KINDS = {"houses": 24, "houses2": 24, "buildings": 24, "patrol_boat": 77, "oth_radar": 43, "land": 0, "sea": 18,
         "carrier": 61, "carrier_object": 72, "airstrip": 73}
APRONS = [(78, 500, -180, 180)]
LAND_COLOUR = 50

# The empty open land: a hamlet here and there.
TEMPLATES["hamlet"] = [(0, 0, "houses"), (170, -120, "houses")]
TEMPLATES["lone_house"] = [(0, 0, "houses")]
FILLER = [{}, {}, {}, {(1, 2): "hamlet"}, {(2, 1): "lone_house"}]
