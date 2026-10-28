# Le débriefing de mission (src/debrief.rs) et le rapport de la simulation.

## Issue du vol

outcome-landed = Atterrissage réussi
outcome-bad-landing = Atterrissage raté
outcome-ejected = Éjecté - posé en parachute
outcome-rescued = Éjecté - récupéré par le CSAR
outcome-captured = Éjecté - capturé
outcome-crashed = Écrasé
outcome-ejection-failed = Éjection ratée
outcome-aborted = Mission abandonnée
outcome-collision = Collision

## Appréciations

rating-unsatisfactory = Insuffisant
rating-marginal = Médiocre
rating-satisfactory = Satisfaisant
rating-excellent = Excellent
rating-outstanding = Exceptionnel

## Points

points-primary = Objectif principal
points-secondary = Objectif secondaire
points-aircraft = Appareils détruits
points-unseen = Abattus sans être vu
points-ground = Objectifs au sol détruits
points-drones = Drones abattus
points-drones-through = Drones passés
points-nuclear = Centrale nucléaire touchée
points-detected = Détecté
points-missile-hits = Touché par des missiles

## Chronologie (noms de lieux et d'appareils tels que dans la mission)

timeline-primary = {" "}(principal)
timeline-secondary = {" "}(secondaire)
timeline-destroyed = { $place } détruit{ $tag }
timeline-destroyed-ground = Objectif au sol détruit{ $tag }
timeline-shot-down = { $aircraft } abattu{ $tag }
timeline-hit = Touché par un missile
timeline-seen = Repéré par un { $aircraft }
timeline-radar-id = Identifié au radar par { $place }
timeline-objective = Objectif atteint{ $tag }
timeline-got-through = { $aircraft } est passé
timeline-photo = Photo prise{ $tag }

## La fenêtre de débriefing

debrief-title = DÉBRIEFING DE MISSION
debrief-training = MISSION D'ENTRAÎNEMENT
debrief-results = RÉSULTATS
debrief-flight-map = CARTE DU VOL
debrief-section-results = Résultats
debrief-accomplished = ACCOMPLI
debrief-failed = ÉCHOUÉ
debrief-primary = PRINCIPAL
debrief-secondary = SECONDAIRE
debrief-aircraft = APPAREILS
debrief-drones = DRONES
debrief-drones-value = { $down } abattus, { $through } passés
debrief-ground = AU SOL
debrief-weapons = ARMES TIRÉES
debrief-radar-ids = ID RADAR
debrief-visual-ids = ID VISUELLES
debrief-hits-taken = IMPACTS REÇUS
debrief-time = DURÉE
debrief-section-points = Points
debrief-difficulty = DIFFICULTÉ
debrief-total = TOTAL
debrief-section-timeline = Chronologie
debrief-no-events = Aucun événement
debrief-section-exposure = Exposition radar
debrief-tracked = POURSUIVI
debrief-closest = AU PLUS PRÈS
debrief-worst = LE PIRE
debrief-at = À { $time }
debrief-altitude = ALTITUDE
debrief-speed = VITESSE
debrief-margin = MARGE
debrief-strongest = LE PLUS FORT
debrief-tracked-by = POURSUIVI PAR { $sites }
debrief-section-map = Carte
debrief-legend-safe = Indétectable
debrief-legend-close = Proche de la détection
debrief-legend-detectable = Détectable
debrief-legend-tracked = Poursuivi / tir de missile
debrief-legend-hit = Impact reçu / victoire
debrief-pause = PAUSE
debrief-play = LECTURE
