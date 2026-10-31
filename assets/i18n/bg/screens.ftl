# Kneeboard (src/kneeboard.rs, щрихов шрифт), планиращият екран (src/planner.rs) и записаните
# мисии (src/savegame.rs). Навигационните съкращения (PT, BRG, ETA, PRI, SEC, KM, KT) остават
# на английски.

kb-brief = БРИФИНГ
kb-route = МАРШРУТ
kb-map = КАРТА
kb-target = ЦЕЛ
kb-keys = КЛАВИШИ
kb-tip = Карти, бележки за мисията и справочник на клавишите
kb-geran-file = GERAN-2 (ДОСИЕ)
kb-objectives = ЦЕЛИ
kb-primary = ОСНОВНА: { $objective }
kb-secondary = ВТОРОСТЕПЕННА: { $objective }
kb-times = ВРЕМЕНА
kb-deadline = КРАЕН СРОК { $time }
kb-now = СЕГА { $time }
kb-conditions = УСЛОВИЯ
kb-moon = ЛУНА: { $phase } ({ $lit }% ОСВЕТЕНА)
kb-armament = ВЪОРЪЖЕНИЕ
kb-roe = ПРАВИЛА ЗА АНГАЖИРАНЕ
kb-total = ОБЩО { $km } KM ПРИ { $kt } KT
kb-fuel = ГОРИВО { $lbs } LBS
kb-preparing = ПОДГОТОВКА...
kb-airborne = ВЪВ ВЪЗДУХА
kb-developing = ПРОЯВЯВАНЕ...
kb-no-photo = НЯМА СНИМКА
kb-airborne-target = ВЪЗДУШНА ЦЕЛ
kb-scroll-vr = ПРЕВЪРТАНЕ: СТИК, ПРЪСТ ВЪРХУ ДЪСКАТА
kb-scroll-desktop = ПРЕВЪРТАНЕ: КОЛЕЛЦЕ НА МИШКАТА НАД ДЪСКАТА

## Планиращ екран

planner-low = НИСКО 500 FT
planner-medium = СРЕДНО 5000 FT
planner-high = ВИСОКО 20000 FT
planner-title = ПЛАНИРАНЕ НА МИСИЯ
planner-route = Маршрут
planner-base = БАЗА
planner-total = ОБЩО { $km } KM  { $time }
planner-briefed = Маршрут по брифинг { $km } km, гориво { $fuel } lbs
planner-coverage = Радарно покритие за
    .note = Радарното покритие предполага хоризонтален полет при 480 kt със затворени врати на отсека.
planner-detected = { $km } km от маршрута са засечени
planner-pulse = Обсег на засичане на импулсен радар (P)
planner-doppler = Обсег на засичане на доплеров радар (D)
planner-missile = Обсег на ракетата
planner-uav = Вражески UAV
planner-edit-hint = Влачете точка, за да я преместите, картата - за да я местите. Щракнете върху отсечка, за да добавите точка, десен бутон - за да я премахнете. Колелце: мащаб.
planner-reset = НУЛИРАЙ
planner-view-hint = Влачете картата, за да я местите. Колелце: мащаб.
planner-layers = СЛОЕВЕ
planner-layer-sams = Радар / позиция SAM
planner-radar-types = Типове радари
planner-layer-pulse = Обсег на засичане на импулсен радар
planner-layer-doppler = Обсег на засичане на доплеров радар
planner-layer-airfield = Летище

## Записани мисии

saves-title = ЗАПИСАНИ МИСИИ
saves-subtitle = Записани по време на полет от менюто: ЗАПИШИ МИСИЯТА
saves-none = Няма записани мисии
saves-line = { $theater }  -  { $aircraft }  -  време на мисията { $time }
saves-saved-at = Записано { $date }
saves-unreadable = Нечетим запис (друга версия)
saves-delete =
    .facts = Изтрийте този запис
saves-back =
    .facts = Назад към екрана за единична мисия
