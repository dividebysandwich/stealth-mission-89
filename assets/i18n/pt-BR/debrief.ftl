# O debriefing da missão (src/debrief.rs) e o relatório da simulação (stealth-sim score.rs).

## Como o voo terminou

outcome-landed = Pouso seguro
outcome-bad-landing = Pouso malsucedido
outcome-ejected = Ejeção - pouso de paraquedas
outcome-rescued = Ejeção - resgatado pelo CSAR
outcome-captured = Ejeção - capturado
outcome-crashed = Acidente
outcome-ejection-failed = Ejeção malsucedida
outcome-aborted = Missão abortada
outcome-collision = Colisão

## Avaliações

rating-unsatisfactory = Insatisfatório
rating-marginal = Sofrível
rating-satisfactory = Satisfatório
rating-excellent = Excelente
rating-outstanding = Excepcional

## Pontos

points-primary = Objetivo principal
points-secondary = Objetivo secundário
points-aircraft = Aeronaves destruídas
points-unseen = Abates sem ser visto
points-ground = Alvos terrestres destruídos
points-drones = Drones abatidos
points-drones-through = Drones que passaram
points-nuclear = Usina nuclear atingida
points-detected = Detectado
points-missile-hits = Impactos de mísseis

## Cronologia (nomes de lugares e de aeronaves como na missão)

timeline-primary = {" "}(principal)
timeline-secondary = {" "}(secundário)
timeline-destroyed = { $place } destruído{ $tag }
timeline-destroyed-ground = Alvo terrestre destruído{ $tag }
timeline-shot-down = { $aircraft } abatido{ $tag }
timeline-hit = Atingido por um míssil
timeline-seen = Avistado por um { $aircraft }
timeline-radar-id = ID radar por { $place }
timeline-objective = Objetivo alcançado{ $tag }
timeline-got-through = { $aircraft } passou
timeline-photo = Foto tirada{ $tag }

## A janela de debriefing

debrief-title = DEBRIEFING DA MISSÃO
debrief-training = MISSÃO DE TREINAMENTO
debrief-results = RESULTADOS
debrief-flight-map = MAPA DO VOO
debrief-section-results = Resultados
debrief-accomplished = CUMPRIDO
debrief-failed = FRACASSADO
debrief-primary = PRINCIPAL
debrief-secondary = SECUNDÁRIO
debrief-aircraft = AERONAVES
debrief-drones = DRONES
debrief-drones-value = { $down } abatidos, { $through } passaram
debrief-ground = SOLO
debrief-weapons = ARMAS
debrief-radar-ids = IDS RADAR
debrief-visual-ids = IDS VISUAIS
debrief-hits-taken = IMPACTOS SOFRIDOS
debrief-time = TEMPO
debrief-section-points = Pontos
debrief-difficulty = DIFICULDADE
debrief-total = TOTAL
debrief-section-timeline = Cronologia
debrief-no-events = Nenhum evento
debrief-section-exposure = Exposição ao radar
debrief-tracked = RASTREADO
debrief-closest = MAIS PRÓXIMO
debrief-worst = PIOR
debrief-at = Às { $time }
debrief-altitude = ALTITUDE
debrief-speed = VELOCIDADE
debrief-margin = MARGEM
debrief-strongest = MAIS FORTE
debrief-tracked-by = RASTREADO POR { $sites }
debrief-section-map = Mapa
debrief-legend-safe = Não detectável
debrief-legend-close = Perto da detecção
debrief-legend-detectable = Detectável
debrief-legend-tracked = Rastreado / lançamento de míssil
debrief-legend-hit = Impacto sofrido / abate
map-symbols = Símbolos
map-legend-clouds = Nuvens / torres de tempestade
map-legend-snow = Neve, de fraca a forte
map-legend-rain = Chuva, de fraca a forte
map-legend-thunderstorm = Célula de tempestade
debrief-pause = PAUSAR
debrief-play = REPRODUZIR
