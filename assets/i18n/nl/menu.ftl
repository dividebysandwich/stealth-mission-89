# Het hoofdmenu en het scherm voor een enkele missie (src/menu.rs).

## Operatiegebieden (stealth_host::theater::THEATERS)

theater-libya = Libië (1986)
    .facts =
        De Golf van Sidra en de Libische kust
        Tijdperk van Operation El Dorado Canyon
        De meeste dagen heldere woestijnluchten
theater-persian_gulf = Perzische Golf (1984)
    .facts =
        Tijdperk van de tankeroorlog tussen Iran en Irak
        Boven water en kustwoestijn
        Het helderste weer van alle operatiegebieden
theater-north_cape = Noordkaap
    .facts =
        Arctisch Noorwegen en het schiereiland Kola
        Thuishaven van de Sovjet-Noordelijke Vloot
        Vaak lage bewolking
theater-central_europe = Centraal-Europa
    .facts =
        NAVO tegen Warschaupact boven Duitsland
        Slechts 15% van de tijd een heldere lucht
theater-vietnam = Vietnam
    .facts =
        De regio Hanoi en Haiphong
        Jungle en bergachtig terrein
        Onweer in 30% van de missies
theater-middle_east = Midden-Oosten
    .facts =
        Het oostelijke Middellandse Zeegebied en de Levant
        Meestal heldere luchten
theater-desert_storm = Desert Storm
    .facts =
        Koeweit en Irak, 1991
        Het echte gevechtsdebuut van de F-117A
        Meestal heldere woestijnluchten
theater-cuba = Cuba
    .facts =
        Operatiegebied op een Caribisch eiland
        Doelen op zee en op land
        Tropisch: onweer in 15% van de missies
theater-korea = Korea
    .facts =
        Het Koreaanse schiereiland ten noorden van de DMZ
        Bergachtig terrein
        Wisselvallig weer, elke lucht is mogelijk
theater-ukraine = Oekraïne (2026)
    .facts =
        Oost-Oekraïne, de Krim en Zuid-Rusland
        Het front van Charkiv tot Cherson staat in brand
        Open akkerland, verwoeste steden aan het front

## Situaties (rules of engagement)

roe-0 = Koude Oorlog
    .facts =
        Geheime operatie in vredestijd
        Onopgemerkt blijven is het belangrijkst
        Minder vijandelijke vliegtuigen op patrouille
        De exacte rules of engagement staan in de briefing
roe-1 = Beperkte oorlog
    .facts =
        Regionaal gewapend conflict
        Vijandelijke luchtverdediging in staat van paraatheid
        Meer vijandelijke vliegtuigen dan in de Koude Oorlog
roe-2 = Conventionele oorlog
    .facts =
        Oorlog op volle schaal
        Alle vijandelijke strijdkrachten actief
        Meer vijandelijke vliegtuigen dan in de Koude Oorlog

## Soorten missies

kind-strike = Aanval
    .facts =
        Een doel op de grond vernietigen, fotograferen of bereiken
        Primair en secundair doel
        Lucht-grondbewapening
kind-air = Lucht-lucht
    .facts =
        Een vijandelijk vliegtuig onderscheppen
        Transportvliegtuigen, bommenwerpers of jagerpatrouilles
        Bewapening met lucht-luchtraketten
kind-strike-training = Aanvalstraining
    .facts =
        Oefenmissie voor een aanval
        Leer de wapens en de navigatie
kind-air-training = Luchttraining
    .facts =
        Lucht-luchtoefenmissie
        Leer onderscheppen en raketten
kind-drone-defense = Droneverdediging
    .facts =
        Alleen Oekraïne
        Stop een golf Geran-aanvalsdrones voor ze een stad of een kerncentrale treffen
        Bombardeer daarna hun lanceerlocatie

## Tegenstanders

opponents-0 = Groen
    .facts =
        Onervaren vijandelijke bemanningen
        Makkelijkste instelling
opponents-1 = Regulier
    .facts =
        Gemiddelde vijandelijke bemanningen
        Standaardinstelling
opponents-2 = Veteraan
    .facts =
        Bekwame vijandelijke bemanningen
        Meer schepen en patrouilles
opponents-3 = Elite
    .facts =
        De beste vijandelijke piloten en SAM-bemanningen
        Moeilijkste instelling

## Weer

sky-random = Willekeurig
    .facts =
        Bepaald door het klimaat van het operatiegebied
        Verandert bij elke nieuwe missie
        Wolken blokkeren het zicht voor beide partijen
sky-clear = Helder
    .facts =
        Geen wolken
        Zicht 40 km
        De klassieke lucht uit het DOS-tijdperk
sky-scattered = Verspreid
    .facts =
        Cumulus 20-45%, basis 2500-6000 ft
        Soms een dunne altostratus hoog erboven
        Zicht 30 km
sky-broken = Gebroken
    .facts =
        Cumulus 45-70%, basis 2000-5000 ft
        Vaak een laag stratocumulus erboven
        Zicht 25 km
sky-overcast = Bewolkt
    .facts =
        Stratus 80-100%, basis 1500-5000 ft
        Enkele cumulusflarden eronder
        Zicht 20 km
sky-storm = Onweer
    .facts =
        Cumulonimbustorens tot 40000 ft
        Lage stratus en cumulus vanaf 1000 ft
        Zicht 8 km
menu-random-cover = Willekeurige bedekking
    .facts =
        Bedekking van de hoofdwolkenlaag bepaald met het weer
        Uitvinken om zelf in te stellen
        Hoofdlaag: stratus bij bewolkt en onweer, anders cumulus
menu-cover-slider =
    .facts =
        Aandeel van de lucht dat de hoofdlaag bedekt
        Geen effect bij een heldere lucht

## Maan

moon-date = Missiedatum
    .facts =
        De echte maan van de briefingdatum
        Vereist de schakelaar voor het tijdstip van de dag (modern)
moon-0 = Nieuwe maan
    .facts =
        Geen maanlicht: een pikzwarte nacht, alleen lichten en sterren
        Vijandelijke piloten zien je op 2 km
moon-1 = Wassende sikkel
    .facts = Een dunne maan aan de avondhemel, onder voor middernacht
moon-2 = Eerste kwartier
    .facts = Half verlicht, 's avonds aan de hemel, onder rond middernacht
moon-3 = Wassende maan
    .facts = Helder, aan de hemel vanaf de middag tot na middernacht
moon-4 = Volle maan
    .facts =
        De hele nacht aan de hemel, hoog om middernacht
        Vijandelijke piloten zien je op 4 km
moon-5 = Afnemende maan
    .facts = Helder, komt 's avonds op, aan de hemel tot de ochtend
moon-6 = Laatste kwartier
    .facts = Half verlicht, komt rond middernacht op
moon-7 = Afnemende sikkel
    .facts = Een dunne maan die voor het ochtendgloren opkomt

## Vliegtuigen

aircraft-f117 =
    .facts =
        Stealth-aanvalsvliegtuig
        Subsonisch, 7 g
        Moderne en klassieke cockpit
aircraft-f16 =
    .facts =
        Multirole-jager
        Klasse Mach 2, 9 g, snelle rol
        F-16-cockpit met RWR
aircraft-tornado =
    .facts =
        Aanvalsvliegtuig voor lage hoogte
        Zwenkvleugels draaien mee met de snelheid
        Moderne cockpit, drie MFD's
aircraft-a10 =
    .facts =
        Vliegtuig voor close air support
        Traag en robuust, 30 mm-kanon
        A-10C-cockpit, twee MFD's
aircraft-av8b =
    .facts =
        V/STOL-aanvalsvliegtuig
        Stuwkrachtvectoring: zweven, landt overal
        Straalpijphendel: R / F, Shift voor de aanslagen
aircraft-uh60 =
    .facts =
        Transporthelikopter
        Zet troepen af: land bij het doel
        Cyclic en collective
aircraft-ah1z =
    .facts =
        Gevechtshelikopter
        Kanon in een koepel, Hellfires, AIM-9
        Kanon en Hellfires volgen de pod

## Secties en groepen

section-theater = Operatiegebied
section-opponents = Tegenstanders
section-situation = Situatie
section-mission = Missie
section-moon = Maan
section-start-time = Starttijd
section-weather = Weer
section-realism = Realisme
section-tacview = Tacview
section-mission-summary = Missieoverzicht
section-briefing = Briefing
group-mission = Missie
group-conditions = Omstandigheden
group-vehicle = Voertuig
group-simulation = Simulatie

## Opties van de missie

menu-start-random = Willekeurig
    .facts =
        Bepaald bij elke nieuwe missie, zoals in de klassieke modus
        06:00-08:00 overdag of 18:00-20:00 's nachts
        Uitvinken om het uur te kiezen
menu-start-slider =
    .facts =
        Tijdstip van de start
        Nacht van 18:00 tot 06:00
        Behouden door NIEUWE MISSIE
menu-realistic-weapons = Realistische wapens
    .facts =
        De beperkingen van het echte vliegtuig
        2 wapenschachten in plaats van 4
        Geen AIM-9- of AIM-120-raketten
        Geen starts of landingen op een vliegdekschip
menu-crash-protection = Crashbeveiliging
    .facts =
        Trekt automatisch op vlak bij de grond (landingsgestel ingetrokken)
        Contact met de grond en botsingen zijn niet dodelijk
        Uit: de grond kan altijd worden geraakt
menu-tacview-export = Exporteren
    .facts =
        Slaat elke vlucht op als Tacview-opname (.acmi)
        In de spelmap voor je bestanden, onder tacview
menu-tacview-real-time = Realtime telemetrie
    .facts =
        Stuurt de vlucht naar Tacview terwijl je vliegt
        In Tacview: Record > Real-Time Telemetry
        Adres 127.0.0.1, poort 42674 (--tacview-address)
switch-game = SPEL
switch-real = ECHT
menu-damage = Schade
    .facts =
        Wapenschade
        Spel: klassieke schade; treffers tellen op maar halen het vliegtuig nooit neer
        Realistisch: een raketinslag kan het vliegtuig vernietigen (uitschieten!), beschadigt meer systemen en veroorzaakt vaak brand, die uitbrandt of het vliegtuig doet ontploffen
        Realistisch: beschadigde stuurvlakken brengen het vliegtuig uit trim of laten het oscilleren
menu-guidance = Geleiding
    .facts =
        Raketgeleiding
        Klassiek: raketten die recht op hun doel afvliegen
        Realistisch: proportionele navigatie naar het botsingspunt; motoren branden uit en de raketten glijden door en verliezen snelheid bij elke bocht (rook alleen terwijl de motor brandt)
        Realistisch: SAM's vliegen een boog naar verre doelen; de AIM-120 vliegt 3 s rechtdoor (trek op om hem te lobben)
menu-air-defence = Luchtverdediging
    .facts =
        Luchtverdediging op lage hoogte
        Klassiek: alleen de SAM-stellingen en jagers
        Realistisch: radargeleide ZSU-23-4-kanonnen (hun radar ziet je zoals die van een SAM) en S-60-flakbatterijen bewaken de vliegvelden en de doelen; MANPADS-teams in de steden vuren SA-14's af op wat ze zien
        Laag vliegen boven verdedigde plaatsen wordt gevaarlijk; bommen en kanonnen vernietigen de kanonnen

## Hoofdscherm

menu-title = STEALTH MISSION 89
menu-subtitle = MISSION PLANNING SYSTEM
menu-single-mission = ENKELE MISSIE
    .facts =
        Eén missie in het operatiegebied van je keuze
        Kies de opties, plan de route en de bewapening
menu-career = LOOPBAAN
    .facts =
        Pilotendossiers: rang, onderscheidingen, vluchten
        Diensttijd: 12 vluchten in één operatiegebied
        Wat je vernietigt, blijft vernietigd
menu-campaign = DYNAMISCHE CAMPAGNE
    .facts =
        Dynamische campagne in real time
        Neem de steden en vliegvelden van de vijand in
        Missies op basis van de actuele situatie
menu-multiplayer = MULTIPLAYER
    .facts = Coöp in de dynamische campagne
    .join = Verbind met een server van de openbare lijst
    .host = Verbind met een server, of host een campagne en vlieg erin
menu-options = OPTIES
    .facts =
        Spel: stijl van cockpit, HUD en avionica, automatische trim
        Grafisch, geluidsvolumes, toetsen- en joystickinstellingen
        Bewaard voor de volgende vluchten
    .short = Spel, grafisch, geluid en besturing
menu-download = DOWNLOADEN
    .facts =
        Het desktopspel voor Windows, Linux en macOS
        Met VR en multiplayer
        Steam Deck en Steam Frame: de Linux-AppImage
web-play = SPELEN IN DE BROWSER
    .facts = Hier en nu, niets te installeren
web-download-platforms = Windows, Linux en macOS, Steam Deck en Steam Frame
web-download-version = De downloadversie
web-gives-vr = VR
    .why = Vlieg in VR met elke OpenXR-compatibele headset en de Steam Frame
web-gives-hotas = HOTAS
    .why = Volledige ondersteuning voor gamepads, joysticks, gashendels en pedalen
web-gives-multiplayer = Betere multiplayer
    .why = Directe UDP-verbinding met servers voor minder lag
web-gives-server = Host je eigen server
    .why = Host een coöpcampagne, of draai een dedicated server
web-gives-head-tracking = Hoofdtracking
    .why = Trackers in TrackIR-stijl via OpenTrack
web-gives-tacview = Tacview
    .why = Vluchtopnames en realtime telemetrie
web-gives-keep = Voor altijd van jou
    .why = Als de website uitvalt, heb je nog je eigen kopie
menu-discord = DISCORD
    .facts =
        Vlieg samen, meld bugs en deel missies
        Volg de ontwikkeling
menu-quit = AFSLUITEN
    .facts = Terug naar het bureaublad

menu-version = VERSIE { $version }

## Scherm voor een enkele missie

menu-career-sortie = LOOPBAANVLUCHT
menu-tour-sortie = { $pilot }  //  DIENSTTIJD VLUCHT { $sortie } VAN { $length }
menu-msn = MISSION PLANNING SYSTEM  //  MSN { $seed }
menu-no-targets = Geen doelen meer over voor deze diensttijd
menu-plan-route = ROUTE PLANNEN
    .facts =
        De kaart van het operatiegebied met de vijandelijke radar- en SAM-dekking
        Navigatiepunten toevoegen, verplaatsen en verwijderen
        Doelen en de landingsbasis liggen vast
menu-loadout = BEWAPENING
    .facts =
        Kies de stores op elke schacht of pylon
        Begint met de gebriefde wapens
menu-generating = GENEREREN...
menu-back-records = Terug naar de pilotendossiers
menu-back-main = Terug naar het hoofdmenu
menu-new-sortie = NIEUWE VLUCHT
menu-new-mission = NIEUWE MISSIE
    .facts =
        Bepaalt andere doelen
        Behoudt alle opties en de starttijd
menu-load = LADEN
    .facts =
        In de vlucht opgeslagen missies (menu: MISSIE OPSLAAN)
        Vlieg verder vanaf waar ze zijn opgeslagen
menu-fly = VLIEGEN
    .facts = Opstijgen voor de gebriefde missie
menu-fly-vr = VLIEGEN IN VR
    .facts =
        Start het spel opnieuw in de VR-headset
        Vliegt exact deze missie
    .no-mission = Genereer eerst een missie
    .no-runtime = Er is geen OpenXR-runtime (Monado, WiVRn, SteamVR) actief
menu-vr-failed = kon niet in VR starten: { $error }

## Missieoverzicht

summary-theater = OPERATIEGEBIED
summary-situation = SITUATIE
summary-mission = MISSIE
summary-opponents = TEGENSTANDERS
summary-time = TIJD
summary-weather = WEER
summary-fuel = BRANDSTOF
summary-night = Nacht
summary-day = Dag
summary-time-value = { $daytime }  vertrek { $depart }
summary-weather-value = { $sky }  zicht { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Bewapening: { $stores }
summary-intelligence = INLICHTINGEN
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
