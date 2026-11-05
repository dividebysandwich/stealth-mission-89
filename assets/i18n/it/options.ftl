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
options-keep-time-accel = Mantieni l'accelerazione del tempo
    .facts =
        L'accelerazione non si interrompe più quando vieni individuato, sganci un'arma o vieni abbattuto
        Solo il pulsante TIME e Shift + Z / Shift + X la cambiano
options-tooltips = Tooltip del cockpit
    .facts = Che cos'è un comando o un display del cockpit, sotto il mouse o un dito in VR
options-player-labels = Nomi dei giocatori
    .facts = Multigiocatore: i nomi degli altri piloti sopra i loro aerei, a qualsiasi distanza
options-label-size =
    .facts =
        Dimensione dei nomi dei giocatori
        50%: predefinita
options-seat-height = Altezza del sedile
    .facts =
        Quanto in alto siedi nell'abitacolo, sullo schermo e in VR
        0 cm: l'altezza degli occhi normale del velivolo
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
options-cloud-quality = Nuvole
cloud-quality-auto = Auto
    .facts = Bassa in VR, alta sullo schermo
cloud-quality-low = Bassa
    .facts = Nuvole grossolane a un terzo della risoluzione (la più veloce)
cloud-quality-medium = Media
    .facts = Metà della risoluzione, dettagli più fini
cloud-quality-high = Alta
    .facts = Risoluzione piena, massimo dettaglio e ombra propria (costa tempo GPU)

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

options-section-effects = Effetti
options-section-retro = Retro
effect-shake = Scosse della camera
    .facts =
        La vista trema con il cannone, le esplosioni vicine, la perdita di portanza, gli atterraggi e la pista
        VR: la cabina vibra, la testa resta ferma
effect-flashes = Bagliori di combattimento
    .facts = Scoppi di antiaerea, esplosioni e lanci di missili illuminano per un attimo nuvole, terreno e cabina
effect-barrage = Sbarramento notturno
    .facts =
        Guerra convenzionale di notte: i cannoni delle città sparano traccianti alla cieca nel cielo, i proiettori li spazzano
        I droni nemici su una città vengono presi dai suoi proiettori e attirano il fuoco dei suoi cannoni
        Solo per l'aspetto: lo sbarramento non colpisce nulla
effect-cockpit-night = Illuminazione notturna della cabina
    .facts =
        Una fioca luce rossa in cabina dopo il tramonto (verde-azzurra con i visori)
        La master caution pulsa color ambra sul pannello finché c'è un avviso attivo
effect-nvg = Visori notturni
    .facts =
        Le luci intense si espandono in aloni, l'immagine ha più grana al buio
        Desktop: il campo visivo rotondo dei visori
effect-snow = Neve
    .facts = Sopra lo zero termico i temporali portano neve invece di pioggia: bassa in un inverno nordico, alta d'estate
effect-mist = Foschia al suolo
    .facts =
        Shader moderni: la foschia si stende sul terreno basso e sul mare all'alba, al tramonto e di notte
        Rada nei deserti, assente con i temporali
effect-cloud-shadows = Ombre delle nuvole
    .facts = Shader moderni: le ombre delle nuvole sul terreno sottostante
effect-vapour = Vapore e flusso del rotore
    .facts =
        Nelle virate strette, scie di vapore dalle estremità alari
        Un elicottero in hovering basso solleva polvere, o spruzzi sull'acqua
        I veicoli in movimento sollevano polvere dietro di sé
effect-night-sky = Cielo notturno
    .facts =
        Ogni tanto stelle cadenti, le stelle basse sull'orizzonte scintillano
        L'aurora boreale su Capo Nord
effect-vga = Colori VGA
    .facts = L'aspetto del 1989 con ombreggiatura moderna: 64 livelli per canale colore, come una scheda VGA, con dithering
effect-scanlines = Linee di scansione
    .facts = Le linee di scansione di un monitor CRT (non in VR)

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
effect-sunbeams = Raggi di sole
    .facts = Fasci di luce solare tra i varchi di una copertura nuvolosa frammentata
options-msaa-forced = Disattivata finché la pixelizzazione (retro) è attiva
options-volume-radio = Radio
    .facts = Gli altri giocatori che parlano nella radio coop, e l'ATC parlato
options-volume-atc = Voce ATC
    .facts = Le chiamate ATC parlate, come parte del volume della radio
options-volume-music = Musica
    .facts = La musica del menu principale
options-section-atc = ATC parlato
options-section-radio = Radio coop
options-push-to-talk = Premi per parlare
    .facts =
        Tieni premuto il tasto per parlare (Bloc Maiusc, o un pulsante di joystick o VR assegnato a PTT) e tutti i giocatori ti sentono
        Il microfono si apre la prima volta che lo premi
options-mic-gain = GUADAGNO MIC
    .facts =
        Quanto forte esce il tuo microfono
        100%: così com'è
options-mic-default = Predefinito
options-voice-text-only = solo testo
options-voices-line = Controllori: { $controllers }   Pilota: { $pilot }
options-voices = VOCI
    .facts =
        Scarica le voci per le chiamate radio, eliminale
        Scegli quelle dei controllori e la tua, e ascoltale
options-voices-web = Solo testo nel browser
options-tracking-mouse = Vista con mouse durante il tracking
    .facts =
        Attivo: il tasto destro del mouse continua a ruotare e ingrandire la vista della cabina
        Disattivo: finché il tracker trasmette, il tasto destro del mouse non fa nulla in cabina
options-atc = Controllo del traffico aereo
    .facts =
        Chiamate radio di torre, avvicinamento e GCA negli aeroporti amici
        Autorizzazione al decollo, break in sovrappasso e un talkdown con nuvole basse o scarsa visibilità
options-pixel-block = Dimensione dei blocchi della pixelizzazione
    .facts = Il lato di un blocco, in pixel dello schermo (2: fine, 5: grossolano)
effect-pixelize = Pixelizzazione
    .facts =
        Tutta la vista 3D (terreno, velivoli, edifici, cabina) in grandi pixel quadrati, come su un PC MS-DOS; i menu e i tooltip restano nitidi (non in VR)
        Il cursore imposta la dimensione di un blocco, in pixel dello schermo (2: fine, 5: grossolano)
        L'anti-aliasing è disattivato finché è attiva
        I tre interruttori sotto lasciano nitidi l'HUD, il cockpit o gli MFD, gli strumenti e il Kneeboard
effect-sharp-hud = Senza HUD
    .facts =
        Pixelizzazione: l'HUD (e il visore del casco) è disegnato nitido sopra la vista pixelizzata, così i suoi simboli sottili restano leggibili
effect-sharp-cockpit = Senza cockpit (pulsanti, interruttori...)
    .facts =
        Pixelizzazione: pannello, telaio, sedile, pulsanti, interruttori, cloche e manetta dell'abitacolo sono disegnati nitidi
effect-sharp-gauges = Senza MFD, strumenti e Kneeboard
    .facts =
        Pixelizzazione: gli MFD, la sfera d'assetto, gli strumenti, le spie, le letture, le etichette e il Kneeboard sono disegnati nitidi
options-needs-pixelize = Richiede la Pixelizzazione
