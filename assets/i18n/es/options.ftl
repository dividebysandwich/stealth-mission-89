# La ventana OPCIONES (src/options.rs) y sus páginas del menú en vuelo.

options-title = OPCIONES
options-tab-game = JUEGO
options-tab-graphics = GRÁFICOS
options-tab-sound = SONIDO
options-tab-controls = CONTROLES
options-footer = Se guarda para los próximos vuelos; los ajustes de realismo se eligen en cada partida

## JUEGO

options-section-language = Idioma
options-section-classic-modern = Clásico / Moderno
options-cockpit = Cabina
    .facts =
        Moderna: pantallas de alta resolución con botones en el marco
        Clásica: el panel de instrumentos de la era DOS y sus pantallas de 77x60 (F-117A)
options-hud = HUD
    .facts =
        Moderno: HUD de alta resolución en verde estilo F-16, con los modos NAV, AA y AG
        Clásico: el HUD de la era DOS
options-avionics = Aviónica
    .facts =
        Moderna: páginas MFD estilo F-16 en cada MFD (situación, HARM, pod de designación, mapa, cargas) con un cursor desplazable
        Clásica: mapa y TAC a la izquierda, cámara, WAY, WPN y ORD a la derecha
options-flight-model = Modelo de vuelo
    .facts =
        Moderno: modelo de vuelo aerodinámico con mandos fly-by-wire: la palanca manda la velocidad de cabeceo y de alabeo; al soltarla, el morro se queda donde está y se mantiene la trayectoria
        Clásico: un modelo de vuelo de la era DOS
options-section-flying = Pilotaje
options-auto-trim = Trim automático
    .facts =
        Con la palanca suelta, el trim mantiene la actitud en la que se dejó
        Helicópteros: el morro ya no sube con la velocidad; reactores: compensa las superficies de mando dañadas
        Trim manual: Alt + flechas, Alt + Inicio lo centra
options-tooltips = Tooltips de la cabina
    .facts = Qué es un mando o una pantalla de la cabina, bajo el ratón o un dedo en VR
options-player-labels = Nombres de los jugadores
    .facts = Multijugador: los nombres de los demás pilotos sobre sus aviones, a cualquier distancia
options-label-size =
    .facts =
        Tamaño de los nombres de los jugadores
        50 %: por defecto
options-section-vr-size = Tamaño de la cabina en VR
options-vr-size =
    .facts =
        Tamaño de la cabina (y del mundo) en el visor
        100 %: tamaño real
options-section-gif = Grabación GIF
options-gif-size = TAMAÑO
options-gif-rate = FPS
options-gif-scale-100 =
    .facts = La resolución completa de la ventana: archivos grandes
options-gif-scale-25 =
    .facts = Archivos pequeños
options-gif-rate-50 =
    .facts = Solo si el juego va a 100 fps o más (se captura una imagen de cada dos)
options-gif-hint = F12 (salvo reasignación) inicia y detiene; se guarda en su carpeta Imágenes

## GRÁFICOS

options-section-rendering = Renderizado
options-shaders = Shaders
    .facts =
        Moderno: sombreado direccional sutil, bruma suave con la distancia
        Clásico: sombreado plano
options-shadows = Sombras
    .facts =
        Moderno: sombras del sol en la cabina y en el mundo (consume tiempo de GPU)
        Clásico: sin sombras
options-point-lights = Luces puntuales
    .facts =
        Moderno: iluminación de la cabina y de las pantallas; explosiones, estelas y luces de pista iluminan la escena
        Clásico: solo luz del sol y del cielo
options-time-of-day = Hora del día
    .facts =
        Moderno: el sol y la luna siguen el reloj de la misión, el cielo se oscurece y se aclara con el sol; estrellas y fase lunar de noche
        Moderno: la luz de la luna fija a qué distancia le ven de noche los pilotos enemigos (2 km sin luna, 4 km con luna llena)
        Clásico: un lento cambio de colores y un sol fijo
options-msaa = Antialiasing (MSAA)
    .facts =
        Bordes suavizados: MSAA 4x, 2x en VR (consume tiempo de GPU)
        Desactivado: bordes de píxel duros
options-auto-detail = Detalle automático del paisaje (VR)
    .facts =
        En el visor, el alcance de pueblos, bosques y campos se reduce cuando los fotogramas van con retraso y vuelve a crecer cuando sobra tiempo
        Los ajustes gráficos se quedan como se eligieron
options-sun-moon = Sol y luna
    .facts =
        El sol en el cielo de día, la luna y las estrellas de noche
        La luna muestra la fase de la misión
options-cloud-quality = Nubes
cloud-quality-auto = Auto
    .facts = Baja en RV, alta en pantalla
cloud-quality-low = Baja
    .facts = Nubes toscas a un tercio de la resolución (lo más rápido)
cloud-quality-medium = Media
    .facts = Mitad de la resolución, más detalle
cloud-quality-high = Alta
    .facts = Resolución completa, máximo detalle y sombra propia (cuesta tiempo de GPU)

## SONIDO

options-section-volume = Volumen
options-volume-master = General
    .facts = Todos los sonidos
options-volume-engine = Motor
    .facts = Su motor o rotor, y las aeronaves que se oyen alrededor
options-volume-effects = Efectos
    .facts = Armas, explosiones, tren, radio y lo demás
options-volume-warnings = Alarmas
    .facts = Bocina de pérdida, avisos de radar y de misil, sirena de incendio, tonos del Sidewinder
options-volume-rwr = RWR
    .facts = Los tonos de aviso radar, como parte del volumen de las alarmas
options-section-rwr-tones = Tonos de aviso radar
rwr-tones-all = TODOS
    .facts = Un radar que le ilumina, cada barrido que le detecta, los lanzamientos de misiles
rwr-tones-detect = SOLO DETECCIÓN
    .facts =
        Un tono cuando un radar le detecta (de nuevo tras un rato sin radares)
        Y los lanzamientos de misiles
rwr-tones-launch = SOLO LANZAMIENTOS
    .facts = Solo los misiles lanzados contra usted
rwr-tones-off = NINGUNO
    .facts = Ningún tono de aviso radar (las luces y pantallas siguen avisando)
options-section-cockpit = Cabina
options-low-pass = Filtro paso bajo del motor
    .facts =
        Dentro de la aeronave el motor suena amortiguado (la cubierta y el casco cortan los agudos)
        Desactivado: igual que fuera

## CONTROLES

options-section-bindings = Asignaciones
options-keys-joystick = TECLAS Y JOYSTICK
    .facts =
        Asignar teclas, ejes y botones del joystick y del HOTAS
        Se guarda para los próximos vuelos
options-section-head-tracking = Seguimiento de cabeza y ojos
options-head-tracking = Seguimiento de cabeza y ojos
    .facts =
        La cabeza gira y desplaza la vista en la cabina (en escritorio; en VR lo hace el visor)
        Programa de seguimiento (OpenTrack...): salida FreeTrack, o UDP a 127.0.0.1 puerto 4242
        Inicio: la posición actual de la cabeza pasa a ser el centro
options-tracking = Seguimiento activo ({ $source })
options-no-tracker = No se detecta ningún seguimiento: iniciar el programa de seguimiento

## EFFECTS

options-section-effects = Efectos
options-section-retro = Retro
effect-shake = Sacudidas de cámara
    .facts =
        La vista se sacude con el cañón, explosiones cercanas, pérdida de sustentación, aterrizajes y la pista
        VR: la cabina vibra, su cabeza permanece quieta
effect-flashes = Destellos de combate
    .facts = Estallidos de flak, explosiones y lanzamientos de misiles iluminan un instante las nubes, el suelo y la cabina
effect-barrage = Barrera nocturna
    .facts =
        Guerra convencional de noche: los cañones de las ciudades disparan trazadoras a ciegas al cielo, los reflectores barren sobre ellas
        Los drones enemigos sobre una ciudad son atrapados por sus reflectores y atraen el fuego de sus cañones
        Solo por el aspecto: la barrera no alcanza nada
effect-cockpit-night = Iluminación nocturna de cabina
    .facts =
        Una tenue luz roja de cabina al oscurecer (azul verdoso bajo las gafas)
        La master caution parpadea en ámbar sobre el panel mientras hay un aviso activo
effect-nvg = Gafas de visión nocturna
    .facts =
        Las luces intensas se expanden en halos, la imagen granula más en la oscuridad
        Escritorio: el campo de visión redondo de las gafas
effect-snow = Nieve
    .facts = Por encima del nivel de congelación las tormentas dejan nieve en vez de lluvia: bajo en un invierno nórdico, alto en verano
effect-mist = Niebla a ras de suelo
    .facts =
        Shaders modernos: la niebla cubre el terreno bajo y el mar al amanecer, al anochecer y de noche
        Fina en los desiertos, ninguna en tormentas
effect-cloud-shadows = Sombras de nubes
    .facts = Shaders modernos: las sombras de las nubes sobre el suelo
effect-vapour = Vapor y corriente descendente
    .facts =
        En virajes cerrados, estelas de vapor en las puntas de ala
        Un helicóptero en estacionario bajo levanta polvo, o espuma sobre el agua
        Los vehículos en marcha levantan polvo tras ellos
effect-night-sky = Cielo nocturno
    .facts =
        De vez en cuando estrellas fugaces, las estrellas bajas sobre el horizonte titilan
        La aurora boreal sobre el Cabo Norte
effect-vga = Colores VGA
    .facts = El aspecto de 1989 con sombreado moderno: 64 niveles por canal de color, como una tarjeta VGA, con tramado
effect-scanlines = Líneas de barrido
    .facts = Las líneas de barrido de un monitor CRT (no en VR)

## LANGUAGE NAMES

lang-name-en = inglés
lang-name-bg = búlgaro
lang-name-zh-Hans = chino (simplificado)
lang-name-hr = croata
lang-name-cs = checo
lang-name-da = danés
lang-name-nl = neerlandés
lang-name-et = estonio
lang-name-fi = finés
lang-name-fr = francés
lang-name-de = alemán
lang-name-el = griego
lang-name-hu = húngaro
lang-name-id = indonesio
lang-name-ga = irlandés
lang-name-it = italiano
lang-name-ja = japonés
lang-name-ko = coreano
lang-name-lv = letón
lang-name-lt = lituano
lang-name-mt = maltés
lang-name-pl = polaco
lang-name-pt-BR = portugués (Brasil)
lang-name-ro = rumano
lang-name-ru = ruso
lang-name-sk = eslovaco
lang-name-sl = esloveno
lang-name-es = español
lang-name-sv = sueco
lang-name-uk = ucraniano

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Idioma del navegador: { $language }. ¿Jugar en este idioma?
language-offer-yes = Sí
language-offer-no = No
language-offer-never = No volver a preguntar
effect-sunbeams = Rayos de sol
    .facts = Haces de luz solar entre los claros de un cielo cubierto con huecos
options-msaa-forced = Desactivada mientras el pixelado (retro) esté activo
options-volume-radio = Radio
    .facts = Los demás jugadores hablando en la radio coop, y el ATC hablado
options-volume-atc = Voz ATC
    .facts = Las llamadas ATC habladas, como parte del volumen de la radio
options-volume-music = Música
    .facts = La música del menú principal
options-section-atc = ATC hablado
options-section-radio = Radio coop
options-push-to-talk = Pulsar para hablar
    .facts =
        Mantenga la tecla de hablar (Bloq Mayús, o un botón de joystick o de VR asignado a PTT) y todos los jugadores le oyen
        El micrófono se abre la primera vez que la pulsa
options-mic-gain = GANANCIA MIC
    .facts =
        Qué tan alto sale su micrófono
        100 %: tal cual
options-mic-default = Predeterminado
options-voice-text-only = solo texto
options-voices-line = Controladores: { $controllers }   Piloto: { $pilot }
options-voices = VOCES
    .facts =
        Descargar voces para las llamadas de radio, borrarlas
        Elegir las de los controladores y la propia, y escucharlas
options-voices-web = Solo texto en el navegador
options-tracking-mouse = Mirada con ratón al seguir
    .facts =
        Activado: el botón derecho del ratón sigue girando y haciendo zoom en la vista de cabina
        Desactivado: mientras el tracker envía, el botón derecho del ratón no hace nada en la cabina
options-atc = Control de tráfico aéreo
    .facts =
        Llamadas de radio de torre, aproximación y GCA en los aeródromos amigos
        Autorización de despegue, break de llegada y una guía GCA con nubes bajas o poca visibilidad
options-pixel-block = Tamaño de bloque del pixelado
    .facts = El lado de un bloque, en píxeles de pantalla (2: fino, 5: grueso)
effect-pixelize = Pixelado
    .facts =
        Toda la vista 3D (terreno, aviones, edificios, cabina) en píxeles cuadrados grandes, como en un PC con MS-DOS; los menús y los tooltips siguen nítidos (no en VR)
        El control deslizante fija el tamaño de un bloque, en píxeles de pantalla (2: fino, 5: grueso)
        El antialiasing está desactivado mientras esté activo
