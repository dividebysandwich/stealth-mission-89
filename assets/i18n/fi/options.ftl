# ASETUKSET-ikkuna (src/options.rs) ja sen sivut lennon aikaisessa valikossa.

options-title = ASETUKSET
options-tab-game = PELI
options-tab-graphics = GRAFIIKKA
options-tab-sound = ÄÄNI
options-tab-controls = OHJAUS
options-footer = Tallennetaan seuraavia lentoja varten; realismikytkimet valitaan aina pelikohtaisesti

## PELI

options-section-language = Kieli
options-section-classic-modern = Klassinen / moderni
options-cockpit = Ohjaamo
    .facts =
        Moderni: korkearesoluutioiset näytöt ja reunapainikkeet
        Klassinen: DOS-ajan mittaristo ja 77x60-näytöt (F-117A)
options-hud = HUD
    .facts =
        Moderni: korkearesoluutioinen vihreä F-16-tyylinen HUD, tilat NAV, AA ja AG
        Klassinen: DOS-ajan HUD
options-avionics = Avioniikka
    .facts =
        Moderni: F-16-tyyliset MFD-formaatit kummallakin MFD:llä (tilannenäyttö, HARM, tähtäinpodi, kartta, asekuorma) ja liikutettava kursori
        Klassinen: kartta ja TAC vasemmalla, kamera, WAY, WPN ja ORD oikealla
options-flight-model = Lentomalli
    .facts =
        Moderni: aerodynaaminen lentomalli ja fly-by-wire: sauva ohjaa nousu- ja kallistusnopeutta; vapautettuna nokka pysyy paikallaan ja lentorata säilyy
        Klassinen: DOS-ajan lentomalli
options-section-flying = Lentäminen
options-auto-trim = Autotrimmi
    .facts =
        Kun sauva on vapaana, trimmi pitää asennon siinä, mihin sen jätit
        Helikopterit: nokka ei enää nouse nopeuden mukana; suihkukoneet: vaurioituneet ohjauspinnat trimmataan pois
        Manuaalinen trimmi: Alt + nuolinäppäimet, Alt + Home keskittää sen
options-keep-time-accel = Pidä ajan kiihdytys päällä
    .facts =
        Kiihdytys ei enää katkea, kun sinut havaitaan, laukaiset aseen tai sinut ammutaan alas
        Sitä muuttavat vain TIME-painike ja Shift + Z / Shift + X
options-tooltips = Ohjaamon työkaluvihjeet
    .facts = Mikä ohjaamon säädin tai näyttö on, hiiren tai VR-sormenpään alla
options-player-labels = Pelaajien nimet
    .facts = Moninpeli: muiden lentäjien nimet heidän koneidensa yllä, etäisyydestä riippumatta
options-label-size =
    .facts =
        Pelaajien nimien koko
        50 %: oletus
options-seat-height = Istuimen korkeus
    .facts =
        Kuinka korkealla istut ohjaamossa, näytöllä ja VR:ssä
        0 cm: koneen tavallinen silmien korkeus
options-section-vr-size = VR-ohjaamon koko
options-vr-size =
    .facts =
        Ohjaamon (ja maailman) koko lasien sisällä
        100 %: luonnollinen koko
options-section-gif = GIF-tallennus
options-gif-size = KOKO
options-gif-rate = KUVAT
options-gif-scale-100 =
    .facts = Ikkunan täysi tarkkuus: suuret tiedostot
options-gif-scale-25 =
    .facts = Pienet tiedostot
options-gif-rate-50 =
    .facts = Vain kun peli pyörii 100 fps:n tai suuremmalla nopeudella (joka toinen kuva tallennetaan)
options-gif-hint = F12 (ellei vaihdettu) aloittaa ja lopettaa; tallennetaan Kuvat-kansioon

## GRAFIIKKA

options-section-rendering = Piirto
options-shaders = Shaderit
    .facts =
        Moderni: hienovarainen suuntavarjostus, pehmeä etäisyyssumu
        Klassinen: tasainen varjostus
options-shadows = Varjot
    .facts =
        Moderni: auringon varjot ohjaamossa ja maailmassa (kuormittaa GPU:ta)
        Klassinen: ei varjoja
options-point-lights = Pistevalot
    .facts =
        Moderni: ohjaamon yleisvalo ja näyttöjen valot; räjähdykset, savupilvet ja kiitotien valot valaisevat näkymää
        Klassinen: vain auringon ja taivaan valo
options-time-of-day = Vuorokaudenaika
    .facts =
        Moderni: aurinko ja kuu liikkuvat tehtävän kellon mukana, taivas tummuu ja kirkastuu auringon mukana; yöllä tähdet ja kuun vaihe
        Moderni: kuunvalo määrää, kuinka kauas vihollisen lentäjät näkevät sinut yöllä (2 km kuuttomana yönä, 4 km täysikuussa)
        Klassinen: hidas värien muutos ja paikallaan pysyvä aurinko
options-msaa = Reunanpehmennys (MSAA)
    .facts =
        Pehmeät reunat: 4x MSAA, VR:ssä 2x (kuormittaa GPU:ta)
        Pois: kovat pikselireunat
options-auto-detail = Maiseman automaattiset yksityiskohdat (VR)
    .facts =
        Lasien kanssa kaupunkien, metsien ja peltojen näkyvyysetäisyys lyhenee, kun ruudunpäivitys myöhästyy, ja kasvaa takaisin, kun aikaa jää yli
        Grafiikka-asetukset pysyvät valittuina
options-sun-moon = Aurinko ja kuu
    .facts =
        Aurinko taivaalla päivällä, kuu ja tähdet yöllä
        Kuu näyttää tehtävän vaiheen
options-scenery-distance = Maiseman piirtoetäisyys
    .facts = Kaukaiset taajamat: 7–30 km; pellot: 24–30 km. Puiden etäisyydet eivät muutu. VR:n automaattinen yksityiskohtaisuus voi lyhentää maiseman etäisyyksiä.
options-city-light-distance = Kaupunkivalojen etäisyys
    .facts = Katuvalot, ikkunat ja rakennusten varoitusvalot maiseman piirtoetäisyydestä riippumatta. Sää voi rajoittaa näkyvyyttä.
options-cloud-quality = Pilvet
cloud-quality-auto = Auto
    .facts = Matala VR:ssä, korkea näytöllä
cloud-quality-low = Matala
    .facts = Karkeat pilvet kolmasosaresoluutiolla (nopein)
cloud-quality-medium = Keskitaso
    .facts = Puolet resoluutiosta, tarkemmat yksityiskohdat
cloud-quality-high = Korkea
    .facts = Täysi resoluutio, tarkimmat yksityiskohdat ja omavarjo (vie GPU-aikaa)

## ÄÄNI

options-section-volume = Äänenvoimakkuus
options-volume-master = Pää
    .facts = Kaikki äänet
options-volume-engine = Moottori
    .facts = Oma moottorisi tai roottorisi sekä ympärilläsi kuuluvat koneet
options-volume-effects = Efektit
    .facts = Aseet, räjähdykset, laskuteline, radio ja muu
options-volume-warnings = Varoitukset
    .facts = Sakkaussummeri, tutka- ja ohjusvaroitukset, palohälytys, Sidewinder-äänet
options-volume-rwr = RWR
    .facts = Tutkavaroitusäänet, osana varoitusten äänenvoimakkuutta
options-section-rwr-tones = Tutkavaroitusäänet
rwr-tones-all = KAIKKI
    .facts = Sinua valaiseva tutka, jokainen sinut havaitseva pyyhkäisy, ohjusten laukaisut
rwr-tones-detect = VAIN HAVAINTO
    .facts =
        Yksi ääni, kun tutka havaitsee sinut (uudelleen, kun tutkat ovat hetken kadottaneet sinut)
        Ja ohjusten laukaisut
rwr-tones-launch = VAIN LAUKAISUT
    .facts = Vain sinua kohti laukaistut ohjukset
rwr-tones-off = POIS
    .facts = Ei tutkavaroitusääniä (merkkivalot ja näytöt varoittavat silti)
options-section-cockpit = Ohjaamo
options-low-pass = Moottorin alipäästö
    .facts =
        Koneen sisällä moottori kuulostaa vaimeammalta (ohjaamokupu ja kypärä leikkaavat korkeat äänet)
        Pois: sama kuin ulkona

## OHJAUS

options-section-bindings = Näppäinmääritykset
options-keys-joystick = NÄPPÄIMET JA OHJAUSSAUVA
    .facts =
        Näppäinten, ohjaussauvan ja HOTAS-akselien sekä -painikkeiden määritys
        Tallennetaan seuraavia lentoja varten
options-section-head-tracking = Pään/silmien seuranta
options-head-tracking = Pään/silmien seuranta
    .facts =
        Pääsi kääntää ja liikuttaa näkymää ohjaamossa (työpöytä; VR:ssä lasit hoitavat sen)
        Seurantaohjelma (OpenTrack...): FreeTrack-ulostulo tai UDP osoitteeseen 127.0.0.1 portti 4242
        Home: nykyisestä pään asennosta tulee keskipiste
options-tracking = Seuranta ({ $source })
options-no-tracker = Seurantalaitetta ei kuulu: käynnistä seurantaohjelma

## EFFECTS

options-section-effects = Efektit
options-section-retro = Retro
effect-shake = Kameran tärinä
    .facts =
        Näkymä tärisee tykin, lähellä olevien räjähdysten, sakkauksen, kosketusten ja kiitotien mukana
        VR: ohjaamo värisee, pää pysyy paikallaan
effect-flashes = Taisteluvälähdykset
    .facts = Ilmatorjuntasirpaleet, räjähdykset ja ohjuslaukaisut valaisevat hetkeksi pilvet, maan ja ohjaamon
effect-barrage = Yöllinen sulkutuli
    .facts =
        Tavanomainen sota yöllä: kaupunkien tykit ampuvat sokkona jälkiluotia taivaalle, valonheittimet pyyhkivät niiden yli
        Kaupungin yllä olevat vihollisen droonit jäävät sen valonheittimiin ja vetävät puoleensa sen tykkien tulen
        Vain ulkonäön vuoksi: sulutus ei osu mihinkään
effect-cockpit-night = Ohjaamon yövalaistus
    .facts =
        Himmeä punainen valo ohjaamossa pimeän tultua (sinivihreä laseissa)
        Master caution sykkii meripihkanvärisenä paneelin yllä niin kauan kuin varoitus on päällä
effect-nvg = Yönäkölasit
    .facts =
        Kirkkaat valot turpoavat sädekehiksi, kuva rakeistuu enemmän pimeässä
        Työpöytä: lasien pyöreä näkökenttä
effect-snow = Lumi
    .facts = Pakkasrajan yläpuolella myrskyt tuovat lunta sateen sijaan: matalalla pohjoisessa talvella, korkealla kesällä
effect-mist = Maasumu
    .facts =
        Nykyaikaiset shaderit: sumu peittää alavan maan ja meren aamunkoitteessa, hämärässä ja yöllä
        Ohutta aavikoilla, ei lainkaan myrskyissä
effect-cloud-shadows = Pilvien varjot
    .facts = Nykyaikaiset shaderit: pilvien varjot alla olevalla maalla
effect-vapour = Höyry ja roottorivirtaus
    .facts =
        Jyrkissä liikkeissä höyryvanat siivenkärjistä
        Matalalla leijuva helikopteri nostaa pölyä, veden yllä vesisumua
        Liikkuvat ajoneuvot nostavat pölyä perässään
effect-night-sky = Yötaivas
    .facts =
        Silloin tällöin tähdenlentoja, horisontin matalat tähdet tuikkivat
        Revontulet Pohjoiskapin yllä
effect-vga = VGA-värit
    .facts = Vuoden 1989 ilme nykyaikaisella varjostuksella: 64 askelta värikanavaa kohti kuten VGA-kortissa, ditteröitynä
effect-scanlines = Raakaviivat
    .facts = CRT-näytön raakaviivat (ei VR:ssä)

## LANGUAGE NAMES

lang-name-en = englanti
lang-name-bg = bulgaria
lang-name-zh-Hans = kiina (yksinkertaistettu)
lang-name-hr = kroatia
lang-name-cs = tšekki
lang-name-da = tanska
lang-name-nl = hollanti
lang-name-et = viro
lang-name-fi = suomi
lang-name-fr = ranska
lang-name-de = saksa
lang-name-el = kreikka
lang-name-hu = unkari
lang-name-id = indonesia
lang-name-ga = iiri
lang-name-it = italia
lang-name-ja = japani
lang-name-ko = korea
lang-name-lv = latvia
lang-name-lt = liettua
lang-name-mt = malta
lang-name-pl = puola
lang-name-pt-BR = portugali (Brasilia)
lang-name-ro = romania
lang-name-ru = venäjä
lang-name-sk = slovakki
lang-name-sl = sloveeni
lang-name-es = espanja
lang-name-sv = ruotsi
lang-name-uk = ukraina

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Selaimen kieli: { $language }. Pelataanko peliä tällä kielellä?
language-offer-yes = Kyllä
language-offer-no = Ei
language-offer-never = Älä kysy uudelleen
effect-sunbeams = Auringonsäteet
    .facts = Auringonvalon kajot rikkonaisen pilvipeitteen raoista
options-msaa-forced = Pois, kun pikselöinti (retro) on päällä
options-volume-radio = Radio
    .facts = Muut pelaajat puhumassa yhteispelin radiossa, ja puhuttu ATC
options-volume-atc = ATC-ääni
    .facts = Puhutut ATC-kutsut radion äänenvoimakkuuden osana
options-volume-music = Musiikki
    .facts = Päävalikon musiikki
options-section-atc = Puhuttu ATC
options-section-radio = Yhteispelin radio
options-push-to-talk = Paina ja puhu
    .facts =
        Pidä puhenäppäintä painettuna (Caps Lock tai PTT:hen liitetty joystick- tai VR-painike) ja kaikki pelaajat kuulevat sinut
        Mikrofoni avautuu ensimmäisellä painalluksella
options-mic-gain = MIKROFONIN VAHVISTUS
    .facts =
        Kuinka kovaa mikrofonisi lähtee
        100 %: sellaisenaan
options-mic-default = Oletus
options-voice-text-only = vain teksti
options-voices-line = Lennonjohtajat: { $controllers }   Lentäjä: { $pilot }
options-voices = ÄÄNET
    .facts =
        Lataa äänet radiokutsuja varten, poista ne
        Valitse lennonjohtajien ja omasi, ja kuuntele ne
options-voices-web = Selaimessa vain teksti
options-tracking-mouse = Hiirellä katselu seurannassa
    .facts =
        Päällä: hiiren oikea painike kääntää ja zoomaa edelleen ohjaamonäkymää
        Pois: kun seurain lähettää, hiiren oikea painike ei tee mitään ohjaamossa
options-atc = Ilmaliikenteen ohjaus
    .facts =
        Tornin, lähestymisen ja GCA:n radiokutsut ystävällisillä lentokentillä
        Lentoonlähtölupa, overhead break ja talkdown matalassa pilvessä tai huonossa näkyvyydessä
options-pixel-block = Pikselöinnin lohkon koko
    .facts = Lohkon sivu näytön pikseleinä (2: hieno, 5: karkea)
effect-pixelize = Pikselöinti
    .facts =
        Koko 3D-näkymä (maasto, koneet, rakennukset, ohjaamo) isoina neliöpikseleinä kuin MS-DOS-koneella; valikot ja työkaluvihjeet pysyvät teräväpiirtoisina (ei VR:ssä)
        Liukusäädin asettaa lohkon koon näytön pikseleinä (2: hieno, 5: karkea)
        Reunojen pehmennys on pois päältä, kun se on päällä
        Sen alla olevat kolme kytkintä jättävät HUD:in, ohjaamon tai MFD-näytöt, mittarit ja Kneeboardin teräviksi
effect-sharp-hud = Ilman HUD:ia
    .facts =
        Pikselöinti: HUD (ja kypärän näyttö) piirretään teräväksi pikselöidyn kuvan päälle, jotta sen ohuet symbolit pysyvät luettavina
effect-sharp-cockpit = Ilman ohjaamoa (painikkeet, kytkimet...)
    .facts =
        Pikselöinti: ohjaamon paneeli, runko, istuin, painikkeet, kytkimet, ohjaussauva ja kaasuvipu piirretään teräviksi
effect-sharp-gauges = Ilman MFD-näyttöjä, mittareita ja Kneeboardia
    .facts =
        Pikselöinti: MFD-näytöt, tekohorisontti, mittarit, merkkivalot, lukemat, tekstit ja Kneeboard piirretään teräviksi
options-needs-pixelize = Vaatii pikselöinnin
