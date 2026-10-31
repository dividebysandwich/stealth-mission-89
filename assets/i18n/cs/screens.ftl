# Kneeboard (src/kneeboard.rs, vektorový font), plánovač mise (src/planner.rs) a
# uložené mise (src/savegame.rs). Navigační zkratky (PT, BRG, ETA, PRI, SEC, KM, KT)
# zůstávají anglicky.

kb-brief = BRIEF
kb-route = TRASA
kb-map = MAPA
kb-target = CÍL
kb-keys = KLÁVESY
kb-tip = Mapy, poznámky k misi a přehled kláves
kb-geran-file = GERAN-2 (ARCHIV)
kb-objectives = CÍLE
kb-primary = HLAVNÍ: { $objective }
kb-secondary = VEDLEJŠÍ: { $objective }
kb-times = ČASY
kb-deadline = TERMÍN { $time }
kb-now = NYNÍ { $time }
kb-conditions = PODMÍNKY
kb-moon = MĚSÍC: { $phase } ({ $lit } % OSVĚTLENO)
kb-armament = VÝZBROJ
kb-roe = PRAVIDLA POUŽITÍ SÍLY
kb-total = CELKEM { $km } KM PŘI { $kt } KT
kb-fuel = PALIVO { $lbs } LBS
kb-preparing = PŘÍPRAVA...
kb-airborne = VE VZDUCHU
kb-developing = VYVOLÁVÁNÍ...
kb-no-photo = ŽÁDNÁ FOTKA
kb-airborne-target = VZDUŠNÝ CÍL
kb-scroll-vr = POSUN: THUMBSTICK, PRST NA DESCE
kb-scroll-desktop = POSUN: KOLEČKO MYŠI NAD DESKOU

## Plánovač mise

planner-low = NÍZKO 500 FT
planner-medium = STŘEDNĚ 5000 FT
planner-high = VYSOKO 20000 FT
planner-title = PLÁNOVAČ MISE
planner-route = Trasa
planner-base = ZÁKLADNA
planner-total = CELKEM { $km } KM  { $time }
planner-briefed = Plánovaná trasa { $km } km, palivo { $fuel } lbs
planner-coverage = Radarové pokrytí pro
    .note = Radarové pokrytí předpokládá přímý let rychlostí 480 kt se zavřenými dveřmi šachty.
planner-detected = { $km } km trasy detekováno
planner-pulse = Dosah detekce pulzního radaru (P)
planner-doppler = Dosah detekce dopplerovského radaru (D)
planner-missile = Dosah střel
planner-uav = Nepřátelský UAV
planner-edit-hint = Tažením bod přesunete, tažením mapy ji posunete. Kliknutím na úsek přidáte bod, pravým tlačítkem ho odeberete. Kolečko: zoom.
planner-reset = OBNOVIT
planner-view-hint = Tažením posunete mapu. Kolečko: zoom.
planner-layers = VRSTVY
planner-layer-sams = Radar / stanoviště SAM
planner-radar-types = Typy radarů
planner-layer-pulse = Dosah detekce pulzního radaru
planner-layer-doppler = Dosah detekce dopplerovského radaru
planner-layer-airfield = Letiště

## Uložené mise

saves-title = ULOŽENÉ MISE
saves-subtitle = Uloženo za letu z nabídky: SAVE MISSION
saves-none = Žádné uložené mise
saves-line = { $theater }  -  { $aircraft }  -  čas mise { $time }
saves-saved-at = Uloženo { $date }
saves-unreadable = Nečitelné uložení (jiná verze)
saves-delete =
    .facts = Smazat toto uložení
saves-back =
    .facts = Zpět na obrazovku jednotlivé mise
