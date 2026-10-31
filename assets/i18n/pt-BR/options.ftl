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
effect-vapour = Vapor e fluxo do rotor
    .facts =
        Em curvas fechadas, rastros de vapor nas pontas das asas
        Um helicóptero pairando baixo levanta poeira, ou spray sobre a água
        Veículos em movimento levantam poeira atrás de si
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
effect-sunbeams = Sunbeams
    .facts =
        Shafts of sunlight through the gaps of a broken overcast
