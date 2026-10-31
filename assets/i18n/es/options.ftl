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

options-section-effects = Effects
options-section-retro = Retro
effect-shake = Camera shake
    .facts =
        The view shakes with the gun, nearby blasts, stall buffet, touchdowns and the runway
        VR: the cockpit vibrates, your head stays still
effect-flashes = Combat flashes
    .facts =
        Flak bursts, explosions and missile launches light the clouds, the ground and the cockpit for a moment
effect-barrage = Night barrage
    .facts =
        Conventional war at night: the towns' guns fire tracer blindly into the sky, searchlights sweep over them
        Only for the look: the barrage hits nothing, the searchlights find nothing
effect-cockpit-night = Cockpit night lighting
    .facts =
        A dim red flood light in the cockpit after dark (blue-green under the goggles)
        The master caution pulses amber over the panel while a warning is on
effect-nvg = Night vision goggles
    .facts =
        Bright lights bloom into halos, the picture grains more in the dark
        Desktop: the goggles' round field of view
effect-snow = Snow
    .facts =
        Above the freezing level storms snow instead of rain: low in a northern winter, high in summer
effect-mist = Ground mist
    .facts =
        Modern shaders: mist lies over the low ground and the sea at dawn, dusk and night
        Thin in the deserts, none in storms
effect-cloud-shadows = Cloud shadows
    .facts =
        Modern shaders: the clouds' shadows on the ground below
effect-vapour = Vapor y corriente descendente
    .facts =
        En virajes cerrados, estelas de vapor en las puntas de ala
        Un helicóptero en estacionario bajo levanta polvo, o espuma sobre el agua
        Los vehículos en marcha levantan polvo tras ellos
effect-night-sky = Night sky
    .facts =
        Shooting stars now and then, the stars low over the horizon twinkle
        The northern lights over the North Cape
effect-vga = VGA colours
    .facts =
        The 1989 look with modern shading: 64 steps a colour channel, as a VGA card's, dithered
effect-scanlines = Scanlines
    .facts =
        A CRT monitor's scanlines (not in VR)

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
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
