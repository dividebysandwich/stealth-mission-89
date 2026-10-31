# Galvenā izvēlne un vienas misijas ekrāns (src/menu.rs).

## Teātri (stealth_host::theater::THEATERS)

theater-libya = Lībija (1986)
    .facts =
        Sidras līcis un Lībijas krasts
        Operācijas El Dorado Canyon laiks
        Lielāko daļu dienu skaidras tuksneša debesis
theater-persian_gulf = Persijas līcis (1984)
    .facts =
        Irānas un Irākas tankkuģu kara laiks
        Virs jūras un piekrastes tuksneša
        Visskaidrākie laikapstākļi no visiem teātriem
theater-north_cape = Ziemeļkaps
    .facts =
        Arktiskā Norvēģija un Kolas pussala
        Padomju Ziemeļu flotes mājvieta
        Bieži zema mākoņu kārta
theater-central_europe = Centrāleiropa
    .facts =
        NATO pret Varšavas paktu virs Vācijas
        Skaidras debesis tikai 15% laika
theater-vietnam = Vjetnama
    .facts =
        Hanojas un Haifonas apgabals
        Džungļi un kalnains reljefs
        Pērkona negaiss 30% misiju
theater-middle_east = Tuvie Austrumi
    .facts =
        Austrumu Vidusjūra un Levants
        Pārsvarā skaidras debesis
theater-desert_storm = Tuksneša vētra
    .facts =
        Kuveita un Irāka, 1991
        F-117A īstais kaujas debijas brīdis
        Pārsvarā skaidras tuksneša debesis
theater-cuba = Kuba
    .facts =
        Karību jūras salas teātris
        Jūras un sauszemes mērķi
        Tropi: negaiss 15% misiju
theater-korea = Koreja
    .facts =
        Korejas pussala uz ziemeļiem no DMZ
        Kalnains reljefs
        Mainīgi laikapstākļi, iespējamas jebkuras debesis
theater-ukraine = Ukraina (2026)
    .facts =
        Ukrainas austrumi, Krima un Krievijas dienvidi
        Frontes līnija deg no Harkivas līdz Hersonai
        Atklāti lauki, frontē sagrautas pilsētas

## Situācijas (iesaistīšanās noteikumi)

roe-0 = Aukstais karš
    .facts =
        Slepena operācija miera laikā
        Svarīgākais ir palikt nepamanītam
        Mazāk ienaidnieka lidmašīnu patruļā
        Precīzi iesaistīšanās noteikumi ir brīfingā
roe-1 = Ierobežots karš
    .facts =
        Reģionāls bruņots konflikts
        Ienaidnieka pretgaisa aizsardzība ir modrībā
        Vairāk ienaidnieka lidmašīnu nekā aukstajā karā
roe-2 = Konvencionāls karš
    .facts =
        Pilna mēroga karš
        Visi ienaidnieka spēki ir aktīvi
        Vairāk ienaidnieka lidmašīnu nekā aukstajā karā

## Misiju veidi

kind-strike = Trieciens
    .facts =
        Iznīcināt, nofotografēt vai sasniegt zemes mērķi
        Galvenais un sekundārais mērķis
        Gaiss-zeme bruņojums
kind-air = Gaiss-gaiss
    .facts =
        Pārtvert ienaidnieka lidmašīnu
        Transporta lidmašīnas, bumbvedēji vai iznīcinātāju patruļas
        Gaiss-gaiss raķešu bruņojums
kind-strike-training = Trieciena treniņš
    .facts =
        Treniņa trieciena misija
        Apgūt ieroču lietošanu un navigāciju
kind-air-training = Gaisa treniņš
    .facts =
        Treniņa gaiss-gaiss misija
        Apgūt pārtveršanu un raķetes
kind-drone-defense = Aizsardzība pret droniem
    .facts =
        Tikai Ukrainā
        Apturēt Geran triecienu dronu vilni, pirms tas trāpa pilsētai vai atomelektrostacijai
        Pēc tam sabombardēt to palaišanas vietu

## Pretinieki

opponents-0 = Zaļie
    .facts =
        Nepieredzējušas ienaidnieka apkalpes
        Vieglākais līmenis
opponents-1 = Parastie
    .facts =
        Vidējas ienaidnieka apkalpes
        Noklusējuma līmenis
opponents-2 = Veterāni
    .facts =
        Pieredzējušas ienaidnieka apkalpes
        Vairāk kuģu un patruļu
opponents-3 = Elite
    .facts =
        Labākie ienaidnieka piloti un SAM apkalpes
        Grūtākais līmenis

## Laikapstākļi

sky-random = Nejauši
    .facts =
        Izvēlēti pēc teātra klimata
        Mainās ar katru jaunu misiju
        Mākoņi aizsedz redzamību abām pusēm
sky-clear = Skaidrs
    .facts =
        Bez mākoņiem
        Redzamība 40 km
        Klasiskās DOS laikmeta debesis
sky-scattered = Izklaidēti mākoņi
    .facts =
        Cumulus 20-45%, pamatne 2500-6000 ft
        Dažkārt plāns altostratus augstu virs galvas
        Redzamība 30 km
sky-broken = Pārrauti mākoņi
    .facts =
        Cumulus 45-70%, pamatne 2000-5000 ft
        Bieži virs tiem stratocumulus slānis
        Redzamība 25 km
sky-overcast = Apmācies
    .facts =
        Stratus 80-100%, pamatne 1500-5000 ft
        Zem tiem daži cumulus fragmenti
        Redzamība 20 km
sky-storm = Vētra
    .facts =
        Cumulonimbus torņi līdz 40000 ft
        Zems stratus un cumulus no 1000 ft
        Redzamība 8 km
menu-random-cover = Nejaušs mākoņainums
    .facts =
        Galvenā mākoņu slāņa segums tiek izvēlēts kopā ar laikapstākļiem
        Noņemiet atzīmi, lai iestatītu pats
        Galvenais slānis: stratus apmācies un vētras laikā, citādi cumulus
menu-cover-slider =
    .facts =
        Debesu daļa, ko aizsedz galvenais slānis
        Neietekmē skaidras debesis

## Mēness

moon-date = Misijas datums
    .facts =
        Īstais Mēness brīfinga datumā
        Nepieciešams diennakts laika slēdzis (moderns)
moon-0 = Jauns Mēness
    .facts =
        Nav mēness gaismas: melna nakts, tikai gaismas un zvaigznes
        Ienaidnieka piloti jūs redz no 2 km
moon-1 = Augošs sirpis
    .facts = Plāns Mēness vakara debesīs, norietējis pirms pusnakts
moon-2 = Pirmais ceturksnis
    .facts = Pa pusei apgaismots, redzams vakarā, riet ap pusnakti
moon-3 = Augošs Mēness
    .facts = Spilgts, redzams no pēcpusdienas līdz pēc pusnakts
moon-4 = Pilnmēness
    .facts =
        Redzams visu nakti, pusnaktī augstu
        Ienaidnieka piloti jūs redz no 4 km
moon-5 = Dilstošs Mēness
    .facts = Spilgts, lec vakarā, redzams līdz rītam
moon-6 = Pēdējais ceturksnis
    .facts = Pa pusei apgaismots, lec ap pusnakti
moon-7 = Dilstošs sirpis
    .facts = Plāns Mēness, kas lec pirms rītausmas

## Lidmašīnas

aircraft-f117 =
    .facts =
        Slepenā triecienlidmašīna
        Zemskaņas, 7 g
        Moderna un klasiska kabīne
aircraft-f16 =
    .facts =
        Daudzfunkcionāls iznīcinātājs
        Mach 2 klase, 9 g, ātra ruļļošanās
        F-16 kabīne ar RWR
aircraft-tornado =
    .facts =
        Zemlidojuma triecienlidmašīna
        Maināmās ģeometrijas spārni mainās līdz ar ātrumu
        Moderna kabīne, trīs MFD
aircraft-a10 =
    .facts =
        Tuvā gaisa atbalsta lidmašīna
        Lēna un izturīga, 30 mm lielgabals
        A-10C kabīne, divi MFD
aircraft-av8b =
    .facts =
        V/STOL uzbrukuma lidmašīna
        Vektorizēta vilkme: karāšanās, nolaižas jebkur
        Sprauslu svira: R / F, Shift atdurēm
aircraft-uh60 =
    .facts =
        Daudzfunkcionāls helikopters
        Izsēdina karaspēku: nosēdieties tuvu mērķim
        Cikliskā un kolektīvā vadība
aircraft-ah1z =
    .facts =
        Uzbrukuma helikopters
        Torņa lielgabals, Hellfire, AIM-9
        Lielgabals un Hellfire seko pod

## Sadaļas un grupas

section-theater = Operāciju teātris
section-opponents = Pretinieki
section-situation = Situācija
section-mission = Misija
section-moon = Mēness
section-start-time = Starta laiks
section-weather = Laikapstākļi
section-realism = Reālisms
section-tacview = Tacview
section-mission-summary = Misijas kopsavilkums
section-briefing = Brīfings
group-mission = Misija
group-conditions = Apstākļi
group-vehicle = Lidaparāts
group-simulation = Simulācija

## Misijas opcijas

menu-start-random = Nejauši
    .facts =
        Izvēlēts ar katru jaunu misiju, kā klasiskajā režīmā
        06:00-08:00 dienā vai 18:00-20:00 naktī
        Noņemiet atzīmi, lai izvēlētos stundu
menu-start-slider =
    .facts =
        Pacelšanās laiks
        Nakts no 18:00 līdz 06:00
        Saglabā JAUNA MISIJA
menu-realistic-weapons = Reālistisks bruņojums
    .facts =
        Īstās lidmašīnas ierobežojumi
        2 ieroču nodalījumi 4 vietā
        Nav AIM-9 un AIM-120 raķešu
        Nav pacelšanās un nolaišanās uz lidmašīnu bāzes kuģa
menu-crash-protection = Aizsardzība pret avārijām
    .facts =
        Automātiski paceļas tuvu zemei (šasija ievilkta)
        Saskare ar zemi un sadursmes nav letālas
        Izslēgta: zemei vienmēr var trāpīt
menu-tacview-export = Eksports
    .facts =
        Saglabā katru lidojumu kā Tacview ierakstu (.acmi)
        Spēles mapē jūsu failiem, apakšmapē tacview
menu-tacview-real-time = Telemetrija reāllaikā
    .facts =
        Sūta lidojumu uz Tacview, kamēr lidojat
        Tacview: Record > Real-Time Telemetry
        Adrese 127.0.0.1, ports 42674 (--tacview-address)
switch-game = SPĒLE
switch-real = REĀLI
menu-damage = Bojājumi
    .facts =
        Ieroču bojājumi
        Spēle: klasiski bojājumi; trāpījumi summējas, bet nekad nenovērš lidmašīnu
        Reālistiski: raķetes trāpījums var iznīcināt lidmašīnu (katapultējieties!), bojā vairāk sistēmu un bieži izraisa ugunsgrēku, kas izdziest vai uzspridzina lidmašīnu
        Reālistiski: bojātas vadības virsmas izsit lidmašīnu no trim vai liek tai svārstīties
menu-guidance = Vadība
    .facts =
        Raķešu vadība
        Klasiski: raķetes lido taisni uz mērķi
        Reālistiski: proporcionālā navigācija uz sadursmes punktu; dzinēji izdeg un raķetes plānē tālāk, zaudējot ātrumu katrā pagriezienā (dūmi tikai, kamēr dzinējs darbojas)
        Reālistiski: SAM paceļas augstāk pret tāliem mērķiem; AIM-120 lido taisni 3 s (velciet augšup, lai to paceltu)
menu-air-defence = Pretgaisa aizsardzība
    .facts =
        Zemlidojuma pretgaisa aizsardzība
        Klasiski: tikai SAM pozīcijas un iznīcinātāji
        Reālistiski: ar radaru vadīti ZSU-23-4 lielgabali (to radars jūs redz kā SAM radars) un S-60 zenītartilērijas baterijas sargā lidlaukus un mērķus; pilsētās MANPADS komandas šauj ar SA-14 uz to, ko redz
        Zems lidojums virs aizsargātām vietām kļūst bīstams; bumbas un lielgabals iznīcina ieročus

## Galvenais ekrāns

menu-title = STEALTH MISSION 89
menu-subtitle = MISIJU PLĀNOŠANAS SISTĒMA
menu-single-mission = VIENA MISIJA
    .facts =
        Viena misija jūsu izvēlētajā teātrī
        Izvēlieties opcijas, plānojiet maršrutu un bruņojumu
menu-career = KARJERA
    .facts =
        Pilotu dati: dienesta pakāpe, apbalvojumi, lidojumi
        Dienesta laiks: 12 lidojumi vienā teātrī
        Tas, ko iznīcināt, paliek iznīcināts
menu-campaign = DINAMISKĀ KAMPAŅA
    .facts =
        Dinamiska kampaņa reāllaikā
        Ieņemiet ienaidnieka pilsētas un lidlaukus
        Misijas no pašreizējās situācijas
menu-multiplayer = DAUDZSPĒLĒTĀJU REŽĪMS
    .facts = Kopīga spēle dinamiskajā kampaņā
    .join = Pievienoties serverim no publiskā saraksta
    .host = Pievienoties serverim vai izveidot kampaņu un lidot tajā
menu-options = OPCIJAS
    .facts =
        Spēle: kabīnes, HUD un avionikas stils, automātiskais trim
        Grafika, skaņas līmeņi, taustiņu un joystick piesaistes
        Saglabātas nākamajiem lidojumiem
    .short = Spēle, grafika, skaņa un vadība
menu-download = LEJUPIELĀDĒT
    .facts =
        Galddatora spēle operētājsistēmām Windows, Linux un macOS
        Ar VR un daudzspēlētāju režīmu
        Steam Deck un Steam Frame: Linux AppImage
web-play = SPĒLĒT PĀRLŪKĀ
    .facts = Tieši šeit, nekas nav jāinstalē
web-download-platforms = Windows, Linux un macOS, Steam Deck un Steam Frame
web-download-version = Lejupielādējamā versija
web-gives-vr = VR
    .why = Lidojiet VR ar jebkuru OpenXR saderīgu austiņu komplektu un Steam Frame
web-gives-hotas = HOTAS
    .why = Pilns atbalsts spēļu vadāmierīcēm, joystick, gāzes sviras un pedāļiem
web-gives-multiplayer = Labāks daudzspēlētāju režīms
    .why = Tiešs UDP savienojums ar serveriem mazākai aizkavei
web-gives-server = Izveidojiet savu serveri
    .why = Izveidojiet kopīgu kampaņu vai palaidiet dedicētu serveri
web-gives-head-tracking = Galvas izsekošana
    .why = TrackIR tipa sekotāji caur OpenTrack
web-gives-tacview = Tacview
    .why = Lidojumu ieraksti un telemetrija reāllaikā
web-gives-keep = Paliek jums
    .why = Ja vietne pazūd, jums paliek sava kopija
menu-discord = DISCORD
    .facts =
        Lidojiet kopā, ziņojiet par kļūdām un dalieties ar misijām
        Sekojiet izstrādei
menu-quit = IZIET
    .facts = Iziet uz darbvirsmu
menu-version = VERSIJA { $version }

## Vienas misijas ekrāns

menu-career-sortie = KARJERAS LIDOJUMS
menu-tour-sortie = { $pilot }  //  DIENESTA LAIKA LIDOJUMS { $sortie } NO { $length }
menu-msn = MISIJU PLĀNOŠANAS SISTĒMA  //  MSN { $seed }
menu-no-targets = Šim dienesta laikam vairs nav neviena nesagrauta mērķa
menu-plan-route = PLĀNOT MARŠRUTU
    .facts =
        Teātra karte ar ienaidnieka radaru un SAM pārklājumu
        Pievienojiet, pārvietojiet un dzēsiet navigācijas punktus
        Mērķi un nosēšanās bāze paliek nemainīgi
menu-loadout = BRUŅOJUMS
    .facts =
        Izvēlieties iekārtas katrā nodalījumā vai pilonā
        Sākas ar brīfinga ieročiem
menu-generating = ĢENERĒ...
menu-back-records = Atpakaļ uz pilotu datiem
menu-back-main = Atpakaļ uz galveno izvēlni
menu-new-sortie = JAUNS LIDOJUMS
menu-new-mission = JAUNA MISIJA
    .facts =
        Izvēlas citus mērķus
        Saglabā visas opcijas un starta laiku
menu-load = IELĀDĒT
    .facts =
        Lidojumā saglabātās misijas (izvēlne: SAGLABĀT MISIJU)
        Turpiniet lidojumu no saglabāšanas vietas
menu-fly = LIDOT
    .facts = Pacelties brīfingā noteiktajai misijai
menu-fly-vr = LIDOT VR
    .facts =
        Restartē spēli VR austiņās
        Lido tieši šo misiju
    .no-mission = Vispirms izveidojiet misiju
    .no-runtime = Nav aktīva OpenXR runtime (Monado, WiVRn, SteamVR)
menu-vr-failed = neizdevās startēt VR: { $error }

## Misijas kopsavilkums

summary-theater = TEĀTRIS
summary-situation = SITUĀCIJA
summary-mission = MISIJA
summary-opponents = PRETINIEKI
summary-time = LAIKS
summary-weather = LAIKAPSTĀKĻI
summary-fuel = DEGVIELA
summary-night = Nakts
summary-day = Diena
summary-time-value = { $daytime }  izlidošana { $depart }
summary-weather-value = { $sky }  redz. { $vis } km
summary-fuel-value = { $fuel } lb
summary-armament = Bruņojums: { $stores }
summary-intelligence = IZLŪKDATI
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
