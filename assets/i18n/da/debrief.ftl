# Missionens debriefing (src/debrief.rs) og simulationens rapport (stealth-sim score.rs).

## Hvordan en flyvning endte

outcome-landed = Sikker landing
outcome-bad-landing = Dårlig landing
outcome-ejected = Udskudt - landet i faldskærm
outcome-rescued = Udskudt - reddet af CSAR
outcome-captured = Udskudt - fanget
outcome-crashed = Styrtet
outcome-ejection-failed = Udskydning mislykkedes
outcome-aborted = Mission afbrudt
outcome-collision = Kollision

## Bedømmelser

rating-unsatisfactory = Utilfredsstillende
rating-marginal = Marginal
rating-satisfactory = Tilfredsstillende
rating-excellent = Fremragende
rating-outstanding = Enestående

## Point

points-primary = Primært mål
points-secondary = Sekundært mål
points-aircraft = Fly ødelagt
points-unseen = Skudt ned uset
points-ground = Jordmål ødelagt
points-drones = Droner skudt ned
points-drones-through = Droner igennem
points-nuclear = Atomkraftværk ramt
points-detected = Opdaget
points-missile-hits = Missiltræffere

## Tidslinje (steds- og flynavne som i missionen)

timeline-primary = {" "}(primært)
timeline-secondary = {" "}(sekundært)
timeline-destroyed = Ødelagt: { $place }{ $tag }
timeline-destroyed-ground = Jordmål ødelagt{ $tag }
timeline-shot-down = Skudt ned: { $aircraft }{ $tag }
timeline-hit = Ramt af et missil
timeline-seen = Set af en { $aircraft }
timeline-radar-id = Radar-ID fra { $place }
timeline-objective = Mål nået{ $tag }
timeline-got-through = { $aircraft } slap igennem
timeline-photo = Foto taget{ $tag }

## Debriefingvinduet

debrief-title = MISSIONSDEBRIEFING
debrief-training = TRÆNINGSMISSION
debrief-results = RESULTATER
debrief-flight-map = FLYVEKORT
debrief-section-results = Resultater
debrief-accomplished = GENNEMFØRT
debrief-failed = MISLYKKET
debrief-primary = PRIMÆRT
debrief-secondary = SEKUNDÆRT
debrief-aircraft = FLY
debrief-drones = DRONER
debrief-drones-value = { $down } nede, { $through } igennem
debrief-ground = JORD
debrief-weapons = VÅBEN
debrief-radar-ids = RADAR-ID'ER
debrief-visual-ids = VISUELLE ID'ER
debrief-hits-taken = MODTAGNE TRÆFFERE
debrief-time = TID
debrief-section-points = Point
debrief-difficulty = SVÆRHEDSGRAD
debrief-total = I ALT
debrief-section-timeline = Tidslinje
debrief-no-events = Ingen hændelser
debrief-section-exposure = Radareksponering
debrief-tracked = SPORET
debrief-closest = NÆRMEST
debrief-worst = VÆRST
debrief-at = Kl. { $time }
debrief-altitude = HØJDE
debrief-speed = FART
debrief-margin = MARGEN
debrief-strongest = STÆRKEST
debrief-tracked-by = SPORET AF { $sites }
debrief-section-map = Kort
debrief-legend-safe = Ikke detekterbar
debrief-legend-close = Tæt på detektion
debrief-legend-detectable = Detekterbar
debrief-legend-tracked = Sporet / missilaffyring
debrief-legend-hit = Modtaget træffer / nedskydning
map-symbols = Symboler
map-legend-clouds = Skyer / tordentårne
map-legend-snow = Sne, let til kraftig
map-legend-rain = Regn, let til kraftig
map-legend-thunderstorm = Tordencelle
debrief-pause = PAUSE
debrief-play = AFSPIL
