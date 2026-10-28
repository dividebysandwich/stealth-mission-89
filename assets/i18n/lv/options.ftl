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
