# Správy hry na HUD a v 2D prekryvoch letu (src/hud, src/combat.rs, src/csar_view.rs,
# src/campaign). HUD ich vykresľuje veľkými písmenami bez diakritiky; vlastné výrazy
# avioniky (CCIP, THR, KM, MAX...) zostávajú po anglicky.

hud-no-ejection-seat = ŽIADNE VYSTRELOVACIE SEDADLO
hud-waypoint-reached = Waypoint { $waypoint } dosiahnutý
hud-select-camera = VYBERTE KAMERU
hud-open-bay = OTVORTE ŠACHTU
hud-camera-page = STRANA { $page }
hud-target-in-view-shoot = CIEĽ V ZÁBERE - FOTO
hud-target-in-view = CIEĽ V ZÁBERE
hud-aim-at-target = NAMIERTE NA CIEĽ
hud-photo = FOTO { $state }
hud-rearmed = Prezbrojené a natankované
hud-landed-rearm = Pristáli ste - prezbrojte sa alebo ukončite bojový let
hud-objective-achieved = Cieľ splnený

## Prekryv konca misie

end-accomplished = splnený
end-not-accomplished = nesplnený
end-primary = Hlavný cieľ { $result }
end-secondary = Vedľajší cieľ { $result }
end-kills = Zostrely: { $air } lietadiel, { $ground } pozemných cieľov
end-time = Čas misie { $time }
end-damage = Poškodenie: { $hits } zásahov
end-training = Cvičná misia
end-press-any-key = Stlačte ľubovoľný kláves

## Panel zostreleného pilota

csar-flare = SIGNÁLNA RAKETA
csar-faster = RÝCHLEJŠIE
csar-normal = NORMÁLNY ČAS
csar-skip = PRESKOČIŤ EVAKUÁCIU
csar-parachute = PADÁK OTVORENÝ
csar-on-ground = NA ZEMI
csar-down = NA ZEMI V NEPRIATEĽSKOM ÚZEMÍ
csar-pickup = ZÁCHRANA   UH-60 NAD VAMI - VYZDVIHNUTIE
csar-inbound = ZÁCHRANA   UH-60 { $km } KM { $direction } - LETÍ K VÁM
csar-fire-flare = ZÁCHRANA   UH-60 { $km } KM { $direction } - AK BUDE BLÍZKO, VYPÁĽTE SIGNÁLNU RAKETU
csar-enemy = NEPRIATEĽ    { $count } HIND, NAJBLIŽŠÍ { $km } KM
csar-flares = SIGNÁLNE RAKETY   { $count }      NA ZEMI { $time }

## Oznámenia kooperácie na paneli (src/player_notices.rs), poznámka konzoly protivzdušnej obrany

notice-joined = { $pilot } sa pripojil
notice-left = { $pilot } odišiel
notice-lost-connection = { $pilot } stratil spojenie
st-select-track = Najprv vyberte stopu
