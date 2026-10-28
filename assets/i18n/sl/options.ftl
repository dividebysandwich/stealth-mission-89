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
