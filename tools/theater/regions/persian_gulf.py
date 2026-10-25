"""Geography of the Persian Gulf theater (April 1988): the Gulf from the Shatt al-Arab to the
Strait of Hormuz and the Gulf of Oman, with south-western Iran up to Esfahan, Yazd and Kerman,
southern Iraq, Kuwait, eastern Saudi Arabia, Bahrain, Qatar, the Emirates and northern Oman.

The theater squeezes 46.4-58.35 E, 22.95-33.5 N (about 1170 by 1170 km) into the 512 km of a
theater. Coordinates here are (lon, lat) degrees. The original theater's places sit 20-40 km
off a true map in places (its Gulf was drawn wider and its Iraq narrower); the projection is
the least-squares fit to them.

Sides: the Tanker War in the days of Operation Praying Mantis (18 April 1988). Iran is the
enemy: its airbases, ports, oil terminals and offshore platforms, the IRGC's boats and its
islands (Kharg, Qeshm, Larak, Abu Musa, the Tunbs, Sirri, Kish, Lavan). Saudi Arabia and
Bahrain are the friendly side, Kuwait, Qatar, the Emirates and Oman neutral (their bases and
towns), and Iraq, at war with Iran, neutral ground too. The front is the Iran-Iraq war front
along the border: the Shatt al-Arab from Basra to the sea (Iraq retook al-Faw on 17 April
1988) and Shalamcheh are the hottest, the Hawizeh marshes and the central sector quieter.
"""

PROJECTION = dict(lon0=46.40, lat0=22.95, lat_ref=28.2, squeeze=2.29)

LIBRARY = "persian_gulf"
LAND_RUBBLE = 78

DATE = (4, 18, 1988)
GRID = ("JZ", 0, 0)
SQUADRONS = 2
PATROL_FIGHTER = "f14"

ORDER_OF_BATTLE = [
    ("an72", "none", "special"),
    ("f5", "aim7", "patrol"),
    ("f4e", "aim9_ai", "escort"),
    ("f14", "aim9_ai", "base"),
    ("f5", "aim7", "patrol"),
    ("f4e", "aim9_ai", "patrol"),
    ("f4e", "aim9_ai", "escort"),
    ("f14", "aim9_ai", "base"),
    ("f18d", "none", "friendly"),
    ("f18d", "none", "friendly"),
    ("f18", "none", "friendly"),
    ("f18", "none", "friendly"),
]

# Radar types (units.ron ids): Iran's Hawk, Rapier and Tigercat sites, the same Rapier and
# Tigercat for its mobile batteries, Seacat on its boats and frigates.
SAM_RADARS = ["hawk", "rapier", "tiger"]
MOBILE_SAM_RADARS = ["rapier", "tiger"]
BOAT_RADARS = ["seacat"]
SLAG_HEAP_TOWNS = set()

# Models this theater builds besides the water, forests and mountains (see models.py).
EXTRA_MODELS = []

# ---------------------------------------------------------------------------------------------
# Land and water: everything outside `LANDS` is sea.

LANDS = [
    # Asia and Arabia round the Gulf as one outline: the Makran coast from Pakistan west to
    # Jask, the Strait of Hormuz, Iran's Gulf coast to the Shatt al-Arab, Iraq's al-Faw,
    # Kuwait, the Saudi coast, Qatar, the Emirates, Musandam and Oman's Batinah coast to Ras
    # al-Hadd, then coarsely round Arabia's south coast.
    [
        (70.0, 40.0), (70.0, 24.5), (66.6, 25.4), (64.0, 25.3), (62.4, 25.1), (61.6, 25.2), (60.6, 25.3),
        (59.5, 25.4), (58.8, 25.55), (58.35, 25.6), (58.00, 25.62), (57.80, 25.66), (57.70, 25.62),
        (57.55, 25.73), (57.33, 25.80), (57.28, 25.95), (57.22, 26.15), (57.15, 26.35), (57.10, 26.52),
        (57.06, 26.68), (57.03, 26.82), (56.95, 26.95), (56.85, 27.05), (56.70, 27.12), (56.55, 27.14),
        (56.40, 27.16), (56.27, 27.165), (56.12, 27.13), (55.95, 27.06), (55.80, 27.02), (55.62, 26.98),
        (55.45, 26.92), (55.30, 26.83), (55.18, 26.74), (55.05, 26.65), (54.95, 26.57), (54.85, 26.49),
        (54.70, 26.50), (54.55, 26.58), (54.40, 26.65), (54.25, 26.72), (54.08, 26.76), (53.90, 26.80),
        (53.75, 26.85), (53.60, 26.90), (53.45, 26.96), (53.30, 27.02), (53.15, 27.08), (53.00, 27.15),
        (52.85, 27.27), (52.72, 27.38), (52.60, 27.47), (52.45, 27.58), (52.30, 27.67), (52.15, 27.76),
        (52.00, 27.83), (51.85, 27.85), (51.65, 27.92), (51.48, 28.03), (51.35, 28.15), (51.28, 28.30),
        (51.20, 28.45), (51.10, 28.60), (51.00, 28.72),
        # Bushehr on its peninsula
        (50.92, 28.80), (50.85, 28.87), (50.80, 28.93), (50.79, 28.98), (50.83, 29.00), (50.88, 28.97),
        (50.90, 28.92), (50.96, 28.93), (51.00, 29.00), (50.97, 29.08),
        # North to Bandar-e Rig, Ganaveh and Deylam, Khor Musa up to Bandar Imam Khomeini
        (50.90, 29.13), (50.80, 29.22), (50.70, 29.35), (50.62, 29.47), (50.55, 29.56), (50.45, 29.65),
        (50.35, 29.78), (50.25, 29.92), (50.15, 30.03), (50.05, 30.08), (49.90, 30.12), (49.75, 30.13),
        (49.62, 30.06), (49.50, 30.02), (49.38, 30.10), (49.25, 30.23), (49.15, 30.33), (49.10, 30.38),
        (49.03, 30.35), (49.04, 30.20), (49.10, 30.06), (49.05, 30.00), (48.95, 30.02), (48.80, 30.03),
        (48.68, 30.01), (48.60, 29.95),
        # Across the Shatt's mouth: al-Faw, Khor Abdullah, Umm Qasr, Kuwait Bay
        (48.55, 29.89), (48.45, 29.91), (48.32, 29.96), (48.18, 30.01), (48.05, 30.05), (47.96, 30.03),
        (47.93, 29.85), (47.92, 29.70), (47.97, 29.58), (48.02, 29.50), (47.90, 29.47), (47.75, 29.45),
        (47.65, 29.40), (47.70, 29.34), (47.85, 29.36), (47.98, 29.37), (48.05, 29.38), (48.10, 29.33),
        # Kuwait's coast, Khafji, Safaniya, Jubail, Ras Tanura, Dammam, the Gulf of Bahrain
        (48.08, 29.20), (48.12, 29.08), (48.15, 28.95), (48.25, 28.80), (48.38, 28.62), (48.45, 28.50),
        (48.53, 28.40), (48.62, 28.25), (48.65, 28.12), (48.75, 28.02), (48.80, 27.95), (48.72, 27.87),
        (48.88, 27.70), (49.05, 27.55), (49.25, 27.40), (49.45, 27.18), (49.60, 27.05), (49.67, 26.95),
        (49.80, 26.85), (49.95, 26.76), (50.05, 26.74), (50.12, 26.70), (50.17, 26.68), (50.15, 26.62),
        (50.08, 26.60), (50.04, 26.60), (50.03, 26.52), (50.10, 26.44), (50.15, 26.40), (50.21, 26.30),
        (50.22, 26.18), (50.18, 26.05), (50.10, 25.90), (50.05, 25.75), (50.22, 25.65),
        # The Gulf of Salwa, Qatar round its north tip, Doha, Khor al-Udeid
        (50.35, 25.45), (50.45, 25.20), (50.55, 25.00), (50.65, 24.85), (50.75, 24.75), (50.85, 24.72),
        (50.88, 24.85), (50.85, 24.95), (50.80, 25.20), (50.76, 25.42), (50.82, 25.55), (50.90, 25.68),
        (50.98, 25.85), (51.05, 26.00), (51.15, 26.12), (51.22, 26.16), (51.35, 26.07), (51.45, 25.97),
        (51.55, 25.90), (51.57, 25.75), (51.55, 25.60), (51.55, 25.45), (51.56, 25.35), (51.62, 25.28),
        (51.64, 25.18), (51.60, 25.02), (51.55, 24.90), (51.48, 24.75), (51.35, 24.62), (51.40, 24.50),
        # The Emirates: Sila, Jebel Dhanna, Ruwais, Mirfa, Abu Dhabi, Jebel Ali, Dubai, Sharjah,
        # Ras al-Khaimah, Musandam, the east coast past Fujairah
        (51.55, 24.35), (51.65, 24.20), (51.80, 24.05), (52.00, 24.00), (52.20, 24.05), (52.45, 24.10),
        (52.60, 24.20), (52.75, 24.15), (53.00, 24.10), (53.30, 24.12), (53.55, 24.10), (53.80, 24.08),
        (54.00, 24.15), (54.20, 24.28), (54.35, 24.45), (54.45, 24.52), (54.60, 24.62), (54.80, 24.80),
        (55.03, 25.00), (55.20, 25.15), (55.32, 25.27), (55.40, 25.36), (55.50, 25.45), (55.60, 25.58),
        (55.75, 25.68), (55.90, 25.75), (56.00, 25.90), (56.05, 26.00), (56.10, 26.12), (56.18, 26.22),
        (56.30, 26.33), (56.40, 26.38), (56.45, 26.28), (56.42, 26.15), (56.38, 25.95), (56.35, 25.80),
        (56.32, 25.62), (56.35, 25.45), (56.36, 25.30), (56.37, 25.10), (56.40, 24.95),
        # Oman: Shinas, Sohar, the Batinah, Seeb, Muscat, Quriyat, Sur, Ras al-Hadd
        (56.50, 24.75), (56.65, 24.55), (56.80, 24.35), (57.00, 24.15), (57.25, 23.98), (57.55, 23.82),
        (57.85, 23.71), (58.15, 23.68), (58.40, 23.64), (58.60, 23.58), (58.80, 23.45), (59.00, 23.20),
        (59.30, 22.85), (59.55, 22.60), (59.80, 22.53),
        # Arabia's south coast, far outside the theater
        (59.6, 22.1), (58.8, 21.0), (58.0, 19.8), (57.7, 18.9), (56.8, 17.9), (55.0, 17.0), (53.0, 16.5),
        (52.0, 15.5), (48.0, 14.0), (43.5, 12.7), (40.0, 12.0), (40.0, 40.0),
    ],
    # Kuwait's Bubiyan and Failaka
    [(48.02, 29.92), (48.15, 29.93), (48.30, 29.88), (48.38, 29.78), (48.35, 29.65), (48.22, 29.60),
     (48.08, 29.62), (47.99, 29.75)],
    [(48.27, 29.45), (48.40, 29.47), (48.43, 29.42), (48.33, 29.40)],
    # Bahrain with Muharraq
    [(50.45, 26.24), (50.55, 26.25), (50.60, 26.29), (50.68, 26.29), (50.67, 26.20), (50.62, 26.12),
     (50.62, 25.95), (50.58, 25.82), (50.53, 25.80), (50.47, 25.95), (50.44, 26.10)],
    # Kharg, Iran's oil terminal island
    [(50.28, 29.31), (50.33, 29.30), (50.35, 29.25), (50.33, 29.19), (50.29, 29.19), (50.27, 29.24)],
    # Qeshm, Hormuz, Larak and Hengam in the Strait
    [(55.27, 26.62), (55.45, 26.60), (55.65, 26.66), (55.85, 26.73), (56.05, 26.82), (56.20, 26.88),
     (56.30, 26.94), (56.25, 26.99), (56.10, 27.00), (55.95, 26.98), (55.75, 26.92), (55.55, 26.82),
     (55.38, 26.73)],
    [(56.43, 27.08), (56.48, 27.10), (56.51, 27.05), (56.46, 27.02)],
    [(56.32, 26.87), (56.38, 26.89), (56.40, 26.84), (56.34, 26.82)],
    [(55.86, 26.66), (55.91, 26.67), (55.92, 26.62), (55.87, 26.61)],
    # Kish, Lavan, Sirri, Abu Musa and the Greater Tunb
    [(53.92, 26.55), (54.02, 26.58), (54.06, 26.52), (53.98, 26.48), (53.91, 26.51)],
    [(53.20, 26.80), (53.30, 26.83), (53.42, 26.83), (53.41, 26.78), (53.25, 26.77)],
    [(54.51, 25.93), (54.56, 25.94), (54.58, 25.89), (54.52, 25.88)],
    [(55.00, 25.90), (55.05, 25.91), (55.07, 25.86), (55.01, 25.85)],
    [(55.27, 26.28), (55.32, 26.28), (55.33, 26.25), (55.28, 26.24)],
]

# Marshes, lagoons and salt lakes.
LAKES = [
    # The Hawizeh marshes (Hoor al-Azim) across the border, the Central and Hammar marshes
    [(47.60, 31.75), (47.75, 31.80), (47.82, 31.55), (47.80, 31.25), (47.70, 31.15), (47.60, 31.30),
     (47.55, 31.55)],
    [(46.90, 31.30), (47.30, 31.30), (47.35, 31.08), (47.10, 31.05), (46.85, 31.15)],
    [(46.55, 30.95), (46.90, 31.00), (47.30, 30.95), (47.55, 30.85), (47.45, 30.75), (47.05, 30.80),
     (46.65, 30.83)],
    # Shadegan lagoon
    [(48.55, 30.55), (48.80, 30.62), (48.98, 30.48), (48.90, 30.30), (48.65, 30.35)],
    # Maharlu by Shiraz, Bakhtegan and Tashk, Gavkhouni below Esfahan
    [(52.68, 29.48), (52.82, 29.47), (52.92, 29.38), (52.85, 29.36), (52.70, 29.42)],
    [(53.55, 29.70), (53.85, 29.62), (54.05, 29.45), (54.20, 29.30), (54.05, 29.22), (53.80, 29.38),
     (53.55, 29.55)],
    [(52.80, 32.20), (52.95, 32.25), (53.00, 32.10), (52.85, 32.05)],
]

# Rivers: polylines of (lon, lat, width in theater km).
RIVERS = [
    # Shatt al-Arab from Qurnah past Basra, Khorramshahr and Abadan to the sea
    [(47.44, 30.98, 0.2), (47.55, 30.85, 0.22), (47.70, 30.65, 0.25), (47.84, 30.52, 0.25), (48.00, 30.47, 0.28),
     (48.18, 30.42, 0.3), (48.26, 30.36, 0.3), (48.33, 30.26, 0.35), (48.40, 30.15, 0.4), (48.50, 30.02, 0.6),
     (48.58, 29.92, 1.0)],
    # Tigris from Kut past Ali al-Gharbi and Amarah to Qurnah
    [(45.82, 32.48, 0.15), (46.25, 32.50, 0.15), (46.68, 32.45, 0.15), (46.95, 32.10, 0.15), (47.18, 31.88, 0.15),
     (47.27, 31.52, 0.14), (47.38, 31.20, 0.14), (47.44, 30.98, 0.15)],
    # Euphrates from Samawah past Nasiriyah into the Hammar marshes
    [(45.29, 31.31, 0.14), (45.70, 31.20, 0.14), (46.26, 31.01, 0.14), (46.58, 30.94, 0.14)],
    # Karun from the Zagros past Shushtar and Ahvaz to Khorramshahr
    [(49.45, 32.10, 0.1), (49.20, 32.15, 0.1), (48.95, 32.20, 0.11), (48.83, 32.10, 0.11), (48.86, 31.95, 0.12),
     (48.89, 31.73, 0.13), (48.78, 31.50, 0.13), (48.66, 31.33, 0.14), (48.58, 31.15, 0.14), (48.45, 30.95, 0.14),
     (48.35, 30.75, 0.14), (48.25, 30.55, 0.14), (48.20, 30.43, 0.15)],
    # Dez past Dezful to Band-e Qir
    [(48.45, 32.60, 0.08), (48.43, 32.38, 0.09), (48.55, 32.15, 0.09), (48.72, 31.90, 0.1), (48.89, 31.73, 0.1)],
    # Karkheh past Shush and Hamidiyeh into the Hawizeh marshes
    [(48.12, 32.75, 0.08), (48.21, 32.40, 0.08), (48.21, 32.19, 0.08), (48.30, 31.85, 0.08), (48.42, 31.52, 0.08),
     (48.30, 31.45, 0.07), (48.00, 31.45, 0.07), (47.80, 31.45, 0.07)],
    # Bahmanshir from Khorramshahr east of Abadan to the sea
    [(48.21, 30.43, 0.1), (48.33, 30.38, 0.1), (48.42, 30.28, 0.1), (48.55, 30.12, 0.12), (48.68, 30.03, 0.2)],
    # Zohreh past Hendijan; the Mand south of Bushehr
    [(50.45, 30.55, 0.06), (50.10, 30.45, 0.06), (49.85, 30.33, 0.07), (49.68, 30.16, 0.08)],
    [(52.40, 29.10, 0.05), (52.05, 28.65, 0.06), (51.70, 28.35, 0.06), (51.40, 28.12, 0.07)],
    # Zayandeh Rud through Esfahan to the Gavkhouni; the Kor past Marvdasht to Bakhtegan
    [(50.90, 32.60, 0.06), (51.20, 32.55, 0.06), (51.45, 32.62, 0.06), (51.67, 32.62, 0.07), (51.90, 32.55, 0.06),
     (52.30, 32.40, 0.05), (52.80, 32.18, 0.05)],
    [(52.30, 30.30, 0.05), (52.80, 29.95, 0.05), (53.25, 29.85, 0.05), (53.55, 29.68, 0.05)],
]

# ---------------------------------------------------------------------------------------------
# Sides

# The Iran-Iraq war front along the border, north to south: Mehran and Dehloran, Fakkeh, the
# Hawizeh marshes, Shalamcheh before Basra, then the Shatt al-Arab to the sea at al-Faw.
FRONT = [
    (45.80, 33.60, 0.4), (46.05, 33.30, 0.5), (46.20, 33.10, 0.6), (46.50, 32.90, 0.5), (46.85, 32.75, 0.45),
    (47.15, 32.48, 0.45), (47.40, 32.30, 0.5), (47.55, 32.10, 0.55), (47.70, 31.90, 0.6), (47.70, 31.60, 0.75),
    (47.68, 31.30, 0.8), (47.70, 31.00, 0.85), (47.86, 30.98, 0.9), (47.97, 30.80, 1.0), (48.01, 30.55, 1.0),
    (48.03, 30.46, 1.0), (48.18, 30.42, 0.95), (48.26, 30.36, 0.9), (48.33, 30.26, 0.9), (48.40, 30.15, 0.95),
    (48.50, 30.02, 1.0), (48.58, 29.92, 1.0),
]

# The friendly side: Arabia and Iraq west of the front, across the Gulf a line south of Iran's
# islands and offshore fields (Kharg, Farsi, Rostam, Sassan, Sirri, Abu Musa, the Tunbs,
# Qeshm), out through the Strait of Hormuz and down the Gulf of Oman.
BLUE = (
    [(40.0, 10.0), (40.0, 40.0), (45.5, 40.0), (45.5, 34.5)]
    + [(lon, lat) for lon, lat, _ in FRONT]
    + [(48.62, 29.88), (48.85, 29.60), (49.15, 29.10), (49.55, 28.45), (49.95, 27.95), (50.17, 27.88),
       (50.60, 27.45), (51.20, 27.00), (51.90, 26.40), (52.50, 25.95), (53.10, 25.55), (53.60, 25.58),
       (54.10, 25.65), (54.40, 25.75), (54.80, 25.78), (55.20, 25.75), (55.60, 25.95), (56.00, 26.20),
       (56.25, 26.45), (56.55, 26.50), (56.80, 25.80), (57.10, 25.35), (57.60, 24.95), (58.50, 24.50),
       (59.50, 24.10), (61.0, 23.8), (62.0, 23.6), (62.0, 10.0)]
)

# Neutral places: Iraq's towns, the neutral Gulf states (Kuwait, Qatar, the Emirates, Oman) and
# the two Saudi border towns the original theater made neutral.
NEUTRAL = {
    "Basra", "Qurnah", "Al Ammarah", "Nasiriyah", "Kut", "Diwaniyah", "Rifai", "Zubair", "Fao",
        "Kuwait City", "Jahra", "Ahmadi",
    "Doha",
    "Abu Dhabi", "Dubai", "Sharjah", "Ras al Khaimah", "Fujairah", "Al Ain", "Ruwais", "Tarif",
    "Muscat", "Seeb", "Sohar", "Khasab",
    "Qaisumah", "Al Mishab",
}

# ---------------------------------------------------------------------------------------------
# Places

# Towns: (name, lon, lat, kind 0 village / 1 town / 2 city, radius in theater metres, ruined
# share, in the world name table).
TOWNS = [
    # Iran: Khuzestan and the war zone
    ("Ahvaz", 48.69, 31.31, 2, 2600, 0.05, True),
    ("Abadan", 48.30, 30.34, 2, 2200, 0.5, True),
    ("Khorramshahr", 48.16, 30.47, 1, 1400, 0.85, True),
    ("Dezful", 48.40, 32.38, 2, 1800, 0.1, True),
    ("Andimeshk", 48.35, 32.46, 1, 1100, 0.05, False),
    ("Shush", 48.25, 32.19, 1, 900, 0.05, False),
    ("Shushtar", 48.84, 32.04, 1, 1000, 0.0, False),
    ("Masjed Soleyman", 49.30, 31.94, 1, 1200, 0.0, True),
    ("Susangerd", 48.18, 31.56, 1, 900, 0.3, False),
    ("Hoveyzeh", 48.07, 31.46, 0, 700, 0.7, False),
    ("Hamidiyeh", 48.43, 31.48, 0, 600, 0.1, False),
    ("Bandar Khomeini", 49.08, 30.43, 1, 1000, 0.0, True),
    ("Bandar Mahshahr", 49.20, 30.56, 1, 1100, 0.0, False),
    ("Ramhormoz", 49.60, 31.28, 1, 900, 0.0, False),
    ("Behbahan", 50.24, 30.60, 1, 1200, 0.0, True),
    ("Izeh", 49.87, 31.83, 1, 900, 0.0, False),
    ("Gachsaran", 50.80, 30.36, 1, 900, 0.0, False),
    # Iran: Bushehr and Fars
    ("Bandar-e Rig", 50.66, 29.47, 0, 600, 0.0, True),
    ("Bandar Ganaveh", 50.54, 29.60, 0, 800, 0.0, False),
    ("Bushehr", 50.83, 28.97, 2, 1800, 0.0, True),
    ("Borazjan", 51.21, 29.27, 1, 900, 0.0, False),
    ("Kazerun", 51.65, 29.62, 1, 1100, 0.0, False),
    ("Shiraz", 52.53, 29.61, 2, 3000, 0.0, True),
    ("Marvdasht", 52.80, 29.87, 1, 1000, 0.0, False),
    ("Fasa", 53.65, 28.94, 1, 900, 0.0, False),
    ("Jahrom", 53.56, 28.50, 1, 1000, 0.0, False),
    ("Darab", 54.55, 28.75, 1, 900, 0.0, False),
    ("Madevan", 54.26, 28.75, 0, 600, 0.0, True),
    ("Lar", 54.33, 27.68, 1, 900, 0.0, False),
    ("Kangan", 52.06, 27.84, 0, 700, 0.0, False),
    # Iran: Hormozgan and the islands
    ("Furg", 55.17, 28.32, 0, 600, 0.0, True),
    ("Fin", 55.89, 27.63, 0, 600, 0.0, True),
    ("Latian", 55.75, 27.23, 0, 600, 0.0, True),
    ("Bandar-e Lengeh", 54.88, 26.56, 1, 900, 0.0, True),
    ("Bandar Khamir", 55.59, 26.99, 0, 600, 0.0, False),
    ("Bandar Abbas", 56.28, 27.20, 2, 2200, 0.0, True),
    ("Qeshm", 56.24, 26.95, 0, 700, 0.0, False),
    ("Minab", 57.08, 27.15, 1, 900, 0.0, False),
    ("Jask", 57.78, 25.67, 0, 700, 0.0, False),
    # Iran: the plateau
    ("Esfahan", 51.67, 32.65, 2, 3200, 0.0, True),
    ("Yazd", 54.37, 31.90, 2, 2000, 0.0, True),
    ("Kerman", 57.08, 30.28, 2, 2000, 0.0, True),
    ("Rafsanjan", 55.99, 30.40, 1, 1000, 0.0, False),
    ("Sirjan", 55.68, 29.45, 1, 1000, 0.0, False),
    # Iraq
    ("Basra", 47.81, 30.51, 2, 2600, 0.05, True),
    ("Zubair", 47.70, 30.39, 1, 1000, 0.0, False),
    ("Fao", 48.47, 29.97, 0, 700, 0.9, False),
    ("Qurnah", 47.40, 31.02, 0, 700, 0.0, True),
    ("Al Ammarah", 47.15, 31.84, 1, 1300, 0.0, True),
    ("Nasiriyah", 46.26, 31.05, 2, 1600, 0.0, True),
    ("Rifai", 46.11, 31.72, 0, 700, 0.0, True),
    ("Kut", 45.82, 32.51, 1, 1300, 0.0, True),
    ("Diwaniyah", 44.93, 31.99, 1, 1300, 0.0, True),
    # Kuwait
    ("Kuwait City", 47.97, 29.34, 2, 2600, 0.0, True),
    ("Jahra", 47.66, 29.33, 1, 900, 0.0, False),
    ("Ahmadi", 48.08, 29.08, 1, 1000, 0.0, False),
    # Saudi Arabia
    ("Qaisumah", 46.13, 28.31, 0, 600, 0.0, True),
    ("Al Mishab", 48.60, 28.11, 0, 600, 0.0, True),
    ("Khafji", 48.47, 28.42, 0, 800, 0.0, False),
    ("Jubail", 49.60, 26.98, 1, 1000, 0.0, False),
    ("Ras Tanura", 50.13, 26.65, 0, 700, 0.0, False),
    ("Qatif", 49.99, 26.55, 1, 900, 0.0, False),
    ("Dammam", 50.06, 26.42, 2, 2000, 0.0, False),
    ("Khobar", 50.19, 26.28, 1, 1200, 0.0, False),
    ("Hofuf", 49.59, 25.38, 2, 1800, 0.0, False),
    ("Riyadh", 46.72, 24.71, 2, 3200, 0.0, False),
    # Bahrain and Qatar
    ("Manama", 50.58, 26.22, 2, 1800, 0.0, True),
    ("Doha", 51.50, 25.28, 2, 2200, 0.0, True),
    # The Emirates
    ("Abu Dhabi", 54.40, 24.44, 2, 2400, 0.0, True),
    ("Dubai", 55.33, 25.24, 2, 2400, 0.0, True),
    ("Sharjah", 55.43, 25.33, 2, 1800, 0.0, False),
    ("Ras al Khaimah", 55.97, 25.78, 1, 1000, 0.0, False),
    ("Fujairah", 56.33, 25.12, 1, 900, 0.0, False),
    ("Al Ain", 55.76, 24.21, 2, 1600, 0.0, False),
    ("Ruwais", 52.73, 24.08, 0, 600, 0.0, False),
    ("Tarif", 53.76, 24.02, 0, 500, 0.0, False),
    # Oman
    ("Khasab", 56.25, 26.18, 0, 600, 0.0, False),
    ("Sohar", 56.72, 24.34, 1, 1000, 0.0, False),
    ("Seeb", 58.19, 23.64, 1, 900, 0.0, False),
    ("Muscat", 58.50, 23.56, 2, 2000, 0.0, True),
]

# Airbases: (name, lon, lat, side: True friendly, False enemy, None neutral). The runway runs
# north-south.
AIRBASES = [
    # Iran: Bandar Abbas (TFB 9), Lengeh, Bushehr (TFB 6), Shiraz (TFB 7), Mahshahr for Bandar
    # Khomeini, Kerman, Masjed Soleyman, Yazd, Esfahan Khatami (TFB 8)
    ("Bandar Abbas", 56.378, 27.218, False),
    ("Bandar-e Lengeh", 54.825, 26.532, False),
    ("Bushehr", 50.835, 28.945, False),
    ("Shiraz", 52.590, 29.540, False),
    ("Bandar Khomeini", 49.152, 30.556, False),
    ("Kerman", 56.951, 30.274, False),
    ("Masjed Soleyman", 49.268, 31.998, False),
    ("Yazd", 54.277, 31.905, False),
    ("Esfahan", 51.692, 32.567, False),
    # Saudi Arabia and Bahrain: Safaniya's strip, Al Kharj (by As Salamiyah), Al Ahsa, Dhahran,
    # Muharraq
    ("Ras Saffaniyah", 48.70, 27.97, True),
    ("As Salamiyah", 47.58, 24.06, True),
    ("Hofuf", 49.485, 25.285, True),
    ("Dhahran", 50.152, 26.265, True),
    ("Bahrain", 50.634, 26.271, True),
    # Neutral: Doha, Kuwait International, Dubai, Ruwais, Tarif, Al Dhafra, Seeb
    ("Qatar", 51.565, 25.261, None),
    ("Kuwait City", 47.969, 29.227, None),
    ("Dubai", 55.364, 25.253, None),
    ("Ruweiss", 52.70, 24.08, None),
    ("Tarif", 53.75, 23.98, None),
    ("Abu Dhabi", 54.548, 24.248, None),
    ("Muscat", 58.284, 23.593, None),
]

# Iran's small strips: Izeh in the Zagros and Rayen south of Kerman.
AIRSTRIPS = [
    ("Izeh", 49.90, 31.86, False),
    ("Rayen", 57.45, 29.58, False),
]

# The US carrier group in the Gulf of Oman, off Sohar.
CARRIER = ("USS Nimitz", 57.40, 24.75)

# The theater's own layouts. The standard ones are redefined with the kinds this theater has
# (an army headquarters, depots and fuel, no houses or mobile SAMs); oil fields with their
# wells and gathering tanks; the oil terminal's tank farms (Kharg); a petrochemical complex.
TEMPLATES = {
    "hq": [(0, 0, "army_hq"), (-380, 0, "depot"), (380, -250, "depot"), (-450, 500, "tank_farm")],
    "depot": [(-300, 0, "depot"), (0, 0, "depot"), (300, 0, "depot"), (-150, -350, "depot"), (150, -350, "depot"),
              (0, 450, "tank_farm")],
    "tankfarm": [(-150, -150, "tank_farm"), (150, -150, "tank_farm"), (-150, 150, "tank_farm"), (150, 150, "tank_farm"),
                 (500, 0, "tank_farm"), (-550, -400, "depot")],
    "ss": [(0, 0, "ss_missile"), (300, 250, "ss_missile"), (-300, 250, "ss_missile"), (0, -500, "depot")],
    "oil_field": [(0, 0, "tank_farm"), (-550, -450, "oil_well"), (350, -600, "oil_well"), (-700, 300, "oil_well"),
                  (250, 250, "oil_well"), (650, 500, "oil_well"), (-150, 750, "oil_well"), (750, -150, "oil_well")],
    "oil_terminal": [(-300, -250, "tank_farm"), (0, -250, "tank_farm"), (300, -250, "tank_farm"),
                     (-300, 100, "tank_farm"), (0, 100, "tank_farm"), (300, 100, "tank_farm"), (0, 500, "depot")],
    "petrochem": [(0, 0, "refinery"), (-400, -300, "tank_farm"), (400, -300, "tank_farm"), (0, 500, "depot")],
}

# Enemy target sites: (template, lon, lat, name).
SITES = [
    # Headquarters: the Khuzestan front, the navy at Bandar Abbas and Bushehr, the garrisons
    ("hq", 48.72, 31.28, "Ahvaz"), ("hq", 48.45, 32.33, "Dezful"), ("hq", 48.33, 32.49, "Andimeshk"),
    ("hq", 48.36, 30.45, "Abadan"), ("hq", 48.22, 31.62, "Susangerd"), ("hq", 49.62, 31.24, "Ramhormoz"),
    ("hq", 56.20, 27.24, "Bandar Abbas"), ("hq", 51.08, 28.95, "Bushehr"), ("hq", 52.48, 29.66, "Shiraz"),
    ("hq", 51.75, 32.60, "Esfahan"), ("hq", 57.12, 30.24, "Kerman"), ("hq", 54.98, 26.62, "Bandar-e Lengeh"),
    # Surface-to-surface missiles: Scuds for the war of the cities behind the front, Silkworms
    # covering the Strait of Hormuz
    ("ss", 48.30, 32.26, "Dezful"), ("ss", 48.32, 32.12, "Shush"), ("ss", 49.38, 31.88, "Masjed Soleyman"),
    ("ss", 48.88, 31.45, "Ahvaz"), ("ss", 50.30, 30.66, "Behbahan"), ("ss", 56.12, 26.90, "Qeshm"),
    ("ss", 57.42, 25.88, "Kuh-e Mobarak"), ("ss", 55.72, 27.06, "Bandar Khamir"), ("ss", 56.08, 27.24, "Bandar Abbas"),
    # Depots behind the front and at the ports
    ("depot", 48.38, 32.42, "Andimeshk"), ("depot", 48.60, 31.38, "Ahvaz"), ("depot", 48.80, 32.00, "Shushtar"),
    ("depot", 49.02, 30.50, "Bandar Khomeini"), ("depot", 49.68, 30.80, "Omidiyeh"), ("depot", 49.55, 31.32, "Ramhormoz"),
    ("depot", 48.50, 31.45, "Hamidiyeh"), ("depot", 56.34, 27.24, "Bandar Abbas"), ("depot", 51.12, 29.03, "Bushehr"),
    ("depot", 50.28, 30.56, "Behbahan"),
    # Fuel
    ("tankfarm", 48.75, 31.20, "Ahvaz"), ("tankfarm", 49.24, 30.62, "Bandar Mahshahr"),
    ("tankfarm", 56.22, 27.22, "Bandar Abbas"), ("tankfarm", 52.66, 29.70, "Shiraz"), ("tankfarm", 51.60, 32.72, "Esfahan"),
    ("tankfarm", 57.00, 30.32, "Kerman"), ("tankfarm", 54.42, 31.86, "Yazd"),
    # Oil fields: Ahvaz, Marun, Aghajari, Gachsaran, Haft Kel, Bibi Hakimeh
    ("oil_field", 48.85, 31.22, "Ahvaz"), ("oil_field", 49.25, 31.05, "Marun"), ("oil_field", 49.88, 30.72, "Aghajari"),
    ("oil_field", 50.86, 30.30, "Gachsaran"), ("oil_field", 49.53, 31.45, "Haft Kel"),
    ("oil_field", 50.55, 30.15, "Bibi Hakimeh"),
    # Oil terminals: Kharg Island, the Ganaveh pumping station, Lavan
    ("oil_terminal", 50.31, 29.25, "Kharg Island"), ("oil_terminal", 50.58, 29.62, "Bandar Ganaveh"),
    ("oil_terminal", 53.33, 26.80, "Lavan"),
    # The Bandar Imam petrochemical complex
    ("petrochem", 49.12, 30.48, "Bandar Khomeini"),
    # Bridges: the Karun at Ahvaz, Shushtar and Khorramshahr, the Dez at Dezful, the Karkheh at
    # Hamidiyeh, the Bahmanshir at Abadan, the Zayandeh Rud at Esfahan
    ("bridge", 48.67, 31.32, "Ahvaz"), ("bridge", 48.85, 32.03, "Shushtar"), ("bridge", 48.22, 30.48, "Khorramshahr"),
    ("bridge", 48.43, 32.38, "Dezful"), ("bridge", 48.42, 31.52, "Hamidiyeh"), ("bridge", 48.33, 30.38, "Abadan"),
    ("bridge", 51.67, 32.62, "Esfahan"),
    # The SAM site on Kharg Island
    ("sam", 50.31, 29.24, "Kharg Island"),
]

# Single objects: the offshore platforms in their real fields (Iran's Nowruz, Soroosh, Dorood,
# Abuzar, Forozan, Rostam, Sassan and Sirri, Saudi Safaniya, Zuluf, Berri and Abu Safah, Qatar's
# Idd el-Shargi and Bul Hanine, Abu Dhabi's Umm Shaif and Zakum, Dubai's Fateh), the
# refineries, the Bushehr nuclear plant (unfinished, bombed by Iraq).
OBJECTS = [
    ("oil_rig", 49.45, 29.48, "Nowruz"), ("oil_rig", 49.52, 29.43, "Nowruz"),
    ("oil_rig", 49.48, 29.02, "Soroosh"), ("oil_rig", 50.22, 29.33, "Dorood"),
    ("oil_rig", 50.03, 29.07, "Abuzar"), ("oil_rig", 49.72, 28.05, "Forozan"),
    ("oil_rig", 52.78, 26.22, "Rostam"), ("oil_rig", 52.85, 26.18, "Rostam"),
    ("oil_rig", 53.12, 25.66, "Sassan"), ("oil_rig", 53.18, 25.68, "Sassan"),
    ("oil_rig", 54.45, 25.98, "Sirri"), ("oil_rig", 54.40, 26.01, "Sirri"),
    ("oil_rig", 48.95, 28.15, "Safaniya"), ("oil_rig", 49.05, 28.05, "Safaniya"), ("oil_rig", 48.92, 28.27, "Safaniya"),
    ("oil_rig", 49.45, 28.40, "Zuluf"), ("oil_rig", 49.85, 27.15, "Berri"), ("oil_rig", 50.50, 26.80, "Abu Safah"),
    ("oil_rig", 52.15, 25.40, "Idd el Shargi"), ("oil_rig", 52.70, 25.45, "Bul Hanine"),
    ("oil_rig", 53.25, 25.20, "Umm Shaif"), ("oil_rig", 53.70, 24.85, "Zakum"), ("oil_rig", 54.42, 25.58, "Fateh"),
    ("refinery", 48.28, 30.36, "Abadan"), ("refinery", 52.70, 29.62, "Shiraz"),
    ("refinery", 51.58, 32.78, "Esfahan"), ("refinery", 50.13, 26.63, "Ras Tanura"),
    ("nuclear_plant", 50.91, 28.84, "Bushehr"),
]

# Iranian towns with a SAM site at their edge (Ahvaz and Dezful as in the original theater).
CITY_SAMS = ["Ahvaz", "Dezful", "Andimeshk", "Shushtar", "Behbahan", "Bandar Mahshahr", "Kazerun", "Minab", "Sirjan",
             "Ramhormoz"]

# IRGC and navy patrol boats: Hormuz, Khor Musa, the northern Gulf, off the Saudi coast, off
# Kangan, south of Lavan, the Gulf of Oman, south of Bushehr.
BOATS = [(55.85, 26.40), (49.00, 29.85), (49.64, 29.31), (49.80, 28.35), (51.88, 27.34), (53.38, 26.51),
         (57.00, 25.90), (50.58, 28.49)]

# Ships: Iran's frigates Sahand and Sabalan (armed, Seacat) off Qeshm; the carrier's escorts and
# the Aegis cruisers; tankers in the shipping lanes, at Kharg, Ras Tanura, Larak and Fujairah.
SHIPS = [
    ("destroyer", 56.10, 26.62, "seacat"), ("destroyer", 55.70, 26.50, "seacat"),
    ("destroyer", 57.30, 24.75, None), ("destroyer", 57.40, 24.68, None), ("destroyer", 57.50, 24.75, None),
    ("destroyer", 57.42, 24.84, None), ("destroyer", 57.32, 24.82, None),
    ("aegis_cruiser", 57.48, 24.82, None), ("aegis_cruiser", 55.00, 25.55, None), ("aegis_cruiser", 50.95, 26.45, None),
    ("oil_tanker", 50.22, 29.18, None), ("oil_tanker", 50.18, 29.27, None), ("oil_tanker", 49.30, 29.05, None),
    ("oil_tanker", 50.35, 26.82, None), ("oil_tanker", 50.30, 27.60, None), ("oil_tanker", 51.20, 26.95, None),
    ("oil_tanker", 52.20, 26.40, None), ("oil_tanker", 53.60, 26.05, None), ("oil_tanker", 54.70, 26.10, None),
    ("oil_tanker", 56.40, 26.75, None), ("oil_tanker", 56.62, 26.30, None), ("oil_tanker", 56.55, 25.20, None),
    ("oil_tanker", 57.60, 25.10, None), ("oil_tanker", 48.75, 29.30, None),
]

# Mountain ranges: (lon, lat, length theater km, width theater km, height m, bearing of the
# long axis in degrees from north).
MOUNTAINS = [
    # The Zagros from Kabir Kuh and Lorestan down past Zard Kuh, Dena and the Fars ranges
    (47.30, 33.15, 30, 10, 2700, 125), (49.10, 33.20, 30, 14, 3800, 130), (50.10, 32.35, 32, 14, 4200, 135),
    (50.40, 31.85, 30, 14, 3500, 135), (50.60, 31.15, 26, 10, 2800, 130), (51.45, 30.90, 32, 14, 4400, 135),
    (51.85, 29.95, 30, 12, 2800, 135), (52.70, 30.35, 30, 12, 3200, 140), (51.45, 28.70, 28, 10, 2000, 135),
    (53.20, 28.80, 30, 12, 2500, 115), (54.50, 27.95, 32, 12, 2400, 100), (56.50, 27.55, 26, 10, 2300, 110),
    # Central Iran: Karkas, Shir Kuh, Kuh-e Hezar, Jebal Barez
    (51.80, 33.35, 28, 10, 3800, 135), (54.05, 31.55, 26, 12, 4000, 140), (57.25, 29.50, 32, 14, 4400, 140),
    (58.10, 28.95, 28, 12, 3700, 140),
    # The Hajar: Musandam, the Emirates' east coast, Western Hajar, Jebel Akhdar
    (56.70, 23.95, 30, 10, 2000, 125),
    (57.40, 23.15, 30, 12, 3000, 105),
]

# Forest regions: (lon, lat, radius theater km, density 0..1): the date palms of the Shatt
# al-Arab, Minab and the Saudi oases, the Zagros oak woods.
FORESTS = [
    (48.05, 30.50, 10, 0.5), (47.78, 30.62, 6, 0.4), (57.10, 27.10, 6, 0.4), (49.62, 25.42, 7, 0.5),
    (49.98, 26.55, 4, 0.4), (49.30, 32.70, 18, 0.15), (51.00, 31.50, 18, 0.15),
]
SPARSE_FOREST = False
GREEN = False


# ---------------------------------------------------------------------------------------------
# Models of this theater's library the layouts use, and its airfield's aprons.

KINDS = {"houses": 40, "houses2": 40, "buildings": 40, "patrol_boat": 69, "land": 0, "carrier": 73,
         "carrier_object": 87, "airstrip": 80}
APRONS = [(-234, -78, -172, -16), (78, 500, -750, -500)]
LAND_COLOUR = 54

# The empty open land: a hamlet here and there.
TEMPLATES["hamlet"] = [(0, 0, "houses"), (170, -120, "houses")]
TEMPLATES["lone_house"] = [(0, 0, "houses")]
FILLER = [{}, {}, {}, {(1, 2): "hamlet"}, {(2, 1): "lone_house"}]
