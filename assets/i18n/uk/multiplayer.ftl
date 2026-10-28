# Вікно кампанії (src/campaign/menu.rs), вікно мультиплеєра (src/multiplayer_menu.rs),
# легенда та жива картина карти війни (src/campaign/picture.rs, unit_view.rs).

delete = ВИДАЛИТИ
delete-campaign = ВИДАЛИТИ ЦЮ КАМПАНІЮ?
delete-save = ВИДАЛИТИ ЦЕ ЗБЕРЕЖЕННЯ?
delete-permanent = Видалення незворотне

## Вікно кампанії

cm-group-campaign = Кампанія
cm-weather-dynamic = Динамічна
    .facts =
        Погода змінюється кожні 10-22 години небесного годинника
        Хмари перекривають пряму видимість обом сторонам
cm-weather-clear = Ясне небо
    .facts = Жодних хмар протягом усієї кампанії
cm-section-day-night = День і ніч
cm-day-real = Реальний час
    .facts = Доба триває 24 години
cm-day-2h = 2 год / доба
    .facts = Доба триває 2 години
cm-day-1h = 1 год / доба
    .facts = Доба триває 1 годину
cm-day-30min = 30 хв / доба
    .facts = Доба триває 30 хвилин
cm-conditions-note = Умови застосовуються до нової кампанії
cm-rules-note = Збережена кампанія зберігає правила, з якими її було розпочато
cm-about =
    Війна триває в реальному часі, поки ви граєте, навіть коли відкрито меню.
    Виконуйте місії з дошки: розвідка, SEAD, CAP, CAS, протикорабельні, удар, супровід.
    Висадка гелікоптерами захоплює міста та аеродроми противника.
    Вас ніколи не вбивають і не беруть у полон: після вильоту ви повертаєтесь до дошки.
cm-section-new = Нова кампанія
cm-new-line = { $theater }  -  { $situation }  -  противники: { $opponents }
cm-start = ПОЧАТИ
cm-section-continue = Продовжити
cm-no-saves = Немає збережених кампаній
cm-played = Награно: { $hours } год
cm-save-gone = Збереження зникло
cm-save-unreadable = Не вдалося прочитати збереження: { $error }
cm-save-undeletable = Не вдалося видалити збереження: { $error }
cm-in-vr = У VR
cm-delete =
    .facts = Видалити цю збережену кампанію

## Карта війни

pic-friendly-aircraft = Дружній літак
pic-friendly-helicopter = Дружній гелікоптер
pic-enemy-fighter = Ворожий винищувач (на радарі)
pic-sam-known = Позиція SAM (відома)
pic-guns-known = Зенітні гармати / MANPADS (відомі)
pic-reconned = Розвідано
pic-you = Ви
pic-site = Позиція: { $place }
uv-live = { $unit } - НАЖИВО
uv-contact-lost = КОНТАКТ ВТРАЧЕНО
uv-hint = Перетягування: обертання  Колесо: масштаб

## Мультиплеєр

mp-disconnected = Відключено: { $why }
mp-connecting = Підключення до { $address }...
mp-joining = Підключено до { $address }, приєднання...
mp-browser-public-only = із браузера можна приєднатися лише до серверів із публічного списку
mp-title-host = МУЛЬТИПЛЕЄР: СТВОРИТИ ГРУ
mp-subtitle = Кооператив у динамічній кампанії: воюйте разом
mp-your-name = Ваше ім'я
    .hint = Ім'я пілота
mp-join-address = Приєднатися за адресою
    .hint = адреса або ім'я[:порт]
mp-connect = ПІДКЛЮЧИТИСЯ
mp-server-name = Назва сервера
mp-udp-port = Порт UDP
mp-address-note = Інші гравці приєднуються за адресою цього комп'ютера; щоб грати через інтернет, відкрийте порт UDP на роутері
mp-list-public = Показувати в публічному списку серверів (інші знайдуть гру у своєму списку серверів)
mp-rules = Правила для всіх
mp-time-of-day = Час доби (місячні ночі)
mp-realistic-weapons = Реалістичне озброєння F-117A
mp-friendly-fire = Дружній вогонь
mp-day-night = День і ніч:
mp-weather = Погода:
mp-host-new = СТВОРИТИ НОВУ
mp-host-saved = Розмістити збережену кампанію
mp-host = СТВОРИТИ
mp-host-game = СТВОРИТИ ГРУ
mp-port-error = Порт UDP { $port }: { $error }
mp-default-server = Кампанія пілота { $pilot }
mp-public-servers = Публічні сервери
mp-refresh = ОНОВИТИ
mp-server-count = Серверів: { $servers }, пілотів: { $pilots }
mp-list-error = Список серверів недоступний: { $error }
mp-no-servers = Наразі немає публічних серверів
mp-col-server = СЕРВЕР
mp-col-pilots = ПІЛОТИ
mp-col-war = ВІЙНА
mp-dedicated = ВИДІЛЕНИЙ
mp-other-version = інша версія
mp-join = ПРИЄДНАТИСЯ
