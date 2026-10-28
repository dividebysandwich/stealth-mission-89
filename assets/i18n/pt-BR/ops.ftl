# A tela de operações da campanha (src/campaign/ops.rs).

ops-thunderstorms = TROVOADAS
ops-wx-short = TEMPO { $now }, { $next } ÀS { $at }
ops-wx-line = { $weather } Mudando para { $next } às { $at }.
ops-clock = DIA { $day }  { $time }
ops-landed-at = POUSADO EM { $base }
ops-ground-crew = A equipe de solo está pronta
ops-rearm = REARMAR E REABASTECER
ops-end-sortie = ENCERRAR SURTIDA
ops-on-server = EM { $server }
ops-serving = SERVIDOR
ops-tab-missions = MISSÕES
ops-tab-map = MAPA DE GUERRA
ops-tab-air-defence = DEFESA AÉREA
ops-tab-forces = FORÇAS
ops-tab-players = JOGADORES
ops-tab-debrief = DEBRIEFING
ops-tab-log = REGISTRO
ops-tab-chat = CHAT
ops-blue = AZUL { $count }
ops-red = VERMELHO { $count }
ops-war-over = A guerra terminou
ops-running = Simulação da guerra em andamento
ops-save-exit = SALVAR E SAIR
ops-disconnect = DESCONECTAR
ops-leave-local = SALVAR E SAIR DA CAMPANHA?
    .text = A guerra para até você continuá-la. As aeronaves no ar são perdidas; as infiltrações em andamento recomeçam da sua base.
ops-leave-host = SALVAR E PARAR DE HOSPEDAR A CAMPANHA?
    .text = Os jogadores conectados são desconectados e a guerra para até você continuá-la. As aeronaves no ar são perdidas; as infiltrações em andamento recomeçam da sua base.
ops-leave-client = DESCONECTAR DO SERVIDOR?
    .text = A campanha continua no servidor; nada é salvo aqui. Sua surtida termina.
ops-unit-destroyed = { $unit } destruído: de volta às operações

## Missões

ops-section-missions = Missões
ops-threat = AMEAÇA { $threat }
ops-no-missions = Nenhuma missão: a frente está calma
ops-section-underway = Infiltrações e ataques
ops-enemy-liftoff = INIMIGO em { $place }: decolagem em { $mins } min
ops-liftoff = { $place }: decolagem em { $mins } min
ops-enemy-helis = INIMIGO em { $place }: { $count } helicópteros se aproximando
ops-helis = { $place }: { $count } helicópteros a caminho
ops-raid-sead = SEAD da IA em { $place }
ops-raid-drones = DRONES sobre { $place }
ops-raid = ATAQUE ao aeródromo de { $place }
ops-raid-launch = { $raid }: lançamento em { $mins } min
ops-raid-on-way = { $raid }: { $count } a caminho
ops-resupply-ai = RESSUPRIMENTO ({ $unit }): UH-60 a caminho
ops-awacs = AWACS INIMIGO no ar por mais { $mins } min
ops-down = ABATIDO: { $pilot } - resgate em até { $mins } min
ops-aboard = A BORDO: { $pilot } - a caminho de casa
ops-none-underway = Nada em andamento
ops-ai-helicopter = Um helicóptero da IA assume em { $time }, a menos que um piloto de UH-60 o faça
ops-defences = Defesas: { $sams } sítios SAM, ameaça { $threat }
ops-recon-intel = INTELIGÊNCIA DE RECON
ops-order-insertion = ORDENAR INFILTRAÇÃO
    .facts = Voe a escolta para cobri-los ou lidere-os no UH-60
ops-from = De
ops-planned-by-group = Planejado pelo grupo
ops-stores = { $aircraft } - cargas: { $stores }
ops-fly-uh60 = VOAR O UH-60
ops-accept = ACEITAR MISSÃO
ops-planning = PLANEJANDO: { $pilots }
ops-flying = EM VOO: { $pilots }
ops-package = PACKAGE com { $kinds }: cumpra dois tipos aqui em { $mins } min, +{ $bonus } pontos cada
ops-sead-on-way = A SEAD da IA está a caminho
ops-request-sead = SOLICITAR SEAD DA IA ({ $cost } de material, { $aircraft } aeronaves)
    .facts = Aviões de ataque amigos atacam um sítio SAM deste lugar antes de você
    .disabled = Material ou aeronaves insuficientes no solo (ver FORÇAS)

## Regras, material, forças

ops-rule-damage = danos realistas
ops-rule-guidance = guiagem realista
ops-rule-no-crashes = sem acidentes
ops-rule-time = tempo contínuo
ops-rule-weapons = armamento realista do F-117A
ops-rule-friendly-fire = fogo amigo
ops-rule-classic = clássicas
ops-materiel = MATERIAL
ops-materiel-blue = AZUL { $materiel } (+{ $income }/h)
ops-materiel-red = VERMELHO { $materiel } (+{ $income }/h)
ops-friendly = AMIGOS
ops-enemy = INIMIGOS
ops-section-forces = Forças
ops-places-held = Lugares controlados
ops-materiel-row = Material
ops-income = Renda por hora
ops-income-from = sendo
ops-income-sources = { $base } + { $places } lugares x { $per_place } + { $cities } cidades x { $per_city }
ops-fighters-ground = Caças no solo
ops-helicopters-ground = Helicópteros no solo
ops-in-air = No ar
ops-in-air-value = { $fighters } caças, { $strike } de ataque, { $helicopters } helicópteros
ops-since-start = Desde o início
ops-earned = ganho
ops-repairs = reparos
ops-operations = surtidas, ataques, infiltrações
ops-new-aircraft = novas aeronaves
ops-lost = perdas (cidades, pilotos)
ops-section-how = Como funciona
ops-how =
    Cada lado ganha material a cada hora: uma parte fixa, mais para cada lugar que controla e para cada cidade intacta entre eles. Bombardeie as cidades inimigas (INTERDIÇÃO) para cortar a renda do inimigo.
    A IA paga por tudo o que faz: reparos, cada surtida de caça, ataques e suas próprias infiltrações. Sem material, os danos dela ficam sem reparo e as aeronaves dela ficam em terra.
    A IA voa apenas as aeronaves que tem; abatidas, estão perdidas; pousadas, voam de novo. A cada hora, cada lado compra novas enquanto o material durar.
    Você nunca fica sem nada: suas aeronaves, suas bases (bombardeadas ou não) e os helicópteros das suas próprias ordens de infiltração estão sempre lá.
ops-section-prices = Preços
ops-price-airfield = Reparo de aeródromo
ops-price-sam = Reparo de sítio SAM
ops-price-city = Cidade reconstruída
ops-price-ship = Reparo de navio
ops-price-gun = Canhão ou equipe MANPADS
ops-price-sortie = Surtida de caça
ops-price-raid = Ataque
ops-price-insertion = Infiltração da IA
ops-price-fighter = Novo caça
ops-price-helicopter = Novo helicóptero
ops-price-city-lost = Cidade destruída (perda)
ops-price-captured = Piloto capturado (perda)

## Mapa de guerra

ops-click-to-watch = { $unit } - clique para acompanhar
ops-section-legend = Legenda
ops-legend-friendly-airfield = Aeródromo amigo
ops-legend-enemy-airfield = Aeródromo inimigo
ops-legend-city = Cidade amiga / inimiga
ops-watch-hint = Clique em uma unidade para acompanhá-la ao vivo
ops-map-hint = Roda: zoom, arrastar: mover
ops-patriot = Bateria Patriot
ops-manned-by = Operada por { $pilot }
ops-manned-by-other = Operada por outro jogador
ops-control = ASSUMIR O CONTROLE
    .facts = Assumir o console dela

## Debriefing, fim, registro, chat

ops-no-sortie = Nenhuma surtida voada ainda
ops-section-sortie = Surtida
ops-achieved = OBJETIVO CUMPRIDO
ops-not-achieved = OBJETIVO NÃO CUMPRIDO
ops-waiting-raid = AGUARDANDO O ATAQUE
ops-waiting-insertion = AGUARDANDO A INFILTRAÇÃO
ops-flight-time = Tempo de voo { $mins } min
ops-kills = Abates: { $air } aeronaves, { $ground } no solo
ops-points = { $points } pontos
ops-points-bonus = { $points } pontos (bônus de package +{ $bonus })
ops-campaign-points = { $rank } - { $points } pontos de campanha
ops-campaign-record = Campanha: { $sorties } surtidas, { $won } objetivos cumpridos
ops-section-meanwhile = Enquanto isso
ops-victory = VITÓRIA
    .text = O inimigo não controla mais nenhum aeródromo: o teatro é nosso.
ops-defeat = DERROTA
    .text = Não resta nenhum aeródromo amigo: o teatro está perdido.
ops-end-day = Dia { $day }. Lugares amigos { $friendly }, lugares inimigos { $enemy }.
ops-next-campaign = A PRÓXIMA CAMPANHA COMEÇA EM { $time }
ops-next-campaign-now = A PRÓXIMA CAMPANHA ESTÁ COMEÇANDO
ops-next-from-menu = Inicie a próxima campanha pelo menu principal.
ops-section-log = Registro da campanha
ops-section-chat = Chat
ops-no-messages = Nenhuma mensagem ainda
ops-message = Mensagem
    .hint = Digite uma mensagem, Enter envia
ops-send = ENVIAR

## Briefing

ops-briefing = BRIEFING DA MISSÃO
ops-briefing-line = De { $base } - { $km } km até o alvo - { $aircraft } - cargas: { $stores }
ops-route-locked = Rota bloqueada: { $pilots } está voando
ops-route-threats = ROTA E AMEAÇAS
ops-take-off = DECOLAR
ops-radars-shown = Radares exibidos: detectados por aeronaves amigas, ainda de pé

## Jogadores

ops-section-players = Jogadores
ops-col-name = NOME
ops-col-status = STATUS
ops-col-sorties = SURTIDAS
ops-col-points = PONTOS
ops-col-pilot = PILOTO
ops-col-objectives = OBJETIVOS
ops-col-kills = ABATES AR/SOLO
ops-col-medal = MELHOR MEDALHA
ops-you = (você)
ops-host = ANFITRIÃO
ops-status-planning = Planejando: { $title }
ops-status-board = No quadro de missões
ops-status-flying = Em voo: { $title } - { $aircraft }
ops-status-reconnecting = Conexão perdida - aeronave mantida por { $seconds } s
ops-flown = { $flown } ({ $won } cumpridas)
ops-section-leaderboard = Classificação
ops-alone = Voando sozinho. Para voar junto, hospede esta campanha em MULTIJOGADOR no menu principal.
ops-rules = Regras desta campanha (fixadas no início): { $rules }
ops-no-sorties = Nenhuma surtida voada ainda
