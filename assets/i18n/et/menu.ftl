# Peamenüü ja üksikmissiooni ekraan (src/menu.rs).

## Operatsioonialad (stealth_host::theater::THEATERS)

theater-libya = Liibüa (1986)
    .facts =
        Sidra laht ja Liibüa rannik
        Operatsiooni El Dorado Canyon aeg
        Enamasti selge kõrbetaevas
theater-persian_gulf = Pärsia laht (1984)
    .facts =
        Iraani-Iraagi tankersõja aeg
        Meri ja rannikukõrb
        Kõigist operatsioonialadest selgeim ilm
theater-north_cape = North Cape
    .facts =
        Arktiline Norra ja Koola poolsaar
        Nõukogude Põhjalaevastiku kodu
        Sage madal pilvekiht
theater-central_europe = Kesk-Euroopa
    .facts =
        NATO vs Varssavi pakt Saksamaa kohal
        Selge taevas vaid 15% ajast
theater-vietnam = Vietnam
    .facts =
        Hanoi ja Haiphongi piirkond
        Džungel ja mägimaastik
        Äikesetormid 30% missioonidest
theater-middle_east = Lähis-Ida
    .facts =
        Ida-Vahemeri ja Levant
        Enamasti selge taevas
theater-desert_storm = Desert Storm
    .facts =
        Kuveit ja Iraak, 1991
        F-117A päris lahingdebüüt
        Enamasti selge kõrbetaevas
theater-cuba = Kuuba
    .facts =
        Kariibi mere saarte operatsiooniala
        Mere- ja maismaasihtmärgid
        Troopiline: tormid 15% missioonidest
theater-korea = Korea
    .facts =
        Korea poolsaar DMZ-st põhja pool
        Mägimaastik
        Muutlik ilm, võimalik on iga taevas
theater-ukraine = Ukraina (2026)
    .facts =
        Ida-Ukraina, Krimm ja Lõuna-Venemaa
        Rindejoon Harkivist Hersonini põleb
        Avatud põllumaa, rindel varemetes linnad

## Olukorrad (kokkupuute reeglid)

roe-0 = Külm sõda
    .facts =
        Varjatud rahuaegne operatsioon
        Kõige olulisem on jääda märkamatuks
        Patrullil vähem vaenlase lennukeid
        Täpsed kokkupuutereeglid on briifingus
roe-1 = Piiratud sõda
    .facts =
        Piirkondlik relvakonflikt
        Vaenlase õhutõrje on valmis
        Vaenlase lennukeid rohkem kui külma sõja ajal
roe-2 = Tavasõda
    .facts =
        Täiemahuline sõda
        Kõik vaenlase väed on aktiivsed
        Vaenlase lennukeid rohkem kui külma sõja ajal

## Missioonitüübid

kind-strike = Löök
    .facts =
        Hävita, pildista või jõua maapealse sihtmärgini
        Põhi- ja lisaeesmärk
        Õhk-maa relvastus
kind-air = Õhk-õhk
    .facts =
        Pealtkuulamine: vaenlase õhusõiduk
        Transpordilennukid, pommitajad või hävitajate patrullid
        Õhk-õhk rakettide relvastus
kind-strike-training = Löögitreening
    .facts =
        Harjutuslöögi missioon
        Õpi relvi ja navigatsiooni
kind-air-training = Õhutreening
    .facts =
        Õhk-õhk harjutusmissioon
        Õpi pealtkuulamisi ja rakette
kind-drone-defense = Droonitõrje
    .facts =
        Ainult Ukraina
        Peata Geran rünnakudroonide laine enne, kui see tabab linna või tuumajaama
        Seejärel pommita nende stardikohta

## Vastased

opponents-0 = Algaja
    .facts =
        Kogenematud vaenlase meeskonnad
        Kõige kergem raskusaste
opponents-1 = Tavaline
    .facts =
        Keskmised vaenlase meeskonnad
        Vaikimisi raskusaste
opponents-2 = Veteran
    .facts =
        Oskuslikud vaenlase meeskonnad
        Rohkem laevu ja patrulle
opponents-3 = Eliit
    .facts =
        Vaenlase parimad piloodid ja SAM-meeskonnad
        Kõige raskem raskusaste

## Ilm

sky-random = Juhuslik
    .facts =
        Loositakse operatsiooniala kliimast
        Muutub iga uue missiooniga
        Pilved varjavad nähtavust mõlemale poolele
sky-clear = Selge
    .facts =
        Pilvi pole
        Nähtavus 40 km
        Klassikaline DOS-ajastu taevas
sky-scattered = Hajusad pilved
    .facts =
        Rünkpilved 20-45%, alus 2500-6000 ft
        Mõnikord õhuke kõrgkihtpilv kõrgemal
        Nähtavus 30 km
sky-broken = Katkendlik pilvisus
    .facts =
        Rünkpilved 45-70%, alus 2000-5000 ft
        Sageli kihtrünkpilvede kiht kõrgemal
        Nähtavus 25 km
sky-overcast = Pilvine
    .facts =
        Kihtpilved 80-100%, alus 1500-5000 ft
        Allpool mõned rünkpilvede killud
        Nähtavus 20 km
sky-storm = Torm
    .facts =
        Rünksajupilved kuni 40000 ft kõrguseni
        Madalad kihtpilved ja rünkpilved alates 1000 ft
        Nähtavus 8 km
menu-random-cover = Juhuslik pilvisus
    .facts =
        Peamise pilvekihi pilvisus loositakse koos ilmaga
        Eemalda linnuke, et see ise määrata
        Peamine kiht: kihtpilved pilvise ja tormise taeva korral, muidu rünkpilved
menu-cover-slider =
    .facts =
        Kui suure osa taevast peamine kiht katab
        Selgele taevale mõju pole

## Kuu

moon-date = Missiooni kuupäev
    .facts =
        Briifingu kuupäeva tegelik kuu
        Vajab kellaaja lülitit (kaasaegne)
moon-0 = Noorkuu
    .facts =
        Kuuvalgust pole: must öö, ainult tuled ja tähed
        Vaenlase piloodid näevad sind 2 km kauguselt
moon-1 = Kasvav sirp
    .facts = Õhuke kuu õhtutaevas, loojub enne südaööd
moon-2 = Esimene veerand
    .facts = Pooleldi valgustatud, õhtul taevas, loojub südaöö paiku
moon-3 = Kasvav kuu
    .facts = Särav, taevas pärastlõunast kuni pärast südaööd
moon-4 = Täiskuu
    .facts =
        Taevas terve öö, südaööl kõrgel
        Vaenlase piloodid näevad sind 4 km kauguselt
moon-5 = Kahanev kuu
    .facts = Särav, tõuseb õhtul, taevas hommikuni
moon-6 = Viimane veerand
    .facts = Pooleldi valgustatud, tõuseb südaöö paiku
moon-7 = Kahanev sirp
    .facts = Õhuke kuu, tõuseb enne koitu

## Õhusõidukid

aircraft-f117 =
    .facts =
        Stealth-ründelennuk
        Alahelikiirus, 7 g
        Kaasaegne ja klassikaline cockpit
aircraft-f16 =
    .facts =
        Mitmeotstarbeline hävitaja
        Mach 2 klass, 9 g, kiire rull
        F-16 cockpit RWR-iga
aircraft-tornado =
    .facts =
        Madallennu ründelennuk
        Muudetava noolega tiivad pöörduvad kiirusega kaasa
        Kaasaegne cockpit, kolm MFD-d
aircraft-a10 =
    .facts =
        Lähitoetuslennuk
        Aeglane ja vastupidav, 30 mm kahur
        A-10C cockpit, kaks MFD-d
aircraft-uh60 =
    .facts =
        Üldotstarbeline kopter
        Maandab vägesid: maandu sihtmärgi lähedal
        Tsükliline ja ühine samm
aircraft-ah1z =
    .facts =
        Ründekopter
        Tornikahur, Hellfire'id, AIM-9
        Kahur ja Hellfire'id järgivad pod'i

## Jaotised ja rühmad

section-theater = Operatsiooniala
section-opponents = Vastased
section-situation = Olukord
section-mission = Missioon
section-moon = Kuu
section-start-time = Algusaeg
section-weather = Ilm
section-realism = Realism
section-tacview = Tacview
section-mission-summary = Missiooni kokkuvõte
section-briefing = Briifing
group-mission = Missioon
group-conditions = Tingimused
group-vehicle = Sõiduk
group-simulation = Simulatsioon

## Missiooni seaded

menu-start-random = Juhuslik
    .facts =
        Loositakse iga uue missiooniga, nagu klassikalises režiimis
        06:00-08:00 päeval või 18:00-20:00 öösel
        Eemalda linnuke, et kellaaeg ise valida
menu-start-slider =
    .facts =
        Õhkutõusu aeg
        Öö kell 18:00 kuni 06:00
        Säilib nupuga UUS MISSIOON
menu-realistic-weapons = Realistlikud relvad
    .facts =
        Päris lennuki piirangud
        2 relvaruumi 4 asemel
        AIM-9 ja AIM-120 rakette pole
        Lennukikandjalt õhkutõusu ega maandumist pole
menu-crash-protection = Kokkupõrkekaitse
    .facts =
        Tõmbab maa lähedal automaatselt üles (telik sees)
        Maapinna puudutus ja kokkupõrked pole saatuslikud
        Väljas: maapinda saab alati tabada
menu-tacview-export = Eksport
    .facts =
        Salvestab iga lennu Tacview salvestisena (.acmi)
        Mängu failikaustas, kausta tacview
menu-tacview-real-time = Reaalaja telemeetria
    .facts =
        Saadab lennu Tacview'sse lennu ajal
        Tacview's: Record > Real-Time Telemetry
        Aadress 127.0.0.1, port 42674 (--tacview-address)
switch-game = MÄNG
switch-real = REAAL
menu-damage = Kahjustused
    .facts =
        Relvade kahjustus
        Mäng: klassikaline kahjustus; tabamused summeeruvad, kuid ei too lennukit alla
        Realistlik: raketitabamus võib lennuki hävitada (katapulteeru!), kahjustab rohkem süsteeme ja süütab tihti tulekahju, mis kustub või lennuki õhku lööb
        Realistlik: kahjustatud juhtpinnad viivad lennuki trimmist välja või panevad selle võnkuma
menu-guidance = Juhtimine
    .facts =
        Rakettide juhtimine
        Klassikaline: raketid lendavad sihtmärgi poole sirgjooneliselt
        Realistlik: proportsionaalne navigatsioon kokkupõrkepunkti; mootorid põlevad läbi ja raketid libisevad edasi, kaotades igas pöördes kiirust (suits ainult mootori töötamise ajal)
        Realistlik: SAM-id tõusevad kaugete sihtmärkide vastu kõrgele kaarele; AIM-120 lendab 3 s sirgelt (tõmba üles, et seda kaarega visata)
menu-air-defence = Õhutõrje
    .facts =
        Madalatel kõrgustel tegutsev õhutõrje
        Klassikaline: ainult SAM-kohad ja hävitajad
        Realistlik: radariga juhitud ZSU-23-4 kahurid (nende radar näeb sind nagu SAM-i oma) ja S-60 õhutõrjepatareid valvavad lennuvälju ja sihtmärke; linnades MANPADS-meeskonnad tulistavad SA-14-sid sihtmärkide pihta, mida nad näevad
        Madal lend kaitstud kohtade kohal muutub ohtlikuks; pommid ja kahurid hävitavad kahureid

## Peaekraan

menu-title = STEALTH MISSION 89
menu-subtitle = MISSIOONIPLANEERIMISE SÜSTEEM
menu-single-mission = ÜKSIKMISSIOON
    .facts =
        Üks missioon sinu valitud operatsioonialal
        Vali seaded, planeeri marsruut ja relvastus
menu-career = KARJÄÄR
    .facts =
        Pilootide andmed: auaste, autasud, lennud
        Teenistusperiood: 12 lendu ühel operatsioonialal
        Mida hävitad, jääb hävitatuks
menu-campaign = DÜNAAMILINE KAMPAANIA
    .facts =
        Dünaamiline kampaania reaalajas
        Vallu vaenlase linnad ja lennuväljad
        Missioonid hetkeolukorrast
menu-multiplayer = MITMIKMÄNG
    .facts = Koostöö dünaamilises kampaanias
    .join = Liitu avalikus nimekirjas oleva serveriga
    .host = Liitu serveriga või korralda kampaania ja lenda selles
menu-options = SEADED
    .facts =
        Mäng: cockpiti, HUD-i ja avioonika stiil, auto trim
        Graafika, helitugevused, klahvide ja joystick'i sidumised
        Salvestatakse järgmisteks lendudeks
    .short = Mäng, graafika, heli ja juhtimine
menu-download = LAADI ALLA
    .facts =
        Töölauamäng Windowsile, Linuxile ja macOS-ile
        VR-i ja mitmikmänguga
        Steam Deck ja Steam Frame: Linuxi AppImage
web-play = MÄNGI BRAUSERIS
    .facts = Just siin, midagi pole vaja paigaldada
web-download-platforms = Windows, Linux ja macOS, Steam Deck ja Steam Frame
web-download-version = Allalaaditav versioon
web-gives-vr = VR
    .why = Lenda VR-is mis tahes OpenXR-iga ühilduva peakomplekti ja Steam Frame'iga
web-gives-hotas = HOTAS
    .why = Täielik tugi gamepad'idele, joystick'idele, gaasihoobadele ja pedaalidele
web-gives-multiplayer = Parem mitmikmäng
    .why = Otsene UDP-ühendus serveritega väiksema viivituse jaoks
web-gives-server = Oma server
    .why = Korralda koostöökampaania või käivita dedikeeritud server
web-gives-head-tracking = Pea jälgimine
    .why = TrackIR-tüüpi jälgijad OpenTracki kaudu
web-gives-tacview = Tacview
    .why = Lennusalvestised ja reaalaja telemeetria
web-gives-keep = Jääb sulle
    .why = Kui veebileht kaob, on sul oma koopia alles
menu-discord = DISCORD
    .facts =
        Lenda koos, teata vigadest ja jaga missioone
        Jälgi arendust
menu-quit = VÄLJU
    .facts = Välju töölauale
menu-version = VERSIOON { $version }

## Üksikmissiooni ekraan

menu-career-sortie = KARJÄÄRI LEND
menu-tour-sortie = { $pilot }  //  PERIOODI LEND { $sortie } / { $length }
menu-msn = MISSIOONIPLANEERIMISE SÜSTEEM  //  MSN { $seed }
menu-no-targets = Sellel teenistusperioodil pole enam püsti seisvaid sihtmärke
menu-plan-route = PLANEERI MARSRUUT
    .facts =
        Operatsiooniala kaart vaenlase radari ja SAM-ide katvusega
        Lisa, nihuta ja eemalda navigatsioonipunkte
        Sihtmärgid ja maandumisbaas jäävad paigale
menu-loadout = RELVASTUS
    .facts =
        Vali varustus igale relvaruumile või pülooni
        Algab briifingus antud relvadega
menu-generating = LOOMINE...
menu-back-records = Tagasi pilootide andmete juurde
menu-back-main = Tagasi peamenüüsse
menu-new-sortie = UUS LEND
menu-new-mission = UUS MISSIOON
    .facts =
        Loosib uued eesmärgid
        Säilitab kõik seaded ja algusaja
menu-load = LAADI
    .facts =
        Lennu ajal salvestatud missioonid (menüü: SALVESTA MISSIOON)
        Jätka sealt, kus need salvestati
menu-fly = LENDA
    .facts = Tõuse õhku briifitud missioonil
menu-fly-vr = LENDA VR-IS
    .facts =
        Taaskäivitab mängu VR-peakomplektis
        Lendab täpselt seda missiooni
    .no-mission = Loo kõigepealt missioon
    .no-runtime = Ükski OpenXR runtime (Monado, WiVRn, SteamVR) pole aktiivne
menu-vr-failed = VR-is käivitamine ebaõnnestus: { $error }

## Missiooni kokkuvõte

summary-theater = OPERATSIOONIALA
summary-situation = OLUKORD
summary-mission = MISSIOON
summary-opponents = VASTASED
summary-time = AEG
summary-weather = ILM
summary-fuel = KÜTUS
summary-night = Öö
summary-day = Päev
summary-time-value = { $daytime }  start { $depart }
summary-weather-value = { $sky }  nähtavus { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Relvastus: { $stores }
summary-intelligence = LUURE
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
