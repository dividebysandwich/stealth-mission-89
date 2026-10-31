# Missiooni debriifing (src/debrief.rs) ja simulatsiooni aruanne (stealth-sim score.rs).

## Kuidas lend lõppes

outcome-landed = Ohutu maandumine
outcome-bad-landing = Ebaõnnestunud maandumine
outcome-ejected = Katapulteerus - maandus varjuga
outcome-rescued = Katapulteerus - päästetud CSAR-iga
outcome-captured = Katapulteerus - vangistati
outcome-crashed = Kukkus alla
outcome-ejection-failed = Katapulteerumine ebaõnnestus
outcome-aborted = Missioon katkestatud
outcome-collision = Kokkupõrge

## Hinnangud

rating-unsatisfactory = Rahuldamatu
rating-marginal = Piiripealne
rating-satisfactory = Rahuldav
rating-excellent = Suurepärane
rating-outstanding = Silmapaistev

## Punktid

points-primary = Põhieesmärk
points-secondary = Teisene eesmärk
points-aircraft = Hävitatud õhusõidukid
points-unseen = Märkamatult alla tulistatud
points-ground = Hävitatud maasihtmärgid
points-drones = Alla tulistatud droonid
points-drones-through = Läbi pääsenud droonid
points-nuclear = Tuumajaam tabatud
points-detected = Avastatud
points-missile-hits = Raketitabamused

## Ajajoon (kohtade ja õhusõidukite nimed nagu missioonil)

timeline-primary = {" "}(põhi)
timeline-secondary = {" "}(teisene)
timeline-destroyed = Hävitatud: { $place }{ $tag }
timeline-destroyed-ground = Hävitatud maasihtmärk{ $tag }
timeline-shot-down = Alla tulistatud: { $aircraft }{ $tag }
timeline-hit = Tabas rakett
timeline-seen = Märkas: { $aircraft }
timeline-radar-id = Radari ID: { $place }
timeline-objective = Eesmärk saavutatud{ $tag }
timeline-got-through = { $aircraft } pääses läbi
timeline-photo = Foto tehtud{ $tag }

## Debriifingu aken

debrief-title = MISSIOONI DEBRIIFING
debrief-training = TREENINGMISSIOON
debrief-results = TULEMUSED
debrief-flight-map = LENNUKAART
debrief-section-results = Tulemused
debrief-accomplished = TÄIDETUD
debrief-failed = EBAÕNNESTUS
debrief-primary = PÕHI
debrief-secondary = TEISENE
debrief-aircraft = ÕHUSÕIDUKID
debrief-drones = DROONID
debrief-drones-value = { $down } alla, { $through } läbi
debrief-ground = MAAL
debrief-weapons = RELVAD
debrief-radar-ids = RADARI ID-D
debrief-visual-ids = VISUAALSED ID-D
debrief-hits-taken = SAADUD TABAMUSI
debrief-time = AEG
debrief-section-points = Punktid
debrief-difficulty = RASKUSASTE
debrief-total = KOKKU
debrief-section-timeline = Ajajoon
debrief-no-events = Sündmusi pole
debrief-section-exposure = Radarikiiritus
debrief-tracked = JÄLGITUD
debrief-closest = LÄHIM
debrief-worst = HALVIM
debrief-at = Kell { $time }
debrief-altitude = KÕRGUS
debrief-speed = KIIRUS
debrief-margin = VARU
debrief-strongest = TUGEVAIM
debrief-tracked-by = JÄLGIJA: { $sites }
debrief-section-map = Kaart
debrief-legend-safe = Avastamatu
debrief-legend-close = Avastamise lähedal
debrief-legend-detectable = Avastatav
debrief-legend-tracked = Jälgitud / raketi väljalase
debrief-legend-hit = Saadud tabamus / hävitamine
map-symbols = Sümbolid
map-legend-clouds = Pilved / äiksetornid
map-legend-snow = Lumi, nõrgast tugevani
map-legend-rain = Vihm, nõrgast tugevani
map-legend-thunderstorm = Äiksekolle
debrief-pause = PAUS
debrief-play = MÄNGI
