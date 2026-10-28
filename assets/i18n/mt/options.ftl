# It-tieqa OPZJONIJIET (src/options.rs) u l-paġni tagħha fil-menu waqt it-titjira.

options-title = OPZJONIJIET
options-tab-game = LGĦAB
options-tab-graphics = GRAFIKA
options-tab-sound = ŻUNA
options-tab-controls = KONTROLLI
options-footer = Maħżuna għat-titjiriet li jmiss; l-iswiċċijiet tar-realiżmu jintgħażlu ma' kull logħba

## LGĦAB

options-section-language = Lingwa
options-section-classic-modern = Klassiku / Modern
options-cockpit = Kokpit
    .facts =
        Modern: skrins b'riżoluzzjoni għolja bi buttuni madwar il-ħitan
        Klassiku: il-pannell tal-istrumenti tal-era tad-DOS u skrins 77x60 (F-117A)
options-hud = HUD
    .facts =
        Modern: HUD stil F-16 b'riżoluzzjoni għolja bl-aħdar bil-modi NAV, AA u AG
        Klassiku: il-HUD tal-era tad-DOS
options-avionics = Avjonika
    .facts =
        Modern: formati MFD stil F-16 fuq kull MFD (situation display, HARM, targeting pod, mappa, stores) b'kursur li jinżamm
        Klassiku: mappa u TAC fuq ix-xellug, kamera, WAY, WPN u ORD fuq il-lemin
options-flight-model = Mudell tat-titjir
    .facts =
        Modern: mudell tat-titjir aerodinamiku bi fly-by-wire: l-istick jikkmanda r-rata tal-pitch u tar-roll; meta jitlaq, il-għonq jibqa' fil-pożizzjoni u t-trajettorja tinżamm
        Klassiku: mudell tat-titjir tal-era tad-DOS
options-section-flying = Titjir
options-auto-trim = Auto trim
    .facts =
        Bl-istick maħlul, it-trim iżomm l-assett fejn halltu
        Ħelikopters: l-għonq ma jibqax jogħla mal-veloċità; ajruplani: it-trim jikkumpensa s-superfiċi tal-kontroll bil-ħsara
        Trim manwali: Alt + vleġeġ, Alt + Home iċċentrah
options-tooltips = Tooltips tal-kokpit
    .facts = X'inhi kontroll jew display tal-kokpit, taħt il-maws jew ras is-swaba' f'VR
options-section-vr-size = Daqs tal-kokpit f'VR
options-vr-size =
    .facts =
        Id-daqs tal-kokpit (u tad-dinja) fil-headset
        100%: daqs naturali

## GRAFIKA

options-section-rendering = Rendering
options-shaders = Shaders
    .facts =
        Modern: dell direzzjonali sottili, ċpar bil-bogħod bla xkiel
        Klassiku: dell ċatt
options-shadows = Dellijiet
    .facts =
        Modern: dellijiet tax-xemx fil-kokpit u fid-dinja (jiswa ħin tal-GPU)
        Klassiku: l-ebda dell
options-point-lights = Point lights
    .facts =
        Modern: dwal tal-kokpit u tad-displays; splużjonijiet, dħaħen u dwal tal-pista jdawlu x-xena
        Klassiku: dawl tax-xemx u tas-sema biss
options-time-of-day = Ħin tal-jum
    .facts =
        Modern: ix-xemx u l-qamar jimxu mal-arloġġ tal-missjoni, is-sema jdallam u jiddi mal-xemx; stilel u fażi tal-qamar bil-lejl
        Modern: id-dawl tal-qamar jiddeċiedi sa fejn jarawk il-piloti tal-għadu bil-lejl (2 km f'lejl mingħajr qamar, 4 km bil-qamar sħiħ)
        Klassiku: bidla bil-mod fil-kuluri u xemx fissa
options-msaa = Anti-aliasing (MSAA)
    .facts =
        Truf lixxi: 4x MSAA, 2x f'VR (jiswa ħin tal-GPU)
        Off: truf tal-pixels iebsin
options-auto-detail = Detall awtomatiku tax-xena (VR)
    .facts =
        Fil-headset, il-medda tal-bliet, tal-foresti u tal-għelieqi tonqos meta l-frames jagħmlu dewmien u terġa' tikber meta jkun hemm ħin żejjed
        L-issettjar grafiku jibqa' kif intgħażel
options-sun-moon = Xemx u qamar
    .facts =
        Ix-xemx fis-sema bin-nhar, il-qamar u l-istilel bil-lejl
        Il-qamar juri l-fażi tal-missjoni

## ŻUNA

options-section-volume = Volum
options-volume-master = Ġenerali
    .facts = Kull ħoss
options-volume-engine = Mutur
    .facts = Il-mutur jew ir-rotor tiegħek, u l-ajruplani li tisma' madwarek
options-volume-effects = Effetti
    .facts = Armi, splużjonijiet, landing gear, radju u l-bqija
options-volume-warnings = Allarmi
    .facts = Il-qarn tal-stall, twissijiet tar-radar u tal-missili, is-sirena tan-nar, it-toni tas-Sidewinder
options-section-cockpit = Kokpit
options-low-pass = Low-pass tal-mutur
    .facts =
        Ġewwa l-ajruplan il-mutur jinstema' mitmum (il-canopy u l-ħelmet jiżolaw il-ħsejjes għoljin)
        Off: l-istess bħal barra

## KONTROLLI

options-section-bindings = Assenjazzjonijiet
options-keys-joystick = TASTI U JOYSTICK
    .facts =
        Tasti tal-mappa, joystick u assi u buttuni tal-HOTAS
        Maħżuna għat-titjiriet li jmiss
options-section-head-tracking = Head/eye tracking
options-head-tracking = Head/Eye Tracking
    .facts =
        Rasek iddur u l-veduta tiċċaqlaq fil-kokpit (desktop; f'VR jagħmel dan il-headset)
        Programm ta' tracking (OpenTrack...): output FreeTrack, jew UDP lil 127.0.0.1 port 4242
        Home: il-pożizzjoni attwali tar-ras issir iċ-ċentru
options-tracking = Tracking ({ $source })
options-no-tracker = L-ebda tracker ma nstema': ibda l-programm ta' tracking

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
