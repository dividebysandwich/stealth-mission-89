# Het kneeboard (streepjeslettertype), de missieplanner en de opgeslagen missies.
# De navigatieafkortingen (PT, BRG, ETA, PRI, SEC, KM, KT) blijven Engels.

kb-brief = BRIEF
kb-route = ROUTE
kb-map = KAART
kb-target = DOEL
kb-keys = TOETSEN
kb-tip = Kaarten, missienotities en de toetsengids
kb-geran-file = GERAN-2 (DOSSIER)
kb-objectives = DOELEN
kb-primary = PRIMAIR: { $objective }
kb-secondary = SECUNDAIR: { $objective }
kb-times = TIJDEN
kb-deadline = DEADLINE { $time }
kb-now = NU { $time }
kb-conditions = OMSTANDIGHEDEN
kb-moon = MAAN: { $phase } ({ $lit }% VERLICHT)
kb-armament = BEWAPENING
kb-roe = INZETREGELS
kb-total = TOTAAL { $km } KM OP { $kt } KT
kb-fuel = BRANDSTOF { $lbs } LBS
kb-preparing = VOORBEREIDEN...
kb-airborne = IN DE LUCHT
kb-developing = ONTWIKKELEN...
kb-no-photo = GEEN FOTO
kb-airborne-target = LUCHTDOEL
kb-scroll-vr = SCROLLEN: THUMBSTICK, VINGER OP HET BORD
kb-scroll-desktop = SCROLLEN: MUISWIEL OVER HET BORD

## Missieplanner

planner-low = LAAG 500 FT
planner-medium = MIDDEN 5000 FT
planner-high = HOOG 20000 FT
planner-title = MISSIEPLANNER
planner-route = Route
planner-base = BASIS
planner-total = TOTAAL { $km } KM  { $time }
planner-briefed = Gebriefde route { $km } km, brandstof { $fuel } lbs
planner-coverage = Radardekking voor
    .note = De radardekking gaat uit van horizontale vlucht op 480 kt met gesloten ruimdeuren.
planner-detected = { $km } km van de route gedetecteerd
planner-vprofile = VERTICAAL PROFIEL: terrein en radardetectie langs de route
planner-pulse = Detectiebereik pulsradar (P)
planner-doppler = Detectiebereik dopplerradar (D)
planner-missile = Raketbereik
planner-uav = Vijandelijke UAV
planner-edit-hint = Sleep een punt om het te verplaatsen, de kaart om te pannen. Klik op een traject om een punt toe te voegen, rechtsklik om het te verwijderen. Wiel: zoomen.
planner-edit-hint-big = Sleep een punt om het te verplaatsen, klik op een stuk om er een toe te voegen, op een punt voor de opties
planner-reset = HERSTEL
planner-add-after = ERNA TOEVOEGEN
planner-view-hint = Sleep de kaart om te pannen. Wiel: zoomen.
planner-layers = LAGEN
planner-layer-sams = Radar / SAM-stelling
planner-radar-types = Radartypen
planner-layer-pulse = Detectiebereik pulsradar
planner-layer-doppler = Detectiebereik dopplerradar
planner-layer-airfield = Vliegveld

## Opgeslagen missies

saves-title = OPGESLAGEN MISSIES
saves-none = Geen opgeslagen missies
saves-line = { $theater }  -  { $aircraft }  -  missietijd { $time }
saves-saved-at = Opgeslagen { $date }
saves-unreadable = Onleesbare save (andere versie)
saves-delete =
    .facts = Deze save verwijderen
saves-rename =
    .facts = Deze save hernoemen
saves-not-renamed = De save kon niet worden hernoemd: { $error }
saves-back =
    .facts = Terug naar het scherm voor losse missies
saves-note = IN DE VLUCHT OPGESLAGEN: MENU > MISSIE OPSLAAN
saves-delete-all = ALLES WISSEN
    .facts = Alle opslagen wissen
saves-count = Opgeslagen missies: { $count }
