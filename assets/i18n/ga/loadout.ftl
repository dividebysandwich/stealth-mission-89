# Eagarthóir an armlóin (src/loadout.rs). Is iad ainmneacha fíor na n-arm atá ann, gan aistriú.

loadout-none = Ceann ar bith
loadout-briefed = BRIEFING
    .facts = Na stóir a pleanáladh sa bhriefing
loadout-clear = FOLAMH
    .facts = Gach pointe armála folamh
loadout-done = DÉANTA
loadout-no-hardpoints = Gan pointí armála
loadout-no-stores = Ní iompraíonn an t-aerárthach seo aon stóir
loadout-fantasy = Níl ar an aerárthach fíor
loadout-empty = Folamh
loadout-out-of-stock = Gan aon cheann i stoc don mhisean seo
loadout-mirror = Mar an gcéanna ar an dá thaobh
    .facts = Luchtaíonn { $station } freisin
loadout-stores = Stóir
loadout-needs-equipment = Teastaíonn a cheamara nó trealamh speisialta ón misean

## Cad chuige a bhfuil gach arm

# AIM-9M Sidewinder
weapon-role-0 = Aer-go-haer, baile infridhearg, raon gearr
# AIM-120A AMRAAM
weapon-role-1 = Aer-go-haer, baile radair, raon meánach
# AGM-88A HARM
weapon-role-2 = Diúracán frith-radair: radair agus láithreáin SAM
# Penguin 3 ASM
weapon-role-3 = Diúracán frith-loinge
# AGM-84A Harpoon
weapon-role-4 = Diúracán frith-loinge fadraoin
# AGM-65 Maverick
weapon-role-5 = Diúracán treoraithe ag teilifís: feithiclí, buncair, foirgnimh
# GBU-12 Paveway
weapon-role-6 = Buama treoraithe ag léasar: spriocanna crua
# Mk 20 Rockeye
weapon-role-7 = Buama braisle: spriocanna boga agus achair
# Durandal
weapon-role-8 = Buama chráitéar rúidbhealaigh
# Mk 82 LDGP
weapon-role-9 = Buama titim shaor 500 lb
# Mk 82 Snakeye
weapon-role-10 = Buama moillithe 500 lb do scaoileadh ísealairde
# Mk 20 Rockeye II
weapon-role-11 = Buama braisle: armúr agus spriocanna achair
# Mk 122 Fireye
weapon-role-12 = Buama dóiteáin
# CBU-72 FAE
weapon-role-13 = Pléascán aer-breosla: spriocanna achair
# Mk 35 incendiary
weapon-role-14 = Buama braisle dóiteáin
# ISCB-1 mines
weapon-role-15 = Mianaigh chosc achair
# IR/optical camera
weapon-role-16 = Grianghraif fhaisnéise den sprioc
# 1900 lb fuel tank
weapon-role-17 = Breosla breise, aistrithe nuair a scaoiltear é
# Special cargo
weapon-role-18 = Soláthairtí nó lasta don mhisean
# 20 mm cannon
weapon-role-19 = An gunna (ní stór é)
# AGM-114 Hellfire
weapon-role-20 = Diúracán frith-armúir treoraithe ag léasar: eitlíonn sé go dtí an áit a bhfuil an targeting pod ag féachaint

## Conas a úsáidtear gach arm (raonta i km an HUD, airde i bhfeet)

employment-0 = Raon: 1-12 km, níos faide ar aghaidh
employment-1 = Raon: 1-38 km, níos faide ar aghaidh
employment-2 =
    Raon: 9-50 km
    Lainseáil: 5,000-20,000 ft
employment-3 =
    Raon: 6-14 km (treoir chlasaiceach: 13-14 km)
    Lainseáil: 100-500 ft
employment-4 =
    Raon: 6-41 km
    Lainseáil: 2,000-5,000 ft
employment-5 =
    Raon: 5-10 km
    Lainseáil: 2,000-5,000 ft
employment-6 =
    Raon: 1.5 km go 2.5 oiread an airde, LOCK seasta (14 km ó 15,000 ft, 18 km ó 25,000 ft)
    Scaoileadh: 15,000-25,000 ft
employment-8 =
    Raon: 0.9-1.3 km roimh an rúidbhealach
    Scaoileadh: 200-500 ft
employment-9 =
    Raon: 5-11 km roimh an sprioc
    Scaoileadh: 3,000-20,000 ft
employment-12 =
    Raon: 0.9-2.2 km roimh an sprioc
    Scaoileadh: 100-500 ft
employment-16 = Raon: grianghraif ó 2-8 km (dá ghiorra, is mó naimhde), srón ar an sprioc
employment-18 = Raon: scaoil ag "Drop supplies now", laistigh de 4 km den limistéar scaoilte
employment-20 =
    Raon: 4-8 km
    Lainseáil: 2,000-3,000 ft
employment-guided-cluster =
    Raon: 1.5 km go 2.5 oiread an airde, LOCK seasta (3.7 km ó 3,000 ft, 6 km ó 5,000 ft)
    Scaoileadh: 3,000-5,000 ft
employment-medium-retarded =
    Raon: 1.1-1.6 km roimh an sprioc
    Scaoileadh: 300-1,000 ft
employment-low-retarded =
    Raon: 0.9-1.3 km roimh an sprioc
    Scaoileadh: 200-500 ft

## Pointí armála

hardpoint-left-bay = Clós clé
hardpoint-right-bay = Clós deas
hardpoint-left-wing-bay = Clós sciatháin chlé
hardpoint-right-wing-bay = Clós sciatháin dheis
hardpoint-station-1-left-wingtip = Stáisiún 1, barr sciatháin chlé
hardpoint-station-2-left-outer-wing = Stáisiún 2, sciathán clé seachtrach
hardpoint-station-3-left-wing = Stáisiún 3, sciathán clé
hardpoint-station-4-left-inner-wing = Stáisiún 4, sciathán clé istigh
hardpoint-station-5-centreline = Stáisiún 5, lárlíne
hardpoint-station-6-right-inner-wing = Stáisiún 6, sciathán deas istigh
hardpoint-station-7-right-wing = Stáisiún 7, sciathán deas
hardpoint-station-8-right-outer-wing = Stáisiún 8, sciathán deas seachtrach
hardpoint-station-9-right-wingtip = Stáisiún 9, barr sciatháin dheis
hardpoint-station-1-left-wingtip-rail = Stáisiún 1, ráille barr sciatháin chlé
hardpoint-station-2-left-outer-pylon = Stáisiún 2, pylon clé seachtrach
hardpoint-station-3-left-inner-pylon = Stáisiún 3, pylon clé istigh
hardpoint-station-4-right-inner-pylon = Stáisiún 4, pylon deas istigh
hardpoint-station-5-right-outer-pylon = Stáisiún 5, pylon deas seachtrach
hardpoint-station-6-right-wingtip-rail = Stáisiún 6, ráille barr sciatháin dheis
hardpoint-tornado-left-wing-rail = Sciathán clé, ráille Sidewinder
hardpoint-tornado-left-wing-pylon = Pylon sciatháin chlé
hardpoint-tornado-left-shoulder-forward = Gualainn chlé, chun tosaigh
hardpoint-tornado-left-shoulder-aft = Gualainn chlé, chun deiridh
hardpoint-tornado-right-shoulder-aft = Gualainn dheas, chun deiridh
hardpoint-tornado-right-shoulder-forward = Gualainn dheas, chun tosaigh
hardpoint-tornado-right-wing-pylon = Pylon sciatháin dheis
hardpoint-tornado-right-wing-rail = Sciathán deas, ráille Sidewinder
hardpoint-a10-1-left-outer-wing = Stáisiún 1, sciathán clé seachtrach
hardpoint-a10-2-left-wing = Stáisiún 2, sciathán clé
hardpoint-a10-3-left-wing = Stáisiún 3, sciathán clé
hardpoint-a10-4-left-centre-wing = Stáisiún 4, sciathán clé láir
hardpoint-a10-5-left-fuselage = Stáisiún 5, fiúslóg chlé
hardpoint-a10-6-centreline = Stáisiún 6, lárlíne
hardpoint-a10-7-right-fuselage = Stáisiún 7, fiúslóg dheas
hardpoint-a10-8-right-centre-wing = Stáisiún 8, sciathán deas láir
hardpoint-a10-9-right-wing = Stáisiún 9, sciathán deas
hardpoint-a10-10-right-wing = Stáisiún 10, sciathán deas
hardpoint-a10-11-right-outer-wing = Stáisiún 11, sciathán deas seachtrach
hardpoint-av8b-1-left-rail = Stáisiún 1, ráille Sidewinder clé
hardpoint-av8b-3-left-middle-wing = Stáisiún 3, sciathán clé, lár
hardpoint-av8b-4-left-inboard-wing = Stáisiún 4, sciathán clé, istigh
hardpoint-av8b-6-right-inboard-wing = Stáisiún 6, sciathán deas, istigh
hardpoint-av8b-7-right-middle-wing = Stáisiún 7, sciathán deas, lár
hardpoint-av8b-9-right-rail = Stáisiún 9, ráille Sidewinder deas
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
