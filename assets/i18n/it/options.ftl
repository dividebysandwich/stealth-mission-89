# La finestra OPZIONI (src/options.rs) e le sue pagine nel menu in volo.

options-title = OPZIONI
options-tab-game = GIOCO
options-tab-graphics = GRAFICA
options-tab-sound = AUDIO
options-tab-controls = COMANDI
options-footer = Salvate per i voli successivi; le impostazioni di realismo si scelgono a ogni partita

## GIOCO

options-section-language = Lingua
options-section-classic-modern = Classico / Moderno
options-cockpit = Cockpit
    .facts =
        Moderno: display ad alta risoluzione con pulsanti sulla cornice
        Classico: il cruscotto dell'era DOS e i suoi display 77x60 (F-117A)
options-hud = HUD
    .facts =
        Moderno: HUD verde ad alta risoluzione in stile F-16, con i modi NAV, AA e AG
        Classico: l'HUD dell'era DOS
options-avionics = Avionica
    .facts =
        Moderno: pagine MFD in stile F-16 su ciascun MFD (situazione, HARM, pod di puntamento, mappa, carichi) con un cursore mobile
        Classico: mappa e TAC a sinistra, telecamera, WAY, WPN e ORD a destra
options-flight-model = Modello di volo
    .facts =
        Moderno: modello di volo aerodinamico con fly-by-wire: la cloche comanda le velocità di beccheggio e di rollio; rilasciata, il muso resta fermo e la traiettoria viene mantenuta
        Classico: un modello di volo dell'era DOS
options-section-flying = Pilotaggio
options-auto-trim = Trim automatico
    .facts =
        Con la cloche rilasciata, il trim mantiene l'assetto in cui è stato lasciato
        Elicotteri: il muso non si alza più con la velocità; jet: compensa le superfici di comando danneggiate
        Trim manuale: Alt + frecce, Alt + Home lo centra
options-tooltips = Tooltip del cockpit
    .facts = Che cos'è un comando o un display del cockpit, sotto il mouse o un dito in VR
options-player-labels = Nomi dei giocatori
    .facts = Multigiocatore: i nomi degli altri piloti sopra i loro aerei, a qualsiasi distanza
options-label-size =
    .facts =
        Dimensione dei nomi dei giocatori
        50%: predefinita
options-section-vr-size = Dimensione del cockpit in VR
options-vr-size =
    .facts =
        Dimensione del cockpit (e del mondo) nel visore
        100%: grandezza reale
options-section-gif = Registrazione GIF
options-gif-size = DIMENSIONE
options-gif-rate = FPS
options-gif-scale-100 =
    .facts = La piena risoluzione della finestra: file grandi
options-gif-scale-25 =
    .facts = File piccoli
options-gif-rate-50 =
    .facts = Solo se il gioco gira a 100 fps o più (viene catturato un fotogramma su due)
options-gif-hint = F12 (salvo riassegnazione) avvia e ferma; salvato nella cartella Immagini

## GRAFICA

options-section-rendering = Rendering
options-shaders = Shader
    .facts =
        Moderno: ombreggiatura direzionale discreta, foschia in lontananza sfumata
        Classico: ombreggiatura piatta
options-shadows = Ombre
    .facts =
        Moderno: ombre del sole nel cockpit e nel mondo (costa tempo GPU)
        Classico: niente ombre
options-point-lights = Luci puntiformi
    .facts =
        Moderno: illuminazione del cockpit e dei display; esplosioni, scie e luci di pista illuminano la scena
        Classico: solo la luce del sole e del cielo
options-time-of-day = Ora del giorno
    .facts =
        Moderno: il sole e la luna seguono l'orologio della missione, il cielo si scurisce e si schiarisce con il sole; stelle e fase lunare di notte
        Moderno: il chiaro di luna stabilisce a che distanza i piloti nemici avvistano l'aereo di notte (2 km senza luna, 4 km con la luna piena)
        Classico: un lento cambio di colori e un sole fisso
options-msaa = Antialiasing (MSAA)
    .facts =
        Bordi smussati: MSAA 4x, 2x in VR (costa tempo GPU)
        Disattivo: bordi dei pixel netti
options-auto-detail = Dettaglio automatico dello scenario (VR)
    .facts =
        Nel visore, la distanza di città, boschi e campi si riduce quando i fotogrammi sono in ritardo e torna a crescere quando avanza tempo
        Le impostazioni grafiche restano quelle scelte
options-sun-moon = Sole e luna
    .facts =
        Il sole nel cielo di giorno, la luna e le stelle di notte
        La luna mostra la fase della missione

## AUDIO

options-section-volume = Volume
options-volume-master = Generale
    .facts = Tutti i suoni
options-volume-engine = Motore
    .facts = Il motore o il rotore del proprio velivolo, e gli aerei sentiti intorno
options-volume-effects = Effetti
    .facts = Armi, esplosioni, carrello, radio e il resto
options-volume-warnings = Allarmi
    .facts = Avvisatore di stallo, allarmi radar e missile, sirena antincendio, toni del Sidewinder
options-volume-rwr = RWR
    .facts = I toni d'allarme radar, come parte del volume degli allarmi
options-section-rwr-tones = Toni d'allarme radar
rwr-tones-all = TUTTI
    .facts = Un radar che vi illumina, ogni scansione che vi rileva, i lanci di missili
rwr-tones-detect = SOLO RILEVAMENTO
    .facts =
        Un tono quando un radar vi rileva (di nuovo dopo che i radar vi hanno perso per un po')
        E i lanci di missili
rwr-tones-launch = SOLO LANCI
    .facts = Solo i missili lanciati contro di voi
rwr-tones-off = NESSUNO
    .facts = Nessun tono d'allarme radar (le spie e i display avvisano comunque)
options-section-cockpit = Cockpit
options-low-pass = Filtro passa-basso del motore
    .facts =
        Dentro l'aereo il motore suona ovattato (il tettuccio e il casco tagliano gli acuti)
        Disattivo: lo stesso suono che all'esterno

## COMANDI

options-section-bindings = Assegnazioni
options-keys-joystick = TASTI E JOYSTICK
    .facts =
        Assegnare tasti, assi e pulsanti del joystick e dell'HOTAS
        Salvate per i voli successivi
options-section-head-tracking = Tracciamento testa/occhi
options-head-tracking = Tracciamento testa/occhi
    .facts =
        La testa ruota e sposta la visuale nel cockpit (su schermo; in VR lo fa il visore)
        Programma di tracciamento (OpenTrack...): uscita FreeTrack, oppure UDP verso 127.0.0.1 porta 4242
        Home: la posizione attuale della testa diventa il centro
options-tracking = Tracciamento ({ $source })
options-no-tracker = Nessun tracker rilevato: avviare il programma di tracciamento

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
effect-vapour = Vapore e flusso del rotore
    .facts =
        Nelle virate strette, scie di vapore dalle estremità alari
        Un elicottero in hovering basso solleva polvere, o spruzzi sull'acqua
        I veicoli in movimento sollevano polvere dietro di sé
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

lang-name-en = inglese
lang-name-bg = bulgaro
lang-name-zh-Hans = cinese (semplificato)
lang-name-hr = croato
lang-name-cs = ceco
lang-name-da = danese
lang-name-nl = olandese
lang-name-et = estone
lang-name-fi = finlandese
lang-name-fr = francese
lang-name-de = tedesco
lang-name-el = greco
lang-name-hu = ungherese
lang-name-id = indonesiano
lang-name-ga = irlandese
lang-name-it = italiano
lang-name-ja = giapponese
lang-name-ko = coreano
lang-name-lv = lettone
lang-name-lt = lituano
lang-name-mt = maltese
lang-name-pl = polacco
lang-name-pt-BR = portoghese (Brasile)
lang-name-ro = rumeno
lang-name-ru = russo
lang-name-sk = slovacco
lang-name-sl = sloveno
lang-name-es = spagnolo
lang-name-sv = svedese
lang-name-uk = ucraino

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Lingua del browser: { $language }. Giocare in questa lingua?
language-offer-yes = Sì
language-offer-no = No
language-offer-never = Non chiedere più
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
