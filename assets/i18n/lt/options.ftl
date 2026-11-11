# NUSTATYMŲ langas (src/options.rs) ir jo puslapiai meniu skrydžio metu.

options-title = NUSTATYMAI
options-tab-game = ŽAIDIMAS
options-tab-graphics = GRAFIKA
options-tab-sound = GARSAS
options-tab-controls = VALDYMAS
options-footer = Išsaugoma kitiems skrydžiams; realizmo jungikliai pasirenkami kiekvienam žaidimui

## ŽAIDIMAS

options-section-language = Kalba
options-section-classic-modern = Klasikinis / moderni
options-cockpit = Kabina
    .facts =
        Moderni: didelės raiškos ekranai su apvadų mygtukais
        Klasikinė: DOS eros prietaisų skydelio išdėstymas ir 77x60 ekranai (F-117A)
options-hud = HUD
    .facts =
        Moderni: didelės raiškos F-16 stiliaus žalias HUD su NAV, AA ir AG režimais
        Klasikinė: DOS eros HUD
options-avionics = Avionika
    .facts =
        Moderni: F-16 stiliaus MFD formatai bet kuriame MFD (situacijos ekranas, HARM, žymėjimo konteineris, žemėlapis, krovinys) su slenkamu žymekliu
        Klasikinė: žemėlapis ir TAC kairėje, kamera, WAY, WPN ir ORD dešinėje
options-flight-model = Skrydžio modelis
    .facts =
        Moderni: aerodinaminis skrydžio modelis su fly-by-wire: stick nurodo posvyrio ir polėkio greitį; paleidus nosis lieka vietoje ir išlaikoma skrydžio trajektorija
        Klasikinė: DOS eros skrydžio modelis
options-section-flying = Skrydis
options-auto-trim = Autom. trim
    .facts =
        Paleidus stick, trim išlaiko padėtį, kurioje ją palikote
        Sraigtasparniai: nosis nebekyla didėjant greičiui; reaktyviniai: kompensuoja pažeistus valdymo paviršius
        Rankinis trim: Alt + rodyklių klavišai, Alt + Home centruoja
options-keep-time-accel = Išlaikyti laiko spartinimą
    .facts =
        Spartinimas nebeišsijungia, kai jus aptinka, paleidžiate ginklą ar numuša
        Jį keičia tik mygtukas TIME ir Shift + Z / Shift + X
options-tooltips = Kabinos paaiškinimai
    .facts = Kas yra kabinos valdiklis ar ekranas, po pele arba VR piršto galiuku
options-player-labels = Žaidėjų vardai
    .facts = Daugelio žaidėjų režimas: kitų pilotų vardai virš jų lėktuvų, bet kokiu atstumu
options-label-size =
    .facts =
        Žaidėjų vardų dydis
        50 %: numatytasis
options-seat-height = Sėdynės aukštis
    .facts =
        Kaip aukštai sėdite kabinoje, ekrane ir VR
        0 cm: įprastas orlaivio akių aukštis
options-section-vr-size = VR kabinos dydis
options-vr-size =
    .facts =
        Kabinos (ir pasaulio) dydis ausinėje
        100 %: natūralus dydis
options-section-gif = GIF įrašymas
options-gif-size = DYDIS
options-gif-rate = KADRAI
options-gif-scale-100 =
    .facts = Visa lango raiška: dideli failai
options-gif-scale-25 =
    .facts = Maži failai
options-gif-rate-50 =
    .facts = Tik kai žaidimas veikia 100 fps ar daugiau (įrašomas kas antras kadras)
options-gif-hint = F12 (jei nepriskirta kitaip) pradeda ir sustabdo; įrašoma į aplanką Paveikslėliai

## GRAFIKA

options-section-rendering = Vaizdavimas
options-shaders = Šeideriai
    .facts =
        Moderni: subtilus kryptinis šešėliavimas, švelnus atstumo rūkas
        Klasikinė: plokščias šešėliavimas
options-shadows = Šešėliai
    .facts =
        Moderni: saulės šešėliai kabinoje ir pasaulyje (naudoja GPU laiką)
        Klasikinė: be šešėlių
options-point-lights = Taškiniai šviestuvai
    .facts =
        Moderni: kabinos ir ekranų apšvietimas; sprogimai, dūmų kamienai ir pakilimo tako žibintai apšviečia sceną
        Klasikinė: tik saulės ir dangaus šviesa
options-time-of-day = Paros laikas
    .facts =
        Moderni: saulė ir Mėnulis juda su misijos laikrodžiu, dangus tamsėja ir šviesėja su saule; naktį žvaigždės ir Mėnulio fazė
        Moderni: mėnesiena nustato, iš kokio atstumo priešo lakūnai jus mato naktį (2 km bemėnulę naktį, 4 km per pilnatį)
        Klasikinė: lėtas spalvų kitimas ir nejudanti saulė
options-msaa = Glotninimas (MSAA)
    .facts =
        Glotnūs kraštai: 4x MSAA, 2x VR (naudoja GPU laiką)
        Išjungta: aštrūs pikselių kraštai
options-auto-detail = Automatinis aplinkos detalumas (VR)
    .facts =
        Ausinėje miestų, miškų ir laukų matomumo atstumas mažėja, kai kadrai vėluoja, ir didėja, kai yra laiko atsargos
        Grafikos nustatymai lieka tokie, kokius pasirinkote
options-sun-moon = Saulė ir Mėnulis
    .facts =
        Saulė danguje dieną, Mėnulis ir žvaigždės naktį
        Mėnulis rodo misijos fazę
options-cloud-quality = Debesys
cloud-quality-auto = Auto
    .facts = Žema VR, aukšta ekrane
cloud-quality-low = Žema
    .facts = Šiurkštūs debesys trečdaliu raiškos (greičiausia)
cloud-quality-medium = Vidutinė
    .facts = Pusė raiškos, smulkesnės detalės
cloud-quality-high = Aukšta
    .facts = Visa raiška, smulkiausios detalės ir savas šešėlis (naudoja GPU laiką)

## GARSAS

options-section-volume = Garsas
options-volume-master = Bendras
    .facts = Visi garsai
options-volume-engine = Variklis
    .facts = Jūsų variklis ar rotorius ir aplink girdimi orlaiviai
options-volume-effects = Efektai
    .facts = Ginklai, sprogimai, važiuoklė, radijas ir kita
options-volume-warnings = Įspėjimai
    .facts = Praradimo signalas, radaro ir raketų įspėjimai, gaisro sirena, Sidewinder tonai
options-volume-rwr = RWR
    .facts = Radaro įspėjimo tonai, kaip įspėjimų garsumo dalis
options-section-rwr-tones = Radaro įspėjimo tonai
rwr-tones-all = VISI
    .facts = Jus apšviečiantis radaras, kiekvienas jus aptinkantis apžvalgos ciklas, raketų paleidimai
rwr-tones-detect = TIK APTIKIMAS
    .facts =
        Vienas tonas, kai radaras jus aptinka (vėl, kai radarai jus kuriam laikui paleidžia)
        Ir raketų paleidimai
rwr-tones-launch = TIK PALEIDIMAI
    .facts = Tik į jus paleistos raketos
rwr-tones-off = IŠJ
    .facts = Jokių radaro įspėjimo tonų (lemputės ir ekranai vis tiek įspėja)
options-section-cockpit = Kabina
options-low-pass = Variklio žemų dažnių filtras
    .facts =
        Orlaivio viduje variklis skamba duslinamai (gaubtas ir šalmas pašalina aukštus dažnius)
        Išjungta: kaip lauke

## VALDYMAS

options-section-bindings = Priskyrimai
options-keys-joystick = KLAVIŠAI IR JOYSTICK
    .facts =
        Žemėlapio klavišai, joystick ir HOTAS ašys bei mygtukai
        Išsaugoma kitiems skrydžiams
options-section-head-tracking = Galvos / akių sekimas
options-head-tracking = Galvos / akių sekimas
    .facts =
        Jūsų galva sukioja ir juda vaizdą kabinoje (kompiuteryje; VR tai daro ausinė)
        Sekimo programa (OpenTrack...): FreeTrack išvestis arba UDP į 127.0.0.1 prievadą 4242
        Home: dabartinė galvos padėtis tampa centru
options-tracking = Sekimas ({ $source })
options-no-tracker = Sekiklis negirdimas: paleiskite sekimo programą

## EFFECTS

options-section-effects = Efektai
options-section-retro = Retro
effect-shake = Kameros drebėjimas
    .facts =
        Vaizdas dreba nuo patrankos, netoli esančių sprogimų, srauto nutrūkimo, nusileidimų ir tako
        VR: kabina vibruoja, galva lieka nejudri
effect-flashes = Mūšio blyksniai
    .facts = Zenitinių sviedinių sprogimai, sprogimai ir raketų paleidimai akimirkai apšviečia debesis, žemę ir kabiną
effect-barrage = Naktinė užtvarinė ugnis
    .facts =
        Įprastinis karas naktį: miestų patrankos aklai šaudo sekamaisiais sviediniais į dangų, prožektoriai juos šluoja
        Priešo dronus virš miesto sučiumpa jo prožektoriai, ir jie pritraukia jo patrankų ugnį
        Tik dėl vaizdo: užtvara nieko nepataiko
effect-cockpit-night = Kabinos naktinis apšvietimas
    .facts =
        Blausi raudona šviesa kabinoje sutemus (melsvai žalia pro akinius)
        Master caution pulsuoja gintaro spalva virš skydelio, kol įspėjimas aktyvus
effect-nvg = Naktinio matymo akiniai
    .facts =
        Ryškios šviesos išsiplečia į aureoles, vaizdas tamsoje grūdėtesnis
        Kompiuteris: akinių apvalus matymo laukas
effect-snow = Sniegas
    .facts = Virš užšalimo ribos audros atneša sniegą vietoj lietaus: žemai šiaurės žiemą, aukštai vasarą
effect-mist = Žemės rūkas
    .facts =
        Šiuolaikiniai šešėliuokliai: rūkas dengia žemumas ir jūrą aušroje, prieblandoje ir naktį
        Retas dykumose, jokio per audras
effect-cloud-shadows = Debesų šešėliai
    .facts = Šiuolaikiniai šešėliuokliai: debesų šešėliai žemėje apačioje
effect-vapour = Garai ir rotoriaus srautas
    .facts =
        Staigiai manevruojant nuo sparnų galų driekiasi garų pėdsakai
        Žemai kabantis sraigtasparnis kelia dulkes, virš vandens - purslus
        Važiuojančios mašinos kelia dulkes už savęs
effect-night-sky = Naktinis dangus
    .facts =
        Retkarčiais krintančios žvaigždės, žemos žvaigždės virš horizonto mirga
        Šiaurės pašvaistė virš Nordkapo
effect-vga = VGA spalvos
    .facts = 1989 m. išvaizda su šiuolaikiniu šešėliavimu: 64 pakopos vienam spalvų kanalui kaip VGA plokštėje, su dithering
effect-scanlines = Skenavimo linijos
    .facts = CRT monitoriaus skenavimo linijos (ne VR)

## LANGUAGE NAMES

lang-name-en = anglų
lang-name-bg = bulgarų
lang-name-zh-Hans = kinų (supaprastinta)
lang-name-hr = kroatų
lang-name-cs = čekų
lang-name-da = danų
lang-name-nl = olandų
lang-name-et = estų
lang-name-fi = suomių
lang-name-fr = prancūzų
lang-name-de = vokiečių
lang-name-el = graikų
lang-name-hu = vengrų
lang-name-id = indoneziečių
lang-name-ga = airių
lang-name-it = italų
lang-name-ja = japonų
lang-name-ko = korėjiečių
lang-name-lv = latvių
lang-name-lt = lietuvių
lang-name-mt = maltiečių
lang-name-pl = lenkų
lang-name-pt-BR = portugalų (Brazilija)
lang-name-ro = rumunų
lang-name-ru = rusų
lang-name-sk = slovakų
lang-name-sl = slovėnų
lang-name-es = ispanų
lang-name-sv = švedų
lang-name-uk = ukrainiečių

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Naršyklės kalba: { $language }. Ar žaisti šia kalba?
language-offer-yes = Taip
language-offer-no = Ne
language-offer-never = Daugiau nebeklausti
effect-sunbeams = Saulės spinduliai
    .facts = Saulės šviesos pluoštai pro suplyšusio debesuotumo properšas
options-msaa-forced = Išjungta, kol pikselizacija (retro) įjungta
options-volume-radio = Radijas
    .facts = Kiti kooperacinio radijo žaidėjai ir kalbamasis ATC
options-volume-atc = ATC balsas
    .facts = Kalbamieji ATC iškvietimai, kaip radijo garsumo dalis
options-volume-music = Muzika
    .facts = Pagrindinio meniu muzika
options-section-atc = Kalbamasis ATC
options-section-radio = Kooperacinis radijas
options-push-to-talk = Spausk ir kalbėk
    .facts =
        Laikykite nuspaudę kalbėjimo klavišą (Caps Lock arba PTT priskirtą joysticko ar VR mygtuką) ir visi žaidėjai jus girdi
        Mikrofonas atsidaro pirmą kartą paspaudus
options-mic-gain = MIKROFONO STIPRINIMAS
    .facts =
        Kaip garsiai eina jūsų mikrofonas
        100 %: kaip yra
options-mic-default = Numatytasis
options-voice-text-only = tik tekstas
options-voices-line = Skrydžių vadovai: { $controllers }   Pilotas: { $pilot }
options-voices = BALSAI
    .facts =
        Atsisiųsti balsus radijo iškvietimams, juos ištrinti
        Pasirinkti skrydžių vadovų balsus ir savo, ir juos paklausyti
options-voices-web = Naršyklėje tik tekstas
options-tracking-mouse = Žvalgymasis pele sekant
    .facts =
        Įjungta: dešinysis pelės mygtukas ir toliau sukioja bei artina kabinos vaizdą
        Išjungta: kol sekiklis siunčia, dešinysis pelės mygtukas kabinoje nieko nedaro
options-atc = Oro eismo valdymas
    .facts =
        Bokšto, artėjimo ir GCA radijo iškvietimai draugiškuose aerodromuose
        Leidimas kilti, overhead break ir talkdown esant žemiems debesims ar blogam matomumui
options-pixel-block = Pikselizacijos bloko dydis
    .facts = Bloko kraštinė ekrano pikseliais (2: smulkus, 5: stambus)
effect-pixelize = Pikselizacija
    .facts =
        Visas 3D vaizdas (reljefas, lėktuvai, pastatai, kabina) dideliais kvadratiniais pikseliais kaip MS-DOS kompiuteryje; meniu ir patarimai lieka ryškūs (ne VR)
        Šliaužiklis nustato bloko dydį ekrano pikseliais (2: smulkus, 5: stambus)
        Kraštų glotninimas išjungtas, kol ji įjungta
        Trys jungikliai po ja palieka HUD, kabiną arba MFD, prietaisus ir Kneeboard ryškius
effect-sharp-hud = Be HUD
    .facts =
        Pikselizacija: HUD (ir šalmo ekranas) piešiamas ryškiai virš pikselizuoto vaizdo, kad plonieji simboliai išliktų įskaitomi
effect-sharp-cockpit = Be kabinos (mygtukai, jungikliai...)
    .facts =
        Pikselizacija: kabinos skydas, rėmas, sėdynė, mygtukai, jungikliai, vairalazdė ir greičio svirtis piešiami ryškiai
effect-sharp-gauges = Be MFD, prietaisų ir Kneeboard
    .facts =
        Pikselizacija: MFD, dirbtinis horizontas, prietaisai, lemputės, rodmenys, užrašai ir Kneeboard piešiami ryškiai
options-needs-pixelize = Reikia pikselizacijos
