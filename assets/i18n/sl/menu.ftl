# Glavni meni in zaslon posamezne misije (src/menu.rs).

## Bojišča (stealth_host::theater::THEATERS)

theater-libya = Libija (1986)
    .facts =
        Zaliv Sidra in libijska obala
        Doba operacije El Dorado Canyon
        Večino dni jasno puščavsko nebo
theater-persian_gulf = Perzijski zaliv (1984)
    .facts =
        Doba tankerske vojne med Iranom in Irakom
        Nad morjem in obalno puščavo
        Najjasnejše vreme med vsemi bojišči
theater-north_cape = Severni rt
    .facts =
        Arktična Norveška in polotok Kola
        Dom sovjetske severne flote
        Pogosta nizka oblačnost
theater-central_europe = Srednja Evropa
    .facts =
        Nato proti Varšavskemu paktu nad Nemčijo
        Jasno nebo le v 15 % primerov
theater-vietnam = Vietnam
    .facts =
        Območje Hanoja in Haiphonga
        Džungla in gorat teren
        Nevihte v 30 % misij
theater-middle_east = Bližnji vzhod
    .facts =
        Vzhodno Sredozemlje in Levanta
        Večinoma jasno nebo
theater-desert_storm = Puščavska nevihta
    .facts =
        Kuvajt in Irak, 1991
        Pravi bojni krst letala F-117A
        Večinoma jasno puščavsko nebo
theater-cuba = Kuba
    .facts =
        Karibsko otoško bojišče
        Pomorski in kopenski cilji
        Tropsko: nevihte v 15 % misij
theater-korea = Koreja
    .facts =
        Korejski polotok severno od DMZ
        Gorat teren
        Spremenljivo vreme, mogoče je vsako nebo
theater-ukraine = Ukrajina (2026)
    .facts =
        Vzhodna Ukrajina, Krim in južna Rusija
        Frontna črta od Harkova do Hersona gori
        Odprta kmetijska zemljišča, porušena mesta ob fronti

## Razmere (pravila spopada)

roe-0 = Hladna vojna
    .facts =
        Prikrita operacija v miru
        Najpomembneje je ostati neodkrit
        Manj sovražnih letal na patrulji
        Natančna pravila spopada so v briefingu
roe-1 = Omejena vojna
    .facts =
        Regionalni oborožen spopad
        Sovražna protizračna obramba je v pripravljenosti
        Več sovražnih letal kot v hladni vojni
roe-2 = Konvencionalna vojna
    .facts =
        Vojna polnih razsežnosti
        Vse sovražne sile so aktivne
        Več sovražnih letal kot v hladni vojni

## Vrste misij

kind-strike = Napad
    .facts =
        Uničite, fotografirajte ali dosezite kopenski cilj
        Glavni in sekundarni cilj
        Oborožitev zrak-zemlja
kind-air = Zrak-zrak
    .facts =
        Prestrezite sovražno letalo
        Transporterji, bombniki ali lovske patrulje
        Oborožitev z izstrelki zrak-zrak
kind-strike-training = Vadba napada
    .facts =
        Vadbena misija napada
        Spoznajte orožje in navigacijo
kind-air-training = Vadba zrak-zrak
    .facts =
        Vadbena misija zrak-zrak
        Spoznajte prestrezanje in izstrelke
kind-drone-defense = Obramba pred droni
    .facts =
        Samo Ukrajina
        Ustavite val napadalnih dronov Geran, preden zadane mesto ali jedrsko elektrarno
        Nato bombardirajte njihovo izstrelišče

## Nasprotniki

opponents-0 = Novinci
    .facts =
        Neizkušene sovražne posadke
        Najlažja stopnja
opponents-1 = Redni
    .facts =
        Povprečne sovražne posadke
        Privzeta stopnja
opponents-2 = Veterani
    .facts =
        Izurjene sovražne posadke
        Več ladij in patrulj
opponents-3 = Elita
    .facts =
        Najboljši sovražni piloti in posadke SAM
        Najtežja stopnja

## Vreme

sky-random = Naključno
    .facts =
        Izžrebano glede na podnebje bojišča
        Spremeni se z vsako novo misijo
        Oblaki zastirajo vidno linijo obema stranema
sky-clear = Jasno
    .facts =
        Brez oblakov
        Vidljivost 40 km
        Klasično nebo dobe DOS
sky-scattered = Razpršena oblačnost
    .facts =
        Kopasti oblaki 20-45 %, spodnja meja 2500-6000 ft
        Včasih tanka višinska plast visoko zgoraj
        Vidljivost 30 km
sky-broken = Razdrobljena oblačnost
    .facts =
        Kopasti oblaki 45-70 %, spodnja meja 2000-5000 ft
        Pogosto plast kopasto-slojastih oblakov nad njimi
        Vidljivost 25 km
sky-overcast = Oblačno
    .facts =
        Slojasti oblaki 80-100 %, spodnja meja 1500-5000 ft
        Nekaj kopastih koscev pod njimi
        Vidljivost 20 km
sky-storm = Nevihta
    .facts =
        Nevihtni stolpi do 40000 ft
        Nizki slojasti in kopasti oblaki od 1000 ft
        Vidljivost 8 km
menu-random-cover = Naključna pokritost
    .facts =
        Pokritost glavne plasti oblakov se izžreba z vremenom
        Odkljukajte, da jo nastavite sami
        Glavna plast: slojasta za oblačno in nevihto, sicer kopasta
menu-cover-slider =
    .facts =
        Delež neba, ki ga pokriva glavna plast
        Brez učinka pri jasnem nebu

## Luna

moon-date = Datum misije
    .facts =
        Prava luna na datum briefinga
        Zahteva stikalo časa dneva (sodobno)
moon-0 = Mlaj
    .facts =
        Brez mesečine: črna noč, samo luči in zvezde
        Sovražni piloti vas vidijo na 2 km
moon-1 = Naraščajoči srp
    .facts = Tanka luna na večernem nebu, zaide pred polnočjo
moon-2 = Prvi krajec
    .facts = Pol osvetljena, zvečer na nebu, zaide okoli polnoči
moon-3 = Naraščajoča luna
    .facts = Svetla, na nebu od popoldneva do po polnoči
moon-4 = Polna luna
    .facts =
        Vso noč na nebu, opolnoči visoko
        Sovražni piloti vas vidijo na 4 km
moon-5 = Padajoča luna
    .facts = Svetla, vzide zvečer, na nebu do jutra
moon-6 = Zadnji krajec
    .facts = Pol osvetljena, vzide okoli polnoči
moon-7 = Padajoči srp
    .facts = Tanka luna, ki vzide pred zoro

## Letala

aircraft-f117 =
    .facts =
        Prikrito jurišno letalo
        Podzvočno, 7 g
        Sodobna in klasična kabina
aircraft-f16 =
    .facts =
        Večnamenski lovec
        Razred Mach 2, 9 g, hitro kotaljenje
        Kabina F-16 z RWR
aircraft-tornado =
    .facts =
        Jurišno letalo za nizko letenje
        Krila se z naraščajočo hitrostjo pomikajo nazaj
        Sodobna kabina, trije MFD
aircraft-a10 =
    .facts =
        Letalo za neposredno podporo kopenskim enotam
        Počasno in robustno, top 30 mm
        Kabina A-10C, dva MFD
aircraft-uh60 =
    .facts =
        Večnamenski helikopter
        Prevaža vojake: pristanite blizu cilja
        Cyclic in collective
aircraft-ah1z =
    .facts =
        Bojni helikopter
        Topovska kupola, Hellfire, AIM-9
        Top in Hellfire sledita podu

## Razdelki in skupine

section-theater = Bojišče
section-opponents = Nasprotniki
section-situation = Razmere
section-mission = Misija
section-moon = Luna
section-start-time = Čas začetka
section-weather = Vreme
section-realism = Realizem
section-tacview = Tacview
section-mission-summary = Povzetek misije
section-briefing = Briefing
group-mission = Misija
group-conditions = Pogoji
group-vehicle = Vozilo
group-simulation = Simulacija

## Možnosti misije

menu-start-random = Naključno
    .facts =
        Izžrebano z vsako novo misijo, kot v klasičnem načinu
        06:00-08:00 podnevi ali 18:00-20:00 ponoči
        Odkljukajte, da izberete uro
menu-start-slider =
    .facts =
        Ura vzleta
        Noč od 18:00 do 06:00
        Ohrani jo NOVA MISIJA
menu-realistic-weapons = Realistično orožje
    .facts =
        Omejitve pravega letala
        2 oborožitvena prostora namesto 4
        Brez izstrelkov AIM-9 ali AIM-120
        Brez vzletov in pristankov na letalonosilki
menu-crash-protection = Zaščita pred strmoglavljenjem
    .facts =
        Samodejno izravna blizu tal (podvozje uvlečeno)
        Stik s tlemi in trki niso usodni
        Izklopljeno: v tla se je vedno mogoče zabiti
menu-tacview-export = Izvoz
    .facts =
        Vsak let shrani kot posnetek Tacview (.acmi)
        V mapi igre za vaše datoteke, pod tacview
menu-tacview-real-time = Telemetrija v realnem času
    .facts =
        Med letenjem pošilja let v Tacview
        V Tacview: Record > Real-Time Telemetry
        Naslov 127.0.0.1, vrata 42674 (--tacview-address)
switch-game = IGRA
switch-real = REALNO
menu-damage = Poškodbe
    .facts =
        Poškodbe od orožja
        Igra: klasične poškodbe; zadetki se seštevajo, a letala nikoli ne sestrelijo
        Realistično: zadetek izstrelka lahko uniči letalo (katapultirajte se!), poškoduje več sistemov in pogosto zaneti požar, ki ugasne ali letalo raznese
        Realistično: poškodovane krmilne ploskve vržejo letalo iz trima ali ga povzročijo nihati
menu-guidance = Vodenje
    .facts =
        Vodenje izstrelkov
        Klasično: izstrelki letijo naravnost proti cilju
        Realistično: proporcionalna navigacija do točke trka; motorji pogorijo in izstrelki jadrajo naprej ter z vsakim zavojem izgubljajo hitrost (dim samo, dokler motor gori)
        Realistično: SAM se proti oddaljenim ciljem dvignejo v lok; AIM-120 3 s leti naravnost (potegnite navzgor, da ga vržete v lok)
menu-air-defence = Protizračna obramba
    .facts =
        Protizračna obramba na nizkih višinah
        Klasično: samo položaji SAM in lovci
        Realistično: radarsko vodeni topovi ZSU-23-4 (njihov radar vas vidi kot radar SAM) in protiletalske baterije S-60 varujejo letališča in cilje; ekipe MANPADS v mestih izstreljujejo SA-14 na vse, kar vidijo
        Nizko letenje nad branjenimi kraji postane nevarno; bombe in topovi uničijo topove

## Glavni zaslon

menu-title = STEALTH MISSION 89
menu-subtitle = SISTEM ZA NAČRTOVANJE MISIJ
menu-single-mission = POSAMEZNA MISIJA
    .facts =
        Ena misija na bojišču po vaši izbiri
        Izberite možnosti, načrtujte pot in oborožitev
menu-career = KARIERA
    .facts =
        Dosjeji pilotov: čin, odlikovanja, bojni leti
        Doba službovanja: 12 bojnih letov na enem bojišču
        Kar uničite, ostane uničeno
menu-campaign = DINAMIČNA KAMPANJA
    .facts =
        Dinamična kampanja v realnem času
        Zavzemite sovražnikova mesta in letališča
        Misije iz žive situacije
menu-multiplayer = VEČIGRALSKI NAČIN
    .facts = Sodelovanje v dinamični kampanji
    .join = Pridružite se strežniku na javnem seznamu
    .host = Pridružite se strežniku ali gostite kampanjo in letite v njej
menu-options = MOŽNOSTI
    .facts =
        Igra: slog kabine, HUD in avionike, samodejni trim
        Grafika, glasnosti zvoka, dodelitve tipk in joysticka
        Shranjeno za naslednje lete
    .short = Igra, grafika, zvok in krmiljenje
menu-download = PRENOS
    .facts =
        Namizna igra za Windows, Linux in macOS
        Z VR in večigralskim načinom
        Steam Deck in Steam Frame: Linux AppImage
web-play = IGRAJ V BRSKALNIKU
    .facts = Kar tukaj, brez namestitve
web-download-platforms = Windows, Linux in macOS, Steam Deck in Steam Frame
web-download-version = Različica za prenos
web-gives-vr = VR
    .why = Letite v VR z vsemi očali, združljivimi z OpenXR, in s Steam Frame
web-gives-hotas = HOTAS
    .why = Polna podpora za igralne ploščke, joysticke, ročice plina in pedale
web-gives-multiplayer = Boljši večigralski način
    .why = Neposredna povezava UDP s strežniki za manj zakasnitve
web-gives-server = Gostite svoj strežnik
    .why = Gostite kampanjo za sodelovanje ali zaženite namenski strežnik
web-gives-head-tracking = Sledenje glavi
    .why = Sledilniki v slogu TrackIR prek OpenTrack
web-gives-tacview = Tacview
    .why = Posnetki letov in telemetrija v realnem času
web-gives-keep = Vaše za vedno
    .why = Če spletno mesto preneha delovati, imate še vedno svojo kopijo
menu-discord = DISCORD
    .facts =
        Letite skupaj, prijavite napake in delite misije
        Spremljajte razvoj
menu-quit = IZHOD
    .facts = Izhod na namizje
menu-version = RAZLIČICA { $version }

## Zaslon posamezne misije

menu-career-sortie = BOJNI LET KARIERE
menu-tour-sortie = { $pilot }  //  BOJNI LET { $sortie } OD { $length } V DOBI SLUŽBOVANJA
menu-msn = SISTEM ZA NAČRTOVANJE MISIJ  //  MSN { $seed }
menu-no-targets = V tej dobi službovanja ni ostalo nobenega cilja
menu-plan-route = NAČRTUJ POT
    .facts =
        Zemljevid bojišča s sovražnim radarskim in SAM pokritjem
        Dodajte, premaknite in odstranite navigacijske točke
        Cilji in pristajalna baza ostanejo nespremenjeni
menu-loadout = OBOROŽITEV
    .facts =
        Izberite oborožitev na vsakem prostoru ali pilonu
        Začne z orožjem iz briefinga
menu-generating = USTVARJANJE...
menu-back-records = Nazaj na dosjeje pilotov
menu-back-main = Nazaj v glavni meni
menu-new-sortie = NOV BOJNI LET
menu-new-mission = NOVA MISIJA
    .facts =
        Izžreba drugačne cilje
        Ohrani vse možnosti in čas začetka
menu-load = NALOŽI
    .facts =
        Misije, shranjene med letom (meni: SHRANI MISIJO)
        Letite naprej od mesta shranjevanja
menu-fly = LETI
    .facts = Vzletite na misijo iz briefinga
menu-fly-vr = LETI V VR
    .facts =
        Ponovno zažene igro v očalih VR
        Leti natanko to misijo
    .no-mission = Najprej ustvarite misijo
    .no-runtime = Noben izvajalnik OpenXR (Monado, WiVRn, SteamVR) ni aktiven
menu-vr-failed = zagon v VR ni uspel: { $error }

## Povzetek misije

summary-theater = BOJIŠČE
summary-situation = RAZMERE
summary-mission = MISIJA
summary-opponents = NASPROTNIKI
summary-time = ČAS
summary-weather = VREME
summary-fuel = GORIVO
summary-night = Noč
summary-day = Dan
summary-time-value = { $daytime }  odhod { $depart }
summary-weather-value = { $sky }  vid. { $vis } km
summary-fuel-value = { $fuel } lb
summary-armament = Oborožitev: { $stores }
summary-intelligence = OBVEŠČEVALNI PODATKI
