# Hovedmenuen og skærmen for enkeltmission (src/menu.rs).

## Operationsområder

theater-libya = Libyen (1986)
    .facts =
        Det Store Syrte og den libyske kyst
        Tiden for Operation El Dorado Canyon
        Klar ørkenhimmel det meste af tiden
theater-persian_gulf = Den Persiske Golf (1984)
    .facts =
        Tiden for tankerkrigen mellem Iran og Irak
        Over vand og kystnær ørken
        Det klareste vejr af alle operationsområder
theater-north_cape = Nordkap
    .facts =
        Arktisk Norge og Kola-halvøen
        Den sovjetiske Nordflådes hjemsted
        Hyppigt lavt skydække
theater-central_europe = Centraleuropa
    .facts =
        NATO mod Warszawapagten over Tyskland
        Klar himmel kun 15 % af tiden
theater-vietnam = Vietnam
    .facts =
        Området omkring Hanoi og Haiphong
        Jungle og bjergrigt terræn
        Tordenvejr i 30 % af missionerne
theater-middle_east = Mellemøsten
    .facts =
        Det østlige Middelhav og Levanten
        For det meste klar himmel
theater-desert_storm = Desert Storm
    .facts =
        Kuwait og Irak, 1991
        F-117A's virkelige kampdebut
        For det meste klar ørkenhimmel
theater-cuba = Cuba
    .facts =
        Ø-operationsområde i Caribien
        Mål til havs og på land
        Tropisk: tordenvejr i 15 % af missionerne
theater-korea = Korea
    .facts =
        Den koreanske halvø nord for DMZ
        Bjergrigt terræn
        Skiftende vejr, enhver himmel er mulig
theater-ukraine = Ukraine (2026)
    .facts =
        Østukraine, Krim og det sydlige Rusland
        Frontlinjen brænder fra Kharkiv til Kherson
        Åbne marker, ødelagte byer ved fronten

## Situationer (engagementsregler)

roe-0 = Den kolde krig
    .facts =
        Hemmelig operation i fredstid
        At forblive uopdaget er det vigtigste
        Færre fjendtlige fly på patrulje
        De præcise engagementsregler står i briefingen
roe-1 = Begrænset krig
    .facts =
        Regional væbnet konflikt
        Fjendens luftforsvar er i alarmberedskab
        Flere fjendtlige fly end under den kolde krig
roe-2 = Konventionel krig
    .facts =
        Fuldskala krig
        Alle fjendtlige styrker er aktive
        Flere fjendtlige fly end under den kolde krig

## Missionstyper

kind-strike = Angreb
    .facts =
        Ødelæg, fotografer eller nå et mål på jorden
        Primært og sekundært mål
        Bevæbning til luft-til-jord
kind-air = Luft-til-luft
    .facts =
        Opfang et fjendtligt fly
        Transportfly, bombefly eller jagerpatruljer
        Bevæbning med luft-til-luft-missiler
kind-strike-training = Angrebstræning
    .facts =
        Øvelsesmission med angreb
        Lær våben og navigation
kind-air-training = Lufttræning
    .facts =
        Øvelsesmission luft-til-luft
        Lær opfangning og missiler
kind-drone-defense = Droneforsvar
    .facts =
        Kun Ukraine
        Stop en bølge af Geran-angrebsdroner, før den rammer en by eller et atomkraftværk
        Bombér derefter deres affyringsplads

## Modstandere

opponents-0 = Grøn
    .facts =
        Uerfarne fjendtlige besætninger
        Letteste indstilling
opponents-1 = Almindelig
    .facts =
        Gennemsnitlige fjendtlige besætninger
        Standardindstilling
opponents-2 = Veteran
    .facts =
        Dygtige fjendtlige besætninger
        Flere skibe og patruljer
opponents-3 = Elite
    .facts =
        Fjendens bedste piloter og SAM-besætninger
        Sværeste indstilling

## Vejr

sky-random = Tilfældig
    .facts =
        Trukket ud fra operationsområdets klima
        Ændres med hver ny mission
        Skyer blokerer sigtelinjen for begge sider
sky-clear = Klar
    .facts =
        Ingen skyer
        Sigtbarhed 40 km
        Den klassiske himmel fra DOS-tiden
sky-scattered = Spredte skyer
    .facts =
        Cumulus 20-45 %, base 2500-6000 ft
        Nogle gange en tynd altostratus højt oppe
        Sigtbarhed 30 km
sky-broken = Opbrudt
    .facts =
        Cumulus 45-70 %, base 2000-5000 ft
        Ofte et lag stratocumulus ovenover
        Sigtbarhed 25 km
sky-overcast = Overskyet
    .facts =
        Stratus 80-100 %, base 1500-5000 ft
        Et par cumulusfragmenter nedenunder
        Sigtbarhed 20 km
sky-storm = Storm
    .facts =
        Cumulonimbustårne op til 40000 ft
        Lav stratus og cumulus fra 1000 ft
        Sigtbarhed 8 km
menu-random-cover = Tilfældigt skydække
    .facts =
        Hovedlagets dækningsgrad trækkes sammen med vejret
        Fjern markeringen for selv at indstille den
        Hovedlag: stratus ved overskyet og storm, ellers cumulus
menu-cover-slider =
    .facts =
        Den del af himlen, hovedlaget dækker
        Ingen virkning ved klar himmel

## Måne

moon-date = Missionsdato
    .facts =
        Den virkelige måne på briefingens dato
        Kræver tidspunkt på dagen (moderne)
moon-0 = Nymåne
    .facts =
        Intet måneskin: sort nat, kun lys og stjerner
        Fjendtlige piloter ser dig på 2 km
moon-1 = Tiltagende segl
    .facts = En tynd måne på aftenhimlen, nede før midnat
moon-2 = Første kvarter
    .facts = Halvt oplyst, oppe om aftenen, går ned omkring midnat
moon-3 = Tiltagende måne
    .facts = Lys, oppe fra eftermiddagen til efter midnat
moon-4 = Fuldmåne
    .facts =
        Oppe hele natten, højt på himlen ved midnat
        Fjendtlige piloter ser dig på 4 km
moon-5 = Aftagende måne
    .facts = Lys, står op om aftenen, oppe til morgen
moon-6 = Sidste kvarter
    .facts = Halvt oplyst, står op omkring midnat
moon-7 = Aftagende segl
    .facts = En tynd måne, der står op før daggry

## Fly

aircraft-f117 =
    .facts =
        Stealth-angrebsfly
        Subsonisk, 7 g
        Moderne og klassisk cockpit
aircraft-f16 =
    .facts =
        Multirolle-jager
        Mach 2-klassen, 9 g, hurtig rulning
        F-16-cockpit med RWR
aircraft-tornado =
    .facts =
        Angrebsfly til lav højde
        Svingvinger, der skifter vinkel med farten
        Moderne cockpit, tre MFD'er
aircraft-a10 =
    .facts =
        Fly til tæt luftstøtte
        Langsomt og robust, 30 mm kanon
        A-10C-cockpit, to MFD'er
aircraft-av8b =
    .facts =
        V/STOL-angrebsfly
        Vektoriseret tryk: svæv, lander hvor som helst
        Dysehåndtag: R / F, Skift for stoppene
aircraft-uh60 =
    .facts =
        Transporthelikopter
        Sætter tropper af: land nær målet
        Cyklisk og kollektiv
aircraft-ah1z =
    .facts =
        Angrebshelikopter
        Tårnkanon, Hellfire, AIM-9
        Kanon og Hellfire følger pod'en

## Sektioner og grupper

section-theater = Operationsområde
section-opponents = Modstandere
section-situation = Situation
section-mission = Mission
section-moon = Måne
section-start-time = Starttidspunkt
section-weather = Vejr
section-realism = Realisme
section-tacview = Tacview
section-mission-summary = Missionsoversigt
section-briefing = Briefing
group-mission = Mission
group-conditions = Forhold
group-vehicle = Fartøj
group-simulation = Simulation

## Missionens indstillinger

menu-start-random = Tilfældig
    .facts =
        Trækkes ved hver ny mission, som i klassisk tilstand
        06:00-08:00 om dagen eller 18:00-20:00 om natten
        Fjern markeringen for selv at vælge tidspunktet
menu-start-slider =
    .facts =
        Starttidspunkt
        Nat fra 18:00 til 06:00
        Bevares af NY MISSION
menu-realistic-weapons = Realistiske våben
    .facts =
        Det rigtige flys begrænsninger
        2 våbenskakte i stedet for 4
        Ingen AIM-9- eller AIM-120-missiler
        Ingen start eller landing på hangarskib
menu-crash-protection = Havariværn
    .facts =
        Trækker automatisk op nær jorden (understel oppe)
        Jordkontakt og kollisioner er ikke dødelige
        Fra: jorden kan altid ramles
menu-tacview-export = Eksport
    .facts =
        Gemmer hver flyvning som en Tacview-optagelse (.acmi)
        I spillets mappe til dine filer, under tacview
menu-tacview-real-time = Telemetri i realtid
    .facts =
        Sender flyvningen til Tacview, mens du flyver
        I Tacview: Record > Real-Time Telemetry
        Adresse 127.0.0.1, port 42674 (--tacview-address)
switch-game = SPIL
switch-real = VIRKELIG
menu-damage = Skade
    .facts =
        Våbenskade
        Spil: klassisk skade; træffere lægges sammen, men fælder aldrig flyet
        Realistisk: et missiltræf kan ødelægge flyet (skyd dig ud!), beskadiger flere systemer og starter ofte en brand, som brænder ud eller får flyet til at eksplodere
        Realistisk: beskadigede ror tager flyet ud af trim eller får det til at svinge
menu-guidance = Målsøgning
    .facts =
        Missilernes målsøgning
        Klassisk: missiler flyver lige mod deres mål
        Realistisk: proportional navigation mod kollisionspunktet; motorerne brænder ud, og missilerne glider videre og mister fart ved hver drejning (røg kun mens motoren brænder)
        Realistisk: SAM'er går i højden mod fjerne mål; AIM-120 flyver lige i 3 s (træk op for at lobbe den)
menu-air-defence = Luftforsvar
    .facts =
        Luftforsvar i lav højde
        Klassisk: kun SAM-stillinger og jagere
        Realistisk: radarstyrede ZSU-23-4-kanoner (deres radar ser dig som en SAM's) og S-60-flakbatterier vogter flyvepladser og mål; MANPADS-hold i byerne affyrer SA-14 mod det, de ser
        Det bliver farligt at flyve lavt over forsvarede steder; bomber og kanon ødelægger kanonerne

## Hovedskærm

menu-title = STEALTH MISSION 89
menu-subtitle = MISSIONSPLANLÆGNINGSSYSTEM
menu-single-mission = ENKELTMISSION
    .facts =
        Én mission i det operationsområde, du vælger
        Vælg indstillinger, planlæg ruten og bevæbningen
menu-career = KARRIERE
    .facts =
        Pilotjournaler: grad, dekorationer, flyvninger
        Tjenesteperiode: 12 flyvninger i ét operationsområde
        Det, du ødelægger, forbliver ødelagt
menu-campaign = DYNAMISK KAMPAGNE
    .facts =
        Dynamisk kampagne i realtid
        Indtag fjendens byer og flyvepladser
        Missioner ud fra den aktuelle situation
menu-multiplayer = MULTIPLAYER
    .facts = Coop i den dynamiske kampagne
    .join = Deltag på en server fra den offentlige liste
    .host = Deltag på en server, eller vær vært for en kampagne og flyv i den
menu-options = INDSTILLINGER
    .facts =
        Spil: cockpit, HUD og avionikstil, automatisk trim
        Grafik, lydstyrker, taste- og joystickbindinger
        Gemmes til de næste flyvninger
    .short = Spil, grafik, lyd og styring
menu-download = DOWNLOAD
    .facts =
        Computerspillet til Windows, Linux og macOS
        Med VR og multiplayer
        Steam Deck og Steam Frame: Linux-AppImage
web-play = SPIL I BROWSEREN
    .facts = Lige her, ingen installation
web-download-platforms = Windows, Linux og macOS, Steam Deck og Steam Frame
web-download-version = Download-versionen
web-gives-vr = VR
    .why = Flyv i VR med et vilkårligt OpenXR-kompatibelt headset og Steam Frame
web-gives-hotas = HOTAS
    .why = Fuld understøttelse af gamepads, joysticks, gashåndtag og pedaler
web-gives-multiplayer = Bedre multiplayer
    .why = Direkte UDP-forbindelse til servere for mindre lag
web-gives-server = Kør din egen server
    .why = Vær vært for en coop-kampagne, eller kør en dedikeret server
web-gives-head-tracking = Head tracking
    .why = Trackere i TrackIR-stil via OpenTrack
web-gives-tacview = Tacview
    .why = Flyoptagelser og telemetri i realtid
web-gives-keep = Din for altid
    .why = Hvis hjemmesiden forsvinder, har du stadig din egen kopi
menu-discord = DISCORD
    .facts =
        Flyv sammen, rapportér fejl og del missioner
        Følg udviklingen
menu-quit = AFSLUT
    .facts = Tilbage til skrivebordet
menu-version = VERSION { $version }

## Skærm for enkeltmission

menu-career-sortie = KARRIEREFLYVNING
menu-tour-sortie = { $pilot }  //  TOGTFLYVNING { $sortie } AF { $length }
menu-msn = MISSIONSPLANLÆGNINGSSYSTEM  //  MSN { $seed }
menu-no-targets = Ingen mål står tilbage i denne tjenesteperiode
menu-plan-route = PLANLÆG RUTE
    .facts =
        Kortet over operationsområdet med fjendens radar- og SAM-dækning
        Tilføj, flyt og fjern navigationspunkter
        Mål og landingsbasen ligger fast
menu-loadout = BEVÆBNING
    .facts =
        Vælg last på hver skakt eller pylon
        Starter med de briefede våben
menu-generating = GENERERER...
menu-back-records = Tilbage til pilotjournalerne
menu-back-main = Tilbage til hovedmenuen
menu-new-sortie = NY FLYVNING
menu-new-mission = NY MISSION
    .facts =
        Trækker andre mål
        Bevarer alle indstillinger og starttidspunktet
menu-load = INDLÆS
    .facts =
        Missioner gemt under flyvning (menu: GEM MISSION)
        Flyv videre fra der, hvor de blev gemt
menu-fly = FLYV
    .facts = Start på den briefede mission
menu-fly-vr = FLYV I VR
    .facts =
        Genstarter spillet i VR-headsettet
        Flyver præcis denne mission
    .no-mission = Generér først en mission
    .no-runtime = Ingen OpenXR-runtime (Monado, WiVRn, SteamVR) er aktiv
menu-vr-failed = kunne ikke starte i VR: { $error }

## Missionsoversigt

summary-theater = OPERATIONSOMRÅDE
summary-situation = SITUATION
summary-mission = MISSION
summary-opponents = MODSTANDERE
summary-time = TID
summary-weather = VEJR
summary-fuel = BRÆNDSTOF
summary-night = Nat
summary-day = Dag
summary-time-value = { $daytime }  afg. { $depart }
summary-weather-value = { $sky }  sigt { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Bevæbning: { $stores }
summary-intelligence = EFTERRETNING
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
