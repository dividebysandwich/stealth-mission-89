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
options-auto-trim = Trim awtomatiku
    .facts =
        Bl-istick maħlul, it-trim iżomm l-assett fejn halltu
        Ħelikopters: l-għonq ma jibqax jogħla mal-veloċità; ajruplani: it-trim jikkumpensa s-superfiċi tal-kontroll bil-ħsara
        Trim manwali: Alt + vleġeġ, Alt + Home iċċentrah
options-tooltips = Tooltips tal-kokpit
    .facts = X'inhi kontroll jew display tal-kokpit, taħt il-maws jew ras is-swaba' f'VR
options-player-labels = Ismijiet tal-plejers
    .facts = Multiplayer: l-ismijiet tal-piloti l-oħra fuq l-ajruplani tagħhom, kemm jekk ikunu 'l bogħod
options-label-size =
    .facts =
        Daqs tal-ismijiet tal-plejers
        50%: id-default
options-section-vr-size = Daqs tal-kokpit f'VR
options-vr-size =
    .facts =
        Id-daqs tal-kokpit (u tad-dinja) fil-headset
        100%: daqs naturali
options-section-gif = Reġistrazzjoni GIF
options-gif-size = DAQS
options-gif-rate = FRAMES
options-gif-scale-100 =
    .facts = Ir-riżoluzzjoni sħiħa tat-tieqa: fajls kbar
options-gif-scale-25 =
    .facts = Fajls żgħar
options-gif-rate-50 =
    .facts = Biss meta l-logħba tmur b'100 fps jew aktar (jinqabad frame kull tnejn)
options-gif-hint = F12 (sakemm ma jinbidilx) jibda u jwaqqaf; jiġi ssejvjat fil-folder Stampi

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
options-point-lights = Dwal puntwali
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
options-cloud-quality = Sħab
cloud-quality-auto = Awtomatiku
    .facts = Baxx fil-VR, għoli fuq l-iskrin
cloud-quality-low = Baxx
    .facts = Sħab mhux raffinat f'terz tar-riżoluzzjoni (l-aktar mgħaġġel)
cloud-quality-medium = Medju
    .facts = Nofs ir-riżoluzzjoni, dettall aktar fin
cloud-quality-high = Għoli
    .facts = Riżoluzzjoni sħiħa, l-iżjed dettall fin u dell proprju (jiswa ħin tal-GPU)

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
options-volume-rwr = RWR
    .facts = It-toni tat-twissija tar-radar, bħala parti mill-volum tat-twissijiet
options-section-rwr-tones = Toni tat-twissija tar-radar
rwr-tones-all = KOLLHA
    .facts = Radar li jdawlek, kull sweep li jsibek, tnedijiet ta' missili
rwr-tones-detect = SKOPERTA BISS
    .facts =
        Ton wieħed meta radar isibek (mill-ġdid wara li r-radars jitilquk għal ftit)
        U tnedijiet ta' missili
rwr-tones-launch = TNEDIJIET BISS
    .facts = Biss missili mniedja fuqek
rwr-tones-off = MITFI
    .facts = L-ebda ton tat-twissija tar-radar (id-dwal u l-iskrins xorta jwissu)
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
options-section-head-tracking = Traċċar tar-ras u tal-għajnejn
options-head-tracking = Traċċar tar-Ras u tal-Għajnejn
    .facts =
        Rasek iddur u l-veduta tiċċaqlaq fil-kokpit (desktop; f'VR jagħmel dan il-headset)
        Programm ta' tracking (OpenTrack...): output FreeTrack, jew UDP lil 127.0.0.1 port 4242
        Home: il-pożizzjoni attwali tar-ras issir iċ-ċentru
options-tracking = Traċċar ({ $source })
options-no-tracker = L-ebda tracker ma nstema': ibda l-programm ta' tracking

## EFFECTS

options-section-effects = Effetti
options-section-retro = Retro
effect-shake = Tihżiża tal-kamera
    .facts =
        Il-veduta tiħżeż mal-kannun, splużjonijiet qrib, stall, inżul u l-pista
        VR: il-kokpit jivvibra, rasek tibqa' kwieta
effect-flashes = Blalen tal-battalja
    .facts = Splużjonijiet tal-flak, splużjonijiet u t-tnedijiet tal-missili jdawlu għal mument is-sħab, l-art u l-kokpit
effect-barrage = Barraġġ bil-lejl
    .facts =
        Gwerra konvenzjonali bil-lejl: il-kanuni tal-bliet jispara traċċanti għall-għama lejn is-sema, projetturi jiknsu fuqhom
        Id-droni tal-għadu fuq belt jinqabdu mill-projetturi tagħha u jiġbdu n-nar tal-kanuni tagħha
        Għad-dehra biss: il-barraġġ ma jolqot xejn
effect-cockpit-night = Dawl tal-lejl tal-kokpit
    .facts =
        Dawl aħmar baxx fil-kokpit wara d-dlam (blu-aħdar taħt in-nuċċalijiet)
        Il-master caution jippulsa bl-ambra fuq il-pannell sakemm twissija tkun attiva
effect-nvg = Nuċċalijiet tal-vista bil-lejl
    .facts =
        Id-dwal qawwija jinfirxu f'halos, l-immaġni tkun aktar granulari fid-dlam
        Desktop: il-kamp tal-vista tond tan-nuċċalijiet
effect-snow = Borra
    .facts = Fuq il-livell tal-iġgelar it-tempesti jġibu borra minflok xita: baxx fix-xitwa tat-tramuntana, għoli fis-sajf
effect-mist = Ċpar mal-art
    .facts =
        Shaders moderni: iċ-ċpar jinsab fuq l-art baxxa u l-baħar fil-ġurnata, fil-għaxija u bil-lejl
        Irqiq fid-deżerti, xejn fit-tempesti
effect-cloud-shadows = Dellijiet tas-sħab
    .facts = Shaders moderni: id-dellijiet tas-sħab fuq l-art taħt
effect-vapour = Fwar u fluss tar-rotor
    .facts =
        F'manuvri ħorox, linji ta' fwar mill-ponot tal-ġwienaħ
        Ħelikopter f'hover baxx iqajjem it-trab, jew bexx fuq l-ilma
        Vetturi jimxu jqajmu t-trab warajhom
effect-night-sky = Sema tal-lejl
    .facts =
        Minn żmien għal żmien stilel jaqgħu, l-istilel baxxi fuq l-orizzont jiddendlu
        Id-dawl tat-tramuntana fuq in-Nordkap
effect-vga = Kuluri VGA
    .facts = Id-dehra tal-1989 b'tiżlil modern: 64 pass għal kull kanal tal-kulur, bħal karta VGA, b'dithering
effect-scanlines = Linji tal-iskannjar
    .facts = Il-linji tal-iskannjar ta' monitor CRT (mhux fil-VR)

## LANGUAGE NAMES

lang-name-en = Ingliż
lang-name-bg = Bulgaru
lang-name-zh-Hans = Ċiniż (Simplifikat)
lang-name-hr = Kroat
lang-name-cs = Ċek
lang-name-da = Daniż
lang-name-nl = Olandiż
lang-name-et = Estonjan
lang-name-fi = Finlandiż
lang-name-fr = Franċiż
lang-name-de = Ġermaniż
lang-name-el = Grieg
lang-name-hu = Ungeriż
lang-name-id = Indoneżjan
lang-name-ga = Irlandiż
lang-name-it = Taljan
lang-name-ja = Ġappuniż
lang-name-ko = Korejan
lang-name-lv = Latvjan
lang-name-lt = Litwan
lang-name-mt = Malti
lang-name-pl = Pollakk
lang-name-pt-BR = Portugiż (Brażil)
lang-name-ro = Rumen
lang-name-ru = Russu
lang-name-sk = Slovakk
lang-name-sl = Sloven
lang-name-es = Spanjol
lang-name-sv = Svediż
lang-name-uk = Ukrain

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Il-lingwa tal-browser: { $language }. Tixtieq tilgħab f'din il-lingwa?
language-offer-yes = Iva
language-offer-no = Le
language-offer-never = Tistaqsinix aktar
effect-sunbeams = Raġġi tax-xemx
    .facts = Raġġi tad-dawl tax-xemx mill-ftuħ ta' sħab maqsum
options-msaa-forced = Mitfi sakemm il-pixelizzazzjoni (retro) tkun mixgħula
options-volume-radio = Radju
    .facts = Il-plejers l-oħra li jitkellmu fir-radju coop, u l-ATC mitkellem
options-volume-atc = Ħoss tal-ATC
    .facts = It-telefonati mitkellma tal-ATC, bħala parti mill-volum tar-radju
options-volume-music = Mużika
    .facts = Il-mużika tal-menu prinċipali
options-section-atc = ATC mitkellem
options-section-radio = Radju coop
options-push-to-talk = Agħfas u kellem
    .facts =
        Żomm l-iżgħar tasti tal-kelma (Caps Lock, jew buttuna ta' joystick jew VR assenjata għall-PTT) u l-plejers kollha jisimgħuk
        Il-mikrofonu jinfetaħ l-ewwel darba li tagħfas
options-mic-gain = GAIN TAL-MIKROFONU
    .facts =
        Kemm joħroġ qawwi l-mikrofonu tiegħek
        100%: kif inhu
options-mic-default = Awtomatiku
options-voice-text-only = test biss
options-voices-line = Kontrolluri: { $controllers }   Pilota: { $pilot }
options-voices = ŻEWĠIEN
    .facts =
        Niżżel vuċijiet għat-telefonati tar-radju, ħassarhom
        Agħżel dawk tal-kontrolluri u tiegħek, u isimhom
options-voices-web = Fil-browser test biss
options-tracking-mouse = Ħarsa bil-fejra fit-tracking
    .facts =
        Mixgħul: il-buttuna tal-lemin tal-ġurdien xorta ddur u tagħmel zoom tal-veduta tal-kokpit
        Maqful: sakemm it-tracker jibgħat, il-buttuna tal-lemin tal-ġurdien ma tagħmel xejn fil-kokpit
options-atc = Kontroll tat-traffiku tal-ajru
    .facts =
        Telefonati tar-radju tat-torri, tal-approċċ u tal-GCA fl-ajrudromi alleati
        Awtorizzazzjoni tat-tlugħ, overhead break u talkdown f'sħab baxx jew viżibilità ħażina
options-pixel-block = Daqs tal-blokka tal-pixelizzazzjoni
    .facts = Il-ġenb ta' blokka, f'pixels tal-iskrin (2: fin, 5: oħxon)
effect-pixelize = Pixelizzazzjoni
    .facts =
        Il-veduta 3D kollha (art, ajruplani, bini, kokpit) f'pixels kwadri kbar bħal fuq PC b'MS-DOS; il-menus u t-tooltips jibqgħu ċari (mhux fil-VR)
        Il-glidder jistabbilixxi d-daqs ta' blokka, f'pixels tal-iskrin (2: fin, 5: oħxon)
        L-anti-aliasing huwa maqful sakemm tkun mixgħula
