# Spillets meddelelser på HUD'en og flyvningens 2D-overlays (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). HUD'en tegner dem med store bogstaver uden accenter;
# avionikkens egne ord (CCIP, THR, KM, MAX...) forbliver på engelsk.

hud-no-ejection-seat = INTET UDSKYDNINGSSÆDE
hud-waypoint-reached = Waypoint { $waypoint } nået
hud-select-camera = VÆLG KAMERA
hud-open-bay = ÅBN BOMBERUM
hud-camera-page = SIDE { $page }
hud-target-in-view-shoot = MÅL I SYNE - FOTOGRAFÉR
hud-target-in-view = MÅL I SYNE
hud-aim-at-target = SIG PÅ MÅL
hud-photo = FOTO { $state }
hud-rearmed = Genbevæbnet og tanket op
hud-landed-rearm = Landet - genbevæbn eller afslut flyvningen
hud-objective-achieved = Mål nået

## Skærm ved missionens slutning

end-accomplished = gennemført
end-not-accomplished = ikke gennemført
end-primary = Primært mål { $result }
end-secondary = Sekundært mål { $result }
end-kills = Nedskydninger: { $air } fly, { $ground } jordmål
end-time = Missionstid { $time }
end-damage = Skade: { $hits } træffere
end-training = Træningsmission
end-press-any-key = Tryk på en vilkårlig tast

## Den nedskudte pilots panel

csar-flare = NØDBLUS
csar-faster = HURTIGERE
csar-normal = NORMAL TID
csar-skip = SPRING UDHENTNING OVER
csar-parachute = FALDSKÆRM UDFOLDET
csar-on-ground = PÅ JORDEN
csar-down = NEDE PÅ FJENDENS OMRÅDE
csar-pickup = REDNING   UH-60 OVER DIG - OPSAMLING
csar-inbound = REDNING   UH-60 { $km } KM { $direction } - INDFLYVENDE
csar-fire-flare = REDNING   UH-60 { $km } KM { $direction } - AFFYR NØDBLUS, NÅR DEN ER TÆT PÅ
csar-enemy = FJENDE    { $count } HIND, NÆRMESTE { $km } KM
csar-flares = NØDBLUS   { $count }      PÅ JORDEN { $time }

## Coop-beskeder på panelet (src/player_notices.rs), luftforsvarskonsollens note

notice-joined = { $pilot } er kommet med
notice-left = { $pilot } er gået
notice-lost-connection = { $pilot } mistede forbindelsen
st-select-track = Vælg først et spor
