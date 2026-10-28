# Misijos debrifingas (src/debrief.rs) ir simuliacijos ataskaita (stealth-sim score.rs).

## Kaip baigėsi skrydis

outcome-landed = Saugus tūpimas
outcome-bad-landing = Nesėkmingas tūpimas
outcome-ejected = Katapultavosi - nusileido parašiutu
outcome-rescued = Katapultavosi - išgelbėtas CSAR
outcome-captured = Katapultavosi - paimtas į nelaisvę
outcome-crashed = Sudužo
outcome-ejection-failed = Katapultavimasis nepavyko
outcome-aborted = Misija nutraukta
outcome-collision = Susidūrimas

## Įvertinimai

rating-unsatisfactory = Nepatenkinamai
rating-marginal = Vos patenkinamai
rating-satisfactory = Patenkinamai
rating-excellent = Puikiai
rating-outstanding = Išskirtinai

## Taškai

points-primary = Pagrindinė užduotis
points-secondary = Antrinė užduotis
points-aircraft = Sunaikinti orlaiviai
points-unseen = Numušta nepastebėtam
points-ground = Sunaikinti žemės taikiniai
points-drones = Numušti dronai
points-drones-through = Prasiveržę dronai
points-nuclear = Pataikyta į branduolinę jėgainę
points-detected = Aptikta
points-missile-hits = Raketų pataikymai

## Eigos juosta (vietovių ir orlaivių pavadinimai kaip misijoje)

timeline-primary = {" "}(pagrindinė)
timeline-secondary = {" "}(antrinė)
timeline-destroyed = Sunaikinta: { $place }{ $tag }
timeline-destroyed-ground = Sunaikintas žemės taikinys{ $tag }
timeline-shot-down = Numuštas: { $aircraft }{ $tag }
timeline-hit = Pataikė raketa
timeline-seen = Pastebėjo: { $aircraft }
timeline-radar-id = Radaro identifikacija: { $place }
timeline-objective = Užduotis pasiekta{ $tag }
timeline-got-through = Prasiveržė: { $aircraft }
timeline-photo = Nuotrauka padaryta{ $tag }

## Debrifingo langas

debrief-title = MISIJOS DEBRIFINGAS
debrief-training = MOKOMOJI MISIJA
debrief-results = REZULTATAI
debrief-flight-map = SKRYDŽIO ŽEMĖLAPIS
debrief-section-results = Rezultatai
debrief-accomplished = ĮVYKDYTA
debrief-failed = NEPAVYKO
debrief-primary = PAGRINDINĖ
debrief-secondary = ANTRINĖ
debrief-aircraft = ORLAIVIAI
debrief-drones = DRONAI
debrief-drones-value = numušta: { $down }, prasiveržė: { $through }
debrief-ground = ŽEMĖ
debrief-weapons = GINKLAI
debrief-radar-ids = RADARO ID
debrief-visual-ids = VIZUALŪS ID
debrief-hits-taken = PATAIKYMAI
debrief-time = LAIKAS
debrief-section-points = Taškai
debrief-difficulty = SUNKUMAS
debrief-total = IŠ VISO
debrief-section-timeline = Eiga
debrief-no-events = Įvykių nėra
debrief-section-exposure = Radaro poveikis
debrief-tracked = SEKTA
debrief-closest = ARTIMIAUSIAI
debrief-worst = BLOGIAUSIAI
debrief-at = Laikas: { $time }
debrief-altitude = AUKŠTIS
debrief-speed = GREITIS
debrief-margin = ATSARGA
debrief-strongest = STIPRIAUSIAS
debrief-tracked-by = SEKĖ: { $sites }
debrief-section-map = Žemėlapis
debrief-legend-safe = Neaptinkama
debrief-legend-close = Beveik aptinkama
debrief-legend-detectable = Aptinkama
debrief-legend-tracked = Sekama / raketos paleidimas
debrief-legend-hit = Pataikė / numušta
debrief-pause = PAUZĖ
debrief-play = LEISTI
