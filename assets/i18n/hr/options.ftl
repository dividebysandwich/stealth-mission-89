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
options-cloud-quality = Oblaci
cloud-quality-auto = Auto
    .facts = Niska u VR-u, visoka na zaslonu
cloud-quality-low = Niska
    .facts = Grubi oblaci u trećini razlučivosti (najbrže)
cloud-quality-medium = Srednja
    .facts = Pola razlučivosti, finiji detalji
cloud-quality-high = Visoka
    .facts = Puna razlučivost, najfiniji detalji i vlastita sjena (troši vrijeme GPU-a)

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

options-section-effects = Učinci
options-section-retro = Retro
effect-shake = Tresenje kamere
    .facts =
        Pogled se trese uz paljbu topa, obližnje eksplozije, gubitak uzgona, slijetanja i stazu
        VR: kabina vibrira, glava ostaje mirna
effect-flashes = Bojni bljeskovi
    .facts = Eksplozije flaka, eksplozije i lansiranja projektila na trenutak osvjetljavaju oblake, tlo i kabinu
effect-barrage = Noćna zaporna vatra
    .facts =
        Konvencionalni rat noću: topovi gradova slijepo pucaju tragačkim mecima u nebo, reflektori ih pretražuju
        Neprijateljske dronove iznad grada hvataju njegovi reflektori i privlače vatru njegovih topova
        Samo radi izgleda: baraža ništa ne pogađa
effect-cockpit-night = Noćno osvjetljenje kabine
    .facts =
        Prigušeno crveno svjetlo u kabini nakon mraka (plavozeleno ispod naočala)
        Master caution pulsira jantarno iznad ploče dok traje upozorenje
effect-nvg = Naočale za noćno gledanje
    .facts =
        Jaka svjetla šire se u oreole, slika je zrnatija u mraku
        Radna površina: okruglo vidno polje naočala
effect-snow = Snijeg
    .facts = Iznad razine smrzavanja oluje donose snijeg umjesto kiše: nisko u sjevernoj zimi, visoko ljeti
effect-mist = Prizemna magla
    .facts =
        Moderni shaderi: magla leži nad nizinom i morem u zoru, sumrak i noću
        Rijetka u pustinjama, nema je u olujama
effect-cloud-shadows = Sjene oblaka
    .facts = Moderni shaderi: sjene oblaka na tlu ispod
effect-vapour = Para i struja rotora
    .facts =
        Pri oštrim manevrima tragovi pare s vrhova krila
        Helikopter koji nisko lebdi diže prašinu, nad vodom pršteću vodu
        Vozila u pokretu dižu prašinu za sobom
effect-night-sky = Noćno nebo
    .facts =
        S vremena na vrijeme zvijezde padalice, niske zvijezde nad obzorom trepere
        Polarna svjetlost nad Nordkappom
effect-vga = VGA boje
    .facts = Izgled iz 1989. s modernim sjenčanjem: 64 razine po kanalu boje, kao kod VGA kartice, s ditheringom
effect-scanlines = Linije skeniranja
    .facts = Linije skeniranja CRT monitora (ne u VR-u)

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
effect-sunbeams = Sunčeve zrake
    .facts = Snopovi sunčeve svjetlosti kroz procjepe razderane naoblake
options-msaa-forced = Isključeno dok je pikselizacija (retro) uključena
options-volume-radio = Radio
    .facts = Ostali igrači koji govore u koop radiju i izgovoreni ATC
options-volume-atc = Glas ATC
    .facts = Izgovoreni pozivi ATC-a, kao dio glasnoće radija
options-volume-music = Glazba
    .facts = Glazba glavnog izbornika
options-section-atc = Izgovoreni ATC
options-section-radio = Koop radio
options-push-to-talk = Pritisni i govori
    .facts =
        Držite tipku za govor (Caps Lock ili tipka joysticka ili VR-a dodijeljena PTT-u) i svi igrači vas čuju
        Mikrofon se otvara pri prvom pritisku
options-mic-gain = POJAČANJE MIK.
    .facts =
        Koliko glasno izlazi vaš mikrofon
        100 %: bez promjene
options-mic-default = Zadano
options-voice-text-only = samo tekst
options-voices-line = Kontrolori: { $controllers }   Pilot: { $pilot }
options-voices = GLASOVI
    .facts =
        Preuzmi glasove za radijske pozive, obriši ih
        Odaberi glasove kontrolora i svoj te ih poslušaj
options-voices-web = U pregledniku samo tekst
options-tracking-mouse = Pogled mišem pri praćenju
    .facts =
        Uklj.: desna tipka miša i dalje okreće i zumira pogled iz kabine
        Isklj.: dok tracker šalje, desna tipka miša u kabini ne radi ništa
options-atc = Kontrola zračnog prometa
    .facts =
        Radijski pozivi tornja, prilaza i GCA-a na savezničkim zračnim lukama
        Dopuštenje za polijetanje, overhead break i talkdown pri niskoj oblačnosti ili lošoj vidljivosti
options-pixel-block = Veličina bloka pikselizacije
    .facts = Stranica bloka u pikselima zaslona (2: fino, 5: grubo)
effect-pixelize = Pikselizacija
    .facts =
        Cijeli 3D prikaz (teren, zrakoplovi, zgrade, kabina) u velikim četvrtastim pikselima kao na računalu s MS-DOS-om; izbornici i savjeti ostaju oštri (ne u VR-u)
        Klizač postavlja veličinu bloka u pikselima zaslona (2: fino, 5: grubo)
        Anti-aliasing je isključen dok je uključena
        Tri prekidača ispod nje ostavljaju HUD, kokpit ili MFD-ove, instrumente i Kneeboard oštrima
effect-sharp-hud = Bez HUD-a
    .facts =
        Pikselizacija: HUD (i zaslon na kacigi) crta se oštro preko pikseliziranog prikaza kako bi njegovi tanki simboli ostali čitljivi
effect-sharp-cockpit = Bez kokpita (tipke, prekidači...)
    .facts =
        Pikselizacija: ploča, okvir, sjedalo, tipke, prekidači, palica i ručica gasa kokpita crtaju se oštro
effect-sharp-gauges = Bez MFD-ova, instrumenata i Kneeboarda
    .facts =
        Pikselizacija: MFD-ovi, umjetni horizont, instrumenti, žaruljice, očitanja, oznake i Kneeboard crtaju se oštro
options-needs-pixelize = Zahtijeva pikselizaciju
