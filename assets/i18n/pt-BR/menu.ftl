# O menu principal e a tela de missão avulsa (src/menu.rs).

## Teatros de operações

theater-libya = Líbia (1986)
    .facts =
        Golfo de Sidra e costa líbia
        Época da Operação El Dorado Canyon
        Céu desértico limpo na maioria dos dias
theater-persian_gulf = Golfo Pérsico (1984)
    .facts =
        Época da Guerra dos Petroleiros Irã-Iraque
        Sobre o mar e o deserto costeiro
        O tempo mais limpo de todos os teatros
theater-north_cape = Cabo Norte
    .facts =
        Noruega ártica e Península de Kola
        Base da Frota do Norte soviética
        Nebulosidade baixa frequente
theater-central_europe = Europa Central
    .facts =
        OTAN contra Pacto de Varsóvia sobre a Alemanha
        Céu limpo só 15% do tempo
theater-vietnam = Vietnã
    .facts =
        Região de Hanói e Haiphong
        Selva e terreno montanhoso
        Trovoadas em 30% das missões
theater-middle_east = Oriente Médio
    .facts =
        Mediterrâneo Oriental e Levante
        Céu limpo na maior parte do tempo
theater-desert_storm = Tempestade no Deserto
    .facts =
        Kuwait e Iraque, 1991
        A verdadeira estreia em combate do F-117A
        Céu desértico limpo na maior parte do tempo
theater-cuba = Cuba
    .facts =
        Teatro insular do Caribe
        Alvos no mar e em terra
        Tropical: tempestades em 15% das missões
theater-korea = Coreia
    .facts =
        Península Coreana ao norte da DMZ
        Terreno montanhoso
        Tempo instável, qualquer céu é possível
theater-ukraine = Ucrânia (2026)
    .facts =
        Leste da Ucrânia, Crimeia e sul da Rússia
        A linha de frente arde de Kharkiv a Kherson
        Campos abertos, cidades em ruínas na frente

## Situações (regras de engajamento)

roe-0 = Guerra Fria
    .facts =
        Operação secreta em tempo de paz
        Não ser detectado é o que mais importa
        Menos aeronaves inimigas em patrulha
        As regras de engajamento exatas estão no briefing
roe-1 = Guerra limitada
    .facts =
        Conflito armado regional
        Defesas aéreas inimigas em alerta
        Mais aeronaves inimigas que na Guerra Fria
roe-2 = Guerra convencional
    .facts =
        Guerra total
        Todas as forças inimigas ativas
        Mais aeronaves inimigas que na Guerra Fria

## Tipos de missão

kind-strike = Ataque
    .facts =
        Destruir, fotografar ou alcançar um alvo em terra
        Objetivo primário e secundário
        Armamento ar-solo
kind-air = Ar-ar
    .facts =
        Interceptar uma aeronave inimiga
        Transportes, bombardeiros ou patrulhas de caças
        Armamento de mísseis ar-ar
kind-strike-training = Treino de ataque
    .facts =
        Missão de ataque de treinamento
        Aprender as armas e a navegação
kind-air-training = Treino ar-ar
    .facts =
        Missão ar-ar de treinamento
        Aprender interceptações e mísseis
kind-drone-defense = Defesa antidrone
    .facts =
        Só na Ucrânia
        Deter uma onda de drones de ataque Geran antes que atinja uma cidade ou uma usina nuclear
        Depois bombardear o local de lançamento

## Adversários

opponents-0 = Novato
    .facts =
        Tripulações inimigas inexperientes
        Nível mais fácil
opponents-1 = Regular
    .facts =
        Tripulações inimigas medianas
        Nível padrão
opponents-2 = Veterano
    .facts =
        Tripulações inimigas experientes
        Mais navios e patrulhas
opponents-3 = Elite
    .facts =
        Os melhores pilotos e operadores de SAM inimigos
        Nível mais difícil

## Meteorologia

sky-random = Aleatório
    .facts =
        Sorteado a partir do clima do teatro
        Muda a cada nova missão
        As nuvens bloqueiam a visão dos dois lados
sky-clear = Limpo
    .facts =
        Sem nuvens
        Visibilidade 40 km
        O céu clássico da era DOS
sky-scattered = Esparso
    .facts =
        Cúmulos 20-45%, base 2500-6000 ft
        Às vezes um altostrato fino bem acima
        Visibilidade 30 km
sky-broken = Nublado
    .facts =
        Cúmulos 45-70%, base 2000-5000 ft
        Muitas vezes uma camada de estratocúmulos acima
        Visibilidade 25 km
sky-overcast = Encoberto
    .facts =
        Estratos 80-100%, base 1500-5000 ft
        Alguns fragmentos de cúmulos abaixo
        Visibilidade 20 km
sky-storm = Tempestade
    .facts =
        Torres de cumulonimbus até 40000 ft
        Estratos e cúmulos baixos a partir de 1000 ft
        Visibilidade 8 km
menu-random-cover = Cobertura aleatória
    .facts =
        Cobertura da camada principal sorteada com a meteorologia
        Desmarque para ajustá-la você mesmo
        Camada principal: estratos com céu encoberto e tempestade, cúmulos nos demais casos
menu-cover-slider =
    .facts =
        Parte do céu coberta pela camada principal
        Sem efeito com céu limpo

## Lua

moon-date = Data da missão
    .facts =
        A lua real da data do briefing
        Requer a opção de hora do dia (moderno)
moon-0 = Lua nova
    .facts =
        Sem luar: noite escura, só luzes e estrelas
        Os pilotos inimigos veem você a 2 km
moon-1 = Lua crescente
    .facts = Uma lua fina no céu do anoitecer, se põe antes da meia-noite
moon-2 = Quarto crescente
    .facts = Metade iluminada, visível ao anoitecer, se põe por volta da meia-noite
moon-3 = Crescente gibosa
    .facts = Brilhante, visível da tarde até depois da meia-noite
moon-4 = Lua cheia
    .facts =
        Visível a noite toda, alta à meia-noite
        Os pilotos inimigos veem você a 4 km
moon-5 = Minguante gibosa
    .facts = Brilhante, nasce ao anoitecer, visível até a manhã
moon-6 = Quarto minguante
    .facts = Metade iluminada, nasce por volta da meia-noite
moon-7 = Lua minguante
    .facts = Uma lua fina que nasce antes do amanhecer

## Aeronaves

aircraft-f117 =
    .facts =
        Aeronave de ataque furtiva
        Subsônica, 7 g
        Cockpit moderno e clássico
aircraft-f16 =
    .facts =
        Caça multifunção
        Classe Mach 2, 9 g, rolamento rápido
        Cockpit de F-16 com RWR
aircraft-tornado =
    .facts =
        Aeronave de ataque em baixa altitude
        Asas de geometria variável conforme a velocidade
        Cockpit moderno, três MFDs
aircraft-a10 =
    .facts =
        Aeronave de apoio aéreo aproximado
        Lenta e robusta, canhão de 30 mm
        Cockpit do A-10C, dois MFDs
aircraft-av8b =
    .facts =
        Aeronave de ataque V/STOL
        Empuxo vetorado: voo pairado, pousa em qualquer lugar
        Alavanca dos bocais: R / F, Shift para os batentes
aircraft-uh60 =
    .facts =
        Helicóptero utilitário
        Infiltra tropas: pouse perto do alvo
        Cíclico e coletivo
aircraft-ah1z =
    .facts =
        Helicóptero de ataque
        Canhão em torreta, Hellfire, AIM-9
        O canhão e os Hellfire seguem o pod

## Seções e grupos

section-theater = Teatro
section-opponents = Adversários
section-situation = Situação
section-mission = Missão
section-moon = Lua
section-start-time = Hora de partida
section-weather = Meteorologia
section-realism = Realismo
section-tacview = Tacview
section-mission-summary = Resumo da missão
section-briefing = Briefing
group-mission = Missão
group-conditions = Condições
group-vehicle = Aeronave
group-simulation = Simulação

## Opções da missão

menu-start-random = Aleatória
    .facts =
        Sorteada a cada nova missão, como no modo clássico
        06:00-08:00 de dia ou 18:00-20:00 à noite
        Desmarque para escolher a hora
menu-start-slider =
    .facts =
        Hora da decolagem
        Noite das 18:00 às 06:00
        Mantida por NOVA MISSÃO
menu-realistic-weapons = Armas realistas
    .facts =
        Os limites da aeronave real
        2 compartimentos de armas em vez de 4
        Sem mísseis AIM-9 ou AIM-120
        Sem decolagens nem pousos em porta-aviões
menu-crash-protection = Proteção contra colisão
    .facts =
        Arremete automaticamente perto do solo (trem recolhido)
        Contato com o solo e colisões não são fatais
        Desligada: o solo sempre pode ser atingido
menu-tacview-export = Exportar
    .facts =
        Salva cada voo como gravação do Tacview (.acmi)
        Na pasta do jogo para seus arquivos, em tacview
menu-tacview-real-time = Telemetria em tempo real
    .facts =
        Envia o voo ao Tacview enquanto você voa
        No Tacview: Record > Real-Time Telemetry
        Endereço 127.0.0.1, porta 42674 (--tacview-address)
switch-game = JOGO
switch-real = REAL
menu-damage = Danos
    .facts =
        Danos das armas
        Jogo: danos clássicos; os impactos se somam, mas nunca derrubam a aeronave
        Realista: um míssil pode destruir a aeronave (ejete!), danifica mais sistemas e muitas vezes causa um incêndio, que se apaga ou explode a aeronave
        Realista: superfícies de controle danificadas tiram a aeronave do trim ou a fazem oscilar
menu-guidance = Guiagem
    .facts =
        Guiagem dos mísseis
        Clássico: os mísseis voam direto para o alvo
        Realista: navegação proporcional até o ponto de colisão; os motores se apagam e os mísseis seguem planando, perdendo velocidade a cada curva (fumaça só com o motor aceso)
        Realista: os SAM ganham altura contra alvos distantes; o AIM-120 voa reto por 3 s (cabre para lançá-lo em arco)
menu-air-defence = Defesa aérea
    .facts =
        Defesas aéreas a baixa altitude
        Clássico: só os sítios SAM e os caças
        Realista: canhões ZSU-23-4 guiados por radar (o radar deles vê você como o de um SAM) e baterias antiaéreas S-60 guardam os aeródromos e os alvos; equipes MANPADS nas cidades disparam SA-14 contra o que veem
        Voar baixo sobre locais defendidos fica perigoso; bombas e canhão destroem as peças antiaéreas

## Tela principal

menu-title = STEALTH MISSION 89
menu-subtitle = SISTEMA DE PLANEJAMENTO DE MISSÃO
menu-single-mission = MISSÃO AVULSA
    .facts =
        Uma missão no teatro de sua escolha
        Escolha as opções, planeje a rota e o armamento
menu-career = CARREIRA
    .facts =
        Fichas dos pilotos: posto, condecorações, surtidas
        Período de serviço: 12 surtidas em um teatro
        O que você destrói continua destruído
menu-campaign = CAMPANHA DINÂMICA
    .facts =
        Campanha dinâmica em tempo real
        Tome as cidades e os aeródromos do inimigo
        Missões tiradas da situação em curso
menu-multiplayer = MULTIJOGADOR
    .facts = Cooperativo na campanha dinâmica
    .join = Entrar em um servidor da lista pública
    .host = Entrar em um servidor, ou hospedar uma campanha e voar nela
menu-options = OPÇÕES
    .facts =
        Jogo: estilo do cockpit, do HUD e dos aviônicos, trim automático
        Gráficos, volumes de som, teclas e joystick
        Salvas para os próximos voos
    .short = Jogo, gráficos, som e controles
menu-download = BAIXAR
    .facts =
        O jogo para desktop Windows, Linux e macOS
        Com VR e multijogador
        Steam Deck e Steam Frame: a AppImage para Linux
web-play = JOGAR NO NAVEGADOR
    .facts = Aqui mesmo, sem instalar nada
web-download-platforms = Windows, Linux e macOS, Steam Deck e Steam Frame
web-download-version = A versão para baixar
web-gives-vr = VR
    .why = Voe em VR com qualquer headset compatível com OpenXR e o Steam Frame
web-gives-hotas = HOTAS
    .why = Suporte completo a gamepads, joysticks, manetes e pedais
web-gives-multiplayer = Multijogador melhor
    .why = Conexão UDP direta com os servidores, com menos lag
web-gives-server = Hospede seu servidor
    .why = Hospede uma campanha cooperativa ou rode um servidor dedicado
web-gives-head-tracking = Head tracking
    .why = Rastreadores no estilo TrackIR via OpenTrack
web-gives-tacview = Tacview
    .why = Gravações de voo e telemetria em tempo real
web-gives-keep = Seu para sempre
    .why = Se o site sair do ar, você continua com a sua própria cópia
menu-discord = DISCORD
    .facts =
        Voe com outros pilotos, relate bugs e compartilhe missões
        Acompanhe o desenvolvimento
menu-quit = SAIR
    .facts = Voltar à área de trabalho
menu-version = VERSÃO { $version }

## Tela de missão avulsa

menu-career-sortie = SURTIDA DE CARREIRA
menu-tour-sortie = { $pilot }  //  SURTIDA { $sortie } DE { $length } DO PERÍODO
menu-msn = SISTEMA DE PLANEJAMENTO DE MISSÃO  //  MSN { $seed }
menu-no-targets = Nenhum alvo de pé neste período de serviço
menu-plan-route = PLANEJAR ROTA
    .facts =
        O mapa do teatro com a cobertura dos radares e dos SAM inimigos
        Adicione, mova e remova pontos de navegação
        Os alvos e a base de pouso ficam fixos
menu-loadout = ARMAMENTO
    .facts =
        Escolha as cargas de cada compartimento ou pilone
        Começa com as armas do briefing
menu-generating = GERANDO...
menu-back-records = Voltar às fichas dos pilotos
menu-back-main = Voltar ao menu principal
menu-new-sortie = NOVA SURTIDA
menu-new-mission = NOVA MISSÃO
    .facts =
        Sorteia outros objetivos
        Mantém todas as opções e a hora de partida
menu-load = CARREGAR
    .facts =
        Missões salvas em voo (menu: SALVAR MISSÃO)
        Retome o voo de onde foi salvo
menu-fly = VOAR
    .facts = Decolar para a missão do briefing
menu-fly-vr = VOAR EM VR
    .facts =
        Reinicia o jogo no headset de VR
        Voa exatamente esta missão
    .no-mission = Gere uma missão primeiro
    .no-runtime = Nenhum runtime OpenXR (Monado, WiVRn, SteamVR) está ativo
menu-vr-failed = não foi possível iniciar em VR: { $error }

## Resumo da missão

summary-theater = TEATRO
summary-situation = SITUAÇÃO
summary-mission = MISSÃO
summary-opponents = ADVERSÁRIOS
summary-time = HORA
summary-weather = METEOROLOGIA
summary-fuel = COMBUSTÍVEL
summary-night = Noite
summary-day = Dia
summary-time-value = { $daytime }  part. { $depart }
summary-weather-value = { $sky }  vis. { $vis } km
summary-fuel-value = { $fuel } lb
summary-armament = Armamento: { $stores }
summary-intelligence = INTELIGÊNCIA
aircraft-rah66 =
    .facts =
        Stealth scout helicopter
        Bays: Hellfires or Stingers
        Hard to see on radar, slow or hovering
