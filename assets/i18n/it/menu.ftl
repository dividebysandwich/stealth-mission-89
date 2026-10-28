# Il menu principale e la schermata di missione singola (src/menu.rs).

## Teatri (stealth_host::theater::THEATERS)

theater-libya = Libia (1986)
    .facts =
        Golfo della Sirte e costa libica
        Epoca dell'operazione El Dorado Canyon
        Cielo desertico sereno quasi sempre
theater-persian_gulf = Golfo Persico (1984)
    .facts =
        Epoca della guerra delle petroliere Iran-Iraq
        Sopra il mare e il deserto costiero
        Il meteo più sereno di tutti i teatri
theater-north_cape = Capo Nord
    .facts =
        Norvegia artica e penisola di Kola
        Base della Flotta del Nord sovietica
        Copertura bassa frequente
theater-central_europe = Europa centrale
    .facts =
        NATO contro Patto di Varsavia sopra la Germania
        Cielo sereno solo il 15% del tempo
theater-vietnam = Vietnam
    .facts =
        Regione di Hanoi e Haiphong
        Giungla e terreno montuoso
        Temporali nel 30% delle missioni
theater-middle_east = Medio Oriente
    .facts =
        Mediterraneo orientale e Levante
        Cielo per lo più sereno
theater-desert_storm = Desert Storm
    .facts =
        Kuwait e Iraq, 1991
        Il vero debutto in combattimento dell'F-117A
        Cielo desertico per lo più sereno
theater-cuba = Cuba
    .facts =
        Teatro insulare dei Caraibi
        Obiettivi in mare e a terra
        Tropicale: temporali nel 15% delle missioni
theater-korea = Corea
    .facts =
        Penisola coreana a nord della DMZ
        Terreno montuoso
        Meteo variabile, ogni cielo è possibile
theater-ukraine = Ucraina (2026)
    .facts =
        Ucraina orientale, Crimea e Russia meridionale
        La linea del fronte brucia da Kharkiv a Kherson
        Campagne aperte, città in rovina sul fronte

## Situazioni (regole d'ingaggio)

roe-0 = Guerra fredda
    .facts =
        Operazione clandestina in tempo di pace
        Restare non rilevati conta più di tutto
        Meno aerei nemici in pattuglia
        Le regole d'ingaggio esatte sono nel briefing
roe-1 = Guerra limitata
    .facts =
        Conflitto armato regionale
        Difese aeree nemiche in allerta
        Più aerei nemici che in guerra fredda
roe-2 = Guerra convenzionale
    .facts =
        Guerra totale
        Tutte le forze nemiche attive
        Più aerei nemici che in guerra fredda

## Tipi di missione

kind-strike = Attacco al suolo
    .facts =
        Distruggere, fotografare o raggiungere un obiettivo a terra
        Obiettivo primario e secondario
        Armamento aria-suolo
kind-air = Aria-aria
    .facts =
        Intercettare un aereo nemico
        Trasporti, bombardieri o pattuglie di caccia
        Armamento di missili aria-aria
kind-strike-training = Addestramento attacco
    .facts =
        Missione di attacco al suolo di addestramento
        Imparare le armi e la navigazione
kind-air-training = Addestramento aria-aria
    .facts =
        Missione aria-aria di addestramento
        Imparare le intercettazioni e i missili
kind-drone-defense = Difesa anti-drone
    .facts =
        Solo Ucraina
        Fermare un'ondata di droni d'attacco Geran prima che colpisca una città o una centrale nucleare
        Poi bombardare il loro sito di lancio

## Avversari

opponents-0 = Recluta
    .facts =
        Equipaggi nemici inesperti
        Livello più facile
opponents-1 = Regolare
    .facts =
        Equipaggi nemici nella media
        Livello predefinito
opponents-2 = Veterano
    .facts =
        Equipaggi nemici esperti
        Più navi e pattuglie
opponents-3 = Élite
    .facts =
        I migliori piloti e serventi SAM nemici
        Livello più difficile

## Meteo

sky-random = Casuale
    .facts =
        Estratto dal clima del teatro
        Cambia a ogni nuova missione
        Le nubi bloccano la visuale a entrambe le parti
sky-clear = Sereno
    .facts =
        Nessuna nube
        Visibilità 40 km
        Il cielo classico dell'era DOS
sky-scattered = Sparse
    .facts =
        Cumuli 20-45%, base 2500-6000 ft
        A volte un sottile altostrato in alto
        Visibilità 30 km
sky-broken = Frammentato
    .facts =
        Cumuli 45-70%, base 2000-5000 ft
        Spesso uno strato di stratocumuli sopra
        Visibilità 25 km
sky-overcast = Coperto
    .facts =
        Strati 80-100%, base 1500-5000 ft
        Qualche frammento di cumulo sotto
        Visibilità 20 km
sky-storm = Temporale
    .facts =
        Torri di cumulonembi fino a 40000 ft
        Strati e cumuli bassi da 1000 ft
        Visibilità 8 km
menu-random-cover = Copertura casuale
    .facts =
        Copertura dello strato principale estratta con il meteo
        Deselezionare per impostarla a mano
        Strato principale: strati con cielo coperto e temporale, cumuli negli altri casi
menu-cover-slider =
    .facts =
        Parte del cielo coperta dallo strato principale
        Nessun effetto con cielo sereno

## Luna

moon-date = Data della missione
    .facts =
        La luna reale alla data del briefing
        Richiede l'ora del giorno in modalità moderna
moon-0 = Luna nuova
    .facts =
        Nessun chiaro di luna: notte nera, solo luci e stelle
        I piloti nemici avvistano l'aereo a 2 km
moon-1 = Falce crescente
    .facts = Una luna sottile nel cielo serale, tramonta prima di mezzanotte
moon-2 = Primo quarto
    .facts = Illuminata a metà, alta la sera, tramonta verso mezzanotte
moon-3 = Gibbosa crescente
    .facts = Luminosa, alta dal pomeriggio fino a dopo mezzanotte
moon-4 = Luna piena
    .facts =
        Alta tutta la notte, al culmine a mezzanotte
        I piloti nemici avvistano l'aereo a 4 km
moon-5 = Gibbosa calante
    .facts = Luminosa, sorge la sera, alta fino al mattino
moon-6 = Ultimo quarto
    .facts = Illuminata a metà, sorge verso mezzanotte
moon-7 = Falce calante
    .facts = Una luna sottile che sorge prima dell'alba

## Velivoli

aircraft-f117 =
    .facts =
        Aereo d'attacco stealth
        Subsonico, 7 g
        Cockpit moderno e classico
aircraft-f16 =
    .facts =
        Caccia multiruolo
        Classe Mach 2, 9 g, rollio rapido
        Cockpit dell'F-16 con RWR
aircraft-tornado =
    .facts =
        Aereo d'attacco a bassa quota
        Le ali a geometria variabile si aprono e chiudono con la velocità
        Cockpit moderno, tre MFD
aircraft-a10 =
    .facts =
        Aereo d'appoggio aereo ravvicinato
        Lento e robusto, cannone da 30 mm
        Cockpit dell'A-10C, due MFD
aircraft-uh60 =
    .facts =
        Elicottero utility
        Inserisce truppe: atterrare vicino all'obiettivo
        Ciclico e collettivo
aircraft-ah1z =
    .facts =
        Elicottero d'attacco
        Cannone in torretta, Hellfire, AIM-9
        Cannone e Hellfire seguono il pod

## Sezioni e gruppi

section-theater = Teatro
section-opponents = Avversari
section-situation = Situazione
section-mission = Missione
section-moon = Luna
section-start-time = Ora di partenza
section-weather = Meteo
section-realism = Realismo
section-tacview = Tacview
section-mission-summary = Riepilogo della missione
section-briefing = Briefing
group-mission = Missione
group-conditions = Condizioni
group-vehicle = Velivolo
group-simulation = Simulazione

## Opzioni della missione

menu-start-random = Casuale
    .facts =
        Estratta a ogni nuova missione, come in modalità classica
        06:00-08:00 di giorno o 18:00-20:00 di notte
        Deselezionare per scegliere l'ora
menu-start-slider =
    .facts =
        Ora di decollo
        Notte dalle 18:00 alle 06:00
        Mantenuta da NUOVA MISSIONE
menu-realistic-weapons = Armi realistiche
    .facts =
        I limiti dell'aereo reale
        2 vani armi invece di 4
        Niente missili AIM-9 né AIM-120
        Niente decolli né appontaggi su portaerei
menu-crash-protection = Protezione anti-schianto
    .facts =
        Richiamata automatica vicino al suolo (carrello retratto)
        Contatti con il suolo e collisioni non sono fatali
        Disattivata: il suolo si può sempre colpire
menu-tacview-export = Esportazione
    .facts =
        Salva ogni volo come registrazione Tacview (.acmi)
        Nella cartella del gioco per i file dell'utente, sotto tacview
menu-tacview-real-time = Telemetria in tempo reale
    .facts =
        Invia il volo a Tacview durante il volo
        In Tacview: Record > Real-Time Telemetry
        Indirizzo 127.0.0.1, porta 42674 (--tacview-address)
switch-game = GIOCO
switch-real = REALE
menu-damage = Danni
    .facts =
        Danni delle armi
        Gioco: danni classici; i colpi si sommano ma non abbattono mai l'aereo
        Realistico: un missile può distruggere l'aereo (eiettarsi!), danneggia più sistemi e spesso provoca un incendio, che si spegne o fa esplodere l'aereo
        Realistico: superfici di comando danneggiate sbilanciano il trim o fanno oscillare l'aereo
menu-guidance = Guida
    .facts =
        Guida dei missili
        Classica: i missili volano dritti verso il bersaglio
        Realistica: navigazione proporzionale verso il punto di collisione; i motori si spengono e i missili proseguono planando, perdendo velocità a ogni virata (fumo solo a motore acceso)
        Realistica: i SAM prendono quota contro bersagli lontani; l'AIM-120 vola dritto per 3 s (cabrare per lanciarlo in parabola)
menu-air-defence = Difesa aerea
    .facts =
        Difese aeree a bassa quota
        Classica: solo i siti SAM e i caccia
        Realistica: cannoni ZSU-23-4 guidati dal radar (il loro radar vede l'aereo come quello di un SAM) e batterie contraeree S-60 difendono gli aeroporti e gli obiettivi; squadre MANPADS nelle città lanciano SA-14 su ciò che vedono
        Volare bassi sopra i luoghi difesi diventa pericoloso; bombe e cannone distruggono i pezzi

## Schermata principale

menu-title = STEALTH MISSION 89
menu-subtitle = SISTEMA DI PIANIFICAZIONE MISSIONE
menu-single-mission = MISSIONE SINGOLA
    .facts =
        Una missione nel teatro scelto
        Scegliere le opzioni, pianificare la rotta e l'armamento
menu-career = CARRIERA
    .facts =
        Fascicoli dei piloti: grado, decorazioni, sortite
        Turno di servizio: 12 sortite in un teatro
        Ciò che viene distrutto resta distrutto
menu-campaign = CAMPAGNA DINAMICA
    .facts =
        Campagna dinamica in tempo reale
        Conquistare le città e gli aeroporti del nemico
        Missioni tratte dalla situazione in corso
menu-multiplayer = MULTIGIOCATORE
    .facts = Cooperativa nella campagna dinamica
    .join = Unirsi a un server dell'elenco pubblico
    .host = Unirsi a un server, oppure ospitare una campagna e volarci
menu-options = OPZIONI
    .facts =
        Gioco: stile del cockpit, dell'HUD e dell'avionica, trim automatico
        Grafica, volumi audio, tasti e joystick
        Salvate per i voli successivi
    .short = Gioco, grafica, audio e comandi
menu-download = SCARICARE
    .facts =
        Il gioco desktop per Windows, Linux e macOS
        Con VR e multigiocatore
        Steam Deck e Steam Frame: l'AppImage Linux
web-play = GIOCARE NEL BROWSER
    .facts = Proprio qui, niente da installare
web-download-platforms = Windows, Linux e macOS, Steam Deck e Steam Frame
web-download-version = La versione da scaricare
web-gives-vr = VR
    .why = Volare in VR con qualsiasi visore compatibile OpenXR e con lo Steam Frame
web-gives-hotas = HOTAS
    .why = Supporto completo di gamepad, joystick, manette e pedaliere
web-gives-multiplayer = Multiplayer migliore
    .why = Connessione UDP diretta ai server per meno lag
web-gives-server = Ospitare il proprio server
    .why = Ospitare una campagna coop o avviare un server dedicato
web-gives-head-tracking = Head tracking
    .why = Tracker in stile TrackIR tramite OpenTrack
web-gives-tacview = Tacview
    .why = Registrazioni di volo e telemetria in tempo reale
web-gives-keep = Per sempre Suo
    .why = Se il sito va offline, resta la propria copia
menu-discord = DISCORD
    .facts =
        Volare insieme, segnalare bug e condividere missioni
        Seguire lo sviluppo
menu-quit = USCIRE
    .facts = Torna al desktop
menu-version = VERSIONE { $version }

## Schermata di missione singola

menu-career-sortie = SORTITA DI CARRIERA
menu-tour-sortie = { $pilot }  //  SORTITA { $sortie } DI { $length } DEL TURNO
menu-msn = SISTEMA DI PIANIFICAZIONE MISSIONE  //  MSN { $seed }
menu-no-targets = Nessun obiettivo ancora in piedi per questo turno
menu-plan-route = PIANIFICARE LA ROTTA
    .facts =
        La mappa del teatro con la copertura dei radar e dei SAM nemici
        Aggiungere, spostare e rimuovere punti di navigazione
        Gli obiettivi e la base di atterraggio restano fissi
menu-loadout = ARMAMENTO
    .facts =
        Scegliere i carichi di ogni vano o pilone
        Parte con le armi del briefing
menu-generating = GENERAZIONE...
menu-back-records = Torna ai fascicoli dei piloti
menu-back-main = Torna al menu principale
menu-new-sortie = NUOVA SORTITA
menu-new-mission = NUOVA MISSIONE
    .facts =
        Estrae altri obiettivi
        Mantiene tutte le opzioni e l'ora di partenza
menu-load = CARICARE
    .facts =
        Le missioni salvate in volo (menu: SALVARE LA MISSIONE)
        Riprendere il volo da dove è stato salvato
menu-fly = VOLARE
    .facts = Decollare per la missione del briefing
menu-fly-vr = VOLARE IN VR
    .facts =
        Riavvia il gioco nel visore VR
        Vola esattamente questa missione
    .no-mission = Generare prima una missione
    .no-runtime = Nessun runtime OpenXR (Monado, WiVRn, SteamVR) è attivo
menu-vr-failed = impossibile avviare in VR: { $error }

## Riepilogo della missione

summary-theater = TEATRO
summary-situation = SITUAZIONE
summary-mission = MISSIONE
summary-opponents = AVVERSARI
summary-time = ORA
summary-weather = METEO
summary-fuel = CARBURANTE
summary-night = Notte
summary-day = Giorno
summary-time-value = { $daytime }  part. { $depart }
summary-weather-value = { $sky }  vis. { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Armamento: { $stores }
summary-intelligence = INTELLIGENCE
