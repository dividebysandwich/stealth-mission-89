# Spelets meddelanden på HUD:en och flygningens 2D-överlägg. HUD:en ritar dem i versaler utan
# accenter; avionikens egna ord (CCIP, THR, KM, MAX...) förblir engelska.

hud-no-ejection-seat = INGEN KATAPULTSTOL
hud-catapult-power = FULL KRAFT FÖR START
hud-catapult-wait = VÄNTAR PÅ KATAPULT
hud-waypoint-reached = Waypoint { $waypoint } nådd
hud-select-camera = VÄLJ KAMERA
hud-open-bay = ÖPPNA UTRYMMET
hud-camera-page = SIDA { $page }
hud-target-in-view-shoot = MÅL I SIKTE - FOTOGRAFERA
hud-target-in-view = MÅL I SIKTE
hud-aim-at-target = RIKTA MOT MÅL
hud-photo = FOTO { $state }
hud-rearmed = Ombeväpnad och tankad
hud-observing = Observerar - avsluta uppdraget från menyn
hud-landed-rearm = Landad - beväpna om eller avsluta flygningen
hud-divert-accept = Omdirigering: { $title } - { $key } för att acceptera
hud-objective-achieved = Målet uppnått

## Skärmen vid uppdragets slut

end-accomplished = genomfört
end-not-accomplished = ej genomfört
end-primary = Primärt mål { $result }
end-secondary = Sekundärt mål { $result }
end-kills = Nedskjutningar: { $air } flygplan, { $ground } markmål
end-time = Uppdragstid { $time }
end-damage = Skador: { $hits } träffar
end-training = Övningsuppdrag

## Panelen för nedskjuten pilot

csar-flare = SIGNALRAKET
csar-faster = SNABBARE
csar-normal = NORMAL TID
csar-skip = HOPPA ÖVER UPPHÄMTNING
csar-parachute = FALLSKÄRM UTLÖST
csar-on-ground = PÅ MARKEN
csar-down = NEDSKJUTEN PÅ FIENTLIGT OMRÅDE
csar-pickup = RÄDDNING   UH-60 STRAX OVANFÖR - UPPHÄMTNING
csar-inbound = RÄDDNING   UH-60 { $km } KM { $direction } - PÅ ANFLYGNING
csar-fire-flare = RÄDDNING   UH-60 { $km } KM { $direction } - SKJUT EN SIGNALRAKET NÄR DEN ÄR NÄRA
csar-enemy = FIENDE    { $count } HIND, NÄRMASTE { $km } KM
csar-flares = SIGNALRAKETER   { $count }      PÅ MARKEN { $time }

## Samarbetsmeddelanden på panelen (src/player_notices.rs), luftvärnskonsolens notering

notice-joined = { $pilot } gick med
notice-left = { $pilot } lämnade
notice-lost-connection = { $pilot } tappade anslutningen
st-select-track = Välj ett spår först
hud-auto-hover = AUTOSVÄVNING
hud-hover-too-fast = FÖR SNABBT FÖR SVÄVNING (ÖVER { $knots } KT)
csar-abort = AVBRYT
radio-wing-title = ROTEKAMRAT
radio-wing-engage = Anfall mitt mål
radio-wing-sams = Anfall SAM
radio-wing-cover = Skydda mig
radio-wing-rejoin = Anslut
radio-wing-rtb = Återvänd till basen
radio-wing-trail = I rad
radio-wing-spread = Bredvid
radio-wing-callsign = Två
