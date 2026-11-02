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
options-player-labels = Spelarnamn
    .facts = Flerspelare: de andra pilotternas namn över deras flygplan, oavsett avstånd
options-label-size =
    .facts =
        Spelarnamnens storlek
        50 %: standard
options-section-vr-size = Cockpitens storlek i VR
options-vr-size =
    .facts =
        Cockpitens (och världens) storlek i headsetet
        100 %: naturlig storlek
options-section-gif = GIF-inspelning
options-gif-size = STORLEK
options-gif-rate = BILDER
options-gif-scale-100 =
    .facts = Fönstrets fulla upplösning: stora filer
options-gif-scale-25 =
    .facts = Små filer
options-gif-rate-50 =
    .facts = Bara när spelet går i 100 fps eller mer (varannan bild fångas)
options-gif-hint = F12 (om den inte bundits om) startar och stoppar; sparas i mappen Bilder

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
options-volume-rwr = RWR
    .facts = Radarvarningstonerna, som en del av varningarnas volym
options-section-rwr-tones = Radarvarningstoner
rwr-tones-all = ALLA
    .facts = En radar som belyser dig, varje svep som upptäcker dig, robotavfyrningar
rwr-tones-detect = BARA UPPTÄCKT
    .facts =
        En ton när en radar upptäcker dig (igen efter att radarerna släppt dig en stund)
        Och robotavfyrningar
rwr-tones-launch = BARA AVFYRNINGAR
    .facts = Bara robotar som avfyras mot dig
rwr-tones-off = AV
    .facts = Inga radarvarningstoner (lampor och skärmar varnar fortfarande)
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

options-section-effects = Effekter
options-section-retro = Retro
effect-shake = Kameraskakning
    .facts =
        Vyn skakar av kanonen, närliggande explosioner, överstegring, landningar och banan
        VR: cockpit vibrerar, huvudet står stilla
effect-flashes = Stridsblixtar
    .facts = Luftvärnssprängningar, explosioner och robotuppskjutningar lyser upp molnen, marken och cockpit ett ögonblick
effect-barrage = Nattlig spärreld
    .facts =
        Konventionellt krig på natten: städernas kanoner skjuter spårljus blint mot himlen, strålkastare sveper över dem
        Fientliga drönare över en stad fångas av dess strålkastare och drar till sig eld från dess kanoner
        Bara för utseendet: spärren träffar ingenting
effect-cockpit-night = Nattbelysning i cockpit
    .facts =
        Ett svagt rött ljus i cockpit efter mörkrets inbrott (blågrönt under kikaren)
        Master caution pulserar bärnstensfärgat över panelen så länge en varning är aktiv
effect-nvg = Nattkikare
    .facts =
        Starka ljus sväller till ljushalos, bilden blir kornigare i mörker
        Dator: kikarens runda synfält
effect-snow = Snö
    .facts = Över nollgradsgränsen ger oväder snö i stället för regn: lågt i en nordlig vinter, högt på sommaren
effect-mist = Marksdimma
    .facts =
        Moderna shaders: dimma ligger över lågland och hav i gryning, skymning och natt
        Tunn i öknarna, ingen i storm
effect-cloud-shadows = Molnskuggor
    .facts = Moderna shaders: molnens skuggor på marken nedanför
effect-vapour = Ånga och rotorvind
    .facts =
        Vid hårda svängar kondensstrimmor från vingspetsarna
        En helikopter som hovrar lågt virvlar upp damm, eller vattenstänk över vatten
        Fordon i rörelse rör upp damm bakom sig
effect-night-sky = Natthimmel
    .facts =
        Då och då stjärnfall, de låga stjärnorna över horisonten tindrar
        Norrskenet över Nordkap
effect-vga = VGA-färger
    .facts = 1989 års utseende med modern skuggning: 64 steg per färgkanal som på ett VGA-kort, med dithering
effect-scanlines = Rasterlinjer
    .facts = En CRT-skärms rasterlinjer (inte i VR)

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

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Webbläsarens språk: { $language }. Vill du spela på det här språket?
language-offer-yes = Ja
language-offer-no = Nej
language-offer-never = Fråga inte igen
effect-sunbeams = Solstrålar
    .facts = Solljusstrålar genom luckorna i ett brutet molntäcke
options-msaa-forced = Av så länge pixelering (retro) är på
options-volume-radio = Radio
    .facts = De andra spelarna som pratar i koop-radion och den talade ATC
options-volume-atc = ATC-röst
    .facts = De talade ATC-anropen, som en del av radions volym
options-volume-music = Musik
    .facts = Huvudmenyns musik
options-section-atc = Talad ATC
options-section-radio = Koop-radio
options-push-to-talk = Tryck och prata
    .facts =
        Håll nere pratknappen (Caps Lock, eller en joystick- eller VR-knapp kopplad till PTT) så hör alla spelare dig
        Mikrofonen öppnas första gången du trycker
options-mic-gain = MIKFÖRSTÄRKNING
    .facts =
        Hur högt din mikrofon går ut
        100 %: som den är
options-mic-default = Standard
options-voice-text-only = endast text
options-voices-line = Flygledare: { $controllers }   Pilot: { $pilot }
options-voices = RÖSTER
    .facts =
        Ladda ner röster till radioanropen, radera dem
        Välj flygledarnas och din egen och lyssna på dem
options-voices-web = Endast text i webbläsaren
options-tracking-mouse = Musblick vid tracking
    .facts =
        På: höger musknapp vrider och zoomar fortfarande cockpitvyn
        Av: medan trackern sänder gör höger musknapp ingenting i cockpiten
options-atc = Flygledning
    .facts =
        Radioanrop från tornet, inflygningen och GCA på de vänliga flygfälten
        Starttillstånd, overhead break och en talkdown vid låga moln eller dålig sikt
options-pixel-block = Blockstorlek för pixelering
    .facts = Ett blocks sida, i skärmpixlar (2: fin, 5: grov)
effect-pixelize = Pixelering
    .facts =
        Hela 3D-vyn (terräng, flygplan, byggnader, cockpit) i stora fyrkantiga pixlar, som på en MS-DOS-dator; menyerna och tooltips förblir skarpa (inte i VR)
        Skjutreglaget anger ett blocks storlek, i skärmpixlar (2: fin, 5: grov)
        Kantutjämningen är av så länge den är på
