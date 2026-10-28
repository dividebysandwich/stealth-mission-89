# Mesajele jocului de pe HUD și din panourile 2D ale zborului (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). HUD-ul le desenează cu majuscule, fără diacritice;
# cuvintele avionicii (CCIP, THR, KM, MAX...) rămân în engleză.

hud-no-ejection-seat = FĂRĂ SCAUN EJECTABIL
hud-waypoint-reached = Waypoint { $waypoint } atins
hud-select-camera = ALEGEȚI CAMERA
hud-open-bay = DESCHIDEȚI SOUTA
hud-camera-page = PAGINA { $page }
hud-target-in-view-shoot = ȚINTĂ ÎN VIZOR - FOTOGRAFIAȚI
hud-target-in-view = ȚINTĂ ÎN VIZOR
hud-aim-at-target = ÎNDREPTAȚI SPRE ȚINTĂ
hud-photo = FOTO { $state }
hud-rearmed = Reînarmat și realimentat
hud-landed-rearm = Aterizat - reînarmați sau încheiați zborul
hud-objective-achieved = Obiectiv îndeplinit

## Ecranul de sfârșit al misiunii

end-accomplished = îndeplinit
end-not-accomplished = neîndeplinit
end-primary = Obiectiv principal { $result }
end-secondary = Obiectiv secundar { $result }
end-kills = Victorii: { $air } aeronave, { $ground } ținte la sol
end-time = Durata misiunii { $time }
end-damage = Avarii: { $hits } lovituri
end-training = Misiune de instruire
end-press-any-key = Apăsați orice tastă

## Panoul pilotului doborât

csar-flare = RACHETĂ DE SEMNALIZARE
csar-faster = MAI RAPID
csar-normal = TIMP NORMAL
csar-skip = SARIȚI PESTE EXTRACȚIE
csar-parachute = PARAȘUTĂ DESCHISĂ
csar-on-ground = PE SOL
csar-down = PE SOL ÎN TERITORIU INAMIC
csar-pickup = SALVARE   UH-60 DEASUPRA - RIDICARE
csar-inbound = SALVARE   UH-60 LA { $km } KM { $direction } - ÎN APROPIERE
csar-fire-flare = SALVARE   UH-60 LA { $km } KM { $direction } - TRAGEȚI O RACHETĂ CÂND ESTE APROAPE
csar-enemy = INAMIC    { $count } HIND, CEL MAI APROPIAT LA { $km } KM
csar-flares = RACHETE   { $count }      PE SOL DE { $time }

## Notificările cooperării pe panou, nota consolei de apărare antiaeriană

notice-joined = { $pilot } s-a alăturat
notice-left = { $pilot } a plecat
notice-lost-connection = { $pilot } a pierdut conexiunea
st-select-track = Alegeți mai întâi o pistă
