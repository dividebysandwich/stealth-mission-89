# Relvastuse redaktor (src/loadout.rs). Relvade nimed on päris nimed, tõlkimata.

loadout-none = Puudub
loadout-briefed = BRIIFING
    .facts = Varustus, millega missioon briifiti
loadout-clear = TÜHJENDA
    .facts = Kõik relvapunktid tühjad
loadout-done = VALMIS
loadout-no-hardpoints = Relvapunkte pole
loadout-no-stores = See õhusõiduk ei kanna lasti
loadout-fantasy = Päris lennukil puudub
loadout-empty = Tühi
loadout-out-of-stock = Selle missiooni jaoks pole varusid
loadout-mirror = Sama mõlemal küljel
    .facts = Laadib ka: { $station }
loadout-stores = Stores
loadout-needs-equipment = Missioon vajab oma kaamerat või erivarustust

## Mille jaoks iga relv on (relvatüübid 0 kuni 20)

# AIM-9M Sidewinder
weapon-role-0 = Õhk-õhk, infrapunajuhtimisega, lühike laskekaugus
# AIM-120A AMRAAM
weapon-role-1 = Õhk-õhk, radarjuhtimisega, keskmine laskekaugus
# AGM-88A HARM
weapon-role-2 = Radarivastane rakett: radarid ja SAM-kohad
# Penguin 3 ASM
weapon-role-3 = Laevatõrjerakett
# AGM-84A Harpoon
weapon-role-4 = Kaugmaa laevatõrjerakett
# AGM-65 Maverick
weapon-role-5 = TV-juhitav rakett: sõidukid, punkrid, hooned
# GBU-12 Paveway
weapon-role-6 = Laserjuhitav pomm: kõvad sihtmärgid
# Mk 20 Rockeye
weapon-role-7 = Kassettpomm: pehmed ja pindsihtmärgid
# Durandal
weapon-role-8 = Maandumisraja kraatripomm
# Mk 82 LDGP
weapon-role-9 = 500 naelane vabalt kukkuv pomm
# Mk 82 Snakeye
weapon-role-10 = 500 naelane pidurdatud pomm madalaks heitmiseks
# Mk 20 Rockeye II
weapon-role-11 = Kassettpomm: soomusmasinad ja pindsihtmärgid
# Mk 122 Fireye
weapon-role-12 = Süütepomm
# CBU-72 FAE
weapon-role-13 = Aerosoolpomm: pindsihtmärgid
# Mk 35 incendiary
weapon-role-14 = Süüte-kassettpomm
# ISCB-1 mines
weapon-role-15 = Ala sulgevad miinid
# IR/optical camera
weapon-role-16 = Sihtmärgi luurefotod
# 1900 lb fuel tank
weapon-role-17 = Lisakütus, pumbatakse üle mahaviskamisel
# Special cargo
weapon-role-18 = Missiooni varustus või last
# 20 mm cannon
weapon-role-19 = Kahur (pole lasti)
# AGM-114 Hellfire
weapon-role-20 = Laserjuhitav tankitõrjerakett: lendab sinna, kuhu targeting pod vaatab

## Kuidas iga relva kasutada (kaugused HUD-i km-des, kõrgused jalgades)

employment-0 = Kaugus: 1-12 km, kaugemale otse vastu
employment-1 = Kaugus: 1-38 km, kaugemale otse vastu
employment-2 =
    Kaugus: 9-50 km
    Laskmine: 5 000-20 000 ft
employment-3 =
    Kaugus: 6-14 km (klassikaline juhtimine: 13-14 km)
    Laskmine: 100-500 ft
employment-4 =
    Kaugus: 6-41 km
    Laskmine: 2 000-5 000 ft
employment-5 =
    Kaugus: 5-10 km
    Laskmine: 2 000-5 000 ft
employment-6 =
    Kaugus: 1,5 km kuni 2,5 korda kõrgus, LOCK stabiilne (14 km 15 000 ft pealt, 18 km 25 000 ft pealt)
    Heide: 15 000-25 000 ft
employment-8 =
    Kaugus: 0,9-1,3 km enne rada
    Heide: 200-500 ft
employment-9 =
    Kaugus: 5-11 km enne sihtmärki
    Heide: 3 000-20 000 ft
employment-12 =
    Kaugus: 0,9-2,2 km enne sihtmärki
    Heide: 100-500 ft
employment-16 = Kaugus: fotod 2-8 km pealt (mida lähemalt, seda rohkem vastaseid), nina sihtmärgil
employment-18 = Kaugus: heida "Drop supplies now" peale, 4 km piires heitealast
employment-20 =
    Kaugus: 4-8 km
    Laskmine: 2 000-3 000 ft
employment-guided-cluster =
    Kaugus: 1,5 km kuni 2,5 korda kõrgus, LOCK stabiilne (3,7 km 3 000 ft pealt, 6 km 5 000 ft pealt)
    Heide: 3 000-5 000 ft
employment-medium-retarded =
    Kaugus: 1,1-1,6 km enne sihtmärki
    Heide: 300-1 000 ft
employment-low-retarded =
    Kaugus: 0,9-1,3 km enne sihtmärki
    Heide: 200-500 ft

## Relvapunktid

hardpoint-left-bay = Vasak relvaruum
hardpoint-right-bay = Parem relvaruum
hardpoint-left-wing-bay = Vasaku tiiva relvaruum
hardpoint-right-wing-bay = Parema tiiva relvaruum
hardpoint-station-1-left-wingtip = Station 1, vasak tiivaots
hardpoint-station-2-left-outer-wing = Station 2, vasak välistiib
hardpoint-station-3-left-wing = Station 3, vasak tiib
hardpoint-station-4-left-inner-wing = Station 4, vasak siseosa tiib
hardpoint-station-5-centreline = Station 5, keskjoon
hardpoint-station-6-right-inner-wing = Station 6, parem siseosa tiib
hardpoint-station-7-right-wing = Station 7, parem tiib
hardpoint-station-8-right-outer-wing = Station 8, parem välistiib
hardpoint-station-9-right-wingtip = Station 9, parem tiivaots
hardpoint-station-1-left-wingtip-rail = Station 1, vasak tiivaotsa rööbas
hardpoint-station-2-left-outer-pylon = Station 2, vasak välispüloon
hardpoint-station-3-left-inner-pylon = Station 3, vasak sisepüloon
hardpoint-station-4-right-inner-pylon = Station 4, parem sisepüloon
hardpoint-station-5-right-outer-pylon = Station 5, parem välispüloon
hardpoint-station-6-right-wingtip-rail = Station 6, parem tiivaotsa rööbas
hardpoint-tornado-left-wing-rail = Vasak tiib, Sidewinderi rööbas
hardpoint-tornado-left-wing-pylon = Vasaku tiiva püloon
hardpoint-tornado-left-shoulder-forward = Vasak õlg, ees
hardpoint-tornado-left-shoulder-aft = Vasak õlg, taga
hardpoint-tornado-right-shoulder-aft = Parem õlg, taga
hardpoint-tornado-right-shoulder-forward = Parem õlg, ees
hardpoint-tornado-right-wing-pylon = Parema tiiva püloon
hardpoint-tornado-right-wing-rail = Parem tiib, Sidewinderi rööbas
hardpoint-a10-1-left-outer-wing = Station 1, vasak välistiib
hardpoint-a10-2-left-wing = Station 2, vasak tiib
hardpoint-a10-3-left-wing = Station 3, vasak tiib
hardpoint-a10-4-left-centre-wing = Station 4, vasak keskosa tiib
hardpoint-a10-5-left-fuselage = Station 5, vasak kere
hardpoint-a10-6-centreline = Station 6, keskjoon
hardpoint-a10-7-right-fuselage = Station 7, parem kere
hardpoint-a10-8-right-centre-wing = Station 8, parem keskosa tiib
hardpoint-a10-9-right-wing = Station 9, parem tiib
hardpoint-a10-10-right-wing = Station 10, parem tiib
hardpoint-a10-11-right-outer-wing = Station 11, parem välistiib
hardpoint-av8b-1-left-rail = Station 1, vasak Sidewinderi rööbas
hardpoint-av8b-3-left-middle-wing = Station 3, vasak tiib, keskel
hardpoint-av8b-4-left-inboard-wing = Station 4, vasak tiib, sees
hardpoint-av8b-6-right-inboard-wing = Station 6, parem tiib, sees
hardpoint-av8b-7-right-middle-wing = Station 7, parem tiib, keskel
hardpoint-av8b-9-right-rail = Station 9, parem Sidewinderi rööbas
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
