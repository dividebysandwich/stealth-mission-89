# A főmenü és az egyedi küldetés képernyő (src/menu.rs).

## Hadszínterek (stealth_host::theater::THEATERS)

theater-libya = Líbia (1986)
    .facts =
        A Szidra-öböl és a líbiai part
        Az El Dorado Canyon hadművelet kora
        Legtöbbször tiszta sivatagi égbolt
theater-persian_gulf = Perzsa-öböl (1984)
    .facts =
        Az iráni-iraki tankerháború kora
        Tenger és tengerparti sivatag felett
        Az összes hadszíntér közül a legtisztább időjárás
theater-north_cape = Északi-fok
    .facts =
        Sarkvidéki Norvégia és a Kola-félsziget
        A szovjet Északi Flotta otthona
        Gyakori alacsony felhőzet
theater-central_europe = Közép-Európa
    .facts =
        NATO a Varsói Szerződés ellen Németország felett
        Az idő csak 15%-ában tiszta az ég
theater-vietnam = Vietnam
    .facts =
        Hanoi és Haiphong térsége
        Dzsungel és hegyvidék
        A küldetések 30%-ában zivatar
theater-middle_east = Közel-Kelet
    .facts =
        A Földközi-tenger keleti medencéje és a Levante
        Többnyire tiszta égbolt
theater-desert_storm = Sivatagi vihar
    .facts =
        Kuvait és Irak, 1991
        Az F-117A valódi harci bemutatkozása
        Többnyire tiszta sivatagi égbolt
theater-cuba = Kuba
    .facts =
        Karib-tengeri szigeti hadszíntér
        Tengeri és szárazföldi célpontok
        Trópusi: a küldetések 15%-ában vihar
theater-korea = Korea
    .facts =
        A Koreai-félsziget a DMZ-től északra
        Hegyvidéki terep
        Változékony időjárás, bármilyen égbolt előfordulhat
theater-ukraine = Ukrajna (2026)
    .facts =
        Kelet-Ukrajna, a Krím és Dél-Oroszország
        A frontvonal Harkivtól Herszonig ég
        Nyílt szántóföldek, romos városok a fronton

## Helyzetek (bevetési szabályok)

roe-0 = Hidegháború
    .facts =
        Titkos békeidős művelet
        A leleplezetlenség a legfontosabb
        Kevesebb ellenséges repülőgép járőrözik
        A pontos bevetési szabályok a briefingben vannak
roe-1 = Korlátozott háború
    .facts =
        Regionális fegyveres konfliktus
        Az ellenséges légvédelem készültségben
        Több ellenséges repülőgép, mint a hidegháborúban
roe-2 = Hagyományos háború
    .facts =
        Teljes körű háború
        Minden ellenséges erő aktív
        Több ellenséges repülőgép, mint a hidegháborúban

## Küldetéstípusok

kind-strike = Csapásmérés
    .facts =
        Földi cél megsemmisítése, lefényképezése vagy elérése
        Elsődleges és másodlagos célpont
        Levegő-föld fegyverzet
kind-air = Légi harc
    .facts =
        Ellenséges repülőgép elfogása
        Szállítók, bombázók vagy vadászjárőrök
        Levegő-levegő rakétás fegyverzet
kind-strike-training = Csapásmérő gyakorlat
    .facts =
        Gyakorló csapásmérő küldetés
        A fegyverek és a navigáció megtanulása
kind-air-training = Légiharc-gyakorlat
    .facts =
        Gyakorló levegő-levegő küldetés
        Az elfogás és a rakéták megtanulása
kind-drone-defense = Drónvédelem
    .facts =
        Csak Ukrajnában
        Geran csapásmérő drónok hullámának megállítása, mielőtt várost vagy atomerőművet érne
        Majd a kilövőhelyük bombázása

## Ellenfelek

opponents-0 = Újonc
    .facts =
        Tapasztalatlan ellenséges személyzetek
        A legkönnyebb fokozat
opponents-1 = Átlagos
    .facts =
        Átlagos ellenséges személyzetek
        Alapértelmezett fokozat
opponents-2 = Veterán
    .facts =
        Képzett ellenséges személyzetek
        Több hajó és járőr
opponents-3 = Elit
    .facts =
        A legjobb ellenséges pilóták és SAM-kezelők
        A legnehezebb fokozat

## Időjárás

sky-random = Véletlen
    .facts =
        A hadszíntér éghajlatából sorsolva
        Minden új küldetéssel változik
        A felhők mindkét fél látóvonalát elzárják
sky-clear = Tiszta
    .facts =
        Nincs felhő
        Látástávolság 40 km
        A klasszikus DOS-kor égboltja
sky-scattered = Szórt
    .facts =
        Gomolyfelhő 20-45%, alap 2500-6000 ft
        Néha vékony magas rétegfelhő fent
        Látástávolság 30 km
sky-broken = Szakadozott
    .facts =
        Gomolyfelhő 45-70%, alap 2000-5000 ft
        Gyakran gomolyos rétegfelhő-réteg fölötte
        Látástávolság 25 km
sky-overcast = Borult
    .facts =
        Rétegfelhő 80-100%, alap 1500-5000 ft
        Néhány gomolyfelhő-töredék alatta
        Látástávolság 20 km
sky-storm = Vihar
    .facts =
        Zivatarfelhő-tornyok 40000 ft-ig
        Alacsony rétegfelhő és gomolyfelhő 1000 ft-tól
        Látástávolság 8 km
menu-random-cover = Véletlen borítottság
    .facts =
        A fő felhőréteg borítottsága az időjárással együtt sorsolva
        Törölje a pipát, ha kézzel állítja be
        Fő réteg: borult és viharos időben rétegfelhő, egyébként gomolyfelhő
menu-cover-slider =
    .facts =
        Az égbolt azon része, amelyet a fő réteg borít
        Tiszta égbolton nincs hatása

## Hold

moon-date = A küldetés dátuma
    .facts =
        A briefing dátumának valódi holdja
        Szükséges hozzá a napszak kapcsoló (modern)
moon-0 = Újhold
    .facts =
        Nincs holdfény: fekete éjszaka, csak fények és csillagok
        Az ellenséges pilóták 2 km-ről látják Önt
moon-1 = Növekvő sarló
    .facts = Vékony hold az esti égen, éjfél előtt lenyugszik
moon-2 = Első negyed
    .facts = Félig megvilágított, este fent van, éjfél körül lenyugszik
moon-3 = Növekvő hold
    .facts = Fényes, délutántól éjfél utánig fent van
moon-4 = Telihold
    .facts =
        Egész éjjel fent van, éjfélkor magasan
        Az ellenséges pilóták 4 km-ről látják Önt
moon-5 = Fogyó hold
    .facts = Fényes, este kel, reggelig fent van
moon-6 = Utolsó negyed
    .facts = Félig megvilágított, éjfél körül kel
moon-7 = Fogyó sarló
    .facts = Vékony hold, hajnal előtt kel

## Repülőgépek

aircraft-f117 =
    .facts =
        Lopakodó csapásmérő repülőgép
        Szubszonikus, 7 g
        Modern és klasszikus pilótafülke
aircraft-f16 =
    .facts =
        Sokoldalú vadászgép
        Mach 2 kategória, 9 g, gyors orsózás
        F-16 pilótafülke RWR-rel
aircraft-tornado =
    .facts =
        Kis magasságú csapásmérő repülőgép
        A változtatható szárnynyilazás a sebességgel áll be
        Modern pilótafülke, három MFD
aircraft-a10 =
    .facts =
        Közvetlen légi támogató repülőgép
        Lassú és strapabíró, 30 mm-es gépágyú
        A-10C pilótafülke, két MFD
aircraft-uh60 =
    .facts =
        Szállítóhelikopter
        Katonákat tesz le: szálljon le a cél közelében
        Ciklikus és kollektív
aircraft-ah1z =
    .facts =
        Támadóhelikopter
        Forgótornyos gépágyú, Hellfire-ok, AIM-9
        Az ágyú és a Hellfire-ok a pod-ot követik

## Szakaszok és csoportok

section-theater = Hadszíntér
section-opponents = Ellenfelek
section-situation = Helyzet
section-mission = Küldetés
section-moon = Hold
section-start-time = Indulási idő
section-weather = Időjárás
section-realism = Realizmus
section-tacview = Tacview
section-mission-summary = Küldetés összegzése
section-briefing = Briefing
group-mission = Küldetés
group-conditions = Körülmények
group-vehicle = Gép
group-simulation = Szimuláció

## A küldetés beállításai

menu-start-random = Véletlen
    .facts =
        Minden új küldetéssel sorsolva, mint a klasszikus módban
        06:00-08:00 nappal vagy 18:00-20:00 éjjel
        Törölje a pipát az óra kiválasztásához
menu-start-slider =
    .facts =
        Felszállás ideje
        Éjszaka 18:00-tól 06:00-ig
        Az ÚJ KÜLDETÉS megtartja
menu-realistic-weapons = Valósághű fegyverzet
    .facts =
        A valódi repülőgép korlátai
        2 fegyverterem 4 helyett
        Nincsenek AIM-9 vagy AIM-120 rakéták
        Nincs repülőgép-hordozós fel- és leszállás
menu-crash-protection = Ütközésvédelem
    .facts =
        Automatikusan felhúz a talaj közelében (behúzott futóval)
        A földet érés és az ütközések nem végzetesek
        Kikapcsolva: a talajnak mindig neki lehet menni
menu-tacview-export = Exportálás
    .facts =
        Minden repülést Tacview-felvételként (.acmi) ment
        A játék fájlmappájában, a tacview alatt
menu-tacview-real-time = Valós idejű telemetria
    .facts =
        Repülés közben a Tacview-nak küldi a repülést
        A Tacview-ban: Record > Real-Time Telemetry
        Cím 127.0.0.1, port 42674 (--tacview-address)
switch-game = JÁTÉK
switch-real = VALÓS
menu-damage = Sérülés
    .facts =
        A fegyverek sérülése
        Játék: klasszikus sérülés; a találatok összeadódnak, de sosem lőnek le
        Valósághű: egy rakétatalálat megsemmisítheti a gépet (katapultáljon!), több rendszert sért és gyakran tüzet okoz, amely kialszik vagy felrobbantja a gépet
        Valósághű: a sérült kormányfelületek kitrimmelik vagy lengésbe hozzák a gépet
menu-guidance = Rávezetés
    .facts =
        A rakéták rávezetése
        Klasszikus: a rakéták egyenesen a célra repülnek
        Valósághű: arányos navigáció az ütközési pontra; a hajtómű kiég, a rakéták tovább siklanak, és minden fordulóban sebességet vesztenek (füst csak égés közben)
        Valósághű: a SAM-ok távoli célpontnál ívben emelkednek; az AIM-120 3 másodpercig egyenesen repül (húzza fel a lobbolásához)
menu-air-defence = Légvédelem
    .facts =
        Kis magasságú légvédelem
        Klasszikus: csak a SAM-állások és a vadászgépek
        Valósághű: radarvezérelt ZSU-23-4 ágyúk (radarjuk SAM-radarként lát Önt) és S-60 légvédelmi ütegek őrzik a repülőtereket és a célpontokat; a városokban MANPADS-csapatok SA-14-esekkel lőnek arra, amit látnak
        Védett helyek felett alacsonyan repülni veszélyessé válik; a bombák és az ágyú megsemmisítik az ágyúkat

## Főképernyő

menu-title = STEALTH MISSION 89
menu-subtitle = KÜLDETÉSTERVEZŐ RENDSZER
menu-single-mission = EGYEDI KÜLDETÉS
    .facts =
        Egy küldetés a választott hadszíntéren
        Válassza ki a beállításokat, tervezze meg az útvonalat és a fegyverzetet
menu-career = KARRIER
    .facts =
        A pilóták nyilvántartása: rendfokozat, kitüntetések, bevetések
        Szolgálati idő: 12 bevetés egy hadszíntéren
        Amit megsemmisít, megsemmisítve marad
menu-campaign = DINAMIKUS KAMPÁNY
    .facts =
        Valós idejű dinamikus kampány
        Foglalja el az ellenség városait és repülőtereit
        Küldetések az élő helyzetből
menu-multiplayer = TÖBBJÁTÉKOS
    .facts = Kooperatív játék a dinamikus kampányban
    .join = Csatlakozás a nyilvános lista egyik szerveréhez
    .host = Csatlakozás szerverhez, vagy kampány hosztolása és repülés benne
menu-options = BEÁLLÍTÁSOK
    .facts =
        Játék: pilótafülke, HUD és avionika stílusa, automatikus trim
        Grafika, hangerők, billentyű- és joystick-kiosztás
        Mentve a következő repülésekhez
    .short = Játék, grafika, hang és vezérlés
menu-download = LETÖLTÉS
    .facts =
        Az asztali játék Windowsra, Linuxra és macOS-re
        VR-rel és többjátékos móddal
        Steam Deck és Steam Frame: a Linux AppImage
web-play = JÁTÉK A BÖNGÉSZŐBEN
    .facts = Itt helyben, semmit sem kell telepíteni
web-download-platforms = Windows, Linux és macOS, Steam Deck és Steam Frame
web-download-version = A letölthető változat
web-gives-vr = VR
    .why = Repüljön VR-ban bármely OpenXR-kompatibilis headsettel és a Steam Frame-mel
web-gives-hotas = HOTAS
    .why = Teljes támogatás játékvezérlőkhöz, joystickokhoz, gázkarokhoz és pedálokhoz
web-gives-multiplayer = Jobb többjátékos mód
    .why = Közvetlen UDP-kapcsolat a szerverekhez, kisebb késleltetéssel
web-gives-server = Saját szerver hosztolása
    .why = Kooperatív kampány hosztolása, vagy dedikált szerver futtatása
web-gives-head-tracking = Fejkövetés
    .why = TrackIR-típusú követők az OpenTracken keresztül
web-gives-tacview = Tacview
    .why = Repülésfelvételek és valós idejű telemetria
web-gives-keep = Örökre az Öné
    .why = Ha a weboldal leáll, megvan a saját példánya
menu-discord = DISCORD
    .facts =
        Repüljenek együtt, jelentsék a hibákat és osszák meg a küldetéseket
        Kövesse a fejlesztést
menu-quit = KILÉPÉS
    .facts = Kilépés az asztalra
menu-version = VERZIÓ { $version }

## Egyedi küldetés képernyő

menu-career-sortie = KARRIER-BEVETÉS
menu-tour-sortie = { $pilot }  //  SZOLGÁLATI IDŐ { $sortie }. BEVETÉS / { $length }
menu-msn = KÜLDETÉSTERVEZŐ RENDSZER  //  MSN { $seed }
menu-no-targets = Nem maradt álló célpont ehhez a szolgálati időhöz
menu-plan-route = ÚTVONALTERVEZÉS
    .facts =
        A hadszíntér térképe az ellenséges radar- és SAM-lefedettséggel
        Navigációs pontok hozzáadása, mozgatása és törlése
        A célpontok és a leszállóbázis rögzítettek
menu-loadout = FEGYVERZET
    .facts =
        Válassza ki a felszerelést minden teremhez vagy fegyverfüggesztőhöz
        A briefingben szereplő fegyverekkel indul
menu-generating = GENERÁLÁS...
menu-back-records = Vissza a pilóták nyilvántartásához
menu-back-main = Vissza a főmenübe
menu-new-sortie = ÚJ BEVETÉS
menu-new-mission = ÚJ KÜLDETÉS
    .facts =
        Más célpontokat sorsol
        Megtartja az összes beállítást és az indulási időt
menu-load = BETÖLTÉS
    .facts =
        A repülés közben mentett küldetések (menü: KÜLDETÉS MENTÉSE)
        Folytassa ott, ahol mentette
menu-fly = REPÜLÉS
    .facts = Felszállás a briefingben szereplő küldetésre
menu-fly-vr = REPÜLÉS VR-BAN
    .facts =
        Újraindítja a játékot a VR headsetben
        Pontosan ezt a küldetést repüli
    .no-mission = Előbb generáljon küldetést
    .no-runtime = Nincs aktív OpenXR runtime (Monado, WiVRn, SteamVR)
menu-vr-failed = nem sikerült VR-ban elindítani: { $error }

## Küldetés összegzése

summary-theater = HADSZÍNTÉR
summary-situation = HELYZET
summary-mission = KÜLDETÉS
summary-opponents = ELLENFELEK
summary-time = IDŐ
summary-weather = IDŐJÁRÁS
summary-fuel = ÜZEMANYAG
summary-night = Éjszaka
summary-day = Nappal
summary-time-value = { $daytime }  indulás { $depart }
summary-weather-value = { $sky }  látástáv. { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Fegyverzet: { $stores }
summary-intelligence = FELDERÍTÉS
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
