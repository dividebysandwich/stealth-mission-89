# Екранът Ops на кампанията (src/campaign/ops.rs).

ops-thunderstorms = ГРЪМОТЕВИЧНИ БУРИ
ops-wx-short = WX { $now }, { $next } В { $at }
ops-wx-line = { $weather } Преминава в { $next } в { $at }.
ops-clock = ДЕН { $day }  { $time }
ops-landed-at = КАЦНАЛ В { $base }
ops-ground-crew = Наземният екип е готов
ops-rearm = ПРЕВЪОРЪЖАВАНЕ И ЗАРЕЖДАНЕ
ops-end-sortie = КРАЙ НА ПОЛЕТА
ops-on-server = НА { $server }
ops-serving = ДОМАКИНСТВАНЕ
ops-tab-missions = МИСИИ
ops-tab-map = КАРТА НА ВОЙНАТА
ops-tab-air-defence = ПРОТИВОВЪЗДУШНА ОТБРАНА
ops-tab-forces = СИЛИ
ops-tab-players = ИГРАЧИ
ops-tab-debrief = ДЕБРИФИНГ
ops-tab-log = ДНЕВНИК
ops-tab-chat = ЧАТ
ops-blue = СИНИ { $count }
ops-red = ЧЕРВЕНИ { $count }
ops-war-over = Войната приключи
ops-running = Симулацията на войната върви
ops-save-exit = ЗАПИС И ИЗХОД
ops-disconnect = ИЗКЛЮЧВАНЕ
ops-leave-local = ДА СЕ ЗАПИШЕ ЛИ И НАПУСНЕ КАМПАНИЯТА?
    .text = Войната спира, докато не я продължите. Самолетите във въздуха се губят; десантите в ход започват отначало от базата си.
ops-leave-host = ДА СЕ ЗАПИШЕ ЛИ И СПРЕ ДОМАКИНСТВОТО НА КАМПАНИЯТА?
    .text = Присъединилите се играчи се изключват и войната спира, докато не я продължите. Самолетите във въздуха се губят; десантите в ход започват отначало от базата си.
ops-leave-client = ДА СЕ ИЗКЛЮЧИТЕ ЛИ ОТ СЪРВЪРА?
    .text = Кампанията продължава на сървъра; тук нищо не се записва. Полетът ви приключва.
ops-unit-destroyed = { $unit } унищожен: обратно към операциите

## Мисии

ops-section-missions = Мисии
ops-threat = ЗАПЛАХА { $threat }
ops-no-missions = Няма мисии: фронтът е спокоен
ops-section-underway = Десанти и набези
ops-enemy-liftoff = ВРАГ при { $place }: излитане след { $mins } мин
ops-liftoff = { $place }: излитане след { $mins } мин
ops-enemy-helis = ВРАГ при { $place }: приближават { $count } хеликоптера
ops-helis = { $place }: { $count } хеликоптера на път
ops-raid-sead = SEAD с ИИ при { $place }
ops-raid-drones = ДРОНОВЕ при { $place }
ops-raid = НАБЕГ срещу летище { $place }
ops-raid-launch = { $raid }: старт след { $mins } мин
ops-raid-on-way = { $raid }: { $count } на път
ops-resupply-ai = ПРЕЗАРЕЖДАНЕ на { $unit }: UH-60 на път
ops-awacs = ВРАЖЕСКИ AWACS във въздуха още { $mins } мин
ops-down = СВАЛЕН: { $pilot } - спасяване до { $mins } мин
ops-aboard = НА БОРДА: { $pilot } - на път към дома
ops-none-underway = Няма в ход
ops-ai-helicopter = Хеликоптер с ИИ го поема след { $time }, ако пилот на UH-60 не го направи
ops-defences = Отбрана: { $sams } SAM позиции, заплаха { $threat }
ops-recon-intel = РАЗУЗНАВАТЕЛНИ ДАННИ
ops-order-insertion = НАРЕДИ ДЕСАНТ
    .facts = Изпратете ескорт да ги прикрива или ги водете с UH-60
ops-from = От
ops-planned-by-group = Планирано от групата
ops-stores = { $aircraft } - въоръжение: { $stores }
ops-fly-uh60 = ЛЕТЕТЕ С UH-60
ops-accept = ПРИЕМИ МИСИЯТА
ops-planning = ПЛАНИРАНЕ: { $pilots }
ops-flying = В ПОЛЕТ: { $pilots }
ops-package = ПАКЕТ с { $kinds }: изпълнете два вида тук до { $mins } мин, +{ $bonus } точки за всеки
ops-sead-on-way = SEAD с ИИ е на път
ops-request-sead = ЗАЯВИ SEAD С ИИ ({ $cost } материали, { $aircraft } самолета)
    .facts = Наши ударни самолети атакуват SAM позиция на това място преди вас
    .disabled = Няма достатъчно материали или самолети на земята (виж СИЛИ)

## Правила, материали, сили

ops-rule-damage = реалистични повреди
ops-rule-guidance = реалистично насочване
ops-rule-no-crashes = без катастрофи
ops-rule-time = непрекъснато време
ops-rule-weapons = реалистично въоръжение на F-117A
ops-rule-friendly-fire = огън по свои
ops-rule-classic = класически
ops-materiel = МАТЕРИАЛИ
ops-materiel-blue = СИНИ { $materiel } (+{ $income }/ч)
ops-materiel-red = ЧЕРВЕНИ { $materiel } (+{ $income }/ч)
ops-friendly = СВОИ
ops-enemy = ВРАГ
ops-section-forces = Сили
ops-places-held = Държани места
ops-materiel-row = Материали
ops-income = Приход на час
ops-income-from = от
ops-income-sources = { $base } + { $places } места x { $per_place } + { $cities } града x { $per_city }
ops-fighters-ground = Изтребители на земята
ops-helicopters-ground = Хеликоптери на земята
ops-in-air = Във въздуха
ops-in-air-value = { $fighters } изтребители, { $strike } ударни, { $helicopters } хеликоптера
ops-since-start = От началото
ops-earned = спечелени
ops-repairs = ремонти
ops-operations = полети, набези, десанти
ops-new-aircraft = нови самолети
ops-lost = загубени (градове, пилоти)
ops-section-how = Как работи
ops-how =
    Всяка страна печели материали всеки час: основна сума, повече за всяко държано място и за всеки здрав град между тях. Бомбардирайте градовете на врага (ИНТЕРДИКЦИЯ), за да намалите прихода му.
    ИИ плаща за всичко, което прави: ремонти, всеки изтребителен полет, набези и собствените си десанти. Без материали повредите му остават неремонтирани, а самолетите му стоят у дома.
    ИИ лети само със самолетите, които има; свалените изчезват, кацналите летят отново. Всеки час всяка страна купува нови, докато стигат материалите.
    Вие никога не оставате без: вашите самолети, вашите бази (бомбардирани или не) и хеликоптерите на собствените ви заповеди за десант винаги са налични.
ops-section-prices = Цени
ops-price-airfield = Ремонт на летище
ops-price-sam = Ремонт на SAM позиция
ops-price-city = Възстановяване на град
ops-price-ship = Ремонт на кораб
ops-price-gun = Оръдие или екип MANPADS
ops-price-sortie = Изтребителен полет
ops-price-raid = Набег
ops-price-insertion = Десант с ИИ
ops-price-fighter = Нов изтребител
ops-price-helicopter = Нов хеликоптер
ops-price-city-lost = Унищожен град (загуба)
ops-price-captured = Заловен пилот (загуба)

## Карта на войната

ops-click-to-watch = { $unit } - щракнете за наблюдение
ops-section-legend = Легенда
ops-legend-friendly-airfield = Наше летище
ops-legend-enemy-airfield = Вражеско летище
ops-legend-city = Наш / вражески град
ops-watch-hint = Щракнете върху част, за да я наблюдавате на живо
ops-map-hint = Колелце: мащаб, влачене: преместване
ops-patriot = Батарея Patriot
ops-manned-by = Обслужва се от { $pilot }
ops-manned-by-other = Обслужва се от друг играч
ops-control = УПРАВЛЕНИЕ
    .facts = Поемете конзолата ѝ

## Дебрифинг, край, дневник, чат

ops-no-sortie = Още няма изпълнен полет
ops-section-sortie = Полет
ops-achieved = ЦЕЛТА Е ПОСТИГНАТА
ops-not-achieved = ЦЕЛТА НЕ Е ПОСТИГНАТА
ops-waiting-raid = ОЧАКВА СЕ НАБЕГЪТ
ops-waiting-insertion = ОЧАКВА СЕ ДЕСАНТЪТ
ops-flight-time = Време на полета { $mins } мин
ops-kills = Поразени: { $air } самолета, { $ground } на земята
ops-points = { $points } точки
ops-points-bonus = { $points } точки (бонус за пакет +{ $bonus })
ops-campaign-points = { $rank } - { $points } точки в кампанията
ops-campaign-record = Кампания: { $sorties } полета, { $won } постигнати цели
ops-section-meanwhile = Междувременно
ops-victory = ПОБЕДА
    .text = Врагът вече не държи нито едно летище: театърът на военните действия е наш.
ops-defeat = ПОРАЖЕНИЕ
    .text = Не е останало нито едно наше летище: театърът на военните действия е загубен.
ops-end-day = Ден { $day }. Наши места { $friendly }, вражески места { $enemy }.
ops-next-campaign = СЛЕДВАЩАТА КАМПАНИЯ ЗАПОЧВА СЛЕД { $time }
ops-next-campaign-now = СЛЕДВАЩАТА КАМПАНИЯ ЗАПОЧВА
ops-next-from-menu = Започнете следващата кампания от главното меню.
ops-section-log = Дневник на кампанията
ops-section-chat = Чат
ops-no-messages = Още няма съобщения
ops-message = Съобщение
    .hint = Въведете съобщение, Enter го изпраща
ops-send = ИЗПРАТИ

## Брифинг

ops-briefing = БРИФИНГ ЗА МИСИЯТА
ops-briefing-line = От { $base } - { $km } км до целта - { $aircraft } - въоръжение: { $stores }
ops-route-locked = Маршрутът е заключен: лети { $pilots }
ops-route-threats = МАРШРУТ И ЗАПЛАХИ
ops-take-off = ИЗЛИТАНЕ
ops-radars-shown = Показани радари: засечени от наши самолети, още действащи

## Играчи

ops-section-players = Играчи
ops-col-name = ИМЕ
ops-col-status = СЪСТОЯНИЕ
ops-col-sorties = ПОЛЕТИ
ops-col-points = ТОЧКИ
ops-col-pilot = ПИЛОТ
ops-col-objectives = ЦЕЛИ
ops-col-kills = ПОРАЗЕНИ ВЪЗД./ЗЕМЯ
ops-col-medal = НАЙ-ДОБЪР МЕДАЛ
ops-you = (вие)
ops-host = ДОМАКИН
ops-status-planning = Планира { $title }
ops-status-board = На таблото с мисии
ops-status-flying = Лети { $title } - { $aircraft }
ops-status-reconnecting = Връзката е загубена - самолетът се задържа { $seconds } s
ops-flown = { $flown } ({ $won } постигнати)
ops-section-leaderboard = Класиране
ops-alone = Летите сам. За да летите заедно, бъдете домакин на тази кампания от МНОГО ИГРАЧИ в главното меню.
ops-rules = Правила на тази кампания (запазени от началото ѝ): { $rules }
ops-no-sorties = Още няма изпълнени полети
