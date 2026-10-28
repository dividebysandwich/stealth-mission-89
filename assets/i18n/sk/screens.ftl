# Kneeboard (src/kneeboard.rs, vektorový font), plánovač misie (src/planner.rs) a uložené
# misie (src/savegame.rs). Navigačné skratky (PT, BRG, ETA, PRI, SEC, KM, KT) zostávajú
# po anglicky.

kb-brief = BRÍFING
kb-route = TRASA
kb-map = MAPA
kb-target = CIEĽ
kb-keys = KLÁVESY
kb-tip = Mapy, poznámky k misii a prehľad klávesov
kb-geran-file = GERAN-2 (SPIS)
kb-objectives = CIELE
kb-primary = HLAVNÝ: { $objective }
kb-secondary = VEDĽAJŠÍ: { $objective }
kb-times = ČASY
kb-deadline = TERMÍN { $time }
kb-now = TERAZ { $time }
kb-conditions = PODMIENKY
kb-moon = MESIAC: { $phase } ({ $lit }% OSVETLENÝ)
kb-armament = VÝZBROJ
kb-roe = PRAVIDLÁ NASADENIA
kb-total = SPOLU { $km } KM PRI { $kt } KT
kb-fuel = PALIVO { $lbs } LBS
kb-preparing = PRIPRAVUJE SA...
kb-airborne = VO VZDUCHU
kb-developing = VYVOLÁVA SA...
kb-no-photo = ŽIADNA FOTOGRAFIA
kb-airborne-target = VZDUŠNÝ CIEĽ
kb-scroll-vr = POSÚVANIE: PÁČKA, PRST NA TABULI
kb-scroll-desktop = POSÚVANIE: KOLIESKO MYŠI NAD TABUĽOU

## Plánovač misie

planner-low = NÍZKO 500 FT
planner-medium = STREDNE 5000 FT
planner-high = VYSOKO 20000 FT
planner-title = PLÁNOVAČ MISIE
planner-route = Trasa
planner-base = ZÁKLADŇA
planner-total = SPOLU { $km } KM  { $time }
planner-briefed = Trasa z brífingu { $km } km, palivo { $fuel } lbs
planner-coverage = Radarové pokrytie pre
    .note = Radarové pokrytie počíta s horizontálnym letom rýchlosťou 480 kt a zatvorenými dverami šachty.
planner-detected = Zistených { $km } km trasy
planner-pulse = Dosah zistenia pulzného radaru (P)
planner-doppler = Dosah zistenia dopplerovského radaru (D)
planner-missile = Dosah rakiet
planner-uav = Nepriateľský UAV
planner-edit-hint = Potiahnutím bodu ho presuniete, potiahnutím mapy ju posuniete. Kliknutím na úsek pridáte bod, pravým kliknutím ho odstránite. Koliesko: priblíženie.
planner-reset = OBNOVIŤ
planner-view-hint = Potiahnutím mapy ju posuniete. Koliesko: priblíženie.

## Uložené misie

saves-title = ULOŽENÉ MISIE
saves-subtitle = Uložené počas letu z menu: ULOŽIŤ MISIU
saves-none = Žiadne uložené misie
saves-line = { $theater }  -  { $aircraft }  -  čas misie { $time }
saves-saved-at = Uložené { $date }
saves-unreadable = Nečitateľné uloženie (iná verzia)
saves-delete =
    .facts = Odstrániť toto uloženie
saves-back =
    .facts = Späť na obrazovku jednotlivej misie
