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
options-section-vr-size = Dimensione del cockpit in VR
options-vr-size =
    .facts =
        Dimensione del cockpit (e del mondo) nel visore
        100%: grandezza reale

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
