# Редакторът на въоръжението (src/loadout.rs). Имената на оръжията са истинските, непреведени.

loadout-none = Няма
loadout-briefed = ПО БРИФИНГ
    .facts = Оборудването, с което е планирана мисията
loadout-clear = ИЗЧИСТВАНЕ
    .facts = Всички точки на окачване празни
loadout-done = ГОТОВО
loadout-no-hardpoints = Няма точки на окачване
loadout-no-stores = Този самолет не носи оборудване
loadout-fantasy = Няма на истинския самолет
loadout-empty = Празно
loadout-out-of-stock = Няма наличност за тази мисия
loadout-mirror = Същото и от двете страни
    .facts = Зарежда и { $station }
loadout-stores = Оборудване
loadout-needs-equipment = Мисията изисква камерата или специално оборудване си

## За какво служи всяко оръжие (типове оръжия от 0 до 20)

# AIM-9M Sidewinder
weapon-role-0 = Въздух-въздух, инфрачервено самонасочване, малък обсег
# AIM-120A AMRAAM
weapon-role-1 = Въздух-въздух, радарно самонасочване, среден обсег
# AGM-88A HARM
weapon-role-2 = Противорадарна ракета: радари и позиции на SAM
# Penguin 3 ASM
weapon-role-3 = Противокорабна ракета
# AGM-84A Harpoon
weapon-role-4 = Противокорабна ракета с голям обсег
# AGM-65 Maverick
weapon-role-5 = Ракета с телевизионно насочване: машини, бункери, сгради
# GBU-12 Paveway
weapon-role-6 = Бомба с лазерно насочване: здрави цели
# Mk 20 Rockeye
weapon-role-7 = Касетъчна бомба: меки и площни цели
# Durandal
weapon-role-8 = Бомба за разрушаване на писта
# Mk 82 LDGP
weapon-role-9 = 500 lb бомба със свободно падане
# Mk 82 Snakeye
weapon-role-10 = 500 lb бомба със спирачка за пускане от малка височина
# Mk 20 Rockeye II
weapon-role-11 = Касетъчна бомба: бронирани и площни цели
# Mk 122 Fireye
weapon-role-12 = Запалителна бомба
# CBU-72 FAE
weapon-role-13 = Обемно-детониращ боеприпас: площни цели
# Mk 35 incendiary
weapon-role-14 = Запалителна касетъчна бомба
# ISCB-1 mines
weapon-role-15 = Мини за забрана на района
# IR/optical camera
weapon-role-16 = Разузнавателни снимки на целта
# 1900 lb fuel tank
weapon-role-17 = Допълнително гориво, прехвърля се при пускане
# Special cargo
weapon-role-18 = Провизии или товар за мисията
# 20 mm cannon
weapon-role-19 = Оръдието (не е оборудване)
# AGM-114 Hellfire
weapon-role-20 = Противотанкова ракета с лазерно насочване: лети натам, накъдето гледа контейнерът за прицелване

## Как се използва всяко оръжие (обсегът в км на HUD, височините във футове)

employment-0 = Обсег: 1-12 км, по-голям челно
employment-1 = Обсег: 1-38 км, по-голям челно
employment-2 =
    Обсег: 9-50 км
    Пускане: 5,000-20,000 ft
employment-3 =
    Обсег: 6-14 км (класическо насочване: 13-14 км)
    Пускане: 100-500 ft
employment-4 =
    Обсег: 6-41 км
    Пускане: 2,000-5,000 ft
employment-5 =
    Обсег: 5-10 км
    Пускане: 2,000-5,000 ft
employment-6 =
    Обсег: 1.5 км до 2.5 пъти височината, LOCK постоянен (14 км от 15,000 ft, 18 км от 25,000 ft)
    Пускане: 15,000-25,000 ft
employment-8 =
    Обсег: 0.9-1.3 км преди пистата
    Пускане: 200-500 ft
employment-9 =
    Обсег: 5-11 км преди целта
    Пускане: 3,000-20,000 ft
employment-12 =
    Обсег: 0.9-2.2 км преди целта
    Пускане: 100-500 ft
employment-16 = Обсег: снимки от 2-8 км (колкото по-близо, толкова повече противници), носът към целта
employment-18 = Обсег: пускане при "Drop supplies now", в рамките на 4 км от зоната за пускане
employment-20 =
    Обсег: 4-8 км
    Пускане: 2,000-3,000 ft
employment-guided-cluster =
    Обсег: 1.5 км до 2.5 пъти височината, LOCK постоянен (3.7 км от 3,000 ft, 6 км от 5,000 ft)
    Пускане: 3,000-5,000 ft
employment-medium-retarded =
    Обсег: 1.1-1.6 км преди целта
    Пускане: 300-1,000 ft
employment-low-retarded =
    Обсег: 0.9-1.3 км преди целта
    Пускане: 200-500 ft

## Точки на окачване

hardpoint-left-bay = Ляв отсек
hardpoint-right-bay = Десен отсек
hardpoint-left-wing-bay = Ляв отсек в крилото
hardpoint-right-wing-bay = Десен отсек в крилото
hardpoint-station-1-left-wingtip = Станция 1, края на лявото крило
hardpoint-station-2-left-outer-wing = Станция 2, външна част на лявото крило
hardpoint-station-3-left-wing = Станция 3, ляво крило
hardpoint-station-4-left-inner-wing = Станция 4, вътрешна част на лявото крило
hardpoint-station-5-centreline = Станция 5, оста на самолета
hardpoint-station-6-right-inner-wing = Станция 6, вътрешна част на дясното крило
hardpoint-station-7-right-wing = Станция 7, дясно крило
hardpoint-station-8-right-outer-wing = Станция 8, външна част на дясното крило
hardpoint-station-9-right-wingtip = Станция 9, края на дясното крило
hardpoint-station-1-left-wingtip-rail = Станция 1, релса в края на лявото крило
hardpoint-station-2-left-outer-pylon = Станция 2, външен пилон вляво
hardpoint-station-3-left-inner-pylon = Станция 3, вътрешен пилон вляво
hardpoint-station-4-right-inner-pylon = Станция 4, вътрешен пилон вдясно
hardpoint-station-5-right-outer-pylon = Станция 5, външен пилон вдясно
hardpoint-station-6-right-wingtip-rail = Станция 6, релса в края на дясното крило
hardpoint-tornado-left-wing-rail = Ляво крило, релса за Sidewinder
hardpoint-tornado-left-wing-pylon = Пилон на лявото крило
hardpoint-tornado-left-shoulder-forward = Ляво рамо, отпред
hardpoint-tornado-left-shoulder-aft = Ляво рамо, отзад
hardpoint-tornado-right-shoulder-aft = Дясно рамо, отзад
hardpoint-tornado-right-shoulder-forward = Дясно рамо, отпред
hardpoint-tornado-right-wing-pylon = Пилон на дясното крило
hardpoint-tornado-right-wing-rail = Дясно крило, релса за Sidewinder
hardpoint-a10-1-left-outer-wing = Станция 1, външна част на лявото крило
hardpoint-a10-2-left-wing = Станция 2, ляво крило
hardpoint-a10-3-left-wing = Станция 3, ляво крило
hardpoint-a10-4-left-centre-wing = Станция 4, средна част на лявото крило
hardpoint-a10-5-left-fuselage = Станция 5, ляв фюзелаж
hardpoint-a10-6-centreline = Станция 6, оста на самолета
hardpoint-a10-7-right-fuselage = Станция 7, десен фюзелаж
hardpoint-a10-8-right-centre-wing = Станция 8, средна част на дясното крило
hardpoint-a10-9-right-wing = Станция 9, дясно крило
hardpoint-a10-10-right-wing = Станция 10, дясно крило
hardpoint-a10-11-right-outer-wing = Станция 11, външна част на дясното крило
hardpoint-av8b-1-left-rail = Станция 1, лява релса за Sidewinder
hardpoint-av8b-3-left-middle-wing = Станция 3, ляво крило, среда
hardpoint-av8b-4-left-inboard-wing = Станция 4, ляво крило, до фюзелажа
hardpoint-av8b-6-right-inboard-wing = Станция 6, дясно крило, до фюзелажа
hardpoint-av8b-7-right-middle-wing = Станция 7, дясно крило, среда
hardpoint-av8b-9-right-rail = Станция 9, дясна релса за Sidewinder
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
