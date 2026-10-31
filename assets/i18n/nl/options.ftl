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
effect-vapour = Damp en rotorwind
    .facts =
        Bij scherpe manoeuvres dampsporen van de vleugeltippen
        Een laag zwevende helikopter blaast stof op, of nevel boven water
        Rijdende voertuigen trekken een stofwolk achter zich
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
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
