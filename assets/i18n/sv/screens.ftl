# Kneeboard (src/kneeboard.rs, linjetypsnitt), uppdragsplaneraren (src/planner.rs) och de
# sparade uppdragen (src/savegame.rs). Navigeringsförkortningarna (PT, BRG, ETA, PRI, SEC, KM,
# KT) förblir engelska.

kb-brief = BRIEF
kb-route = RUTT
kb-map = KARTA
kb-target = MÅL
kb-keys = TANGENTER
kb-tip = Kartor, uppdragsanteckningar och tangentguiden
kb-geran-file = GERAN-2 (AKT)
kb-objectives = MÅL
kb-primary = PRIMÄRT: { $objective }
kb-secondary = SEKUNDÄRT: { $objective }
kb-times = TIDER
kb-deadline = SENAST { $time }
kb-now = NU { $time }
kb-conditions = FÖRHÅLLANDEN
kb-moon = MÅNE: { $phase } ({ $lit } % UPPLYST)
kb-armament = BEVÄPNING
kb-roe = INSATSREGLER
kb-total = TOTALT { $km } KM VID { $kt } KT
kb-fuel = BRÄNSLE { $lbs } LBS
kb-preparing = FÖRBEREDER...
kb-airborne = I LUFTEN
kb-developing = FRAMKALLAR...
kb-no-photo = INGET FOTO
kb-airborne-target = LUFTMÅL
kb-scroll-vr = SCROLLA: STYRSPAK, FINGRET PÅ BRÄDAN
kb-scroll-desktop = SCROLLA: MUSHJULET ÖVER BRÄDAN

## Uppdragsplanerare

planner-low = LÅG 500 FT
planner-medium = MEDEL 5000 FT
planner-high = HÖG 20000 FT
planner-title = UPPDRAGSPLANERARE
planner-route = Rutt
planner-base = BAS
planner-total = TOTALT { $km } KM  { $time }
planner-briefed = Briefad rutt { $km } km, bränsle { $fuel } lbs
planner-coverage = Radartäckning för
    .note = Radartäckningen förutsätter planflygning i 480 kt med stängda bombutrymmen.
planner-detected = { $km } km av rutten upptäckt
planner-pulse = Pulsradar (P) detektionsavstånd
planner-doppler = Dopplerradar (D) detektionsavstånd
planner-missile = Robotens räckvidd
planner-uav = Fientlig UAV
planner-edit-hint = Dra en punkt för att flytta den, kartan för att panorera. Klicka på en sträcka för att lägga till en punkt, högerklicka för att ta bort den. Hjul: zoom.
planner-reset = ÅTERSTÄLL
planner-view-hint = Dra kartan för att panorera. Hjul: zoom.

## Sparade uppdrag

saves-title = SPARADE UPPDRAG
saves-subtitle = Sparade under flygning från menyn: SPARA UPPDRAG
saves-none = Inga sparade uppdrag
saves-line = { $theater }  -  { $aircraft }  -  uppdragstid { $time }
saves-saved-at = Sparad { $date }
saves-unreadable = Oläsbar sparfil (annan version)
saves-delete =
    .facts = Ta bort denna sparfil
saves-back =
    .facts = Tillbaka till skärmen för enskilda uppdrag
