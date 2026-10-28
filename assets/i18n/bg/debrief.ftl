# Дебрифингът на мисията (src/debrief.rs) и докладът на симулацията (stealth-sim score.rs).

## Как е завършил полетът

outcome-landed = Безопасно кацане
outcome-bad-landing = Лошо кацане
outcome-ejected = Катапултирал - кацнал с парашут
outcome-rescued = Катапултирал - спасен от CSAR
outcome-captured = Катапултирал - заловен
outcome-crashed = Катастрофирал
outcome-ejection-failed = Катапултирането е неуспешно
outcome-aborted = Мисията е прекратена
outcome-collision = Сблъсък

## Оценки

rating-unsatisfactory = Незадоволително
rating-marginal = Гранично
rating-satisfactory = Задоволително
rating-excellent = Отлично
rating-outstanding = Изключително

## Точки

points-primary = Основна цел
points-secondary = Второстепенна цел
points-aircraft = Унищожени самолети
points-unseen = Свалени незабелязано
points-ground = Унищожени наземни цели
points-drones = Свалени дронове
points-drones-through = Дронове, стигнали целта
points-nuclear = Поразена ядрена централа
points-detected = Засечен
points-missile-hits = Попадения на ракети

## Хронология (имената на места и самолети са както в мисията)

timeline-primary = {" "}(основна)
timeline-secondary = {" "}(второстепенна)
timeline-destroyed = Унищожено: { $place }{ $tag }
timeline-destroyed-ground = Унищожена наземна цел{ $tag }
timeline-shot-down = Свален: { $aircraft }{ $tag }
timeline-hit = Поразен от ракета
timeline-seen = Забелязан от { $aircraft }
timeline-radar-id = Радарна идентификация от { $place }
timeline-objective = Целта е достигната{ $tag }
timeline-got-through = { $aircraft } премина
timeline-photo = Снимката е направена{ $tag }

## Прозорецът на дебрифинга

debrief-title = ДЕБРИФИНГ НА МИСИЯТА
debrief-training = ТРЕНИРОВЪЧНА МИСИЯ
debrief-results = РЕЗУЛТАТИ
debrief-flight-map = КАРТА НА ПОЛЕТА
debrief-section-results = Резултати
debrief-accomplished = ИЗПЪЛНЕНА
debrief-failed = ПРОВАЛЕНА
debrief-primary = ОСНОВНА
debrief-secondary = ВТОРОСТЕПЕННА
debrief-aircraft = САМОЛЕТИ
debrief-drones = ДРОНОВЕ
debrief-drones-value = { $down } свалени, { $through } стигнали
debrief-ground = ЗЕМЯ
debrief-weapons = ОРЪЖИЕ
debrief-radar-ids = РАДАРНИ ИДЕНТ.
debrief-visual-ids = ВИЗУАЛНИ ИДЕНТ.
debrief-hits-taken = ПОЛУЧЕНИ ПОПАДЕНИЯ
debrief-time = ВРЕМЕ
debrief-section-points = Точки
debrief-difficulty = ТРУДНОСТ
debrief-total = ОБЩО
debrief-section-timeline = Хронология
debrief-no-events = Няма събития
debrief-section-exposure = Радарна експозиция
debrief-tracked = СЛЕДЕН
debrief-closest = НАЙ-БЛИЗО
debrief-worst = НАЙ-ЛОШО
debrief-at = В { $time }
debrief-altitude = ВИСОЧИНА
debrief-speed = СКОРОСТ
debrief-margin = ЗАПАС
debrief-strongest = НАЙ-СИЛЕН
debrief-tracked-by = СЛЕДЕН ОТ { $sites }
debrief-section-map = Карта
debrief-legend-safe = Не може да бъде засечен
debrief-legend-close = Близо до засичане
debrief-legend-detectable = Може да бъде засечен
debrief-legend-tracked = Следен / пуск на ракета
debrief-legend-hit = Получено попадение / поразена цел
debrief-pause = ПАУЗА
debrief-play = ПУСНИ
