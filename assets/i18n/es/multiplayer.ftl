# La ventana de campaña, la ventana multijugador, la leyenda del mapa de guerra y la vista
# en directo.

delete = ELIMINAR
delete-campaign = ¿ELIMINAR ESTA CAMPAÑA?
delete-save = ¿ELIMINAR ESTA PARTIDA GUARDADA?
delete-permanent = La eliminación es definitiva

## Ventana de campaña

cm-group-campaign = Campaña
cm-weather-dynamic = Dinámica
    .facts =
        La meteorología cambia cada 10 a 22 horas del reloj del cielo
        Las nubes bloquean la línea de visión de ambos bandos
cm-weather-clear = Cielo despejado
    .facts = Sin nubes durante toda la campaña
cm-section-day-night = Día y noche
cm-day-real = Tiempo real
    .facts = Un día dura 24 horas
cm-day-2h = 2 h / día
    .facts = Un día dura 2 horas
cm-day-1h = 1 h / día
    .facts = Un día dura 1 hora
cm-day-30min = 30 min / día
    .facts = Un día dura 30 minutos
cm-conditions-note = Las condiciones se aplican a una campaña nueva
cm-rules-note = Una campaña guardada conserva las reglas con las que se inició
cm-about =
    La guerra transcurre en tiempo real mientras juega, también con el menú abierto.
    Vuele las misiones del tablón: reco, SEAD, CAP, CAS, antibuque, ataque, escolta.
    Las inserciones en helicóptero toman las ciudades y los aeródromos enemigos.
    Nunca muere ni es capturado: tras una salida, vuelve al tablón.
cm-section-new = Nueva campaña
cm-new-line = { $theater }  -  { $situation }  -  adversarios { $opponents }
cm-start = EMPEZAR
cm-section-continue = Continuar
cm-no-saves = No hay campañas guardadas
cm-played = { $hours } h jugadas
cm-save-gone = La partida guardada ya no existe
cm-save-unreadable = No se ha podido leer la partida guardada: { $error }
cm-save-undeletable = No se ha podido eliminar la partida guardada: { $error }
cm-in-vr = EN VR
cm-delete =
    .facts = Eliminar esta campaña guardada

## Mapa de guerra

pic-friendly-aircraft = Avión amigo
pic-friendly-helicopter = Helicóptero amigo
pic-enemy-fighter = Caza enemigo (en el radar)
pic-sam-known = Emplazamiento SAM (conocido)
pic-guns-known = Cañones / MANPADS (conocidos)
pic-reconned = Reconocido
pic-you = Usted
pic-site = Emplazamiento de { $place }
uv-live = { $unit } - EN DIRECTO
uv-contact-lost = CONTACTO PERDIDO
uv-hint = Arrastrar: orbitar  Rueda: zoom

## Multijugador

mp-disconnected = Desconectado: { $why }
mp-connecting = Conectando con { $address }...
mp-joining = Conectado con { $address }, entrando en la partida...
mp-browser-public-only = desde el navegador solo se puede entrar en los servidores de la lista pública
mp-title-host = MULTIJUGADOR: ALOJAR UNA PARTIDA
mp-subtitle = Cooperativo en la campaña dinámica: hagan la guerra juntos
mp-your-name = Su nombre
    .hint = Nombre del piloto
mp-join-address = Unirse por dirección
    .hint = dirección o nombre[:puerto]
mp-connect = CONECTAR
mp-server-name = Nombre del servidor
mp-udp-port = Puerto UDP
mp-address-note = Los demás jugadores se unen con la dirección de este ordenador; abra el puerto UDP en su router para jugar por internet
mp-list-public = Mostrar en la lista pública de servidores (los demás encuentran la partida en su lista)
mp-rules = Reglas para todos
mp-time-of-day = Hora del día (noches de luna)
mp-realistic-weapons = Armamento realista del F-117A
mp-friendly-fire = Fuego amigo
mp-day-night = Día y noche:
mp-weather = Meteorología:
mp-host-new = ALOJAR NUEVA
mp-host-saved = Alojar una campaña guardada
mp-host = ALOJAR
mp-host-game = ALOJAR UNA PARTIDA
mp-port-error = Puerto UDP { $port }: { $error }
mp-default-server = Campaña de { $pilot }
mp-public-servers = Servidores públicos
mp-refresh = ACTUALIZAR
mp-server-count = { $servers } servidores, { $pilots } pilotos
mp-list-error = No se puede acceder a la lista de servidores: { $error }
mp-no-servers = Ahora mismo no hay servidores públicos
mp-col-server = SERVIDOR
mp-col-pilots = PILOTOS
mp-col-war = GUERRA
mp-dedicated = DEDICADO
mp-other-version = otra versión
mp-join = UNIRSE
