# Glavni izbornik i zaslon pojedinačne misije (src/menu.rs).

## Ratišta (stealth_host::theater::THEATERS)

theater-libya = Libija (1986)
    .facts =
        Zaljev Sidra i libijska obala
        Doba operacije El Dorado Canyon
        Većinom vedro pustinjsko nebo
theater-persian_gulf = Perzijski zaljev (1984)
    .facts =
        Doba iransko-iračkog tankerskog rata
        Iznad mora i obalne pustinje
        Najvedrije vrijeme od svih ratišta
theater-north_cape = Nordkapp
    .facts =
        Arktička Norveška i poluotok Kola
        Dom sovjetske Sjeverne flote
        Česta niska naoblaka
theater-central_europe = Srednja Europa
    .facts =
        NATO protiv Varšavskog pakta iznad Njemačke
        Vedro nebo samo 15 % vremena
theater-vietnam = Vijetnam
    .facts =
        Područje Hanoja i Haiphonga
        Džungla i planinski teren
        Oluje u 30 % misija
theater-middle_east = Bliski istok
    .facts =
        Istočni Mediteran i Levant
        Većinom vedro nebo
theater-desert_storm = Pustinjska oluja
    .facts =
        Kuvajt i Irak, 1991.
        Pravi borbeni debi F-117A
        Većinom vedro pustinjsko nebo
theater-cuba = Kuba
    .facts =
        Otočno ratište na Karibima
        Ciljevi na moru i kopnu
        Tropsko: oluje u 15 % misija
theater-korea = Koreja
    .facts =
        Korejski poluotok sjeverno od DMZ-a
        Planinski teren
        Promjenjivo vrijeme, moguće je svako nebo
theater-ukraine = Ukrajina (2026)
    .facts =
        Istočna Ukrajina, Krim i južna Rusija
        Bojišnica gori od Harkiva do Hersona
        Otvorena polja, razrušeni gradovi na fronti

## Situacije (pravila angažiranja)

roe-0 = Hladni rat
    .facts =
        Tajna operacija u doba mira
        Najvažnije je ostati neotkriven
        Manje neprijateljskih zrakoplova u patroli
        Točna pravila angažiranja nalaze se u briefingu
roe-1 = Ograničeni rat
    .facts =
        Regionalni oružani sukob
        Neprijateljska protuzračna obrana u pripravnosti
        Više neprijateljskih zrakoplova nego u hladnom ratu
roe-2 = Konvencionalni rat
    .facts =
        Rat punih razmjera
        Sve neprijateljske snage aktivne
        Više neprijateljskih zrakoplova nego u hladnom ratu

## Vrste misija

kind-strike = Napad
    .facts =
        Uništiti, fotografirati ili doseći cilj na tlu
        Glavni i sporedni cilj
        Naoružanje zrak-zemlja
kind-air = Zrak-zrak
    .facts =
        Presresti neprijateljski zrakoplov
        Transportni zrakoplovi, bombarderi ili lovačke patrole
        Naoružanje projektilima zrak-zrak
kind-strike-training = Napadna obuka
    .facts =
        Vježbovna napadna misija
        Naučite oružje i navigaciju
kind-air-training = Zračna obuka
    .facts =
        Vježbovna misija zrak-zrak
        Naučite presretanje i projektile
kind-drone-defense = Obrana od dronova
    .facts =
        Samo Ukrajina
        Zaustavite val napadačkih dronova Geran prije nego pogode grad ili nuklearnu elektranu
        Zatim bombardirajte njihovo lansirno mjesto

## Protivnici

opponents-0 = Novak
    .facts =
        Neiskusne neprijateljske posade
        Najlakša razina
opponents-1 = Redovan
    .facts =
        Prosječne neprijateljske posade
        Zadana razina
opponents-2 = Veteran
    .facts =
        Vješte neprijateljske posade
        Više brodova i patrola
opponents-3 = Elita
    .facts =
        Najbolji neprijateljski piloti i posluga SAM-ova
        Najteža razina

## Vrijeme

sky-random = Nasumično
    .facts =
        Izvučeno iz klime ratišta
        Mijenja se sa svakom novom misijom
        Oblaci blokiraju vidljivost objema stranama
sky-clear = Vedro
    .facts =
        Bez oblaka
        Vidljivost 40 km
        Klasično nebo iz DOS-ova doba
sky-scattered = Raštrkano
    .facts =
        Kumulus 20-45 %, baza 2500-6000 ft
        Katkad tanki altostratus visoko iznad
        Vidljivost 30 km
sky-broken = Isprekidano
    .facts =
        Kumulus 45-70 %, baza 2000-5000 ft
        Često sloj stratokumulusa iznad
        Vidljivost 25 km
sky-overcast = Oblačno
    .facts =
        Stratus 80-100 %, baza 1500-5000 ft
        Nekoliko komadića kumulusa ispod
        Vidljivost 20 km
sky-storm = Oluja
    .facts =
        Tornjevi kumulonimbusa do 40000 ft
        Nizak stratus i kumulus od 1000 ft
        Vidljivost 8 km
menu-random-cover = Nasumična naoblaka
    .facts =
        Pokrivenost glavnog sloja oblaka izvlači se s vremenom
        Isključite za ručno podešavanje
        Glavni sloj: stratus za oblačno i oluju, inače kumulus
menu-cover-slider =
    .facts =
        Dio neba koji pokriva glavni sloj
        Bez učinka pri vedrom nebu

## Mjesec

moon-date = Datum misije
    .facts =
        Stvarni Mjesec na datum briefinga
        Zahtijeva prekidač doba dana (moderno)
moon-0 = Mlađak
    .facts =
        Bez mjesečine: crna noć, samo svjetla i zvijezde
        Neprijateljski piloti vide vas na 2 km
moon-1 = Mlada mijena
    .facts = Tanak Mjesec na večernjem nebu, zalazi prije ponoći
moon-2 = Prva četvrt
    .facts = Pola osvijetljen, večer na nebu, zalazi oko ponoći
moon-3 = Rastući Mjesec
    .facts = Svijetao, na nebu od poslijepodneva do nakon ponoći
moon-4 = Uštap
    .facts =
        Na nebu cijelu noć, visoko u ponoć
        Neprijateljski piloti vide vas na 4 km
moon-5 = Padajući Mjesec
    .facts = Svijetao, izlazi navečer, na nebu do jutra
moon-6 = Posljednja četvrt
    .facts = Pola osvijetljen, izlazi oko ponoći
moon-7 = Stara mijena
    .facts = Tanak Mjesec koji izlazi prije zore

## Zrakoplovi

aircraft-f117 =
    .facts =
        Stealth jurišni zrakoplov
        Podzvučni, 7 g
        Moderni i klasični kokpit
aircraft-f16 =
    .facts =
        Višenamjenski lovac
        Klasa Mach 2, 9 g, brzo valjanje
        Kokpit F-16 s RWR-om
aircraft-tornado =
    .facts =
        Jurišni zrakoplov za niske visine
        Krila promjenjive geometrije prilagođavaju se brzini
        Moderni kokpit, tri MFD-a
aircraft-a10 =
    .facts =
        Zrakoplov za izravnu zračnu potporu
        Spor i otporan, top od 30 mm
        Kokpit A-10C, dva MFD-a
aircraft-av8b =
    .facts =
        Jurišni zrakoplov V/STOL
        Vektorski potisak: lebdenje, slijeće bilo gdje
        Ručica mlaznica: R / F, Shift za graničnike
aircraft-uh60 =
    .facts =
        Višenamjenski helikopter
        Iskrcava vojnike: sletite blizu cilja
        Ciklik i kolektiv
aircraft-ah1z =
    .facts =
        Jurišni helikopter
        Top u kupoli, Hellfire, AIM-9
        Top i Hellfire prate pod

## Odjeljci i skupine

section-theater = Ratište
section-opponents = Protivnici
section-situation = Situacija
section-mission = Misija
section-moon = Mjesec
section-start-time = Vrijeme polaska
section-weather = Vrijeme
section-realism = Realizam
section-tacview = Tacview
section-mission-summary = Sažetak misije
section-briefing = Briefing
group-mission = Misija
group-conditions = Uvjeti
group-vehicle = Letjelica
group-simulation = Simulacija

## Opcije misije

menu-start-random = Nasumično
    .facts =
        Izvlači se sa svakom novom misijom, kao u klasičnom načinu
        06:00-08:00 danju ili 18:00-20:00 noću
        Isključite za odabir sata
menu-start-slider =
    .facts =
        Vrijeme polijetanja
        Noć od 18:00 do 06:00
        Zadržava ga NOVA MISIJA
menu-realistic-weapons = Realistično naoružanje
    .facts =
        Ograničenja stvarnog zrakoplova
        2 odjeljka za naoružanje umjesto 4
        Bez projektila AIM-9 ili AIM-120
        Bez polijetanja i slijetanja na nosač zrakoplova
menu-crash-protection = Zaštita od sudara
    .facts =
        Automatski izvlači blizu tla (podvozje uvučeno)
        Dodir s tlom i sudari nisu smrtonosni
        Isključeno: u tlo se uvijek može udariti
menu-tacview-export = Izvoz
    .facts =
        Sprema svaki let kao Tacview snimku (.acmi)
        U mapi igre za vaše datoteke, pod tacview
menu-tacview-real-time = Telemetrija u stvarnom vremenu
    .facts =
        Šalje let u Tacview dok letite
        U Tacviewu: Record > Real-Time Telemetry
        Adresa 127.0.0.1, port 42674 (--tacview-address)
switch-game = IGRA
switch-real = STVARNO
menu-damage = Oštećenja
    .facts =
        Oštećenja od oružja
        Igra: klasična oštećenja; pogoci se zbrajaju, ali nikad ne obaraju zrakoplov
        Realistično: pogodak projektila može uništiti zrakoplov (katapultirajte se!), oštećuje više sustava i često izaziva požar koji se ugasi ili raznese zrakoplov
        Realistično: oštećene upravljačke površine odbacuju trim ili izazivaju njihanje zrakoplova
menu-guidance = Navođenje
    .facts =
        Navođenje projektila
        Klasično: projektili lete ravno na cilj
        Realistično: proporcionalna navigacija do točke susreta; motori izgore pa projektili jedre dalje gubeći brzinu u svakom zavoju (dim samo dok motor gori)
        Realistično: SAM-ovi se podižu prema udaljenim ciljevima; AIM-120 leti ravno 3 s (povucite palicu za lob)
menu-air-defence = Protuzračna obrana
    .facts =
        Protuzračna obrana na niskim visinama
        Klasično: samo SAM položaji i lovci
        Realistično: radarski navođeni topovi ZSU-23-4 (njihov radar vidi vas kao radar SAM-a) i flak baterije S-60 čuvaju zračne luke i ciljeve; MANPADS timovi u gradovima ispaljuju SA-14 na ono što vide
        Let nisko iznad branjenih mjesta postaje opasan; bombe i top uništavaju topove

## Glavni zaslon

menu-title = STEALTH MISSION 89
menu-subtitle = SUSTAV PLANIRANJA MISIJE
menu-single-mission = POJEDINAČNA MISIJA
    .facts =
        Jedna misija na ratištu po vašem izboru
        Odaberite opcije, isplanirajte rutu i naoružanje
menu-career = KARIJERA
    .facts =
        Dosjei pilota: čin, odličja, borbeni letovi
        Razdoblje službe: 12 borbenih letova na jednom ratištu
        Ono što uništite ostaje uništeno
menu-campaign = DINAMIČNA KAMPANJA
    .facts =
        Dinamična kampanja u stvarnom vremenu
        Zauzmite neprijateljske gradove i zračne luke
        Misije prema trenutnoj situaciji
menu-multiplayer = VIŠE IGRAČA
    .facts = Koop u dinamičnoj kampanji
    .join = Pridružite se poslužitelju s javnog popisa
    .host = Pridružite se poslužitelju ili ugostite kampanju i letite u njoj
menu-options = OPCIJE
    .facts =
        Igra: stil kokpita, HUD-a i avionike, automatski trim
        Grafika, glasnoća zvuka, tipke i joystick
        Spremljeno za sljedeće letove
    .short = Igra, grafika, zvuk i upravljanje
menu-download = PREUZMI
    .facts =
        Desktop igra za Windows, Linux i macOS
        S VR-om i igrom za više igrača
        Steam Deck i Steam Frame: Linux AppImage
web-play = IGRAJ U PREGLEDNIKU
    .facts = Odmah ovdje, bez instalacije
web-download-platforms = Windows, Linux i macOS, Steam Deck i Steam Frame
web-download-version = Verzija za preuzimanje
web-gives-vr = VR
    .why = Letite u VR-u s bilo kojim OpenXR kompatibilnim headsetom i Steam Frameom
web-gives-hotas = HOTAS
    .why = Potpuna podrška za gamepadove, joystickove, ručice gasa i pedale
web-gives-multiplayer = Bolja igra za više igrača
    .why = Izravna UDP veza s poslužiteljima za manji lag
web-gives-server = Ugostite vlastiti poslužitelj
    .why = Ugostite koop kampanju ili pokrenite namjenski poslužitelj
web-gives-head-tracking = Praćenje glave
    .why = Pratitelji tipa TrackIR putem OpenTracka
web-gives-tacview = Tacview
    .why = Snimke letova i telemetrija u stvarnom vremenu
web-gives-keep = Zauvijek vaše
    .why = Ako web-stranica nestane, i dalje imate vlastitu kopiju
menu-discord = DISCORD
    .facts =
        Letite zajedno, prijavite greške i dijelite misije
        Pratite razvoj
menu-quit = IZLAZ
    .facts = Izlaz na radnu površinu
menu-version = VERZIJA { $version }

## Zaslon pojedinačne misije

menu-career-sortie = KARIJERNI BORBENI LET
menu-tour-sortie = { $pilot }  //  BORBENI LET { $sortie } OD { $length } U RAZDOBLJU
menu-msn = SUSTAV PLANIRANJA MISIJE  //  MSN { $seed }
menu-no-targets = Za ovo razdoblje službe nije ostao nijedan cilj
menu-plan-route = PLANIRAJ RUTU
    .facts =
        Karta ratišta s pokrivenošću neprijateljskih radara i SAM-ova
        Dodajte, pomičite i uklanjajte navigacijske točke
        Ciljevi i baza slijetanja ostaju nepromijenjeni
menu-loadout = NAORUŽANJE
    .facts =
        Odaberite teret na svakom odjeljku ili nosaču
        Počinje s naoružanjem iz briefinga
menu-generating = GENERIRANJE...
menu-back-records = Natrag na dosjee pilota
menu-back-main = Natrag na glavni izbornik
menu-new-sortie = NOVI BORBENI LET
menu-new-mission = NOVA MISIJA
    .facts =
        Izvlači drukčije ciljeve
        Zadržava sve opcije i vrijeme polaska
menu-load = UČITAJ
    .facts =
        Misije spremljene tijekom leta (izbornik: SPREMI MISIJU)
        Nastavite let od mjesta spremanja
menu-fly = LETI
    .facts = Poletite na misiju iz briefinga
menu-fly-vr = LETI U VR-U
    .facts =
        Ponovno pokreće igru u VR headsetu
        Leti upravo ovu misiju
    .no-mission = Prvo generirajte misiju
    .no-runtime = Nijedan OpenXR runtime (Monado, WiVRn, SteamVR) nije aktivan
menu-vr-failed = pokretanje u VR-u nije uspjelo: { $error }

## Sažetak misije

summary-theater = RATIŠTE
summary-situation = SITUACIJA
summary-mission = MISIJA
summary-opponents = PROTIVNICI
summary-time = VRIJEME
summary-weather = METEO
summary-fuel = GORIVO
summary-night = Noć
summary-day = Dan
summary-time-value = { $daytime }  polazak { $depart }
summary-weather-value = { $sky }  vid. { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Naoružanje: { $stores }
summary-intelligence = OBAVJEŠTAJNI PODACI
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
