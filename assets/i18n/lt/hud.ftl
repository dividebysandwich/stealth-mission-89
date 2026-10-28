# Žaidimo pranešimai HUD ir skrydžio 2D skydeliuose (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). HUD juos piešia didžiosiomis raidėmis be diakritikų;
# avionikos žodžiai (CCIP, THR, KM, MAX...) lieka angliški.

hud-no-ejection-seat = NĖRA KATAPULTOS
hud-waypoint-reached = Waypoint { $waypoint } pasiektas
hud-select-camera = PASIRINKTI KAMERĄ
hud-open-bay = ATIDARYTI SKYRIŲ
hud-camera-page = { $page } PUSLAPIS
hud-target-in-view-shoot = TAIKINYS MATOMAS - FOTOGRAFUOKITE
hud-target-in-view = TAIKINYS MATOMAS
hud-aim-at-target = TAIKYKITĖS Į TAIKINĮ
hud-photo = NUOTRAUKA { $state }
hud-rearmed = Ginklai ir kuras papildyti
hud-landed-rearm = Nusileista - papildykite arba baikite skrydį
hud-objective-achieved = Užduotis įvykdyta

## Misijos pabaigos užsklanda

end-accomplished = įvykdyta
end-not-accomplished = neįvykdyta
end-primary = Pagrindinė užduotis: { $result }
end-secondary = Antrinė užduotis: { $result }
end-kills = Pergalės: orlaivių { $air }, žemės taikinių { $ground }
end-time = Misijos trukmė: { $time }
end-damage = Pažeidimai: pataikymų { $hits }
end-training = Mokomoji misija
end-press-any-key = Paspauskite bet kurį klavišą

## Numušto piloto skydelis

csar-flare = SIGNALINĖ RAKETA
csar-faster = GREITIAU
csar-normal = ĮPRASTAS LAIKAS
csar-skip = PRALEISTI EVAKUACIJĄ
csar-parachute = PARAŠIUTAS ATSIVĖRĖ
csar-on-ground = ANT ŽEMĖS
csar-down = NUMUŠTAS PRIEŠO TERITORIJOJE
csar-pickup = GELBĖJIMAS   UH-60 VIRŠ JŪSŲ - PAIMA
csar-inbound = GELBĖJIMAS   UH-60 { $km } KM { $direction } - ARTĖJA
csar-fire-flare = GELBĖJIMAS   UH-60 { $km } KM { $direction } - ARTI PALEISKITE SIGNALINĘ RAKETĄ
csar-enemy = PRIEŠAS    HIND: { $count }, ARTIMIAUSIAS { $km } KM
csar-flares = RAKETOS   { $count }      ANT ŽEMĖS { $time }

## Bendro žaidimo pranešimai skydelyje (src/player_notices.rs), priešlėktuvinės gynybos pulto pastaba

notice-joined = { $pilot } prisijungė
notice-left = { $pilot } išėjo
notice-lost-connection = { $pilot } prarado ryšį
st-select-track = Pirmiausia pasirinkite takelį
