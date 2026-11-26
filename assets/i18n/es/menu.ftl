# El menú principal y la pantalla de misión individual (src/menu.rs).

## Teatros de operaciones

theater-libya = Libia (1986)
    .facts =
        Golfo de Sidra y costa libia
        Época de la operación El Dorado Canyon
        Cielo desértico despejado casi siempre
theater-persian_gulf = Golfo Pérsico (1984)
    .facts =
        Época de la guerra de los petroleros Irán-Irak
        Sobre el mar y el desierto costero
        El tiempo más despejado de todos los teatros
theater-north_cape = Cabo Norte
    .facts =
        Noruega ártica y península de Kola
        Base de la Flota del Norte soviética
        Nubosidad baja frecuente
theater-central_europe = Europa central
    .facts =
        OTAN contra Pacto de Varsovia sobre Alemania
        Cielo despejado solo el 15 % del tiempo
theater-vietnam = Vietnam
    .facts =
        Región de Hanói y Haiphong
        Selva y relieve montañoso
        Tormentas en el 30 % de las misiones
theater-middle_east = Oriente Próximo
    .facts =
        Mediterráneo oriental y Levante
        Cielo despejado la mayor parte del tiempo
theater-desert_storm = Tormenta del Desierto
    .facts =
        Kuwait e Irak, 1991
        El verdadero bautismo de fuego del F-117A
        Cielo desértico despejado la mayor parte del tiempo
theater-cuba = Cuba
    .facts =
        Teatro insular del Caribe
        Objetivos en el mar y en tierra
        Tropical: tormentas en el 15 % de las misiones
theater-korea = Corea
    .facts =
        Península de Corea al norte de la DMZ
        Relieve montañoso
        Tiempo variable, cualquier cielo es posible
theater-ukraine = Ucrania (2026)
    .facts =
        Este de Ucrania, Crimea y sur de Rusia
        El frente arde de Járkov a Jersón
        Llanuras agrícolas, pueblos en ruinas en el frente
theater-gulf_of_finland = Golfo de Finlandia (2027)
    .facts =
        Sur de Finlandia, Estonia y la región de San Petersburgo
        Estalla la guerra en las fronteras de Finlandia y Estonia
        Taiga, lagos y un mar gris del norte
theater-suwalki = Corredor de Suwałki (2027)
    .facts =
        Kaliningrado, noreste de Polonia, Lituania y oeste de Bielorrusia
        Las pinzas rusas se cierran sobre el corredor de Suwałki
        Lagos, bosques y tierras de cultivo

## Situaciones (reglas de enfrentamiento)

roe-0 = Guerra Fría
    .facts =
        Operación encubierta en tiempo de paz
        Lo primero es no ser detectado
        Menos aviones enemigos de patrulla
        Las reglas de enfrentamiento exactas están en el briefing
roe-1 = Guerra limitada
    .facts =
        Conflicto armado regional
        Defensas aéreas enemigas en alerta
        Más aviones enemigos que en la Guerra Fría
roe-2 = Guerra convencional
    .facts =
        Guerra total
        Todas las fuerzas enemigas activas
        Más aviones enemigos que en la Guerra Fría

## Tipos de misión

kind-strike = Ataque
    .facts =
        Destruir, fotografiar o alcanzar un objetivo en tierra
        Objetivo principal y secundario
        Armamento aire-suelo
kind-air = Aire-aire
    .facts =
        Interceptar un avión enemigo
        Transportes, bombarderos o patrullas de caza
        Armamento de misiles aire-aire
kind-strike-training = Entrenamiento de ataque
    .facts =
        Misión de ataque de entrenamiento
        Aprender las armas y la navegación
kind-air-training = Entrenamiento aire-aire
    .facts =
        Misión aire-aire de entrenamiento
        Aprender las interceptaciones y los misiles
kind-drone-defense = Defensa antidrones
    .facts =
        Solo Ucrania, golfo de Finlandia y corredor de Suwałki
        Detener una oleada de drones de ataque Geran antes de que alcance una ciudad o una central nuclear
        Después, bombardear su lugar de lanzamiento

## Adversarios

opponents-0 = Novato
    .facts =
        Tripulaciones enemigas sin experiencia
        Nivel más fácil
opponents-1 = Estándar
    .facts =
        Tripulaciones enemigas de nivel medio
        Nivel por defecto
opponents-2 = Veterano
    .facts =
        Tripulaciones enemigas curtidas
        Más buques y patrullas
opponents-3 = Élite
    .facts =
        Los mejores pilotos y sirvientes de SAM enemigos
        Nivel más difícil
opponents-free = Vuelo libre
    .facts =
        Sin fuerzas enemigas, solo aviones aliados
        Vuela una ruta sobre lugares al azar, o donde quieras
ff-title = Vuelo libre
ff-summary = No hay fuerzas enemigas en ninguna parte: solo aviones y unidades aliados. Vuela adonde quieras.
ff-objective = Ruta sugerida: { $route }, y vuelta a { $base }.

## Meteorología

sky-random = Aleatorio
    .facts =
        Sorteado según el clima del teatro
        Cambia con cada nueva misión
        Las nubes tapan la vista a ambos bandos
sky-clear = Despejado
    .facts =
        Sin nubes
        Visibilidad 40 km
        El cielo clásico de la era DOS
sky-scattered = Disperso
    .facts =
        Cúmulos 20-45 %, base 2500-6000 ft
        A veces un fino altoestrato muy alto
        Visibilidad 30 km
sky-broken = Fragmentado
    .facts =
        Cúmulos 45-70 %, base 2000-5000 ft
        A menudo una capa de estratocúmulos por encima
        Visibilidad 25 km
sky-overcast = Cubierto
    .facts =
        Estratos 80-100 %, base 1500-5000 ft
        Algunos fragmentos de cúmulos por debajo
        Visibilidad 20 km
sky-storm = Tormenta
    .facts =
        Torres de cumulonimbos hasta 40000 ft
        Estratos y cúmulos bajos desde 1000 ft
        Visibilidad 8 km
menu-random-cover = Cobertura aleatoria
    .facts =
        Cobertura de la capa principal sorteada con el tiempo
        Desmarcar para ajustarla manualmente
        Capa principal: estratos con cielo cubierto y tormenta, cúmulos en los demás casos
menu-cover-slider =
    .facts =
        Parte del cielo cubierta por la capa principal
        Sin efecto con cielo despejado

## Luna

moon-date = Fecha de la misión
    .facts =
        La luna real en la fecha del briefing
        Requiere la hora del día en modo moderno
moon-0 = Luna nueva
    .facts =
        Sin luz de luna: noche cerrada, solo luces y estrellas
        Los pilotos enemigos le ven a 2 km
moon-1 = Luna creciente
    .facts = Una luna fina en el cielo del atardecer, puesta antes de medianoche
moon-2 = Cuarto creciente
    .facts = Medio iluminada, visible al anochecer, se pone hacia medianoche
moon-3 = Gibosa creciente
    .facts = Brillante, visible desde la tarde hasta pasada la medianoche
moon-4 = Luna llena
    .facts =
        Visible toda la noche, alta a medianoche
        Los pilotos enemigos le ven a 4 km
moon-5 = Gibosa menguante
    .facts = Brillante, sale al anochecer, visible hasta la mañana
moon-6 = Cuarto menguante
    .facts = Medio iluminada, sale hacia medianoche
moon-7 = Luna menguante
    .facts = Una luna fina que sale antes del amanecer

## Aeronaves

aircraft-f117 =
    .facts =
        Avión de ataque furtivo
        Subsónico, 7 g
        Cabina moderna y clásica
aircraft-f16 =
    .facts =
        Caza polivalente
        Clase Mach 2, 9 g, alabeo rápido
        Cabina de F-16 con RWR
aircraft-f22 =
    .facts =
        Caza furtivo de superioridad aérea
        Clase Mach 2, supercrucero, 9 g
        Bodegas de armas internas, empuje vectorial
aircraft-fa18e =
    .facts =
        Caza polivalente, desde los portaaviones
        Mach 1,6 – 7,5 g
        Cabina de cristal, UFC táctil
aircraft-ea18g =
    .facts =
        Ataque electrónico, desde los portaaviones
        Perturba los radares enemigos para el ataque
        Dos plazas: piloto y oficial de guerra electrónica
aircraft-f14 =
    .facts =
        Caza embarcado de defensa de flota, dos tripulantes en tándem
        Mach 1,9, alas de geometría variable, misiles Phoenix
        Cabina analógica, las pantallas de radar del RIO
aircraft-a6 =
    .facts =
        Avión de ataque embarcado todo tiempo, dos tripulantes lado a lado
        Subsónico, 6,5 g, sin poscombustión
        Cabina analógica: VDI y pantallas de radar
aircraft-tornado =
    .facts =
        Avión de ataque a baja cota
        Alas de geometría variable según la velocidad
        Cabina moderna, tres MFD
aircraft-a10 =
    .facts =
        Avión de apoyo aéreo cercano
        Lento y robusto, cañón de 30 mm
        Cabina A-10C, dos MFD
aircraft-av8b =
    .facts =
        Avión de ataque V/STOL
        Empuje vectorial: vuelo estacionario, aterriza en cualquier sitio
        Palanca de toberas: R / F, Mayús para los topes
aircraft-uh60 =
    .facts =
        Helicóptero de transporte
        Inserta tropas: aterrizar cerca del objetivo
        Cíclico y colectivo
aircraft-ah1z =
    .facts =
        Helicóptero de ataque
        Cañón en torreta, Hellfire, AIM-9
        El cañón y los Hellfire siguen al pod
aircraft-ah64 =
    .facts =
        Helicóptero de ataque biplaza
        Cañón en torreta, Hellfire, Stinger
        Un segundo jugador puede volar de artillero
aircraft-mh60r =
    .facts =
        Helicóptero antisubmarino, desde los portaaviones
        Sonoboyas, sonar calable, MAD, Mk 54
        Un segundo jugador puede ser el operador de sensores

## Secciones y grupos

section-theater = Teatro
section-opponents = Adversarios
section-situation = Situación
section-mission = Misión
section-moon = Luna
section-start-time = Hora de inicio
section-weather = Meteorología
section-realism = Realismo
section-tacview = Tacview
section-mission-summary = Resumen de la misión
section-briefing = Briefing
group-mission = Misión
group-conditions = Condiciones
group-vehicle = Aeronave
group-simulation = Simulación

## Opciones de la misión

menu-start-random = Aleatoria
    .facts =
        Sorteada con cada nueva misión, como en modo clásico
        06:00-08:00 de día o 18:00-20:00 de noche
        Desmarcar para elegir la hora
menu-start-slider =
    .facts =
        Hora de despegue
        Noche de 18:00 a 06:00
        Se conserva con NUEVA MISIÓN
menu-realistic-weapons = Armamento realista
    .facts =
        Los límites del avión real
        2 bodegas de armas en lugar de 4
        Sin misiles AIM-9 ni AIM-120
        Sin despegues ni apontajes en portaaviones
menu-crash-protection = Protección anticolisión
    .facts =
        Recupera automáticamente cerca del suelo (tren arriba)
        Los contactos con el suelo y las colisiones no son mortales
        Desactivada: siempre se puede chocar contra el suelo
menu-tacview-export = Exportar
    .facts =
        Guarda cada vuelo como grabación de Tacview (.acmi)
        En la carpeta del juego para sus archivos, en tacview
menu-tacview-real-time = Telemetría en tiempo real
    .facts =
        Envía el vuelo a Tacview mientras se vuela
        En Tacview: Record > Real-Time Telemetry
        Dirección 127.0.0.1, puerto 42674 (--tacview-address)
switch-game = JUEGO
switch-real = REAL
menu-damage = Daños
    .facts =
        Daños de las armas
        Juego: daños clásicos; los impactos se suman pero nunca derriban el avión
        Realista: un impacto de misil puede destruir el avión (¡eyéctese!), daña más sistemas y a menudo provoca un incendio, que se apaga o hace estallar el avión
        Realista: las superficies de mando dañadas descompensan el trim o hacen oscilar el avión
menu-guidance = Guiado
    .facts =
        Guiado de los misiles
        Clásico: los misiles vuelan directos hacia su blanco
        Realista: navegación proporcional hacia el punto de colisión; los motores se apagan y los misiles siguen planeando, perdiendo velocidad en cada viraje (humo solo con el motor encendido)
        Realista: los SAM ganan altura contra blancos lejanos; el AIM-120 vuela recto durante 3 s (tirar hacia arriba para lanzarlo en parábola)
menu-air-defence = Defensa aérea
    .facts =
        Defensas aéreas a baja cota
        Clásico: solo los emplazamientos SAM y los cazas
        Realista: cañones ZSU-23-4 dirigidos por radar (su radar le ve como el de un SAM) y baterías antiaéreas S-60 protegen los aeródromos y los objetivos; equipos MANPADS en las ciudades disparan SA-14 contra lo que ven
        Volar bajo sobre zonas defendidas se vuelve peligroso; las bombas y el cañón destruyen las piezas

## Pantalla principal

menu-title = STEALTH MISSION 89
menu-subtitle = SISTEMA DE PLANIFICACIÓN DE MISIONES
menu-single-mission = MISIÓN INDIVIDUAL
    .facts =
        Una misión en el teatro que se elija
        Elegir las opciones, planificar la ruta y el armamento
menu-career = CARRERA
    .facts =
        Expedientes de los pilotos: rango, condecoraciones, salidas
        Período de servicio: 12 salidas en un teatro
        Lo que se destruye queda destruido
menu-campaign = CAMPAÑA DINÁMICA
    .facts =
        Campaña dinámica en tiempo real
        Tomar las ciudades y los aeródromos del enemigo
        Misiones según la situación del momento
menu-multiplayer = MULTIJUGADOR
    .facts = Cooperativo en la campaña dinámica
    .join = Unirse a un servidor de la lista pública
    .host = Unirse a un servidor, o alojar una campaña y volar en ella
menu-options = OPCIONES
    .facts =
        Juego: estilo de la cabina, del HUD y de la aviónica, trim automático
        Gráficos, volúmenes de sonido, teclas y joystick
        Se guardan para los próximos vuelos
    .short = Juego, gráficos, sonido y controles
menu-download = DESCARGAR
    .facts =
        El juego de escritorio para Windows, Linux y macOS
        Con VR y multijugador
        Steam Deck y Steam Frame: la AppImage de Linux
menu-discord = DISCORD
    .facts =
        Volar juntos, informar de errores y compartir misiones
        Seguir el desarrollo
menu-quit = SALIR
    .facts = Volver al escritorio
menu-quit-confirm = ¿SALIR DEL JUEGO?
    .text = Volver al escritorio.
menu-version = VERSIÓN { $version }

## Pantalla de misión individual

menu-career-sortie = SALIDA DE CARRERA
menu-tour-sortie = { $pilot }  //  SALIDA { $sortie } DE { $length } DEL PERÍODO
menu-msn = SISTEMA DE PLANIFICACIÓN DE MISIONES  //  MSN { $seed }
menu-no-targets = No queda ningún objetivo en pie en este período
menu-plan-route = PLANIFICAR RUTA
    .facts =
        El mapa del teatro con la cobertura de los radares y los SAM enemigos
        Añadir, mover y quitar puntos de navegación
        Los objetivos y la base de aterrizaje son fijos
menu-loadout = ARMAMENTO
    .facts =
        Elegir las cargas de cada bodega o pilón
        Empieza con las armas del briefing
menu-generating = GENERANDO...
menu-back-records = Volver a los expedientes de los pilotos
menu-back-main = Volver al menú principal
menu-new-sortie = NUEVA SALIDA
menu-new-mission = NUEVA MISIÓN
    .facts =
        Sortea otros objetivos
        Conserva todas las opciones y la hora de inicio
menu-load = CARGAR
    .facts =
        Las misiones guardadas en vuelo (menú: GUARDAR MISIÓN)
        Reanudar el vuelo donde se guardó
menu-fly = VOLAR
    .facts = Despegar para la misión del briefing
menu-fly-vr = VOLAR EN VR
    .facts =
        Reinicia el juego en el visor VR
        Vuela exactamente esta misión
    .no-mission = Generar primero una misión
    .no-runtime = No hay ningún runtime OpenXR (Monado, WiVRn, SteamVR) activo
menu-vr-failed = no se pudo iniciar en VR: { $error }

## Resumen de la misión

summary-theater = TEATRO
summary-situation = SITUACIÓN
summary-mission = MISIÓN
summary-opponents = ADVERSARIOS
summary-time = HORA
summary-weather = METEOROLOGÍA
summary-fuel = COMBUSTIBLE
summary-night = Noche
summary-day = Día
summary-time-value = { $daytime }  salida { $depart }
summary-weather-value = { $sky }  vis. { $vis } km
summary-fuel-value = { $fuel } lb
summary-armament = Armamento: { $stores }
summary-intelligence = INTELIGENCIA
aircraft-rah66 =
    .facts =
        Helicóptero de reconocimiento furtivo
        Bodegas: Hellfire o Stinger
        Difícil de ver en el radar, lento o en estacionario
menu-ai-flight = Vuelo de la IA
    .facts =
        Aviones de la IA
        Clásico: los cazas de la IA vuelan sobre raíles a velocidad fija e ignoran los misiles que les disparan
        Realista: un modelo de vuelo como el tuyo (empuje, resistencia, límites de g): los virajes cerrados y los ascensos les cuestan velocidad
        Realista: se defienden de los misiles que detectan: virajes de lado a lado (crank) contra disparos lejanos, picado con bengalas o chaff contra los cercanos
menu-terrain = Terreno
    .facts =
        Elevación del terreno
        Clásico: el suelo llano del juego original (las montañas se quedan)
        Realista: colinas ondulantes y mesetas más altas; las unidades terrestres, los sitios SAM y los pilotos pueden esconderse tras ellas
aircraft-oh58 =
    .facts =
        Helicóptero de reconocimiento armado, ágil
        Pilones: Hellfire, Stinger o vainas de 12,7 mm
        Mira montada en el mástil, visor de casco
aircraft-ch47 =
    .facts =
        Helicóptero de carga de rotores en tándem
        Eslinga cargas de reabastecimiento y logística
        Sin par que compensar: los rotores se anulan
aircraft-c5 =
    .facts =
        Avión de transporte estratégico cuatrimotor
        Lleva munición entre las bases aéreas
        Pesado: lento al alabear y al rotar
aircraft-ac130 =
    .facts =
        Cañonero de tiro lateral
        Cañones de 40 mm y 105 mm apuntados con el pod
        Orbite a la izquierda para que los cañones apunten

menu-wind = Viento
wind-random = Aleatorio
    .facts =
        Según el tiempo: las tormentas soplan más fuerte
        Cambia con cada misión nueva
wind-calm = En calma
    .facts =
        0-3 kt
        Aire tranquilo, mar como un espejo
wind-light = Flojo
    .facts =
        5-10 kt
        Olas pequeñas
wind-moderate = Moderado
    .facts =
        12-18 kt, rachas de hasta 25 kt
        Borreguillos, los barcos se balancean un poco
wind-strong = Fuerte
    .facts =
        20-30 kt, rachas de hasta 45 kt
        Turbulencia a baja altura, mar agitada
wind-gale = Temporal
    .facts =
        32-45 kt, rachas de hasta 65 kt
        Turbulencia severa, mar gruesa, cubiertas de portaaviones en movimiento
