# Prozor OPCIJE (src/options.rs) i njegove stranice u izborniku tijekom leta.

options-title = OPCIJE
options-tab-game = IGRA
options-tab-graphics = GRAFIKA
options-tab-sound = ZVUK
options-tab-controls = UPRAVLJANJE
options-footer = Spremljeno za sljedeće letove; prekidači realizma biraju se uz svaku igru

## IGRA

options-section-language = Jezik
options-section-classic-modern = Klasično / Moderno
options-cockpit = Kokpit
    .facts =
        Moderno: zasloni visoke razlučivosti s tipkama na rubu
        Klasično: raspored instrumentne ploče iz DOS-ova doba i zasloni 77x60 (F-117A)
options-hud = HUD
    .facts =
        Moderno: zeleni HUD visoke razlučivosti u stilu F-16 s načinima NAV, AA i AG
        Klasično: HUD iz DOS-ova doba
options-avionics = Avionika
    .facts =
        Moderno: MFD formati u stilu F-16 na oba MFD-a (prikaz situacije, HARM, ciljnički pod, karta, naoružanje) s pomičnim kursorom
        Klasično: karta i TAC lijevo, kamera, WAY, WPN i ORD desno
options-flight-model = Model leta
    .facts =
        Moderno: aerodinamički model leta s električnim upravljanjem (fly-by-wire): palica zapovijeda brzinu propinjanja i valjanja; otpuštena, nos ostaje na mjestu i putanja se drži
        Klasično: model leta iz DOS-ova doba
options-section-flying = Pilotiranje
options-auto-trim = Automatski trim
    .facts =
        S otpuštenom palicom trim drži stav u kojem ste je ostavili
        Helikopteri: nos se više ne diže s brzinom; mlažnjaci: kompenzira oštećene upravljačke površine
        Ručni trim: Alt + strelice, Alt + Home ga centrira
options-tooltips = Tooltipi kokpita
    .facts = Što je pojedina komanda ili zaslon u kokpitu, pod mišem ili vrhom prsta u VR-u
options-player-labels = Imena igrača
    .facts = Više igrača: imena drugih pilota iznad njihovih zrakoplova, na bilo kojoj udaljenosti
options-label-size =
    .facts =
        Veličina imena igrača
        50 %: zadano
options-section-vr-size = Veličina kokpita u VR-u
options-vr-size =
    .facts =
        Veličina kokpita (i svijeta) u headsetu
        100 %: prirodna veličina
options-section-gif = Snimanje GIF-a
options-gif-size = VELIČINA
options-gif-rate = SLIKE
options-gif-scale-100 =
    .facts = Puna razlučivost prozora: velike datoteke
options-gif-scale-25 =
    .facts = Male datoteke
options-gif-rate-50 =
    .facts = Samo kad igra radi na 100 fps ili više (snima se svaka druga slika)
options-gif-hint = F12 (osim ako je preraspoređen) pokreće i zaustavlja; sprema se u mapu Slike

## GRAFIKA

options-section-rendering = Iscrtavanje
options-shaders = Shaderi
    .facts =
        Moderno: suptilno usmjereno sjenčanje, blaga izmaglica na daljinu
        Klasično: ravno sjenčanje
options-shadows = Sjene
    .facts =
        Moderno: sunčeve sjene u kokpitu i svijetu (troši GPU)
        Klasično: bez sjena
options-point-lights = Točkasta svjetla
    .facts =
        Moderno: rasvjeta kokpita i zaslona; eksplozije, stupovi dima i svjetla piste osvjetljavaju scenu
        Klasično: samo svjetlo Sunca i neba
options-time-of-day = Doba dana
    .facts =
        Moderno: Sunce i Mjesec prate sat misije, nebo tamni i svijetli sa Suncem; zvijezde i mjesečeva mijena noću
        Moderno: mjesečina određuje s koje vas daljine neprijateljski piloti vide noću (2 km u noći bez Mjeseca, 4 km pod uštapom)
        Klasično: spora promjena boja i nepomično Sunce
options-msaa = Izglađivanje rubova (MSAA)
    .facts =
        Glatki rubovi: 4x MSAA, 2x u VR-u (troši GPU)
        Isključeno: oštri rubovi piksela
options-auto-detail = Automatski detalji okoliša (VR)
    .facts =
        U headsetu se domet gradova, šuma i polja smanjuje dok sličice kasne i vraća kad ima viška vremena
        Grafičke postavke ostaju kako su odabrane
options-sun-moon = Sunce i Mjesec
    .facts =
        Sunce na nebu danju, Mjesec i zvijezde noću
        Mjesec prikazuje mijenu misije

## ZVUK

options-section-volume = Glasnoća
options-volume-master = Glavna
    .facts = Svaki zvuk
options-volume-engine = Motor
    .facts = Vaš motor ili rotor i zrakoplovi koje čujete oko sebe
options-volume-effects = Efekti
    .facts = Oružje, eksplozije, podvozje, radio i ostalo
options-volume-warnings = Upozorenja
    .facts = Sirena prekida uzgona, radarska upozorenja i upozorenja na projektile, požarna sirena, tonovi Sidewindera
options-volume-rwr = RWR
    .facts = Tonovi radarskog upozorenja, kao dio glasnoće upozorenja
options-section-rwr-tones = Tonovi radarskog upozorenja
rwr-tones-all = SVI
    .facts = Radar koji vas osvjetljava, svaki prelet koji vas otkrije, lansiranja projektila
rwr-tones-detect = SAMO OTKRIVANJE
    .facts =
        Jedan ton kad vas radar otkrije (ponovno nakon što vas radari neko vrijeme izgube)
        I lansiranja projektila
rwr-tones-launch = SAMO LANSIRANJA
    .facts = Samo projektili lansirani na vas
rwr-tones-off = ISKLJ
    .facts = Bez tonova radarskog upozorenja (svjetla i zasloni i dalje upozoravaju)
options-section-cockpit = Kokpit
options-low-pass = Niski propust motora
    .facts =
        Unutar zrakoplova motor zvuči prigušeno (kabina i kaciga odsijecaju visoke tonove)
        Isključeno: isto kao izvana

## UPRAVLJANJE

options-section-bindings = Dodjele
options-keys-joystick = TIPKE I JOYSTICK
    .facts =
        Dodjela tipki, osi i gumba joysticka i HOTAS-a
        Spremljeno za sljedeće letove
options-section-head-tracking = Praćenje glave i očiju
options-head-tracking = Praćenje glave i očiju
    .facts =
        Vaša glava okreće i pomiče pogled u kokpitu (na zaslonu; u VR-u to radi headset)
        Program za praćenje (OpenTrack...): FreeTrack izlaz ili UDP na 127.0.0.1 port 4242
        Home: trenutni položaj glave postaje središte
options-tracking = Praćenje ({ $source })
options-no-tracker = Nijedan tracker nije otkriven: pokrenite program za praćenje

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
effect-vapour = Para i struja rotora
    .facts =
        Pri oštrim manevrima tragovi pare s vrhova krila
        Helikopter koji nisko lebdi diže prašinu, nad vodom pršteću vodu
        Vozila u pokretu dižu prašinu za sobom
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

lang-name-en = engleski
lang-name-bg = bugarski
lang-name-zh-Hans = kineski (pojednostavljeni)
lang-name-hr = hrvatski
lang-name-cs = češki
lang-name-da = danski
lang-name-nl = nizozemski
lang-name-et = estonski
lang-name-fi = finski
lang-name-fr = francuski
lang-name-de = njemački
lang-name-el = grčki
lang-name-hu = mađarski
lang-name-id = indonezijski
lang-name-ga = irski
lang-name-it = talijanski
lang-name-ja = japanski
lang-name-ko = korejski
lang-name-lv = latvijski
lang-name-lt = litavski
lang-name-mt = malteški
lang-name-pl = poljski
lang-name-pt-BR = portugalski (Brazil)
lang-name-ro = rumunjski
lang-name-ru = ruski
lang-name-sk = slovački
lang-name-sl = slovenski
lang-name-es = španjolski
lang-name-sv = švedski
lang-name-uk = ukrajinski

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Jezik preglednika: { $language }. Želite li igrati na tom jeziku?
language-offer-yes = Da
language-offer-no = Ne
language-offer-never = Ne pitaj ponovno
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
