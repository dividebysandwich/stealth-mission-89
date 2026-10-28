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
options-section-vr-size = Cockpitgrootte in VR
options-vr-size =
    .facts =
        Grootte van de cockpit (en de wereld) in de headset
        100%: ware grootte

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
