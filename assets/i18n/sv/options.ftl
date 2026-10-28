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
