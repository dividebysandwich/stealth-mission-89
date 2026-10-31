# Наколенный планшет (векторный шрифт), планировщик миссии и сохранённые миссии.
# Навигационные сокращения (PT, BRG, ETA, PRI, SEC, KM, KT) остаются на английском.

kb-brief = БРИФИНГ
kb-route = МАРШРУТ
kb-map = КАРТА
kb-target = ЦЕЛЬ
kb-keys = КЛАВИШИ
kb-tip = Карты, заметки к миссии и справка по клавишам
kb-geran-file = GERAN-2 (ДОСЬЕ)
kb-objectives = ЗАДАЧИ
kb-primary = ОСНОВНАЯ: { $objective }
kb-secondary = ЗАПАСНАЯ: { $objective }
kb-times = ВРЕМЯ
kb-deadline = СРОК { $time }
kb-now = СЕЙЧАС { $time }
kb-conditions = УСЛОВИЯ
kb-moon = ЛУНА: { $phase } (ОСВЕЩЕНО { $lit }%)
kb-armament = ВООРУЖЕНИЕ
kb-roe = ПРАВИЛА ПРИМЕНЕНИЯ ОРУЖИЯ
kb-total = ВСЕГО { $km } KM НА { $kt } KT
kb-fuel = ТОПЛИВО { $lbs } LBS
kb-preparing = ПОДГОТОВКА...
kb-airborne = В ВОЗДУХЕ
kb-developing = ПРОЯВКА...
kb-no-photo = НЕТ СНИМКА
kb-airborne-target = ВОЗДУШНАЯ ЦЕЛЬ
kb-scroll-vr = ПРОКРУТКА: СТИК, ПАЛЕЦ НА ПЛАНШЕТЕ
kb-scroll-desktop = ПРОКРУТКА: КОЛЕСО МЫШИ НАД ПЛАНШЕТОМ

## Планировщик миссии

planner-low = МАЛАЯ 500 FT
planner-medium = СРЕДНЯЯ 5000 FT
planner-high = БОЛЬШАЯ 20000 FT
planner-title = ПЛАНИРОВЩИК МИССИИ
planner-route = Маршрут
planner-base = БАЗА
planner-total = ВСЕГО { $km } KM  { $time }
planner-briefed = Маршрут по брифингу { $km } км, топливо { $fuel } lbs
planner-coverage = Радиолокационное покрытие для
    .note = Покрытие рассчитано для горизонтального полёта на 480 kt с закрытыми створками бомбоотсеков.
planner-detected = Обнаруживаемый участок маршрута: { $km } км
planner-pulse = Дальность обнаружения импульсной РЛС (P)
planner-doppler = Дальность обнаружения доплеровской РЛС (D)
planner-missile = Дальность ракет
planner-uav = БПЛА противника
planner-edit-hint = Перетащите точку, чтобы сдвинуть её, или карту, чтобы её прокрутить. Щелчок по участку добавляет точку, правый щелчок удаляет её. Колесо: масштаб.
planner-reset = СБРОСИТЬ
planner-view-hint = Перетащите карту, чтобы её прокрутить. Колесо: масштаб.
planner-layers = СЛОИ
planner-layer-sams = РЛС / позиция ЗРК
planner-radar-types = Типы РЛС
planner-layer-pulse = Дальность обнаружения импульсной РЛС
planner-layer-doppler = Дальность обнаружения доплеровской РЛС
planner-layer-airfield = Аэродром

## Сохранённые миссии

saves-title = СОХРАНЁННЫЕ МИССИИ
saves-subtitle = Сохраняются в полёте через меню: СОХРАНИТЬ МИССИЮ
saves-none = Нет сохранённых миссий
saves-line = { $theater }  -  { $aircraft }  -  время миссии { $time }
saves-saved-at = Сохранено: { $date }
saves-unreadable = Сохранение не читается (другая версия)
saves-delete =
    .facts = Удалить это сохранение
saves-back =
    .facts = Назад к экрану одиночной миссии
