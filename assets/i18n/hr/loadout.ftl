# Uređivač naoružanja (src/loadout.rs). Nazivi oružja su pravi, neprevedeni.

loadout-none = Nema
loadout-briefed = BRIEFING
    .facts = Teret predviđen briefingom misije
loadout-clear = ISPRAZNI
    .facts = Svi nosači prazni
loadout-done = GOTOVO
loadout-no-hardpoints = Nema nosača
loadout-no-stores = Ovaj zrakoplov ne nosi teret
loadout-fantasy = Ne postoji na stvarnom zrakoplovu
loadout-empty = Prazno
loadout-out-of-stock = Nema zaliha za ovu misiju
loadout-mirror = Isto na obje strane
    .facts = Puni i { $station }
loadout-stores = Teret
loadout-needs-equipment = Misija treba svoju kameru ili posebnu opremu

## Čemu služi koje oružje (vrste oružja 0 do 20)

# AIM-9M Sidewinder
weapon-role-0 = Zrak-zrak, infracrveno samonavođenje, kratki domet
# AIM-120A AMRAAM
weapon-role-1 = Zrak-zrak, radarsko samonavođenje, srednji domet
# AGM-88A HARM
weapon-role-2 = Proturadarski projektil: radari i SAM položaji
# Penguin 3 ASM
weapon-role-3 = Protubrodski projektil
# AGM-84A Harpoon
weapon-role-4 = Protubrodski projektil dugog dometa
# AGM-65 Maverick
weapon-role-5 = TV navođeni projektil: vozila, bunkeri, zgrade
# GBU-12 Paveway
weapon-role-6 = Laserski navođena bomba: ojačani ciljevi
# Mk 20 Rockeye
weapon-role-7 = Kasetna bomba: meki i površinski ciljevi
# Durandal
weapon-role-8 = Bomba za probijanje piste
# Mk 82 LDGP
weapon-role-9 = Bomba slobodnog pada od 500 lb
# Mk 82 Snakeye
weapon-role-10 = Kočena bomba od 500 lb za otpuštanje s niske visine
# Mk 20 Rockeye II
weapon-role-11 = Kasetna bomba: oklopna vozila i površinski ciljevi
# Mk 122 Fireye
weapon-role-12 = Zapaljiva bomba
# CBU-72 FAE
weapon-role-13 = Eksploziv gorivo-zrak: površinski ciljevi
# Mk 35 incendiary
weapon-role-14 = Zapaljiva kasetna bomba
# ISCB-1 mines
weapon-role-15 = Mine za zabranu prolaza
# IR/optical camera
weapon-role-16 = Izviđačke fotografije cilja
# 1900 lb fuel tank
weapon-role-17 = Dodatno gorivo, prebacuje se pri otpuštanju
# Special cargo
weapon-role-18 = Zalihe ili teret za misiju
# 20 mm cannon
weapon-role-19 = Top (nije teret)
# AGM-114 Hellfire
weapon-role-20 = Laserski navođen protuoklopni projektil: leti tamo kamo gleda ciljnički pod

## Kako se koje oružje koristi (dometi u km HUD-a, visine u stopama)

employment-0 = Domet: 1-12 km, dalje čeono
employment-1 = Domet: 1-38 km, dalje čeono
employment-2 =
    Domet: 9-50 km
    Lansiranje: 5.000-20.000 ft
employment-3 =
    Domet: 6-14 km (klasično navođenje: 13-14 km)
    Lansiranje: 100-500 ft
employment-4 =
    Domet: 6-41 km
    Lansiranje: 2.000-5.000 ft
employment-5 =
    Domet: 5-10 km
    Lansiranje: 2.000-5.000 ft
employment-6 =
    Domet: od 1,5 km do 2,5 puta visine, LOCK stabilan (14 km s 15.000 ft, 18 km s 25.000 ft)
    Otpuštanje: 15.000-25.000 ft
employment-8 =
    Domet: 0,9-1,3 km prije piste
    Otpuštanje: 200-500 ft
employment-9 =
    Domet: 5-11 km prije cilja
    Otpuštanje: 3.000-20.000 ft
employment-12 =
    Domet: 0,9-2,2 km prije cilja
    Otpuštanje: 100-500 ft
employment-16 = Domet: fotografije s 2-8 km (što više protivnika, to bliže), nos na cilj
employment-18 = Domet: otpustite na "Drop supplies now", unutar 4 km od zone otpuštanja
employment-20 =
    Domet: 4-8 km
    Lansiranje: 2.000-3.000 ft
employment-guided-cluster =
    Domet: od 1,5 km do 2,5 puta visine, LOCK stabilan (3,7 km s 3.000 ft, 6 km s 5.000 ft)
    Otpuštanje: 3.000-5.000 ft
employment-medium-retarded =
    Domet: 1,1-1,6 km prije cilja
    Otpuštanje: 300-1.000 ft
employment-low-retarded =
    Domet: 0,9-1,3 km prije cilja
    Otpuštanje: 200-500 ft

## Nosači

hardpoint-left-bay = Lijevi odjeljak
hardpoint-right-bay = Desni odjeljak
hardpoint-left-wing-bay = Odjeljak lijevog krila
hardpoint-right-wing-bay = Odjeljak desnog krila
hardpoint-station-1-left-wingtip = Nosač 1, vrh lijevog krila
hardpoint-station-2-left-outer-wing = Nosač 2, lijevo vanjsko krilo
hardpoint-station-3-left-wing = Nosač 3, lijevo krilo
hardpoint-station-4-left-inner-wing = Nosač 4, lijevo unutarnje krilo
hardpoint-station-5-centreline = Nosač 5, središnja os
hardpoint-station-6-right-inner-wing = Nosač 6, desno unutarnje krilo
hardpoint-station-7-right-wing = Nosač 7, desno krilo
hardpoint-station-8-right-outer-wing = Nosač 8, desno vanjsko krilo
hardpoint-station-9-right-wingtip = Nosač 9, vrh desnog krila
hardpoint-station-1-left-wingtip-rail = Nosač 1, tračnica na vrhu lijevog krila
hardpoint-station-2-left-outer-pylon = Nosač 2, lijevi vanjski pilon
hardpoint-station-3-left-inner-pylon = Nosač 3, lijevi unutarnji pilon
hardpoint-station-4-right-inner-pylon = Nosač 4, desni unutarnji pilon
hardpoint-station-5-right-outer-pylon = Nosač 5, desni vanjski pilon
hardpoint-station-6-right-wingtip-rail = Nosač 6, tračnica na vrhu desnog krila
hardpoint-tornado-left-wing-rail = Lijevo krilo, tračnica za Sidewinder
hardpoint-tornado-left-wing-pylon = Pilon lijevog krila
hardpoint-tornado-left-shoulder-forward = Lijevo rame, naprijed
hardpoint-tornado-left-shoulder-aft = Lijevo rame, straga
hardpoint-tornado-right-shoulder-aft = Desno rame, straga
hardpoint-tornado-right-shoulder-forward = Desno rame, naprijed
hardpoint-tornado-right-wing-pylon = Pilon desnog krila
hardpoint-tornado-right-wing-rail = Desno krilo, tračnica za Sidewinder
hardpoint-a10-1-left-outer-wing = Nosač 1, lijevo vanjsko krilo
hardpoint-a10-2-left-wing = Nosač 2, lijevo krilo
hardpoint-a10-3-left-wing = Nosač 3, lijevo krilo
hardpoint-a10-4-left-centre-wing = Nosač 4, lijevo središnje krilo
hardpoint-a10-5-left-fuselage = Nosač 5, lijevi trup
hardpoint-a10-6-centreline = Nosač 6, središnja os
hardpoint-a10-7-right-fuselage = Nosač 7, desni trup
hardpoint-a10-8-right-centre-wing = Nosač 8, desno središnje krilo
hardpoint-a10-9-right-wing = Nosač 9, desno krilo
hardpoint-a10-10-right-wing = Nosač 10, desno krilo
hardpoint-a10-11-right-outer-wing = Nosač 11, desno vanjsko krilo
hardpoint-av8b-1-left-rail = Nosač 1, lijeva Sidewinder tračnica
hardpoint-av8b-3-left-middle-wing = Nosač 3, lijevo krilo, sredina
hardpoint-av8b-4-left-inboard-wing = Nosač 4, lijevo krilo, uz trup
hardpoint-av8b-6-right-inboard-wing = Nosač 6, desno krilo, uz trup
hardpoint-av8b-7-right-middle-wing = Nosač 7, desno krilo, sredina
hardpoint-av8b-9-right-rail = Nosač 9, desna Sidewinder tračnica
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
