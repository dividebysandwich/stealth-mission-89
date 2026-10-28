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
options-section-vr-size = VR-ohjaamon koko
options-vr-size =
    .facts =
        Ohjaamon (ja maailman) koko lasien sisällä
        100 %: luonnollinen koko

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
