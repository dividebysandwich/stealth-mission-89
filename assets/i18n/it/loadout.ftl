# L'editor dell'armamento (src/loadout.rs). I nomi delle armi sono quelli reali, non tradotti.

loadout-none = Nessuno
loadout-briefed = BRIEFING
    .facts = I carichi previsti dal briefing della missione
loadout-clear = SVUOTARE
    .facts = Tutti i punti di aggancio vuoti
loadout-done = FATTO
loadout-no-hardpoints = Nessun punto di aggancio
loadout-no-stores = Questo velivolo non trasporta carichi
loadout-fantasy = Non presente sull'aereo reale
loadout-empty = Vuoto
loadout-out-of-stock = Nessuno in dotazione per questa missione
loadout-mirror = Uguale su entrambi i lati
    .facts = Carica anche { $station }
loadout-stores = Carichi
loadout-realistic-f117 = Armamento realistico dell'F-117A
loadout-stations = Piloni
loadout-click-drawing = Oppure fai clic su un pilone nel disegno
loadout-needs-equipment = La missione richiede la sua telecamera o il suo equipaggiamento speciale

## A che cosa serve ogni arma (tipi di arma da 0 a 20)

# AIM-9M Sidewinder
weapon-role-0 = Aria-aria, guida a infrarossi, corto raggio
# AIM-120A AMRAAM
weapon-role-1 = Aria-aria, guida radar, medio raggio
# AGM-88A HARM
weapon-role-2 = Missile antiradiazione: radar e siti SAM
# Penguin 3 ASM
weapon-role-3 = Missile antinave
# AGM-84A Harpoon
weapon-role-4 = Missile antinave a lungo raggio
# AGM-65 Maverick
weapon-role-5 = Missile a guida TV: veicoli, bunker, edifici
# GBU-12 Paveway
weapon-role-6 = Bomba a guida laser: bersagli protetti
# Mk 20 Rockeye
weapon-role-7 = Bomba a grappolo: bersagli leggeri e d'area
# Durandal
weapon-role-8 = Bomba antipista
# Mk 82 LDGP
weapon-role-9 = Bomba a caduta libera da 500 lb
# Mk 82 Snakeye
weapon-role-10 = Bomba frenata da 500 lb per sganci a bassa quota
# Mk 20 Rockeye II
weapon-role-11 = Bomba a grappolo: mezzi corazzati e bersagli d'area
# Mk 122 Fireye
weapon-role-12 = Bomba incendiaria
# CBU-72 FAE
weapon-role-13 = Esplosivo aria-combustibile: bersagli d'area
# Mk 35 incendiary
weapon-role-14 = Bomba a grappolo incendiaria
# ISCB-1 mines
weapon-role-15 = Mine di interdizione d'area
# IR/optical camera
weapon-role-16 = Foto di ricognizione dell'obiettivo
# 1900 lb fuel tank
weapon-role-17 = Carburante supplementare, trasferito allo sgancio
# Special cargo
weapon-role-18 = Rifornimenti o carico per la missione
# 20 mm cannon
weapon-role-19 = Il cannone (non è un carico)
# AGM-114 Hellfire
weapon-role-20 = Missile anticarro a guida laser: vola dove guarda il pod di puntamento

## Impiego di ogni arma (distanze in km dell'HUD, quote in piedi)

employment-0 = Portata: 1-12 km, di più frontalmente
employment-1 = Portata: 1-38 km, di più frontalmente
employment-2 =
    Portata: 9-50 km
    Lancio: 5.000-20.000 ft
employment-3 =
    Portata: 6-14 km (guida classica: 13-14 km)
    Lancio: 100-500 ft
employment-4 =
    Portata: 6-41 km
    Lancio: 2.000-5.000 ft
employment-5 =
    Portata: 5-10 km
    Lancio: 2.000-5.000 ft
employment-6 =
    Portata: da 1,5 km a 2,5 volte la quota, LOCK fisso (14 km da 15.000 ft, 18 km da 25.000 ft)
    Sgancio: 15.000-25.000 ft
employment-8 =
    Portata: 0,9-1,3 km prima della pista
    Sgancio: 200-500 ft
employment-9 =
    Portata: 5-11 km prima dell'obiettivo
    Sgancio: 3.000-20.000 ft
employment-12 =
    Portata: 0,9-2,2 km prima dell'obiettivo
    Sgancio: 100-500 ft
employment-16 = Portata: foto a 2-8 km (più avversari, più vicino), muso sull'obiettivo
employment-18 = Portata: sganciare a "Drop supplies now", entro 4 km dalla zona di lancio
employment-20 =
    Portata: 4-8 km
    Lancio: 2.000-3.000 ft
employment-guided-cluster =
    Portata: da 1,5 km a 2,5 volte la quota, LOCK fisso (3,7 km da 3.000 ft, 6 km da 5.000 ft)
    Sgancio: 3.000-5.000 ft
employment-medium-retarded =
    Portata: 1,1-1,6 km prima dell'obiettivo
    Sgancio: 300-1.000 ft
employment-low-retarded =
    Portata: 0,9-1,3 km prima dell'obiettivo
    Sgancio: 200-500 ft

## Punti di aggancio

hardpoint-left-bay = Vano sinistro
hardpoint-right-bay = Vano destro
hardpoint-left-wing-bay = Vano alare sinistro
hardpoint-right-wing-bay = Vano alare destro
hardpoint-station-1-left-wingtip = Stazione 1, estremità alare sinistra
hardpoint-station-2-left-outer-wing = Stazione 2, ala sinistra esterna
hardpoint-station-3-left-wing = Stazione 3, ala sinistra
hardpoint-station-4-left-inner-wing = Stazione 4, ala sinistra interna
hardpoint-station-5-centreline = Stazione 5, ventrale
hardpoint-station-6-right-inner-wing = Stazione 6, ala destra interna
hardpoint-station-7-right-wing = Stazione 7, ala destra
hardpoint-station-8-right-outer-wing = Stazione 8, ala destra esterna
hardpoint-station-9-right-wingtip = Stazione 9, estremità alare destra
hardpoint-station-1-left-wingtip-rail = Stazione 1, rotaia d'estremità sinistra
hardpoint-station-2-left-outer-pylon = Stazione 2, pilone sinistro esterno
hardpoint-station-3-left-inner-pylon = Stazione 3, pilone sinistro interno
hardpoint-station-4-right-inner-pylon = Stazione 4, pilone destro interno
hardpoint-station-5-right-outer-pylon = Stazione 5, pilone destro esterno
hardpoint-station-6-right-wingtip-rail = Stazione 6, rotaia d'estremità destra
hardpoint-tornado-left-wing-rail = Ala sinistra, rotaia Sidewinder
hardpoint-tornado-left-wing-pylon = Pilone dell'ala sinistra
hardpoint-tornado-left-shoulder-forward = Spalla sinistra, anteriore
hardpoint-tornado-left-shoulder-aft = Spalla sinistra, posteriore
hardpoint-tornado-right-shoulder-aft = Spalla destra, posteriore
hardpoint-tornado-right-shoulder-forward = Spalla destra, anteriore
hardpoint-tornado-right-wing-pylon = Pilone dell'ala destra
hardpoint-tornado-right-wing-rail = Ala destra, rotaia Sidewinder
hardpoint-a10-1-left-outer-wing = Stazione 1, ala sinistra esterna
hardpoint-a10-2-left-wing = Stazione 2, ala sinistra
hardpoint-a10-3-left-wing = Stazione 3, ala sinistra
hardpoint-a10-4-left-centre-wing = Stazione 4, ala sinistra centrale
hardpoint-a10-5-left-fuselage = Stazione 5, fusoliera sinistra
hardpoint-a10-6-centreline = Stazione 6, asse centrale
hardpoint-a10-7-right-fuselage = Stazione 7, fusoliera destra
hardpoint-a10-8-right-centre-wing = Stazione 8, ala destra centrale
hardpoint-a10-9-right-wing = Stazione 9, ala destra
hardpoint-a10-10-right-wing = Stazione 10, ala destra
hardpoint-a10-11-right-outer-wing = Stazione 11, ala destra esterna
hardpoint-av8b-1-left-rail = Stazione 1, rotaia Sidewinder sinistra
hardpoint-av8b-3-left-middle-wing = Stazione 3, ala sinistra intermedia
hardpoint-av8b-4-left-inboard-wing = Stazione 4, ala sinistra interna
hardpoint-av8b-6-right-inboard-wing = Stazione 6, ala destra interna
hardpoint-av8b-7-right-middle-wing = Stazione 7, ala destra intermedia
hardpoint-av8b-9-right-rail = Stazione 9, rotaia Sidewinder destra
hardpoint-f22-left-side-bay = Vano laterale sinistro (Sidewinder)
hardpoint-f22-left-main-bay = Vano principale, metà sinistra
hardpoint-f22-right-main-bay = Vano principale, metà destra
hardpoint-f22-right-side-bay = Vano laterale destro (Sidewinder)
hardpoint-fa18e-8-right-inner-wing = Stazione 8, ala destra interna
hardpoint-fa18e-10-right-outer-wing = Stazione 10, ala destra esterna
hardpoint-fa18e-11-right-wingtip = Stazione 11, estremità alare destra
hardpoint-c5-hold = Stiva (pallet da aviolancio)
hardpoint-ch47-cabin = Cabina (paracadutisti)
weapon-role-21 = Aria-aria, guida a infrarossi, gittata molto breve
employment-21 = Gittata: 1-6 km, maggiore in frontale
loadout-none-in-stock = (esaurito)
loadout-empties = Svuota la stazione { $stations } quando è caricato
loadout-empties-two = Svuota le stazioni { $first } e { $second } quando è caricato
hardpoint-oh58-left-pylon = Pilone armi sinistro
hardpoint-oh58-right-pylon = Pilone armi destro
hardpoint-mh60r-left-outer = Stazione esterna sinistra (lanciatore Hellfire)
hardpoint-mh60r-left-inner = Stazione interna sinistra
hardpoint-mh60r-right-inner = Stazione interna destra
weapon-role-22 = Contenitore mitragliatrice: spara come il cannone, lungo il muso
employment-22 = Gittata: fino a 1,5 km, 500 colpi per contenitore
weapon-role-23 = Arma a spoletta sensoriale: quaranta skeet sparano verso il basso sui veicoli
employment-23 =
    Guidata al punto marcato con il pod di puntamento al rilascio
    Rilascio: sopra la quota di apertura impostata nella pagina SMS (500-3.000 ft)
weapon-role-24 = Siluro leggero: scende col paracadute, cerca sott'acqua e si dirige su un sommergibile
employment-24 =
    Programmato sulla pagina WPN dell'operatore sensori: rotta, ritardo di attivazione, schema di ricerca
    Sgancio: sull'acqua, sotto 300 ft e 100 kt
weapon-role-25 = Missile da attacco al suolo a distanza: vola fino al bersaglio, poi il pilota lo guida con l'immagine del suo sensore
employment-25 =
    Programmato sulla pagina SLAM: quota di crociera, distanza del sensore, bersaglio primario o secondario
    Portata: fino a 250 km, quattro in volo alla volta
weapon-role-26 = Bomba planante: plana fino al punto designato, la gittata cresce con quota e velocità di sgancio
employment-26 =
    Puntata sul punto del pod o sull'aggancio a terra, altrimenti sul bersaglio primario
    Gittata: circa 20 km a bassa quota, fino a 130 km da alta quota; la difesa aerea può abbatterla
weapon-role-27 = Esca aviolanciata: mostra alla difesa aerea la firma di un aereo e ne attira i missili
employment-27 =
    Vola fino al punto designato, poi dritto per al massimo 10 minuti
    Innocua per i bersagli
weapon-role-28 = Paracadutisti: uno stick di soldati da lanciare su un luogo che conquistano
employment-28 =
    Dalle porte del C-5 (due stick) o dalla rampa del CH-47 (uno stick) sopra la zona di lancio
    Lancio tra 500 e 1500 ft dal suolo, sotto 150 kt; la maggior parte dello stick deve atterrare nella zona
weapon-role-29 = Pod di disturbo tattico: riduce i radar disturbati alla loro portata di bruciatura
employment-29 =
    Impostato sulla pagina JAM: un radar o un settore per pod, acceso o spento
    Disturbare ti rende visibile da più lontano
weapon-role-30 = Dispenser d'attacco agli aeroporti: semina bombette da cratere e mine anticarro con paracadute su una striscia di 500 m
employment-30 =
    Resta sul pilone e rilascia per due secondi; vola livellato e basso sull'obiettivo
    Da non più di 1000 ft dal suolo; tre crateri chiudono una pista, le mine aspettano i veicoli
