# The OPTIONS window (src/options.rs) and its pages in the in-flight menu.

options-title = OPTIONS
options-tab-game = GAME
options-tab-graphics = GRAPHICS
options-tab-sound = SOUND
options-tab-controls = CONTROLS
options-footer = Saved for the next flights; the realism switches are chosen with each game

## GAME

options-section-language = Language
options-section-classic-modern = Classic / Modern
options-cockpit = Cockpit
    .facts =
        Modern: high-resolution displays with bezel buttons
        Classic: the DOS-era instrument panel layout and 77x60 displays (F-117A)
options-hud = HUD
    .facts =
        Modern: high-resolution F-16 style HUD in green with NAV, AA and AG modes
        Classic: the DOS-era HUD
options-avionics = Avionics
    .facts =
        Modern: F-16 style MFD formats on either MFD (situation display, HARM, targeting pod, map, stores) with a slewable cursor
        Classic: map and TAC left, camera, WAY, WPN and ORD right
options-flight-model = Flight model
    .facts =
        Modern: aerodynamic flight model with fly-by-wire: the stick commands pitch and roll rate; released, the nose stays put and the flight path is held
        Classic: a DOS-era flight model
options-section-flying = Flying
options-auto-trim = Auto trim
    .facts =
        With the stick released, the trim holds the attitude where you left it
        Helicopters: no more nose rising with speed; jets: trims out damaged control surfaces
        Manual trim: Alt + arrow keys, Alt + Home centres it
options-tooltips = Cockpit tooltips
    .facts = What a cockpit control or display is, under the mouse or a VR fingertip
options-section-vr-size = VR cockpit size
options-vr-size =
    .facts =
        Size of the cockpit (and the world) in the headset
        100%: life size

## GRAPHICS

options-section-rendering = Rendering
options-shaders = Shaders
    .facts =
        Modern: subtle directional shading, smooth distance haze
        Classic: flat shading
options-shadows = Shadows
    .facts =
        Modern: sun shadows in the cockpit and the world (costs GPU time)
        Classic: no shadows
options-point-lights = Point lights
    .facts =
        Modern: cockpit flood and display lights; explosions, plumes and runway lights light the scene
        Classic: sun and sky light only
options-time-of-day = Time of day
    .facts =
        Modern: the sun and the moon move with the mission clock, the sky darkens and brightens with the sun; stars and the moon's phase at night
        Modern: moonlight sets how far enemy pilots see you at night (2 km on a moonless night, 4 km under a full moon)
        Classic: a slow change of colours and a fixed sun
options-msaa = Anti-aliasing (MSAA)
    .facts =
        Smooth edges: 4x MSAA, 2x in VR (costs GPU time)
        Off: hard pixel edges
options-auto-detail = Automatic scenery detail (VR)
    .facts =
        In the headset, the range of towns, forests and fields shrinks while frames run late and grows back when there is time to spare
        The graphics settings stay as chosen
options-sun-moon = Sun and moon
    .facts =
        The sun in the sky by day, the moon and the stars by night
        The moon shows the mission's phase

## SOUND

options-section-volume = Volume
options-volume-master = Master
    .facts = Every sound
options-volume-engine = Engine
    .facts = Your engine or rotor, and the aircraft heard around you
options-volume-effects = Effects
    .facts = Weapons, explosions, gear, radio and the rest
options-volume-warnings = Warnings
    .facts = Stall horn, radar and missile warnings, the fire siren, the Sidewinder tones
options-section-cockpit = Cockpit
options-low-pass = Engine low-pass
    .facts =
        Inside the aircraft the engine sounds muffled (the canopy and helmet take the highs off)
        Off: the same as outside

## CONTROLS

options-section-bindings = Bindings
options-keys-joystick = KEYS AND JOYSTICK
    .facts =
        Map keys, joystick and HOTAS axes and buttons
        Saved for the next flights
options-section-head-tracking = Head/eye tracking
options-head-tracking = Head/Eye Tracking
    .facts =
        Your head turns and moves the view in the cockpit (desktop; in VR the headset does)
        Tracking program (OpenTrack...): FreeTrack output, or UDP to 127.0.0.1 port 4242
        Home: the current head position becomes the centre
options-tracking = Tracking ({ $source })
options-no-tracker = No tracker heard: start the tracking program

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

lang-name-en = English
lang-name-bg = Bulgarian
lang-name-zh-Hans = Chinese (Simplified)
lang-name-hr = Croatian
lang-name-cs = Czech
lang-name-da = Danish
lang-name-nl = Dutch
lang-name-et = Estonian
lang-name-fi = Finnish
lang-name-fr = French
lang-name-de = German
lang-name-el = Greek
lang-name-hu = Hungarian
lang-name-id = Indonesian
lang-name-ga = Irish
lang-name-it = Italian
lang-name-ja = Japanese
lang-name-ko = Korean
lang-name-lv = Latvian
lang-name-lt = Lithuanian
lang-name-mt = Maltese
lang-name-pl = Polish
lang-name-pt-BR = Portuguese (Brazil)
lang-name-ro = Romanian
lang-name-ru = Russian
lang-name-sk = Slovak
lang-name-sl = Slovenian
lang-name-es = Spanish
lang-name-sv = Swedish
lang-name-uk = Ukrainian

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Browser language: { $language }. Play the game in this language?
language-offer-yes = Yes
language-offer-no = No
language-offer-never = Do not ask again
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
