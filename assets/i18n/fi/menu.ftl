# Päävalikko ja yksittäisen tehtävän näyttö (src/menu.rs).

## Operaatioalueet (stealth_host::theater::THEATERS)

theater-libya = Libya (1986)
    .facts =
        Sirtinlahti ja Libyan rannikko
        Operaatio El Dorado Canyonin aika
        Useimpina päivinä selkeä aavikkotaivas
theater-persian_gulf = Persianlahti (1984)
    .facts =
        Iranin ja Irakin tankkerisodan aika
        Meren ja rannikkoaavikon yllä
        Kaikista operaatioalueista selkein sää
theater-north_cape = Pohjoiskap
    .facts =
        Arktinen Norja ja Kuolan niemimaa
        Neuvostoliiton Pohjoisen laivaston kotipaikka
        Usein matala pilvikerros
theater-central_europe = Keski-Eurooppa
    .facts =
        Nato vastaan Varsovan liitto Saksan yllä
        Selkeä taivas vain 15 % ajasta
theater-vietnam = Vietnam
    .facts =
        Hanoin ja Haiphongin alue
        Viidakkoa ja vuoristoa
        Ukkosmyrskyjä 30 %:ssa tehtävistä
theater-middle_east = Lähi-itä
    .facts =
        Itäinen Välimeri ja Levantti
        Enimmäkseen selkeä taivas
theater-desert_storm = Desert Storm
    .facts =
        Kuwait ja Irak, 1991
        F-117A:n todellinen taistelukaste
        Enimmäkseen selkeä aavikkotaivas
theater-cuba = Kuuba
    .facts =
        Karibian saarioperaatioalue
        Meri- ja maamaaleja
        Trooppinen: myrskyjä 15 %:ssa tehtävistä
theater-korea = Korea
    .facts =
        Korean niemimaa DMZ:n pohjoispuolella
        Vuoristoista maastoa
        Vaihteleva sää, mikä tahansa taivas on mahdollinen
theater-ukraine = Ukraina (2026)
    .facts =
        Itä-Ukraina, Krim ja Etelä-Venäjä
        Rintamalinja Harkovasta Hersoniin palaa
        Avointa viljelysmaata, rintaman raunioituneita kaupunkeja

## Tilanteet (toimintasäännöt)

roe-0 = Kylmä sota
    .facts =
        Salainen rauhanajan operaatio
        Huomaamatta pysyminen on tärkeintä
        Vähemmän vihollisen koneita partioimassa
        Tarkat toimintasäännöt ovat tehtävänannossa
roe-1 = Rajoitettu sota
    .facts =
        Alueellinen asekonflikti
        Vihollisen ilmatorjunta on hälytystilassa
        Enemmän vihollisen koneita kuin kylmässä sodassa
roe-2 = Perinteinen sota
    .facts =
        Täysimittainen sota
        Kaikki vihollisen joukot ovat toiminnassa
        Enemmän vihollisen koneita kuin kylmässä sodassa

## Tehtävätyypit

kind-strike = Isku
    .facts =
        Tuhoa, valokuvaa tai saavuta maamaali
        Pää- ja sivutavoite
        Ilma-maa-aseistus
kind-air = Ilmataistelu
    .facts =
        Sieppaa vihollisen kone
        Kuljetuskoneita, pommittajia tai hävittäjäpartioita
        Ilma-ilma-ohjusaseistus
kind-strike-training = Iskuharjoitus
    .facts =
        Harjoitusiskutehtävä
        Opettele aseet ja navigointi
kind-air-training = Ilmataisteluharjoitus
    .facts =
        Ilma-ilma-harjoitustehtävä
        Opettele siepaukset ja ohjukset
kind-drone-defense = Dronetorjunta
    .facts =
        Vain Ukraina
        Pysäytä Geran-hyökkäysdronejen aalto ennen kuin se osuu kaupunkiin tai ydinvoimalaan
        Pommita sen jälkeen niiden laukaisupaikka

## Vastustajat

opponents-0 = Vihreät
    .facts =
        Kokemattomat vihollismiehistöt
        Helpoin taso
opponents-1 = Tavalliset
    .facts =
        Keskitasoiset vihollismiehistöt
        Oletustaso
opponents-2 = Veteraanit
    .facts =
        Taitavat vihollismiehistöt
        Enemmän aluksia ja partioita
opponents-3 = Eliitti
    .facts =
        Vihollisen parhaat lentäjät ja SAM-miehistöt
        Vaikein taso

## Sää

sky-random = Satunnainen
    .facts =
        Arvotaan operaatioalueen ilmaston mukaan
        Muuttuu jokaisen uuden tehtävän mukana
        Pilvet estävät näköyhteyden kummaltakin puolelta
sky-clear = Selkeä
    .facts =
        Ei pilviä
        Näkyvyys 40 km
        Klassisen DOS-ajan taivas
sky-scattered = Hajanainen
    .facts =
        Kumpupilviä 20-45 %, alaraja 2500-6000 ft
        Joskus ohut keskipilvi korkealla
        Näkyvyys 30 km
sky-broken = Puolipilvinen
    .facts =
        Kumpupilviä 45-70 %, alaraja 2000-5000 ft
        Usein kerrospilvikerros yllä
        Näkyvyys 25 km
sky-overcast = Pilvinen
    .facts =
        Kerrospilviä 80-100 %, alaraja 1500-5000 ft
        Muutamia kumpupilven riekaleita alapuolella
        Näkyvyys 20 km
sky-storm = Myrsky
    .facts =
        Ukkospilvitornit jopa 40000 ft:iin
        Matalia kerros- ja kumpupilviä 1000 ft:stä alkaen
        Näkyvyys 8 km
menu-random-cover = Satunnainen peitto
    .facts =
        Pääpilvikerroksen peitto arvotaan sään mukana
        Poista valinta, jos haluat asettaa sen itse
        Pääkerros: kerrospilvi pilviselle ja myrskylle, muuten kumpupilvi
menu-cover-slider =
    .facts =
        Kuinka suuren osan taivaasta pääkerros peittää
        Ei vaikutusta selkeällä taivaalla

## Kuu

moon-date = Tehtävän päivämäärä
    .facts =
        Tehtävänannon päivämäärän todellinen kuu
        Vaatii vuorokaudenajan kytkimen (moderni)
moon-0 = Uusikuu
    .facts =
        Ei kuunvaloa: musta yö, vain valot ja tähdet
        Vihollisen lentäjät näkevät sinut 2 km:n päästä
moon-1 = Kasvava sirppi
    .facts = Ohut kuu iltataivaalla, laskee ennen keskiyötä
moon-2 = Ensimmäinen neljännes
    .facts = Puoliksi valaistu, illalla taivaalla, laskee keskiyön aikaan
moon-3 = Kasvava kuu
    .facts = Kirkas, taivaalla iltapäivästä keskiyön jälkeiseen aikaan
moon-4 = Täysikuu
    .facts =
        Taivaalla koko yön, keskiyöllä korkealla
        Vihollisen lentäjät näkevät sinut 4 km:n päästä
moon-5 = Vähenevä kuu
    .facts = Kirkas, nousee illalla, taivaalla aamuun asti
moon-6 = Viimeinen neljännes
    .facts = Puoliksi valaistu, nousee keskiyön aikaan
moon-7 = Vähenevä sirppi
    .facts = Ohut kuu nousee ennen aamunkoittoa

## Koneet

aircraft-f117 =
    .facts =
        Häivehyökkäyskone
        Aliääninen, 7 g
        Moderni ja klassinen ohjaamo
aircraft-f16 =
    .facts =
        Monitoimihävittäjä
        Mach 2 -luokka, 9 g, nopea kallistus
        F-16-ohjaamo RWR:llä
aircraft-tornado =
    .facts =
        Matalalento-iskukone
        Kääntösiivet kääntyvät nopeuden mukana
        Moderni ohjaamo, kolme MFD:tä
aircraft-a10 =
    .facts =
        Lähitukikone
        Hidas ja kestävä, 30 mm:n tykki
        A-10C-ohjaamo, kaksi MFD:tä
aircraft-uh60 =
    .facts =
        Yleiskäyttöhelikopteri
        Kuljettaa joukkoja: laskeudu maalin lähelle
        Syklinen ja kollektiivinen ohjain
aircraft-ah1z =
    .facts =
        Taisteluhelikopteri
        Tornitykki, Hellfire-ohjuksia, AIM-9
        Tykki ja Hellfire-ohjukset seuraavat podia

## Osiot ja ryhmät

section-theater = Operaatioalue
section-opponents = Vastustajat
section-situation = Tilanne
section-mission = Tehtävä
section-moon = Kuu
section-start-time = Aloitusaika
section-weather = Sää
section-realism = Realismi
section-tacview = Tacview
section-mission-summary = Tehtävän yhteenveto
section-briefing = Tehtävänanto
group-mission = Tehtävä
group-conditions = Olosuhteet
group-vehicle = Ajoneuvo
group-simulation = Simulaatio

## Tehtävän asetukset

menu-start-random = Satunnainen
    .facts =
        Arvotaan jokaiselle uudelle tehtävälle, kuten klassisessa tilassa
        06:00-08:00 päivällä tai 18:00-20:00 yöllä
        Poista valinta, jos haluat valita kellonajan
menu-start-slider =
    .facts =
        Lentoonlähdön aika
        Yö klo 18:00-06:00
        UUSI TEHTÄVÄ säilyttää sen
menu-realistic-weapons = Realistiset aseet
    .facts =
        Oikean koneen rajoitukset
        2 asekuilua 4:n sijaan
        Ei AIM-9- tai AIM-120-ohjuksia
        Ei lentotukialuksen lentoonlähtöjä eikä laskuja
menu-crash-protection = Törmäyssuoja
    .facts =
        Nostaa koneen automaattisesti lähellä maata (teline sisällä)
        Maakosketus ja törmäykset eivät ole kohtalokkaita
        Pois: maahan voi aina osua
menu-tacview-export = Vienti
    .facts =
        Tallentaa jokaisen lennon Tacview-tallenteeksi (.acmi)
        Pelin tiedostokansiossa, kansiossa tacview
menu-tacview-real-time = Reaaliaikainen telemetria
    .facts =
        Lähettää lennon Tacviewiin lentäessäsi
        Tacviewissa: Record > Real-Time Telemetry
        Osoite 127.0.0.1, portti 42674 (--tacview-address)
switch-game = PELI
switch-real = TODELLINEN
menu-damage = Vauriot
    .facts =
        Aseiden vaikutus
        Peli: klassinen vaurio; osumat kasautuvat mutta eivät koskaan pudota konetta
        Realistinen: ohjusosuma voi tuhota koneen (pelastaudu!), vaurioittaa useampia järjestelmiä ja aiheuttaa usein tulipalon, joka sammuu itsestään tai räjäyttää koneen
        Realistinen: vaurioituneet ohjauspinnat saavat koneen epätrimmiin tai heilumaan
menu-guidance = Ohjaus
    .facts =
        Ohjusten ohjaus
        Klassinen: ohjukset lentävät suoraan maaliinsa
        Realistinen: suhteellinen navigointi törmäyspisteeseen; moottorit palavat loppuun ja ohjukset liitävät eteenpäin menettäen nopeutta jokaisessa käännöksessä (savua vain moottorin palaessa)
        Realistinen: SAM-ohjukset nousevat kaukaisiin maaleihin korkealentoratana; AIM-120 lentää 3 s suoraan (nosta nokkaa lobbausta varten)
menu-air-defence = Ilmatorjunta
    .facts =
        Matalan korkeuden ilmatorjunta
        Klassinen: vain SAM-asemat ja hävittäjät
        Realistinen: tutkaohjatut ZSU-23-4-tykit (niiden tutka näkee sinut kuten SAM:n) ja S-60-ilmatorjuntapatterit vartioivat lentokenttiä ja maaleja; kaupunkien MANPADS-ryhmät ampuvat SA-14-ohjuksia näkemäänsä
        Matala lento puolustettujen paikkojen yllä muuttuu vaaralliseksi; pommit ja tykit tuhoavat tykit

## Päänäyttö

menu-title = STEALTH MISSION 89
menu-subtitle = TEHTÄVÄNSUUNNITTELUJÄRJESTELMÄ
menu-single-mission = YKSITTÄINEN TEHTÄVÄ
    .facts =
        Yksi tehtävä valitsemallasi operaatioalueella
        Valitse asetukset, suunnittele reitti ja aseistus
menu-career = URA
    .facts =
        Lentäjän tiedot: arvo, kunniamerkit, lennot
        Palveluskausi: 12 lentoa yhdellä operaatioalueella
        Se mitä tuhoat, pysyy tuhottuna
menu-campaign = DYNAAMINEN KAMPANJA
    .facts =
        Dynaaminen kampanja reaaliajassa
        Valtaa vihollisen kaupungit ja lentokentät
        Tehtävät elävästä tilanteesta
menu-multiplayer = MONINPELI
    .facts = Yhteistyö dynaamisessa kampanjassa
    .join = Liity palvelimeen julkiselta listalta
    .host = Liity palvelimeen tai isännöi kampanja ja lennä siinä
menu-options = ASETUKSET
    .facts =
        Peli: ohjaamon, HUD:n ja avioniikan tyyli, autotrimmi
        Grafiikka, äänenvoimakkuudet, näppäin- ja ohjaussauvamääritykset
        Tallennetaan seuraavia lentoja varten
    .short = Peli, grafiikka, ääni ja ohjaus
menu-download = LATAA
    .facts =
        Työpöytäpeli Windowsille, Linuxille ja macOS:lle
        VR:llä ja moninpelillä
        Steam Deck ja Steam Frame: Linuxin AppImage
web-play = PELAA SELAIMESSA
    .facts = Tässä ja nyt, mitään ei tarvitse asentaa
web-download-platforms = Windows, Linux ja macOS, Steam Deck ja Steam Frame
web-download-version = Ladattava versio
web-gives-vr = VR
    .why = Lennä VR:ssä millä tahansa OpenXR-yhteensopivalla laitteella ja Steam Framella
web-gives-hotas = HOTAS
    .why = Täysi tuki peliohjaimille, ohjaussauvoille, tehonsäätimille ja polkimille
web-gives-multiplayer = Parempi moninpeli
    .why = Suora UDP-yhteys palvelimiin pienemmällä viiveellä
web-gives-server = Isännöi oma palvelin
    .why = Isännöi yhteistyökampanja tai aja dedikoitua palvelinta
web-gives-head-tracking = Pään seuranta
    .why = TrackIR-tyyliset seurantalaitteet OpenTrackin kautta
web-gives-tacview = Tacview
    .why = Lentotallenteet ja reaaliaikainen telemetria
web-gives-keep = Pysyy sinulla
    .why = Vaikka verkkosivusto menisi alas, oma kopiosi säilyy
menu-discord = DISCORD
    .facts =
        Lennä yhdessä, ilmoita virheistä ja jaa tehtäviä
        Seuraa kehitystä
menu-quit = LOPETA
    .facts = Poistu työpöydälle
menu-version = VERSIO { $version }

## Yksittäisen tehtävän näyttö

menu-career-sortie = URAN LENTO
menu-tour-sortie = { $pilot }  //  PALVELUSKAUDEN LENTO { $sortie } / { $length }
menu-msn = TEHTÄVÄNSUUNNITTELUJÄRJESTELMÄ  //  MSN { $seed }
menu-no-targets = Tälle palveluskaudelle ei ole jäänyt pystyssä olevia maaleja
menu-plan-route = SUUNNITTELE REITTI
    .facts =
        Operaatioalueen kartta vihollisen tutka- ja SAM-peittoalueineen
        Lisää, siirrä ja poista navigointipisteitä
        Maalit ja laskeutumistukikohta pysyvät paikoillaan
menu-loadout = ASEISTUS
    .facts =
        Valitse asekuorma jokaiseen kuiluun tai kannattimeen
        Alkaa tehtävänannon mukaisilla aseilla
menu-generating = LUODAAN...
menu-back-records = Takaisin lentäjätietoihin
menu-back-main = Takaisin päävalikkoon
menu-new-sortie = UUSI LENTO
menu-new-mission = UUSI TEHTÄVÄ
    .facts =
        Arpoo uudet tavoitteet
        Säilyttää kaikki asetukset ja aloitusajan
menu-load = LATAA
    .facts =
        Lennon aikana tallennetut tehtävät (valikko: TALLENNA TEHTÄVÄ)
        Jatka siitä, mihin ne tallennettiin
menu-fly = LENNÄ
    .facts = Lähde lentoon tehtävänannon mukaiseen tehtävään
menu-fly-vr = LENNÄ VR:SSÄ
    .facts =
        Käynnistää pelin uudelleen VR-laseissa
        Lentää täsmälleen tämän tehtävän
    .no-mission = Luo ensin tehtävä
    .no-runtime = Mikään OpenXR-ajoympäristö (Monado, WiVRn, SteamVR) ei ole käynnissä
menu-vr-failed = VR-käynnistys epäonnistui: { $error }

## Tehtävän yhteenveto

summary-theater = OPERAATIOALUE
summary-situation = TILANNE
summary-mission = TEHTÄVÄ
summary-opponents = VASTUSTAJAT
summary-time = AIKA
summary-weather = SÄÄ
summary-fuel = POLTTOAINE
summary-night = Yö
summary-day = Päivä
summary-time-value = { $daytime }  lähtö { $depart }
summary-weather-value = { $sky }  näkyvyys { $vis } km
summary-fuel-value = { $fuel } lb
summary-armament = Aseistus: { $stores }
summary-intelligence = TIEDUSTELU
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
