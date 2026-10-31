# Прозорецът ОПЦИИ (src/options.rs) и страниците му в менюто по време на полет.

options-title = ОПЦИИ
options-tab-game = ИГРА
options-tab-graphics = ГРАФИКА
options-tab-sound = ЗВУК
options-tab-controls = УПРАВЛЕНИЕ
options-footer = Запазват се за следващите полети; превключвателите за реализъм се избират при всяка игра

## ИГРА

options-section-language = Език
options-section-classic-modern = Класически / Съвременен
options-cockpit = Кабина
    .facts =
        Съвременна: дисплеи с висока разделителна способност с бутони по рамката
        Класическа: разположението на приборното табло от времето на DOS и дисплеи 77x60 (F-117A)
options-hud = HUD
    .facts =
        Съвременен: зелен HUD с висока разделителна способност в стила на F-16, с режими NAV, AA и AG
        Класически: HUD от времето на DOS
options-avionics = Авионика
    .facts =
        Съвременна: формати на MFD в стила на F-16 на който и да е MFD (обстановка, HARM, контейнер за прицелване, карта, оборудване) с подвижен курсор
        Класическа: карта и TAC отляво, камера, WAY, WPN и ORD отдясно
options-flight-model = Модел на полета
    .facts =
        Съвременен: аеродинамичен модел на полета с fly-by-wire: ръчката задава скоростта на тангаж и крен; при пусната ръчка носът остава на място и траекторията се поддържа
        Класически: модел на полета от времето на DOS
options-section-flying = Пилотиране
options-auto-trim = Автоматичен трим
    .facts =
        При пусната ръчка тримът държи положението, в което сте я оставили
        Хеликоптери: носът вече не се вдига със скоростта; самолети: тримира повредените управляващи повърхности
        Ръчен трим: Alt + стрелките, Alt + Home го центрира
options-tooltips = Подсказки в кабината
    .facts = Какво е даден орган за управление или дисплей в кабината, под мишката или връхчето на пръста във VR
options-player-labels = Имена на играчите
    .facts = Мултиплейър: имената на другите пилоти над самолетите им, от всяко разстояние
options-label-size =
    .facts =
        Размер на имената на играчите
        50%: по подразбиране
options-section-vr-size = Размер на кабината във VR
options-vr-size =
    .facts =
        Размерът на кабината (и на света) в шлема
        100%: в естествен размер
options-section-gif = Запис на GIF
options-gif-size = РАЗМЕР
options-gif-rate = КАДРИ
options-gif-scale-100 =
    .facts = Пълната резолюция на прозореца: големи файлове
options-gif-scale-25 =
    .facts = Малки файлове
options-gif-rate-50 =
    .facts = Само когато играта върви със 100 fps или повече (записва се всеки втори кадър)
options-gif-hint = F12 (ако не е пренастроен) пуска и спира; записва се в папката „Снимки“

## ГРАФИКА

options-section-rendering = Изобразяване
options-shaders = Шейдъри
    .facts =
        Съвременни: фино насочено засенчване, плавна мараня в далечината
        Класически: плоско засенчване
options-shadows = Сенки
    .facts =
        Съвременни: слънчеви сенки в кабината и в света (натоварва графичния процесор)
        Класически: без сенки
options-point-lights = Точкови светлини
    .facts =
        Съвременни: осветление на кабината и дисплеите; експлозиите, струите и светлините на пистата осветяват сцената
        Класически: само светлина от слънцето и небето
options-time-of-day = Време на денонощието
    .facts =
        Съвременно: слънцето и луната се движат според часовника на мисията, небето потъмнява и изсветлява със слънцето; звезди и лунна фаза нощем
        Съвременно: лунната светлина определя от колко далеч вражеските пилоти ви виждат нощем (2 км в безлунна нощ, 4 км при пълнолуние)
        Класическо: бавна смяна на цветовете и неподвижно слънце
options-msaa = Изглаждане (MSAA)
    .facts =
        Гладки краища: 4x MSAA, 2x във VR (натоварва графичния процесор)
        Изключено: груби пикселни краища
options-auto-detail = Автоматична детайлност на околността (VR)
    .facts =
        В шлема обхватът на градовете, горите и нивите се свива, докато кадрите закъсняват, и се връща, когато има резерв от време
        Графичните настройки остават както са избрани
options-sun-moon = Слънце и луна
    .facts =
        Слънцето на небето денем, луната и звездите нощем
        Луната показва фазата на мисията

## ЗВУК

options-section-volume = Сила на звука
options-volume-master = Общо
    .facts = Всички звуци
options-volume-engine = Двигател
    .facts = Вашият двигател или ротор и самолетите, чути около вас
options-volume-effects = Ефекти
    .facts = Оръжия, експлозии, колесник, радио и останалото
options-volume-warnings = Предупреждения
    .facts = Сирената за сваляне, радарните и ракетните предупреждения, сирената за пожар, тоновете на Sidewinder
options-volume-rwr = RWR
    .facts = Тоновете на радарното предупреждение, като част от силата на предупрежденията
options-section-rwr-tones = Тонове на радарното предупреждение
rwr-tones-all = ВСИЧКИ
    .facts = Радар, който ви облъчва, всяко обхождане, което ви засича, изстрелвания на ракети
rwr-tones-detect = САМО ЗАСИЧАНЕ
    .facts =
        Един тон, когато радар ви засече (отново, след като радарите ви изпуснат за известно време)
        И изстрелванията на ракети
rwr-tones-launch = САМО ИЗСТРЕЛВАНИЯ
    .facts = Само ракети, изстреляни срещу вас
rwr-tones-off = ИЗКЛ
    .facts = Без тонове на радарно предупреждение (лампите и дисплеите пак предупреждават)
options-section-cockpit = Кабина
options-low-pass = Нискочестотен филтър на двигателя
    .facts =
        Вътре в самолета двигателят звучи приглушено (фенерът и шлемът отнемат високите честоти)
        Изключен: същото като отвън

## УПРАВЛЕНИЕ

options-section-bindings = Назначения
options-keys-joystick = КЛАВИШИ И ДЖОЙСТИК
    .facts =
        Клавиши, оси и бутони на джойстик и HOTAS
        Запазват се за следващите полети
options-section-head-tracking = Проследяване на главата/очите
options-head-tracking = Проследяване на главата/очите
    .facts =
        Главата ви върти и мести гледката в кабината (на компютър; във VR това прави шлемът)
        Програма за проследяване (OpenTrack...): изход FreeTrack или UDP към 127.0.0.1 порт 4242
        Home: текущото положение на главата става център
options-tracking = Проследяване ({ $source })
options-no-tracker = Не се чува тракер: стартирайте програмата за проследяване

## EFFECTS

options-section-effects = Effects
options-section-retro = Retro
effect-shake = Camera shake
    .facts =
        The view shakes with the gun, nearby blasts, stall buffet, touchdowns and the runway
        VR: the cockpit vibrates, your head stays still
effect-flashes = Combat flashes
    .facts =
        Flak bursts, explosions and missile launches light the clouds, the ground and the cockpit for a moment
effect-barrage = Night barrage
    .facts =
        Conventional war at night: the towns' guns fire tracer blindly into the sky, searchlights sweep over them
        Only for the look: the barrage hits nothing, the searchlights find nothing
effect-cockpit-night = Cockpit night lighting
    .facts =
        A dim red flood light in the cockpit after dark (blue-green under the goggles)
        The master caution pulses amber over the panel while a warning is on
effect-nvg = Night vision goggles
    .facts =
        Bright lights bloom into halos, the picture grains more in the dark
        Desktop: the goggles' round field of view
effect-snow = Snow
    .facts =
        Above the freezing level storms snow instead of rain: low in a northern winter, high in summer
effect-mist = Ground mist
    .facts =
        Modern shaders: mist lies over the low ground and the sea at dawn, dusk and night
        Thin in the deserts, none in storms
effect-cloud-shadows = Cloud shadows
    .facts =
        Modern shaders: the clouds' shadows on the ground below
effect-vapour = Пара и поток от витлото
    .facts =
        При остри маневри от краищата на крилата се откъсват следи от пара
        Ниско висящ хеликоптер вдига прах, над вода - пръски
        Движещите се машини вдигат прах след себе си
effect-night-sky = Night sky
    .facts =
        Shooting stars now and then, the stars low over the horizon twinkle
        The northern lights over the North Cape
effect-vga = VGA colours
    .facts =
        The 1989 look with modern shading: 64 steps a colour channel, as a VGA card's, dithered
effect-scanlines = Scanlines
    .facts =
        A CRT monitor's scanlines (not in VR)

## LANGUAGE NAMES

lang-name-en = английски
lang-name-bg = български
lang-name-zh-Hans = китайски (опростен)
lang-name-hr = хърватски
lang-name-cs = чешки
lang-name-da = датски
lang-name-nl = нидерландски
lang-name-et = естонски
lang-name-fi = фински
lang-name-fr = френски
lang-name-de = немски
lang-name-el = гръцки
lang-name-hu = унгарски
lang-name-id = индонезийски
lang-name-ga = ирландски
lang-name-it = италиански
lang-name-ja = японски
lang-name-ko = корейски
lang-name-lv = латвийски
lang-name-lt = литовски
lang-name-mt = малтийски
lang-name-pl = полски
lang-name-pt-BR = португалски (Бразилия)
lang-name-ro = румънски
lang-name-ru = руски
lang-name-sk = словашки
lang-name-sl = словенски
lang-name-es = испански
lang-name-sv = шведски
lang-name-uk = украински

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Език на браузъра: { $language }. Искате ли да играете на този език?
language-offer-yes = Да
language-offer-no = Не
language-offer-never = Не питай отново
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
