# La pantalla de operaciones de la campaña (src/campaign/ops.rs).

ops-thunderstorms = TORMENTAS
ops-wx-short = METEO { $now }, { $next } A LAS { $at }
ops-wx-line = { $weather } Cambio a { $next } a las { $at }.
ops-clock = DÍA { $day }  { $time }
ops-landed-at = ATERRIZADO EN { $base }
ops-ground-crew = El personal de tierra está listo
ops-rearm = REARMAR Y REPOSTAR
ops-end-sortie = TERMINAR LA SALIDA
ops-on-server = EN { $server }
ops-serving = SERVIDOR
ops-tab-missions = MISIONES
ops-tab-map = MAPA DE GUERRA
ops-tab-air-defence = DEFENSA AÉREA
ops-tab-forces = FUERZAS
ops-tab-players = JUGADORES
ops-tab-debrief = DEBRIEFING
ops-tab-log = REGISTRO
ops-tab-chat = CHAT
ops-blue = AZUL { $count }
ops-red = ROJO { $count }
ops-war-over = La guerra ha terminado
ops-running = Simulación de la guerra en curso
ops-save-exit = GUARDAR Y SALIR
ops-disconnect = DESCONECTAR
ops-leave-local = ¿GUARDAR Y ABANDONAR LA CAMPAÑA?
    .text = La guerra se detiene hasta que la reanude. Los aviones en vuelo se pierden; las inserciones en curso vuelven a empezar desde su base.
ops-leave-host = ¿GUARDAR Y DEJAR DE SERVIR LA CAMPAÑA?
    .text = Los jugadores conectados se desconectan y la guerra se detiene hasta que la reanude. Los aviones en vuelo se pierden; las inserciones en curso vuelven a empezar desde su base.
ops-leave-client = ¿DESCONECTARSE DEL SERVIDOR?
    .text = La campaña continúa en el servidor; aquí no se guarda nada. Su salida termina.
ops-unit-destroyed = Destruido: { $unit }. Vuelta a operaciones

## Misiones

ops-section-missions = Misiones
ops-threat = AMENAZA { $threat }
ops-no-missions = No hay misiones: el frente está en calma
ops-section-underway = Inserciones e incursiones
ops-enemy-liftoff = ENEMIGO en { $place }: despegue en { $mins } min
ops-liftoff = { $place }: despegue en { $mins } min
ops-enemy-helis = ENEMIGO en { $place }: { $count } helicópteros aproximándose
ops-helis = { $place }: { $count } helicópteros en camino
ops-raid-sead = SEAD de la IA en { $place }
ops-raid-drones = DRONES sobre { $place }
ops-raid = INCURSIÓN contra el aeródromo de { $place }
ops-raid-launch = { $raid }: lanzamiento en { $mins } min
ops-raid-on-way = { $raid }: { $count } en camino
ops-resupply-ai = REABASTECIMIENTO ({ $unit }): UH-60 en camino
ops-awacs = AWACS ENEMIGO en vuelo { $mins } min más
ops-down = DERRIBADO: { $pilot } - rescate en los próximos { $mins } min
ops-aboard = A BORDO: { $pilot } - de regreso
ops-none-underway = Nada en curso
ops-ai-helicopter = Un helicóptero de la IA se encarga en { $time } si ningún piloto de UH-60 lo hace
ops-defences = Defensas: { $sams } emplazamientos SAM, amenaza { $threat }
ops-recon-intel = INTELIGENCIA DE RECO
ops-order-insertion = ORDENAR LA INSERCIÓN
    .facts = Vuele la escolta para cubrirlos, o diríjalos en el UH-60
ops-from = Desde
ops-planned-by-group = Planificada por el grupo
ops-stores = { $aircraft } - cargas: { $stores }
ops-fly-uh60 = VOLAR EL UH-60
ops-accept = ACEPTAR LA MISIÓN
ops-planning = PLANIFICANDO: { $pilots }
ops-flying = EN VUELO: { $pilots }
ops-package = PAQUETE con { $kinds }: cumpla dos tipos aquí en { $mins } min, +{ $bonus } puntos cada uno
ops-sead-on-way = La SEAD de la IA está en camino
ops-request-sead = SOLICITAR SEAD DE LA IA ({ $cost } de material, { $aircraft } aviones)
    .facts = Aviones de ataque amigos atacan un emplazamiento SAM de este lugar antes que usted
    .disabled = No hay suficiente material o aviones en tierra (ver FUERZAS)

## Reglas, material, fuerzas

ops-rule-damage = daños realistas
ops-rule-guidance = guiado realista
ops-rule-no-crashes = protección contra colisiones
ops-rule-time = tiempo continuo
ops-rule-weapons = armamento realista del F-117A
ops-rule-friendly-fire = fuego amigo
ops-rule-classic = clásicas
ops-materiel = MATERIAL
ops-materiel-blue = AZUL { $materiel } (+{ $income }/h)
ops-materiel-red = ROJO { $materiel } (+{ $income }/h)
ops-friendly = AMIGOS
ops-enemy = ENEMIGOS
ops-section-forces = Fuerzas
ops-places-held = Lugares controlados
ops-materiel-row = Material
ops-income = Ingresos por hora
ops-income-from = de ellos
ops-income-sources = { $base } + { $places } lugares x { $per_place } + { $cities } ciudades x { $per_city }
ops-fighters-ground = Cazas en tierra
ops-helicopters-ground = Helicópteros en tierra
ops-in-air = En vuelo
ops-in-air-value = { $fighters } cazas, { $strike } de ataque, { $helicopters } helicópteros
ops-since-start = Desde el inicio
ops-earned = ganado
ops-repairs = reparaciones
ops-operations = salidas, incursiones, inserciones
ops-new-aircraft = nuevos aviones
ops-lost = pérdidas (ciudades, pilotos)
ops-section-how = Funcionamiento
ops-how =
    Cada bando gana material cada hora: una cantidad fija, más por cada lugar que controla y por cada ciudad intacta entre ellos. Bombardee las ciudades enemigas (INTERDICCIÓN) para reducir sus ingresos.
    La IA paga todo lo que hace: reparaciones, cada salida de caza, incursiones y sus propias inserciones. Sin material, sus daños quedan sin reparar y sus aviones se quedan en tierra.
    La IA solo vuela los aviones que tiene; derribados, se pierden; aterrizados, vuelven a volar. Cada hora, cada bando compra otros nuevos mientras le dure el material.
    A usted nunca le falta nada: sus aviones, sus bases (bombardeadas o no) y los helicópteros de sus propias órdenes de inserción siempre están ahí.
ops-section-prices = Precios
ops-price-airfield = Reparación de aeródromo
ops-price-sam = Reparación de emplazamiento SAM
ops-price-city = Ciudad reconstruida
ops-price-ship = Reparación de buque
ops-price-gun = Cañón o equipo MANPADS
ops-price-sortie = Salida de caza
ops-price-raid = Incursión
ops-price-insertion = Inserción de la IA
ops-price-fighter = Nuevo caza
ops-price-helicopter = Nuevo helicóptero
ops-price-city-lost = Ciudad destruida (pérdida)
ops-price-captured = Piloto capturado (pérdida)

## Mapa de guerra

ops-click-to-watch = { $unit } - clic para seguir
ops-section-legend = Leyenda
ops-legend-friendly-airfield = Aeródromo amigo
ops-legend-enemy-airfield = Aeródromo enemigo
ops-legend-city = Ciudad amiga / enemiga
ops-legend-airfield = Aeródromo propio / enemigo
ops-legend-insertion = Ruta de inserción
ops-watch-hint = Haga clic en una unidad para seguirla en directo
ops-map-hint = Rueda: zoom, arrastrar: desplazar
ops-patriot = Batería Patriot
ops-drone-center = Centro de mando de drones
ops-manned-by = Operada por { $pilot }
ops-manned-by-other = Operada por otro jugador
ops-control = TOMAR EL CONTROL
    .facts = Ocupar su consola

## Debriefing, final, registro, chat

ops-no-sortie = Todavía no se ha volado ninguna salida
ops-section-sortie = Salida
ops-achieved = OBJETIVO CUMPLIDO
ops-not-achieved = OBJETIVO NO CUMPLIDO
ops-waiting-raid = A LA ESPERA DE LA INCURSIÓN
ops-waiting-insertion = A LA ESPERA DE LA INSERCIÓN
ops-flight-time = Tiempo de vuelo { $mins } min
ops-kills = Derribos: { $air } aviones, { $ground } en tierra
ops-points = { $points } puntos
ops-points-bonus = { $points } puntos (bonificación de paquete +{ $bonus })
ops-campaign-points = { $rank } - { $points } puntos de campaña
ops-campaign-record = Campaña: { $sorties } salidas, { $won } objetivos cumplidos
ops-section-meanwhile = Mientras tanto
ops-victory = VICTORIA
    .text = El enemigo ya no controla ningún aeródromo: el teatro es nuestro.
ops-defeat = DERROTA
    .text = No queda ningún aeródromo amigo: el teatro está perdido.
ops-end-day = Día { $day }. Lugares amigos { $friendly }, lugares enemigos { $enemy }.
ops-next-campaign = LA PRÓXIMA CAMPAÑA EMPIEZA EN { $time }
ops-next-campaign-now = LA PRÓXIMA CAMPAÑA ESTÁ EMPEZANDO
ops-next-from-menu = Inicie la próxima campaña desde el menú principal.
ops-section-log = Registro de campaña
ops-section-chat = Chat
ops-no-messages = Todavía no hay mensajes
ops-message = Mensaje
    .hint = Escriba un mensaje; Intro lo envía
ops-send = ENVIAR

## Briefing

ops-briefing = BRIEFING DE MISIÓN
ops-briefing-line = Desde { $base } - { $km } km hasta el objetivo - { $aircraft } - cargas: { $stores }
ops-route-locked = Ruta bloqueada: { $pilots } la está volando
ops-route-threats = RUTA Y AMENAZAS
ops-take-off = DESPEGAR
ops-radars-shown = Radares mostrados: detectados por aviones amigos, aún en pie

## Jugadores

ops-section-players = Jugadores
ops-col-name = NOMBRE
ops-col-status = ESTADO
ops-col-sorties = SALIDAS
ops-col-points = PUNTOS
ops-col-pilot = PILOTO
ops-col-objectives = OBJETIVOS
ops-col-kills = DERRIBOS AIRE/TIERRA
ops-col-medal = MEJOR MEDALLA
ops-you = (usted)
ops-host = ANFITRIÓN
ops-status-planning = Planificando: { $title }
ops-status-board = En el tablón de misiones
ops-status-flying = En vuelo: { $title } - { $aircraft }
ops-status-reconnecting = Conexión perdida - avión mantenido { $seconds } s
ops-flown = { $flown } ({ $won } cumplidas)
ops-section-leaderboard = Clasificación
ops-alone = En solitario. Para volar juntos, aloje esta campaña desde MULTIJUGADOR en el menú principal.
ops-rules = Reglas de esta campaña (fijadas al inicio): { $rules }
ops-no-sorties = Todavía no se ha volado ninguna salida
