# Hlavná ponuka a obrazovka jednotlivej misie (src/menu.rs).

## Operačné priestory (stealth_host::theater::THEATERS)

theater-libya = Líbya (1986)
    .facts =
        Sirtský záliv a líbyjské pobrežie
        Čas operácie El Dorado Canyon
        Väčšinu dní jasná púštna obloha
theater-persian_gulf = Perzský záliv (1984)
    .facts =
        Čas tankerovej vojny medzi Iránom a Irakom
        Nad morom a pobrežnou púšťou
        Najjasnejšie počasie zo všetkých operačných priestorov
theater-north_cape = Nordkapp
    .facts =
        Arktické Nórsko a polostrov Kola
        Domov sovietskej Severnej flotily
        Často nízka oblačnosť
theater-central_europe = Stredná Európa
    .facts =
        NATO proti Varšavskej zmluve nad Nemeckom
        Jasná obloha iba v 15 % prípadov
theater-vietnam = Vietnam
    .facts =
        Oblasť Hanoja a Haiphongu
        Džungľa a horský terén
        Búrky v 30 % misií
theater-middle_east = Blízky východ
    .facts =
        Východné Stredozemie a Levanta
        Väčšinou jasná obloha
theater-desert_storm = Púštna búrka
    .facts =
        Kuvajt a Irak, 1991
        Skutočný bojový debut F-117A
        Väčšinou jasná púštna obloha
theater-cuba = Kuba
    .facts =
        Operačný priestor na karibskom ostrove
        Morské a pozemné ciele
        Tropický: búrky v 15 % misií
theater-korea = Kórea
    .facts =
        Kórejský polostrov severne od DMZ
        Hornatý terén
        Premenlivé počasie, možná je akákoľvek obloha
theater-ukraine = Ukrajina (2026)
    .facts =
        Východná Ukrajina, Krym a juh Ruska
        Frontová línia od Charkova po Cherson horí
        Otvorená poľnohospodárska krajina, zničené mestá pri fronte

## Situácie (pravidlá nasadenia)

roe-0 = Studená vojna
    .facts =
        Skrytá operácia v čase mieru
        Najdôležitejšie je zostať nezistený
        Menej nepriateľských lietadiel na hliadke
        Presné pravidlá nasadenia sú v briefingu
roe-1 = Obmedzená vojna
    .facts =
        Regionálny ozbrojený konflikt
        Nepriateľská protivzdušná obrana v pohotovosti
        Viac nepriateľských lietadiel ako v studenej vojne
roe-2 = Konvenčná vojna
    .facts =
        Vojna v plnom rozsahu
        Všetky nepriateľské sily sú aktívne
        Viac nepriateľských lietadiel ako v studenej vojne

## Druhy misií

kind-strike = Úder
    .facts =
        Zničiť, vyfotografovať alebo dosiahnuť pozemný cieľ
        Primárny a sekundárny cieľ
        Výzbroj vzduch-zem
kind-air = Vzduch-vzduch
    .facts =
        Zachytiť nepriateľské lietadlo
        Transportéry, bombardéry alebo stíhacie hliadky
        Výzbroj s raketami vzduch-vzduch
kind-strike-training = Výcvik úderov
    .facts =
        Cvičná úderná misia
        Naučte sa zbrane a navigáciu
kind-air-training = Vzdušný výcvik
    .facts =
        Cvičná misia vzduch-vzduch
        Naučte sa zachytávanie a rakety
kind-drone-defense = Obrana proti dronom
    .facts =
        Iba Ukrajina
        Zastavte vlnu útočných dronov Geran skôr, než zasiahne mesto alebo jadrovú elektráreň
        Potom zbombardujte ich štartovacie miesto

## Protivníci

opponents-0 = Nováčikovia
    .facts =
        Neskúsené nepriateľské posádky
        Najľahšie nastavenie
opponents-1 = Štandardní
    .facts =
        Priemerné nepriateľské posádky
        Predvolené nastavenie
opponents-2 = Veteráni
    .facts =
        Skúsené nepriateľské posádky
        Viac lodí a hliadok
opponents-3 = Elita
    .facts =
        Najlepší nepriateľskí piloti a obsluhy SAM
        Najťažšie nastavenie

## Počasie

sky-random = Náhodné
    .facts =
        Vyžrebované podľa podnebia operačného priestoru
        Mení sa s každou novou misiou
        Oblaky bránia priamej viditeľnosti pre obe strany
sky-clear = Jasno
    .facts =
        Bez oblakov
        Viditeľnosť 40 km
        Klasická obloha z čias DOSu
sky-scattered = Malá oblačnosť
    .facts =
        Kupovité oblaky 20-45 %, základňa 2500-6000 ft
        Občas tenká vysoká vrstva altostratov
        Viditeľnosť 30 km
sky-broken = Polojasno
    .facts =
        Kupovité oblaky 45-70 %, základňa 2000-5000 ft
        Často vrstva stratocumulov nad nimi
        Viditeľnosť 25 km
sky-overcast = Zamračené
    .facts =
        Stratusy 80-100 %, základňa 1500-5000 ft
        Pod nimi niekoľko kúskov kupovitých oblakov
        Viditeľnosť 20 km
sky-storm = Búrka
    .facts =
        Veže cumulonimbov do 40000 ft
        Nízke stratusy a kupovité oblaky od 1000 ft
        Viditeľnosť 8 km
menu-random-cover = Náhodná oblačnosť
    .facts =
        Pokrytie hlavnej vrstvy oblakov sa vyžrebuje spolu s počasím
        Zrušte začiarknutie a nastavte si ho sami
        Hlavná vrstva: stratusy pri zamračenej oblohe a búrke, inak kupovité oblaky
menu-cover-slider =
    .facts =
        Podiel oblohy, ktorý hlavná vrstva pokrýva
        Pri jasnej oblohe bez účinku

## Mesiac

moon-date = Dátum misie
    .facts =
        Skutočný mesiac v deň briefingu
        Vyžaduje prepínač dennej doby (moderný)
moon-0 = Nov
    .facts =
        Bez mesačného svetla: čierna noc, iba svetlá a hviezdy
        Nepriateľskí piloti vás vidia na 2 km
moon-1 = Dorastajúci kosáčik
    .facts = Tenký mesiac na večernej oblohe, zapadá pred polnocou
moon-2 = Prvá štvrť
    .facts = Z polovice osvetlený, večer na oblohe, zapadá okolo polnoci
moon-3 = Dorastajúci mesiac
    .facts = Jasný, na oblohe od popoludnia až po polnoc
moon-4 = Spln
    .facts =
        Na oblohe celú noc, o polnoci vysoko
        Nepriateľskí piloti vás vidia na 4 km
moon-5 = Ubúdajúci mesiac
    .facts = Jasný, vychádza večer, na oblohe až do rána
moon-6 = Posledná štvrť
    .facts = Z polovice osvetlený, vychádza okolo polnoci
moon-7 = Ubúdajúci kosáčik
    .facts = Tenký mesiac vychádzajúci pred svitaním

## Lietadlá

aircraft-f117 =
    .facts =
        Neviditeľné útočné lietadlo
        Podzvukové, 7 g
        Moderný a klasický kokpit
aircraft-f16 =
    .facts =
        Viacúčelová stíhačka
        Trieda Mach 2, 9 g, rýchle klonenie
        Kokpit F-16 s RWR
aircraft-tornado =
    .facts =
        Útočné lietadlo pre lety v malej výške
        Premenná geometria krídel sa mení s rýchlosťou
        Moderný kokpit, tri MFD
aircraft-a10 =
    .facts =
        Lietadlo na priamu leteckú podporu
        Pomalé a odolné, kanón 30 mm
        Kokpit A-10C, dve MFD
aircraft-uh60 =
    .facts =
        Univerzálny vrtuľník
        Vysadzuje vojakov: pristaňte pri cieli
        Cyklické riadenie a kolektív
aircraft-ah1z =
    .facts =
        Bojový vrtuľník
        Kanón vo veži, Hellfire, AIM-9
        Kanón a Hellfire sledujú gondolu

## Sekcie a skupiny

section-theater = Operačný priestor
section-opponents = Protivníci
section-situation = Situácia
section-mission = Misia
section-moon = Mesiac
section-start-time = Čas štartu
section-weather = Počasie
section-realism = Realizmus
section-tacview = Tacview
section-mission-summary = Súhrn misie
section-briefing = Briefing
group-mission = Misia
group-conditions = Podmienky
group-vehicle = Stroj
group-simulation = Simulácia

## Možnosti misie

menu-start-random = Náhodne
    .facts =
        Vyžrebuje sa pri každej novej misii ako v klasickom režime
        06:00-08:00 cez deň alebo 18:00-20:00 v noci
        Zrušte začiarknutie a zvoľte hodinu
menu-start-slider =
    .facts =
        Čas vzletu
        Noc od 18:00 do 06:00
        Zachová sa pri NOVEJ MISII
menu-realistic-weapons = Realistické zbrane
    .facts =
        Obmedzenia skutočného lietadla
        2 zbraňové priestory namiesto 4
        Žiadne rakety AIM-9 ani AIM-120
        Žiadne vzlety a pristátia na lodi
menu-crash-protection = Ochrana pred nárazom
    .facts =
        Pri zemi automaticky vyberie (podvozok zasunutý)
        Dotyk so zemou a zrážky nie sú smrteľné
        Vyp: do zeme sa dá vždy naraziť
menu-tacview-export = Export
    .facts =
        Uloží každý let ako záznam Tacview (.acmi)
        V priečinku hry pre vaše súbory, pod tacview
menu-tacview-real-time = Telemetria v reálnom čase
    .facts =
        Posiela let do Tacview počas pilotovania
        V Tacview: Record > Real-Time Telemetry
        Adresa 127.0.0.1, port 42674 (--tacview-address)
switch-game = HRA
switch-real = REÁL
menu-damage = Poškodenie
    .facts =
        Poškodenie zbraňami
        Hra: klasické poškodenie; zásahy sa sčítavajú, ale lietadlo nikdy nezostrelia
        Realistické: zásah raketou môže lietadlo zničiť (katapultujte sa!), poškodí viac systémov a často spôsobí požiar, ktorý dohorí alebo lietadlo roztrhne
        Realistické: poškodené kormidlá vychýlia lietadlo z trimu alebo ho rozkmitajú
menu-guidance = Navádzanie
    .facts =
        Navádzanie rakiet
        Klasické: rakety letia priamo na cieľ
        Realistické: proporcionálna navigácia do bodu zrážky; motory dohoria a rakety sa kĺžu ďalej, s každou zákrutou strácajú rýchlosť (dym iba kým motor horí)
        Realistické: rakety SAM stúpajú oblúkom na vzdialené ciele; AIM-120 letí 3 s rovno (potiahnutím vyšplhajte pre oblúk)
menu-air-defence = Protivzdušná obrana
    .facts =
        Protivzdušná obrana v malej výške
        Klasické: iba pozície SAM a stíhačky
        Realistické: radarom naviedané kanóny ZSU-23-4 (ich radar vás vidí ako radar SAM) a batérie flaku S-60 strážia letiská a ciele; skupiny MANPADS v mestách strieľajú SA-14 na to, čo vidia
        Let v malej výške nad bránenými miestami je nebezpečný; bomby a kanóny ničia kanóny

## Hlavná obrazovka

menu-title = STEALTH MISSION 89
menu-subtitle = SYSTÉM PLÁNOVANIA MISIÍ
menu-single-mission = JEDNA MISIA
    .facts =
        Jedna misia v operačnom priestore podľa vášho výberu
        Zvoľte možnosti, naplánujte trasu a výzbroj
menu-career = KARIÉRA
    .facts =
        Záznamy pilota: hodnosť, vyznamenania, bojové lety
        Doba služby: 12 bojových letov v jednom operačnom priestore
        Čo zničíte, zostane zničené
menu-campaign = DYNAMICKÁ KAMPAŇ
    .facts =
        Dynamická kampaň v reálnom čase
        Dobite nepriateľské mestá a letiská
        Misie podľa aktuálnej situácie
menu-multiplayer = MULTIPLAYER
    .facts = Coop v dynamickej kampani
    .join = Pripojiť sa k serveru z verejného zoznamu
    .host = Pripojiť sa k serveru alebo založiť kampaň a lietať v nej
menu-options = MOŽNOSTI
    .facts =
        Hra: štýl kokpitu, HUD a avioniky, automatický trim
        Grafika, hlasitosti, priradenie klávesov a joysticku
        Uložené pre ďalšie lety
    .short = Hra, grafika, zvuk a ovládanie
menu-download = STIAHNUŤ
    .facts =
        Desktopová hra pre Windows, Linux a macOS
        S VR a multiplayerom
        Steam Deck a Steam Frame: AppImage pre Linux
web-play = HRAŤ V PREHLIADAČI
    .facts = Priamo tu, nič sa neinštaluje
web-download-platforms = Windows, Linux a macOS, Steam Deck a Steam Frame
web-download-version = Verzia na stiahnutie
web-gives-vr = VR
    .why = Lietajte vo VR na akomkoľvek headsete kompatibilnom s OpenXR a na Steam Frame
web-gives-hotas = HOTAS
    .why = Plná podpora gamepadov, joystickov, plynových pák a pedálov
web-gives-multiplayer = Lepší multiplayer
    .why = Priame spojenie UDP so servermi pre menšie oneskorenie
web-gives-server = Vlastný server
    .why = Založte coop kampaň alebo spustite dedikovaný server
web-gives-head-tracking = Sledovanie hlavy
    .why = Trackery typu TrackIR cez OpenTrack
web-gives-tacview = Tacview
    .why = Záznamy letov a telemetria v reálnom čase
web-gives-keep = Navždy vaše
    .why = Ak web zanikne, stále máte vlastnú kópiu
menu-discord = DISCORD
    .facts =
        Lietajte spolu, hláste chyby a zdieľajte misie
        Sledujte vývoj
menu-quit = UKONČIŤ
    .facts = Odísť na plochu
menu-version = VERZIA { $version }

## Obrazovka jednej misie

menu-career-sortie = BOJOVÝ LET KARIÉRY
menu-tour-sortie = { $pilot }  //  BOJOVÝ LET { $sortie } Z { $length }
menu-msn = SYSTÉM PLÁNOVANIA MISIÍ  //  MSN { $seed }
menu-no-targets = Pre túto dobu služby nezostali žiadne ciele
menu-plan-route = PLÁNOVAŤ TRASU
    .facts =
        Mapa operačného priestoru s nepriateľským radarom a dosahom SAM
        Pridávanie, presúvanie a odstraňovanie navigačných bodov
        Ciele a pristávacia základňa zostávajú pevné
menu-loadout = VÝZBROJ
    .facts =
        Zvoľte náklad na každom zbraňovom priestore alebo pylóne
        Začína sa so zbraňami z briefingu
menu-generating = GENERUJEM...
menu-back-records = Späť na záznamy pilotov
menu-back-main = Späť do hlavnej ponuky
menu-new-sortie = NOVÝ BOJOVÝ LET
menu-new-mission = NOVÁ MISIA
    .facts =
        Vyžrebuje iné ciele
        Zachová všetky možnosti a čas štartu
menu-load = NAČÍTAŤ
    .facts =
        Misie uložené za letu (ponuka: ULOŽIŤ MISIU)
        Pokračujte odtiaľ, kde boli uložené
menu-fly = LIETAŤ
    .facts = Vzlietnuť na misiu z briefingu
menu-fly-vr = LIETAŤ VO VR
    .facts =
        Reštartuje hru v headsete VR
        Poletíte presne túto misiu
    .no-mission = Najprv vygenerujte misiu
    .no-runtime = Nie je aktívny žiadny runtime OpenXR (Monado, WiVRn, SteamVR)
menu-vr-failed = nepodarilo sa spustiť vo VR: { $error }

## Súhrn misie

summary-theater = OPERAČNÝ PRIESTOR
summary-situation = SITUÁCIA
summary-mission = MISIA
summary-opponents = PROTIVNÍCI
summary-time = ČAS
summary-weather = POČASIE
summary-fuel = PALIVO
summary-night = Noc
summary-day = Deň
summary-time-value = { $daytime }  odlet { $depart }
summary-weather-value = { $sky }  viditeľnosť { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Výzbroj: { $stores }
summary-intelligence = ROZPOZNÁVANIE
