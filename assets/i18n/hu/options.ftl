# A BEÁLLÍTÁSOK ablak (src/options.rs) és oldalai a repülés közbeni menüben.

options-title = BEÁLLÍTÁSOK
options-tab-game = JÁTÉK
options-tab-graphics = GRAFIKA
options-tab-sound = HANG
options-tab-controls = VEZÉRLÉS
options-footer = Mentve a következő repülésekhez; a realizmus kapcsolóit minden játékhoz külön kell megadni

## JÁTÉK

options-section-language = Nyelv
options-section-classic-modern = Klasszikus / Modern
options-cockpit = Pilótafülke
    .facts =
        Modern: nagy felbontású kijelzők peremgombokkal
        Klasszikus: a DOS-kor műszerfal-elrendezése és 77x60-as kijelzői (F-117A)
options-hud = HUD
    .facts =
        Modern: nagy felbontású, zöld F-16-stílusú HUD NAV, AA és AG módokkal
        Klasszikus: a DOS-kor HUD-ja
options-avionics = Avionika
    .facts =
        Modern: F-16-stílusú MFD-oldalak bármelyik MFD-n (helyzetkép, HARM, célzópod, térkép, függesztmények) mozgatható kurzorral
        Klasszikus: térkép és TAC balra, kamera, WAY, WPN és ORD jobbra
options-flight-model = Repülési modell
    .facts =
        Modern: aerodinamikai repülési modell fly-by-wire vezérléssel: a botkormány a bólintási és orsózási sebességet parancsolja; elengedve az orr a helyén marad és a repülési pálya megmarad
        Klasszikus: DOS-kori repülési modell
options-section-flying = Repülés
options-auto-trim = Automatikus trim
    .facts =
        Elengedett botkormánynál a trim ott tartja a helyzetet, ahol hagyta
        Helikopterek: az orr nem emelkedik tovább a sebességgel; vadászgépek: kitrimmeli a sérült kormányfelületeket
        Kézi trim: Alt + nyílbillentyűk, az Alt + Home középre állítja
options-tooltips = Pilótafülke-tooltipek
    .facts = Mi a pilótafülke egy kezelőszerve vagy kijelzője, az egér alatt vagy VR-ban egy ujjhegynél
options-section-vr-size = VR pilótafülke mérete
options-vr-size =
    .facts =
        A pilótafülke (és a világ) mérete a headsetben
        100%: valódi méret

## GRAFIKA

options-section-rendering = Megjelenítés
options-shaders = Shaderek
    .facts =
        Modern: finom irányított árnyalás, lágy távolsági köd
        Klasszikus: lapos árnyalás
options-shadows = Árnyékok
    .facts =
        Modern: napárnyékok a pilótafülkében és a világban (GPU-időbe kerül)
        Klasszikus: nincsenek árnyékok
options-point-lights = Pontfények
    .facts =
        Modern: a pilótafülke és a kijelzők megvilágítása; a robbanások, füstoszlopok és kifutópálya-fények megvilágítják a jelenetet
        Klasszikus: csak nap- és égbolt-fény
options-time-of-day = Napszak
    .facts =
        Modern: a nap és a hold a küldetés órájával mozog, az égbolt a nappal sötétedik és világosodik; éjjel csillagok és a hold fázisa
        Modern: a holdfény szabja meg, milyen messziről látják Önt az ellenséges pilóták éjjel (2 km holdtalan éjszakán, 4 km teliholdnál)
        Klasszikus: lassú színváltozás és rögzített nap
options-msaa = Élsimítás (MSAA)
    .facts =
        Sima élek: 4x MSAA, VR-ban 2x (GPU-időbe kerül)
        Kikapcsolva: kemény pixelélek
options-auto-detail = Automatikus tájdetail (VR)
    .facts =
        A headsetben a városok, erdők és mezők látótávolsága csökken, amíg a képkockák késnek, és visszanő, ha van tartalék idő
        A grafikai beállítások a választottak maradnak
options-sun-moon = Nap és hold
    .facts =
        Nappal a nap az égen, éjjel a hold és a csillagok
        A hold a küldetés fázisát mutatja

## HANG

options-section-volume = Hangerő
options-volume-master = Fő
    .facts = Minden hang
options-volume-engine = Hajtómű
    .facts = A saját hajtóműve vagy rotorja, és az Ön körül hallható gépek
options-volume-effects = Effektek
    .facts = Fegyverek, robbanások, futó, rádió és a többi
options-volume-warnings = Figyelmeztetések
    .facts = Átesési kürt, radar- és rakétafigyelmeztetések, a tűzjelző szirénája, a Sidewinder hangjai
options-section-cockpit = Pilótafülke
options-low-pass = Hajtómű aluláteresztő szűrő
    .facts =
        A gép belsejében a hajtómű tompán szól (a kabinfedél és a sisak levágja a magasakat)
        Kikapcsolva: ugyanaz, mint kint

## VEZÉRLÉS

options-section-bindings = Kiosztások
options-keys-joystick = BILLENTYŰK ÉS JOYSTICK
    .facts =
        A billentyűk, a joystick és a HOTAS tengelyeinek és gombjainak kiosztása
        Mentve a következő repülésekhez
options-section-head-tracking = Fej- és szemkövetés
options-head-tracking = Fej-/szemkövetés
    .facts =
        A feje elfordítja és mozgatja a nézetet a pilótafülkében (asztali gépen; VR-ban a headset teszi)
        Követőprogram (OpenTrack...): FreeTrack kimenet, vagy UDP a 127.0.0.1 címre, 4242-es porton
        Home: a fej jelenlegi helyzete lesz a középpont
options-tracking = Követés ({ $source })
options-no-tracker = Nem érkezik követőjel: indítsa el a követőprogramot
