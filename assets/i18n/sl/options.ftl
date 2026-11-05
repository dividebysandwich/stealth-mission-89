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
options-keep-time-accel = Ohrani pospeševanje časa
    .facts =
        Pospeševanje se ne prekine več, ko vas odkrijejo, izstrelite orožje ali vas sestrelijo
        Spremenita ga le gumb TIME in Shift + Z / Shift + X
options-tooltips = Tooltipi kabine
    .facts = Kaj je krmilni element ali zaslon v kabini, pod miško ali konico prsta v VR
options-player-labels = Imena igralcev
    .facts = Več igralcev: imena drugih pilotov nad njihovimi letali, na katerikoli razdalji
options-label-size =
    .facts =
        Velikost imen igralcev
        50 %: privzeto
options-seat-height = Višina sedeža
    .facts =
        Kako visoko sedite v pilotski kabini, na zaslonu in v VR
        0 cm: običajna višina oči v letalu
options-section-vr-size = Velikost kabine v VR
options-vr-size =
    .facts =
        Velikost kabine (in sveta) v očalih
        100 %: naravna velikost
options-section-gif = Snemanje GIF
options-gif-size = VELIKOST
options-gif-rate = SLIKE
options-gif-scale-100 =
    .facts = Polna ločljivost okna: velike datoteke
options-gif-scale-25 =
    .facts = Majhne datoteke
options-gif-rate-50 =
    .facts = Samo ko igra teče pri 100 fps ali več (zajeta je vsaka druga slika)
options-gif-hint = F12 (če ni preslikana) začne in ustavi; shranjeno v mapo Slike

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
options-cloud-quality = Oblaki
cloud-quality-auto = Samodejno
    .facts = Nizka v VR, visoka na zaslonu
cloud-quality-low = Nizka
    .facts = Grobi oblaki pri tretjini ločljivosti (najhitreje)
cloud-quality-medium = Srednja
    .facts = Polovična ločljivost, finejše podrobnosti
cloud-quality-high = Visoka
    .facts = Polna ločljivost, najfinejše podrobnosti in lastna senca (porabi čas GPE)

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
options-volume-rwr = RWR
    .facts = Toni radarskega opozorila, kot del glasnosti opozoril
options-section-rwr-tones = Toni radarskega opozorila
rwr-tones-all = VSI
    .facts = Radar, ki vas obseva, vsak preplet, ki vas zazna, izstrelitve izstrelkov
rwr-tones-detect = SAMO ZAZNAVA
    .facts =
        En ton, ko vas radar zazna (znova, ko vas radarji za nekaj časa izgubijo)
        In izstrelitve izstrelkov
rwr-tones-launch = SAMO IZSTRELITVE
    .facts = Samo izstrelki, izstreljeni proti vam
rwr-tones-off = IZKLOP
    .facts = Brez tonov radarskega opozorila (lučke in zasloni še vedno opozarjajo)
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

options-section-effects = Učinki
options-section-retro = Retro
effect-shake = Tresenje kamere
    .facts =
        Pogled se trese ob streljanju topa, bližnjih eksplozijah, zlomu vzgona, pristankih in na stezi
        VR: kabina vibrira, glava ostane pri miru
effect-flashes = Bojni bliski
    .facts = Eksplozije flaka, eksplozije in izstrelitve raket za trenutek osvetlijo oblake, tla in kabino
effect-barrage = Nočna zaporna ogenj
    .facts =
        Konvencionalna vojna ponoči: topovi mest slepo streljajo sledilne naboje v nebo, žarometi jih prečesavajo
        Sovražne drone nad mestom ujamejo njegovi žarometi in pritegnejo ogenj njegovih topov
        Samo za videz: zapora ne zadene ničesar
effect-cockpit-night = Nočna osvetlitev kabine
    .facts =
        Medla rdeča luč v kabini po mraku (modrozelena pod očali)
        Master caution utripa jantarno nad ploščo, dokler traja opozorilo
effect-nvg = Očala za nočno gledanje
    .facts =
        Močne luči se razlijejo v svetlobne kroge, slika je v temi zrnatejša
        Namizje: okroglo vidno polje očal
effect-snow = Sneg
    .facts = Nad ničelno izotermo nevihte prinesejo sneg namesto dežja: nizko v severni zimi, visoko poleti
effect-mist = Prizemna megla
    .facts =
        Sodobni shaderji: megla leži nad nižino in morjem ob zori, mraku in ponoči
        Redka v puščavah, je ni ob nevihtah
effect-cloud-shadows = Sence oblakov
    .facts = Sodobni shaderji: sence oblakov na tleh spodaj
effect-vapour = Para in tok rotorja
    .facts =
        Pri ostrih manevrih sledi pare s konic kril
        Nizko lebdeči helikopter dviguje prah, nad vodo pršec
        Vozila v gibanju za seboj dvigujejo prah
effect-night-sky = Nočno nebo
    .facts =
        Občasno zvezdne utrinke, nizke zvezde nad obzorjem utripajo
        Severni sij nad Nordkappom
effect-vga = Barve VGA
    .facts = Videz iz leta 1989 s sodobnim senčenjem: 64 stopenj na barvni kanal kot pri kartici VGA, z ditheringom
effect-scanlines = Vrstice skeniranja
    .facts = Vrstice monitorja CRT (ne v VR)

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
effect-sunbeams = Sončni žarki
    .facts = Snopi sončne svetlobe skozi vrzeli razdrte oblačnosti
options-msaa-forced = Izklopljeno, dokler je pikselizacija (retro) vklopljena
options-volume-radio = Radio
    .facts = Drugi igralci, ki govorijo v koop radiu, in govorjeni ATC
options-volume-atc = Glas ATC
    .facts = Govorjeni klici ATC, kot del glasnosti radia
options-volume-music = Glasba
    .facts = Glasba glavnega menija
options-section-atc = Govorjeni ATC
options-section-radio = Koop radio
options-push-to-talk = Pritisni in govori
    .facts =
        Držite tipko za govor (Caps Lock ali gumb joysticka ali VR, dodeljen PTT) in vsi igralci vas slišijo
        Mikrofon se odpre ob prvem pritisku
options-mic-gain = OJAČANJE MIK.
    .facts =
        Kako glasno gre vaš mikrofon ven
        100 %: nespremenjeno
options-mic-default = Privzeto
options-voice-text-only = samo besedilo
options-voices-line = Kontrolorji: { $controllers }   Pilot: { $pilot }
options-voices = GLASOVI
    .facts =
        Prenesi glasove za radijske klice, jih izbriši
        Izberi glasove kontrolorjev in svojega ter jih poslušaj
options-voices-web = V brskalniku samo besedilo
options-tracking-mouse = Pogled z miško pri sledenju
    .facts =
        Vkl.: desna tipka miške še vedno vrti in približuje pogled iz kabine
        Izkl.: dokler sledilnik pošilja, desna tipka miške v kabini ne naredi nič
options-atc = Kontrola zračnega prometa
    .facts =
        Radijski klici stolpa, približevanja in GCA na zavezniških letališčih
        Dovoljenje za vzlet, overhead break in talkdown pri nizki oblačnosti ali slabi vidljivosti
options-pixel-block = Velikost bloka pikselizacije
    .facts = Stranica bloka v slikovnih pikah zaslona (2: fino, 5: grobo)
effect-pixelize = Pikselizacija
    .facts =
        Ves 3D-pogled (teren, letala, stavbe, kabina) v velikih kvadratnih pikslih kot na računalniku z MS-DOS; meniji in namigi ostanejo ostri (ne v VR)
        Drsnik nastavi velikost bloka v slikovnih pikah zaslona (2: fino, 5: grobo)
        Glajenje robov je izklopljeno, dokler je vklopljena
        Tri stikala pod njo pustijo HUD, kokpit ali MFD-je, instrumente in Kneeboard ostre
effect-sharp-hud = Brez HUD
    .facts =
        Pikselizacija: HUD (in zaslon na čeladi) se riše ostro čez pikselizirani pogled, da njegovi tanki simboli ostanejo berljivi
effect-sharp-cockpit = Brez kokpita (gumbi, stikala...)
    .facts =
        Pikselizacija: plošča, okvir, sedež, gumbi, stikala, ročica in ročica za plin kokpita se rišejo ostro
effect-sharp-gauges = Brez MFD-jev, instrumentov in Kneeboarda
    .facts =
        Pikselizacija: MFD-ji, umetni horizont, instrumenti, lučke, odčitki, oznake in Kneeboard se rišejo ostro
options-needs-pixelize = Zahteva pikselizacijo
