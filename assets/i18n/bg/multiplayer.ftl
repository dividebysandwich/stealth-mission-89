# Прозорецът на кампанията (src/campaign/menu.rs), прозорецът за мултиплейър (src/multiplayer_menu.rs),
# легендата и живата гледка на картата на войната (src/campaign/picture.rs, unit_view.rs).

delete = ИЗТРИЙ
delete-campaign = ДА СЕ ИЗТРИЕ ЛИ ТАЗИ КАМПАНИЯ?
delete-save = ДА СЕ ИЗТРИЕ ЛИ ТОЗИ ЗАПИС?
delete-permanent = Изтриването е необратимо

## Прозорец на кампанията

cm-group-campaign = Кампания
cm-weather-dynamic = Динамично
    .facts =
        Времето се променя на всеки 10-22 часа по часовника на небето
        Облаците блокират линията на видимост и за двете страни
cm-weather-clear = Ясно небе
    .facts = Без облаци през цялата кампания
cm-section-day-night = Ден и нощ
cm-day-real = Реално време
    .facts = Денят трае 24 часа
cm-day-2h = 2 ч / ден
    .facts = Денят трае 2 часа
cm-day-1h = 1 ч / ден
    .facts = Денят трае 1 час
cm-day-30min = 30 мин / ден
    .facts = Денят трае 30 минути
cm-conditions-note = Условията важат за нова кампания
cm-rules-note = Записаната кампания запазва правилата, с които е започнала
cm-about =
    Войната върви в реално време, докато играете, дори когато менюто е отворено.
    Изпълнявайте мисиите от таблото: разузнаване, SEAD, CAP, CAS, противокорабна, удар, ескорт.
    Хеликоптерните десанти превземат градовете и летищата на врага.
    Никога не бивате убит или заловен: след полет се връщате на таблото.
cm-section-new = Нова кампания
cm-new-line = { $theater }  -  { $situation }  -  противници { $opponents }
cm-start = СТАРТ
cm-section-continue = Продължаване
cm-no-saves = Няма записани кампании
cm-played = { $hours } ч игра
cm-save-gone = Записът го няма
cm-save-unreadable = Записът не можа да бъде прочетен: { $error }
cm-save-undeletable = Записът не можа да бъде изтрит: { $error }
cm-in-vr = VR
cm-delete =
    .facts = Изтрийте тази записана кампания

## Карта на войната

pic-friendly-aircraft = Наш самолет
pic-friendly-helicopter = Наш хеликоптер
pic-enemy-fighter = Вражески изтребител (на радара)
pic-sam-known = SAM позиция (известна)
pic-guns-known = Оръдия / MANPADS (известни)
pic-reconned = Разузнато
pic-you = Вие
pic-site = Обект { $place }
uv-live = { $unit } - НА ЖИВО
uv-contact-lost = КОНТАКТЪТ Е ЗАГУБЕН
uv-hint = Влачене: въртене  Колелце: мащаб

## Мултиплейър

mp-disconnected = Връзката е прекъсната: { $why }
mp-connecting = Свързване с { $address }...
mp-joining = Свързан с { $address }, присъединяване...
mp-browser-public-only = от браузъра може да се влезе само в сървъри от публичния списък
mp-title-host = МНОГО ИГРАЧИ: СЪЗДАЙТЕ ИГРА
mp-subtitle = Коопе в динамичната кампания: водете войната заедно
mp-your-name = Вашето име
    .hint = Име на пилота
mp-join-address = Влизане по адрес
    .hint = адрес или име[:порт]
mp-connect = СВЪРЖИ СЕ
mp-server-name = Име на сървъра
mp-udp-port = UDP порт
mp-address-note = Другите играчи влизат с адреса на този компютър; отворете UDP порта в рутера си, за да играете през интернет
mp-list-public = Покажи в публичния списък със сървъри (останалите намират играта в списъка си)
mp-rules = Правила за всички
mp-time-of-day = Часът от денонощието (лунни нощи)
mp-realistic-weapons = Реалистично въоръжение на F-117A
mp-friendly-fire = Огън по свои
mp-day-night = Ден и нощ:
mp-weather = Време:
mp-host-new = НОВА ИГРА
mp-host-saved = Стартирайте записана кампания
mp-host = СТАРТИРАЙ
mp-host-game = СЪЗДАЙ ИГРА
mp-port-error = UDP порт { $port }: { $error }
mp-default-server = Кампанията на { $pilot }
mp-public-servers = Публични сървъри
mp-refresh = ОБНОВИ
mp-server-count = { $servers } сървъра, { $pilots } пилота
mp-list-error = Списъкът със сървъри е недостъпен: { $error }
mp-no-servers = В момента няма публични сървъри
mp-col-server = СЪРВЪР
mp-col-pilots = ПИЛОТИ
mp-col-war = ВОЙНА
mp-dedicated = ДЕДИКИРАН
mp-other-version = друга версия
mp-join = ВЛЕЗ
