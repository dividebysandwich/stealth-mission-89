# Экран операций кампании (src/campaign/ops.rs).

ops-thunderstorms = ГРОЗЫ
ops-wx-short = ПОГОДА { $now }, { $next } В { $at }
ops-wx-line = { $weather } В { $at } сменится на: { $next }.
ops-clock = ДЕНЬ { $day }  { $time }
ops-landed-at = ПОСАДКА: { $base }
ops-ground-crew = Наземный экипаж готов
ops-rearm = ПЕРЕВООРУЖИТЬ И ЗАПРАВИТЬ
ops-end-sortie = ЗАВЕРШИТЬ ВЫЛЕТ
ops-on-server = СЕРВЕР { $server }
ops-serving = СЕРВЕР
ops-tab-missions = МИССИИ
ops-tab-map = КАРТА ВОЙНЫ
ops-tab-air-defence = ПВО
ops-tab-forces = СИЛЫ
ops-tab-players = ИГРОКИ
ops-tab-debrief = РАЗБОР ПОЛЁТА
ops-tab-log = ЖУРНАЛ
ops-tab-chat = ЧАТ
ops-blue = СИНИЕ { $count }
ops-red = КРАСНЫЕ { $count }
ops-war-over = Война окончена
ops-running = Идёт симуляция войны
ops-save-exit = СОХРАНИТЬ И ВЫЙТИ
ops-disconnect = ОТКЛЮЧИТЬСЯ
ops-leave-local = СОХРАНИТЬ И ПОКИНУТЬ КАМПАНИЮ?
    .text = Война остановится, пока вы её не продолжите. Самолёты в воздухе будут потеряны; идущие высадки начнутся заново со своей базы.
ops-leave-host = СОХРАНИТЬ И ПРЕКРАТИТЬ РАЗМЕЩЕНИЕ КАМПАНИИ?
    .text = Подключившиеся игроки будут отключены, и война остановится, пока вы её не продолжите. Самолёты в воздухе будут потеряны; идущие высадки начнутся заново со своей базы.
ops-leave-client = ОТКЛЮЧИТЬСЯ ОТ СЕРВЕРА?
    .text = Кампания продолжится на сервере; здесь ничего не сохраняется. Ваш вылет завершится.
ops-unit-destroyed = { $unit } уничтожен: возврат к операциям

## Миссии

ops-section-missions = Миссии
ops-threat = УГРОЗА { $threat }
ops-no-missions = Миссий нет: на фронте затишье
ops-section-underway = Высадки и налёты
ops-enemy-liftoff = ПРОТИВНИК у { $place }: взлёт через { $mins } мин
ops-liftoff = { $place }: взлёт через { $mins } мин
ops-enemy-helis = ПРОТИВНИК у { $place }: на подходе { $count ->
        [one] { $count } вертолёт
        [few] { $count } вертолёта
        [many] { $count } вертолётов
       *[other] { $count } вертолёта
    }
ops-helis = { $place }: в пути { $count ->
        [one] { $count } вертолёт
        [few] { $count } вертолёта
        [many] { $count } вертолётов
       *[other] { $count } вертолёта
    }
ops-raid-sead = SEAD ИИ у { $place }
ops-raid-drones = ДРОНЫ на { $place }
ops-raid = НАЛЁТ на аэродром { $place }
ops-raid-launch = { $raid }: запуск через { $mins } мин
ops-raid-on-way = { $raid }: в пути { $count }
ops-resupply-ai = СНАБЖЕНИЕ ({ $unit }): UH-60 в пути
ops-awacs = AWACS ПРОТИВНИКА в воздухе ещё { $mins } мин
ops-down = СБИТ: { $pilot } - спасение в течение { $mins } мин
ops-aboard = НА БОРТУ: { $pilot } - на пути домой
ops-none-underway = Ничего не происходит
ops-ai-helicopter = Через { $time } её выполнит вертолёт ИИ, если за неё не возьмётся пилот UH-60
ops-defences = Оборона: { $sams ->
        [one] { $sams } позиция SAM
        [few] { $sams } позиции SAM
        [many] { $sams } позиций SAM
       *[other] { $sams } позиции SAM
    }, угроза { $threat }
ops-recon-intel = РАЗВЕДДАННЫЕ
ops-order-insertion = ПРИКАЗАТЬ ВЫСАДКУ
    .facts = Летите в сопровождении, чтобы прикрыть их, или ведите их на UH-60
ops-from = Откуда
ops-planned-by-group = Планирует группа
ops-stores = { $aircraft } - подвеска: { $stores }
ops-fly-uh60 = ЛЕТЕТЬ НА UH-60
ops-accept = ПРИНЯТЬ МИССИЮ
ops-planning = ПЛАНИРУЮТ: { $pilots }
ops-flying = В ВОЗДУХЕ: { $pilots }
ops-package = ПАКЕТ с { $kinds }: выполните здесь задачи двух типов за { $mins } мин, +{ $bonus ->
        [one] { $bonus } очко
        [few] { $bonus } очка
        [many] { $bonus } очков
       *[other] { $bonus } очка
    } за каждую
ops-sead-on-way = SEAD ИИ в пути
ops-request-sead = ЗАПРОСИТЬ SEAD ИИ (матчасть: { $cost }, самолётов: { $aircraft })
    .facts = Дружественные ударные самолёты атакуют позицию SAM этого места раньше вас
    .disabled = Недостаточно матчасти или самолётов на земле (см. СИЛЫ)

## Правила, матчасть, силы

ops-rule-damage = реалистичные повреждения
ops-rule-guidance = реалистичное наведение
ops-rule-no-crashes = без крушений
ops-rule-time = непрерывное время
ops-rule-weapons = реалистичное вооружение F-117A
ops-rule-friendly-fire = огонь по своим
ops-rule-classic = классические
ops-materiel = МАТЧАСТЬ
ops-materiel-blue = СИНИЕ { $materiel } (+{ $income }/ч)
ops-materiel-red = КРАСНЫЕ { $materiel } (+{ $income }/ч)
ops-friendly = СВОИ
ops-enemy = ПРОТИВНИК
ops-section-forces = Силы
ops-places-held = Удерживаемые места
ops-materiel-row = Матчасть
ops-income = Доход в час
ops-income-from = из них
ops-income-sources = { $base } + мест: { $places } x { $per_place } + городов: { $cities } x { $per_city }
ops-fighters-ground = Истребители на земле
ops-helicopters-ground = Вертолёты на земле
ops-in-air = В воздухе
ops-in-air-value = истребителей: { $fighters }, ударных: { $strike }, вертолётов: { $helicopters }
ops-since-start = С начала
ops-earned = заработано
ops-repairs = ремонт
ops-operations = вылеты, налёты, высадки
ops-new-aircraft = новые самолёты
ops-lost = потери (города, пилоты)
ops-section-how = Как это работает
ops-how =
    Каждая сторона каждый час получает матчасть: базовую часть и добавку за каждое удерживаемое место и за каждый уцелевший город среди них. Бомбите города противника (ИЗОЛЯЦИЯ), чтобы урезать его доход.
    ИИ платит за всё, что делает: ремонт, каждый вылет истребителей, налёты и собственные высадки. Без матчасти его повреждения остаются невосстановленными, а самолёты - на земле.
    ИИ летает только на тех самолётах, что у него есть; сбитые потеряны, севшие летают снова. Каждый час каждая сторона покупает новые, пока хватает матчасти.
    Вы ни в чём не нуждаетесь: ваши самолёты, ваши базы (разбомбленные или нет) и вертолёты ваших собственных приказов на высадку всегда на месте.
ops-section-prices = Цены
ops-price-airfield = Ремонт аэродрома
ops-price-sam = Ремонт позиции SAM
ops-price-city = Восстановление города
ops-price-ship = Ремонт корабля
ops-price-gun = Орудие или расчёт MANPADS
ops-price-sortie = Вылет истребителя
ops-price-raid = Налёт
ops-price-insertion = Высадка ИИ
ops-price-fighter = Новый истребитель
ops-price-helicopter = Новый вертолёт
ops-price-city-lost = Город уничтожен (потеря)
ops-price-captured = Пилот в плену (потеря)

## Карта войны

ops-click-to-watch = { $unit } - щёлкните, чтобы наблюдать
ops-section-legend = Условные обозначения
ops-legend-friendly-airfield = Свой аэродром
ops-legend-enemy-airfield = Аэродром противника
ops-legend-city = Свой город / город противника
ops-legend-airfield = Свой / вражеский аэродром
ops-legend-insertion = Маршрут высадки
ops-watch-hint = Щёлкните по подразделению, чтобы наблюдать за ним в реальном времени
ops-map-hint = Колесо: масштаб, перетаскивание: сдвиг
ops-patriot = Батарея Patriot
ops-drone-center = Пункт управления дронами
ops-manned-by = Управляет: { $pilot }
ops-manned-by-other = Управляет другой игрок
ops-control = УПРАВЛЯТЬ
    .facts = Занять её пульт

## Разбор, итоги, журнал, чат

ops-no-sortie = Вылетов пока нет
ops-section-sortie = Вылет
ops-achieved = ЗАДАЧА ВЫПОЛНЕНА
ops-not-achieved = ЗАДАЧА НЕ ВЫПОЛНЕНА
ops-waiting-raid = ОЖИДАНИЕ НАЛЁТА
ops-waiting-insertion = ОЖИДАНИЕ ВЫСАДКИ
ops-flight-time = Время полёта { $mins } мин
ops-kills = Победы: в воздухе { $air }, на земле { $ground }
ops-points = { $points ->
        [one] { $points } очко
        [few] { $points } очка
        [many] { $points } очков
       *[other] { $points } очка
    }
ops-points-bonus = { $points ->
        [one] { $points } очко
        [few] { $points } очка
        [many] { $points } очков
       *[other] { $points } очка
    } (бонус пакета +{ $bonus })
ops-campaign-points = { $rank } - очков кампании: { $points }
ops-campaign-record = Кампания: вылетов { $sorties }, выполнено задач { $won }
ops-section-meanwhile = Тем временем
ops-victory = ПОБЕДА
    .text = У противника не осталось ни одного аэродрома: ТВД наш.
ops-defeat = ПОРАЖЕНИЕ
    .text = Не осталось ни одного своего аэродрома: ТВД потерян.
ops-end-day = День { $day }. Своих мест: { $friendly }, мест противника: { $enemy }.
ops-next-campaign = СЛЕДУЮЩАЯ КАМПАНИЯ НАЧНЁТСЯ ЧЕРЕЗ { $time }
ops-next-campaign-now = СЛЕДУЮЩАЯ КАМПАНИЯ НАЧИНАЕТСЯ
ops-next-from-menu = Начните следующую кампанию из главного меню.
ops-section-log = Журнал кампании
ops-section-chat = Чат
ops-no-messages = Сообщений пока нет
ops-message = Сообщение
    .hint = Введите сообщение, Enter - отправить
ops-send = ОТПРАВИТЬ

## Брифинг

ops-briefing = БРИФИНГ МИССИИ
ops-briefing-line = С базы { $base } - { $km } км до цели - { $aircraft } - подвеска: { $stores }
ops-route-locked = Маршрут заблокирован: его выполняет { $pilots }
ops-route-threats = МАРШРУТ И УГРОЗЫ
ops-take-off = ВЗЛЁТ
ops-radars-shown = Показаны РЛС: обнаруженные своими самолётами и ещё действующие

## Игроки

ops-section-players = Игроки
ops-col-name = ИМЯ
ops-col-status = СОСТОЯНИЕ
ops-col-sorties = ВЫЛЕТЫ
ops-col-points = ОЧКИ
ops-col-pilot = ПИЛОТ
ops-col-objectives = ЗАДАЧИ
ops-col-kills = ПОБЕДЫ ВОЗД./ЗЕМЛ.
ops-col-medal = ЛУЧШАЯ НАГРАДА
ops-you = (вы)
ops-host = ХОСТ
ops-status-planning = Планирует: { $title }
ops-status-board = У доски миссий
ops-status-flying = В воздухе: { $title } - { $aircraft }
ops-status-reconnecting = Связь потеряна - самолёт удерживается { $seconds } с
ops-flown = { $flown } (выполнено { $won })
ops-section-leaderboard = Таблица лидеров
ops-alone = Вы летаете один. Чтобы летать вместе, разместите эту кампанию через МУЛЬТИПЛЕЕР в главном меню.
ops-rules = Правила этой кампании (заданы при её начале): { $rules }
ops-no-sorties = Вылетов пока нет
