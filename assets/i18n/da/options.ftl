# Vinduet INDSTILLINGER (src/options.rs) og dets sider i flyvemenuen.

options-title = INDSTILLINGER
options-tab-game = SPIL
options-tab-graphics = GRAFIK
options-tab-sound = LYD
options-tab-controls = STYRING
options-footer = Gemmes til de næste flyvninger; realismeindstillingerne vælges for hvert spil

## SPIL

options-section-language = Sprog
options-section-classic-modern = Klassisk / Moderne
options-cockpit = Cockpit
    .facts =
        Moderne: højopløselige skærme med knapper i kanten
        Klassisk: instrumentpanelet fra DOS-tiden og 77x60-skærme (F-117A)
options-hud = HUD
    .facts =
        Moderne: højopløselig grøn HUD i F-16-stil med tilstandene NAV, AA og AG
        Klassisk: HUD'en fra DOS-tiden
options-avionics = Avionik
    .facts =
        Moderne: MFD-sider i F-16-stil på hver MFD (situationsbillede, HARM, targeting pod, kort, last) med en bevægelig markør
        Klassisk: kort og TAC til venstre, kamera, WAY, WPN og ORD til højre
options-flight-model = Flymodel
    .facts =
        Moderne: aerodynamisk flymodel med fly-by-wire: stikken styrer stignings- og rulningshastighed; når den slippes, bliver næsen, hvor den er, og flyvebanen holdes
        Klassisk: en flymodel fra DOS-tiden
options-section-flying = Flyvning
options-auto-trim = Automatisk trim
    .facts =
        Når stikken slippes, holder trimmet den stilling, du forlod
        Helikoptere: næsen stiger ikke mere med farten; jetfly: trimmer beskadigede ror ud
        Manuel trim: Alt + piletaster, Alt + Home centrerer den
options-tooltips = Cockpit-tooltips
    .facts = Hvad et cockpitinstrument eller en skærm er, under musen eller en VR-fingerspids
options-player-labels = Spillernavne
    .facts = Multiplayer: de andre piloters navne over deres fly, uanset afstand
options-label-size =
    .facts =
        Spillernavnenes størrelse
        50 %: standard
options-section-vr-size = Cockpitstørrelse i VR
options-vr-size =
    .facts =
        Cockpittets (og verdens) størrelse i headsettet
        100 %: i naturlig størrelse
options-section-gif = GIF-optagelse
options-gif-size = STØRRELSE
options-gif-rate = BILLEDER
options-gif-scale-100 =
    .facts = Vinduets fulde opløsning: store filer
options-gif-scale-25 =
    .facts = Små filer
options-gif-rate-50 =
    .facts = Kun når spillet kører med 100 fps eller mere (hvert andet billede optages)
options-gif-hint = F12 (medmindre omdefineret) starter og stopper; gemmes i mappen Billeder

## GRAFIK

options-section-rendering = Gengivelse
options-shaders = Shaders
    .facts =
        Moderne: diskret retningsbestemt skygge, blød afstandsdis
        Klassisk: flad skygge
options-shadows = Skygger
    .facts =
        Moderne: solskygger i cockpittet og verdenen (koster GPU-tid)
        Klassisk: ingen skygger
options-point-lights = Punktlys
    .facts =
        Moderne: cockpit- og skærmbelysning; eksplosioner, røgfaner og banelys oplyser scenen
        Klassisk: kun sol- og himmellys
options-time-of-day = Tidspunkt på dagen
    .facts =
        Moderne: solen og månen følger missionens ur, himlen bliver mørkere og lysere med solen; stjerner og månens fase om natten
        Moderne: måneskinnet afgør, hvor langt fjendtlige piloter ser dig om natten (2 km uden måne, 4 km under fuldmåne)
        Klassisk: en langsom ændring af farver og en fast sol
options-msaa = Anti-aliasing (MSAA)
    .facts =
        Bløde kanter: 4x MSAA, 2x i VR (koster GPU-tid)
        Fra: hårde pixelkanter
options-auto-detail = Automatisk landskabsdetalje (VR)
    .facts =
        I headsettet skrumper rækkevidden af byer, skove og marker, når billederne kommer for sent, og vokser igen, når der er tid til overs
        Grafikindstillingerne forbliver, som de er valgt
options-sun-moon = Sol og måne
    .facts =
        Solen på himlen om dagen, månen og stjernerne om natten
        Månen viser missionens fase

## LYD

options-section-volume = Lydstyrke
options-volume-master = Samlet
    .facts = Alle lyde
options-volume-engine = Motor
    .facts = Din motor eller rotor og de fly, man hører omkring dig
options-volume-effects = Effekter
    .facts = Våben, eksplosioner, understel, radio og resten
options-volume-warnings = Advarsler
    .facts = Stall-horn, radar- og missiladvarsler, brandsirenen, Sidewinder-tonerne
options-volume-rwr = RWR
    .facts = Radaradvarselstonerne, som en del af advarslernes lydstyrke
options-section-rwr-tones = Radaradvarselstoner
rwr-tones-all = ALLE
    .facts = En radar, der belyser dig, hvert sweep, der opdager dig, missilaffyringer
rwr-tones-detect = KUN OPDAGELSE
    .facts =
        Én tone, når en radar opdager dig (igen efter at radarerne har sluppet dig et stykke tid)
        Og missilaffyringer
rwr-tones-launch = KUN AFFYRINGER
    .facts = Kun missiler affyret mod dig
rwr-tones-off = FRA
    .facts = Ingen radaradvarselstoner (lamperne og skærmene advarer stadig)
options-section-cockpit = Cockpit
options-low-pass = Lavpasfilter på motor
    .facts =
        Inde i flyet lyder motoren dæmpet (kabinehætten og hjelmen tager højderne af)
        Fra: det samme som udenfor

## STYRING

options-section-bindings = Bindinger
options-keys-joystick = TASTER OG JOYSTICK
    .facts =
        Tildel taster samt joystickets og HOTAS'ens akser og knapper
        Gemmes til de næste flyvninger
options-section-head-tracking = Hoved- og øjensporing
options-head-tracking = Hoved- og øjensporing
    .facts =
        Dit hoved drejer og flytter udsigten i cockpittet (på skærm; i VR er det headsettet)
        Sporingsprogram (OpenTrack...): FreeTrack-udgang eller UDP til 127.0.0.1 port 4242
        Home: den aktuelle hovedposition bliver midten
options-tracking = Sporing ({ $source })
options-no-tracker = Ingen tracker hørt: start sporingsprogrammet

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
effect-vapour = Damp og rotorvind
    .facts =
        Ved hårde manøvrer dampstriber fra vingespidserne
        En helikopter, der svæver lavt, hvirvler støv op, eller sprøjt over vand
        Køretøjer i bevægelse hvirvler støv op bag sig
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

lang-name-en = engelsk
lang-name-bg = bulgarsk
lang-name-zh-Hans = kinesisk (forenklet)
lang-name-hr = kroatisk
lang-name-cs = tjekkisk
lang-name-da = dansk
lang-name-nl = nederlandsk
lang-name-et = estisk
lang-name-fi = finsk
lang-name-fr = fransk
lang-name-de = tysk
lang-name-el = græsk
lang-name-hu = ungarsk
lang-name-id = indonesisk
lang-name-ga = irsk
lang-name-it = italiensk
lang-name-ja = japansk
lang-name-ko = koreansk
lang-name-lv = lettisk
lang-name-lt = litauisk
lang-name-mt = maltesisk
lang-name-pl = polsk
lang-name-pt-BR = portugisisk (Brasilien)
lang-name-ro = rumænsk
lang-name-ru = russisk
lang-name-sk = slovakisk
lang-name-sl = slovensk
lang-name-es = spansk
lang-name-sv = svensk
lang-name-uk = ukrainsk

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Browserens sprog: { $language }. Vil du spille på dette sprog?
language-offer-yes = Ja
language-offer-no = Nej
language-offer-never = Spørg ikke igen
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
