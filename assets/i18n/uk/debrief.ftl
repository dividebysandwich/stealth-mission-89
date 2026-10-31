# Дебрифінг місії (src/debrief.rs) та звіт симуляції (stealth-sim score.rs).

## Як завершився політ

outcome-landed = Успішна посадка
outcome-bad-landing = Невдала посадка
outcome-ejected = Катапультування - приземлення на парашуті
outcome-rescued = Катапультування - врятовано групою CSAR
outcome-captured = Катапультування - потрапили в полон
outcome-crashed = Катастрофа
outcome-ejection-failed = Катапультування невдале
outcome-aborted = Місію перервано
outcome-collision = Зіткнення

## Оцінки

rating-unsatisfactory = Незадовільно
rating-marginal = Погранично
rating-satisfactory = Задовільно
rating-excellent = Добре
rating-outstanding = Відмінно

## Бали

points-primary = Основне завдання
points-secondary = Додаткове завдання
points-aircraft = Знищено літаків
points-unseen = Збито непоміченим
points-ground = Знищено наземних цілей
points-drones = Збито дронів
points-drones-through = Дронів прорвалось
points-nuclear = Влучання в АЕС
points-detected = Виявлено
points-missile-hits = Влучання ракет

## Хронологія (назви місць і літаків як у місії)

timeline-primary = {" "}(основне)
timeline-secondary = {" "}(додаткове)
timeline-destroyed = Знищено: { $place }{ $tag }
timeline-destroyed-ground = Знищено наземну ціль{ $tag }
timeline-shot-down = Збито: { $aircraft }{ $tag }
timeline-hit = Влучання ракети
timeline-seen = Помічено: { $aircraft }
timeline-radar-id = Radar ID: { $place }
timeline-objective = Завдання досягнуто{ $tag }
timeline-got-through = { $aircraft } прорвався
timeline-photo = Фото зроблено{ $tag }

## Вікно дебрифінгу

debrief-title = ДЕБРИФІНГ МІСІЇ
debrief-training = НАВЧАЛЬНА МІСІЯ
debrief-results = РЕЗУЛЬТАТИ
debrief-flight-map = КАРТА ПОЛЬОТУ
debrief-section-results = Результати
debrief-accomplished = ВИКОНАНО
debrief-failed = НЕ ВИКОНАНО
debrief-primary = ОСНОВНЕ
debrief-secondary = ДОДАТКОВЕ
debrief-aircraft = ЛІТАКИ
debrief-drones = ДРОНИ
debrief-drones-value = збито { $down }, прорвалось { $through }
debrief-ground = НАЗЕМНІ
debrief-weapons = ЗБРОЯ
debrief-radar-ids = RADAR ID
debrief-visual-ids = ВІЗУАЛЬНІ ID
debrief-hits-taken = ВЛУЧАННЯ В ВАС
debrief-time = ЧАС
debrief-section-points = Бали
debrief-difficulty = СКЛАДНІСТЬ
debrief-total = УСЬОГО
debrief-section-timeline = Хронологія
debrief-no-events = Подій немає
debrief-section-exposure = Радіолокаційне опромінення
debrief-tracked = СУПРОВІД
debrief-closest = НАЙБЛИЖЧЕ
debrief-worst = НАЙГІРШЕ
debrief-at = О { $time }
debrief-altitude = ВИСОТА
debrief-speed = ШВИДКІСТЬ
debrief-margin = ЗАПАС
debrief-strongest = НАЙСИЛЬНІШЕ
debrief-tracked-by = СУПРОВІД: { $sites }
debrief-section-map = Карта
debrief-legend-safe = Не виявляється
debrief-legend-close = Близько до виявлення
debrief-legend-detectable = Виявляється
debrief-legend-tracked = Супровід / пуск ракети
debrief-legend-hit = Влучання в вас / знищення
map-symbols = Символи
map-legend-clouds = Хмари / грозові вежі
map-legend-snow = Сніг, від слабкого до сильного
map-legend-rain = Дощ, від слабкого до сильного
map-legend-thunderstorm = Грозова комірка
debrief-pause = ПАУЗА
debrief-play = ПУСК
