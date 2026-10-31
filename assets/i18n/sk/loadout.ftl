# Editor výzbroje (src/loadout.rs). Názvy zbraní sú skutočné, nepreložené.

loadout-none = Žiadne
loadout-briefed = BRIEFING
    .facts = Výzbroj, s ktorou bola misia zadaná v briefingu
loadout-clear = VYPRÁZDNIŤ
    .facts = Všetky závesníky prázdne
loadout-done = HOTOVO
loadout-no-hardpoints = Žiadne závesníky
loadout-no-stores = Toto lietadlo nenesie žiadny náklad
loadout-fantasy = Na skutočnom lietadle neexistuje
loadout-empty = Prázdne
loadout-out-of-stock = Pre túto misiu nič nie je na sklade
loadout-mirror = Rovnako na oboch stranách
    .facts = Naloží aj { $station }
loadout-stores = Náklad
loadout-needs-equipment = Misia potrebuje svoju kameru alebo špeciálne vybavenie

## Na čo slúži každá zbraň (typy zbraní 0 až 20)

# AIM-9M Sidewinder
weapon-role-0 = Vzduch-vzduch, infračervené navádzanie, krátky dosah
# AIM-120A AMRAAM
weapon-role-1 = Vzduch-vzduch, radarové navádzanie, stredný dosah
# AGM-88A HARM
weapon-role-2 = Protiradarová raketa: radary a pozície SAM
# Penguin 3 ASM
weapon-role-3 = Protilodná raketa
# AGM-84A Harpoon
weapon-role-4 = Protilodná raketa dlhého dosahu
# AGM-65 Maverick
weapon-role-5 = Raketa navádzaná televíznym obrazom: vozidlá, bunkre, budovy
# GBU-12 Paveway
weapon-role-6 = Laserom navádzaná bomba: spevnené ciele
# Mk 20 Rockeye
weapon-role-7 = Kazetová bomba: nechránené a plošné ciele
# Durandal
weapon-role-8 = Bomba na vytváranie kráterov v dráhe
# Mk 82 LDGP
weapon-role-9 = 500 lb bomba vo voľnom páde
# Mk 82 Snakeye
weapon-role-10 = 500 lb brzdená bomba pre zhadzovanie z malej výšky
# Mk 20 Rockeye II
weapon-role-11 = Kazetová bomba: obrnené a plošné ciele
# Mk 122 Fireye
weapon-role-12 = Zápalná bomba
# CBU-72 FAE
weapon-role-13 = Termobarická nálož: plošné ciele
# Mk 35 incendiary
weapon-role-14 = Zápalná kazetová bomba
# ISCB-1 mines
weapon-role-15 = Míny na zablokovanie priestoru
# IR/optical camera
weapon-role-16 = Prieskumné snímky cieľa
# 1900 lb fuel tank
weapon-role-17 = Prídavné palivo, po odhodení sa prečerpá
# Special cargo
weapon-role-18 = Zásoby alebo náklad pre misiu
# 20 mm cannon
weapon-role-19 = Kanón (nie je to náklad)
# AGM-114 Hellfire
weapon-role-20 = Laserom navádzaná protitanková raketa: letí tam, kam sa pozerá zameriavacia gondola

## Ako sa každá zbraň používa (dosahy v km na HUD, výšky v stopách)

employment-0 = Dosah: 1-12 km, vo vzájomnom čelnom kurze väčší
employment-1 = Dosah: 1-38 km, vo vzájomnom čelnom kurze väčší
employment-2 =
    Dosah: 9-50 km
    Odpálenie: 5 000-20 000 ft
employment-3 =
    Dosah: 6-14 km (klasické navádzanie: 13-14 km)
    Odpálenie: 100-500 ft
employment-4 =
    Dosah: 6-41 km
    Odpálenie: 2 000-5 000 ft
employment-5 =
    Dosah: 5-10 km
    Odpálenie: 2 000-5 000 ft
employment-6 =
    Dosah: 1,5 km až 2,5-násobok výšky, LOCK stabilný (14 km z 15 000 ft, 18 km z 25 000 ft)
    Zhodenie: 15 000-25 000 ft
employment-8 =
    Dosah: 0,9-1,3 km pred dráhou
    Zhodenie: 200-500 ft
employment-9 =
    Dosah: 5-11 km pred cieľom
    Zhodenie: 3 000-20 000 ft
employment-12 =
    Dosah: 0,9-2,2 km pred cieľom
    Zhodenie: 100-500 ft
employment-16 = Dosah: snímky z 2-8 km (čím bližšie, tým viac protivníkov), nos na cieľ
employment-18 = Dosah: zhodiť pri hlásení "Drop supplies now", do 4 km od zhadzovacej zóny
employment-20 =
    Dosah: 4-8 km
    Odpálenie: 2 000-3 000 ft
employment-guided-cluster =
    Dosah: 1,5 km až 2,5-násobok výšky, LOCK stabilný (3,7 km z 3 000 ft, 6 km z 5 000 ft)
    Zhodenie: 3 000-5 000 ft
employment-medium-retarded =
    Dosah: 1,1-1,6 km pred cieľom
    Zhodenie: 300-1 000 ft
employment-low-retarded =
    Dosah: 0,9-1,3 km pred cieľom
    Zhodenie: 200-500 ft

## Závesníky

hardpoint-left-bay = Ľavý zbraňový priestor
hardpoint-right-bay = Pravý zbraňový priestor
hardpoint-left-wing-bay = Ľavý zbraňový priestor v krídle
hardpoint-right-wing-bay = Pravý zbraňový priestor v krídle
hardpoint-station-1-left-wingtip = Stanica 1, ľavá koncovka krídla
hardpoint-station-2-left-outer-wing = Stanica 2, ľavé vonkajšie krídlo
hardpoint-station-3-left-wing = Stanica 3, ľavé krídlo
hardpoint-station-4-left-inner-wing = Stanica 4, ľavé vnútorné krídlo
hardpoint-station-5-centreline = Stanica 5, os lietadla
hardpoint-station-6-right-inner-wing = Stanica 6, pravé vnútorné krídlo
hardpoint-station-7-right-wing = Stanica 7, pravé krídlo
hardpoint-station-8-right-outer-wing = Stanica 8, pravé vonkajšie krídlo
hardpoint-station-9-right-wingtip = Stanica 9, pravá koncovka krídla
hardpoint-station-1-left-wingtip-rail = Stanica 1, ľavá koncová lyžina krídla
hardpoint-station-2-left-outer-pylon = Stanica 2, ľavý vonkajší pylón
hardpoint-station-3-left-inner-pylon = Stanica 3, ľavý vnútorný pylón
hardpoint-station-4-right-inner-pylon = Stanica 4, pravý vnútorný pylón
hardpoint-station-5-right-outer-pylon = Stanica 5, pravý vonkajší pylón
hardpoint-station-6-right-wingtip-rail = Stanica 6, pravá koncová lyžina krídla
hardpoint-tornado-left-wing-rail = Ľavé krídlo, lyžina Sidewinder
hardpoint-tornado-left-wing-pylon = Pylón ľavého krídla
hardpoint-tornado-left-shoulder-forward = Ľavé rameno, vpredu
hardpoint-tornado-left-shoulder-aft = Ľavé rameno, vzadu
hardpoint-tornado-right-shoulder-aft = Pravé rameno, vzadu
hardpoint-tornado-right-shoulder-forward = Pravé rameno, vpredu
hardpoint-tornado-right-wing-pylon = Pylón pravého krídla
hardpoint-tornado-right-wing-rail = Pravé krídlo, lyžina Sidewinder
hardpoint-a10-1-left-outer-wing = Stanica 1, ľavé vonkajšie krídlo
hardpoint-a10-2-left-wing = Stanica 2, ľavé krídlo
hardpoint-a10-3-left-wing = Stanica 3, ľavé krídlo
hardpoint-a10-4-left-centre-wing = Stanica 4, stred ľavého krídla
hardpoint-a10-5-left-fuselage = Stanica 5, ľavý trup
hardpoint-a10-6-centreline = Stanica 6, os lietadla
hardpoint-a10-7-right-fuselage = Stanica 7, pravý trup
hardpoint-a10-8-right-centre-wing = Stanica 8, stred pravého krídla
hardpoint-a10-9-right-wing = Stanica 9, pravé krídlo
hardpoint-a10-10-right-wing = Stanica 10, pravé krídlo
hardpoint-a10-11-right-outer-wing = Stanica 11, pravé vonkajšie krídlo
hardpoint-av8b-1-left-rail = Stanica 1, ľavá lyžina Sidewinder
hardpoint-av8b-3-left-middle-wing = Stanica 3, ľavé krídlo, stred
hardpoint-av8b-4-left-inboard-wing = Stanica 4, ľavé krídlo, pri trupe
hardpoint-av8b-6-right-inboard-wing = Stanica 6, pravé krídlo, pri trupe
hardpoint-av8b-7-right-middle-wing = Stanica 7, pravé krídlo, stred
hardpoint-av8b-9-right-rail = Stanica 9, pravá lyžina Sidewinder
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
