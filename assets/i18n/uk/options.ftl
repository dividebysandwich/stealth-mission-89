# Вікно ПАРАМЕТРИ (src/options.rs) та його сторінки в меню польоту.

options-title = ПАРАМЕТРИ
options-tab-game = ГРА
options-tab-graphics = ГРАФІКА
options-tab-sound = ЗВУК
options-tab-controls = КЕРУВАННЯ
options-footer = Зберігається для наступних польотів; перемикачі реалізму обираються для кожної гри

## ГРА

options-section-language = Мова
options-section-classic-modern = Класичний / Сучасний
options-cockpit = Кабіна
    .facts =
        Сучасна: дисплеї високої роздільності з кнопками по краях
        Класична: розташування приладів епохи DOS і дисплеї 77x60 (F-117A)
options-hud = HUD
    .facts =
        Сучасний: зелений HUD високої роздільності в стилі F-16 з режимами NAV, AA і AG
        Класичний: HUD епохи DOS
options-avionics = Авіоніка
    .facts =
        Сучасна: формати MFD в стилі F-16 на будь-якому з MFD (ситуація, HARM, прицільний контейнер, карта, підвіски) із зсувним курсором
        Класична: карта і TAC ліворуч, камера, WAY, WPN і ORD праворуч
options-flight-model = Модель польоту
    .facts =
        Сучасна: аеродинамічна модель польоту з електродистанційним керуванням: ручка задає швидкість тангажа й крену; відпущена, ніс лишається на місці, а траєкторія утримується
        Класична: модель польоту епохи DOS
options-section-flying = Пілотування
options-auto-trim = Автотрим
    .facts =
        З відпущеною ручкою трим утримує положення, у якому ви її залишили
        Гелікоптери: ніс більше не задирається зі швидкістю; літаки: балансує пошкоджені керівні поверхні
        Ручний трим: Alt + стрілки, Alt + Home центрує
options-tooltips = Підказки кабіни
    .facts = Що це за елемент або дисплей кабіни, під мишею чи кінчиком пальця у VR
options-section-vr-size = Розмір кабіни у VR
options-vr-size =
    .facts =
        Розмір кабіни (і світу) в шоломі
        100%: натуральна величина

## ГРАФІКА

options-section-rendering = Візуалізація
options-shaders = Шейдери
    .facts =
        Сучасні: ледь помітне спрямоване затінення, плавна димка вдалині
        Класичні: пласке затінення
options-shadows = Тіні
    .facts =
        Сучасні: тіні від сонця в кабіні та у світі (навантажує GPU)
        Класичні: без тіней
options-point-lights = Точкове освітлення
    .facts =
        Сучасне: заливне світло кабіни та підсвітка дисплеїв; вибухи, шлейфи та вогні смуги освітлюють сцену
        Класичне: лише світло сонця й неба
options-time-of-day = Пора доби
    .facts =
        Сучасна: сонце й місяць рухаються за годинником місії, небо темніє й світлішає разом із сонцем; зорі та фаза місяця вночі
        Сучасна: місячне світло визначає, з якої відстані вороги бачать вас уночі (2 км безмісячної ночі, 4 км у повню)
        Класична: повільна зміна кольорів і нерухоме сонце
options-msaa = Згладжування (MSAA)
    .facts =
        Гладкі краї: 4x MSAA, 2x у VR (навантажує GPU)
        Вимкнено: різкі пікселі на краях
options-auto-detail = Автодеталізація місцевості (VR)
    .facts =
        У шоломі дальність видимості міст, лісів і полів скорочується, коли кадри запізнюються, і зростає, коли є запас часу
        Налаштування графіки лишаються як обрано
options-sun-moon = Сонце і місяць
    .facts =
        Сонце на небі вдень, місяць і зорі вночі
        Місяць показує фазу, задану місією

## ЗВУК

options-section-volume = Гучність
options-volume-master = Загальна
    .facts = Усі звуки
options-volume-engine = Двигун
    .facts = Ваш двигун чи гвинт і літаки, що чутно довкола
options-volume-effects = Ефекти
    .facts = Зброя, вибухи, шасі, радіо та решта
options-volume-warnings = Сигнали
    .facts = Сигнал зриву, попередження радара й ракет, пожежна сирена, тони Sidewinder
options-section-cockpit = Кабіна
options-low-pass = Низькочастотний фільтр двигуна
    .facts =
        Усередині літака двигун звучить приглушено (ліхтар і шолом зрізають верхні частоти)
        Вимкнено: так само, як зовні

## КЕРУВАННЯ

options-section-bindings = Призначення
options-keys-joystick = КЛАВІШІ ТА ДЖОЙСТИК
    .facts =
        Клавіші карти, осі та кнопки джойстика й HOTAS
        Зберігається для наступних польотів
options-section-head-tracking = Трекінг голови/очей
options-head-tracking = Трекінг голови/очей
    .facts =
        Ваша голова повертає й зсуває вид у кабіні (на ПК; у VR це робить шолом)
        Програма трекінгу (OpenTrack...): вивід FreeTrack або UDP на 127.0.0.1, порт 4242
        Home: поточне положення голови стає центром
options-tracking = Трекінг ({ $source })
options-no-tracker = Трекер не виявлено: запустіть програму трекінгу

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

lang-name-en = англійська
lang-name-bg = болгарська
lang-name-zh-Hans = китайська (спрощена)
lang-name-hr = хорватська
lang-name-cs = чеська
lang-name-da = данська
lang-name-nl = нідерландська
lang-name-et = естонська
lang-name-fi = фінська
lang-name-fr = французька
lang-name-de = німецька
lang-name-el = грецька
lang-name-hu = угорська
lang-name-id = індонезійська
lang-name-ga = ірландська
lang-name-it = італійська
lang-name-ja = японська
lang-name-ko = корейська
lang-name-lv = латвійська
lang-name-lt = литовська
lang-name-mt = мальтійська
lang-name-pl = польська
lang-name-pt-BR = португальська (Бразилія)
lang-name-ro = румунська
lang-name-ru = російська
lang-name-sk = словацька
lang-name-sl = словенська
lang-name-es = іспанська
lang-name-sv = шведська
lang-name-uk = українська

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Мова браузера: { $language }. Грати цією мовою?
language-offer-yes = Так
language-offer-no = Ні
language-offer-never = Більше не питати
