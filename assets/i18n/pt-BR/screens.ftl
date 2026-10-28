# A prancheta de perna (src/kneeboard.rs, fonte vetorial), o planejador de missão
# (src/planner.rs) e as missões salvas (src/savegame.rs). As abreviações de navegação (PT, BRG,
# ETA, PRI, SEC, KM, KT) ficam em inglês.

kb-brief = BRIEF
kb-route = ROTA
kb-map = MAPA
kb-target = ALVO
kb-keys = TECLAS
kb-tip = Mapas, notas da missão e o guia de teclas
kb-geran-file = GERAN-2 (FICHA)
kb-objectives = OBJETIVOS
kb-primary = PRINCIPAL: { $objective }
kb-secondary = SECUNDÁRIO: { $objective }
kb-times = HORÁRIOS
kb-deadline = LIMITE { $time }
kb-now = AGORA { $time }
kb-conditions = CONDIÇÕES
kb-moon = LUA: { $phase } ({ $lit }% ILUMINADA)
kb-armament = ARMAMENTO
kb-roe = REGRAS DE ENGAJAMENTO
kb-total = TOTAL { $km } KM A { $kt } KT
kb-fuel = COMBUSTÍVEL { $lbs } LB
kb-preparing = PREPARANDO...
kb-airborne = EM VOO
kb-developing = REVELANDO...
kb-no-photo = SEM FOTO
kb-airborne-target = ALVO EM VOO
kb-scroll-vr = ROLAR: THUMBSTICK, DEDO NA PRANCHETA
kb-scroll-desktop = ROLAR: RODA DO MOUSE SOBRE A PRANCHETA

## Planejador de missão

planner-low = BAIXO 500 FT
planner-medium = MÉDIO 5.000 FT
planner-high = ALTO 20.000 FT
planner-title = PLANEJADOR DE MISSÃO
planner-route = Rota
planner-base = BASE
planner-total = TOTAL { $km } KM  { $time }
planner-briefed = Rota do briefing { $km } km, combustível { $fuel } lb
planner-coverage = Cobertura radar para
    .note = A cobertura radar supõe voo nivelado a 480 kt, portas do compartimento fechadas.
planner-detected = { $km } km da rota detectados
planner-pulse = Alcance de detecção do radar pulsado (P)
planner-doppler = Alcance de detecção do radar Doppler (D)
planner-missile = Alcance dos mísseis
planner-uav = VANT inimigo
planner-edit-hint = Arraste um ponto para movê-lo, o mapa para deslocá-lo. Clique em um trecho para adicionar um ponto, clique direito para removê-lo. Roda: zoom.
planner-reset = REDEFINIR
planner-view-hint = Arraste o mapa para deslocá-lo. Roda: zoom.

## Missões salvas

saves-title = MISSÕES SALVAS
saves-subtitle = Salvas em voo pelo menu: SALVAR MISSÃO
saves-none = Nenhuma missão salva
saves-line = { $theater }  -  { $aircraft }  -  tempo de missão { $time }
saves-saved-at = Salva em { $date }
saves-unreadable = Salvamento ilegível (outra versão)
saves-delete =
    .facts = Excluir este salvamento
saves-back =
    .facts = Voltar à tela de missão avulsa
