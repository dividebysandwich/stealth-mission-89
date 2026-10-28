# Uppdragets debrief (src/debrief.rs) och simuleringens rapport (stealth-sim score.rs).

## Hur en flygning slutade

outcome-landed = Säker landning
outcome-bad-landing = Hård landning
outcome-ejected = Katapulterade - landade i fallskärm
outcome-rescued = Katapulterade - räddad av CSAR
outcome-captured = Katapulterade - tillfångatagen
outcome-crashed = Havererade
outcome-ejection-failed = Katapulteringen misslyckades
outcome-aborted = Uppdraget avbrutet
outcome-collision = Kollision

## Betyg

rating-unsatisfactory = Otillfredsställande
rating-marginal = Marginellt
rating-satisfactory = Tillfredsställande
rating-excellent = Utmärkt
rating-outstanding = Enastående

## Poäng

points-primary = Primärt mål
points-secondary = Sekundärt mål
points-aircraft = Flygplan förstörda
points-unseen = Nedskjutna oupptäckt
points-ground = Markmål förstörda
points-drones = Drönare nedskjutna
points-drones-through = Drönare som tog sig igenom
points-nuclear = Kärnkraftverk träffat
points-detected = Upptäckt
points-missile-hits = Robotträffar

## Tidslinje (plats- och flygplansnamn som i uppdraget)

timeline-primary = {" "}(primärt)
timeline-secondary = {" "}(sekundärt)
timeline-destroyed = Förstörde { $place }{ $tag }
timeline-destroyed-ground = Förstörde markmål{ $tag }
timeline-shot-down = Sköt ned { $aircraft }{ $tag }
timeline-hit = Träffad av en robot
timeline-seen = Upptäckt av en { $aircraft }
timeline-radar-id = Radar-ID av { $place }
timeline-objective = Målet nått{ $tag }
timeline-got-through = { $aircraft } tog sig igenom
timeline-photo = Foto taget{ $tag }

## Debriefingfönstret

debrief-title = UPPDRAGSDEBRIEF
debrief-training = ÖVNINGSUPPDRAG
debrief-results = RESULTAT
debrief-flight-map = FLYGKARTA
debrief-section-results = Resultat
debrief-accomplished = GENOMFÖRT
debrief-failed = MISSLYCKAT
debrief-primary = PRIMÄRT
debrief-secondary = SEKUNDÄRT
debrief-aircraft = FLYGPLAN
debrief-drones = DRÖNARE
debrief-drones-value = { $down } nedskjutna, { $through } igenom
debrief-ground = MARK
debrief-weapons = VAPEN
debrief-radar-ids = RADAR-ID
debrief-visual-ids = VISUELLA ID
debrief-hits-taken = TAGNA TRÄFFAR
debrief-time = TID
debrief-section-points = Poäng
debrief-difficulty = SVÅRIGHETSGRAD
debrief-total = TOTALT
debrief-section-timeline = Tidslinje
debrief-no-events = Inga händelser
debrief-section-exposure = Radarexponering
debrief-tracked = SPÅRAD
debrief-closest = NÄRMAST
debrief-worst = VÄRST
debrief-at = Vid { $time }
debrief-altitude = HÖJD
debrief-speed = FART
debrief-margin = MARGINAL
debrief-strongest = STARKAST
debrief-tracked-by = SPÅRAD AV { $sites }
debrief-section-map = Karta
debrief-legend-safe = Ej detekterbar
debrief-legend-close = Nära detektion
debrief-legend-detectable = Detekterbar
debrief-legend-tracked = Spårad / robotuppskjutning
debrief-legend-hit = Tagen träff / nedskjutning
debrief-pause = PAUS
debrief-play = SPELA
