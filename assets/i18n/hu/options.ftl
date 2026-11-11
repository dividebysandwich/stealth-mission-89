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
options-keep-time-accel = Időgyorsítás megtartása
    .facts =
        A gyorsítás már nem szakad meg, ha felfednek, fegyvert lősz ki, vagy lelőnek
        Csak a TIME gomb és a Shift + Z / Shift + X változtatja meg
options-tooltips = Pilótafülke-tooltipek
    .facts = Mi a pilótafülke egy kezelőszerve vagy kijelzője, az egér alatt vagy VR-ban egy ujjhegynél
options-player-labels = Játékosnevek
    .facts = Többjátékos: a többi pilóta neve a gépe fölött, bármilyen távolságból
options-label-size =
    .facts =
        A játékosnevek mérete
        50%: alapértelmezett
options-seat-height = Ülésmagasság
    .facts =
        Milyen magasan ülsz a pilótafülkében, a képernyőn és VR-ben
        0 cm: a gép szokásos szemmagassága
options-section-vr-size = VR pilótafülke mérete
options-vr-size =
    .facts =
        A pilótafülke (és a világ) mérete a headsetben
        100%: valódi méret
options-section-gif = GIF-felvétel
options-gif-size = MÉRET
options-gif-rate = KÉPKOCKA
options-gif-scale-100 =
    .facts = Az ablak teljes felbontása: nagy fájlok
options-gif-scale-25 =
    .facts = Kis fájlok
options-gif-rate-50 =
    .facts = Csak ha a játék 100 fps-sel vagy gyorsabban fut (minden második képkocka kerül rögzítésre)
options-gif-hint = F12 (ha nincs átállítva) indítja és leállítja; a Képek mappába menti

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
options-cloud-quality = Felhők
cloud-quality-auto = Auto
    .facts = Alacsony VR-ben, magas a képernyőn
cloud-quality-low = Alacsony
    .facts = Durva felhők harmad felbontásban (leggyorsabb)
cloud-quality-medium = Közepes
    .facts = Fél felbontás, finomabb részletek
cloud-quality-high = Magas
    .facts = Teljes felbontás, legfinomabb részletek és saját árnyék (GPU-időbe kerül)

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
options-volume-rwr = RWR
    .facts = A radarfigyelmeztető hangok, a figyelmeztetések hangerejének részeként
options-section-rwr-tones = Radarfigyelmeztető hangok
rwr-tones-all = MIND
    .facts = Egy radar, amely megvilágít, minden pásztázás, amely észlel, rakétaindítások
rwr-tones-detect = CSAK ÉSZLELÉS
    .facts =
        Egy hang, amikor egy radar észlel (újra, miután a radarok egy időre elengedtek)
        És a rakétaindítások
rwr-tones-launch = CSAK INDÍTÁSOK
    .facts = Csak a rád indított rakéták
rwr-tones-off = KI
    .facts = Nincs radarfigyelmeztető hang (a lámpák és a kijelzők továbbra is figyelmeztetnek)
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

## EFFECTS

options-section-effects = Effektek
options-section-retro = Retro
effect-shake = Kamerarázkódás
    .facts =
        A nézet remeg az ágyútűztől, a közeli robbanásoktól, átesésektől, földetérésektől és a kifutótól
        VR: a pilótafülke vibrál, a fejed nyugodt marad
effect-flashes = Harci villanások
    .facts = Légvédelmi lövedékrobbanások, robbanások és rakétakilövések egy pillanatra megvilágítják a felhőket, a talajt és a pilótafülkét
effect-barrage = Éjszakai záróvonal
    .facts =
        Hagyományos háború éjjel: a városok ágyúi vakon nyomjelzőt lőnek az égre, fényszórók pásztázzák
        A város fölötti ellenséges drónokat elkapják a fényszórói, és magukra vonják az ágyúi tüzét
        Csak a látvány kedvéért: a zárótűz semmit sem talál
effect-cockpit-night = Éjszakai pilótafülke-világítás
    .facts =
        Halvány vörös fény a pilótafülkében sötétedés után (kékeszöld a szemüveg alatt)
        A master caution borostyánsárgán pulzál a panel fölött, amíg figyelmeztetés aktív
effect-nvg = Éjjellátó szemüveg
    .facts =
        Az erős fények glóriává dagadnak, a kép szemcsésebb sötétben
        Asztali: a szemüveg kerek látómezeje
effect-snow = Hó
    .facts = A fagypont fölött a viharok esők helyett havat hoznak: északi télen alacsonyan, nyáron magasan
effect-mist = Talajköd
    .facts =
        Modern shaderek: köd fekszik az alacsony terep és a tenger fölött hajnalban, alkonyatkor és éjjel
        Vékony a sivatagokban, viharban nincs
effect-cloud-shadows = Felhőárnyékok
    .facts = Modern shaderek: a felhők árnyéka az alatta lévő talajon
effect-vapour = Pára és rotorleáramlás
    .facts =
        Éles manővereknél párafátyol a szárnyvégeken
        Az alacsonyan függő helikopter port ver fel, víz fölött permetet
        A mozgó járművek port vernek fel maguk mögött
effect-night-sky = Éjszakai égbolt
    .facts =
        Néha hullócsillagok, a horizont fölötti alacsony csillagok pislákolnak
        A sarki fény az Északi-fok fölött
effect-vga = VGA-színek
    .facts = Az 1989-es megjelenés modern árnyalással: színcsatornánként 64 fokozat, mint egy VGA-kártyánál, ditheringgel
effect-scanlines = Pásztázó sorok
    .facts = Egy CRT monitor pásztázó sorai (VR-ban nem)

## LANGUAGE NAMES

lang-name-en = angol
lang-name-bg = bolgár
lang-name-zh-Hans = kínai (egyszerűsített)
lang-name-hr = horvát
lang-name-cs = cseh
lang-name-da = dán
lang-name-nl = holland
lang-name-et = észt
lang-name-fi = finn
lang-name-fr = francia
lang-name-de = német
lang-name-el = görög
lang-name-hu = magyar
lang-name-id = indonéz
lang-name-ga = ír
lang-name-it = olasz
lang-name-ja = japán
lang-name-ko = koreai
lang-name-lv = lett
lang-name-lt = litván
lang-name-mt = máltai
lang-name-pl = lengyel
lang-name-pt-BR = portugál (brazil)
lang-name-ro = román
lang-name-ru = orosz
lang-name-sk = szlovák
lang-name-sl = szlovén
lang-name-es = spanyol
lang-name-sv = svéd
lang-name-uk = ukrán

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = A böngésző nyelve: { $language }. Ezen a nyelven szeretnél játszani?
language-offer-yes = Igen
language-offer-no = Nem
language-offer-never = Ne kérdezd újra
effect-sunbeams = Napsugarak
    .facts = Napfénynyalábok egy szakadozott felhőzet résein át
options-msaa-forced = Kikapcsolva, amíg a pixelesítés (retro) be van kapcsolva
options-volume-radio = Rádió
    .facts = A többi játékos, aki a koop rádión beszél, és a beszélt ATC
options-volume-atc = ATC hang
    .facts = A beszélt ATC-hívások, a rádió hangerejének részeként
options-volume-music = Zene
    .facts = A főmenü zenéje
options-section-atc = Beszélt ATC
options-section-radio = Koop rádió
options-push-to-talk = Nyomd és beszélj
    .facts =
        Tartsd lenyomva a beszédgombot (Caps Lock, vagy egy PTT-hez rendelt joystick- vagy VR-gomb), és minden játékos hall téged
        A mikrofon az első megnyomáskor nyílik meg
options-mic-gain = MIKROFON ERŐSÍTÉS
    .facts =
        Mennyire hangosan szól a mikrofonod
        100%: úgy, ahogy van
options-mic-default = Alapértelmezett
options-voice-text-only = csak szöveg
options-voices-line = Irányítók: { $controllers }   Pilóta: { $pilot }
options-voices = HANGOK
    .facts =
        Hangok letöltése a rádióhívásokhoz, törlésük
        Az irányítók és a saját hang kiválasztása, meghallgatása
options-voices-web = A böngészőben csak szöveg
options-tracking-mouse = Egérrel nézelődés követéskor
    .facts =
        Be: a jobb egérgomb továbbra is forgatja és nagyítja a pilótafülke nézetét
        Ki: amíg a követő küld, a jobb egérgomb nem csinál semmit a pilótafülkében
options-atc = Légiforgalmi irányítás
    .facts =
        Torony-, megközelítési és GCA rádióhívások a baráti repülőtereken
        Felszállási engedély, overhead break és talkdown alacsony felhőzetnél vagy rossz látásnál
options-pixel-block = A pixelesítés blokkmérete
    .facts = Egy blokk oldala képernyőpixelben (2: finom, 5: durva)
effect-pixelize = Pixelesítés
    .facts =
        A teljes 3D nézet (terep, repülőgépek, épületek, pilótafülke) nagy négyzet alakú pixelekkel, mint egy MS-DOS-os PC-n; a menük és a súgók éles marad (VR-ban nem)
        A csúszka a blokk méretét állítja képernyőpixelben (2: finom, 5: durva)
        Az élsimítás ki van kapcsolva, amíg be van kapcsolva
        Az alatta lévő három kapcsoló élesen hagyja a HUD-ot, a pilótafülkét vagy az MFD-ket, a műszereket és a Kneeboardot
effect-sharp-hud = HUD nélkül
    .facts =
        Pixelesítés: A HUD (és a sisakkijelző) élesen jelenik meg a pixelesített kép felett, így vékony jelei olvashatók maradnak
effect-sharp-cockpit = Pilótafülke nélkül (gombok, kapcsolók...)
    .facts =
        Pixelesítés: A pilótafülke panelje, kerete, ülése, gombjai, kapcsolói, botkormánya és gázkarja élesen jelenik meg
effect-sharp-gauges = MFD-k, műszerek és Kneeboard nélkül
    .facts =
        Pixelesítés: Az MFD-k, a műhorizont, a műszerek, a lámpák, a kijelzések, a feliratok és a Kneeboard élesen jelennek meg
options-needs-pixelize = Pixelesítést igényel
