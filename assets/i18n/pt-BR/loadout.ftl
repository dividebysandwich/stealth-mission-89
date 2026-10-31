# O editor de armamento (src/loadout.rs). Os nomes das armas são os reais, sem tradução.

loadout-none = Nenhum
loadout-briefed = BRIEFING
    .facts = As cargas previstas no briefing da missão
loadout-clear = LIMPAR
    .facts = Todos os pontos de fixação vazios
loadout-done = OK
loadout-no-hardpoints = Nenhum ponto de fixação
loadout-no-stores = Esta aeronave não leva cargas
loadout-fantasy = Não existe na aeronave real
loadout-empty = Vazio
loadout-out-of-stock = Nenhum em estoque para esta missão
loadout-mirror = Igual nos dois lados
    .facts = Carrega também { $station }
loadout-stores = Cargas
loadout-needs-equipment = A missão precisa de sua câmera ou de seu equipamento especial

## Para que serve cada arma

# AIM-9M Sidewinder
weapon-role-0 = Ar-ar, guiagem infravermelha, curto alcance
# AIM-120A AMRAAM
weapon-role-1 = Ar-ar, guiagem por radar, médio alcance
# AGM-88A HARM
weapon-role-2 = Míssil antirradiação: radares e sítios SAM
# Penguin 3 ASM
weapon-role-3 = Míssil antinavio
# AGM-84A Harpoon
weapon-role-4 = Míssil antinavio de longo alcance
# AGM-65 Maverick
weapon-role-5 = Míssil guiado por TV: veículos, bunkers, edifícios
# GBU-12 Paveway
weapon-role-6 = Bomba guiada a laser: alvos reforçados
# Mk 20 Rockeye
weapon-role-7 = Bomba de fragmentação: alvos leves e de área
# Durandal
weapon-role-8 = Bomba antipista
# Mk 82 LDGP
weapon-role-9 = Bomba de queda livre de 500 lb
# Mk 82 Snakeye
weapon-role-10 = Bomba retardada de 500 lb para lançamentos baixos
# Mk 20 Rockeye II
weapon-role-11 = Bomba de fragmentação: blindados e alvos de área
# Mk 122 Fireye
weapon-role-12 = Bomba incendiária
# CBU-72 FAE
weapon-role-13 = Explosivo ar-combustível: alvos de área
# Mk 35 incendiary
weapon-role-14 = Bomba de fragmentação incendiária
# ISCB-1 mines
weapon-role-15 = Minas de negação de área
# IR/optical camera
weapon-role-16 = Fotos de reconhecimento do alvo
# 1900 lb fuel tank
weapon-role-17 = Combustível extra, transferido ao ser lançado
# Special cargo
weapon-role-18 = Suprimentos ou carga para a missão
# 20 mm cannon
weapon-role-19 = O canhão (não é uma carga)
# AGM-114 Hellfire
weapon-role-20 = Míssil anticarro guiado a laser: voa até onde o pod de designação aponta

## Emprego de cada arma (alcances em km do HUD, alturas em pés)

employment-0 = Alcance: 1-12 km, mais de frente
employment-1 = Alcance: 1-38 km, mais de frente
employment-2 =
    Alcance: 9-50 km
    Disparo: 5.000-20.000 ft
employment-3 =
    Alcance: 6-14 km (guiagem clássica: 13-14 km)
    Disparo: 100-500 ft
employment-4 =
    Alcance: 6-41 km
    Disparo: 2.000-5.000 ft
employment-5 =
    Alcance: 5-10 km
    Disparo: 2.000-5.000 ft
employment-6 =
    Alcance: de 1,5 km a 2,5 vezes a altura, LOCK fixo (14 km a 15.000 ft, 18 km a 25.000 ft)
    Lançamento: 15.000-25.000 ft
employment-8 =
    Alcance: 0,9-1,3 km antes da pista
    Lançamento: 200-500 ft
employment-9 =
    Alcance: 5-11 km antes do alvo
    Lançamento: 3.000-20.000 ft
employment-12 =
    Alcance: 0,9-2,2 km antes do alvo
    Lançamento: 100-500 ft
employment-16 = Alcance: fotos a 2-8 km (quanto mais adversários, mais perto), nariz no alvo
employment-18 = Alcance: lançar em "Drop supplies now", a menos de 4 km da zona de lançamento
employment-20 =
    Alcance: 4-8 km
    Disparo: 2.000-3.000 ft
employment-guided-cluster =
    Alcance: de 1,5 km a 2,5 vezes a altura, LOCK fixo (3,7 km a 3.000 ft, 6 km a 5.000 ft)
    Lançamento: 3.000-5.000 ft
employment-medium-retarded =
    Alcance: 1,1-1,6 km antes do alvo
    Lançamento: 300-1.000 ft
employment-low-retarded =
    Alcance: 0,9-1,3 km antes do alvo
    Lançamento: 200-500 ft

## Pontos de fixação

hardpoint-left-bay = Compartimento esquerdo
hardpoint-right-bay = Compartimento direito
hardpoint-left-wing-bay = Compartimento da asa esquerda
hardpoint-right-wing-bay = Compartimento da asa direita
hardpoint-station-1-left-wingtip = Estação 1, ponta da asa esquerda
hardpoint-station-2-left-outer-wing = Estação 2, asa esquerda externa
hardpoint-station-3-left-wing = Estação 3, asa esquerda
hardpoint-station-4-left-inner-wing = Estação 4, asa esquerda interna
hardpoint-station-5-centreline = Estação 5, linha central
hardpoint-station-6-right-inner-wing = Estação 6, asa direita interna
hardpoint-station-7-right-wing = Estação 7, asa direita
hardpoint-station-8-right-outer-wing = Estação 8, asa direita externa
hardpoint-station-9-right-wingtip = Estação 9, ponta da asa direita
hardpoint-station-1-left-wingtip-rail = Estação 1, trilho da ponta da asa esquerda
hardpoint-station-2-left-outer-pylon = Estação 2, pilone esquerdo externo
hardpoint-station-3-left-inner-pylon = Estação 3, pilone esquerdo interno
hardpoint-station-4-right-inner-pylon = Estação 4, pilone direito interno
hardpoint-station-5-right-outer-pylon = Estação 5, pilone direito externo
hardpoint-station-6-right-wingtip-rail = Estação 6, trilho da ponta da asa direita
hardpoint-tornado-left-wing-rail = Asa esquerda, trilho do Sidewinder
hardpoint-tornado-left-wing-pylon = Pilone da asa esquerda
hardpoint-tornado-left-shoulder-forward = Ombro esquerdo, dianteiro
hardpoint-tornado-left-shoulder-aft = Ombro esquerdo, traseiro
hardpoint-tornado-right-shoulder-aft = Ombro direito, traseiro
hardpoint-tornado-right-shoulder-forward = Ombro direito, dianteiro
hardpoint-tornado-right-wing-pylon = Pilone da asa direita
hardpoint-tornado-right-wing-rail = Asa direita, trilho do Sidewinder
hardpoint-a10-1-left-outer-wing = Estação 1, asa esquerda externa
hardpoint-a10-2-left-wing = Estação 2, asa esquerda
hardpoint-a10-3-left-wing = Estação 3, asa esquerda
hardpoint-a10-4-left-centre-wing = Estação 4, centro da asa esquerda
hardpoint-a10-5-left-fuselage = Estação 5, fuselagem esquerda
hardpoint-a10-6-centreline = Estação 6, linha central
hardpoint-a10-7-right-fuselage = Estação 7, fuselagem direita
hardpoint-a10-8-right-centre-wing = Estação 8, centro da asa direita
hardpoint-a10-9-right-wing = Estação 9, asa direita
hardpoint-a10-10-right-wing = Estação 10, asa direita
hardpoint-a10-11-right-outer-wing = Estação 11, asa direita externa
hardpoint-av8b-1-left-rail = Estação 1, trilho Sidewinder esquerdo
hardpoint-av8b-3-left-middle-wing = Estação 3, asa esquerda intermediária
hardpoint-av8b-4-left-inboard-wing = Estação 4, asa esquerda interna
hardpoint-av8b-6-right-inboard-wing = Estação 6, asa direita interna
hardpoint-av8b-7-right-middle-wing = Estação 7, asa direita intermediária
hardpoint-av8b-9-right-rail = Estação 9, trilho Sidewinder direito
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
