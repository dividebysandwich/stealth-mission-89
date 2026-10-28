# El debriefing de la misión (src/debrief.rs) y el informe de la simulación.

## Cómo terminó el vuelo

outcome-landed = Aterrizaje correcto
outcome-bad-landing = Mal aterrizaje
outcome-ejected = Eyectado - aterrizaje en paracaídas
outcome-rescued = Eyectado - rescatado por el CSAR
outcome-captured = Eyectado - capturado
outcome-crashed = Estrellado
outcome-ejection-failed = Eyección fallida
outcome-aborted = Misión abortada
outcome-collision = Colisión

## Calificaciones

rating-unsatisfactory = Insatisfactorio
rating-marginal = Deficiente
rating-satisfactory = Satisfactorio
rating-excellent = Excelente
rating-outstanding = Sobresaliente

## Puntos

points-primary = Objetivo principal
points-secondary = Objetivo secundario
points-aircraft = Aviones destruidos
points-unseen = Derribos sin ser visto
points-ground = Objetivos terrestres destruidos
points-drones = Drones derribados
points-drones-through = Drones que pasaron
points-nuclear = Central nuclear alcanzada
points-detected = Detectado
points-missile-hits = Impactos de misil

## Cronología (nombres de lugares y aviones tal como en la misión)

timeline-primary = {" "}(principal)
timeline-secondary = {" "}(secundario)
timeline-destroyed = Destruido: { $place }{ $tag }
timeline-destroyed-ground = Objetivo terrestre destruido{ $tag }
timeline-shot-down = Derribado: { $aircraft }{ $tag }
timeline-hit = Alcanzado por un misil
timeline-seen = Visto por un { $aircraft }
timeline-radar-id = Identificado por radar desde { $place }
timeline-objective = Objetivo alcanzado{ $tag }
timeline-got-through = { $aircraft } ha pasado
timeline-photo = Foto tomada{ $tag }

## La ventana del debriefing

debrief-title = DEBRIEFING DE LA MISIÓN
debrief-training = MISIÓN DE ENTRENAMIENTO
debrief-results = RESULTADOS
debrief-flight-map = MAPA DEL VUELO
debrief-section-results = Resultados
debrief-accomplished = CUMPLIDO
debrief-failed = FALLIDO
debrief-primary = PRINCIPAL
debrief-secondary = SECUNDARIO
debrief-aircraft = AVIONES
debrief-drones = DRONES
debrief-drones-value = { $down } derribados, { $through } pasaron
debrief-ground = EN TIERRA
debrief-weapons = ARMAS DISPARADAS
debrief-radar-ids = ID POR RADAR
debrief-visual-ids = ID VISUALES
debrief-hits-taken = IMPACTOS RECIBIDOS
debrief-time = DURACIÓN
debrief-section-points = Puntos
debrief-difficulty = DIFICULTAD
debrief-total = TOTAL
debrief-section-timeline = Cronología
debrief-no-events = Ningún evento
debrief-section-exposure = Exposición al radar
debrief-tracked = SEGUIDO
debrief-closest = MÁS CERCA
debrief-worst = PEOR MOMENTO
debrief-at = A las { $time }
debrief-altitude = ALTITUD
debrief-speed = VELOCIDAD
debrief-margin = MARGEN
debrief-strongest = MÁS INTENSO
debrief-tracked-by = SEGUIDO POR { $sites }
debrief-section-map = Mapa
debrief-legend-safe = No detectable
debrief-legend-close = Cerca de la detección
debrief-legend-detectable = Detectable
debrief-legend-tracked = Seguido / lanzamiento de misil
debrief-legend-hit = Impacto recibido / derribo
debrief-pause = PAUSA
debrief-play = REPRODUCIR
