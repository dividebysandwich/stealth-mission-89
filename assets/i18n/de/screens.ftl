# Das Kniebrett (src/kneeboard.rs, Vektorschrift), der Missionsplaner (src/planner.rs) und die
# gespeicherten Missionen (src/savegame.rs). Navigationsabkürzungen (PT, BRG, ETA, PRI, SEC, KM, KT)
# bleiben englisch.

kb-brief = BRIEF
kb-route = ROUTE
kb-map = KARTE
kb-target = ZIEL
kb-keys = TASTEN
kb-tip = Karten, Missionsnotizen und die Tastenübersicht
kb-geran-file = GERAN-2 (AKTE)
kb-objectives = ZIELE
kb-primary = PRIMÄR: { $objective }
kb-secondary = SEKUNDÄR: { $objective }
kb-times = ZEITEN
kb-deadline = FRIST { $time }
kb-now = JETZT { $time }
kb-conditions = BEDINGUNGEN
kb-moon = MOND: { $phase } ({ $lit } % BELEUCHTET)
kb-armament = BEWAFFNUNG
kb-roe = EINSATZREGELN
kb-total = GESAMT { $km } KM BEI { $kt } KT
kb-fuel = TREIBSTOFF { $lbs } LBS
kb-preparing = WIRD VORBEREITET...
kb-airborne = IN DER LUFT
kb-developing = WIRD ENTWICKELT...
kb-no-photo = KEIN FOTO
kb-airborne-target = LUFTZIEL
kb-scroll-vr = BLÄTTERN: STICK, FINGER AUF DEM KNIEBRETT
kb-scroll-desktop = BLÄTTERN: MAUSRAD ÜBER DEM KNIEBRETT

## Missionsplaner

planner-low = TIEF 500 FT
planner-medium = MITTEL 5000 FT
planner-high = HOCH 20000 FT
planner-title = MISSIONSPLANER
planner-route = Route
planner-base = BASIS
planner-total = GESAMT { $km } KM  { $time }
planner-briefed = Route des Briefings { $km } km, Treibstoff { $fuel } lbs
planner-coverage = Radarabdeckung für
    .note = Die Radarabdeckung gilt für Horizontalflug mit 480 kt bei geschlossenen Waffenschächten.
planner-detected = { $km } km der Route entdeckt
planner-pulse = Ortungsreichweite Pulsradar (P)
planner-doppler = Ortungsreichweite Dopplerradar (D)
planner-missile = Raketenreichweite
planner-uav = Feindliche Drohne
planner-edit-hint = Einen Punkt ziehen, um ihn zu verschieben, die Karte, um sie zu verschieben. Auf einen Abschnitt klicken, um einen Punkt hinzuzufügen, Rechtsklick, um ihn zu entfernen. Mausrad: Zoom.
planner-reset = ZURÜCKSETZEN
planner-view-hint = Die Karte ziehen, um sie zu verschieben. Mausrad: Zoom.

## Gespeicherte Missionen

saves-title = GESPEICHERTE MISSIONEN
saves-subtitle = Im Flug über das Menü gespeichert: MISSION SPEICHERN
saves-none = Keine gespeicherten Missionen
saves-line = { $theater }  -  { $aircraft }  -  Missionszeit { $time }
saves-saved-at = Gespeichert am { $date }
saves-unreadable = Unlesbarer Spielstand (andere Version)
saves-delete =
    .facts = Diesen Spielstand löschen
saves-back =
    .facts = Zurück zum Einzelmissionsbildschirm
