# A janela de campanha (src/campaign/menu.rs), a janela multijogador (src/multiplayer_menu.rs),
# a legenda do mapa de guerra e a visão ao vivo (src/campaign/picture.rs, unit_view.rs).

delete = EXCLUIR
delete-campaign = EXCLUIR ESTA CAMPANHA?
delete-save = EXCLUIR ESTE SALVAMENTO?
delete-permanent = A exclusão é definitiva

## Janela de campanha

cm-group-campaign = Campanha
cm-weather-dynamic = Dinâmico
    .facts =
        O tempo muda a cada 10 a 22 horas do relógio do céu
        As nuvens bloqueiam a linha de visada dos dois lados
cm-weather-clear = Céu limpo
    .facts = Nenhuma nuvem durante toda a campanha
cm-section-day-night = Dia e noite
cm-day-real = Tempo real
    .facts = Um dia dura 24 horas
cm-day-2h = 2 h / dia
    .facts = Um dia dura 2 horas
cm-day-1h = 1 h / dia
    .facts = Um dia dura 1 hora
cm-day-30min = 30 min / dia
    .facts = Um dia dura 30 minutos
cm-conditions-note = As condições se aplicam a uma nova campanha
cm-rules-note = Uma campanha salva mantém as regras com que foi iniciada
cm-about =
    A guerra corre em tempo real enquanto você joga, mesmo com o menu aberto.
    Voe as missões do quadro: recon, SEAD, CAP, CAS, antinavio, ataque, escolta.
    As infiltrações de helicóptero tomam as cidades e os aeródromos do inimigo.
    Você nunca é morto nem capturado: depois de uma surtida, você volta ao quadro.
cm-section-new = Nova campanha
cm-new-line = { $theater }  -  { $situation }  -  adversários { $opponents }
cm-start = INICIAR
cm-section-continue = Continuar
cm-no-saves = Nenhuma campanha salva
cm-played = { $hours } h jogadas
cm-save-gone = O salvamento não existe mais
cm-save-unreadable = Não foi possível ler o salvamento: { $error }
cm-save-undeletable = Não foi possível excluir o salvamento: { $error }
cm-in-vr = EM VR
cm-delete =
    .facts = Excluir esta campanha salva

## Mapa de guerra

pic-friendly-aircraft = Aeronave amiga
pic-friendly-helicopter = Helicóptero amigo
pic-enemy-fighter = Caça inimigo (no radar)
pic-sam-known = Sítio SAM (conhecido)
pic-guns-known = Canhões / MANPADS (conhecidos)
pic-reconned = Reconhecido
pic-you = Você
pic-site = Sítio de { $place }
uv-live = { $unit } - AO VIVO
uv-contact-lost = CONTATO PERDIDO
uv-hint = Arrastar: orbitar  Roda: zoom

## Multijogador

mp-disconnected = Desconectado: { $why }
mp-connecting = Conectando a { $address }...
mp-joining = Conectado a { $address }, entrando...
mp-browser-public-only = pelo navegador, só é possível entrar em servidores da lista pública
mp-title-host = MULTIJOGADOR: HOSPEDAR UMA PARTIDA
mp-subtitle = Cooperativo na campanha dinâmica: voem a guerra juntos
mp-your-name = Seu nome
    .hint = Nome do piloto
mp-join-address = Entrar por endereço
    .hint = endereço ou nome[:porta]
mp-connect = CONECTAR
mp-server-name = Nome do servidor
mp-udp-port = Porta UDP
mp-address-note = Os outros jogadores entram com o endereço deste computador; abra a porta UDP no seu roteador para jogar pela internet
mp-list-public = Listar na lista pública de servidores (os outros encontram a partida na lista de servidores deles)
mp-rules = Regras para todos
mp-time-of-day = Hora do dia (noites de luar)
mp-realistic-weapons = Armamento realista do F-117A
mp-friendly-fire = Fogo amigo
mp-day-night = Dia e noite:
mp-weather = Tempo:
mp-host-new = HOSPEDAR NOVA
mp-host-saved = Hospedar uma campanha salva
mp-host = HOSPEDAR
mp-host-game = HOSPEDAR UMA PARTIDA
mp-port-error = Porta UDP { $port }: { $error }
mp-default-server = Campanha de { $pilot }
mp-public-servers = Servidores públicos
mp-refresh = ATUALIZAR
mp-server-count = { $servers } servidores, { $pilots } pilotos
mp-list-error = A lista de servidores está inacessível: { $error }
mp-no-servers = Nenhum servidor público no momento
mp-col-server = SERVIDOR
mp-col-pilots = PILOTOS
mp-col-war = GUERRA
mp-dedicated = DEDICADO
mp-other-version = outra versão
mp-join = ENTRAR
