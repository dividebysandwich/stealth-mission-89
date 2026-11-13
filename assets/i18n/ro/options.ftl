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
options-keep-time-accel = Păstrează accelerarea timpului
    .facts =
        Accelerarea nu se mai oprește când ești detectat, lansezi o armă sau ești doborât
        Doar butonul TIME și Shift + Z / Shift + X o schimbă
options-tooltips = Tooltip-uri din cockpit
    .facts = Ce este o comandă sau un afișaj din cockpit, sub mouse sau sub vârful degetului în VR
options-player-labels = Numele jucătorilor
    .facts = Multiplayer: numele celorlalți piloți deasupra aeronavelor lor, la orice distanță
options-label-size =
    .facts =
        Mărimea numelor jucătorilor
        50%: implicit
options-seat-height = Înălțimea scaunului
    .facts =
        Cât de sus stai în carlingă, pe ecran și în VR
        0 cm: înălțimea normală a ochilor în avion
options-section-vr-size = Dimensiunea cockpitului în VR
options-vr-size =
    .facts =
        Dimensiunea cockpitului (și a lumii) în cască
        100%: mărime naturală
options-section-gif = Înregistrare GIF
options-gif-size = MĂRIME
options-gif-rate = CADRE
options-gif-scale-100 =
    .facts = Rezoluția completă a ferestrei: fișiere mari
options-gif-scale-25 =
    .facts = Fișiere mici
options-gif-rate-50 =
    .facts = Doar când jocul rulează la 100 fps sau mai mult (se captează un cadru din două)
options-gif-hint = F12 (dacă nu e reasignat) pornește și oprește; salvat în dosarul Imagini

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
options-scenery-distance = Distanța peisajului
    .facts = Localități îndepărtate: 7–30 km; câmpuri: 24–30 km. Distanțele copacilor rămân neschimbate. Detaliile automate VR pot reduce distanțele peisajului.
options-city-light-distance = Distanța luminilor urbane
    .facts = Lămpi stradale, ferestre și lumini de avertizare ale clădirilor, independent de distanța peisajului. Vremea poate limita vizibilitatea.
options-cloud-quality = Nori
cloud-quality-auto = Auto
    .facts = Scăzută în VR, ridicată pe ecran
cloud-quality-low = Scăzută
    .facts = Nori grosieri la o treime din rezoluție (cel mai rapid)
cloud-quality-medium = Medie
    .facts = Jumătate din rezoluție, detalii mai fine
cloud-quality-high = Ridicată
    .facts = Rezoluție completă, cele mai fine detalii și umbră proprie (costă timp GPU)

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
options-volume-rwr = RWR
    .facts = Tonurile de avertizare radar, ca parte a volumului alarmelor
options-section-rwr-tones = Tonuri de avertizare radar
rwr-tones-all = TOATE
    .facts = Un radar care vă iluminează, fiecare baleiaj care vă detectează, lansările de rachete
rwr-tones-detect = DOAR DETECȚIE
    .facts =
        Un ton când un radar vă detectează (din nou după ce radarele vă pierd o vreme)
        Și lansările de rachete
rwr-tones-launch = DOAR LANSĂRI
    .facts = Doar rachetele lansate asupra dumneavoastră
rwr-tones-off = OPRIT
    .facts = Niciun ton de avertizare radar (luminile și ecranele avertizează în continuare)
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

## EFFECTS

options-section-effects = Efecte
options-section-retro = Retro
effect-shake = Scuturarea camerei
    .facts =
        Vederea se scutură la tragerea tunului, explozii apropiate, pierderea portanței, aterizări și pistă
        VR: cabina vibrează, capul rămâne nemișcat
effect-flashes = Fulgere de luptă
    .facts = Explozii de antiaeriană, explozii și lansări de rachete luminează o clipă norii, solul și cabina
effect-barrage = Baraj de noapte
    .facts =
        Război convențional noaptea: tunurile orașelor trag orbește cu trasoare în cer, proiectoarele le mătură
        Dronele inamice deasupra unui oraș sunt prinse de proiectoarele lui și atrag focul tunurilor lui
        Doar pentru aspect: bariera nu lovește nimic
effect-cockpit-night = Iluminarea de noapte a cabinei
    .facts =
        O lumină roșie slabă în cabină după lăsarea întunericului (albastru-verzui sub ochelari)
        Master caution pulsează chihlimbar peste panou cât timp o avertizare este activă
effect-nvg = Ochelari de vedere nocturnă
    .facts =
        Luminile puternice se revarsă în aureole, imaginea este mai granulată în întuneric
        Desktop: câmpul vizual rotund al ochelarilor
effect-snow = Zăpadă
    .facts = Deasupra nivelului de îngheț furtunile aduc zăpadă în loc de ploaie: jos în iarna nordică, sus vara
effect-mist = Ceață la sol
    .facts =
        Shadere moderne: ceața stă peste terenul jos și mare în zori, la amurg și noaptea
        Subțire în deșerturi, deloc în furtuni
effect-cloud-shadows = Umbrele norilor
    .facts = Shadere moderne: umbrele norilor pe solul de dedesubt
effect-vapour = Vapori și curent de rotor
    .facts =
        La manevre strânse, dâre de vapori la vârfurile aripilor
        Un elicopter în zbor staționar jos ridică praf, sau spumă deasupra apei
        Vehiculele în mișcare ridică praf în urma lor
effect-night-sky = Cer de noapte
    .facts =
        din când în când stele căzătoare, stelele joase peste orizont scânteiază
        Aurora boreală peste Capul Nord
effect-vga = Culori VGA
    .facts = Aspectul din 1989 cu umbrire modernă: 64 de trepte pe canal de culoare, ca la o placă VGA, cu dithering
effect-scanlines = Linii de scanare
    .facts = Liniile de scanare ale unui monitor CRT (nu în VR)

## LANGUAGE NAMES

lang-name-en = engleză
lang-name-bg = bulgară
lang-name-zh-Hans = chineză (simplificată)
lang-name-hr = croată
lang-name-cs = cehă
lang-name-da = daneză
lang-name-nl = olandeză
lang-name-et = estonă
lang-name-fi = finlandeză
lang-name-fr = franceză
lang-name-de = germană
lang-name-el = greacă
lang-name-hu = maghiară
lang-name-id = indoneziană
lang-name-ga = irlandeză
lang-name-it = italiană
lang-name-ja = japoneză
lang-name-ko = coreeană
lang-name-lv = letonă
lang-name-lt = lituaniană
lang-name-mt = malteză
lang-name-pl = poloneză
lang-name-pt-BR = portugheză (Brazilia)
lang-name-ro = română
lang-name-ru = rusă
lang-name-sk = slovacă
lang-name-sl = slovenă
lang-name-es = spaniolă
lang-name-sv = suedeză
lang-name-uk = ucraineană

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Limba browserului: { $language }. Doriți să jucați în această limbă?
language-offer-yes = Da
language-offer-no = Nu
language-offer-never = Nu mai întreba
effect-sunbeams = Raze de soare
    .facts = Fascicule de lumină solară prin golurile unui înnorat fragmentat
options-msaa-forced = Dezactivat cât timp pixelarea (retro) este activă
options-volume-radio = Radio
    .facts = Ceilalți jucători care vorbesc la radioul coop și ATC-ul vorbit
options-volume-atc = Voce ATC
    .facts = Apelurile ATC vorbite, ca parte a volumului radioului
options-volume-music = Muzică
    .facts = Muzica meniului principal
options-section-atc = ATC vorbit
options-section-radio = Radio coop
options-push-to-talk = Apasă și vorbește
    .facts =
        Țineți apăsată tasta de vorbit (Caps Lock sau un buton de joystick ori VR atribuit PTT) și toți jucătorii vă aud
        Microfonul se deschide la prima apăsare
options-mic-gain = AMPLIFICARE MIC.
    .facts =
        Cât de tare iese microfonul dvs.
        100%: așa cum este
options-mic-default = Implicit
options-voice-text-only = doar text
options-voices-line = Controlori: { $controllers }   Pilot: { $pilot }
options-voices = VOCI
    .facts =
        Descărcați voci pentru apelurile radio, ștergeți-le
        Alegeți vocile controlorilor și pe a dvs. și ascultați-le
options-voices-web = În browser doar text
options-tracking-mouse = Privire cu mouse-ul la tracking
    .facts =
        Activat: butonul drept al mouse-ului tot rotește și mărește vederea din cabină
        Dezactivat: cât timp trackerul transmite, butonul drept al mouse-ului nu face nimic în cabină
options-atc = Controlul traficului aerian
    .facts =
        Apeluri radio ale turnului, apropierii și GCA pe aerodromurile aliate
        Autorizare de decolare, overhead break și talkdown pe nori joși sau vizibilitate slabă
options-pixel-block = Dimensiunea blocului de pixelare
    .facts = Latura unui bloc, în pixeli de ecran (2: fin, 5: grosier)
effect-pixelize = Pixelare
    .facts =
        Toată vederea 3D (teren, avioane, clădiri, cabină) în pixeli pătrați mari, ca pe un PC cu MS-DOS; meniurile și sfaturile rămân clare (nu în VR)
        Cursorul stabilește dimensiunea unui bloc, în pixeli de ecran (2: fin, 5: grosier)
        Antialiasing-ul este dezactivat cât timp este activă
        Cele trei comutatoare de sub ea lasă clare HUD-ul, cabina sau MFD-urile, instrumentele și planșeta
effect-sharp-hud = Fără HUD
    .facts =
        Pixelare: HUD-ul (și afișajul căștii) este desenat clar peste imaginea pixelată, ca simbolurile sale subțiri să rămână lizibile
effect-sharp-cockpit = Fără cabină (butoane, comutatoare...)
    .facts =
        Pixelare: panoul, cadrul, scaunul, butoanele, comutatoarele, manșa și maneta de gaze ale cabinei sunt desenate clar
effect-sharp-gauges = Fără MFD-uri, instrumente și planșetă
    .facts =
        Pixelare: MFD-urile, orizontul artificial, instrumentele, becurile, afișajele, etichetele și planșeta sunt desenate clar
options-needs-pixelize = Necesită Pixelare
