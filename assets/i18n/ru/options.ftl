# Окно НАСТРОЙКИ (src/options.rs) и его страницы в меню полёта.

options-title = НАСТРОЙКИ
options-tab-game = ИГРА
options-tab-graphics = ГРАФИКА
options-tab-sound = ЗВУК
options-tab-controls = УПРАВЛЕНИЕ
options-footer = Сохраняются для следующих полётов; параметры реализма выбираются в каждой игре

## ИГРА

options-section-language = Язык
options-section-classic-modern = Классический / Современный
options-cockpit = Кабина
    .facts =
        Современная: дисплеи высокого разрешения с кнопками по краю
        Классическая: приборная панель эпохи DOS и дисплеи 77x60 (F-117A)
options-hud = HUD
    .facts =
        Современный: зелёный HUD высокого разрешения в стиле F-16 с режимами NAV, AA и AG
        Классический: HUD эпохи DOS
options-avionics = Авионика
    .facts =
        Современная: форматы MFD в стиле F-16 на любом MFD (обстановка, HARM, прицельный контейнер, карта, подвеска) с подвижным курсором
        Классическая: карта и TAC слева, камера, WAY, WPN и ORD справа
options-flight-model = Модель полёта
    .facts =
        Современная: аэродинамическая модель полёта с ЭДСУ: ручка задаёт угловые скорости тангажа и крена; если её отпустить, нос остаётся на месте и траектория сохраняется
        Классическая: модель полёта эпохи DOS
options-section-flying = Пилотирование
options-auto-trim = Автотриммер
    .facts =
        При отпущенной ручке триммер удерживает положение, в котором вы его оставили
        Вертолёты: нос больше не задирается с ростом скорости; самолёты: компенсирует повреждённые рули
        Ручной триммер: Alt + стрелки, Alt + Home возвращает в нейтраль
options-tooltips = Подсказки в кабине
    .facts = Что это за орган управления или дисплей кабины, под мышью или пальцем в VR
options-section-vr-size = Размер кабины в VR
options-vr-size =
    .facts =
        Размер кабины (и мира) в шлеме
        100%: натуральная величина

## ГРАФИКА

options-section-rendering = Рендеринг
options-shaders = Шейдеры
    .facts =
        Современные: мягкое направленное затенение, плавная дымка на расстоянии
        Классические: плоское затенение
options-shadows = Тени
    .facts =
        Современные: тени от солнца в кабине и в мире (нагружает GPU)
        Классические: без теней
options-point-lights = Точечные источники света
    .facts =
        Современные: подсветка кабины и дисплеев; взрывы, факелы и огни ВПП освещают сцену
        Классические: только свет солнца и неба
options-time-of-day = Время суток
    .facts =
        Современное: солнце и луна движутся по часам миссии, небо темнеет и светлеет вместе с солнцем; звёзды и фаза луны ночью
        Современное: лунный свет определяет, с какого расстояния вражеские пилоты видят вас ночью (2 км в безлунную ночь, 4 км в полнолуние)
        Классическое: медленная смена цветов и неподвижное солнце
options-msaa = Сглаживание (MSAA)
    .facts =
        Сглаженные края: MSAA 4x, 2x в VR (нагружает GPU)
        Выкл.: резкие края пикселей
options-auto-detail = Автоматическая детализация местности (VR)
    .facts =
        В шлеме дальность прорисовки городов, лесов и полей уменьшается, когда кадры запаздывают, и возвращается, когда есть запас времени
        Настройки графики остаются выбранными
options-sun-moon = Солнце и луна
    .facts =
        Солнце в небе днём, луна и звёзды ночью
        Луна показывает фазу миссии

## ЗВУК

options-section-volume = Громкость
options-volume-master = Общая
    .facts = Все звуки
options-volume-engine = Двигатель
    .facts = Ваш двигатель или несущий винт и летательные аппараты, слышимые вокруг
options-volume-effects = Эффекты
    .facts = Оружие, взрывы, шасси, радио и прочее
options-volume-warnings = Сигнализация
    .facts = Сигнал сваливания, предупреждения о радаре и ракетах, пожарная сирена, тоны Sidewinder
options-section-cockpit = Кабина
options-low-pass = Фильтр низких частот двигателя
    .facts =
        В кабине двигатель звучит приглушённо (фонарь и шлем срезают высокие частоты)
        Выкл.: так же, как снаружи

## УПРАВЛЕНИЕ

options-section-bindings = Назначения
options-keys-joystick = КЛАВИШИ И ДЖОЙСТИК
    .facts =
        Назначение клавиш, осей и кнопок джойстика и HOTAS
        Сохраняются для следующих полётов
options-section-head-tracking = Трекинг головы и глаз
options-head-tracking = Трекинг головы и глаз
    .facts =
        Повороты и движения головы управляют обзором в кабине (на мониторе; в VR это делает шлем)
        Программа трекинга (OpenTrack...): выход FreeTrack или UDP на 127.0.0.1 порт 4242
        Home: текущее положение головы становится центром
options-tracking = Трекинг ({ $source })
options-no-tracker = Трекер не обнаружен: запустите программу трекинга

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
effect-vapour = Vapour and downwash
    .facts =
        Pulling hard, vapour trails from the wingtips
        A helicopter hovering low whips up dust, or spray over water
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

lang-name-en = английский
lang-name-bg = болгарский
lang-name-zh-Hans = китайский (упрощённый)
lang-name-hr = хорватский
lang-name-cs = чешский
lang-name-da = датский
lang-name-nl = нидерландский
lang-name-et = эстонский
lang-name-fi = финский
lang-name-fr = французский
lang-name-de = немецкий
lang-name-el = греческий
lang-name-hu = венгерский
lang-name-id = индонезийский
lang-name-ga = ирландский
lang-name-it = итальянский
lang-name-ja = японский
lang-name-ko = корейский
lang-name-lv = латышский
lang-name-lt = литовский
lang-name-mt = мальтийский
lang-name-pl = польский
lang-name-pt-BR = португальский (Бразилия)
lang-name-ro = румынский
lang-name-ru = русский
lang-name-sk = словацкий
lang-name-sl = словенский
lang-name-es = испанский
lang-name-sv = шведский
lang-name-uk = украинский

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Язык браузера: { $language }. Играть на этом языке?
language-offer-yes = Да
language-offer-no = Нет
language-offer-never = Больше не спрашивать
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
