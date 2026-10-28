# Aseistuseditori (src/loadout.rs). Aseiden nimet ovat oikeita, käännöksettömiä.

loadout-none = Ei mitään
loadout-briefed = TEHTÄVÄNANTO
    .facts = Aseistus, jolla tehtävä on suunniteltu
loadout-clear = TYHJENNÄ
    .facts = Kaikki kannattimet tyhjiksi
loadout-done = VALMIS
loadout-no-hardpoints = Ei kannattimia
loadout-no-stores = Tämä kone ei kanna asekuormaa
loadout-fantasy = Ei oikeassa koneessa
loadout-empty = Tyhjä
loadout-out-of-stock = Ei varastossa tälle tehtävälle
loadout-mirror = Sama molemmin puolin
    .facts = Lataa myös { $station }
loadout-stores = Asekuorma
loadout-needs-equipment = Tehtävä tarvitsee kameransa tai erikoisvarusteensa

## Mihin kutakin asetta käytetään (asetyypit 0-20)

# AIM-9M Sidewinder
weapon-role-0 = Ilma-ilma, infrapunahakeutuva, lyhyt kantama
# AIM-120A AMRAAM
weapon-role-1 = Ilma-ilma, tutkahakeutuva, keskipitkä kantama
# AGM-88A HARM
weapon-role-2 = Tutkantorjuntaohjus: tutkat ja SAM-asemat
# Penguin 3 ASM
weapon-role-3 = Laivantorjuntaohjus
# AGM-84A Harpoon
weapon-role-4 = Pitkän kantaman laivantorjuntaohjus
# AGM-65 Maverick
weapon-role-5 = TV-ohjattu ohjus: ajoneuvot, bunkkerit, rakennukset
# GBU-12 Paveway
weapon-role-6 = Laserohjattu pommi: koviin maaleihin
# Mk 20 Rockeye
weapon-role-7 = Rypälepommi: pehmeät maalit ja aluemaalit
# Durandal
weapon-role-8 = Kiitotien rikkova pommi
# Mk 82 LDGP
weapon-role-9 = 500 lb:n vapaasti putoava pommi
# Mk 82 Snakeye
weapon-role-10 = 500 lb:n jarrutettu pommi matalalta pudotukseen
# Mk 20 Rockeye II
weapon-role-11 = Rypälepommi: panssarit ja aluemaalit
# Mk 122 Fireye
weapon-role-12 = Sytytyspommi
# CBU-72 FAE
weapon-role-13 = Polttoaine-ilmaräjähde: aluemaalit
# Mk 35 incendiary
weapon-role-14 = Sytyttävä rypälepommi
# ISCB-1 mines
weapon-role-15 = Alueen eston miinat
# IR/optical camera
weapon-role-16 = Valokuvatiedustelu maalista
# 1900 lb fuel tank
weapon-role-17 = Lisäpolttoainetta, siirretään pudotettaessa
# Special cargo
weapon-role-18 = Tehtävän tarvikkeet tai rahti
# 20 mm cannon
weapon-role-19 = Tykki (ei asekuormaa)
# AGM-114 Hellfire
weapon-role-20 = Laserohjattu panssarintorjuntaohjus: lentää sinne, minne tähtäinpodi katsoo

## Kuinka kutakin asetta käytetään (kantamat HUD:n km:nä, korkeudet jalkoina)

employment-0 = Kantama: 1-12 km, kauempana vastakkain
employment-1 = Kantama: 1-38 km, kauempana vastakkain
employment-2 =
    Kantama: 9-50 km
    Laukaisu: 5 000-20 000 ft
employment-3 =
    Kantama: 6-14 km (klassinen ohjaus: 13-14 km)
    Laukaisu: 100-500 ft
employment-4 =
    Kantama: 6-41 km
    Laukaisu: 2 000-5 000 ft
employment-5 =
    Kantama: 5-10 km
    Laukaisu: 2 000-5 000 ft
employment-6 =
    Kantama: 1,5 km - 2,5 kertaa korkeus, LOCK tasainen (14 km 15 000 ft:stä, 18 km 25 000 ft:stä)
    Pudotus: 15 000-25 000 ft
employment-8 =
    Kantama: 0,9-1,3 km ennen kiitotietä
    Pudotus: 200-500 ft
employment-9 =
    Kantama: 5-11 km ennen maalia
    Pudotus: 3 000-20 000 ft
employment-12 =
    Kantama: 0,9-2,2 km ennen maalia
    Pudotus: 100-500 ft
employment-16 = Kantama: kuvat 2-8 km:n päästä (mitä lyhyempi, sitä enemmän vastustajia), nokka maaliin
employment-18 = Kantama: pudota kehotteella "Drop supplies now", 4 km:n sisällä pudotusalueesta
employment-20 =
    Kantama: 4-8 km
    Laukaisu: 2 000-3 000 ft
employment-guided-cluster =
    Kantama: 1,5 km - 2,5 kertaa korkeus, LOCK tasainen (3,7 km 3 000 ft:stä, 6 km 5 000 ft:stä)
    Pudotus: 3 000-5 000 ft
employment-medium-retarded =
    Kantama: 1,1-1,6 km ennen maalia
    Pudotus: 300-1 000 ft
employment-low-retarded =
    Kantama: 0,9-1,3 km ennen maalia
    Pudotus: 200-500 ft

## Kannattimet

hardpoint-left-bay = Vasen asekuilu
hardpoint-right-bay = Oikea asekuilu
hardpoint-left-wing-bay = Vasen siiven asekuilu
hardpoint-right-wing-bay = Oikea siiven asekuilu
hardpoint-station-1-left-wingtip = Asema 1, vasen siivenkärki
hardpoint-station-2-left-outer-wing = Asema 2, vasen ulkosiipi
hardpoint-station-3-left-wing = Asema 3, vasen siipi
hardpoint-station-4-left-inner-wing = Asema 4, vasen sisäsiipi
hardpoint-station-5-centreline = Asema 5, keskilinja
hardpoint-station-6-right-inner-wing = Asema 6, oikea sisäsiipi
hardpoint-station-7-right-wing = Asema 7, oikea siipi
hardpoint-station-8-right-outer-wing = Asema 8, oikea ulkosiipi
hardpoint-station-9-right-wingtip = Asema 9, oikea siivenkärki
hardpoint-station-1-left-wingtip-rail = Asema 1, vasemman siivenkärjen kisko
hardpoint-station-2-left-outer-pylon = Asema 2, vasen ulompi kannatin
hardpoint-station-3-left-inner-pylon = Asema 3, vasen sisempi kannatin
hardpoint-station-4-right-inner-pylon = Asema 4, oikea sisempi kannatin
hardpoint-station-5-right-outer-pylon = Asema 5, oikea ulompi kannatin
hardpoint-station-6-right-wingtip-rail = Asema 6, oikean siivenkärjen kisko
hardpoint-tornado-left-wing-rail = Vasen siipi, Sidewinder-kisko
hardpoint-tornado-left-wing-pylon = Vasemman siiven kannatin
hardpoint-tornado-left-shoulder-forward = Vasen olka, edessä
hardpoint-tornado-left-shoulder-aft = Vasen olka, takana
hardpoint-tornado-right-shoulder-aft = Oikea olka, takana
hardpoint-tornado-right-shoulder-forward = Oikea olka, edessä
hardpoint-tornado-right-wing-pylon = Oikean siiven kannatin
hardpoint-tornado-right-wing-rail = Oikea siipi, Sidewinder-kisko
hardpoint-a10-1-left-outer-wing = Asema 1, vasen ulkosiipi
hardpoint-a10-2-left-wing = Asema 2, vasen siipi
hardpoint-a10-3-left-wing = Asema 3, vasen siipi
hardpoint-a10-4-left-centre-wing = Asema 4, vasen keskisiipi
hardpoint-a10-5-left-fuselage = Asema 5, vasen runko
hardpoint-a10-6-centreline = Asema 6, keskilinja
hardpoint-a10-7-right-fuselage = Asema 7, oikea runko
hardpoint-a10-8-right-centre-wing = Asema 8, oikea keskisiipi
hardpoint-a10-9-right-wing = Asema 9, oikea siipi
hardpoint-a10-10-right-wing = Asema 10, oikea siipi
hardpoint-a10-11-right-outer-wing = Asema 11, oikea ulkosiipi
