# Huvudmenyn och skärmen för enstaka uppdrag (src/menu.rs).

## Operationsområden

theater-libya = Libyen (1986)
    .facts =
        Stora Syrten och den libyska kusten
        Tiden för Operation El Dorado Canyon
        Klar ökenhimmel de flesta dagar
theater-persian_gulf = Persiska viken (1984)
    .facts =
        Tiden för tankerkriget mellan Iran och Irak
        Över hav och kustöken
        Klaraste vädret av alla operationsområden
theater-north_cape = Nordkap
    .facts =
        Arktiska Norge och Kolahalvön
        Sovjetiska Nordflottans hemmahamn
        Ofta lågt molntäcke
theater-central_europe = Centraleuropa
    .facts =
        Nato mot Warszawapakten över Tyskland
        Klar himmel bara 15 % av tiden
theater-vietnam = Vietnam
    .facts =
        Området kring Hanoi och Haiphong
        Djungel och bergig terräng
        Åskväder i 30 % av uppdragen
theater-middle_east = Mellanöstern
    .facts =
        Östra Medelhavet och Levanten
        Oftast klar himmel
theater-desert_storm = Desert Storm
    .facts =
        Kuwait och Irak, 1991
        F-117A:s verkliga stridsdebut
        Oftast klar ökenhimmel
theater-cuba = Kuba
    .facts =
        Operationsområde på en karibisk ö
        Mål till havs och på land
        Tropiskt: oväder i 15 % av uppdragen
theater-korea = Korea
    .facts =
        Koreahalvön norr om DMZ
        Bergig terräng
        Växlande väder, alla himlar är möjliga
theater-ukraine = Ukraina (2026)
    .facts =
        Östra Ukraina, Krim och södra Ryssland
        Frontlinjen brinner från Charkiv till Cherson
        Öppen jordbruksmark, förstörda städer vid fronten

## Lägen (insatsregler)

roe-0 = Kalla kriget
    .facts =
        Dold operation i fredstid
        Att förbli oupptäckt är viktigast
        Färre fientliga flygplan på patrull
        De exakta insatsreglerna står i briefingen
roe-1 = Begränsat krig
    .facts =
        Regional väpnad konflikt
        Fiendens luftvärn i beredskap
        Fler fientliga flygplan än under kalla kriget
roe-2 = Konventionellt krig
    .facts =
        Fullskaligt krig
        Alla fientliga styrkor är aktiva
        Fler fientliga flygplan än under kalla kriget

## Uppdragstyper

kind-strike = Anfall
    .facts =
        Förstöra, fotografera eller nå ett markmål
        Primärt och sekundärt mål
        Beväpning för luft-mark
kind-air = Luft-luft
    .facts =
        Avfånga ett fientligt flygplan
        Transportflygplan, bombplan eller jaktpatruller
        Beväpning med luft-luft-robotar
kind-strike-training = Anfallsövning
    .facts =
        Övningsuppdrag för anfall
        Lär dig vapen och navigering
kind-air-training = Luftstridsövning
    .facts =
        Övningsuppdrag för luft-luft
        Lär dig avfångning och robotar
kind-drone-defense = Drönarförsvar
    .facts =
        Endast Ukraina
        Stoppa en våg av Geran-attackdrönare innan den når en stad eller ett kärnkraftverk
        Bomba sedan deras utskjutningsplats

## Motståndare

opponents-0 = Nybörjare
    .facts =
        Oerfarna fientliga besättningar
        Lättaste nivån
opponents-1 = Normal
    .facts =
        Genomsnittliga fientliga besättningar
        Standardnivå
opponents-2 = Veteran
    .facts =
        Skickliga fientliga besättningar
        Fler fartyg och patruller
opponents-3 = Elit
    .facts =
        Fiendens bästa piloter och SAM-besättningar
        Svåraste nivån

## Väder

sky-random = Slumpmässigt
    .facts =
        Dras ur operationsområdets klimat
        Ändras för varje nytt uppdrag
        Moln skymmer sikten för båda sidor
sky-clear = Klart
    .facts =
        Inga moln
        Sikt 40 km
        Den klassiska himlen från DOS-eran
sky-scattered = Lätt molnighet
    .facts =
        Cumulus 20-45 %, bas 2500-6000 ft
        Ibland en tunn altostratus högt upp
        Sikt 30 km
sky-broken = Molnigt
    .facts =
        Cumulus 45-70 %, bas 2000-5000 ft
        Ofta ett lager stratocumulus ovanför
        Sikt 25 km
sky-overcast = Mulet
    .facts =
        Stratus 80-100 %, bas 1500-5000 ft
        Några cumulusfragment under
        Sikt 20 km
sky-storm = Oväder
    .facts =
        Cumulonimbustorn upp till 40000 ft
        Låg stratus och cumulus från 1000 ft
        Sikt 8 km
menu-random-cover = Slumpad molnighet
    .facts =
        Huvudlagrets molntäcke dras med vädret
        Avmarkera för att ställa in det själv
        Huvudlager: stratus vid mulet och oväder, annars cumulus
menu-cover-slider =
    .facts =
        Andel av himlen som huvudlagret täcker
        Ingen verkan vid klar himmel

## Månen

moon-date = Uppdragets datum
    .facts =
        Den verkliga månen på briefingens datum
        Kräver omkopplaren för tid på dygnet (modern)
moon-0 = Nymåne
    .facts =
        Inget månljus: kolsvart natt, bara ljus och stjärnor
        Fientliga piloter ser dig på 2 km
moon-1 = Tilltagande skära
    .facts = En tunn måne på kvällshimlen, går ned före midnatt
moon-2 = Första kvarteret
    .facts = Halvt belyst, uppe på kvällen, går ned runt midnatt
moon-3 = Tilltagande måne
    .facts = Klar, uppe från eftermiddagen till efter midnatt
moon-4 = Fullmåne
    .facts =
        Uppe hela natten, högt vid midnatt
        Fientliga piloter ser dig på 4 km
moon-5 = Avtagande måne
    .facts = Klar, går upp på kvällen, uppe till morgonen
moon-6 = Sista kvarteret
    .facts = Halvt belyst, går upp runt midnatt
moon-7 = Avtagande skära
    .facts = En tunn måne som går upp före gryningen

## Flygplan

aircraft-f117 =
    .facts =
        Attackflygplan med smygteknik
        Underljud, 7 g
        Modern och klassisk cockpit
aircraft-f16 =
    .facts =
        Mångsidigt jaktflygplan
        Mach 2-klass, 9 g, snabb rullning
        F-16-cockpit med RWR
aircraft-tornado =
    .facts =
        Attackflygplan för låg höjd
        Svepvingar som ändrar pilform med farten
        Modern cockpit, tre MFD
aircraft-a10 =
    .facts =
        Flygplan för nära understöd
        Långsamt och robust, 30 mm kanon
        A-10C-cockpit, två MFD
aircraft-uh60 =
    .facts =
        Transporthelikopter
        Sätter av trupp: landa nära målet
        Cyklisk och kollektiv
aircraft-ah1z =
    .facts =
        Attackhelikopter
        Kanon i torn, Hellfire, AIM-9
        Kanonen och Hellfire följer pod:en

## Sektioner och grupper

section-theater = Operationsområde
section-opponents = Motståndare
section-situation = Läge
section-mission = Uppdrag
section-moon = Månen
section-start-time = Starttid
section-weather = Väder
section-realism = Realism
section-tacview = Tacview
section-mission-summary = Uppdragssammanfattning
section-briefing = Briefing
group-mission = Uppdrag
group-conditions = Förhållanden
group-vehicle = Farkost
group-simulation = Simulering

## Uppdragets alternativ

menu-start-random = Slumpmässig
    .facts =
        Dras för varje nytt uppdrag, som i klassiskt läge
        06:00-08:00 på dagen eller 18:00-20:00 på natten
        Avmarkera för att välja timme
menu-start-slider =
    .facts =
        Starttid
        Natt från 18:00 till 06:00
        Behålls av NYTT UPPDRAG
menu-realistic-weapons = Realistiska vapen
    .facts =
        Det verkliga flygplanets begränsningar
        2 vapenutrymmen i stället för 4
        Inga AIM-9- eller AIM-120-robotar
        Inga starter eller landningar på hangarfartyg
menu-crash-protection = Kraschskydd
    .facts =
        Rättar upp automatiskt nära marken (med infällt landningsställ)
        Markkontakt och kollisioner är inte dödliga
        Av: marken kan alltid träffas
menu-tacview-export = Export
    .facts =
        Sparar varje flygning som en Tacview-inspelning (.acmi)
        I spelets mapp för dina filer, under tacview
menu-tacview-real-time = Telemetri i realtid
    .facts =
        Skickar flygningen till Tacview medan du flyger
        I Tacview: Record > Real-Time Telemetry
        Adress 127.0.0.1, port 42674 (--tacview-address)
switch-game = SPEL
switch-real = VERKLIGT
menu-damage = Skador
    .facts =
        Vapnens skadeverkan
        Spel: klassiska skador; träffar läggs ihop men fäller aldrig flygplanet
        Realistiskt: en robotträff kan förstöra flygplanet (katapultera!), skadar fler system och orsakar ofta brand, som slocknar eller får flygplanet att explodera
        Realistiskt: skadade roder gör att flygplanet kommer ur trim eller börjar svänga
menu-guidance = Målsökning
    .facts =
        Robotarnas målsökning
        Klassisk: robotar som flyger rakt mot sitt mål
        Realistiskt: proportionell navigering mot kollisionspunkten; motorerna brinner ut och robotarna glider vidare och tappar fart i varje sväng (rök bara medan motorn brinner)
        Realistiskt: SAM-robotar går upp i en båge mot avlägsna mål; AIM-120 flyger rakt i 3 s (dra upp för att lobba den)
menu-air-defence = Luftvärn
    .facts =
        Luftvärn på låg höjd
        Klassiskt: bara SAM-ställningarna och jaktflygplanen
        Realistiskt: radarstyrda ZSU-23-4-kanoner (deras radar ser dig som en SAM:s) och S-60-luftvärnsbatterier bevakar flygfälten och målen; MANPADS-grupper i städerna avfyrar SA-14 mot det de ser
        Att flyga lågt över försvarade platser blir farligt; bomber och kanon förstör pjäserna

## Huvudskärm

menu-title = STEALTH MISSION 89
menu-subtitle = UPPDRAGSPLANERINGSSYSTEM
menu-single-mission = ENSTAKA UPPDRAG
    .facts =
        Ett uppdrag i valfritt operationsområde
        Välj alternativ, planera rutten och beväpningen
menu-career = KARRIÄR
    .facts =
        Pilotjournaler: grad, dekorationer, flygningar
        Tjänstgöringstid: 12 flygningar i ett operationsområde
        Det du förstör förblir förstört
menu-campaign = DYNAMISK KAMPANJ
    .facts =
        Dynamisk kampanj i realtid
        Ta fiendens städer och flygfält
        Uppdrag utifrån det aktuella läget
menu-multiplayer = FLERSPELARE
    .facts = Samarbete i den dynamiska kampanjen
    .join = Anslut till en server på den publika listan
    .host = Anslut till en server, eller värd en kampanj och flyg i den
menu-options = ALTERNATIV
    .facts =
        Spel: stil för cockpit, HUD och avionik, automatisk trim
        Grafik, ljudvolymer, tangent- och joystickinställningar
        Sparas till nästa flygningar
    .short = Spel, grafik, ljud och styrning
menu-download = LADDA NER
    .facts =
        Skrivbordsspelet för Windows, Linux och macOS
        Med VR och flerspelare
        Steam Deck och Steam Frame: Linux-AppImage
web-play = SPELA I WEBBLÄSAREN
    .facts = Precis här, inget att installera
web-download-platforms = Windows, Linux och macOS, Steam Deck och Steam Frame
web-download-version = Nedladdningsversionen
web-gives-vr = VR
    .why = Flyg i VR med valfritt OpenXR-kompatibelt headset och Steam Frame
web-gives-hotas = HOTAS
    .why = Fullt stöd för handkontroller, joysticks, gasreglage och pedaler
web-gives-multiplayer = Bättre flerspelare
    .why = Direkt UDP-anslutning till servrar för mindre fördröjning
web-gives-server = Var värd för din egen server
    .why = Var värd för en samarbetskampanj, eller kör en dedikerad server
web-gives-head-tracking = Head tracking
    .why = Trackers av TrackIR-typ via OpenTrack
web-gives-tacview = Tacview
    .why = Flyginspelningar och telemetri i realtid
web-gives-keep = Din att behålla
    .why = Om webbplatsen försvinner har du fortfarande din egen kopia
menu-discord = DISCORD
    .facts =
        Flyg tillsammans, rapportera buggar och dela uppdrag
        Följ utvecklingen
menu-quit = AVSLUTA
    .facts = Tillbaka till skrivbordet
menu-version = VERSION { $version }

## Skärmen för enstaka uppdrag

menu-career-sortie = KARRIÄRFLYGNING
menu-tour-sortie = { $pilot }  //  TJÄNSTGÖRINGSFLYGNING { $sortie } AV { $length }
menu-msn = UPPDRAGSPLANERINGSSYSTEM  //  MSN { $seed }
menu-no-targets = Inga mål står kvar för denna tjänstgöringstid
menu-plan-route = PLANERA RUTT
    .facts =
        Operationsområdets karta med fiendens radar- och SAM-täckning
        Lägg till, flytta och ta bort navigeringspunkter
        Målen och landningsbasen ligger fast
menu-loadout = BEVÄPNING
    .facts =
        Välj last på varje utrymme eller pylon
        Börjar med briefingens vapen
menu-generating = GENERERAR...
menu-back-records = Tillbaka till pilotjournalerna
menu-back-main = Tillbaka till huvudmenyn
menu-new-sortie = NY FLYGNING
menu-new-mission = NYTT UPPDRAG
    .facts =
        Drar andra mål
        Behåller alla alternativ och starttiden
menu-load = LADDA
    .facts =
        Uppdrag som sparats under flygning (meny: SPARA UPPDRAG)
        Flyg vidare från där de sparades
menu-fly = FLYG
    .facts = Starta på det briefade uppdraget
menu-fly-vr = FLYG I VR
    .facts =
        Startar om spelet i VR-headsetet
        Flyger exakt detta uppdrag
    .no-mission = Generera ett uppdrag först
    .no-runtime = Ingen OpenXR-runtime (Monado, WiVRn, SteamVR) är aktiv
menu-vr-failed = kunde inte starta i VR: { $error }

## Uppdragssammanfattning

summary-theater = OPERATIONSOMRÅDE
summary-situation = LÄGE
summary-mission = UPPDRAG
summary-opponents = MOTSTÅNDARE
summary-time = TID
summary-weather = VÄDER
summary-fuel = BRÄNSLE
summary-night = Natt
summary-day = Dag
summary-time-value = { $daytime }  avgång { $depart }
summary-weather-value = { $sky }  sikt { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Beväpning: { $stores }
summary-intelligence = UNDERRÄTTELSER
