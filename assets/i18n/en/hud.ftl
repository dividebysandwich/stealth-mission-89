# The game's messages on the HUD and the flight's 2D overlays (src/hud, src/combat.rs,
# src/csar_view.rs, src/campaign). The HUD draws them in capitals without accents; the
# avionics' own words (CCIP, THR, KM, MAX...) stay in English.

hud-no-ejection-seat = NO EJECTION SEAT
hud-waypoint-reached = Waypoint { $waypoint } reached
hud-select-camera = SELECT CAMERA
hud-open-bay = OPEN BAY
hud-camera-page = { $page } PAGE
hud-target-in-view-shoot = TARGET IN VIEW - SHOOT
hud-target-in-view = TARGET IN VIEW
hud-aim-at-target = AIM AT TGT
hud-photo = PHOTO { $state }
hud-rearmed = Rearmed and refuelled
hud-landed-rearm = Landed - rearm or end the sortie
hud-objective-achieved = Objective achieved

## End of mission overlay

end-accomplished = accomplished
end-not-accomplished = not accomplished
end-primary = Primary objective { $result }
end-secondary = Secondary objective { $result }
end-kills = Kills: { $air } aircraft, { $ground } ground targets
end-time = Mission time { $time }
end-damage = Damage: { $hits } hits
end-training = Training mission
end-press-any-key = Press any key

## The downed pilot's panel

csar-flare = SIGNAL FLARE
csar-faster = FASTER
csar-normal = NORMAL TIME
csar-skip = SKIP EXTRACTION
csar-parachute = PARACHUTE DEPLOYED
csar-on-ground = ON THE GROUND
csar-down = DOWN IN ENEMY TERRITORY
csar-pickup = RESCUE   UH-60 OVERHEAD - PICKUP
csar-inbound = RESCUE   UH-60 { $km } KM { $direction } - INBOUND
csar-fire-flare = RESCUE   UH-60 { $km } KM { $direction } - FIRE A FLARE WHEN CLOSE
csar-enemy = ENEMY    { $count } HIND, NEAREST { $km } KM
csar-flares = FLARES   { $count }      ON THE GROUND { $time }

## Coop notices on the panel (src/player_notices.rs), the air defence console's note

notice-joined = { $pilot } joined
notice-left = { $pilot } left
notice-lost-connection = { $pilot } lost connection
st-select-track = Select a track first
