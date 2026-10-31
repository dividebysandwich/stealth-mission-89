# La tablilla de rodilla (fuente vectorial), el planificador de misión y las misiones guardadas.
# Las abreviaturas de navegación (PT, BRG, ETA, PRI, SEC, KM, KT) quedan en inglés.

kb-brief = BRIEF
kb-route = RUTA
kb-map = MAPA
kb-target = BLANCO
kb-keys = TECLAS
kb-tip = Mapas, notas de la misión y guía de teclas
kb-geran-file = GERAN-2 (EXPEDIENTE)
kb-objectives = OBJETIVOS
kb-primary = PRINCIPAL: { $objective }
kb-secondary = SECUNDARIO: { $objective }
kb-times = HORARIOS
kb-deadline = LÍMITE { $time }
kb-now = AHORA { $time }
kb-conditions = CONDICIONES
kb-moon = LUNA: { $phase } ({ $lit } % ILUMINADA)
kb-armament = ARMAMENTO
kb-roe = REGLAS DE ENFRENTAMIENTO
kb-total = TOTAL { $km } KM A { $kt } KT
kb-fuel = COMBUSTIBLE { $lbs } LB
kb-preparing = PREPARANDO...
kb-airborne = EN VUELO
kb-developing = REVELANDO...
kb-no-photo = SIN FOTO
kb-airborne-target = BLANCO EN VUELO
kb-scroll-vr = DESPLAZAR: STICK, DEDO SOBRE LA TABLILLA
kb-scroll-desktop = DESPLAZAR: RUEDA DEL RATÓN SOBRE LA TABLILLA

## Planificador de misión

planner-low = BAJO 500 FT
planner-medium = MEDIO 5000 FT
planner-high = ALTO 20 000 FT
planner-title = PLANIFICADOR DE MISIÓN
planner-route = Ruta
planner-base = BASE
planner-total = TOTAL { $km } KM  { $time }
planner-briefed = Ruta del briefing { $km } km, combustible { $fuel } lb
planner-coverage = Cobertura radar para
    .note = La cobertura radar supone vuelo nivelado a 480 kt, con las compuertas de las bodegas cerradas.
planner-detected = { $km } km de la ruta detectados
planner-pulse = Alcance de detección del radar de impulsos (P)
planner-doppler = Alcance de detección del radar Doppler (D)
planner-missile = Alcance de los misiles
planner-uav = Dron enemigo
planner-edit-hint = Arrastre un punto para moverlo y el mapa para desplazarlo. Haga clic en un tramo para añadir un punto, clic derecho para quitarlo. Rueda: zoom.
planner-reset = RESTABLECER
planner-view-hint = Arrastre el mapa para desplazarlo. Rueda: zoom.
planner-layers = CAPAS
planner-layer-sams = Radar / emplazamiento SAM
planner-radar-types = Tipos de radar
planner-layer-pulse = Alcance de detección del radar de impulsos
planner-layer-doppler = Alcance de detección del radar Doppler
planner-layer-airfield = Aeródromo

## Misiones guardadas

saves-title = MISIONES GUARDADAS
saves-subtitle = Guardadas en vuelo desde el menú: GUARDAR MISIÓN
saves-none = No hay misiones guardadas
saves-line = { $theater }  -  { $aircraft }  -  tiempo de misión { $time }
saves-saved-at = Guardada el { $date }
saves-unreadable = Partida guardada ilegible (otra versión)
saves-delete =
    .facts = Eliminar esta partida guardada
saves-back =
    .facts = Volver a la pantalla de misión individual
