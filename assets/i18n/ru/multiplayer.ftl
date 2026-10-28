# Окно кампании, окно мультиплеера, условные обозначения карты войны и наблюдение в реальном
# времени.

delete = УДАЛИТЬ
delete-campaign = УДАЛИТЬ ЭТУ КАМПАНИЮ?
delete-save = УДАЛИТЬ ЭТО СОХРАНЕНИЕ?
delete-permanent = Удаление необратимо

## Окно кампании

cm-group-campaign = Кампания
cm-weather-dynamic = Динамическая
    .facts =
        Погода меняется каждые 10-22 часа по небесным часам
        Облака закрывают обзор обеим сторонам
cm-weather-clear = Ясное небо
    .facts = Ни одного облака за всю кампанию
cm-section-day-night = День и ночь
cm-day-real = Реальное время
    .facts = Сутки длятся 24 часа
cm-day-2h = 2 ч / сутки
    .facts = Сутки длятся 2 часа
cm-day-1h = 1 ч / сутки
    .facts = Сутки длятся 1 час
cm-day-30min = 30 мин / сутки
    .facts = Сутки длятся 30 минут
cm-conditions-note = Условия применяются к новой кампании
cm-rules-note = Сохранённая кампания сохраняет правила, с которыми была начата
cm-about =
    Война идёт в реальном времени, пока вы играете, в том числе при открытом меню.
    Выполняйте миссии с доски: разведка, SEAD, CAP, CAS, противокорабельные, удар, сопровождение.
    Вертолётные высадки захватывают города и аэродромы противника.
    Вы никогда не погибаете и не попадаете в плен: после вылета вы снова у доски миссий.
cm-section-new = Новая кампания
cm-new-line = { $theater }  -  { $situation }  -  противники: { $opponents }
cm-start = НАЧАТЬ
cm-section-continue = Продолжить
cm-no-saves = Нет сохранённых кампаний
cm-played = { $hours } ч в игре
cm-save-gone = Сохранение больше не существует
cm-save-unreadable = Не удалось прочитать сохранение: { $error }
cm-save-undeletable = Не удалось удалить сохранение: { $error }
cm-in-vr = В VR
cm-delete =
    .facts = Удалить эту сохранённую кампанию

## Карта войны

pic-friendly-aircraft = Свой самолёт
pic-friendly-helicopter = Свой вертолёт
pic-enemy-fighter = Истребитель противника (на радаре)
pic-sam-known = Позиция SAM (известна)
pic-guns-known = Орудия / MANPADS (известны)
pic-reconned = Разведано
pic-you = Вы
pic-site = Позиция { $place }
uv-live = { $unit } - ПРЯМОЙ ЭФИР
uv-contact-lost = КОНТАКТ ПОТЕРЯН
uv-hint = Перетаскивание: облёт  Колесо: масштаб

## Мультиплеер

mp-disconnected = Отключено: { $why }
mp-connecting = Подключение к { $address }...
mp-joining = Подключено к { $address }, вход в игру...
mp-browser-public-only = из браузера можно подключиться только к серверам из общего списка
mp-title-host = МУЛЬТИПЛЕЕР: СОЗДАТЬ ИГРУ
mp-subtitle = Совместное прохождение динамической кампании: воюйте вместе
mp-your-name = Ваше имя
    .hint = Имя пилота
mp-join-address = Подключиться по адресу
    .hint = адрес или имя[:порт]
mp-connect = ПОДКЛЮЧИТЬСЯ
mp-server-name = Название сервера
mp-udp-port = Порт UDP
mp-address-note = Другие игроки подключаются по адресу этого компьютера; чтобы играть через интернет, откройте порт UDP на роутере
mp-list-public = Показывать в общем списке серверов (другие найдут игру в своём списке)
mp-rules = Правила для всех
mp-time-of-day = Время суток (лунные ночи)
mp-realistic-weapons = Реалистичное вооружение F-117A
mp-friendly-fire = Огонь по своим
mp-day-night = День и ночь:
mp-weather = Погода:
mp-host-new = СОЗДАТЬ НОВУЮ
mp-host-saved = Разместить сохранённую кампанию
mp-host = РАЗМЕСТИТЬ
mp-host-game = СОЗДАТЬ ИГРУ
mp-port-error = Порт UDP { $port }: { $error }
mp-default-server = Кампания { $pilot }
mp-public-servers = Общие серверы
mp-refresh = ОБНОВИТЬ
mp-server-count = серверов: { $servers }, пилотов: { $pilots }
mp-list-error = Список серверов недоступен: { $error }
mp-no-servers = Сейчас нет общих серверов
mp-col-server = СЕРВЕР
mp-col-pilots = ПИЛОТЫ
mp-col-war = ВОЙНА
mp-dedicated = ВЫДЕЛЕННЫЙ
mp-other-version = другая версия
mp-join = ПОДКЛЮЧИТЬСЯ
