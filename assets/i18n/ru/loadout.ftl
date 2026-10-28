# Редактор вооружения (src/loadout.rs). Названия оружия настоящие, без перевода.

loadout-none = Нет
loadout-briefed = ПО БРИФИНГУ
    .facts = Подвеска, предусмотренная брифингом миссии
loadout-clear = ОЧИСТИТЬ
    .facts = Все точки подвески пусты
loadout-done = ГОТОВО
loadout-no-hardpoints = Нет точек подвески
loadout-no-stores = Этот летательный аппарат не несёт подвески
loadout-fantasy = Нет на настоящем самолёте
loadout-empty = Пусто
loadout-out-of-stock = Нет на складе для этой миссии
loadout-mirror = Одинаково с обеих сторон
    .facts = Также загружает: { $station }
loadout-stores = Подвеска
loadout-needs-equipment = Миссии нужна камера или специальное оборудование

## Назначение каждого оружия (типы оружия от 0 до 20)

# AIM-9M Sidewinder
weapon-role-0 = «Воздух-воздух», ИК-наведение, малая дальность
# AIM-120A AMRAAM
weapon-role-1 = «Воздух-воздух», радиолокационное наведение, средняя дальность
# AGM-88A HARM
weapon-role-2 = Противорадиолокационная ракета: радары и позиции SAM
# Penguin 3 ASM
weapon-role-3 = Противокорабельная ракета
# AGM-84A Harpoon
weapon-role-4 = Противокорабельная ракета большой дальности
# AGM-65 Maverick
weapon-role-5 = Ракета с телевизионным наведением: техника, бункеры, здания
# GBU-12 Paveway
weapon-role-6 = Управляемая авиабомба с лазерным наведением: защищённые цели
# Mk 20 Rockeye
weapon-role-7 = Кассетная бомба: незащищённые и площадные цели
# Durandal
weapon-role-8 = Бетонобойная бомба для разрушения ВПП
# Mk 82 LDGP
weapon-role-9 = Свободнопадающая бомба 500 lb
# Mk 82 Snakeye
weapon-role-10 = Тормозная бомба 500 lb для сброса с малой высоты
# Mk 20 Rockeye II
weapon-role-11 = Кассетная бомба: бронетехника и площадные цели
# Mk 122 Fireye
weapon-role-12 = Зажигательная бомба
# CBU-72 FAE
weapon-role-13 = Объёмно-детонирующая бомба: площадные цели
# Mk 35 incendiary
weapon-role-14 = Зажигательная кассетная бомба
# ISCB-1 mines
weapon-role-15 = Мины для минирования местности
# IR/optical camera
weapon-role-16 = Разведывательная съёмка цели
# 1900 lb fuel tank
weapon-role-17 = Дополнительное топливо, перекачивается до сброса бака
# Special cargo
weapon-role-18 = Снабжение или груз для миссии
# 20 mm cannon
weapon-role-19 = Пушка (не подвеска)
# AGM-114 Hellfire
weapon-role-20 = Противотанковая ракета с лазерным наведением: летит туда, куда смотрит прицельный контейнер

## Применение каждого оружия (дальности в км HUD, высоты в футах)

employment-0 = Дальность: 1-12 км, больше на встречном курсе
employment-1 = Дальность: 1-38 км, больше на встречном курсе
employment-2 =
    Дальность: 9-50 км
    Пуск: 5 000-20 000 ft
employment-3 =
    Дальность: 6-14 км (классическое наведение: 13-14 км)
    Пуск: 100-500 ft
employment-4 =
    Дальность: 6-41 км
    Пуск: 2 000-5 000 ft
employment-5 =
    Дальность: 5-10 км
    Пуск: 2 000-5 000 ft
employment-6 =
    Дальность: от 1,5 км до 2,5 высоты, LOCK горит постоянно (14 км с 15 000 ft, 18 км с 25 000 ft)
    Сброс: 15 000-25 000 ft
employment-8 =
    Дальность: 0,9-1,3 км до ВПП
    Сброс: 200-500 ft
employment-9 =
    Дальность: 5-11 км до цели
    Сброс: 3 000-20 000 ft
employment-12 =
    Дальность: 0,9-2,2 км до цели
    Сброс: 100-500 ft
employment-16 = Дальность: съёмка с 2-8 км (чем больше противников, тем ближе), нос на цель
employment-18 = Дальность: сброс по команде «Drop supplies now», не далее 4 км от зоны выброски
employment-20 =
    Дальность: 4-8 км
    Пуск: 2 000-3 000 ft
employment-guided-cluster =
    Дальность: от 1,5 км до 2,5 высоты, LOCK горит постоянно (3,7 км с 3 000 ft, 6 км с 5 000 ft)
    Сброс: 3 000-5 000 ft
employment-medium-retarded =
    Дальность: 1,1-1,6 км до цели
    Сброс: 300-1 000 ft
employment-low-retarded =
    Дальность: 0,9-1,3 км до цели
    Сброс: 200-500 ft

## Точки подвески

hardpoint-left-bay = Левый отсек
hardpoint-right-bay = Правый отсек
hardpoint-left-wing-bay = Отсек левого крыла
hardpoint-right-wing-bay = Отсек правого крыла
hardpoint-station-1-left-wingtip = Точка 1, законцовка левого крыла
hardpoint-station-2-left-outer-wing = Точка 2, левое крыло, внешняя
hardpoint-station-3-left-wing = Точка 3, левое крыло
hardpoint-station-4-left-inner-wing = Точка 4, левое крыло, внутренняя
hardpoint-station-5-centreline = Точка 5, подфюзеляжная
hardpoint-station-6-right-inner-wing = Точка 6, правое крыло, внутренняя
hardpoint-station-7-right-wing = Точка 7, правое крыло
hardpoint-station-8-right-outer-wing = Точка 8, правое крыло, внешняя
hardpoint-station-9-right-wingtip = Точка 9, законцовка правого крыла
hardpoint-station-1-left-wingtip-rail = Точка 1, направляющая на левой законцовке
hardpoint-station-2-left-outer-pylon = Точка 2, левый внешний пилон
hardpoint-station-3-left-inner-pylon = Точка 3, левый внутренний пилон
hardpoint-station-4-right-inner-pylon = Точка 4, правый внутренний пилон
hardpoint-station-5-right-outer-pylon = Точка 5, правый внешний пилон
hardpoint-station-6-right-wingtip-rail = Точка 6, направляющая на правой законцовке
hardpoint-tornado-left-wing-rail = Левое крыло, направляющая Sidewinder
hardpoint-tornado-left-wing-pylon = Пилон левого крыла
hardpoint-tornado-left-shoulder-forward = Левое «плечо», передняя
hardpoint-tornado-left-shoulder-aft = Левое «плечо», задняя
hardpoint-tornado-right-shoulder-aft = Правое «плечо», задняя
hardpoint-tornado-right-shoulder-forward = Правое «плечо», передняя
hardpoint-tornado-right-wing-pylon = Пилон правого крыла
hardpoint-tornado-right-wing-rail = Правое крыло, направляющая Sidewinder
hardpoint-a10-1-left-outer-wing = Точка 1, левое крыло, внешняя
hardpoint-a10-2-left-wing = Точка 2, левое крыло
hardpoint-a10-3-left-wing = Точка 3, левое крыло
hardpoint-a10-4-left-centre-wing = Точка 4, левое крыло, центральная
hardpoint-a10-5-left-fuselage = Точка 5, левая сторона фюзеляжа
hardpoint-a10-6-centreline = Точка 6, подфюзеляжная
hardpoint-a10-7-right-fuselage = Точка 7, правая сторона фюзеляжа
hardpoint-a10-8-right-centre-wing = Точка 8, правое крыло, центральная
hardpoint-a10-9-right-wing = Точка 9, правое крыло
hardpoint-a10-10-right-wing = Точка 10, правое крыло
hardpoint-a10-11-right-outer-wing = Точка 11, правое крыло, внешняя
