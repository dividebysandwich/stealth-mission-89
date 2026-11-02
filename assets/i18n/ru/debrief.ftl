# Разбор полёта (src/debrief.rs) и отчёт симуляции.

## Чем закончился полёт

outcome-landed = Успешная посадка
outcome-bad-landing = Неудачная посадка
outcome-ejected = Катапультирование - приземление на парашюте
outcome-rescued = Катапультирование - спасён CSAR
outcome-captured = Катапультирование - в плену
outcome-crashed = Крушение
outcome-ejection-failed = Неудачное катапультирование
outcome-aborted = Миссия прервана
outcome-collision = Столкновение

## Оценки

rating-unsatisfactory = Неудовлетворительно
rating-marginal = Посредственно
rating-satisfactory = Удовлетворительно
rating-excellent = Отлично
rating-outstanding = Превосходно

## Очки

points-primary = Основная цель
points-secondary = Запасная цель
points-aircraft = Уничтожено самолётов
points-unseen = Сбито незаметно
points-ground = Уничтожено наземных целей
points-drones = Сбито дронов
points-drones-through = Прорвалось дронов
points-nuclear = Поражение атомной станции
points-detected = Обнаружение
points-missile-hits = Попадания ракет

## Хронология (названия мест и самолётов как в миссии)

timeline-primary = {" "}(основная)
timeline-secondary = {" "}(запасная)
timeline-destroyed = Уничтожено: { $place }{ $tag }
timeline-destroyed-ground = Уничтожена наземная цель{ $tag }
timeline-shot-down = Сбит { $aircraft }{ $tag }
timeline-hit = Попадание ракеты
timeline-seen = Обнаружен самолётом { $aircraft }
timeline-radar-id = Опознан РЛС: { $place }
timeline-objective = Цель достигнута{ $tag }
timeline-got-through = { $aircraft } прорвался
timeline-photo = Снимок сделан{ $tag }

## Окно разбора

debrief-title = РАЗБОР ПОЛЁТА
debrief-training = УЧЕБНАЯ МИССИЯ
debrief-results = ИТОГИ
debrief-flight-map = КАРТА ПОЛЁТА
debrief-section-results = Итоги
debrief-accomplished = ВЫПОЛНЕНО
debrief-failed = НЕ ВЫПОЛНЕНО
debrief-primary = ОСНОВНАЯ
debrief-secondary = ЗАПАСНАЯ
debrief-aircraft = САМОЛЁТЫ
debrief-drones = ДРОНЫ
debrief-drones-value = сбито { $down }, прорвалось { $through }
debrief-ground = НАЗЕМНЫЕ
debrief-weapons = ОРУЖИЕ
debrief-radar-ids = ОПОЗНАНИЯ РЛС
debrief-visual-ids = ВИЗУАЛЬНЫЕ ОПОЗНАНИЯ
debrief-hits-taken = ПОЛУЧЕНО ПОПАДАНИЙ
debrief-time = ВРЕМЯ
debrief-section-points = Очки
debrief-difficulty = СЛОЖНОСТЬ
debrief-total = ИТОГО
debrief-section-timeline = Хронология
debrief-no-events = Событий нет
debrief-section-exposure = Радиолокационная заметность
debrief-tracked = СОПРОВОЖДЕНИЕ
debrief-closest = БЛИЖЕ ВСЕГО
debrief-worst = ХУДШИЙ МОМЕНТ
debrief-at = В { $time }
debrief-altitude = ВЫСОТА
debrief-speed = СКОРОСТЬ
debrief-margin = ЗАПАС
debrief-strongest = СИЛЬНЕЙШИЙ
debrief-tracked-by = СОПРОВОЖДАЛИ: { $sites }
debrief-section-map = Карта
debrief-legend-safe = Не обнаруживается
debrief-legend-close = Близко к обнаружению
debrief-legend-detectable = Обнаруживается
debrief-legend-tracked = Сопровождение / пуск ракеты
debrief-legend-hit = Попадание / победа
map-symbols = Символы
map-legend-clouds = Облака / грозовые башни
map-legend-snow = Снег, от слабого до сильного
map-legend-rain = Дождь, от слабого до сильного
map-legend-thunderstorm = Грозовая ячейка
debrief-pause = ПАУЗА
debrief-play = ВОСПРОИЗВЕСТИ
