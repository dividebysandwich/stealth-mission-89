# Il debriefing della missione (src/debrief.rs) e il rapporto della simulazione.

## Come si è concluso il volo

outcome-landed = Atterraggio riuscito
outcome-bad-landing = Atterraggio mancato
outcome-ejected = Eiettato - atterrato col paracadute
outcome-rescued = Eiettato - recuperato dal CSAR
outcome-captured = Eiettato - catturato
outcome-crashed = Precipitato
outcome-ejection-failed = Eiezione fallita
outcome-aborted = Missione interrotta
outcome-collision = Collisione

## Valutazioni

rating-unsatisfactory = Insufficiente
rating-marginal = Mediocre
rating-satisfactory = Soddisfacente
rating-excellent = Eccellente
rating-outstanding = Eccezionale

## Punti

points-primary = Obiettivo primario
points-secondary = Obiettivo secondario
points-aircraft = Aerei distrutti
points-unseen = Abbattuti senza essere visto
points-ground = Obiettivi a terra distrutti
points-drones = Droni abbattuti
points-drones-through = Droni passati
points-nuclear = Centrale nucleare colpita
points-detected = Rilevato
points-missile-hits = Colpito da missili

## Cronologia (nomi di luoghi e aerei come nella missione)

timeline-primary = {" "}(primario)
timeline-secondary = {" "}(secondario)
timeline-destroyed = { $place } distrutto{ $tag }
timeline-destroyed-ground = Obiettivo a terra distrutto{ $tag }
timeline-shot-down = { $aircraft } abbattuto{ $tag }
timeline-hit = Colpito da un missile
timeline-seen = Avvistato da un { $aircraft }
timeline-radar-id = Identificato al radar da { $place }
timeline-objective = Obiettivo raggiunto{ $tag }
timeline-got-through = { $aircraft } è passato
timeline-photo = Foto scattata{ $tag }

## La finestra del debriefing

debrief-title = DEBRIEFING DELLA MISSIONE
debrief-training = MISSIONE DI ADDESTRAMENTO
debrief-results = RISULTATI
debrief-flight-map = MAPPA DEL VOLO
debrief-section-results = Risultati
debrief-accomplished = COMPIUTO
debrief-failed = FALLITO
debrief-primary = PRIMARIO
debrief-secondary = SECONDARIO
debrief-aircraft = AEREI
debrief-drones = DRONI
debrief-drones-value = { $down } abbattuti, { $through } passati
debrief-ground = A TERRA
debrief-weapons = ARMI SGANCIATE
debrief-radar-ids = ID RADAR
debrief-visual-ids = ID VISIVE
debrief-hits-taken = COLPI SUBITI
debrief-time = DURATA
debrief-section-points = Punti
debrief-difficulty = DIFFICOLTÀ
debrief-total = TOTALE
debrief-section-timeline = Cronologia
debrief-no-events = Nessun evento
debrief-section-exposure = Esposizione radar
debrief-tracked = TRACCIATO
debrief-closest = DISTANZA MINIMA
debrief-worst = CASO PEGGIORE
debrief-at = Alle { $time }
debrief-altitude = QUOTA
debrief-speed = VELOCITÀ
debrief-margin = MARGINE
debrief-strongest = PIÙ INTENSO
debrief-tracked-by = TRACCIATO DA { $sites }
debrief-section-map = Mappa
debrief-legend-safe = Non rilevabile
debrief-legend-close = Vicino al rilevamento
debrief-legend-detectable = Rilevabile
debrief-legend-tracked = Tracciato / lancio missile
debrief-legend-hit = Colpo subito / abbattimento
debrief-pause = PAUSA
debrief-play = RIPRODURRE
