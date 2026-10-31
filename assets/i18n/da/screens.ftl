# Kneeboard (src/kneeboard.rs, stregfont), missionsplanlæggeren (src/planner.rs) og de gemte
# missioner (src/savegame.rs). Navigationsforkortelserne (PT, BRG, ETA, PRI, SEC, KM, KT)
# forbliver på engelsk.

kb-brief = BRIEF
kb-route = RUTE
kb-map = KORT
kb-target = MÅL
kb-keys = TASTER
kb-tip = Kort, missionsnoter og tastguiden
kb-geran-file = GERAN-2 (FIL)
kb-objectives = MÅL
kb-primary = PRIMÆRT: { $objective }
kb-secondary = SEKUNDÆRT: { $objective }
kb-times = TIDER
kb-deadline = FRIST { $time }
kb-now = NU { $time }
kb-conditions = FORHOLD
kb-moon = MÅNE: { $phase } ({ $lit }% OPLYST)
kb-armament = BEVÆBNING
kb-roe = INDSATSREGLER
kb-total = I ALT { $km } KM VED { $kt } KT
kb-fuel = BRÆNDSTOF { $lbs } LBS
kb-preparing = FORBEREDER...
kb-airborne = I LUFTEN
kb-developing = FREMKALDER...
kb-no-photo = INTET FOTO
kb-airborne-target = LUFTMÅL
kb-scroll-vr = RUL: THUMBSTICK, FINGER PÅ TAVLEN
kb-scroll-desktop = RUL: MUSEHJUL OVER TAVLEN

## Missionsplanlægger

planner-low = LAV 500 FT
planner-medium = MELLEM 5000 FT
planner-high = HØJ 20000 FT
planner-title = MISSIONSPLANLÆGGER
planner-route = Rute
planner-base = BASE
planner-total = I ALT { $km } KM  { $time }
planner-briefed = Briefet rute { $km } km, brændstof { $fuel } lbs
planner-coverage = Radardækning for
    .note = Radardækningen forudsætter 480 kt vandret flyvning og lukkede bomberumsdøre.
planner-detected = { $km } km af ruten opdaget
planner-pulse = Pulsradarens (P) detektionsrækkevidde
planner-doppler = Dopplerradarens (D) detektionsrækkevidde
planner-missile = Missilrækkevidde
planner-uav = Fjendtlig UAV
planner-edit-hint = Træk et punkt for at flytte det, kortet for at panorere. Klik på en strækning for at tilføje et punkt, højreklik for at fjerne det. Hjul: zoom.
planner-reset = NULSTIL
planner-view-hint = Træk kortet for at panorere. Hjul: zoom.
planner-layers = LAG
planner-layer-sams = Radar / SAM-stilling
planner-radar-types = Radartyper
planner-layer-pulse = Pulsradarens detektionsrækkevidde
planner-layer-doppler = Dopplerradarens detektionsrækkevidde
planner-layer-airfield = Flyveplads

## Gemte missioner

saves-title = GEMTE MISSIONER
saves-subtitle = Gemt under flyvning fra menuen: GEM MISSION
saves-none = Ingen gemte missioner
saves-line = { $theater }  -  { $aircraft }  -  missionstid { $time }
saves-saved-at = Gemt { $date }
saves-unreadable = Ulæseligt gemt spil (en anden version)
saves-delete =
    .facts = Slet dette gemte spil
saves-back =
    .facts = Tilbage til skærmen for enkeltmissioner
