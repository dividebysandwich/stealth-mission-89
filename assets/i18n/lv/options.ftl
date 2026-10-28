# Logs OPCIJAS (src/options.rs) un tā lapas lidojuma izvēlnē.

options-title = OPCIJAS
options-tab-game = SPĒLE
options-tab-graphics = GRAFIKA
options-tab-sound = SKAŅA
options-tab-controls = VADĪBA
options-footer = Saglabāts nākamajiem lidojumiem; reālisma slēdži tiek izvēlēti katrai spēlei

## SPĒLE

options-section-language = Valoda
options-section-classic-modern = Klasisks / Moderns
options-cockpit = Kabīne
    .facts =
        Moderns: augstas izšķirtspējas displeji ar apmales pogām
        Klasisks: DOS laikmeta instrumentu paneļa izkārtojums un 77x60 displeji (F-117A)
options-hud = HUD
    .facts =
        Moderns: augstas izšķirtspējas zaļš F-16 stila HUD ar NAV, AA un AG režīmiem
        Klasisks: DOS laikmeta HUD
options-avionics = Avionika
    .facts =
        Moderns: F-16 stila MFD formāti uz katra MFD (situācijas displejs, HARM, targeting pod, karte, bruņojums) ar pārvietojamu kursoru
        Klasisks: karte un TAC pa kreisi, kamera, WAY, WPN un ORD pa labi
options-flight-model = Lidojuma modelis
    .facts =
        Moderns: aerodinamisks lidojuma modelis ar fly-by-wire: stick komandē ruļļa un tangāžas ātrumu; atlaists, deguns paliek vietā un lidojuma trajektorija tiek noturēta
        Klasisks: DOS laikmeta lidojuma modelis
options-section-flying = Pilotēšana
options-auto-trim = Automātiskais trim
    .facts =
        Ar atlaistu stick trim notur stāvokli, kādā to atstājāt
        Helikopteri: deguns vairs necelas līdz ar ātrumu; reaktīvās lidmašīnas: kompensē bojātas vadības virsmas
        Manuālais trim: Alt + bulttaustiņi, Alt + Home to centrē
options-tooltips = Kabīnes tooltips
    .facts = Kas ir kabīnes vadības elements vai displejs, zem peles vai VR pirksta gala
options-section-vr-size = VR kabīnes izmērs
options-vr-size =
    .facts =
        Kabīnes (un pasaules) izmērs austiņās
        100%: dabiskais izmērs

## GRAFIKA

options-section-rendering = Renderēšana
options-shaders = Shaders
    .facts =
        Moderns: smalks virziena ēnojums, mīksta attāluma dūmaka
        Klasisks: plakans ēnojums
options-shadows = Ēnas
    .facts =
        Moderns: saules ēnas kabīnē un pasaulē (patērē GPU laiku)
        Klasisks: bez ēnām
options-point-lights = Punktveida gaismas
    .facts =
        Moderns: kabīnes un displeju apgaismojums; sprādzieni, dūmu joslas un skrejceļa gaismas apgaismo ainu
        Klasisks: tikai saules un debesu gaisma
options-time-of-day = Diennakts laiks
    .facts =
        Moderns: saule un Mēness seko misijas pulkstenim, debesis satumst un noskaidrojas līdz ar sauli; naktī zvaigznes un Mēness fāze
        Moderns: mēness gaisma nosaka, no cik tāluma ienaidnieka piloti jūs redz naktī (2 km bezmēness naktī, 4 km pilnmēness laikā)
        Klasisks: lēna krāsu maiņa un nekustīga saule
options-msaa = Kropļojumu mazināšana (MSAA)
    .facts =
        Gludas malas: 4x MSAA, 2x VR (patērē GPU laiku)
        Izslēgta: asas pikseļu malas
options-auto-detail = Automātiska ainavas detalizācija (VR)
    .facts =
        Austiņās pilsētu, mežu un lauku redzamības rādiuss samazinās, kamēr kadri kavējas, un atkal pieaug, kad ir laika rezerve
        Grafikas iestatījumi paliek, kā izvēlēti
options-sun-moon = Saule un Mēness
    .facts =
        Saule debesīs dienā, Mēness un zvaigznes naktī
        Mēness parāda misijas fāzi

## SKAŅA

options-section-volume = Skaļums
options-volume-master = Kopējais
    .facts = Visas skaņas
options-volume-engine = Dzinējs
    .facts = Jūsu dzinējs vai rotors un lidmašīnas, kas dzirdamas jums apkārt
options-volume-effects = Efekti
    .facts = Ieroči, sprādzieni, šasija, radio un pārējais
options-volume-warnings = Brīdinājumi
    .facts = Pārvilkšanas signāls, radara un raķešu brīdinājumi, ugunsgrēka sirēna, Sidewinder toņi
options-section-cockpit = Kabīne
options-low-pass = Dzinēja zemfrekvenču filtrs
    .facts =
        Lidmašīnas iekšpusē dzinējs skan apslāpēti (nojume un ķivere nogriež augstos toņus)
        Izslēgts: tāpat kā ārpusē

## VADĪBA

options-section-bindings = Piesaistes
options-keys-joystick = TAUSTIŅI UN JOYSTICK
    .facts =
        Piesaistīt taustiņus, joystick un HOTAS asis un pogas
        Saglabāts nākamajiem lidojumiem
options-section-head-tracking = Galvas un acu izsekošana
options-head-tracking = Galvas un acu izsekošana
    .facts =
        Jūsu galva griež un pārvieto skatu kabīnē (galddatorā; VR to dara austiņas)
        Izsekošanas programma (OpenTrack...): FreeTrack izvade vai UDP uz 127.0.0.1 portu 4242
        Home: pašreizējā galvas pozīcija kļūst par centru
options-tracking = Izsekošana ({ $source })
options-no-tracker = Izsekotājs nav konstatēts: palaidiet izsekošanas programmu

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

lang-name-en = angļu
lang-name-bg = bulgāru
lang-name-zh-Hans = ķīniešu (vienkāršotā)
lang-name-hr = horvātu
lang-name-cs = čehu
lang-name-da = dāņu
lang-name-nl = holandiešu
lang-name-et = igauņu
lang-name-fi = somu
lang-name-fr = franču
lang-name-de = vācu
lang-name-el = grieķu
lang-name-hu = ungāru
lang-name-id = indonēziešu
lang-name-ga = īru
lang-name-it = itāļu
lang-name-ja = japāņu
lang-name-ko = korejiešu
lang-name-lv = latviešu
lang-name-lt = lietuviešu
lang-name-mt = maltiešu
lang-name-pl = poļu
lang-name-pt-BR = portugāļu (Brazīlija)
lang-name-ro = rumāņu
lang-name-ru = krievu
lang-name-sk = slovāku
lang-name-sl = slovēņu
lang-name-es = spāņu
lang-name-sv = zviedru
lang-name-uk = ukraiņu
