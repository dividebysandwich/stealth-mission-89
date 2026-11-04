# Het venster OPTIES (src/options.rs) en zijn pagina's in het vluchtmenu.

options-title = OPTIES
options-tab-game = SPEL
options-tab-graphics = GRAFISCH
options-tab-sound = GELUID
options-tab-controls = BESTURING
options-footer = Bewaard voor de volgende vluchten; de realisme-instellingen kies je bij elk spel

## SPEL

options-section-language = Taal
options-section-classic-modern = Klassiek / Modern
options-cockpit = Cockpit
    .facts =
        Modern: displays met hoge resolutie en bezelknoppen
        Klassiek: de indeling van het instrumentenpaneel uit het DOS-tijdperk en 77x60-displays (F-117A)
options-hud = HUD
    .facts =
        Modern: groene HUD met hoge resolutie in F-16-stijl, met NAV-, AA- en AG-modi
        Klassiek: de HUD uit het DOS-tijdperk
options-avionics = Avionica
    .facts =
        Modern: MFD-formaten in F-16-stijl op elke MFD (situatiedisplay, HARM, targeting pod, kaart, stores) met een verschuifbare cursor
        Klassiek: kaart en TAC links, camera, WAY, WPN en ORD rechts
options-flight-model = Vliegmodel
    .facts =
        Modern: aerodynamisch vliegmodel met fly-by-wire: de stick stuurt de stampsnelheid en rolsnelheid aan; losgelaten blijft de neus staan en wordt het vliegpad aangehouden
        Klassiek: een vliegmodel uit het DOS-tijdperk
options-section-flying = Vliegen
options-auto-trim = Automatische trim
    .facts =
        Met losgelaten stick houdt de trim de stand aan waarin je hem hebt achtergelaten
        Helikopters: de neus komt niet meer omhoog met de snelheid; jets: trimt beschadigde stuurvlakken uit
        Handmatige trim: Alt + pijltjestoetsen, Alt + Home centreert hem
options-tooltips = Cockpit-tooltips
    .facts = Wat een bedieningselement of display in de cockpit is, onder de muis of een vingertop in VR
options-player-labels = Spelersnamen
    .facts = Multiplayer: de namen van de andere piloten boven hun toestel, op elke afstand
options-label-size =
    .facts =
        Grootte van de spelersnamen
        50%: standaard
options-seat-height = Stoelhoogte
    .facts =
        Hoe hoog je in de cockpit zit, op het scherm en in VR
        0 cm: de normale ooghoogte van het toestel
options-section-vr-size = Cockpitgrootte in VR
options-vr-size =
    .facts =
        Grootte van de cockpit (en de wereld) in de headset
        100%: ware grootte
options-section-gif = GIF-opname
options-gif-size = GROOTTE
options-gif-rate = BEELDEN
options-gif-scale-100 =
    .facts = De volle resolutie van het venster: grote bestanden
options-gif-scale-25 =
    .facts = Kleine bestanden
options-gif-rate-50 =
    .facts = Alleen als het spel op 100 fps of meer draait (elk tweede beeld wordt vastgelegd)
options-gif-hint = F12 (tenzij anders toegewezen) start en stopt; opgeslagen in uw map Afbeeldingen

## GRAFISCH

options-section-rendering = Weergave
options-shaders = Shaders
    .facts =
        Modern: subtiele richtingsschaduw, zachte afstandsnevel
        Klassiek: vlakke schaduw
options-shadows = Schaduwen
    .facts =
        Modern: zonneschaduwen in de cockpit en de wereld (kost GPU-tijd)
        Klassiek: geen schaduwen
options-point-lights = Puntlichten
    .facts =
        Modern: cockpit- en displayverlichting; explosies, pluimen en baanlichten verlichten het tafereel
        Klassiek: alleen zon- en hemellicht
options-time-of-day = Tijdstip van de dag
    .facts =
        Modern: de zon en de maan volgen de missieklok, de lucht wordt donkerder en lichter met de zon; sterren en maanfase 's nachts
        Modern: het maanlicht bepaalt hoe ver vijandelijke piloten je 's nachts zien (2 km zonder maan, 4 km bij volle maan)
        Klassiek: een trage kleurverandering en een vaste zon
options-msaa = Anti-aliasing (MSAA)
    .facts =
        Gladde randen: 4x MSAA, 2x in VR (kost GPU-tijd)
        Uit: harde pixelranden
options-auto-detail = Automatisch landschapsdetail (VR)
    .facts =
        In de headset wordt het bereik van steden, bossen en velden kleiner zolang beelden te laat komen en groeit het terug als er tijd over is
        De grafische instellingen blijven zoals gekozen
options-sun-moon = Zon en maan
    .facts =
        De zon aan de hemel overdag, de maan en de sterren 's nachts
        De maan toont de fase van de missie
options-cloud-quality = Wolken
cloud-quality-auto = Auto
    .facts = Laag in VR, hoog op het scherm
cloud-quality-low = Laag
    .facts = Grove wolken op een derde van de resolutie (snelst)
cloud-quality-medium = Middel
    .facts = Halve resolutie, fijnere details
cloud-quality-high = Hoog
    .facts = Volledige resolutie, fijnste details en eigen schaduw (kost GPU-tijd)

## GELUID

options-section-volume = Volume
options-volume-master = Algemeen
    .facts = Al het geluid
options-volume-engine = Motor
    .facts = Je motor of rotor, en de vliegtuigen die je om je heen hoort
options-volume-effects = Effecten
    .facts = Wapens, explosies, landingsgestel, radio en de rest
options-volume-warnings = Waarschuwingen
    .facts = Stalhoorn, radar- en raketwaarschuwingen, de brandsirene, de Sidewinder-tonen
options-volume-rwr = RWR
    .facts = De radarwaarschuwingstonen, als deel van het volume van de waarschuwingen
options-section-rwr-tones = Radarwaarschuwingstonen
rwr-tones-all = ALLE
    .facts = Een radar die u belicht, elke zwaai die u detecteert, raketlanceringen
rwr-tones-detect = ALLEEN DETECTIE
    .facts =
        Eén toon wanneer een radar u detecteert (opnieuw nadat de radars u een tijd kwijt waren)
        En raketlanceringen
rwr-tones-launch = ALLEEN LANCERINGEN
    .facts = Alleen raketten die op u worden afgevuurd
rwr-tones-off = UIT
    .facts = Geen radarwaarschuwingstonen (de lampjes en schermen waarschuwen nog steeds)
options-section-cockpit = Cockpit
options-low-pass = Laagdoorlaatfilter motor
    .facts =
        In het vliegtuig klinkt de motor gedempt (de kap en de helm halen de hoge tonen weg)
        Uit: hetzelfde als buiten

## BESTURING

options-section-bindings = Toewijzingen
options-keys-joystick = TOETSEN EN JOYSTICK
    .facts =
        Toetsen, joystick- en HOTAS-assen en -knoppen toewijzen
        Bewaard voor de volgende vluchten
options-section-head-tracking = Hoofd-/oogtracking
options-head-tracking = Hoofd-/oogtracking
    .facts =
        Je hoofd draait en verplaatst het beeld in de cockpit (op het scherm; in VR doet de headset dat)
        Trackingprogramma (OpenTrack...): FreeTrack-uitvoer, of UDP naar 127.0.0.1 poort 4242
        Home: de huidige hoofdpositie wordt het midden
options-tracking = Tracking actief ({ $source })
options-no-tracker = Geen tracker gevonden: start het trackingprogramma

## EFFECTS

options-section-effects = Effecten
options-section-retro = Retro
effect-shake = Camerabeving
    .facts =
        Het beeld beeft bij het kanon, nabije explosies, overtrek, landingen en de baan
        VR: de cockpit trilt, uw hoofd blijft stil
effect-flashes = Gevechtsflitsen
    .facts = Flakexplosies, ontploffingen en raketlanceringen verlichten even de wolken, de grond en de cockpit
effect-barrage = Nachtelijk spervuur
    .facts =
        Conventionele oorlog bij nacht: de kanonnen van de steden schieten blind lichtspoor de lucht in, zoeklichten vegen erover
        Vijandelijke drones boven een stad worden door de zoeklichten gevangen en trekken het vuur van de kanonnen aan
        Alleen voor de show: het spervuur raakt niets
effect-cockpit-night = Cockpitverlichting bij nacht
    .facts =
        Een zwak rood schijnsel in de cockpit na het donker worden (blauwgroen onder de bril)
        De master caution pulseert amber boven het paneel zolang een waarschuwing actief is
effect-nvg = Nachtzichtbril
    .facts =
        Felle lichten lopen uit tot halo's, het beeld korrelt meer in het donker
        Desktop: het ronde gezichtsveld van de bril
effect-snow = Sneeuw
    .facts = Boven het vriespunt sneeuwt het bij onweer in plaats van te regenen: laag in een noordelijke winter, hoog in de zomer
effect-mist = Grondmist
    .facts =
        Moderne shaders: mist ligt bij dageraad, schemering en nacht over het lage land en de zee
        Ijl in de woestijnen, geen bij storm
effect-cloud-shadows = Wolkenschaduwen
    .facts = Moderne shaders: de schaduwen van de wolken op de grond eronder
effect-vapour = Damp en rotorwind
    .facts =
        Bij scherpe manoeuvres dampsporen van de vleugeltippen
        Een laag zwevende helikopter blaast stof op, of nevel boven water
        Rijdende voertuigen trekken een stofwolk achter zich
effect-night-sky = Nachtelijke hemel
    .facts =
        Af en toe vallende sterren, de lage sterren boven de horizon fonkelen
        Het noorderlicht boven de Noordkaap
effect-vga = VGA-kleuren
    .facts = Het uiterlijk van 1989 met moderne schaduwing: 64 niveaus per kleurkanaal, als van een VGA-kaart, met dithering
effect-scanlines = Scanlines
    .facts = De scanlines van een CRT-monitor (niet in VR)

## LANGUAGE NAMES

lang-name-en = Engels
lang-name-bg = Bulgaars
lang-name-zh-Hans = Chinees (vereenvoudigd)
lang-name-hr = Kroatisch
lang-name-cs = Tsjechisch
lang-name-da = Deens
lang-name-nl = Nederlands
lang-name-et = Estisch
lang-name-fi = Fins
lang-name-fr = Frans
lang-name-de = Duits
lang-name-el = Grieks
lang-name-hu = Hongaars
lang-name-id = Indonesisch
lang-name-ga = Iers
lang-name-it = Italiaans
lang-name-ja = Japans
lang-name-ko = Koreaans
lang-name-lv = Lets
lang-name-lt = Litouws
lang-name-mt = Maltees
lang-name-pl = Pools
lang-name-pt-BR = Portugees (Brazilië)
lang-name-ro = Roemeens
lang-name-ru = Russisch
lang-name-sk = Slowaaks
lang-name-sl = Sloveens
lang-name-es = Spaans
lang-name-sv = Zweeds
lang-name-uk = Oekraïens

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Taal van de browser: { $language }. Het spel in deze taal spelen?
language-offer-yes = Ja
language-offer-no = Nee
language-offer-never = Niet meer vragen
effect-sunbeams = Zonnestralen
    .facts = Bundels zonlicht door de gaten van een gebroken bewolking
options-msaa-forced = Uit zolang pixeleren (retro) aan staat
options-volume-radio = Radio
    .facts = De andere spelers die op de coop-radio praten, en de gesproken ATC
options-volume-atc = ATC-stem
    .facts = De gesproken ATC-oproepen, als deel van het radiovolume
options-volume-music = Muziek
    .facts = De muziek van het hoofdmenu
options-section-atc = Gesproken ATC
options-section-radio = Coop-radio
options-push-to-talk = Push-to-talk
    .facts =
        Houd de spreektoets ingedrukt (Caps Lock, of een joystick- of VR-knop die aan PTT is gekoppeld) en alle spelers horen u
        De microfoon gaat open de eerste keer dat u indrukt
options-mic-gain = MIC-GAIN
    .facts =
        Hoe luid uw microfoon uitgaat
        100%: zoals het is
options-mic-default = Standaard
options-voice-text-only = alleen tekst
options-voices-line = Verkeersleiders: { $controllers }   Piloot: { $pilot }
options-voices = STEMMEN
    .facts =
        Stemmen voor de radio-oproepen downloaden, verwijderen
        Die van de verkeersleiders en uw eigen kiezen, en beluisteren
options-voices-web = Alleen tekst in de browser
options-tracking-mouse = Muisblik bij tracking
    .facts =
        Aan: de rechtermuisknop draait en zoomt de cockpitweergave nog steeds
        Uit: zolang de tracker zendt, doet de rechtermuisknop niets in de cockpit
options-atc = Luchtverkeersleiding
    .facts =
        Radio-oproepen van toren, nadering en GCA op de eigen vliegvelden
        Startvergunning, overhead break en een talkdown bij lage wolken of slecht zicht
options-pixel-block = Blokgrootte van het pixeleren
    .facts = De zijde van een blok, in schermpixels (2: fijn, 5: grof)
effect-pixelize = Pixeleren
    .facts =
        De hele 3D-weergave (terrein, vliegtuigen, gebouwen, cockpit) in grote vierkante pixels, zoals op een MS-DOS-pc; de menu's en tooltips blijven scherp (niet in VR)
        De schuifregelaar stelt de grootte van een blok in, in schermpixels (2: fijn, 5: grof)
        Anti-aliasing is uit zolang het aan staat
        De drie schakelaars eronder laten de HUD, de cockpit of de MFD's, instrumenten en het Kneeboard scherp
effect-sharp-hud = Zonder HUD
    .facts =
        Pixeleren: De HUD (en het helmdisplay) wordt scherp getekend boven het gepixelde beeld, zodat de dunne symbolen leesbaar blijven
effect-sharp-cockpit = Zonder cockpit (knoppen, schakelaars...)
    .facts =
        Pixeleren: Paneel, frame, stoel, knoppen, schakelaars, stuurknuppel en gashendel van de cockpit worden scherp getekend
effect-sharp-gauges = Zonder MFD's, instrumenten en Kneeboard
    .facts =
        Pixeleren: De MFD's, de kunstmatige horizon, de instrumenten, de lampjes, de uitlezingen, de labels en het Kneeboard worden scherp getekend
options-needs-pixelize = Vereist Pixeleren
