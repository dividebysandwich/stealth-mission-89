# Il-menu prinċipali u l-iskrin tal-missjoni waħda (src/menu.rs).

## Teatri (stealth_host::theater::THEATERS)

theater-libya = Libja (1986)
    .facts =
        Il-Golf tas-Sidra u x-xatt Libjan
        L-era tal-Operation El Dorado Canyon
        Smewwiet safja tad-deżert il-biċċa l-kbira tal-jiem
theater-persian_gulf = Golf Persjan (1984)
    .facts =
        L-era tal-Gwerra tat-Tankers bejn l-Iran u l-Iraq
        Fuq il-baħar u d-deżert kostali
        L-aktar temp ċar mit-teatri kollha
theater-north_cape = North Cape
    .facts =
        In-Norveġja Artika u l-Peniżola ta' Kola
        Id-dar tal-Flotta tat-Tramuntana Sovjetika
        Sħab baxx spiss
theater-central_europe = Ewropa Ċentrali
    .facts =
        In-NATO vs il-Patt ta' Varsavja fuq il-Ġermanja
        Sema safi biss 15% tal-ħin
theater-vietnam = Vjetnam
    .facts =
        Ir-reġjun ta' Hanoi u Haiphong
        Art ta' ġungla u muntanji
        Maltempati f'30% tal-missjonijiet
theater-middle_east = Lvant Nofsani
    .facts =
        Il-Mediterran tal-Lvant u l-Levant
        Is-sema l-biċċa l-kbira ċar
theater-desert_storm = Desert Storm
    .facts =
        Il-Kuwajt u l-Iraq, 1991
        L-ewwel dehra tal-F-117A fil-ġlied veru
        Smewwiet tad-deżert il-biċċa l-kbira ċari
theater-cuba = Kuba
    .facts =
        Teatru fuq gżira tal-Karibew
        Miri fuq il-baħar u fuq l-art
        Tropikali: maltempati f'15% tal-missjonijiet
theater-korea = Korea
    .facts =
        Il-peniżola Koreana fit-tramuntana tad-DMZ
        Art muntanjuża
        Temp li jinbidel, kull sema huwa possibbli
theater-ukraine = Ukrajna (2026)
    .facts =
        L-Ukrajna tal-Lvant, il-Krimea u n-nofsinhar tar-Russja
        Il-linja tal-quddiem minn Kharkiv sa Kherson qed taħraq
        Għelieqi miftuħa, bliet imħarbta fuq il-front

## Sitwazzjonijiet (regoli tal-ingaġġ)

roe-0 = Gwerra Bierda
    .facts =
        Operazzjoni moħbija fi żmien il-paċi
        Li ma tinqabadx huwa l-aktar importanti
        Inqas ajruplani tal-għadu fuq il-patrol
        Ir-regoli eżatti tal-ingaġġ jinsabu fil-briefing
roe-1 = Gwerra Limitata
    .facts =
        Kunflitt reġjonali bi sparar
        Id-difiżi tal-ajru tal-għadu f'alert
        Aktar ajruplani tal-għadu milli fil-Gwerra Bierda
roe-2 = Gwerra Konvenzjonali
    .facts =
        Gwerra fuq skala sħiħa
        Il-forzi kollha tal-għadu attivi
        Aktar ajruplani tal-għadu milli fil-Gwerra Bierda

## Tipi ta' missjoni

kind-strike = Strike
    .facts =
        Qered, ritratta jew wasal fil-mira fuq l-art
        Għan primarju u sekondarju
        Armamenti ajru-art
kind-air = Ajru-ajru
    .facts =
        Intercept ta' ajruplan tal-għadu
        Trasport, bombers jew patrols tal-ġellieda
        Armamenti ta' missili ajru-ajru
kind-strike-training = Taħriġ ta' strike
    .facts =
        Missjoni ta' strike għall-prattika
        Tgħallem l-armi u n-navigazzjoni
kind-air-training = Taħriġ ajru-ajru
    .facts =
        Missjoni ajru-ajru għall-prattika
        Tgħallem l-intercepts u l-missili
kind-drone-defense = Difiża mid-drones
    .facts =
        Ukrajna biss
        Waqqaf mewġa ta' drones tal-attakk Geran qabel ma jolqtu belt jew impjant nukleari
        Imbagħad ibbombarda s-sit tat-tnedija tagħhom

## Avversarji

opponents-0 = Novizzi
    .facts =
        Ekwipaġġi tal-għadu bla esperjenza
        L-aktar issettjar faċli
opponents-1 = Regolari
    .facts =
        Ekwipaġġi medji tal-għadu
        Issettjar predefinit
opponents-2 = Veterani
    .facts =
        Ekwipaġġi tal-għadu b'ħila
        Aktar vapuri u patrols
opponents-3 = Elite
    .facts =
        L-aqwa piloti u ekwipaġġi SAM tal-għadu
        L-aktar issettjar diffiċli

## Temp

sky-random = Każwali
    .facts =
        Magħżul mill-klima tat-teatru
        Jinbidel ma' kull missjoni ġdida
        Is-sħab jimblokka l-linja tal-vista taż-żewġ naħat
sky-clear = Ċar
    .facts =
        L-ebda sħab
        Viżibilità ta' 40 km
        Is-sema klassiku tal-era tad-DOS
sky-scattered = Mxerred
    .facts =
        Cumulus 20-45%, bażi 2500-6000 ft
        Kultant altostratus irqiq 'il fuq
        Viżibilità ta' 30 km
sky-broken = Maqsum
    .facts =
        Cumulus 45-70%, bażi 2000-5000 ft
        Spiss saff ta' stratocumulus fuq
        Viżibilità ta' 25 km
sky-overcast = Mgħobbi
    .facts =
        Stratus 80-100%, bażi 1500-5000 ft
        Ftit biċċiet ta' cumulus taħt
        Viżibilità ta' 20 km
sky-storm = Maltempata
    .facts =
        Torrijiet ta' cumulonimbus sa 40000 ft
        Stratus u cumulus baxx minn 1000 ft
        Viżibilità ta' 8 km
menu-random-cover = Kopertura każwali
    .facts =
        Il-kopertura tas-saff ewlieni tas-sħab magħżula mat-temp
        Neħħi s-sinjal biex taġġustaha int
        Saff ewlieni: stratus għal sema mgħobbi u maltempata, cumulus mod ieħor
menu-cover-slider =
    .facts =
        Sehem tas-sema li s-saff ewlieni jkopri
        Ebda effett fuq sema ċar

## Qamar

moon-date = Data tal-missjoni
    .facts =
        Il-qamar veru tad-data tal-briefing
        Jeħtieġ l-iswiċċ tal-ħin tal-jum (modern)
moon-0 = Qamar ġdid
    .facts =
        L-ebda dawl tal-qamar: lejl iswed, dwal u stilel biss
        Il-piloti tal-għadu jarawk minn 2 km
moon-1 = Qamar li qed jikber (nieqes)
    .facts = Qamar irqiq fis-sema tal-għaxija, jinżel qabel nofs il-lejl
moon-2 = L-ewwel kwart
    .facts = Nofsu imdawwal, għoli filgħaxija, jinżel xi ħabta ta' nofs il-lejl
moon-3 = Qamar li qed jikber (gibbous)
    .facts = Qawwi, għoli mill-wara nofsinhar sa wara nofs il-lejl
moon-4 = Qamar mimli
    .facts =
        Għoli l-lejl kollu, fil-quċċata f'nofs il-lejl
        Il-piloti tal-għadu jarawk minn 4 km
moon-5 = Qamar li qed jonqos (gibbous)
    .facts = Qawwi, jitla' filgħaxija, għoli sas-sebħ
moon-6 = L-aħħar kwart
    .facts = Nofsu imdawwal, jitla' xi ħabta ta' nofs il-lejl
moon-7 = Qamar li qed jonqos (nieqes)
    .facts = Qamar irqiq li jitla' qabel is-sebħ

## Ajruplani

aircraft-f117 =
    .facts =
        Ajruplan stealth tal-attakk
        Subsoniku, 7 g
        Kokpit modern u klassiku
aircraft-f16 =
    .facts =
        Ġellied multirole
        Klassi Mach 2, 9 g, roll veloċi
        Kokpit F-16 bl-RWR
aircraft-tornado =
    .facts =
        Ajruplan ta' strike f'altitudni baxxa
        Il-ġwienaħ jiċċaqalqu mal-veloċità
        Kokpit modern, tliet MFDs
aircraft-a10 =
    .facts =
        Ajruplan ta' appoġġ ajru qrib
        Bil-mod u robust, kanun ta' 30 mm
        Kokpit A-10C, żewġ MFDs
aircraft-uh60 =
    .facts =
        Ħelikopter utilitarju
        Iniżżel it-truppi: artab qrib il-mira
        Cyclic u collective
aircraft-ah1z =
    .facts =
        Ħelikopter tal-attakk
        Kanun fuq turret, Hellfires, AIM-9
        Il-kanun u l-Hellfires isegwu l-pod

## Sezzjonijiet u gruppi

section-theater = Teatru
section-opponents = Avversarji
section-situation = Sitwazzjoni
section-mission = Missjoni
section-moon = Qamar
section-start-time = Ħin tal-bidu
section-weather = Temp
section-realism = Realiżmu
section-tacview = Tacview
section-mission-summary = Sommarju tal-missjoni
section-briefing = Briefing
group-mission = Missjoni
group-conditions = Kundizzjonijiet
group-vehicle = Vettura
group-simulation = Simulazzjoni

## Opzjonijiet tal-missjoni

menu-start-random = Każwali
    .facts =
        Magħżul ma' kull missjoni ġdida, bħal fil-mod klassiku
        06:00-08:00 bin-nhar jew 18:00-20:00 bil-lejl
        Neħħi s-sinjal biex tagħżel is-siegħa
menu-start-slider =
    .facts =
        Ħin tat-tluq
        Lejl minn 18:00 sa 06:00
        Jinżamm minn NEW MISSION
menu-realistic-weapons = Armi realistiċi
    .facts =
        Il-limiti tal-ajruplan veru
        2 weapon bays minflok 4
        L-ebda missili AIM-9 jew AIM-120
        L-ebda tluq jew inżul fuq carrier
menu-crash-protection = Protezzjoni mill-ħabtiet
    .facts =
        Titla' awtomatikament qrib l-art (gear mgħolli)
        Il-kuntatt mal-art u l-kolliżjonijiet mhumiex fatali
        Off: l-art dejjem tista' tintlaqat
menu-tacview-export = Esporta
    .facts =
        Jaħżen kull titjira bħala reġistrazzjoni Tacview (.acmi)
        Fil-folder tal-logħba għall-fajls tiegħek, taħt tacview
menu-tacview-real-time = Telemetrija fil-ħin reali
    .facts =
        Jibgħat it-titjira lil Tacview waqt li int tittajjar
        F'Tacview: Record > Real-Time Telemetry
        Indirizz 127.0.0.1, port 42674 (--tacview-address)
switch-game = LOGĦBA
switch-real = VERU
menu-damage = Ħsara
    .facts =
        Ħsara tal-armi
        Logħba: ħsara klassika; il-ħbit jinġabar iżda qatt ma jwaqqa' l-ajruplan
        Realistiku: ħabta ta' missila tista' teqred l-ajruplan (eject!), tagħmel ħsara lil aktar sistemi u spiss tqabbad in-nar, li jintefa jew jisplodi l-ajruplan
        Realistiku: superfiċi tal-kontroll bil-ħsara jwaqqgħu l-ajruplan barra mit-trim jew jagħmluh joxxilla
menu-guidance = Gwida
    .facts =
        Gwida tal-missili
        Klassika: missili li jtiru dritt lejn il-mira
        Realistika: navigazzjoni proporzjonali lejn il-punt tal-kolliżjoni; il-muturi jispiċċaw u l-missili jiżżerżqu, jitilfu veloċità ma' kull dawra (duħħan biss waqt li jaħdem il-mutur)
        Realistika: is-SAMs jitilgħu 'l fuq kontra miri bogħod; l-AIM-120 jtir dritt għal 3 s (itla' biex tarmih 'il fuq)
menu-air-defence = Difiża tal-ajru
    .facts =
        Difiżi tal-ajru f'altitudni baxxa
        Klassika: is-siti SAM u l-ġellieda biss
        Realistika: kanuni ZSU-23-4 immexxija bir-radar (ir-radar tagħhom jarak bħal ta' SAM) u batteriji flak S-60 jgħassu l-ajrudromi u l-miri; timijiet MANPADS fil-bliet jisparaw SA-14s fuq dak li jaraw
        It-titjir baxx fuq postijiet difiżi jsir perikoluż; bombi u kanuni jeqirdu l-kanuni

## Skrin prinċipali

menu-title = STEALTH MISSION 89
menu-subtitle = SISTEMA TA' IPPJANAR TAL-MISSJONI
menu-single-mission = MISSJONI WAĦDA
    .facts =
        Missjoni waħda fit-teatru li tagħżel
        Agħżel l-opzjonijiet, ippjana r-rotta u l-armamenti
menu-career = KARRIERA
    .facts =
        Rekords tal-pilota: grad, dekorazzjonijiet, titjiriet
        Perjodu ta' servizz: 12-il titjira f'teatru wieħed
        Dak li teqred jibqa' meqrud
menu-campaign = KAMPANJA DINAMIKA
    .facts =
        Kampanja dinamika fil-ħin reali
        Ħu l-bliet u l-ajrudromi tal-għadu
        Missjonijiet mis-sitwazzjoni ħajja
menu-multiplayer = MULTIPLAYER
    .facts = Coop fil-kampanja dinamika
    .join = Ingħaqad ma' server mil-lista pubblika
    .host = Ingħaqad ma' server, jew oħloq kampanja u ttajjar fiha
menu-options = OPZJONIJIET
    .facts =
        Logħba: stil tal-kokpit, tal-HUD u tal-avjonika, auto trim
        Grafika, volumi tal-ħoss, assenjazzjonijiet tat-tasti u tal-joystick
        Maħżuna għat-titjiriet li jmiss
    .short = Logħba, grafika, ħoss u kontrolli
menu-download = DOWNLOAD
    .facts =
        Il-logħba desktop għal Windows, Linux u macOS
        Bil-VR u l-multiplayer
        Steam Deck u Steam Frame: l-AppImage għal Linux
web-play = LAGĦAB FIL-BROWSER
    .facts = Hawn stess, xejn x'tinstalla
web-download-platforms = Windows, Linux u macOS, Steam Deck u Steam Frame
web-download-version = Il-verżjoni tad-download
web-gives-vr = VR
    .why = Itir f'VR fuq kwalunkwe headset kompatibbli ma' OpenXR u s-Steam Frame
web-gives-hotas = HOTAS
    .why = Appoġġ sħiħ għall-gamepads, joysticks, throttles u pedali
web-gives-multiplayer = Multiplayer aħjar
    .why = Konnessjoni UDP diretta mas-servers għal inqas lag
web-gives-server = Oħloq server tiegħek
    .why = Oħloq kampanja coop, jew mexxi server dedikat
web-gives-head-tracking = Head tracking
    .why = Trackers stil TrackIR permezz ta' OpenTrack
web-gives-tacview = Tacview
    .why = Reġistrazzjonijiet tat-titjir u telemetrija fil-ħin reali
web-gives-keep = Tiegħek għal dejjem
    .why = Jekk is-sit jaqa' xorta jibqagħlek il-kopja tiegħek
menu-discord = DISCORD
    .facts =
        Ittajjru flimkien, irrapporta bugs u aqsam missjonijiet
        Segwi l-iżvilupp
menu-quit = OĦROĠ
    .facts = Oħroġ lejn id-desktop
menu-version = VERŻJONI { $version }

## Skrin tal-missjoni waħda

menu-career-sortie = TITJIRA TAL-KARRIERA
menu-tour-sortie = { $pilot }  //  TITJIRA { $sortie } MINN { $length }
menu-msn = SISTEMA TA' IPPJANAR TAL-MISSJONI  //  MSN { $seed }
menu-no-targets = Ma fadlux miri wieqfa għal dan il-perjodu
menu-plan-route = IPPJANA R-ROTTA
    .facts =
        Il-mappa tat-teatru bil-kopertura tar-radar u tas-SAM tal-għadu
        Żid, ċaqlaq u neħħi punti ta' navigazzjoni
        Il-miri u l-bażi tal-inżul jibqgħu fissi
menu-loadout = ARMAMENTI
    .facts =
        Agħżel l-istores fuq kull bay jew pylon
        Jibda bl-armi tal-briefing
menu-generating = QED JINĦOLOQ...
menu-back-records = Lura għar-rekords tal-piloti
menu-back-main = Lura għall-menu prinċipali
menu-new-sortie = TITJIRA ĠDIDA
menu-new-mission = MISSJONI ĠDIDA
    .facts =
        Jagħżel għanijiet differenti
        Iżomm l-opzjonijiet kollha u l-ħin tal-bidu
menu-load = ILLOADJA
    .facts =
        Missjonijiet maħżuna waqt it-titjira (menu: SAVE MISSION)
        Kompli minn fejn inħażnu
menu-fly = ITIR
    .facts = Itlaq fuq il-missjoni tal-briefing
menu-fly-vr = ITIR F'VR
    .facts =
        Jerġa' jibda l-logħba fil-headset VR
        Jittajjar eżattament din il-missjoni
    .no-mission = Oħloq missjoni l-ewwel
    .no-runtime = L-ebda runtime OpenXR (Monado, WiVRn, SteamVR) mhu attiv
menu-vr-failed = ma seta' jibda f'VR: { $error }

## Sommarju tal-missjoni

summary-theater = TEATRU
summary-situation = SITWAZZJONI
summary-mission = MISSJONI
summary-opponents = AVVERSARJI
summary-time = ĦIN
summary-weather = TEMP
summary-fuel = FUEL
summary-night = Lejl
summary-day = Nhar
summary-time-value = { $daytime }  tluq { $depart }
summary-weather-value = { $sky }  viż. { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Armamenti: { $stores }
summary-intelligence = INTELLIGENCE
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
