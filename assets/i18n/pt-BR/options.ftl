# A janela OPÇÕES (src/options.rs) e suas páginas no menu em voo.

options-title = OPÇÕES
options-tab-game = JOGO
options-tab-graphics = GRÁFICOS
options-tab-sound = SOM
options-tab-controls = CONTROLES
options-footer = Salvo para os próximos voos; as opções de realismo são escolhidas a cada partida

## JOGO

options-section-language = Idioma
options-section-classic-modern = Clássico / Moderno
options-cockpit = Cockpit
    .facts =
        Moderno: telas de alta resolução com botões na moldura
        Clássico: o painel de instrumentos da era DOS e suas telas 77x60 (F-117A)
options-hud = HUD
    .facts =
        Moderno: HUD verde de alta resolução no estilo do F-16, com os modos NAV, AA e AG
        Clássico: o HUD da era DOS
options-avionics = Aviônicos
    .facts =
        Moderno: páginas MFD no estilo do F-16 em cada MFD (situação, HARM, pod de designação, mapa, cargas) com um cursor móvel
        Clássico: mapa e TAC à esquerda, câmera, WAY, WPN e ORD à direita
options-flight-model = Modelo de voo
    .facts =
        Moderno: modelo de voo aerodinâmico com fly-by-wire: o manche comanda as razões de arfagem e de rolamento; solto, o nariz fica onde está e a trajetória é mantida
        Clássico: um modelo de voo da era DOS
options-section-flying = Pilotagem
options-auto-trim = Trim automático
    .facts =
        Com o manche solto, o trim mantém a atitude onde você a deixou
        Helicópteros: o nariz não sobe mais com a velocidade; jatos: compensa superfícies de controle danificadas
        Trim manual: Alt + setas, Alt + Home o centraliza
options-tooltips = Tooltips do cockpit
    .facts = O que é um comando ou uma tela do cockpit, sob o mouse ou a ponta do dedo em VR
options-player-labels = Nomes dos jogadores
    .facts = Multijogador: os nomes dos outros pilotos sobre suas aeronaves, a qualquer distância
options-label-size =
    .facts =
        Tamanho dos nomes dos jogadores
        50%: o padrão
options-seat-height = Altura do assento
    .facts =
        A altura em que você se senta na cabine, na tela e em VR
        0 cm: a altura normal dos olhos na aeronave
options-section-vr-size = Tamanho do cockpit em VR
options-vr-size =
    .facts =
        Tamanho do cockpit (e do mundo) no headset
        100%: tamanho real
options-section-gif = Gravação de GIF
options-gif-size = TAMANHO
options-gif-rate = FPS
options-gif-scale-100 =
    .facts = A resolução total da janela: arquivos grandes
options-gif-scale-25 =
    .facts = Arquivos pequenos
options-gif-rate-50 =
    .facts = Só quando o jogo roda a 100 fps ou mais (um quadro a cada dois é capturado)
options-gif-hint = F12 (salvo se remapeado) inicia e para; salvo na sua pasta Imagens

## GRÁFICOS

options-section-rendering = Renderização
options-shaders = Shaders
    .facts =
        Moderno: sombreamento direcional sutil, névoa de distância suave
        Clássico: sombreamento plano
options-shadows = Sombras
    .facts =
        Moderno: sombras do sol no cockpit e no mundo (consome tempo de GPU)
        Clássico: sem sombras
options-point-lights = Luzes pontuais
    .facts =
        Moderno: iluminação do cockpit e das telas; explosões, plumas e luzes de pista iluminam a cena
        Clássico: só a luz do sol e do céu
options-time-of-day = Hora do dia
    .facts =
        Moderno: o sol e a lua seguem o relógio da missão, o céu escurece e clareia com o sol; estrelas e fase da lua à noite
        Moderno: o luar define a que distância os pilotos inimigos veem você à noite (2 km sem lua, 4 km com lua cheia)
        Clássico: uma lenta mudança de cores e um sol fixo
options-msaa = Antisserrilhamento (MSAA)
    .facts =
        Bordas suaves: MSAA 4x, 2x em VR (consome tempo de GPU)
        Desligado: bordas de pixel nítidas
options-auto-detail = Detalhe automático do cenário (VR)
    .facts =
        No headset, o alcance de cidades, florestas e campos diminui quando os quadros atrasam e volta a crescer quando sobra tempo
        As configurações gráficas ficam como escolhidas
options-sun-moon = Sol e lua
    .facts =
        O sol no céu de dia, a lua e as estrelas à noite
        A lua mostra a fase da missão
options-cloud-quality = Nuvens
cloud-quality-auto = Auto
    .facts = Baixa em RV, alta na tela
cloud-quality-low = Baixa
    .facts = Nuvens grosseiras a um terço da resolução (mais rápido)
cloud-quality-medium = Média
    .facts = Metade da resolução, mais detalhes
cloud-quality-high = Alta
    .facts = Resolução total, máximo detalhe e sombra própria (custa tempo de GPU)

## SOM

options-section-volume = Volume
options-volume-master = Geral
    .facts = Todos os sons
options-volume-engine = Motor
    .facts = Seu motor ou rotor, e as aeronaves ouvidas ao seu redor
options-volume-effects = Efeitos
    .facts = Armas, explosões, trem de pouso, rádio e o resto
options-volume-warnings = Alarmes
    .facts = Alarme de estol, alertas de radar e de míssil, a sirene de incêndio, os tons do Sidewinder
options-volume-rwr = RWR
    .facts = Os tons de alerta de radar, como parte do volume dos alarmes
options-section-rwr-tones = Tons de alerta de radar
rwr-tones-all = TODOS
    .facts = Um radar iluminando você, cada varredura que o detecta, os lançamentos de mísseis
rwr-tones-detect = SÓ DETECÇÃO
    .facts =
        Um tom quando um radar o detecta (de novo depois que os radares o soltam por um tempo)
        E os lançamentos de mísseis
rwr-tones-launch = SÓ LANÇAMENTOS
    .facts = Só os mísseis lançados contra você
rwr-tones-off = NENHUM
    .facts = Nenhum tom de alerta de radar (as luzes e telas ainda alertam)
options-section-cockpit = Cockpit
options-low-pass = Passa-baixa do motor
    .facts =
        Dentro da aeronave o motor soa abafado (o canopi e o capacete cortam os agudos)
        Desligado: o mesmo som de fora

## CONTROLES

options-section-bindings = Atribuições
options-keys-joystick = TECLAS E JOYSTICK
    .facts =
        Atribuir teclas, eixos e botões do joystick e do HOTAS
        Salvo para os próximos voos
options-section-head-tracking = Rastreamento de cabeça/olhos
options-head-tracking = Rastreamento de cabeça/olhos
    .facts =
        Sua cabeça gira e move a vista no cockpit (desktop; em VR, o headset faz isso)
        Programa de rastreamento (OpenTrack...): saída FreeTrack, ou UDP para 127.0.0.1 porta 4242
        Home: a posição atual da cabeça vira o centro
options-tracking = Rastreamento ({ $source })
options-no-tracker = Nenhum rastreador detectado: inicie o programa de rastreamento

## EFFECTS

options-section-effects = Efeitos
options-section-retro = Retrô
effect-shake = Tremor da câmera
    .facts =
        A vista treme com o canhão, explosões próximas, perda de sustentação, pousos e a pista
        VR: a cabine vibra, sua cabeça fica parada
effect-flashes = Clarões de combate
    .facts = Explosões de antiaérea, explosões e lançamentos de mísseis iluminam por um instante as nuvens, o chão e a cabine
effect-barrage = Barragem noturna
    .facts =
        Guerra convencional à noite: os canhões das cidades disparam traçantes às cegas para o céu, holofotes varrem por cima
        Drones inimigos sobre uma cidade são pegos por seus holofotes e atraem o fogo de seus canhões
        Só pelo visual: a barragem não atinge nada
effect-cockpit-night = Iluminação noturna da cabine
    .facts =
        Uma luz vermelha fraca na cabine depois de escurecer (azul-esverdeada sob os óculos)
        A master caution pulsa em âmbar sobre o painel enquanto há um alerta ativo
effect-nvg = Óculos de visão noturna
    .facts =
        Luzes fortes viram halos, a imagem fica mais granulada no escuro
        Desktop: o campo de visão redondo dos óculos
effect-snow = Neve
    .facts = Acima do nível de congelamento as tempestades trazem neve em vez de chuva: baixo no inverno do norte, alto no verão
effect-mist = Névoa no solo
    .facts =
        Shaders modernos: a névoa cobre o terreno baixo e o mar ao amanhecer, ao entardecer e à noite
        Fina nos desertos, nenhuma em tempestades
effect-cloud-shadows = Sombras de nuvens
    .facts = Shaders modernos: as sombras das nuvens no chão abaixo
effect-vapour = Vapor e fluxo do rotor
    .facts =
        Em curvas fechadas, rastros de vapor nas pontas das asas
        Um helicóptero pairando baixo levanta poeira, ou spray sobre a água
        Veículos em movimento levantam poeira atrás de si
effect-night-sky = Céu noturno
    .facts =
        De vez em quando estrelas cadentes, as estrelas baixas no horizonte cintilam
        A aurora boreal sobre o Cabo Norte
effect-vga = Cores VGA
    .facts = O visual de 1989 com sombreamento moderno: 64 níveis por canal de cor, como numa placa VGA, com dithering
effect-scanlines = Linhas de varredura
    .facts = As linhas de varredura de um monitor CRT (não em VR)

## LANGUAGE NAMES

lang-name-en = inglês
lang-name-bg = búlgaro
lang-name-zh-Hans = chinês (simplificado)
lang-name-hr = croata
lang-name-cs = tcheco
lang-name-da = dinamarquês
lang-name-nl = holandês
lang-name-et = estoniano
lang-name-fi = finlandês
lang-name-fr = francês
lang-name-de = alemão
lang-name-el = grego
lang-name-hu = húngaro
lang-name-id = indonésio
lang-name-ga = irlandês
lang-name-it = italiano
lang-name-ja = japonês
lang-name-ko = coreano
lang-name-lv = letão
lang-name-lt = lituano
lang-name-mt = maltês
lang-name-pl = polonês
lang-name-pt-BR = português (Brasil)
lang-name-ro = romeno
lang-name-ru = russo
lang-name-sk = eslovaco
lang-name-sl = esloveno
lang-name-es = espanhol
lang-name-sv = sueco
lang-name-uk = ucraniano

## Browser build: offer to play in the browser's language (src/i18n.rs)

language-offer-question = Idioma do navegador: { $language }. Jogar neste idioma?
language-offer-yes = Sim
language-offer-no = Não
language-offer-never = Não perguntar novamente
effect-sunbeams = Raios de sol
    .facts = Feixes de luz do sol pelas frestas de uma cobertura de nuvens quebrada
options-msaa-forced = Desligado enquanto a pixelização (retrô) estiver ativa
options-volume-radio = Rádio
    .facts = Os outros jogadores falando no rádio coop, e o ATC falado
options-volume-atc = Voz do ATC
    .facts = As chamadas de ATC faladas, como parte do volume do rádio
options-volume-music = Música
    .facts = A música do menu principal
options-section-atc = ATC falado
options-section-radio = Rádio coop
options-push-to-talk = Aperte para falar
    .facts =
        Segure a tecla de falar (Caps Lock, ou um botão de joystick ou VR atribuído ao PTT) e todos os jogadores ouvem você
        O microfone abre na primeira vez que você aperta
options-mic-gain = GANHO DO MIC
    .facts =
        Quão alto o seu microfone sai
        100%: como está
options-mic-default = Padrão
options-voice-text-only = só texto
options-voices-line = Controladores: { $controllers }   Piloto: { $pilot }
options-voices = VOZES
    .facts =
        Baixar vozes para as chamadas de rádio, apagá-las
        Escolher as dos controladores e a sua, e ouvi-las
options-voices-web = Só texto no navegador
options-tracking-mouse = Olhar com o mouse no tracking
    .facts =
        Ligado: o botão direito do mouse ainda gira e dá zoom na vista da cabine
        Desligado: enquanto o tracker envia, o botão direito do mouse não faz nada na cabine
options-atc = Controle de tráfego aéreo
    .facts =
        Chamadas de rádio de torre, aproximação e GCA nos aeródromos aliados
        Autorização de decolagem, break de chegada e um talkdown com nuvens baixas ou pouca visibilidade
options-pixel-block = Tamanho do bloco da pixelização
    .facts = O lado de um bloco, em pixels de tela (2: fino, 5: grosso)
effect-pixelize = Pixelização
    .facts =
        Toda a vista 3D (terreno, aeronaves, prédios, cabine) em pixels quadrados grandes, como num PC com MS-DOS; os menus e os tooltips continuam nítidos (não em VR)
        O controle deslizante define o tamanho de um bloco, em pixels de tela (2: fino, 5: grosso)
        O antisserrilhado fica desligado enquanto ela estiver ativa
        As três chaves abaixo deixam nítidos o HUD, o cockpit ou os MFDs, os instrumentos e a prancheta de perna
effect-sharp-hud = Sem HUD
    .facts =
        Pixelização: o HUD (e o display do capacete) é desenhado nítido sobre a vista pixelizada, para que seus símbolos finos continuem legíveis
effect-sharp-cockpit = Sem cockpit (botões, chaves...)
    .facts =
        Pixelização: o painel, a estrutura, o assento, os botões, as chaves, o manche e a manete do cockpit são desenhados nítidos
effect-sharp-gauges = Sem MFDs, instrumentos e prancheta de perna
    .facts =
        Pixelização: os MFDs, a esfera de atitude, os instrumentos, as luzes, os mostradores, os rótulos e a prancheta de perna são desenhados nítidos
options-needs-pixelize = Requer Pixelização
