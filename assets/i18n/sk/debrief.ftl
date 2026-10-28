# Debriefing misie (src/debrief.rs) a hlásenie simulácie (stealth-sim score.rs).

## Ako sa let skončil

outcome-landed = Bezpečné pristátie
outcome-bad-landing = Nepodarené pristátie
outcome-ejected = Katapultáž - pristátie na padáku
outcome-rescued = Katapultáž - zachránený jednotkou CSAR
outcome-captured = Katapultáž - zajatie
outcome-crashed = Havária
outcome-ejection-failed = Katapultáž zlyhala
outcome-aborted = Misia prerušená
outcome-collision = Zrážka

## Hodnotenia

rating-unsatisfactory = Nevyhovujúce
rating-marginal = Hraničné
rating-satisfactory = Vyhovujúce
rating-excellent = Výborné
rating-outstanding = Vynikajúce

## Body

points-primary = Hlavný cieľ
points-secondary = Vedľajší cieľ
points-aircraft = Zničené lietadlá
points-unseen = Zostrelené nepozorovane
points-ground = Zničené pozemné ciele
points-drones = Zostrelené drony
points-drones-through = Drony, ktoré prenikli
points-nuclear = Zásah jadrovej elektrárne
points-detected = Zistenie
points-missile-hits = Zásahy raketami

## Časová os (názvy miest a lietadiel ako v misii)

timeline-primary = {" "}(hlavný)
timeline-secondary = {" "}(vedľajší)
timeline-destroyed = Zničené: { $place }{ $tag }
timeline-destroyed-ground = Zničený pozemný cieľ{ $tag }
timeline-shot-down = Zostrelený: { $aircraft }{ $tag }
timeline-hit = Zásah raketou
timeline-seen = Spozorovaný lietadlom: { $aircraft }
timeline-radar-id = Rádiolokačná identifikácia: { $place }
timeline-objective = Cieľ dosiahnutý{ $tag }
timeline-got-through = Prenikol: { $aircraft }
timeline-photo = Fotografia nasnímaná{ $tag }

## Okno debriefingu

debrief-title = DEBRIEFING MISIE
debrief-training = CVIČNÁ MISIA
debrief-results = VÝSLEDKY
debrief-flight-map = MAPA LETU
debrief-section-results = Výsledky
debrief-accomplished = SPLNENÉ
debrief-failed = NESPLNENÉ
debrief-primary = HLAVNÝ
debrief-secondary = VEDĽAJŠÍ
debrief-aircraft = LIETADLÁ
debrief-drones = DRONY
debrief-drones-value = { $down } zostrelených, { $through } ktoré prenikli
debrief-ground = POZEMNÉ
debrief-weapons = ZBRANE
debrief-radar-ids = RADAR ID
debrief-visual-ids = VIZUÁLNE ID
debrief-hits-taken = UTRPENÉ ZÁSAHY
debrief-time = ČAS
debrief-section-points = Body
debrief-difficulty = OBTIAŽNOSŤ
debrief-total = SPOLU
debrief-section-timeline = Časová os
debrief-no-events = Žiadne udalosti
debrief-section-exposure = Radarová expozícia
debrief-tracked = SLEDOVANÝ
debrief-closest = NAJBLIŽŠIE
debrief-worst = NAJHORŠIE
debrief-at = V čase { $time }
debrief-altitude = VÝŠKA
debrief-speed = RÝCHLOSŤ
debrief-margin = REZERVA
debrief-strongest = NAJSILNEJŠÍ
debrief-tracked-by = SLEDOVANÝ: { $sites }
debrief-section-map = Mapa
debrief-legend-safe = Nezistiteľný
debrief-legend-close = Blízko zistenia
debrief-legend-detectable = Zistiteľný
debrief-legend-tracked = Sledovaný / odpal rakety
debrief-legend-hit = Utrpený zásah / zostrel
debrief-pause = PAUZA
debrief-play = PREHRAŤ
