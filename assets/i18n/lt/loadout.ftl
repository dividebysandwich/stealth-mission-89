# Ginkluotės redaktorius (src/loadout.rs). Ginklų pavadinimai tikrieji, neversti.

loadout-none = Nėra
loadout-briefed = INSTRUKTAŽAS
    .facts = Krovinys, su kuriuo misija buvo suplanuota instruktaže
loadout-clear = IŠVALYTI
    .facts = Visi pakabos taškai tušti
loadout-done = GERAI
loadout-no-hardpoints = Nėra pakabos taškų
loadout-no-stores = Šis orlaivis nenešioja krovinio
loadout-fantasy = Tikrame orlaivyje jo nėra
loadout-empty = Tuščia
loadout-out-of-stock = Šiai misijai atsargų nėra
loadout-mirror = Vienodai abiejose pusėse
    .facts = Pakrauna ir { $station }
loadout-stores = Krovinys
loadout-realistic-f117 = Realistiška F-117A ginkluotė
loadout-stations = Pakabinimo taškai
loadout-click-drawing = Arba spustelėkite tašką brėžinyje
loadout-needs-equipment = Misijai reikia jos kameros arba specialios įrangos

## Kam skirtas kiekvienas ginklas (ginklų tipai nuo 0 iki 20)

# AIM-9M Sidewinder
weapon-role-0 = Oras-oras, infraraudonųjų spindulių vedimas, mažas nuotolis
# AIM-120A AMRAAM
weapon-role-1 = Oras-oras, radaro vedimas, vidutinis nuotolis
# AGM-88A HARM
weapon-role-2 = Priešradarinė raketa: radarai ir SAM objektai
# Penguin 3 ASM
weapon-role-3 = Priešlaivinė raketa
# AGM-84A Harpoon
weapon-role-4 = Tolimojo nuotolio priešlaivinė raketa
# AGM-65 Maverick
weapon-role-5 = TV vedama raketa: transporto priemonės, bunkeriai, pastatai
# GBU-12 Paveway
weapon-role-6 = Lazeriu vedama bomba: sutvirtinti taikiniai
# Mk 20 Rockeye
weapon-role-7 = Kasetinė bomba: minkšti ir ploto taikiniai
# Durandal
weapon-role-8 = Pakilimo tako kraterius darančioji bomba
# Mk 82 LDGP
weapon-role-9 = 500 lb laisvai krintanti bomba
# Mk 82 Snakeye
weapon-role-10 = 500 lb stabdoma bomba žemam numetimui
# Mk 20 Rockeye II
weapon-role-11 = Kasetinė bomba: šarvuotiems ir ploto taikiniams
# Mk 122 Fireye
weapon-role-12 = Padegamoji bomba
# CBU-72 FAE
weapon-role-13 = Kuro ir oro mišinio sprogmuo: ploto taikiniai
# Mk 35 incendiary
weapon-role-14 = Padegamoji kasetinė bomba
# ISCB-1 mines
weapon-role-15 = Ploto uždraudimo minos
# IR/optical camera
weapon-role-16 = Taikinio žvalgybinės nuotraukos
# 1900 lb fuel tank
weapon-role-17 = Papildomas kuras, perpumpuojamas numetus
# Special cargo
weapon-role-18 = Misijos reikmenys ar krovinys
# 20 mm cannon
weapon-role-19 = Patranka (ne krovinys)
# AGM-114 Hellfire
weapon-role-20 = Lazeriu vedama prieššarvinė raketa: skrenda ten, kur žiūri žymėjimo konteineris

## Kaip naudojamas kiekvienas ginklas (nuotoliai HUD km, aukščiai pėdomis)

employment-0 = Nuotolis: 1-12 km, toliau iš priekio
employment-1 = Nuotolis: 1-38 km, toliau iš priekio
employment-2 =
    Nuotolis: 9-50 km
    Paleidimas: 5 000-20 000 ft
employment-3 =
    Nuotolis: 6-14 km (klasikinis vedimas: 13-14 km)
    Paleidimas: 100-500 ft
employment-4 =
    Nuotolis: 6-41 km
    Paleidimas: 2 000-5 000 ft
employment-5 =
    Nuotolis: 5-10 km
    Paleidimas: 2 000-5 000 ft
employment-6 =
    Nuotolis: nuo 1,5 km iki 2,5 aukščio, LOCK pastovus (14 km iš 15 000 ft, 18 km iš 25 000 ft)
    Numetimas: 15 000-25 000 ft
employment-8 =
    Nuotolis: 0,9-1,3 km prieš pakilimo taką
    Numetimas: 200-500 ft
employment-9 =
    Nuotolis: 5-11 km prieš taikinį
    Numetimas: 3 000-20 000 ft
employment-12 =
    Nuotolis: 0,9-2,2 km prieš taikinį
    Numetimas: 100-500 ft
employment-16 = Nuotolis: nuotraukos iš 2-8 km (kuo arčiau, tuo daugiau priešininkų), nosis į taikinį
employment-18 = Nuotolis: numeskite paskelbus "Drop supplies now", ne toliau kaip 4 km nuo numetimo zonos
employment-20 =
    Nuotolis: 4-8 km
    Paleidimas: 2 000-3 000 ft
employment-guided-cluster =
    Nuotolis: nuo 1,5 km iki 2,5 aukščio, LOCK pastovus (3,7 km iš 3 000 ft, 6 km iš 5 000 ft)
    Numetimas: 3 000-5 000 ft
employment-medium-retarded =
    Nuotolis: 1,1-1,6 km prieš taikinį
    Numetimas: 300-1 000 ft
employment-low-retarded =
    Nuotolis: 0,9-1,3 km prieš taikinį
    Numetimas: 200-500 ft

## Pakabos taškai

hardpoint-left-bay = Kairysis skyrius
hardpoint-right-bay = Dešinysis skyrius
hardpoint-left-wing-bay = Kairiojo sparno skyrius
hardpoint-right-wing-bay = Dešiniojo sparno skyrius
hardpoint-station-1-left-wingtip = Stotis 1, kairysis sparno galas
hardpoint-station-2-left-outer-wing = Stotis 2, kairysis išorinis sparnas
hardpoint-station-3-left-wing = Stotis 3, kairysis sparnas
hardpoint-station-4-left-inner-wing = Stotis 4, kairysis vidinis sparnas
hardpoint-station-5-centreline = Stotis 5, centrinė linija
hardpoint-station-6-right-inner-wing = Stotis 6, dešinysis vidinis sparnas
hardpoint-station-7-right-wing = Stotis 7, dešinysis sparnas
hardpoint-station-8-right-outer-wing = Stotis 8, dešinysis išorinis sparnas
hardpoint-station-9-right-wingtip = Stotis 9, dešinysis sparno galas
hardpoint-station-1-left-wingtip-rail = Stotis 1, kairiojo sparno galo bėgelis
hardpoint-station-2-left-outer-pylon = Stotis 2, kairysis išorinis pylonas
hardpoint-station-3-left-inner-pylon = Stotis 3, kairysis vidinis pylonas
hardpoint-station-4-right-inner-pylon = Stotis 4, dešinysis vidinis pylonas
hardpoint-station-5-right-outer-pylon = Stotis 5, dešinysis išorinis pylonas
hardpoint-station-6-right-wingtip-rail = Stotis 6, dešiniojo sparno galo bėgelis
hardpoint-tornado-left-wing-rail = Kairysis sparnas, Sidewinder bėgelis
hardpoint-tornado-left-wing-pylon = Kairiojo sparno pylonas
hardpoint-tornado-left-shoulder-forward = Kairysis petys, priekyje
hardpoint-tornado-left-shoulder-aft = Kairysis petys, gale
hardpoint-tornado-right-shoulder-aft = Dešinysis petys, gale
hardpoint-tornado-right-shoulder-forward = Dešinysis petys, priekyje
hardpoint-tornado-right-wing-pylon = Dešiniojo sparno pylonas
hardpoint-tornado-right-wing-rail = Dešinysis sparnas, Sidewinder bėgelis
hardpoint-a10-1-left-outer-wing = Stotis 1, kairysis išorinis sparnas
hardpoint-a10-2-left-wing = Stotis 2, kairysis sparnas
hardpoint-a10-3-left-wing = Stotis 3, kairysis sparnas
hardpoint-a10-4-left-centre-wing = Stotis 4, kairysis vidurinis sparnas
hardpoint-a10-5-left-fuselage = Stotis 5, kairysis fiuzeliažas
hardpoint-a10-6-centreline = Stotis 6, centrinė linija
hardpoint-a10-7-right-fuselage = Stotis 7, dešinysis fiuzeliažas
hardpoint-a10-8-right-centre-wing = Stotis 8, dešinysis vidurinis sparnas
hardpoint-a10-9-right-wing = Stotis 9, dešinysis sparnas
hardpoint-a10-10-right-wing = Stotis 10, dešinysis sparnas
hardpoint-a10-11-right-outer-wing = Stotis 11, dešinysis išorinis sparnas
hardpoint-av8b-1-left-rail = Stotis 1, kairysis Sidewinder bėgelis
hardpoint-av8b-3-left-middle-wing = Stotis 3, kairysis sparnas, vidurys
hardpoint-av8b-4-left-inboard-wing = Stotis 4, kairysis sparnas, prie fiuzeliažo
hardpoint-av8b-6-right-inboard-wing = Stotis 6, dešinysis sparnas, prie fiuzeliažo
hardpoint-av8b-7-right-middle-wing = Stotis 7, dešinysis sparnas, vidurys
hardpoint-av8b-9-right-rail = Stotis 9, dešinysis Sidewinder bėgelis
hardpoint-f22-left-side-bay = Kairysis šoninis skyrius (Sidewinder)
hardpoint-f22-left-main-bay = Pagrindinis skyrius, kairioji pusė
hardpoint-f22-right-main-bay = Pagrindinis skyrius, dešinioji pusė
hardpoint-f22-right-side-bay = Dešinysis šoninis skyrius (Sidewinder)
hardpoint-fa18e-8-right-inner-wing = Stotis 8, dešinysis vidinis sparnas
hardpoint-fa18e-10-right-outer-wing = Stotis 10, dešinysis išorinis sparnas
hardpoint-fa18e-11-right-wingtip = Stotis 11, dešinysis sparno galas
hardpoint-c5-hold = Krovinių skyrius (numetami padėklai)
hardpoint-ch47-cabin = Salonas (parašiutininkai)
weapon-role-21 = Oras-oras, infraraudonųjų spindulių vedimas, labai trumpas nuotolis
employment-21 = Nuotolis: 1-6 km, didesnis iš priekio
loadout-none-in-stock = (nėra sandėlyje)
loadout-empties = Ištuština stotį { $stations } pakraunant
loadout-empties-two = Ištuština stotis { $first } ir { $second } pakraunant
hardpoint-oh58-left-pylon = Kairysis ginkluotės pilonas
hardpoint-oh58-right-pylon = Dešinysis ginkluotės pilonas
hardpoint-mh60r-left-outer = Kairysis išorinis pakabinimo taškas (Hellfire paleidimo įrenginys)
hardpoint-mh60r-left-inner = Kairysis vidinis pakabinimo taškas
hardpoint-mh60r-right-inner = Dešinysis vidinis pakabinimo taškas
weapon-role-22 = Kulkosvaidžio konteineris: šaudo kaip patranka, išilgai nosies
employment-22 = Nuotolis: iki 1,5 km, 500 šovinių konteineryje
weapon-role-23 = Jutikliniu sprogdikliu valdomas ginklas: keturiasdešimt skeet šauna į apačioje esančias mašinas
employment-23 =
    Nukreipiamas į tašką, pažymėtą taikymo konteineriu numetimo metu
    Numetimas: virš SMS puslapyje nustatyto atsidarymo aukščio (500-3 000 ft)
weapon-role-24 = Lengvoji torpeda: leidžiasi parašiutu, ieško po vandeniu ir nusitaiko į povandeninį laivą
employment-24 =
    Programuojama jutiklių operatoriaus WPN puslapyje: kursas, aktyvinimo delsa, paieškos schema
    Numetimas: virš vandens, žemiau 300 ft ir 100 kt
weapon-role-25 = Tolimo nuotolio raketa prieš antžeminius taikinius: skrenda iki taikinio, o tada pilotas ją valdo pagal ieškiklio vaizdą
employment-25 =
    Programuojama SLAM puslapyje: skrydžio aukštis, ieškiklio įjungimo nuotolis, pagrindinis ar atsarginis taikinys
    Nuotolis: iki 250 km, keturios ore vienu metu
weapon-role-26 = Sklandanti bomba: sklendžia į nurodytą tašką; nuotolis auga su numetimo aukščiu ir greičiu
employment-26 =
    Taikoma į konteinerio tašką arba žemės užrakinimą, kitaip į pagrindinį taikinį
    Nuotolis: apie 20 km žemai, iki 130 km iš aukštai; oro gynyba gali ją numušti
weapon-role-27 = Iš oro paleidžiamas masalas: skrenda su lėktuvo signatūra į oro gynybą ir pritraukia jos raketas
employment-27 =
    Skrenda į nurodytą tašką, tada tiesiai iki 10 minučių
    Taikiniams nepavojingas
weapon-role-28 = Parašiutininkai: karių grupė, išmetama virš vietovės, kurią jie užima
employment-28 =
    Pro C-5 duris (dvi grupės) arba CH-47 rampą (viena grupė) virš išmetimo zonos
    Šuolis 500–1500 ft virš žemės, lėčiau nei 150 kt; dauguma grupės turi nusileisti zonoje
weapon-role-29 = Taktinis trukdžių konteineris: sumažina trukdomų radarų nuotolį iki prasiveržimo nuotolio
employment-29 =
    Nustatomas JAM puslapyje: kiekvienam konteineriui radaras ar sektorius, įjungta ar išjungta
    Trukdymas daro tave matomą iš toliau
