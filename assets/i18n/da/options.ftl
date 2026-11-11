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
options-keep-time-accel = Behold tidsacceleration
    .facts =
        Accelerationen afbrydes ikke længere, når du bliver opdaget, affyrer et våben eller bliver skudt ned
        Kun TIME-knappen og Shift + Z / Shift + X ændrer den
options-tooltips = Cockpit-tooltips
    .facts = Hvad et cockpitinstrument eller en skærm er, under musen eller en VR-fingerspids
options-player-labels = Spillernavne
    .facts = Multiplayer: de andre piloters navne over deres fly, uanset afstand
options-label-size =
    .facts =
        Spillernavnenes størrelse
        50 %: standard
options-seat-height = Sædehøjde
    .facts =
        Hvor højt du sidder i cockpittet, på skærmen og i VR
        0 cm: flyets normale øjenhøjde
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
options-cloud-quality = Skyer
cloud-quality-auto = Auto
    .facts = Lav i VR, høj på skærmen
cloud-quality-low = Lav
    .facts = Grove skyer i en tredjedel af opløsningen (hurtigst)
cloud-quality-medium = Middel
    .facts = Halv opløsning, finere detaljer
cloud-quality-high = Høj
    .facts = Fuld opløsning, fineste detaljer og egen skygge (koster GPU-tid)

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

options-section-effects = Effekter
options-section-retro = Retro
effect-shake = Kamerarystelser
    .facts =
        Visningen ryster af kanonen, nære eksplosioner, stall, landinger og banen
        VR: cockpittet vibrerer, hovedet står stille
effect-flashes = Kampglimt
    .facts = Flak-sprængninger, eksplosioner og missilopsendelser lyser skyer, jord og cockpit op et øjeblik
effect-barrage = Natlig spærreild
    .facts =
        Konventionel krig om natten: byernes kanoner skyder blindt sporlys op i himlen, projektører fejer hen over dem
        Fjendtlige droner over en by fanges af dens projektører og drager ild fra dens kanoner
        Kun for udseendets skyld: spærringen rammer ingenting
effect-cockpit-night = Natbelysning i cockpittet
    .facts =
        Et svagt rødt lys i cockpittet efter mørkets frembrud (blågrønt under brillerne)
        Master caution pulserer rav over panelet, så længe en advarsel er aktiv
effect-nvg = Nattesynsbriller
    .facts =
        Kraftige lys svulmer til glorier, billedet bliver mere kornet i mørke
        Computer: brillernes runde synsfelt
effect-snow = Sne
    .facts = Over frysegrænsen giver storme sne i stedet for regn: lavt i en nordlig vinter, højt om sommeren
effect-mist = Jordtåge
    .facts =
        Moderne shaders: tåge ligger over lavt terræn og havet ved daggry, skumring og nat
        Tynd i ørkenerne, ingen i storm
effect-cloud-shadows = Skyskygger
    .facts = Moderne shaders: skyernes skygger på jorden nedenunder
effect-vapour = Damp og rotorvind
    .facts =
        Ved hårde manøvrer dampstriber fra vingespidserne
        En helikopter, der svæver lavt, hvirvler støv op, eller sprøjt over vand
        Køretøjer i bevægelse hvirvler støv op bag sig
effect-night-sky = Nattehimmel
    .facts =
        Af og til stjerneskud, de lave stjerner over horisonten blinker
        Nordlyset over Nordkap
effect-vga = VGA-farver
    .facts = 1989-udseendet med moderne skygning: 64 trin pr. farvekanal som på et VGA-kort, med dithering
effect-scanlines = Rasterlinjer
    .facts = En CRT-skærms rasterlinjer (ikke i VR)

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
effect-sunbeams = Solstråler
    .facts = Solstråler gennem hullerne i et brudt skydække
options-msaa-forced = Fra, så længe pixelering (retro) er til
options-volume-radio = Radio
    .facts = De andre spillere, der taler i koop-radioen, og den talte ATC
options-volume-atc = ATC-stemme
    .facts = De talte ATC-opkald, som en del af radioens lydstyrke
options-volume-music = Musik
    .facts = Hovedmenuens musik
options-section-atc = Talt ATC
options-section-radio = Koop-radio
options-push-to-talk = Tryk og tal
    .facts =
        Hold taletasten nede (Caps Lock eller en joystick- eller VR-knap tildelt PTT), og alle spillere hører dig
        Mikrofonen åbner første gang, du trykker
options-mic-gain = MIKROFONFORSTÆRKNING
    .facts =
        Hvor højt din mikrofon går ud
        100 %: som den er
options-mic-default = Standard
options-voice-text-only = kun tekst
options-voices-line = Flyveledere: { $controllers }   Pilot: { $pilot }
options-voices = STEMMER
    .facts =
        Hent stemmer til radioopkaldene, slet dem
        Vælg flyvelederes og din egen, og lyt til dem
options-voices-web = Kun tekst i browseren
options-tracking-mouse = Museblik ved tracking
    .facts =
        Til: højre museknap drejer og zoomer stadig cockpitvisningen
        Fra: mens trackeren sender, gør højre museknap ikke noget i cockpittet
options-atc = Flyveledelse
    .facts =
        Radioopkald fra tårnet, indflyvningen og GCA på de venlige flyvepladser
        Starttilladelse, overhead break og en talkdown ved lave skyer eller dårlig sigt
options-pixel-block = Blokstørrelse for pixelering
    .facts = Et bloks side, i skærmpixel (2: fin, 5: grov)
effect-pixelize = Pixelering
    .facts =
        Hele 3D-visningen (terræn, fly, bygninger, cockpit) i store firkantede pixel, som på en MS-DOS-pc; menuerne og tooltips forbliver skarpe (ikke i VR)
        Skyderen angiver et bloks størrelse, i skærmpixel (2: fin, 5: grov)
        Kantudjævning er slået fra, så længe den er til
        De tre kontakter under den lader HUD'et, cockpittet eller MFD'erne, instrumenterne og Kneeboard være skarpe
effect-sharp-hud = Uden HUD
    .facts =
        Pixelering: HUD'et (og hjelmdisplayet) tegnes skarpt oven på det pixelerede billede, så de tynde symboler forbliver læselige
effect-sharp-cockpit = Uden cockpit (knapper, kontakter...)
    .facts =
        Pixelering: cockpittets panel, ramme, sæde, knapper, kontakter, styrepind og gashåndtag tegnes skarpt
effect-sharp-gauges = Uden MFD'er, instrumenter og kneeboard
    .facts =
        Pixelering: MFD'erne, den kunstige horisont, instrumenterne, lamperne, udlæsningerne, mærkaterne og Kneeboard tegnes skarpt
options-needs-pixelize = Kræver Pixelering
