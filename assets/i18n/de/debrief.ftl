# Die Missionsnachbesprechung (src/debrief.rs) und der Bericht der Simulation (stealth-sim score.rs).

## Wie ein Flug endete

outcome-landed = Sichere Landung
outcome-bad-landing = Missglückte Landung
outcome-ejected = Ausgestiegen - am Fallschirm gelandet
outcome-rescued = Ausgestiegen - durch CSAR gerettet
outcome-captured = Ausgestiegen - gefangen genommen
outcome-crashed = Abgestürzt
outcome-ejection-failed = Ausstieg misslungen
outcome-aborted = Mission abgebrochen
outcome-collision = Kollision

## Bewertungen

rating-unsatisfactory = Ungenügend
rating-marginal = Mangelhaft
rating-satisfactory = Befriedigend
rating-excellent = Ausgezeichnet
rating-outstanding = Hervorragend

## Punkte

points-primary = Primärziel
points-secondary = Sekundärziel
points-aircraft = Flugzeuge zerstört
points-unseen = Unbemerkt abgeschossen
points-ground = Bodenziele zerstört
points-drones = Drohnen abgeschossen
points-drones-through = Drohnen durchgekommen
points-nuclear = Kernkraftwerk getroffen
points-detected = Entdeckt
points-missile-hits = Raketentreffer

## Zeitleiste (Orts- und Flugzeugnamen wie in der Mission)

timeline-primary = {" "}(primär)
timeline-secondary = {" "}(sekundär)
timeline-destroyed = { $place } zerstört{ $tag }
timeline-destroyed-ground = Bodenziel zerstört{ $tag }
timeline-shot-down = { $aircraft } abgeschossen{ $tag }
timeline-hit = Von einer Rakete getroffen
timeline-seen = Gesehen von: { $aircraft }
timeline-radar-id = Radar-ID durch { $place }
timeline-objective = Ziel erreicht{ $tag }
timeline-got-through = { $aircraft } durchgekommen
timeline-photo = Foto aufgenommen{ $tag }

## Das Nachbesprechungsfenster

debrief-title = MISSIONSNACHBESPRECHUNG
debrief-training = ÜBUNGSMISSION
debrief-results = ERGEBNISSE
debrief-flight-map = FLUGKARTE
debrief-section-results = Ergebnisse
debrief-accomplished = ERFÜLLT
debrief-failed = GESCHEITERT
debrief-primary = PRIMÄR
debrief-secondary = SEKUNDÄR
debrief-aircraft = FLUGZEUGE
debrief-drones = DROHNEN
debrief-drones-value = { $down } abgeschossen, { $through } durchgekommen
debrief-ground = BODEN
debrief-weapons = WAFFEN
debrief-radar-ids = RADAR-IDS
debrief-visual-ids = SICHT-IDS
debrief-hits-taken = ERLITTENE TREFFER
debrief-time = ZEIT
debrief-section-points = Punkte
debrief-difficulty = SCHWIERIGKEIT
debrief-total = GESAMT
debrief-section-timeline = Zeitleiste
debrief-no-events = Keine Ereignisse
debrief-section-exposure = Radarerfassung
debrief-tracked = VERFOLGT
debrief-closest = AM NÄCHSTEN
debrief-worst = AM SCHLIMMSTEN
debrief-at = Um { $time }
debrief-altitude = HÖHE
debrief-speed = FAHRT
debrief-margin = ABSTAND
debrief-strongest = AM STÄRKSTEN
debrief-tracked-by = VERFOLGT VON { $sites }
debrief-section-map = Karte
debrief-legend-safe = Nicht erfassbar
debrief-legend-close = Nahe an der Erfassung
debrief-legend-detectable = Erfassbar
debrief-legend-tracked = Verfolgt / Raketenstart
debrief-legend-hit = Treffer erhalten / Abschuss
map-symbols = Symbole
map-legend-clouds = Wolken / Gewittertürme
map-legend-snow = Schnee, leicht bis stark
map-legend-rain = Regen, leicht bis stark
map-legend-thunderstorm = Gewitterzelle
debrief-pause = PAUSE
debrief-play = ABSPIELEN
