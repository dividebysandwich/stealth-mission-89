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
options-section-vr-size = VR-cockpiti suurus
options-vr-size =
    .facts =
        Cockpiti (ja maailma) suurus peakomplektis
        100%: elusuurus

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

options-section-effects = Effects
options-section-retro = Retro
effect-shake = Camera shake
    .facts =
        The view shakes with the gun, nearby blasts, stall buffet, touchdowns and the runway
        VR: the cockpit vibrates, your head stays still
effect-flashes = Combat flashes
    .facts =
        Flak bursts, explosions and missile launches light the clouds, the ground and the cockpit for a moment
effect-barrage = Night barrage
    .facts =
        Conventional war at night: the towns' guns fire tracer blindly into the sky, searchlights sweep over them
        Only for the look: the barrage hits nothing, the searchlights find nothing
effect-cockpit-night = Cockpit night lighting
    .facts =
        A dim red flood light in the cockpit after dark (blue-green under the goggles)
        The master caution pulses amber over the panel while a warning is on
effect-nvg = Night vision goggles
    .facts =
        Bright lights bloom into halos, the picture grains more in the dark
        Desktop: the goggles' round field of view
effect-snow = Snow
    .facts =
        Above the freezing level storms snow instead of rain: low in a northern winter, high in summer
effect-mist = Ground mist
    .facts =
        Modern shaders: mist lies over the low ground and the sea at dawn, dusk and night
        Thin in the deserts, none in storms
effect-cloud-shadows = Cloud shadows
    .facts =
        Modern shaders: the clouds' shadows on the ground below
effect-vapour = Vapour and downwash
    .facts =
        Pulling hard, vapour trails from the wingtips
        A helicopter hovering low whips up dust, or spray over water
effect-night-sky = Night sky
    .facts =
        Shooting stars now and then, the stars low over the horizon twinkle
        The northern lights over the North Cape
effect-vga = VGA colours
    .facts =
        The 1989 look with modern shading: 64 steps a colour channel, as a VGA card's, dithered
effect-scanlines = Scanlines
    .facts =
        A CRT monitor's scanlines (not in VR)

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
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
