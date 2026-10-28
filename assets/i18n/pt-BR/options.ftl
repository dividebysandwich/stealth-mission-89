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
options-section-vr-size = Tamanho do cockpit em VR
options-vr-size =
    .facts =
        Tamanho do cockpit (e do mundo) no headset
        100%: tamanho real

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
