# Екран операцій кампанії (src/campaign/ops.rs).

ops-thunderstorms = ГРОЗИ
ops-wx-short = WX { $now }, { $next } О { $at }
ops-wx-line = { $weather } Зміна на { $next } о { $at }.
ops-clock = ДЕНЬ { $day }  { $time }
ops-landed-at = ПОСАДКА: { $base }
ops-ground-crew = Наземна команда готова
ops-rearm = ПЕРЕОЗБРОЇТИ ТА ЗАПРАВИТИ
ops-end-sortie = ЗАВЕРШИТИ ВИЛЬОТ
ops-on-server = НА { $server }
ops-serving = СЕРВЕР ПРАЦЮЄ
ops-tab-missions = МІСІЇ
ops-tab-map = КАРТА ВІЙНИ
ops-tab-air-defence = ПРОТИПОВІТРЯНА ОБОРОНА
ops-tab-forces = СИЛИ
ops-tab-players = ГРАВЦІ
ops-tab-debrief = ДЕБРИФІНГ
ops-tab-log = ЖУРНАЛ
ops-tab-chat = ЧАТ
ops-blue = СИНІ { $count }
ops-red = ЧЕРВОНІ { $count }
ops-war-over = Війну закінчено
ops-running = Симуляція війни працює
ops-save-exit = ЗБЕРЕГТИ Й ВИЙТИ
ops-disconnect = ВІДКЛЮЧИТИСЯ
ops-leave-local = ЗБЕРЕГТИ Й ПОКИНУТИ КАМПАНІЮ?
    .text = Війна зупиняється, доки ви її не продовжите. Літаки в повітрі втрачаються; висадки, що тривають, починаються знову зі своєї бази.
ops-leave-host = ЗБЕРЕГТИ Й ПРИПИНИТИ РОЗМІЩЕННЯ КАМПАНІЇ?
    .text = Гравців, що приєдналися, буде відключено, а війна зупиниться, доки ви її не продовжите. Літаки в повітрі втрачаються; висадки, що тривають, починаються знову зі своєї бази.
ops-leave-client = ВІДКЛЮЧИТИСЯ ВІД СЕРВЕРА?
    .text = Кампанія триває на сервері; тут нічого не зберігається. Ваш вильот завершується.
ops-unit-destroyed = { $unit } знищено: повернення до операцій

## Місії

ops-section-missions = Місії
ops-threat = ЗАГРОЗА { $threat }
ops-no-missions = Місій немає: фронт спокійний
ops-ai-helicopter = Гелікоптер ШІ забере його через { $time }, якщо цього не зробить пілот UH-60
ops-defences = Оборона: позицій SAM: { $sams }, загроза { $threat }
ops-recon-intel = РОЗВІДДАНІ
ops-order-insertion = ЗАМОВИТИ ВИСАДКУ
    .facts = Прикрийте її супроводом або ведіть десант на UH-60 самі
ops-from = Звідки
ops-planned-by-group = Планує група
ops-stores = { $aircraft } - підвіски: { $stores }
ops-fly-uh60 = ПІЛОТУВАТИ UH-60
ops-accept = ПРИЙНЯТИ МІСІЮ
ops-fly-with-ai = ЗЛІТ
ops-take-control = ПЕРЕЙНЯТИ КЕРУВАННЯ
    .facts = Продовжити політ на машині ланки ШІ з того місця, де вона
ops-planning = ПЛАНУЮТЬ: { $pilots }
ops-flying = У ПОЛЬОТІ: { $pilots }
ops-package = PACKAGE з видами: { $kinds }. Виконайте два види тут протягом { $mins } хв, +{ $bonus } балів за кожен
ops-sead-on-way = SEAD від ШІ у дорозі
ops-request-sead = ЗАМОВИТИ SEAD ВІД ШІ ({ $cost } матеріальних засобів, літаків: { $aircraft })
    .facts = Дружні ударні літаки атакують позицію SAM цього місця раніше за вас
    .disabled = Бракує матеріальних засобів або літаків на землі (див. СИЛИ)

## Правила, матеріальні засоби, сили

ops-rule-damage = реалістичні пошкодження
ops-rule-guidance = реалістичне наведення
ops-rule-no-crashes = без катастроф
ops-rule-time = безперервний час
ops-rule-weapons = реалістичне озброєння F-117A
ops-rule-friendly-fire = дружній вогонь
ops-rule-airframe-losses = втрати коштують ресурсів
ops-rule-no-ai-missions = без завдань союзного ШІ
ops-rule-classic = класичні
ops-materiel = МАТЕРІАЛЬНІ ЗАСОБИ
ops-materiel-blue = СИНІ { $materiel } (+{ $income }/год)
ops-materiel-red = ЧЕРВОНІ { $materiel } (+{ $income }/год)
ops-crew-tempo = ТЕМП ВОРОГА ×{ $pace }
ops-due = T-{ $mins } хв
ops-due-now = ЗАРАЗ
ops-front-blue = ФРОНТ: СИНІ НАСТУПАЮТЬ
ops-front-red = ФРОНТ: ЧЕРВОНІ НАСТУПАЮТЬ
ops-front-even = ФРОНТ: ТРИМАЄТЬСЯ
ops-operation = Захоплення { $place }:
ops-friendly = СВОЇ
ops-enemy = ПРОТИВНИК
ops-section-forces = Сили
ops-places-held = Утримувані пункти
ops-materiel-row = Матеріальні засоби
ops-income = Надходження за годину
ops-income-from = із
ops-income-sources = { $base } + пунктів: { $places } x { $per_place } + міст: { $cities } x { $per_city }
ops-fighters-ground = Винищувачі на землі
ops-helicopters-ground = Гелікоптери на землі
ops-in-air = У повітрі
ops-in-air-value = винищувачів: { $fighters }, ударних: { $strike }, гелікоптерів: { $helicopters }
ops-since-start = Від початку
ops-earned = зароблено
ops-repairs = ремонт
ops-operations = вильоти, рейди, висадки
ops-new-aircraft = нові літаки
ops-lost = втрати (міста, пілоти)
ops-section-how = Як це працює
ops-how =
    Кожна сторона щогодини отримує матеріальні засоби: базову суму, більше за кожен утримуваний пункт і за кожне неушкоджене місто серед них. Бомбіть міста противника (INTERDICTION), щоб скоротити його надходження.
    ШІ платить за все, що робить: ремонт, кожен вильот винищувачів, рейди та власні висадки. Якщо матеріальних засобів бракує, його пошкодження лишаються неусунутими, а літаки залишаються вдома.
    ШІ літає лише на тих літаках, що в нього є; збиті зникають, ті, що сіли, літають знову. Щогодини кожна сторона купує нові, поки вистачає матеріальних засобів.
    Вам вистачає завжди: ваші літаки, ваші бази (розбомблені чи ні) та гелікоптери ваших власних замовлень на висадку завжди на місці.
ops-how-repairs = Кожна сторона ремонтує водночас лише кілька позицій (більше за більшої кількості цілих міст), спершу найближчі до фронту, а позиція, знову зруйнована невдовзі після ремонту, щоразу ремонтується довше.
ops-how-airframes = Втрати коштують ресурсів: кожна машина, втрачена пілотом, це на один винищувач або гелікоптер менше у свого ШІ.
ops-section-prices = Ціни
ops-price-airfield = Ремонт аеродрому
ops-price-sam = Ремонт позиції SAM
ops-price-city = Відбудова міста
ops-price-ship = Ремонт корабля
ops-price-gun = Гармата або розрахунок MANPADS
ops-price-sortie = Вильот винищувачів
ops-price-raid = Рейд
ops-price-insertion = Висадка від ШІ
ops-price-fighter = Новий винищувач
ops-price-helicopter = Новий гелікоптер
ops-price-city-lost = Місто знищено (втрата)
ops-price-captured = Пілота взято в полон (втрата)

## Карта війни

ops-click-to-watch = { $unit } - клікніть, щоб стежити
ops-section-legend = Легенда
ops-legend-friendly-airfield = Свій аеродром
ops-legend-enemy-airfield = Аеродром противника
ops-legend-city = Місто: своє / противника
ops-legend-airfield = Свій / ворожий аеродром
ops-legend-insertion = Маршрут висадки
ops-watch-hint = Клікніть на одиницю, щоб стежити за нею наживо
ops-map-hint = Колесо: масштаб, перетягування: зсув
ops-patriot = Батарея Patriot
ops-nasams = Батарея NASAMS
ops-manned-by = Обслуговує: { $pilot }
ops-manned-by-other = Обслуговує інший гравець
ops-control = КЕРУВАННЯ
    .facts = Перейняти її консоль

## Дебрифінг, кінець, журнал, чат

ops-no-sortie = Вильотів ще не було
ops-section-sortie = Вильот
ops-achieved = ЗАВДАННЯ ВИКОНАНО
ops-not-achieved = ЗАВДАННЯ НЕ ВИКОНАНО
ops-waiting-raid = ОЧІКУВАННЯ РЕЙДУ
ops-waiting-insertion = ОЧІКУВАННЯ ВИСАДКИ
ops-flight-time = Час польоту: { $mins } хв
ops-kills = Знищено: літаків { $air }, на землі { $ground }
ops-points = Балів: { $points }
ops-points-bonus = Балів: { $points } (бонус за package +{ $bonus })
ops-campaign-points = { $rank } - балів кампанії: { $points }
ops-campaign-record = Кампанія: вильотів { $sorties }, виконаних завдань { $won }
ops-section-meanwhile = Тим часом
ops-section-effects = Твій вплив на війну
ops-victory = ПЕРЕМОГА
    .text = Противник більше не тримає жодного аеродрому: театр воєнних дій наш.
ops-defeat = ПОРАЗКА
    .text = Не лишилося жодного свого аеродрому: театр воєнних дій втрачено.
ops-end-day = День { $day }. Наші пункти: { $friendly }, пункти противника: { $enemy }.
ops-next-campaign = НАСТУПНА КАМПАНІЯ ПОЧНЕТЬСЯ ЧЕРЕЗ { $time }
ops-next-campaign-now = НАСТУПНА КАМПАНІЯ ПОЧИНАЄТЬСЯ
ops-next-from-menu = Розпочніть наступну кампанію з головного меню.
ops-section-log = Журнал кампанії
ops-section-chat = Чат
ops-no-messages = Повідомлень ще немає
ops-message = Повідомлення
    .hint = Введіть повідомлення, Enter надсилає
ops-send = НАДІСЛАТИ

## Брифінг

ops-briefing = БРИФІНГ МІСІЇ
ops-briefing-line = З { $base } - до цілі { $km } км - { $aircraft } - підвіски: { $stores }
ops-route-locked = Маршрут заблоковано: летять { $pilots }
ops-route-threats = МАРШРУТ І ЗАГРОЗИ
ops-take-off = ЗЛІТ
ops-radars-shown = Показано радари, виявлені дружніми літаками й досі діючі

## Гравці

ops-section-players = Гравці
ops-col-name = ІМ'Я
ops-col-status = СТАТУС
ops-col-sorties = ВИЛЬОТИ
ops-col-points = БАЛИ
ops-col-pilot = ПІЛОТ
ops-col-objectives = ЗАВДАННЯ
ops-col-kills = ЗНИЩЕНО ПОВ./НАЗ.
ops-col-medal = НАЙКРАЩА МЕДАЛЬ
ops-you = (ви)
ops-host = ХОСТ
ops-status-planning = Планує: { $title }
ops-status-board = На дошці місій
ops-status-flying = У польоті: { $title } - { $aircraft }
ops-status-reconnecting = З'єднання втрачено - літак утримується { $seconds } с
ops-flown = { $flown } (виконано: { $won })
ops-section-leaderboard = Таблиця лідерів
ops-alone = Ви літаєте самі. Щоб літати разом, розмістіть цю кампанію через MULTIPLAYER у головному меню.
ops-rules = Правила цієї кампанії (зберігаються з її початку): { $rules }
ops-no-sorties = Вильотів ще не було
ops-ai-load = Гелікоптер ШІ забере вантаж через { $time }, якщо цього не зробить пілот UH-60
ops-ai-takes-off = Зліт через { $time }
ops-ai-aircraft = ШІ { $aircraft }
ops-cargo-in = Вантаж { $delivered } з { $loads } доставлено - майданчик чекає ще { $time } год
ops-rule-classic-terrain = класичний рельєф
ops-rule-classic-ai-flight = класичний політ ШІ
ops-layers-hint = ШАРИ (вгорі праворуч): що показує карта
ops-map-tab = КАРТА
ops-on-radio = У РАДІО
ops-mute = ВИМКНУТИ ЗВУК
    .facts = Чути (чи ні) цього гравця в радіо
ops-unmute = УВІМКНУТИ ЗВУК
