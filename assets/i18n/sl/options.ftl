# Okno MOŽNOSTI (src/options.rs) in njegove strani v meniju med letom.

options-title = MOŽNOSTI
options-tab-game = IGRA
options-tab-graphics = GRAFIKA
options-tab-sound = ZVOK
options-tab-controls = KRMILJENJE
options-footer = Shranjeno za naslednje lete; stikala realizma se izberejo ob vsaki igri

## IGRA

options-section-language = Jezik
options-section-classic-modern = Klasično / Sodobno
options-cockpit = Kabina
    .facts =
        Sodobno: zasloni visoke ločljivosti z gumbi ob okvirju
        Klasično: razporeditev instrumentne plošče iz dobe DOS in zasloni 77x60 (F-117A)
options-hud = HUD
    .facts =
        Sodobno: zeleni HUD visoke ločljivosti v slogu F-16 z načini NAV, AA in AG
        Klasično: HUD iz dobe DOS
options-avionics = Avionika
    .facts =
        Sodobno: formati MFD v slogu F-16 na obeh MFD (prikaz situacije, HARM, ciljniški pod, zemljevid, oborožitev) s premičnim kazalcem
        Klasično: zemljevid in TAC na levi, kamera, WAY, WPN in ORD na desni
options-flight-model = Letalski model
    .facts =
        Sodobno: aerodinamični letalski model z električnim krmiljenjem (fly-by-wire): palica določa hitrost nagiba in kotaljenja; spuščena obdrži smer nosu in tir leta
        Klasično: letalski model iz dobe DOS
options-section-flying = Letenje
options-auto-trim = Samodejni trim
    .facts =
        Pri spuščeni palici trim obdrži lego, v kateri ste jo pustili
        Helikopterji: nos se s hitrostjo ne dviguje več; letala: uravnovesi poškodovane krmilne ploskve
        Ročni trim: Alt + smerne tipke, Alt + Home ga centrira
options-tooltips = Tooltipi kabine
    .facts = Kaj je krmilni element ali zaslon v kabini, pod miško ali konico prsta v VR
options-section-vr-size = Velikost kabine v VR
options-vr-size =
    .facts =
        Velikost kabine (in sveta) v očalih
        100 %: naravna velikost

## GRAFIKA

options-section-rendering = Izris
options-shaders = Shaderji
    .facts =
        Sodobno: subtilno usmerjeno senčenje, mehka dimna zamegljenost v daljavi
        Klasično: ravno senčenje
options-shadows = Sence
    .facts =
        Sodobno: sončne sence v kabini in svetu (obremeni GPE)
        Klasično: brez senc
options-point-lights = Točkaste luči
    .facts =
        Sodobno: osvetlitev kabine in zaslonov; eksplozije, plameni in luči steze osvetlijo prizor
        Klasično: samo svetloba sonca in neba
options-time-of-day = Čas dneva
    .facts =
        Sodobno: sonce in luna se premikata z uro misije, nebo se s soncem temni in svetli; ponoči zvezde in lunine mene
        Sodobno: mesečina določa, kako daleč vas sovražni piloti vidijo ponoči (2 km v noči brez lune, 4 km pri polni luni)
        Klasično: počasno spreminjanje barv in negibno sonce
options-msaa = Glajenje robov (MSAA)
    .facts =
        Gladki robovi: 4x MSAA, 2x v VR (obremeni GPE)
        Izklopljeno: ostri robovi slikovnih pik
options-auto-detail = Samodejna podrobnost okolja (VR)
    .facts =
        V očalih se doseg mest, gozdov in polj skrči, ko sličice zamujajo, in se vrne, ko je časa dovolj
        Grafične nastavitve ostanejo, kot ste jih izbrali
options-sun-moon = Sonce in luna
    .facts =
        Podnevi sonce na nebu, ponoči luna in zvezde
        Luna kaže mena misije

## ZVOK

options-section-volume = Glasnost
options-volume-master = Glavna
    .facts = Vsi zvoki
options-volume-engine = Motor
    .facts = Vaš motor ali rotor in letala, ki jih slišite okoli sebe
options-volume-effects = Učinki
    .facts = Orožje, eksplozije, podvozje, radio in ostalo
options-volume-warnings = Opozorila
    .facts = Hupa zaradi izgube vzgona, radarska opozorila in opozorila pred izstrelki, sirena požara, toni Sidewinderja
options-section-cockpit = Kabina
options-low-pass = Nizki propust motorja
    .facts =
        V letalu je motor slišati zamolklo (kabina in čelada odrežeta visoke tone)
        Izklopljeno: enako kot zunaj

## KRMILJENJE

options-section-bindings = Dodelitve
options-keys-joystick = TIPKE IN JOYSTICK
    .facts =
        Tipke zemljevida, osi in gumbi joysticka in HOTAS
        Shranjeno za naslednje lete
options-section-head-tracking = Sledenje glavi/očem
options-head-tracking = Sledenje glavi/očem
    .facts =
        Vaša glava obrača in premika pogled v kabini (namizje; v VR to počnejo očala)
        Program za sledenje (OpenTrack...): izhod FreeTrack ali UDP na 127.0.0.1, vrata 4242
        Home: trenutni položaj glave postane središče
options-tracking = Sledenje ({ $source })
options-no-tracker = Sledilnika ni slišati: zaženite program za sledenje

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

lang-name-en = angleščina
lang-name-bg = bolgarščina
lang-name-zh-Hans = kitajščina (poenostavljena)
lang-name-hr = hrvaščina
lang-name-cs = češčina
lang-name-da = danščina
lang-name-nl = nizozemščina
lang-name-et = estonščina
lang-name-fi = finščina
lang-name-fr = francoščina
lang-name-de = nemščina
lang-name-el = grščina
lang-name-hu = madžarščina
lang-name-id = indonezijščina
lang-name-ga = irščina
lang-name-it = italijanščina
lang-name-ja = japonščina
lang-name-ko = korejščina
lang-name-lv = latvijščina
lang-name-lt = litovščina
lang-name-mt = malteščina
lang-name-pl = poljščina
lang-name-pt-BR = portugalščina (Brazilija)
lang-name-ro = romunščina
lang-name-ru = ruščina
lang-name-sk = slovaščina
lang-name-sl = slovenščina
lang-name-es = španščina
lang-name-sv = švedščina
lang-name-uk = ukrajinščina

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Jezik brskalnika: { $language }. Želite igrati v tem jeziku?
language-offer-yes = Da
language-offer-no = Ne
language-offer-never = Ne sprašuj več
