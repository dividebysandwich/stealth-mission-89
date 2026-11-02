# SEADETE aken (src/options.rs) ja selle lehed lennusisese menüüs.

options-title = SEADED
options-tab-game = MÄNG
options-tab-graphics = GRAAFIKA
options-tab-sound = HELI
options-tab-controls = JUHTIMINE
options-footer = Salvestatakse järgmisteks lendudeks; realismilülitid valitakse iga mänguga

## MÄNG

options-section-language = Keel
options-section-classic-modern = Klassikaline / kaasaegne
options-cockpit = Cockpit
    .facts =
        Kaasaegne: kõrglahutusega ekraanid raami nuppudega
        Klassikaline: DOS-ajastu instrumendipaneeli paigutus ja 77x60 ekraanid (F-117A)
options-hud = HUD
    .facts =
        Kaasaegne: kõrglahutusega F-16 stiilis roheline HUD režiimidega NAV, AA ja AG
        Klassikaline: DOS-ajastu HUD
options-avionics = Avioonika
    .facts =
        Kaasaegne: F-16 stiilis MFD formaadid kummalgi MFD-l (olukorra ekraan, HARM, targeting pod, kaart, stores) liigutatava kursoriga
        Klassikaline: kaart ja TAC vasakul, kaamera, WAY, WPN ja ORD paremal
options-flight-model = Lennumudel
    .facts =
        Kaasaegne: aerodünaamiline lennumudel fly-by-wire'iga: stick määrab tõusu- ja rullikiiruse; lahti lastuna jääb nina paigale ja lennutrajektoor säilib
        Klassikaline: DOS-ajastu lennumudel
options-section-flying = Lendamine
options-auto-trim = Auto trim
    .facts =
        Kui stick on lahti, hoiab trim lennuasendit seal, kus selle jätsid
        Kopterid: nina ei tõuse enam kiirusega; jetid: trimmib kahjustatud juhtpinnad välja
        Käsitsi trim: Alt + nooleklahvid, Alt + Home tsentreerib
options-tooltips = Cockpiti tooltips
    .facts = Mis on cockpiti juhtelement või ekraan, hiire all või VR-i sõrmeotsas
options-player-labels = Mängijate nimed
    .facts = Mitmikmäng: teiste pilootide nimed nende lennukite kohal, mis tahes kauguselt
options-label-size =
    .facts =
        Mängijate nimede suurus
        50%: vaikimisi
options-section-vr-size = VR-cockpiti suurus
options-vr-size =
    .facts =
        Cockpiti (ja maailma) suurus peakomplektis
        100%: elusuurus
options-section-gif = GIF-salvestus
options-gif-size = SUURUS
options-gif-rate = KAADRID
options-gif-scale-100 =
    .facts = Akna täislahutus: suured failid
options-gif-scale-25 =
    .facts = Väikesed failid
options-gif-rate-50 =
    .facts = Ainult siis, kui mäng jookseb 100 fps või rohkem (salvestatakse iga teine kaader)
options-gif-hint = F12 (kui pole ümber määratud) alustab ja lõpetab; salvestatakse kausta Pildid

## GRAAFIKA

options-section-rendering = Renderdus
options-shaders = Shaders
    .facts =
        Kaasaegne: õrn suunatud varjutus, sujuv kauguse uduhõõre
        Klassikaline: lame varjutus
options-shadows = Varjud
    .facts =
        Kaasaegne: päikesevarjud cockpitis ja maailmas (koormab GPU-d)
        Klassikaline: varje pole
options-point-lights = Punktvalgustid
    .facts =
        Kaasaegne: cockpiti ja ekraanide valgustus; plahvatused, suitsusabad ja rajatuled valgustavad stseeni
        Klassikaline: ainult päikese- ja taevavalgus
options-time-of-day = Kellaaeg
    .facts =
        Kaasaegne: päike ja kuu liiguvad missiooni kellaga, taevas tumeneb ja heleneb päikesega; öösel tähed ja kuufaas
        Kaasaegne: kuuvalgus määrab, kui kaugele vaenlase piloodid sind öösel näevad (2 km kuuta ööl, 4 km täiskuu all)
        Klassikaline: aeglane värvide muutus ja fikseeritud päike
options-msaa = Anti-aliasing (MSAA)
    .facts =
        Siledad servad: 4x MSAA, VR-is 2x (koormab GPU-d)
        Väljas: teravad piksliservad
options-auto-detail = Automaatne maastiku detailsus (VR)
    .facts =
        Peakomplektis kahaneb linnade, metsade ja põldude kuvamiskaugus, kui kaadrid hilinevad, ja kasvab tagasi, kui aega jätkub
        Graafikaseaded jäävad valituks
options-sun-moon = Päike ja kuu
    .facts =
        Päike taevas päeval, kuu ja tähed öösel
        Kuu näitab missiooni faasi
options-cloud-quality = Pilved
cloud-quality-auto = Auto
    .facts = Madal VR-is, kõrge ekraanil
cloud-quality-low = Madal
    .facts = Jämedad pilved kolmandiku eraldusvõimega (kiireim)
cloud-quality-medium = Keskmine
    .facts = Pool eraldusvõimest, peenemad detailid
cloud-quality-high = Kõrge
    .facts = Täielik eraldusvõime, peenimad detailid ja omavari (kulutab GPU aega)

## HELI

options-section-volume = Helitugevus
options-volume-master = Üldine
    .facts = Kõik helid
options-volume-engine = Mootor
    .facts = Sinu mootor või rootor ja sinu ümber kuuldavad lennukid
options-volume-effects = Efektid
    .facts = Relvad, plahvatused, telik, raadio ja ülejäänu
options-volume-warnings = Hoiatused
    .facts = Kukkumishoiatus, radari ja raketi hoiatused, tulesireen, Sidewinderi toonid
options-volume-rwr = RWR
    .facts = Radarihoiatuse toonid, osana hoiatuste helitugevusest
options-section-rwr-tones = Radarihoiatuse toonid
rwr-tones-all = KÕIK
    .facts = Sind valgustav radar, iga sind avastav pühkimine, rakettide stardid
rwr-tones-detect = AINULT AVASTAMINE
    .facts =
        Üks toon, kui radar sind avastab (uuesti, kui radarid on sind mõneks ajaks kaotanud)
        Ja rakettide stardid
rwr-tones-launch = AINULT STARDID
    .facts = Ainult sinu pihta lastud raketid
rwr-tones-off = VÄLJAS
    .facts = Radarihoiatuse toone pole (tuled ja ekraanid hoiatavad endiselt)
options-section-cockpit = Cockpit
options-low-pass = Mootori madalpääs
    .facts =
        Lennuki sees kõlab mootor summutatult (kupu ja kiiver võtavad kõrged toonid ära)
        Väljas: sama mis väljas

## JUHTIMINE

options-section-bindings = Sidumised
options-keys-joystick = KLAHVID JA JOYSTICK
    .facts =
        Kaardi klahvid, joystick'i ja HOTAS-i teljed ja nupud
        Salvestatakse järgmisteks lendudeks
options-section-head-tracking = Pea/silmade jälgimine
options-head-tracking = Pea/silmade jälgimine
    .facts =
        Sinu pea pöörab ja liigutab vaadet cockpitis (töölaual; VR-is teeb seda peakomplekt)
        Jälgimisprogramm (OpenTrack...): FreeTracki väljund või UDP aadressile 127.0.0.1 port 4242
        Home: praegune pea asend saab keskpunktiks
options-tracking = Jälgimine ({ $source })
options-no-tracker = Jälgijat ei kuule: käivita jälgimisprogramm

## EFFECTS

options-section-effects = Efektid
options-section-retro = Retro
effect-shake = Kaamera värin
    .facts =
        Vaade väriseb kahuri, lähedaste plahvatuste, kaotatud tõstejõu, maandumiste ja raja tõttu
        VR: kabiin vibreerib, pea püsib paigal
effect-flashes = Lahingusähvatused
    .facts = Õhutõrjelaengute plahvatused, plahvatused ja rakettide start valgustavad hetkeks pilvi, maad ja kabiini
effect-barrage = Öine tõkkekuri
    .facts =
        Tavasõda öösel: linnade kahurid tulistavad pimesi jälgkuule taevasse, prožektorid pühivad nende kohal
        Vaenlase droonid linna kohal jäävad selle prožektoritesse ja tõmbavad selle kahurite tule ligi
        Ainult välimuse pärast: tõke ei taba midagi
effect-cockpit-night = Kabiini öövalgustus
    .facts =
        Hämar punane valgus kabiinis pärast pimenemist (sinakasroheline prillide all)
        Master caution pulseerib merevaigukarva paneeli kohal, kuni hoiatus on aktiivne
effect-nvg = Öönägemisprillid
    .facts =
        Eredad tuled paisuvad halodeks, pilt on pimedas teralisem
        Töölaud: prillide ümar vaateväli
effect-snow = Lumi
    .facts = Külmumispiirist kõrgemal toovad tormid vihma asemel lund: madalal põhjapoolsel talvel, kõrgel suvel
effect-mist = Maaudu
    .facts =
        Kaasaegsed shaderid: udu katab madalat maad ja merd koidikul, hämaruses ja öösel
        Õhuke kõrbetes, tormis puudub
effect-cloud-shadows = Pilvevarjud
    .facts = Kaasaegsed shaderid: pilvede varjud all olevale maale
effect-vapour = Aur ja rootori allavool
    .facts =
        Järskudel manöövritel auruvööd tiivaotstest
        Madalal hõljuv helikopter tõstab tolmu, vee kohal pritsmeid
        Liikuvad masinad tõstavad enda taha tolmu
effect-night-sky = Öötaevas
    .facts =
        Aeg-ajalt langevad tähed, horisondi kohal madalad tähed vilguvad
        Virmalised Nordkapi kohal
effect-vga = VGA värvid
    .facts = 1989. aasta ilme kaasaegse varjutusega: 64 astet värvikanali kohta nagu VGA-kaardil, ditheringuga
effect-scanlines = Skaneerimisjooned
    .facts = CRT-monitori skaneerimisjooned (mitte VR-is)

## LANGUAGE NAMES

lang-name-en = inglise
lang-name-bg = bulgaaria
lang-name-zh-Hans = hiina (lihtsustatud)
lang-name-hr = horvaadi
lang-name-cs = tšehhi
lang-name-da = taani
lang-name-nl = hollandi
lang-name-et = eesti
lang-name-fi = soome
lang-name-fr = prantsuse
lang-name-de = saksa
lang-name-el = kreeka
lang-name-hu = ungari
lang-name-id = indoneesia
lang-name-ga = iiri
lang-name-it = itaalia
lang-name-ja = jaapani
lang-name-ko = korea
lang-name-lv = läti
lang-name-lt = leedu
lang-name-mt = malta
lang-name-pl = poola
lang-name-pt-BR = portugali (Brasiilia)
lang-name-ro = rumeenia
lang-name-ru = vene
lang-name-sk = slovaki
lang-name-sl = sloveeni
lang-name-es = hispaania
lang-name-sv = rootsi
lang-name-uk = ukraina

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Brauseri keel: { $language }. Kas mängida selles keeles?
language-offer-yes = Jah
language-offer-no = Ei
language-offer-never = Ära küsi enam
effect-sunbeams = Päikesekiired
    .facts = Päikesevalguse kiired rebenenud pilvkatte pragudest
options-msaa-forced = Väljas, kuni pikseldamine (retro) on sees
options-volume-radio = Raadio
    .facts = Teised koop-raadios rääkivad mängijad ja kõneldud ATC
options-volume-atc = ATC hääl
    .facts = Kõneldud ATC kutsed, raadio helitugevuse osana
options-volume-music = Muusika
    .facts = Peamenüü muusika
options-section-atc = Kõneldud ATC
options-section-radio = Koop-raadio
options-push-to-talk = Vajuta ja räägi
    .facts =
        Hoia all rääkimisklahvi (Caps Lock või PTT-le määratud joysticki või VR-i nupp) ja kõik mängijad kuulevad sind
        Mikrofon avaneb esimesel vajutusel
options-mic-gain = MIKROFONI VÕIMENDUS
    .facts =
        Kui valjult su mikrofon väljub
        100%: nagu on
options-mic-default = Vaikimisi
options-voice-text-only = ainult tekst
options-voices-line = Juhid: { $controllers }   Piloot: { $pilot }
options-voices = HÄÄLED
    .facts =
        Laadi alla hääled raadiokutsete jaoks, kustuta need
        Vali juhtide ja enda hääl ning kuula neid
options-voices-web = Brauseris ainult tekst
options-tracking-mouse = Hiirega vaatamine jälgimisel
    .facts =
        Sees: parem hiirenupp pöörab ja suumib endiselt kabiinivaadet
        Väljas: kuni jälgija saadab, ei tee parem hiirenupp kabiinis midagi
options-atc = Lennuliikluse juhtimine
    .facts =
        Torni, lähenemise ja GCA raadiokutsed sõbralikel lennuväljadel
        Stardiluba, overhead break ja talkdown madala pilvisuse või halva nähtavuse korral
options-pixel-block = Pikseldamise ploki suurus
    .facts = Ploki külg ekraanipikslites (2: peen, 5: jäme)
effect-pixelize = Pikseldamine
    .facts =
        Kogu 3D-vaade (maastik, lennukid, hooned, kabiin) suurte ruudukujuliste pikslitena nagu MS-DOS-arvutis; menüüd ja kohtspikrid jäävad teravaks (mitte VR-is)
        Liugur määrab ploki suuruse ekraanipikslites (2: peen, 5: jäme)
        Ääntesilumine on väljas, kuni see on sees
