# Kneeboard-ul (src/kneeboard.rs, font vectorial), planificatorul de misiune (src/planner.rs)
# și misiunile salvate (src/savegame.rs). Abrevierile de navigație (PT, BRG, ETA, PRI, SEC,
# KM, KT) rămân în engleză.

kb-brief = BRIEF
kb-route = RUTĂ
kb-map = HARTĂ
kb-target = ȚINTĂ
kb-keys = TASTE
kb-tip = Hărți, note de misiune și ghidul tastelor
kb-geran-file = GERAN-2 (DOSAR)
kb-objectives = OBIECTIVE
kb-primary = PRINCIPAL: { $objective }
kb-secondary = SECUNDAR: { $objective }
kb-times = ORARE
kb-deadline = LIMITĂ { $time }
kb-now = ACUM { $time }
kb-conditions = CONDIȚII
kb-moon = LUNA: { $phase } ({ $lit }% LUMINATĂ)
kb-armament = ARMAMENT
kb-roe = REGULI DE ANGAJARE
kb-total = TOTAL { $km } KM LA { $kt } KT
kb-fuel = COMBUSTIBIL { $lbs } LBS
kb-preparing = SE PREGĂTEȘTE...
kb-airborne = ÎN ZBOR
kb-developing = SE DEZVOLTĂ...
kb-no-photo = FĂRĂ FOTO
kb-airborne-target = ȚINTĂ AERIANĂ
kb-scroll-vr = DERULARE: THUMBSTICK, DEGETUL PE KNEEBOARD
kb-scroll-desktop = DERULARE: ROTIȚA MOUSE-ULUI PESTE KNEEBOARD

## Planificatorul de misiune

planner-low = JOASĂ 500 FT
planner-medium = MEDIE 5000 FT
planner-high = ÎNALTĂ 20000 FT
planner-title = PLANIFICATOR DE MISIUNE
planner-route = Rută
planner-base = BAZĂ
planner-total = TOTAL { $km } KM  { $time }
planner-briefed = Rută din briefing { $km } km, combustibil { $fuel } lbs
planner-coverage = Acoperire radar pentru
    .note = Acoperirea radar presupune zbor orizontal la 480 kt, soute închise.
planner-detected = { $km } km din rută detectați
planner-pulse = Rază de detecție radar cu impulsuri (P)
planner-doppler = Rază de detecție radar Doppler (D)
planner-missile = Raza rachetei
planner-uav = UAV inamic
planner-edit-hint = Trageți un punct pentru a-l muta, harta pentru a o deplasa. Clic pe un tronson pentru a adăuga un punct, clic dreapta pentru a-l șterge. Rotiță: zoom.
planner-reset = RESETARE
planner-view-hint = Trageți harta pentru a o deplasa. Rotiță: zoom.
planner-layers = STRATURI
planner-layer-sams = Radar / amplasament SAM
planner-radar-types = Tipuri de radar
planner-layer-pulse = Rază de detecție radar cu impulsuri
planner-layer-doppler = Rază de detecție radar Doppler
planner-layer-airfield = Aerodrom

## Misiuni salvate

saves-title = MISIUNI SALVATE
saves-subtitle = Salvate în zbor din meniu: SAVE MISSION
saves-none = Nicio misiune salvată
saves-line = { $theater }  -  { $aircraft }  -  durata misiunii { $time }
saves-saved-at = Salvat { $date }
saves-unreadable = Salvare ilizibilă (altă versiune)
saves-delete =
    .facts = Șterge această salvare
saves-back =
    .facts = Înapoi la ecranul misiunii individuale
