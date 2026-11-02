# Debriefing mise (src/debrief.rs) a hlášení simulace (stealth-sim score.rs).

## Jak let skončil

outcome-landed = Bezpečné přistání
outcome-bad-landing = Špatné přistání
outcome-ejected = Katapultoval se - přistál na padáku
outcome-rescued = Katapultoval se - zachráněn CSAR
outcome-captured = Katapultoval se - zajat
outcome-crashed = Havaroval
outcome-ejection-failed = Katapultáž selhala
outcome-aborted = Mise přerušena
outcome-collision = Srážka

## Hodnocení

rating-unsatisfactory = Nevyhovující
rating-marginal = Hraniční
rating-satisfactory = Uspokojivé
rating-excellent = Výborné
rating-outstanding = Vynikající

## Body

points-primary = Hlavní cíl
points-secondary = Vedlejší cíl
points-aircraft = Zničená letadla
points-unseen = Sestřeleno nepozorovaně
points-ground = Zničené pozemní cíle
points-drones = Sestřelené drony
points-drones-through = Prošlé drony
points-nuclear = Zasažená jaderná elektrárna
points-detected = Detekován
points-missile-hits = Zásahy střelami

## Časová osa (názvy míst a letadel jako v misi)

timeline-primary = {" "}(hlavní)
timeline-secondary = {" "}(vedlejší)
timeline-destroyed = Zničeno: { $place }{ $tag }
timeline-destroyed-ground = Zničen pozemní cíl{ $tag }
timeline-shot-down = Sestřeleno: { $aircraft }{ $tag }
timeline-hit = Zasažen střelou
timeline-seen = Spatřen letounem { $aircraft }
timeline-radar-id = Radar ID: { $place }
timeline-objective = Cíl dosažen{ $tag }
timeline-got-through = { $aircraft } prošel
timeline-photo = Pořízena fotografie{ $tag }

## Okno debriefingu

debrief-title = DEBRIEFING MISE
debrief-training = CVIČNÁ MISE
debrief-results = VÝSLEDKY
debrief-flight-map = MAPA LETU
debrief-section-results = Výsledky
debrief-accomplished = SPLNĚNO
debrief-failed = NESPLNĚNO
debrief-primary = HLAVNÍ
debrief-secondary = VEDLEJŠÍ
debrief-aircraft = LETADLA
debrief-drones = DRONY
debrief-drones-value = sestřeleno { $down }, prošlo { $through }
debrief-ground = POZEMNÍ
debrief-weapons = ZBRANĚ
debrief-radar-ids = RADAR ID
debrief-visual-ids = VIZUÁLNÍ ID
debrief-hits-taken = UTRŽENÉ ZÁSAHY
debrief-time = ČAS
debrief-section-points = Body
debrief-difficulty = OBTÍŽNOST
debrief-total = CELKEM
debrief-section-timeline = Časová osa
debrief-no-events = Žádné události
debrief-section-exposure = Radarová expozice
debrief-tracked = SLEDOVÁN
debrief-closest = NEJBLÍŽE
debrief-worst = NEJHORŠÍ
debrief-at = V { $time }
debrief-altitude = VÝŠKA
debrief-speed = RYCHLOST
debrief-margin = REZERVA
debrief-strongest = NEJSILNĚJŠÍ
debrief-tracked-by = SLEDOVÁN: { $sites }
debrief-section-map = Mapa
debrief-legend-safe = Nedetekovatelný
debrief-legend-close = Blízko detekce
debrief-legend-detectable = Detekovatelný
debrief-legend-tracked = Sledován / odpal střely
debrief-legend-hit = Utržený zásah / sestřel
map-symbols = Symboly
map-legend-clouds = Oblaka / bouřkové věže
map-legend-snow = Sníh, slabý až silný
map-legend-rain = Déšť, slabý až silný
map-legend-thunderstorm = Bouřková buňka
debrief-pause = PAUZA
debrief-play = PŘEHRÁT
