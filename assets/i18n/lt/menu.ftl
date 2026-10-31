# Pagrindinis meniu ir pavienės misijos ekranas (src/menu.rs).

## Operacijų teatrai (stealth_host::theater::THEATERS)

theater-libya = Libija (1986)
    .facts =
        Sidros įlanka ir Libijos pakrantė
        Operacijos El Dorado Canyon laikai
        Dažniausiai giedras dykumos dangus
theater-persian_gulf = Persijos įlanka (1984)
    .facts =
        Irano ir Irako tanklaivių karo laikai
        Virš vandens ir pakrantės dykumos
        Giedriausias oras iš visų operacijų teatrų
theater-north_cape = Šiaurės kyšulys
    .facts =
        Arktinė Norvegija ir Kolos pusiasalis
        Sovietų Šiaurės laivyno namai
        Dažnas žemas debesuotumas
theater-central_europe = Vidurio Europa
    .facts =
        NATO prieš Varšuvos paktą virš Vokietijos
        Giedras dangus tik 15 % laiko
theater-vietnam = Vietnamas
    .facts =
        Hanojaus ir Haifongo sritis
        Džiunglių ir kalnų vietovė
        Perkūnijos 30 % misijų
theater-middle_east = Artieji Rytai
    .facts =
        Rytų Viduržemio jūra ir Levantas
        Dažniausiai giedras dangus
theater-desert_storm = Dykumos audra
    .facts =
        Kuveitas ir Irakas, 1991 m.
        Tikrasis F-117A kovinis debiutas
        Dažniausiai giedras dykumos dangus
theater-cuba = Kuba
    .facts =
        Karibų salos operacijų teatras
        Jūros ir sausumos taikiniai
        Tropikai: audros 15 % misijų
theater-korea = Korėja
    .facts =
        Korėjos pusiasalis į šiaurę nuo DMZ
        Kalnuota vietovė
        Kintantis oras, galimas bet kokis dangus
theater-ukraine = Ukraina (2026)
    .facts =
        Rytų Ukraina, Krymas ir pietų Rusija
        Fronto linija nuo Charkivo iki Chersono liepsnoja
        Atviri laukai, subombarduoti miestai prie fronto

## Situacijos (kovos veiksmų taisyklės)

roe-0 = Šaltasis karas
    .facts =
        Slapta taikos meto operacija
        Svarbiausia likti nepastebėtam
        Mažiau priešo orlaivių patruliuoja
        Tikslios kovos veiksmų taisyklės pateiktos instruktaže
roe-1 = Ribotas karas
    .facts =
        Regioninis ginkluotas konfliktas
        Priešo priešlėktuvinė gynyba budi
        Daugiau priešo orlaivių nei per Šaltąjį karą
roe-2 = Įprastinis karas
    .facts =
        Visaverčio masto karas
        Visos priešo pajėgos aktyvios
        Daugiau priešo orlaivių nei per Šaltąjį karą

## Misijų rūšys

kind-strike = Smūgis
    .facts =
        Sunaikinti, nufotografuoti arba pasiekti antžeminį taikinį
        Pagrindinis ir antrinis tikslas
        Oras-žemė ginkluotė
kind-air = Oras-oras
    .facts =
        Perimti priešo orlaivį
        Transportiniai lėktuvai, bombonešiai arba naikintuvų patruliai
        Oras-oras raketų ginkluotė
kind-strike-training = Smūgio treniruotė
    .facts =
        Treniruočių smūgio misija
        Išmokite ginklų ir navigacijos
kind-air-training = Oro kovos treniruotė
    .facts =
        Treniruočių oras-oras misija
        Išmokite perėmimų ir raketų
kind-drone-defense = Dronų gynyba
    .facts =
        Tik Ukrainoje
        Sustabdykite Geran atakos dronų bangą, kol ji nepasiekė miesto ar atominės elektrinės
        Paskui subombarduokite jų paleidimo vietą

## Priešininkai

opponents-0 = Naujokai
    .facts =
        Nepatyrusios priešo įgulos
        Lengviausias nustatymas
opponents-1 = Įprasti
    .facts =
        Vidutinės priešo įgulos
        Numatytasis nustatymas
opponents-2 = Veteranai
    .facts =
        Patyrusios priešo įgulos
        Daugiau laivų ir patruliavimų
opponents-3 = Elitas
    .facts =
        Geriausi priešo lakūnai ir SAM įgulos
        Sunkiausias nustatymas

## Oras

sky-random = Atsitiktinis
    .facts =
        Parenkamas pagal operacijų teatro klimatą
        Keičiasi su kiekviena nauja misija
        Debesys užstoja matomumą abiem pusėms
sky-clear = Giedras
    .facts =
        Be debesų
        Matomumas 40 km
        Klasikinis DOS eros dangus
sky-scattered = Išsklaidyti debesys
    .facts =
        Kamuoliniai debesys 20-45 %, pagrindas 2 500-6 000 ft
        Kartais aukštai plonas aukštutinis sluoksninis debesuotumas
        Matomumas 30 km
sky-broken = Plyšiuojantis debesuotumas
    .facts =
        Kamuoliniai debesys 45-70 %, pagrindas 2 000-5 000 ft
        Dažnai viršuje sluoksninių kamuolinių debesų sluoksnis
        Matomumas 25 km
sky-overcast = Apniukęs
    .facts =
        Sluoksniniai debesys 80-100 %, pagrindas 1 500-5 000 ft
        Žemiau keli kamuolinių debesų gabalai
        Matomumas 20 km
sky-storm = Audra
    .facts =
        Kupolinių lietaus debesų bokštai iki 40 000 ft
        Žemi sluoksniniai ir kamuoliniai debesys nuo 1 000 ft
        Matomumas 8 km
menu-random-cover = Atsitiktinis debesuotumas
    .facts =
        Pagrindinio debesų sluoksnio padengimas parenkamas kartu su oru
        Išjunkite, kad nustatytumėte patys
        Pagrindinis sluoksnis: sluoksniniai debesys apniukusiam dangui ir audrai, kitu atveju kamuoliniai
menu-cover-slider =
    .facts =
        Dangaus dalis, kurią dengia pagrindinis sluoksnis
        Giedram dangui įtakos neturi

## Mėnulis

moon-date = Misijos data
    .facts =
        Tikras Mėnulis instruktažo dieną
        Reikia paros laiko jungiklio (moderni)
moon-0 = Jaunatis
    .facts =
        Nėra mėnesienos: juoda naktis, tik šviesos ir žvaigždės
        Priešo lakūnai jus mato iš 2 km
moon-1 = Jaunas pjautuvas
    .facts = Plonas Mėnulis vakaro danguje, nusileidžia prieš vidurnaktį
moon-2 = Pirmasis ketvirtis
    .facts = Pusė apšviesta, vakare virš horizonto, nusileidžia apie vidurnaktį
moon-3 = Priaugantis Mėnulis
    .facts = Ryškus, virš horizonto nuo popietės iki po vidurnakčio
moon-4 = Pilnatis
    .facts =
        Visą naktį virš horizonto, vidurnaktį aukštai
        Priešo lakūnai jus mato iš 4 km
moon-5 = Slenkantis Mėnulis
    .facts = Ryškus, teka vakare, matomas iki ryto
moon-6 = Paskutinis ketvirtis
    .facts = Pusė apšviesta, teka apie vidurnaktį
moon-7 = Senas pjautuvas
    .facts = Plonas Mėnulis, tekantis prieš aušrą

## Orlaiviai

aircraft-f117 =
    .facts =
        Slaptas atakos orlaivis
        Požeminis greitis, 7 g
        Moderni ir klasikinė kabina
aircraft-f16 =
    .facts =
        Daugiafunkcis naikintuvas
        Mach 2 klasės, 9 g, greitas posvyris
        F-16 kabina su RWR
aircraft-tornado =
    .facts =
        Žemo aukščio smūgio orlaivis
        Kintamos geometrijos sparnai keičia kampą su greičiu
        Moderni kabina, trys MFD
aircraft-a10 =
    .facts =
        Artimosios oro paramos orlaivis
        Lėtas ir patvarus, 30 mm patranka
        A-10C kabina, du MFD
aircraft-av8b =
    .facts =
        V/STOL atakos lėktuvas
        Vektorinė trauka: kabojimas, nusileidžia bet kur
        Antgalių svirtis: R / F, Shift ribotuvams
aircraft-uh60 =
    .facts =
        Universalus sraigtasparnis
        Išlaipina kareivius: nusileiskite šalia taikinio
        Cyclic ir collective
aircraft-ah1z =
    .facts =
        Atakos sraigtasparnis
        Bokšteliu valdoma patranka, Hellfire, AIM-9
        Patranka ir Hellfire seka nacelę

## Skyriai ir grupės

section-theater = Operacijų teatras
section-opponents = Priešininkai
section-situation = Situacija
section-mission = Misija
section-moon = Mėnulis
section-start-time = Pradžios laikas
section-weather = Oras
section-realism = Realizmas
section-tacview = Tacview
section-mission-summary = Misijos santrauka
section-briefing = Instruktažas
group-mission = Misija
group-conditions = Sąlygos
group-vehicle = Transporto priemonė
group-simulation = Simuliacija

## Misijos nustatymai

menu-start-random = Atsitiktinis
    .facts =
        Parenkamas su kiekviena nauja misija, kaip klasikiniame režime
        06:00-08:00 dieną arba 18:00-20:00 naktį
        Išjunkite, kad pasirinktumėte valandą
menu-start-slider =
    .facts =
        Pakilimo laikas
        Naktis nuo 18:00 iki 06:00
        Išlaikomas paspaudus NAUJA MISIJA
menu-realistic-weapons = Realistiški ginklai
    .facts =
        Tikro orlaivio apribojimai
        2 ginklų skyriai vietoj 4
        Be AIM-9 ar AIM-120 raketų
        Be pakilimų nuo lėktuvnešio ir nusileidimų ant jo
menu-crash-protection = Apsauga nuo sudužimo
    .facts =
        Prie žemės automatiškai pakyla (važiuoklė įtraukta)
        Susidūrimas su žeme ir kitais objektais nėra mirtinas
        Išjungta: į žemę visada galima rėžtis
menu-tacview-export = Eksportas
    .facts =
        Išsaugo kiekvieną skrydį kaip Tacview įrašą (.acmi)
        Žaidimo failų aplanke, poaplankyje tacview
menu-tacview-real-time = Telemetrija realiuoju laiku
    .facts =
        Siunčia skrydį į Tacview, kol skrendate
        Tacview: Record > Real-Time Telemetry
        Adresas 127.0.0.1, prievadas 42674 (--tacview-address)
switch-game = ŽAIDIMAS
switch-real = REALUS
menu-damage = Pažeidimai
    .facts =
        Ginklų daroma žala
        Žaidimas: klasikinė žala; pataikymai kaupiasi, bet orlaivio nenumuša
        Realistiška: raketos pataikymas gali sunaikinti orlaivį (katapultuokitės!), pažeidžia daugiau sistemų ir dažnai sukelia gaisrą, kuris išdega arba susprogdina orlaivį
        Realistiška: pažeisti valdymo paviršiai išveda orlaivį iš trim arba priverčia jį svyruoti
menu-guidance = Pavedimas
    .facts =
        Raketų pavedimas
        Klasikinis: raketos skrenda tiesiai į taikinį
        Realistiškas: proporcinė navigacija į susidūrimo tašką; varikliai išdega, o raketos sklendžia toliau, su kiekvienu posūkiu prarasdamos greitį (dūmai tik kol dega variklis)
        Realistiškas: SAM raketos kyla aukštyn prieš tolimus taikinius; AIM-120 skrenda tiesiai 3 s (patraukite aukštyn, kad ją užmestumėte)
menu-air-defence = Priešlėktuvinė gynyba
    .facts =
        Žemo aukščio priešlėktuvinė gynyba
        Klasikinė: tik SAM objektai ir naikintuvai
        Realistiška: radaru vedamos ZSU-23-4 patrankos (jų radaras jus mato kaip SAM radaras) ir S-60 zenitinės baterijos saugo aerodromus ir taikinius; MANPADS komandos miestuose šaudo SA-14 į tai, ką mato
        Skraidyti žemai virš ginamų vietų tampa pavojinga; bombos ir patrankos sunaikina šias patrankas

## Pagrindinis ekranas

menu-title = STEALTH MISSION 89
menu-subtitle = MISIJŲ PLANAVIMO SISTEMA
menu-single-mission = PAVIENĖ MISIJA
    .facts =
        Viena misija jūsų pasirinktame operacijų teatre
        Pasirinkite nustatymus, suplanuokite maršrutą ir ginkluotę
menu-career = KARJERA
    .facts =
        Lakūno įrašai: laipsnis, apdovanojimai, skrydžiai
        Tarnybos laikotarpis: 12 skrydžių viename operacijų teatre
        Tai, ką sunaikinate, lieka sunaikinta
menu-campaign = DINAMINĖ KAMPANIJA
    .facts =
        Dinaminė kampanija realiuoju laiku
        Užimkite priešo miestus ir aerodromus
        Misijos pagal esamą situaciją
menu-multiplayer = DAUGIAVIETIS ŽAIDIMAS
    .facts = Coop dinaminėje kampanijoje
    .join = Prisijunkite prie serverio viešame sąraše
    .host = Prisijunkite prie serverio arba sukurkite kampaniją ir skraidykite joje
menu-options = NUSTATYMAI
    .facts =
        Žaidimas: kabinos, HUD ir avionikos stilius, autom. trim
        Grafika, garso lygiai, klavišų ir joystick priskyrimai
        Išsaugoma kitiems skrydžiams
    .short = Žaidimas, grafika, garsas ir valdymas
menu-download = ATSISIŲSTI
    .facts =
        Kompiuterinis žaidimas Windows, Linux ir macOS
        Su VR ir daugiavičiu žaidimu
        Steam Deck ir Steam Frame: Linux AppImage
web-play = ŽAISTI NARŠYKLĖJE
    .facts = Čia pat, nieko diegti nereikia
web-download-platforms = Windows, Linux ir macOS, Steam Deck ir Steam Frame
web-download-version = Atsisiunčiama versija
web-gives-vr = VR
    .why = Skraidykite VR su bet kokia OpenXR suderinama ausine ir Steam Frame
web-gives-hotas = HOTAS
    .why = Visiškas gamepad, joystick, throttle ir pedalų palaikymas
web-gives-multiplayer = Geresnis daugiavietis žaidimas
    .why = Tiesioginis UDP ryšys su serveriais, mažesnis vėlinimas
web-gives-server = Savas serveris
    .why = Sukurkite coop kampaniją arba paleiskite dedikuotą serverį
web-gives-head-tracking = Galvos sekimas
    .why = TrackIR tipo sekikliai per OpenTrack
web-gives-tacview = Tacview
    .why = Skrydžių įrašai ir telemetrija realiuoju laiku
web-gives-keep = Lieka jums
    .why = Jei svetainė išsijungtų, vis tiek turėsite savo kopiją
menu-discord = DISCORD
    .facts =
        Skraidykite kartu, praneškite apie klaidas ir dalinkitės misijomis
        Sekite kūrimo eigą
menu-quit = IŠEITI
    .facts = Išeiti į darbalaukį
menu-version = VERSIJA { $version }

## Pavienės misijos ekranas

menu-career-sortie = KARJEROS SKRYDIS
menu-tour-sortie = { $pilot }  //  TARNYBOS LAIKOTARPIO SKRYDIS { $sortie } IŠ { $length }
menu-msn = MISIJŲ PLANAVIMO SISTEMA  //  MSN { $seed }
menu-no-targets = Šiam tarnybos laikotarpiui neliko nesunaikintų taikinių
menu-plan-route = PLANUOTI MARŠRUTĄ
    .facts =
        Operacijų teatro žemėlapis su priešo radarų ir SAM aprėptimi
        Pridėkite, perkelkite ir pašalinkite navigacijos taškus
        Taikiniai ir nusileidimo bazė lieka vietoje
menu-loadout = GINKLUOTĖ
    .facts =
        Pasirinkite krovinį kiekviename skyriuje ar pylone
        Pradedama nuo instruktaže numatytų ginklų
menu-generating = KURIAMA...
menu-back-records = Atgal į lakūnų įrašus
menu-back-main = Atgal į pagrindinį meniu
menu-new-sortie = NAUJAS SKRYDIS
menu-new-mission = NAUJA MISIJA
    .facts =
        Sugeneruoja kitus tikslus
        Išlaiko visus nustatymus ir pradžios laiką
menu-load = ĮKELTI
    .facts =
        Skrydžio metu išsaugotos misijos (meniu: IŠSAUGOTI MISIJĄ)
        Skriskite toliau iš ten, kur išsaugojote
menu-fly = SKRISTI
    .facts = Pakilti vykdyti instruktaže numatytą misiją
menu-fly-vr = SKRISTI VR
    .facts =
        Iš naujo paleidžia žaidimą VR ausinėje
        Skrendama būtent ši misija
    .no-mission = Pirmiausia sugeneruokite misiją
    .no-runtime = Neaktyvi jokia OpenXR vykdymo aplinka (Monado, WiVRn, SteamVR)
menu-vr-failed = nepavyko paleisti VR: { $error }

## Misijos santrauka

summary-theater = OPERACIJŲ TEATRAS
summary-situation = SITUACIJA
summary-mission = MISIJA
summary-opponents = PRIEŠININKAI
summary-time = LAIKAS
summary-weather = ORAS
summary-fuel = KURAS
summary-night = Naktis
summary-day = Diena
summary-time-value = { $daytime }  pakilimas { $depart }
summary-weather-value = { $sky }  matom. { $vis } km
summary-fuel-value = { $fuel } lb
summary-armament = Ginkluotė: { $stores }
summary-intelligence = žvalgyba
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
