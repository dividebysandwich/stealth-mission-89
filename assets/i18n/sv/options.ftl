# Fönstret ALTERNATIV (src/options.rs) och dess sidor i flygmenyn.

options-title = ALTERNATIV
options-tab-game = SPEL
options-tab-graphics = GRAFIK
options-tab-sound = LJUD
options-tab-controls = STYRNING
options-footer = Sparas till nästa flygningar; realismreglagen väljs för varje spel

## SPEL

options-section-language = Språk
options-section-classic-modern = Klassisk / Modern
options-cockpit = Cockpit
    .facts =
        Modern: högupplösta skärmar med ramknappar
        Klassisk: instrumentpanelen från DOS-eran och skärmar på 77x60 (F-117A)
options-hud = HUD
    .facts =
        Modern: högupplöst grön HUD i F-16-stil med lägena NAV, AA och AG
        Klassisk: HUD:en från DOS-eran
options-avionics = Avionik
    .facts =
        Modern: MFD-format i F-16-stil på båda MFD:erna (lägesbild, HARM, målpod, karta, last) med en förflyttbar markör
        Klassisk: karta och TAC till vänster, kamera, WAY, WPN och ORD till höger
options-flight-model = Flygmodell
    .facts =
        Modern: aerodynamisk flygmodell med fly-by-wire: styrspaken styr stignings- och rullhastighet; när den släpps stannar nosen kvar och flygbanan hålls
        Klassisk: en flygmodell från DOS-eran
options-section-flying = Flygning
options-auto-trim = Automatisk trim
    .facts =
        När styrspaken släpps håller trimmen attityden där du lämnade den
        Helikoptrar: nosen stiger inte längre med farten; jetplan: trimmar bort skadade roder
        Manuell trim: Alt + piltangenter, Alt + Home centrerar den
options-tooltips = Tooltips i cockpit
    .facts = Vad ett reglage eller en skärm i cockpit är, under musen eller en fingertopp i VR
options-section-vr-size = Cockpitens storlek i VR
options-vr-size =
    .facts =
        Cockpitens (och världens) storlek i headsetet
        100 %: naturlig storlek

## GRAFIK

options-section-rendering = Rendering
options-shaders = Shaders
    .facts =
        Modern: diskret riktad skuggning, mjuk avståndsdis
        Klassisk: platt skuggning
options-shadows = Skuggor
    .facts =
        Modern: solskuggor i cockpit och i världen (kostar GPU-tid)
        Klassisk: inga skuggor
options-point-lights = Punktljus
    .facts =
        Modern: belysning av cockpit och skärmar; explosioner, rökpelare och banljus lyser upp scenen
        Klassisk: bara sol- och himmelsljus
options-time-of-day = Tid på dygnet
    .facts =
        Modern: solen och månen följer uppdragsklockan, himlen mörknar och ljusnar med solen; stjärnor och månens fas på natten
        Modern: månljuset avgör hur långt fientliga piloter ser dig på natten (2 km en månlös natt, 4 km vid fullmåne)
        Klassisk: en långsam färgförändring och en fast sol
options-msaa = Kantutjämning (MSAA)
    .facts =
        Mjuka kanter: 4x MSAA, 2x i VR (kostar GPU-tid)
        Av: hårda pixelkanter
options-auto-detail = Automatisk terrängdetalj (VR)
    .facts =
        I headsetet krymper räckvidden för städer, skogar och fält när bildrutorna släpar efter och växer tillbaka när det finns tid över
        Grafikinställningarna förblir som valda
options-sun-moon = Sol och måne
    .facts =
        Solen på himlen om dagen, månen och stjärnorna om natten
        Månen visar uppdragets fas

## LJUD

options-section-volume = Volym
options-volume-master = Huvudvolym
    .facts = Alla ljud
options-volume-engine = Motor
    .facts = Din motor eller rotor, och flygplanen du hör omkring dig
options-volume-effects = Effekter
    .facts = Vapen, explosioner, landningsställ, radio och resten
options-volume-warnings = Varningar
    .facts = Stallvarning, radar- och robotvarningar, brandsirenen, Sidewinder-tonerna
options-section-cockpit = Cockpit
options-low-pass = Lågpassfilter för motor
    .facts =
        Inne i flygplanet låter motorn dämpad (kapellet och hjälmen tar bort diskanten)
        Av: samma ljud som utanför

## STYRNING

options-section-bindings = Tangentbindningar
options-keys-joystick = TANGENTER OCH JOYSTICK
    .facts =
        Koppla tangenter, joystick- och HOTAS-axlar och knappar
        Sparas till nästa flygningar
options-section-head-tracking = Huvud-/ögonspårning
options-head-tracking = Huvud-/ögonspårning
    .facts =
        Ditt huvud vrider och flyttar vyn i cockpit (på skärm; i VR gör headsetet det)
        Spårningsprogram (OpenTrack...): FreeTrack-utgång, eller UDP till 127.0.0.1 port 4242
        Home: huvudets nuvarande position blir mitten
options-tracking = Spårning ({ $source })
options-no-tracker = Ingen spårare hittad: starta spårningsprogrammet

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

lang-name-en = engelska
lang-name-bg = bulgariska
lang-name-zh-Hans = kinesiska (förenklad)
lang-name-hr = kroatiska
lang-name-cs = tjeckiska
lang-name-da = danska
lang-name-nl = nederländska
lang-name-et = estniska
lang-name-fi = finska
lang-name-fr = franska
lang-name-de = tyska
lang-name-el = grekiska
lang-name-hu = ungerska
lang-name-id = indonesiska
lang-name-ga = iriska
lang-name-it = italienska
lang-name-ja = japanska
lang-name-ko = koreanska
lang-name-lv = lettiska
lang-name-lt = litauiska
lang-name-mt = maltesiska
lang-name-pl = polska
lang-name-pt-BR = portugisiska (Brasilien)
lang-name-ro = rumänska
lang-name-ru = ryska
lang-name-sk = slovakiska
lang-name-sl = slovenska
lang-name-es = spanska
lang-name-sv = svenska
lang-name-uk = ukrainska
