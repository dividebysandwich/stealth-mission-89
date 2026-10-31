# As mensagens do jogo no HUD e nos painéis 2D do voo (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). O HUD as exibe em maiúsculas sem acentos; as palavras da
# própria aviônica (CCIP, THR, KM, MAX...) ficam em inglês.

hud-no-ejection-seat = SEM ASSENTO EJETÁVEL
hud-waypoint-reached = Waypoint { $waypoint } alcançado
hud-select-camera = SELECIONAR CÂMERA
hud-open-bay = ABRIR COMPARTIMENTO
hud-camera-page = PÁGINA { $page }
hud-target-in-view-shoot = ALVO À VISTA - FOTOGRAFAR
hud-target-in-view = ALVO À VISTA
hud-aim-at-target = MIRAR NO ALVO
hud-photo = FOTO { $state }
hud-rearmed = Rearmado e reabastecido
hud-landed-rearm = Pousado - rearme ou encerre a surtida
hud-objective-achieved = Objetivo cumprido

## Tela de fim de missão

end-accomplished = cumprido
end-not-accomplished = não cumprido
end-primary = Objetivo principal { $result }
end-secondary = Objetivo secundário { $result }
end-kills = Abates: { $air } aeronaves, { $ground } alvos terrestres
end-time = Tempo de missão { $time }
end-damage = Danos: { $hits } impactos
end-training = Missão de treinamento
end-press-any-key = Pressione qualquer tecla

## O painel do piloto abatido

csar-flare = SINALIZADOR
csar-faster = MAIS RÁPIDO
csar-normal = TEMPO NORMAL
csar-skip = PULAR EXTRAÇÃO
csar-parachute = PARAQUEDAS ABERTO
csar-on-ground = NO SOLO
csar-down = ABATIDO EM TERRITÓRIO INIMIGO
csar-pickup = RESGATE   UH-60 SOBRE VOCÊ - RECOLHIMENTO
csar-inbound = RESGATE   UH-60 A { $km } KM { $direction } - SE APROXIMANDO
csar-fire-flare = RESGATE   UH-60 A { $km } KM { $direction } - DISPARE UM SINALIZADOR QUANDO ESTIVER PERTO
csar-enemy = INIMIGOS { $count } HIND, O MAIS PRÓXIMO A { $km } KM
csar-flares = SINALIZ. { $count }      NO SOLO HÁ { $time }

## Avisos da cooperação no painel (src/player_notices.rs), nota do console de defesa aérea

notice-joined = { $pilot } entrou
notice-left = { $pilot } saiu
notice-lost-connection = { $pilot } perdeu a conexão
st-select-track = Selecione uma trilha primeiro
