# Debriefingul misiunii (src/debrief.rs) și raportul simulării (stealth-sim score.rs).

## Cum s-a încheiat un zbor

outcome-landed = Aterizare reușită
outcome-bad-landing = Aterizare grea
outcome-ejected = Ejectat - aterizat cu parașuta
outcome-rescued = Ejectat - salvat de CSAR
outcome-captured = Ejectat - capturat
outcome-crashed = Prăbușit
outcome-ejection-failed = Ejectare eșuată
outcome-aborted = Misiune întreruptă
outcome-collision = Coliziune

## Calificative

rating-unsatisfactory = Nesatisfăcător
rating-marginal = La limită
rating-satisfactory = Satisfăcător
rating-excellent = Excelent
rating-outstanding = Remarcabil

## Puncte

points-primary = Obiectiv principal
points-secondary = Obiectiv secundar
points-aircraft = Aeronave distruse
points-unseen = Doborâte nevăzut
points-ground = Ținte terestre distruse
points-drones = Drone doborâte
points-drones-through = Drone care au trecut
points-nuclear = Centrală nucleară lovită
points-detected = Detectat
points-missile-hits = Lovituri de rachetă

## Cronologie (denumirile de locuri și aeronave ca în misiune)

timeline-primary = {" "}(principal)
timeline-secondary = {" "}(secundar)
timeline-destroyed = Distrus: { $place }{ $tag }
timeline-destroyed-ground = Țintă terestră distrusă{ $tag }
timeline-shot-down = Doborât: { $aircraft }{ $tag }
timeline-hit = Lovit de o rachetă
timeline-seen = Văzut de un { $aircraft }
timeline-radar-id = ID radar de la { $place }
timeline-objective = Obiectiv atins{ $tag }
timeline-got-through = { $aircraft } a trecut
timeline-photo = Fotografie făcută{ $tag }

## Fereastra de debriefing

debrief-title = DEBRIEFING MISIUNE
debrief-training = MISIUNE DE INSTRUIRE
debrief-results = REZULTATE
debrief-flight-map = HARTA ZBORULUI
debrief-section-results = Rezultate
debrief-accomplished = ÎNDEPLINIT
debrief-failed = EȘUAT
debrief-primary = PRINCIPAL
debrief-secondary = SECUNDAR
debrief-aircraft = AERONAVE
debrief-drones = DRONE
debrief-drones-value = { $down } doborâte, { $through } au trecut
debrief-ground = SOL
debrief-weapons = ARMAMENT
debrief-radar-ids = ID RADAR
debrief-visual-ids = ID VIZUALE
debrief-hits-taken = LOVITURI PRIMITE
debrief-time = TIMP
debrief-section-points = Puncte
debrief-difficulty = DIFICULTATE
debrief-total = TOTAL
debrief-section-timeline = Cronologie
debrief-no-events = Niciun eveniment
debrief-section-exposure = Expunere radar
debrief-tracked = URMĂRIT
debrief-closest = CEL MAI APROAPE
debrief-worst = CEL MAI GRAV
debrief-at = La { $time }
debrief-altitude = ALTITUDINE
debrief-speed = VITEZĂ
debrief-margin = MARJĂ
debrief-strongest = CEL MAI PUTERNIC
debrief-tracked-by = URMĂRIT DE { $sites }
debrief-section-map = Hartă
debrief-legend-safe = Nedetectabil
debrief-legend-close = Aproape de detecție
debrief-legend-detectable = Detectabil
debrief-legend-tracked = Urmărit / lansare de rachetă
debrief-legend-hit = Lovitură primită / doborâre
debrief-pause = PAUZĂ
debrief-play = REDARE
