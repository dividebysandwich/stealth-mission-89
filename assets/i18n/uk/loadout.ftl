# Редактор озброєння (src/loadout.rs). Назви зброї справжні, не перекладені.

loadout-none = Немає
loadout-briefed = ЗА БРИФІНГОМ
    .facts = Підвіски, з якими місію було заплановано на брифінгу
loadout-clear = ОЧИСТИТИ
    .facts = Усі точки підвісу порожні
loadout-done = ГОТОВО
loadout-no-hardpoints = Немає точок підвісу
loadout-no-stores = Цей літак не несе підвісок
loadout-fantasy = Немає на справжньому літаку
loadout-empty = Порожньо
loadout-out-of-stock = Для цієї місії немає в наявності
loadout-mirror = Так само з обох боків
    .facts = Завантажує також: { $station }
loadout-stores = Підвіски
loadout-needs-equipment = Місії потрібні камера чи спеціальне обладнання

## Для чого призначена кожна зброя (типи зброї від 0 до 20)

# AIM-9M Sidewinder
weapon-role-0 = Повітря-повітря, інфрачервоне самонаведення, мала дальність
# AIM-120A AMRAAM
weapon-role-1 = Повітря-повітря, радіолокаційне самонаведення, середня дальність
# AGM-88A HARM
weapon-role-2 = Протирадіолокаційна ракета: радари та позиції ЗРК
# Penguin 3 ASM
weapon-role-3 = Протикорабельна ракета
# AGM-84A Harpoon
weapon-role-4 = Протикорабельна ракета великої дальності
# AGM-65 Maverick
weapon-role-5 = Ракета з телевізійним наведенням: техніка, бункери, будівлі
# GBU-12 Paveway
weapon-role-6 = Лазерна керована бомба: міцні цілі
# Mk 20 Rockeye
weapon-role-7 = Касетна бомба: м'які та площинні цілі
# Durandal
weapon-role-8 = Бомба для руйнування злітно-посадкової смуги
# Mk 82 LDGP
weapon-role-9 = Вільнопадаюча бомба 500 lb
# Mk 82 Snakeye
weapon-role-10 = Гальмівна бомба 500 lb для скидання з малої висоти
# Mk 20 Rockeye II
weapon-role-11 = Касетна бомба: броня та площинні цілі
# Mk 122 Fireye
weapon-role-12 = Запальна бомба
# CBU-72 FAE
weapon-role-13 = Об'ємно-детонувальний боєприпас: площинні цілі
# Mk 35 incendiary
weapon-role-14 = Запальна касетна бомба
# ISCB-1 mines
weapon-role-15 = Міни для заборони доступу на території
# IR/optical camera
weapon-role-16 = Розвідувальні знімки цілі
# 1900 lb fuel tank
weapon-role-17 = Додаткове пальне, переходить в основні баки при скиданні
# Special cargo
weapon-role-18 = Припаси чи вантаж для місії
# 20 mm cannon
weapon-role-19 = Гармата (не підвіска)
# AGM-114 Hellfire
weapon-role-20 = Лазерна протитанкова ракета: летить туди, куди дивиться прицільний контейнер

## Як застосовується кожна зброя (дальність у км на HUD, висоти у футах)

employment-0 = Дальність: 1-12 км, більше в лоб
employment-1 = Дальність: 1-38 км, більше в лоб
employment-2 =
    Дальність: 9-50 км
    Пуск: 5 000-20 000 ft
employment-3 =
    Дальність: 6-14 км (класичне наведення: 13-14 км)
    Пуск: 100-500 ft
employment-4 =
    Дальність: 6-41 км
    Пуск: 2 000-5 000 ft
employment-5 =
    Дальність: 5-10 км
    Пуск: 2 000-5 000 ft
employment-6 =
    Дальність: від 1,5 км до 2,5 висоти, LOCK стабільний (14 км з 15 000 ft, 18 км з 25 000 ft)
    Скидання: 15 000-25 000 ft
employment-8 =
    Дальність: 0,9-1,3 км до смуги
    Скидання: 200-500 ft
employment-9 =
    Дальність: 5-11 км до цілі
    Скидання: 3 000-20 000 ft
employment-12 =
    Дальність: 0,9-2,2 км до цілі
    Скидання: 100-500 ft
employment-16 = Дальність: знімки з 2-8 км (чим ближче, тим більше супротивників), носом на ціль
employment-18 = Дальність: скидати за командою "Drop supplies now", не далі 4 км від зони скидання
employment-20 =
    Дальність: 4-8 км
    Пуск: 2 000-3 000 ft
employment-guided-cluster =
    Дальність: від 1,5 км до 2,5 висоти, LOCK стабільний (3,7 км з 3 000 ft, 6 км з 5 000 ft)
    Скидання: 3 000-5 000 ft
employment-medium-retarded =
    Дальність: 1,1-1,6 км до цілі
    Скидання: 300-1 000 ft
employment-low-retarded =
    Дальність: 0,9-1,3 км до цілі
    Скидання: 200-500 ft

## Точки підвісу

hardpoint-left-bay = Лівий відсік
hardpoint-right-bay = Правий відсік
hardpoint-left-wing-bay = Лівий відсік у крилі
hardpoint-right-wing-bay = Правий відсік у крилі
hardpoint-station-1-left-wingtip = Станція 1, кінець лівого крила
hardpoint-station-2-left-outer-wing = Станція 2, зовнішня частина лівого крила
hardpoint-station-3-left-wing = Станція 3, ліве крило
hardpoint-station-4-left-inner-wing = Станція 4, внутрішня частина лівого крила
hardpoint-station-5-centreline = Станція 5, центральна вісь
hardpoint-station-6-right-inner-wing = Станція 6, внутрішня частина правого крила
hardpoint-station-7-right-wing = Станція 7, праве крило
hardpoint-station-8-right-outer-wing = Станція 8, зовнішня частина правого крила
hardpoint-station-9-right-wingtip = Станція 9, кінець правого крила
hardpoint-station-1-left-wingtip-rail = Станція 1, напрямна на кінці лівого крила
hardpoint-station-2-left-outer-pylon = Станція 2, зовнішній лівий пілон
hardpoint-station-3-left-inner-pylon = Станція 3, внутрішній лівий пілон
hardpoint-station-4-right-inner-pylon = Станція 4, внутрішній правий пілон
hardpoint-station-5-right-outer-pylon = Станція 5, зовнішній правий пілон
hardpoint-station-6-right-wingtip-rail = Станція 6, напрямна на кінці правого крила
hardpoint-tornado-left-wing-rail = Ліве крило, напрямна Sidewinder
hardpoint-tornado-left-wing-pylon = Пілон лівого крила
hardpoint-tornado-left-shoulder-forward = Лівий борт, передня
hardpoint-tornado-left-shoulder-aft = Лівий борт, задня
hardpoint-tornado-right-shoulder-aft = Правий борт, задня
hardpoint-tornado-right-shoulder-forward = Правий борт, передня
hardpoint-tornado-right-wing-pylon = Пілон правого крила
hardpoint-tornado-right-wing-rail = Праве крило, напрямна Sidewinder
hardpoint-a10-1-left-outer-wing = Станція 1, зовнішня частина лівого крила
hardpoint-a10-2-left-wing = Станція 2, ліве крило
hardpoint-a10-3-left-wing = Станція 3, ліве крило
hardpoint-a10-4-left-centre-wing = Станція 4, середина лівого крила
hardpoint-a10-5-left-fuselage = Станція 5, лівий бік фюзеляжу
hardpoint-a10-6-centreline = Станція 6, центральна вісь
hardpoint-a10-7-right-fuselage = Станція 7, правий бік фюзеляжу
hardpoint-a10-8-right-centre-wing = Станція 8, середина правого крила
hardpoint-a10-9-right-wing = Станція 9, праве крило
hardpoint-a10-10-right-wing = Станція 10, праве крило
hardpoint-a10-11-right-outer-wing = Станція 11, зовнішня частина правого крила
hardpoint-av8b-1-left-rail = Станція 1, ліва напрямна Sidewinder
hardpoint-av8b-3-left-middle-wing = Станція 3, ліве крило, середина
hardpoint-av8b-4-left-inboard-wing = Станція 4, ліве крило, біля фюзеляжу
hardpoint-av8b-6-right-inboard-wing = Станція 6, праве крило, біля фюзеляжу
hardpoint-av8b-7-right-middle-wing = Станція 7, праве крило, середина
hardpoint-av8b-9-right-rail = Станція 9, права напрямна Sidewinder
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
