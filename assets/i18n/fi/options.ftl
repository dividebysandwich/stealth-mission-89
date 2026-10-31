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
options-tooltips = Ohjaamon työkaluvihjeet
    .facts = Mikä ohjaamon säädin tai näyttö on, hiiren tai VR-sormenpään alla
options-player-labels = Pelaajien nimet
    .facts = Moninpeli: muiden lentäjien nimet heidän koneidensa yllä, etäisyydestä riippumatta
options-label-size =
    .facts =
        Pelaajien nimien koko
        50 %: oletus
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
effect-vapour = Höyry ja roottorivirtaus
    .facts =
        Jyrkissä liikkeissä höyryvanat siivenkärjistä
        Matalalla leijuva helikopteri nostaa pölyä, veden yllä vesisumua
        Liikkuvat ajoneuvot nostavat pölyä perässään
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
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
