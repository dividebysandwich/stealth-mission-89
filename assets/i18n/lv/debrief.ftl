# Misijas debrīfings (src/debrief.rs) un simulācijas pārskats (stealth-sim score.rs).

## Kā lidojums beidzās

outcome-landed = Droša nosēšanās
outcome-bad-landing = Neveiksmīga nosēšanās
outcome-ejected = Katapultējās - nolaidās ar izpletni
outcome-rescued = Katapultējās - izglābts ar CSAR
outcome-captured = Katapultējās - sagūstīts
outcome-crashed = Avarēja
outcome-ejection-failed = Katapultēšanās neizdevās
outcome-aborted = Misija pārtraukta
outcome-collision = Sadursme

## Vērtējumi

rating-unsatisfactory = Neapmierinoši
rating-marginal = Viduvēji
rating-satisfactory = Apmierinoši
rating-excellent = Teicami
rating-outstanding = Izcili

## Punkti

points-primary = Galvenais uzdevums
points-secondary = Papilduzdevums
points-aircraft = Iznīcinātas lidmašīnas
points-unseen = Notriekts nepamanīts
points-ground = Iznīcināti zemes mērķi
points-drones = Notriekti droni
points-drones-through = Cauri izlauzušies droni
points-nuclear = Trāpījums kodolrūpnīcai
points-detected = Atklāts
points-missile-hits = Raķešu trāpījumi

## Hronoloģija (vietu un lidmašīnu nosaukumi kā misijā)

timeline-primary = {" "}(galvenais)
timeline-secondary = {" "}(papild)
timeline-destroyed = Iznīcināts: { $place }{ $tag }
timeline-destroyed-ground = Iznīcināts zemes mērķis{ $tag }
timeline-shot-down = Notriekts: { $aircraft }{ $tag }
timeline-hit = Trāpījums no raķetes
timeline-seen = Pamanīja: { $aircraft }
timeline-radar-id = Radara identifikācija: { $place }
timeline-objective = Mērķis sasniegts{ $tag }
timeline-got-through = Izlauzās cauri: { $aircraft }
timeline-photo = Uzņemts foto{ $tag }

## Debrīfinga logs

debrief-title = MISIJAS DEBRĪFINGS
debrief-training = TRENIŅA MISIJA
debrief-results = REZULTĀTI
debrief-flight-map = LIDOJUMA KARTE
debrief-section-results = Rezultāti
debrief-accomplished = IZPILDĪTS
debrief-failed = NEIZPILDĪTS
debrief-primary = GALVENAIS
debrief-secondary = PAPILDU
debrief-aircraft = LIDMAŠĪNAS
debrief-drones = DRONI
debrief-drones-value = { $down } notriekti, { $through } cauri
debrief-ground = ZEME
debrief-weapons = BRUŅOJUMS
debrief-radar-ids = RADARA ID
debrief-visual-ids = VIZUĀLIE ID
debrief-hits-taken = SAŅEMTIE TRĀPĪJUMI
debrief-time = LAIKS
debrief-section-points = Punkti
debrief-difficulty = GRŪTĪBA
debrief-total = KOPĀ
debrief-section-timeline = Hronoloģija
debrief-no-events = Nav notikumu
debrief-section-exposure = Radaru ietekme
debrief-tracked = IZSEKOTS
debrief-closest = TUVĀKAIS
debrief-worst = SLIKTĀKAIS
debrief-at = Plkst. { $time }
debrief-altitude = AUGSTUMS
debrief-speed = ĀTRUMS
debrief-margin = REZERVE
debrief-strongest = STIPRĀKAIS
debrief-tracked-by = IZSEKO: { $sites }
debrief-section-map = Karte
debrief-legend-safe = Nav atklājams
debrief-legend-close = Tuvu atklāšanai
debrief-legend-detectable = Atklājams
debrief-legend-tracked = Izsekots / raķetes palaišana
debrief-legend-hit = Saņemts trāpījums / notriekts
map-symbols = Simboli
map-legend-clouds = Mākoņi / negaisa torņi
map-legend-snow = Sniegs, no vāja līdz stipram
map-legend-rain = Lietus, no vāja līdz stipram
map-legend-thunderstorm = Negaisa šūna
debrief-pause = PAUZE
debrief-play = ATSKAŅOT
