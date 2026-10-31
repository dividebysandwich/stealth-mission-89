# A küldetés kiértékelése (src/debrief.rs) és a szimuláció jelentése (stealth-sim score.rs).

## Hogyan ért véget a repülés

outcome-landed = Biztonságos leszállás
outcome-bad-landing = Rossz leszállás
outcome-ejected = Katapultált - ejtőernyővel leszállt
outcome-rescued = Katapultált - CSAR kimentette
outcome-captured = Katapultált - fogságba esett
outcome-crashed = Lezuhant
outcome-ejection-failed = A katapultálás nem sikerült
outcome-aborted = Küldetés megszakítva
outcome-collision = Ütközés

## Értékelések

rating-unsatisfactory = Elégtelen
rating-marginal = Gyenge
rating-satisfactory = Megfelelő
rating-excellent = Kiváló
rating-outstanding = Kimagasló

## Pontok

points-primary = Elsődleges cél
points-secondary = Másodlagos cél
points-aircraft = Megsemmisített gépek
points-unseen = Észrevétlenül lelőve
points-ground = Megsemmisített földi célpontok
points-drones = Lelőtt drónok
points-drones-through = Átjutott drónok
points-nuclear = Atomerőmű eltalálva
points-detected = Észlelve
points-missile-hits = Rakétatalálatok

## Idővonal (a hely- és gépnevek a küldetésből)

timeline-primary = {" "}(elsődleges)
timeline-secondary = {" "}(másodlagos)
timeline-destroyed = Megsemmisítve: { $place }{ $tag }
timeline-destroyed-ground = Földi célpont megsemmisítve{ $tag }
timeline-shot-down = Lelőve: { $aircraft }{ $tag }
timeline-hit = Rakéta eltalálta
timeline-seen = Észlelte: { $aircraft }
timeline-radar-id = Radaros azonosítás: { $place }
timeline-objective = Cél elérve{ $tag }
timeline-got-through = Átjutott: { $aircraft }
timeline-photo = Fénykép elkészítve{ $tag }

## A kiértékelő ablak

debrief-title = KÜLDETÉS-KIÉRTÉKELÉS
debrief-training = GYAKORLÓ KÜLDETÉS
debrief-results = EREDMÉNYEK
debrief-flight-map = REPÜLÉSI TÉRKÉP
debrief-section-results = Eredmények
debrief-accomplished = TELJESÍTVE
debrief-failed = SIKERTELEN
debrief-primary = ELSŐDLEGES
debrief-secondary = MÁSODLAGOS
debrief-aircraft = GÉPEK
debrief-drones = DRÓNOK
debrief-drones-value = { $down } lelőve, { $through } átjutott
debrief-ground = FÖLDI
debrief-weapons = FEGYVEREK
debrief-radar-ids = RADAROS AZON.
debrief-visual-ids = VIZUÁLIS AZON.
debrief-hits-taken = KAPOTT TALÁLATOK
debrief-time = IDŐ
debrief-section-points = Pontok
debrief-difficulty = NEHÉZSÉG
debrief-total = ÖSSZESEN
debrief-section-timeline = Idővonal
debrief-no-events = Nincs esemény
debrief-section-exposure = Radarkitettség
debrief-tracked = KÖVETVE
debrief-closest = LEGKÖZELEBB
debrief-worst = LEGROSSZABB
debrief-at = Ekkor: { $time }
debrief-altitude = MAGASSÁG
debrief-speed = SEBESSÉG
debrief-margin = TARTALÉK
debrief-strongest = LEGERŐSEBB
debrief-tracked-by = KÖVETI: { $sites }
debrief-section-map = Térkép
debrief-legend-safe = Nem észlelhető
debrief-legend-close = Az észlelés közelében
debrief-legend-detectable = Észlelhető
debrief-legend-tracked = Követve / rakétaindítás
debrief-legend-hit = Kapott találat / lelövés
map-symbols = Jelek
map-legend-clouds = Felhők / zivatartornyok
map-legend-snow = Hó, gyengétől erősig
map-legend-rain = Eső, gyengétől erősig
map-legend-thunderstorm = Zivatarcella
debrief-pause = SZÜNET
debrief-play = LEJÁTSZÁS
