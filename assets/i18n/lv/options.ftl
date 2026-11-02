# Logs OPCIJAS (src/options.rs) un tā lapas lidojuma izvēlnē.

options-title = OPCIJAS
options-tab-game = SPĒLE
options-tab-graphics = GRAFIKA
options-tab-sound = SKAŅA
options-tab-controls = VADĪBA
options-footer = Saglabāts nākamajiem lidojumiem; reālisma slēdži tiek izvēlēti katrai spēlei

## SPĒLE

options-section-language = Valoda
options-section-classic-modern = Klasisks / Moderns
options-cockpit = Kabīne
    .facts =
        Moderns: augstas izšķirtspējas displeji ar apmales pogām
        Klasisks: DOS laikmeta instrumentu paneļa izkārtojums un 77x60 displeji (F-117A)
options-hud = HUD
    .facts =
        Moderns: augstas izšķirtspējas zaļš F-16 stila HUD ar NAV, AA un AG režīmiem
        Klasisks: DOS laikmeta HUD
options-avionics = Avionika
    .facts =
        Moderns: F-16 stila MFD formāti uz katra MFD (situācijas displejs, HARM, targeting pod, karte, bruņojums) ar pārvietojamu kursoru
        Klasisks: karte un TAC pa kreisi, kamera, WAY, WPN un ORD pa labi
options-flight-model = Lidojuma modelis
    .facts =
        Moderns: aerodinamisks lidojuma modelis ar fly-by-wire: stick komandē ruļļa un tangāžas ātrumu; atlaists, deguns paliek vietā un lidojuma trajektorija tiek noturēta
        Klasisks: DOS laikmeta lidojuma modelis
options-section-flying = Pilotēšana
options-auto-trim = Automātiskais trim
    .facts =
        Ar atlaistu stick trim notur stāvokli, kādā to atstājāt
        Helikopteri: deguns vairs necelas līdz ar ātrumu; reaktīvās lidmašīnas: kompensē bojātas vadības virsmas
        Manuālais trim: Alt + bulttaustiņi, Alt + Home to centrē
options-tooltips = Kabīnes tooltips
    .facts = Kas ir kabīnes vadības elements vai displejs, zem peles vai VR pirksta gala
options-player-labels = Spēlētāju vārdi
    .facts = Daudzspēlētāju režīms: citu pilotu vārdi virs viņu lidmašīnām, jebkurā attālumā
options-label-size =
    .facts =
        Spēlētāju vārdu izmērs
        50 %: noklusējums
options-section-vr-size = VR kabīnes izmērs
options-vr-size =
    .facts =
        Kabīnes (un pasaules) izmērs austiņās
        100%: dabiskais izmērs
options-section-gif = GIF ierakstīšana
options-gif-size = IZMĒRS
options-gif-rate = KADRI
options-gif-scale-100 =
    .facts = Pilna loga izšķirtspēja: lieli faili
options-gif-scale-25 =
    .facts = Mazi faili
options-gif-rate-50 =
    .facts = Tikai ja spēle darbojas ar 100 fps vai vairāk (tiek tverts katrs otrais kadrs)
options-gif-hint = F12 (ja nav pārpiešķirts) sāk un aptur; saglabāts mapē Attēli

## GRAFIKA

options-section-rendering = Renderēšana
options-shaders = Shaders
    .facts =
        Moderns: smalks virziena ēnojums, mīksta attāluma dūmaka
        Klasisks: plakans ēnojums
options-shadows = Ēnas
    .facts =
        Moderns: saules ēnas kabīnē un pasaulē (patērē GPU laiku)
        Klasisks: bez ēnām
options-point-lights = Punktveida gaismas
    .facts =
        Moderns: kabīnes un displeju apgaismojums; sprādzieni, dūmu joslas un skrejceļa gaismas apgaismo ainu
        Klasisks: tikai saules un debesu gaisma
options-time-of-day = Diennakts laiks
    .facts =
        Moderns: saule un Mēness seko misijas pulkstenim, debesis satumst un noskaidrojas līdz ar sauli; naktī zvaigznes un Mēness fāze
        Moderns: mēness gaisma nosaka, no cik tāluma ienaidnieka piloti jūs redz naktī (2 km bezmēness naktī, 4 km pilnmēness laikā)
        Klasisks: lēna krāsu maiņa un nekustīga saule
options-msaa = Kropļojumu mazināšana (MSAA)
    .facts =
        Gludas malas: 4x MSAA, 2x VR (patērē GPU laiku)
        Izslēgta: asas pikseļu malas
options-auto-detail = Automātiska ainavas detalizācija (VR)
    .facts =
        Austiņās pilsētu, mežu un lauku redzamības rādiuss samazinās, kamēr kadri kavējas, un atkal pieaug, kad ir laika rezerve
        Grafikas iestatījumi paliek, kā izvēlēti
options-sun-moon = Saule un Mēness
    .facts =
        Saule debesīs dienā, Mēness un zvaigznes naktī
        Mēness parāda misijas fāzi
options-cloud-quality = Mākoņi
cloud-quality-auto = Auto
    .facts = Zema VR, augsta ekrānā
cloud-quality-low = Zema
    .facts = Rupji mākoņi trešdaļas izšķirtspējā (ātrākais)
cloud-quality-medium = Vidēja
    .facts = Puse izšķirtspējas, smalkākas detaļas
cloud-quality-high = Augsta
    .facts = Pilna izšķirtspēja, smalkākās detaļas un pašēna (patērē GPU laiku)

## SKAŅA

options-section-volume = Skaļums
options-volume-master = Kopējais
    .facts = Visas skaņas
options-volume-engine = Dzinējs
    .facts = Jūsu dzinējs vai rotors un lidmašīnas, kas dzirdamas jums apkārt
options-volume-effects = Efekti
    .facts = Ieroči, sprādzieni, šasija, radio un pārējais
options-volume-warnings = Brīdinājumi
    .facts = Pārvilkšanas signāls, radara un raķešu brīdinājumi, ugunsgrēka sirēna, Sidewinder toņi
options-volume-rwr = RWR
    .facts = Radara brīdinājuma toņi, kā daļa no brīdinājumu skaļuma
options-section-rwr-tones = Radara brīdinājuma toņi
rwr-tones-all = VISI
    .facts = Radars, kas jūs apgaismo, katrs apskats, kas jūs atklāj, raķešu palaišanas
rwr-tones-detect = TIKAI ATKLĀŠANA
    .facts =
        Viens tonis, kad radars jūs atklāj (atkal, kad radari jūs kādu laiku zaudējuši)
        Un raķešu palaišanas
rwr-tones-launch = TIKAI PALAIŠANAS
    .facts = Tikai uz jums palaistās raķetes
rwr-tones-off = IZSL
    .facts = Bez radara brīdinājuma toņiem (lampiņas un displeji joprojām brīdina)
options-section-cockpit = Kabīne
options-low-pass = Dzinēja zemfrekvenču filtrs
    .facts =
        Lidmašīnas iekšpusē dzinējs skan apslāpēti (nojume un ķivere nogriež augstos toņus)
        Izslēgts: tāpat kā ārpusē

## VADĪBA

options-section-bindings = Piesaistes
options-keys-joystick = TAUSTIŅI UN JOYSTICK
    .facts =
        Piesaistīt taustiņus, joystick un HOTAS asis un pogas
        Saglabāts nākamajiem lidojumiem
options-section-head-tracking = Galvas un acu izsekošana
options-head-tracking = Galvas un acu izsekošana
    .facts =
        Jūsu galva griež un pārvieto skatu kabīnē (galddatorā; VR to dara austiņas)
        Izsekošanas programma (OpenTrack...): FreeTrack izvade vai UDP uz 127.0.0.1 portu 4242
        Home: pašreizējā galvas pozīcija kļūst par centru
options-tracking = Izsekošana ({ $source })
options-no-tracker = Izsekotājs nav konstatēts: palaidiet izsekošanas programmu

## EFFECTS

options-section-effects = Efekti
options-section-retro = Retro
effect-shake = Kameras drebēšana
    .facts =
        Skats trīc no lielgabala, tuviem sprādzieniem, pārrāvuma, nosēšanās un skrejceļa
        VR: kabīne vibrē, galva paliek nekustīga
effect-flashes = Kaujas uzliesmojumi
    .facts = Zenītlielgabalu sprādzieni, sprādzieni un raķešu palaišanas uz mirkli apgaismo mākoņus, zemi un kabīni
effect-barrage = Nakts aizsprosta uguns
    .facts =
        Parasts karš naktī: pilsētu lielgabali aklā šauj trasējošos šāviņus debesīs, prožektori tos pārlūko
        Ienaidnieka dronus virs pilsētas notver tās prožektori, un tie piesaista tās lielgabalu uguni
        Tikai izskatam: aizsprosts neko netrāpa
effect-cockpit-night = Kabīnes nakts apgaismojums
    .facts =
        Blāva sarkana gaisma kabīnē pēc krēslas (zili zaļa zem brillēm)
        Master caution pulsē dzintara krāsā virs paneļa, kamēr ir aktīvs brīdinājums
effect-nvg = Nakts redzamības brilles
    .facts =
        Spilgtas gaismas pārvēršas oreolos, attēls tumsā kļūst graudaināks
        Galddators: briļļu apaļais redzeslauks
effect-snow = Sniegs
    .facts = Virs sasalšanas līmeņa vētras nes sniegu lietus vietā: zemu ziemeļu ziemā, augstu vasarā
effect-mist = Zemes migla
    .facts =
        Mūsdienīgi šaderi: migla klāj zemo apvidu un jūru rītausmā, krēslā un naktī
        Reta tuksnešos, nav vētrās
effect-cloud-shadows = Mākoņu ēnas
    .facts = Mūsdienīgi šaderi: mākoņu ēnas uz zemes zemāk
effect-vapour = Tvaiks un rotora plūsma
    .facts =
        Asos manevros no spārnu galiem stiepjas tvaika sliedes
        Zemu karājošs helikopters saceļ putekļus, virs ūdens - šļakatas
        Braucoši transportlīdzekļi saceļ putekļus aiz sevis
effect-night-sky = Nakts debesis
    .facts =
        Laiku pa laikam krītošas zvaigznes, zemās zvaigznes virs apvāršņa mirgo
        Ziemeļblāzma virs Nordkapa
effect-vga = VGA krāsas
    .facts = 1989. gada izskats ar mūsdienīgu ēnojumu: 64 pakāpes uz krāsu kanālu kā VGA kartei, ar dithering
effect-scanlines = Rindu līnijas
    .facts = CRT monitora rindu līnijas (ne VR)

## LANGUAGE NAMES

lang-name-en = angļu
lang-name-bg = bulgāru
lang-name-zh-Hans = ķīniešu (vienkāršotā)
lang-name-hr = horvātu
lang-name-cs = čehu
lang-name-da = dāņu
lang-name-nl = holandiešu
lang-name-et = igauņu
lang-name-fi = somu
lang-name-fr = franču
lang-name-de = vācu
lang-name-el = grieķu
lang-name-hu = ungāru
lang-name-id = indonēziešu
lang-name-ga = īru
lang-name-it = itāļu
lang-name-ja = japāņu
lang-name-ko = korejiešu
lang-name-lv = latviešu
lang-name-lt = lietuviešu
lang-name-mt = maltiešu
lang-name-pl = poļu
lang-name-pt-BR = portugāļu (Brazīlija)
lang-name-ro = rumāņu
lang-name-ru = krievu
lang-name-sk = slovāku
lang-name-sl = slovēņu
lang-name-es = spāņu
lang-name-sv = zviedru
lang-name-uk = ukraiņu

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Pārlūka valoda: { $language }. Vai spēlēt šajā valodā?
language-offer-yes = Jā
language-offer-no = Nē
language-offer-never = Vairs nejautāt
effect-sunbeams = Saules stari
    .facts = Saules gaismas kūļi cauri plīsuša mākoņu segas spraugām
options-msaa-forced = Izslēgts, kamēr pikselizācija (retro) ir ieslēgta
options-volume-radio = Radio
    .facts = Pārējie spēlētāji, kas runā kooperatīvajā radio, un ierunātais ATC
options-volume-atc = ATC balss
    .facts = Ierunātie ATC izsaukumi kā daļa no radio skaļuma
options-volume-music = Mūzika
    .facts = Galvenās izvēlnes mūzika
options-section-atc = Ierunātais ATC
options-section-radio = Kooperatīvais radio
options-push-to-talk = Nospied un runā
    .facts =
        Turiet nospiestu runas taustiņu (Caps Lock vai PTT piešķirta joysticka vai VR poga), un visi spēlētāji jūs dzird
        Mikrofons atveras pirmajā nospiešanas reizē
options-mic-gain = MIKROFONA PIEAUGUMS
    .facts =
        Cik skaļi iziet jūsu mikrofons
        100%: kā ir
options-mic-default = Noklusējums
options-voice-text-only = tikai teksts
options-voices-line = Dispečeri: { $controllers }   Pilots: { $pilot }
options-voices = BALSIS
    .facts =
        Lejupielādēt balsis radio izsaukumiem, dzēst tās
        Izvēlēties dispečeru balsis un savējo un noklausīties tās
options-voices-web = Pārlūkā tikai teksts
options-tracking-mouse = Skatīšanās ar peli sekošanas laikā
    .facts =
        Ieslēgts: peles labā poga joprojām griež un tuvina kabīnes skatu
        Izslēgts: kamēr izsekotājs raida, peles labā poga kabīnē neko nedara
options-atc = Gaisa satiksmes vadība
    .facts =
        Torņa, tuvošanās un GCA radio izsaukumi draudzīgajos lidlaukos
        Pacelšanās atļauja, overhead break un talkdown zemos mākoņos vai sliktā redzamībā
options-pixel-block = Pikselizācijas bloka izmērs
    .facts = Bloka mala ekrāna pikseļos (2: smalks, 5: rupjš)
effect-pixelize = Pikselizācija
    .facts =
        Viss 3D skats (reljefs, lidmašīnas, ēkas, kabīne) lielos kvadrātveida pikseļos kā MS-DOS datorā; izvēlnes un padomi paliek asi (ne VR)
        Slīdnis nosaka bloka izmēru ekrāna pikseļos (2: smalks, 5: rupjš)
        Malu izlīdzināšana ir izslēgta, kamēr tā ir ieslēgta
