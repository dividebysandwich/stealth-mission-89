# Kneeboard (src/kneeboard.rs, kontūru fonts), misiju plānotājs (src/planner.rs) un
# saglabātās misijas (src/savegame.rs). Navigācijas saīsinājumi (PT, BRG, ETA, PRI, SEC, KM, KT)
# paliek angļu valodā.

kb-brief = BRĪFINGS
kb-route = MARŠRUTS
kb-map = KARTE
kb-target = MĒRĶIS
kb-keys = TAUSTIŅI
kb-tip = Kartes, misijas piezīmes un taustiņu pārskats
kb-geran-file = GERAN-2 (DOSIERS)
kb-objectives = UZDEVUMI
kb-primary = GALVENAIS: { $objective }
kb-secondary = PAPILDU: { $objective }
kb-times = LAIKI
kb-deadline = TERMIŅŠ { $time }
kb-now = TAGAD { $time }
kb-conditions = APSTĀKĻI
kb-moon = MĒNESS: { $phase } ({ $lit }% APGAISMOTS)
kb-armament = BRUŅOJUMS
kb-roe = IESAISTĪŠANĀS NOTEIKUMI
kb-total = KOPĀ { $km } KM PIE { $kt } KT
kb-fuel = DEGVIELA { $lbs } LBS
kb-preparing = SAGATAVO...
kb-airborne = GAISĀ
kb-developing = IZSTRĀDĀ...
kb-no-photo = NAV FOTO
kb-airborne-target = GAISA MĒRĶIS
kb-scroll-vr = RITINĀT: KURSORSVIRA, PIRKSTS UZ DĒĻA
kb-scroll-desktop = RITINĀT: PELES RITENĪTIS VIRS DĒĻA

## Misiju plānotājs

planner-low = ZEMS 500 FT
planner-medium = VIDĒJS 5000 FT
planner-high = AUGSTS 20000 FT
planner-title = MISIJU PLĀNOTĀJS
planner-route = Maršruts
planner-base = BĀZE
planner-total = KOPĀ { $km } KM  { $time }
planner-briefed = Brīfinga maršruts { $km } km, degviela { $fuel } lbs
planner-coverage = Radaru pārklājums:
    .note = Radaru pārklājums pieņem 480 kt horizontālu lidojumu, nodalījuma durvis aizvērtas.
planner-detected = Atklāti { $km } km no maršruta
planner-pulse = Impulsu radara (P) atklāšanas rādiuss
planner-doppler = Doplera radara (D) atklāšanas rādiuss
planner-missile = Raķetes darbības rādiuss
planner-uav = Ienaidnieka UAV
planner-edit-hint = Velciet punktu, lai to pārvietotu, karti - lai bīdītu. Klikšķis uz posma pievieno punktu, labais klikšķis noņem. Ritenītis: tālummaiņa.
planner-reset = ATIESTATĪT
planner-view-hint = Velciet karti, lai bīdītu. Ritenītis: tālummaiņa.
planner-layers = SLĀŅI
planner-layer-sams = Radars / SAM pozīcija
planner-radar-types = Radaru tipi
planner-layer-pulse = Impulsu radara atklāšanas rādiuss
planner-layer-doppler = Doplera radara atklāšanas rādiuss
planner-layer-airfield = Lidlauks

## Saglabātās misijas

saves-title = SAGLABĀTĀS MISIJAS
saves-subtitle = Saglabātas lidojumā no izvēlnes: SAVE MISSION
saves-none = Nav saglabātu misiju
saves-line = { $theater }  -  { $aircraft }  -  misijas laiks { $time }
saves-saved-at = Saglabāts { $date }
saves-unreadable = Nenolasāms saglabājums (cita versija)
saves-delete =
    .facts = Dzēst šo saglabājumu
saves-back =
    .facts = Atpakaļ uz atsevišķās misijas ekrānu
