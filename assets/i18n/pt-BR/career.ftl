# Carreiras dos pilotos (src/career.rs): postos, condecorações, período de serviço.

## Postos da US Air Force (traduzidos pelos equivalentes da FAB), suas abreviaturas

rank-0 = Segundo-tenente
    .short = 2º Ten
rank-1 = Primeiro-tenente
    .short = 1º Ten
rank-2 = Capitão
    .short = Cap
rank-3 = Major
    .short = Maj
rank-4 = Tenente-coronel
    .short = Ten Cel
rank-5 = Coronel
    .short = Cel

## Condecorações (medalhas americanas: seus nomes próprios)

medal-air-medal = Air Medal
medal-distinguished-flying-cross = Distinguished Flying Cross
medal-silver-star = Silver Star
medal-air-force-cross = Air Force Cross
medal-medal-of-honor = Medal of Honor
medal-purple-heart = Purple Heart
medal-prisoner-of-war-medal = Prisoner of War Medal

## Notícias após uma surtida

career-awarded = Condecorado: { $medal }
career-kia = Morto em combate
    .short = KIA
career-pow = Prisioneiro de guerra
    .short = POW
career-active = Na ativa
career-captured = Feito prisioneiro: a carreira terminou
career-tour-sortie = Período de serviço: surtida { $sortie } de { $length }
career-tour-complete = Período de serviço concluído: { $points } pontos (+{ $bonus })
career-promoted = Promovido a { $rank }

## A janela de carreira

career-title = FICHAS DOS PILOTOS
career-group-tour = Período
career-set-by-tour = Definido pelo período de serviço do piloto
career-pilots = Pilotos
career-new-pilot = Novo piloto
career-name = Nome
career-enlist = ALISTAR
career-select = Selecione ou aliste um piloto
career-service-record = Folha de serviço
career-rank = POSTO
career-points = PONTOS
career-sorties = SURTIDAS
career-tours = PERÍODOS
career-status = SITUAÇÃO
career-decorations = Condecorações
career-tour-of-duty = Período de serviço
career-tour-line = { $theater }  { $situation }  surtida { $sortie } de { $length }  { $points } pontos
career-destroyed = { $count ->
    [one] { $count } alvo destruído até agora
   *[other] { $count } alvos destruídos até agora
}
career-fly-next = VOAR A PRÓXIMA SURTIDA
career-new-tour = { $count ->
    [one] { $count } surtida
   *[other] { $count } surtidas
} - teatro: { $theater }, { $situation }, adversários: { $opponents }
    .facts = O que você destrói continua destruído até o fim do período de serviço
career-start-tour = INICIAR PERÍODO
career-retired = Transferido para a reserva
career-last-sorties = Últimas surtidas
