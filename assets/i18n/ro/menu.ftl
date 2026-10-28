# Meniul principal și ecranul misiunii simple (src/menu.rs).

## Teatre de operații (stealth_host::theater::THEATERS)

theater-libya = Libia (1986)
    .facts =
        Golful Sirtei și coasta libiană
        Epoca Operațiunii El Dorado Canyon
        Cer senin de deșert în majoritatea zilelor
theater-persian_gulf = Golful Persic (1984)
    .facts =
        Epoca Războiului Petrolierelor dintre Iran și Irak
        Deasupra apei și a deșertului de coastă
        Cel mai senin timp dintre toate teatrele
theater-north_cape = Capul Nord
    .facts =
        Norvegia arctică și Peninsula Kola
        Baza Flotei Nordului sovietice
        Nori joși frecvenți
theater-central_europe = Europa Centrală
    .facts =
        NATO contra Pactului de la Varșovia deasupra Germaniei
        Cer senin doar 15% din timp
theater-vietnam = Vietnam
    .facts =
        Regiunea Hanoi și Haiphong
        Junglă și relief muntos
        Furtuni în 30% din misiuni
theater-middle_east = Orientul Mijlociu
    .facts =
        Estul Mediteranei și Levantul
        Cer senin în cea mai mare parte
theater-desert_storm = Desert Storm
    .facts =
        Kuweit și Irak, 1991
        Adevăratul debut de luptă al lui F-117A
        Cer de deșert senin în cea mai mare parte
theater-cuba = Cuba
    .facts =
        Teatru insular în Caraibe
        Ținte maritime și terestre
        Tropical: furtuni în 15% din misiuni
theater-korea = Coreea
    .facts =
        Peninsula Coreeană la nord de DMZ
        Relief muntos
        Vreme schimbătoare, orice cer este posibil
theater-ukraine = Ucraina (2026)
    .facts =
        Estul Ucrainei, Crimeea și sudul Rusiei
        Linia frontului arde de la Harkov la Herson
        Câmpuri deschise, orașe în ruine pe front

## Situații (reguli de angajare)

roe-0 = Războiul Rece
    .facts =
        Operațiune secretă în timp de pace
        Contează mai ales să rămâneți nedetectat
        Mai puține aeronave inamice în patrulare
        Regulile de angajare exacte sunt în briefing
roe-1 = Război limitat
    .facts =
        Conflict regional cu schimb de focuri
        Apărarea antiaeriană inamică este în alertă
        Mai multe aeronave inamice decât în Războiul Rece
roe-2 = Război convențional
    .facts =
        Război la scară largă
        Toate forțele inamice sunt active
        Mai multe aeronave inamice decât în Războiul Rece

## Tipuri de misiune

kind-strike = Lovitură
    .facts =
        Distrugeți, fotografiați sau atingeți o țintă terestră
        Obiectiv principal și secundar
        Armament aer-sol
kind-air = Aer-aer
    .facts =
        Interceptați o aeronavă inamică
        Avioane de transport, bombardiere sau patrule de vânătoare
        Armament cu rachete aer-aer
kind-strike-training = Antrenament de lovitură
    .facts =
        Misiune de lovitură de antrenament
        Învățați armele și navigația
kind-air-training = Antrenament aer-aer
    .facts =
        Misiune aer-aer de antrenament
        Învățați interceptările și rachetele
kind-drone-defense = Apărare antidronă
    .facts =
        Doar în Ucraina
        Opriți un val de drone de atac Geran înainte să lovească un oraș sau o centrală nucleară
        Apoi bombardați baza lor de lansare

## Adversari

opponents-0 = Novici
    .facts =
        Echipaje inamice fără experiență
        Cea mai ușoară setare
opponents-1 = Obișnuiți
    .facts =
        Echipaje inamice medii
        Setarea implicită
opponents-2 = Veterani
    .facts =
        Echipaje inamice pricepute
        Mai multe nave și patrule
opponents-3 = Elită
    .facts =
        Cei mai buni piloți inamici și echipaje SAM
        Cea mai grea setare

## Vremea

sky-random = Aleatoriu
    .facts =
        Stabilită după clima teatrului de operații
        Se schimbă la fiecare misiune nouă
        Norii blochează linia de vedere pentru ambele părți
sky-clear = Senin
    .facts =
        Fără nori
        Vizibilitate 40 km
        Cerul clasic din epoca DOS
sky-scattered = Nori răzleți
    .facts =
        Cumulus 20-45%, baza la 2500-6000 ft
        Uneori un altostratus subțire sus
        Vizibilitate 30 km
sky-broken = Nori fragmentați
    .facts =
        Cumulus 45-70%, baza la 2000-5000 ft
        Adesea un strat de stratocumulus deasupra
        Vizibilitate 25 km
sky-overcast = Înnorat complet
    .facts =
        Stratus 80-100%, baza la 1500-5000 ft
        Câteva fragmente de cumulus dedesubt
        Vizibilitate 20 km
sky-storm = Furtună
    .facts =
        Turnuri de cumulonimbus până la 40000 ft
        Stratus jos și cumulus de la 1000 ft
        Vizibilitate 8 km
menu-random-cover = Acoperire aleatorie
    .facts =
        Acoperirea stratului principal de nori este stabilită odată cu vremea
        Debifați pentru a o alege singur
        Stratul principal: stratus pentru cer înnorat și furtună, altfel cumulus
menu-cover-slider =
    .facts =
        Partea de cer acoperită de stratul principal
        Fără efect pe cer senin

## Luna

moon-date = Data misiunii
    .facts =
        Luna reală din data briefing-ului
        Necesită comutatorul ora zilei (modern)
moon-0 = Lună nouă
    .facts =
        Fără lumina lunii: noapte neagră, doar lumini și stele
        Piloții inamici vă văd de la 2 km
moon-1 = Semilună crescătoare
    .facts = O lună subțire pe cerul serii, apune înainte de miezul nopții
moon-2 = Primul pătrar
    .facts = Pe jumătate luminată, sus seara, apune în jurul miezului nopții
moon-3 = Lună giboasă crescătoare
    .facts = Strălucitoare, sus din după-amiază până după miezul nopții
moon-4 = Lună plină
    .facts =
        Sus toată noaptea, la zenit la miezul nopții
        Piloții inamici vă văd de la 4 km
moon-5 = Lună giboasă descrescătoare
    .facts = Strălucitoare, răsare seara, sus până dimineața
moon-6 = Ultimul pătrar
    .facts = Pe jumătate luminată, răsare în jurul miezului nopții
moon-7 = Semilună descrescătoare
    .facts = O lună subțire care răsare înainte de zori

## Aeronave

aircraft-f117 =
    .facts =
        Aeronavă de atac stealth
        Subsonică, 7 g
        Cockpit modern și clasic
aircraft-f16 =
    .facts =
        Avion de vânătoare multirol
        Clasa Mach 2, 9 g, ruliu rapid
        Cockpit F-16 cu RWR
aircraft-tornado =
    .facts =
        Aeronavă de lovitură la joasă înălțime
        Aripile cu geometrie variabilă se deschid odată cu viteza
        Cockpit modern, trei MFD-uri
aircraft-a10 =
    .facts =
        Aeronavă de sprijin aerian apropiat
        Lentă și robustă, tun de 30 mm
        Cockpit A-10C, două MFD-uri
aircraft-uh60 =
    .facts =
        Elicopter utilitar
        Debarcă trupe: aterizați lângă țintă
        Ciclic și colectiv
aircraft-ah1z =
    .facts =
        Elicopter de atac
        Tun în turelă, Hellfire, AIM-9
        Tunul și Hellfire urmăresc nacela

## Secțiuni și grupuri

section-theater = Teatru de operații
section-opponents = Adversari
section-situation = Situație
section-mission = Misiune
section-moon = Luna
section-start-time = Ora de start
section-weather = Vremea
section-realism = Realism
section-tacview = Tacview
section-mission-summary = Rezumatul misiunii
section-briefing = Briefing
group-mission = Misiune
group-conditions = Condiții
group-vehicle = Aeronavă
group-simulation = Simulare

## Opțiunile misiunii

menu-start-random = Aleatorie
    .facts =
        Stabilită la fiecare misiune nouă, ca în modul clasic
        06:00-08:00 ziua sau 18:00-20:00 noaptea
        Debifați pentru a alege ora
menu-start-slider =
    .facts =
        Ora decolării
        Noapte de la 18:00 la 06:00
        Păstrată de MISIUNE NOUĂ
menu-realistic-weapons = Armament realist
    .facts =
        Limitele aeronavei reale
        2 compartimente de armament în loc de 4
        Fără rachete AIM-9 sau AIM-120
        Fără decolări sau apuntări pe portavion
menu-crash-protection = Protecție la impact
    .facts =
        Redresează automat aproape de sol (tren retras)
        Contactul cu solul și coliziunile nu sunt fatale
        Oprită: solul poate fi mereu lovit
menu-tacview-export = Export
    .facts =
        Salvează fiecare zbor ca înregistrare Tacview (.acmi)
        În dosarul jocului pentru fișierele dumneavoastră, sub tacview
menu-tacview-real-time = Telemetrie în timp real
    .facts =
        Trimite zborul către Tacview cât timp zburați
        În Tacview: Record > Real-Time Telemetry
        Adresa 127.0.0.1, portul 42674 (--tacview-address)
switch-game = JOC
switch-real = REAL
menu-damage = Avarii
    .facts =
        Avariile provocate de arme
        Joc: avarii clasice; loviturile se adună, dar nu doboară niciodată aeronava
        Realist: o lovitură de rachetă poate distruge aeronava (catapultați!), avariază mai multe sisteme și deseori provoacă un incendiu, care se stinge sau face aeronava să explodeze
        Realist: suprafețele de comandă avariate dereglează trimul sau fac aeronava să oscileze
menu-guidance = Ghidare
    .facts =
        Ghidarea rachetelor
        Clasic: rachetele zboară drept spre țintă
        Realist: navigație proporțională spre punctul de coliziune; motoarele se sting, iar rachetele planează mai departe, pierzând viteză la fiecare viraj (fum doar cât arde motorul)
        Realist: SAM-urile urcă în arc împotriva țintelor îndepărtate; AIM-120 zboară drept 3 s (trageți de manșă pentru a o lansa în arc)
menu-air-defence = Apărare antiaeriană
    .facts =
        Apărarea antiaeriană la joasă înălțime
        Clasic: doar siturile SAM și avioanele de vânătoare
        Realist: tunuri ZSU-23-4 conduse prin radar (radarul lor vă vede ca pe cel al unui SAM) și baterii de artilerie antiaeriană S-60 păzesc aerodromurile și obiectivele; echipe MANPADS din orașe trag cu SA-14 în ce văd
        Zborul la joasă înălțime deasupra locurilor apărate devine periculos; bombele și tunul distrug piesele

## Ecranul principal

menu-title = STEALTH MISSION 89
menu-subtitle = SISTEM DE PLANIFICARE A MISIUNII
menu-single-mission = MISIUNE SIMPLĂ
    .facts =
        O misiune în teatrul de operații ales
        Alegeți opțiunile, planificați ruta și armamentul
menu-career = CARIERĂ
    .facts =
        Dosarele piloților: grad, decorații, zboruri
        Perioadă de serviciu: 12 zboruri într-un teatru de operații
        Ce distrugeți rămâne distrus
menu-campaign = CAMPANIE DINAMICĂ
    .facts =
        Campanie dinamică în timp real
        Cuceriți orașele și aerodromurile inamicului
        Misiuni generate din situația curentă
menu-multiplayer = MULTIPLAYER
    .facts = Cooperare în campania dinamică
    .join = Alăturați-vă unui server din lista publică
    .host = Alăturați-vă unui server sau găzduiți o campanie și zburați în ea
menu-options = OPȚIUNI
    .facts =
        Joc: stilul cockpitului, al HUD-ului și al avionicii, trim automat
        Grafică, volume sonore, taste și joystick
        Salvate pentru zborurile următoare
    .short = Joc, grafică, sunet și comenzi
menu-download = DESCARCĂ
    .facts =
        Jocul pentru desktop, pentru Windows, Linux și macOS
        Cu VR și multiplayer
        Steam Deck și Steam Frame: AppImage pentru Linux
web-play = JOACĂ ÎN BROWSER
    .facts = Chiar aici, nimic de instalat
web-download-platforms = Windows, Linux și macOS, Steam Deck și Steam Frame
web-download-version = Versiunea de descărcat
web-gives-vr = VR
    .why = Zburați în VR cu orice cască compatibilă OpenXR și cu Steam Frame
web-gives-hotas = HOTAS
    .why = Suport complet pentru gamepaduri, joystickuri, manete de gaze și pedale
web-gives-multiplayer = Multiplayer mai bun
    .why = Conexiune UDP directă la servere, pentru mai puțină latență
web-gives-server = Găzduiți propriul server
    .why = Găzduiți o campanie cooperativă sau rulați un server dedicat
web-gives-head-tracking = Urmărirea capului
    .why = Trackere de tip TrackIR prin OpenTrack
web-gives-tacview = Tacview
    .why = Înregistrări de zbor și telemetrie în timp real
web-gives-keep = Rămâne al dumneavoastră
    .why = Dacă site-ul dispare, aveți în continuare propria copie
menu-discord = DISCORD
    .facts =
        Zburați împreună, raportați erori și împărtășiți misiuni
        Urmăriți dezvoltarea
menu-quit = IEȘIRE
    .facts = Ieșire pe desktop
menu-version = VERSIUNEA { $version }

## Ecranul misiunii simple

menu-career-sortie = ZBOR DE CARIERĂ
menu-tour-sortie = { $pilot }  //  ZBORUL { $sortie } DIN { $length }
menu-msn = SISTEM DE PLANIFICARE A MISIUNII  //  MSN { $seed }
menu-no-targets = Nu a mai rămas nicio țintă în picioare pentru această perioadă de serviciu
menu-plan-route = PLANIFICĂ RUTA
    .facts =
        Harta teatrului de operații cu acoperirea radarelor și a SAM-urilor inamice
        Adăugați, mutați și ștergeți puncte de navigație
        Țintele și baza de aterizare rămân fixe
menu-loadout = ARMAMENT
    .facts =
        Alegeți încărcăturile de pe fiecare compartiment sau pilon
        Pornește cu armele din briefing
menu-generating = SE GENEREAZĂ...
menu-back-records = Înapoi la dosarele piloților
menu-back-main = Înapoi la meniul principal
menu-new-sortie = ZBOR NOU
menu-new-mission = MISIUNE NOUĂ
    .facts =
        Generează alte obiective
        Păstrează toate opțiunile și ora de start
menu-load = ÎNCARCĂ
    .facts =
        Misiuni salvate în zbor (meniu: SALVEAZĂ MISIUNEA)
        Continuați zborul de unde au fost salvate
menu-fly = ZBOARĂ
    .facts = Decolați în misiunea din briefing
menu-fly-vr = ZBOARĂ ÎN VR
    .facts =
        Repornește jocul în casca VR
        Zboară exact această misiune
    .no-mission = Generați mai întâi o misiune
    .no-runtime = Niciun runtime OpenXR (Monado, WiVRn, SteamVR) nu este activ
menu-vr-failed = pornirea în VR a eșuat: { $error }

## Rezumatul misiunii

summary-theater = TEATRU DE OPERAȚII
summary-situation = SITUAȚIE
summary-mission = MISIUNE
summary-opponents = ADVERSARI
summary-time = ORA
summary-weather = VREMEA
summary-fuel = COMBUSTIBIL
summary-night = Noapte
summary-day = Zi
summary-time-value = { $daytime }  decolare { $depart }
summary-weather-value = { $sky }  vizib. { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Armament: { $stores }
summary-intelligence = INFORMAȚII
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
