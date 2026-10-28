# Fereastra OPȚIUNI (src/options.rs) și paginile ei din meniul din zbor.

options-title = OPȚIUNI
options-tab-game = JOC
options-tab-graphics = GRAFICĂ
options-tab-sound = SUNET
options-tab-controls = COMENZI
options-footer = Salvate pentru zborurile următoare; comutatoarele de realism se aleg la fiecare joc

## JOC

options-section-language = Limba
options-section-classic-modern = Clasic / Modern
options-cockpit = Cockpit
    .facts =
        Modern: afișaje de înaltă rezoluție cu butoane pe margine
        Clasic: dispunerea panoului de instrumente din epoca DOS și afișaje 77x60 (F-117A)
options-hud = HUD
    .facts =
        Modern: HUD verde de înaltă rezoluție, în stil F-16, cu modurile NAV, AA și AG
        Clasic: HUD-ul din epoca DOS
options-avionics = Avionică
    .facts =
        Modern: formate MFD în stil F-16 pe oricare MFD (afișaj tactic, HARM, nacelă de desemnare, hartă, încărcături) cu un cursor deplasabil
        Clasic: harta și TAC în stânga, camera, WAY, WPN și ORD în dreapta
options-flight-model = Model de zbor
    .facts =
        Modern: model de zbor aerodinamic cu comenzi electrice: stick-ul comandă viteza de tangaj și de ruliu; eliberat, botul rămâne pe loc și traiectoria este menținută
        Clasic: un model de zbor din epoca DOS
options-section-flying = Zbor
options-auto-trim = Trim automat
    .facts =
        Cu stick-ul eliberat, trimul menține atitudinea în care ați lăsat aeronava
        Elicoptere: botul nu se mai ridică odată cu viteza; avioane: compensează suprafețele de comandă avariate
        Trim manual: Alt + tastele săgeți, Alt + Home îl centrează
options-tooltips = Tooltip-uri din cockpit
    .facts = Ce este o comandă sau un afișaj din cockpit, sub mouse sau sub vârful degetului în VR
options-section-vr-size = Dimensiunea cockpitului în VR
options-vr-size =
    .facts =
        Dimensiunea cockpitului (și a lumii) în cască
        100%: mărime naturală

## GRAFICĂ

options-section-rendering = Randare
options-shaders = Shadere
    .facts =
        Modern: umbrire direcțională discretă, ceață de distanță fină
        Clasic: umbrire plată
options-shadows = Umbre
    .facts =
        Modern: umbre solare în cockpit și în lume (consumă timp GPU)
        Clasic: fără umbre
options-point-lights = Lumini punctuale
    .facts =
        Modern: lumini ambientale ale cockpitului și ale afișajelor; exploziile, penele de fum și luminile pistei luminează scena
        Clasic: doar lumina soarelui și a cerului
options-time-of-day = Ora zilei
    .facts =
        Modern: soarele și luna se mișcă odată cu ceasul misiunii, cerul se întunecă și se luminează odată cu soarele; stele și faza lunii noaptea
        Modern: lumina lunii stabilește de la ce distanță vă văd piloții inamici noaptea (2 km pe o noapte fără lună, 4 km sub luna plină)
        Clasic: o schimbare lentă a culorilor și un soare fix
options-msaa = Anti-aliasing (MSAA)
    .facts =
        Margini netede: MSAA 4x, 2x în VR (consumă timp GPU)
        Oprit: margini dure de pixeli
options-auto-detail = Detalii automate ale peisajului (VR)
    .facts =
        În cască, raza de afișare a orașelor, pădurilor și câmpurilor scade cât timp cadrele întârzie și crește la loc când rămâne timp
        Setările grafice rămân cele alese
options-sun-moon = Soare și lună
    .facts =
        Soarele pe cer ziua, luna și stelele noaptea
        Luna arată faza din data misiunii

## SUNET

options-section-volume = Volum
options-volume-master = General
    .facts = Toate sunetele
options-volume-engine = Motor
    .facts = Motorul sau rotorul dumneavoastră și aeronavele auzite în jur
options-volume-effects = Efecte
    .facts = Arme, explozii, tren de aterizare, radio și restul
options-volume-warnings = Alarme
    .facts = Claxonul de angajare, avertizările radar și de rachetă, sirena de incendiu, tonurile Sidewinder
options-section-cockpit = Cockpit
options-low-pass = Filtru trece-jos motor
    .facts =
        În interiorul aeronavei motorul sună înfundat (carlinga și casca taie frecvențele înalte)
        Oprit: la fel ca în exterior

## COMENZI

options-section-bindings = Asignări
options-keys-joystick = TASTE ȘI JOYSTICK
    .facts =
        Taste pentru hartă, axe și butoane de joystick și HOTAS
        Salvate pentru zborurile următoare
options-section-head-tracking = Urmărirea capului/ochilor
options-head-tracking = Urmărirea capului/ochilor
    .facts =
        Capul dumneavoastră întoarce și mută vederea în cockpit (desktop; în VR o face casca)
        Program de urmărire (OpenTrack...): ieșire FreeTrack sau UDP către 127.0.0.1 portul 4242
        Home: poziția curentă a capului devine centrul
options-tracking = Urmărire ({ $source })
options-no-tracker = Niciun tracker detectat: porniți programul de urmărire
