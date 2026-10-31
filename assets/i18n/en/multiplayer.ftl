# The campaign window (src/campaign/menu.rs), the multiplayer window (src/multiplayer_menu.rs),
# the war map's legend and live view (src/campaign/picture.rs, unit_view.rs).

delete = DELETE
delete-campaign = DELETE THIS CAMPAIGN?
delete-save = DELETE THIS SAVE?
delete-permanent = Deletion is permanent

## Campaign window

cm-group-campaign = Campaign
cm-weather-dynamic = Dynamic
    .facts =
        The weather changes every 10-22 hours of the sky's clock
        Clouds block line of sight for both sides
cm-weather-clear = Clear sky
    .facts = No clouds for the whole campaign
cm-section-day-night = Day and night
cm-day-real = Real time
    .facts = A day lasts 24 hours
cm-day-2h = 2 h / day
    .facts = A day lasts 2 hours
cm-day-1h = 1 h / day
    .facts = A day lasts 1 hour
cm-day-30min = 30 min / day
    .facts = A day lasts 30 minutes
cm-conditions-note = Conditions apply to a new campaign
cm-rules-note = A saved campaign keeps the rules it was started with
cm-about =
    The war runs in real time while you play, also while the menu is open.
    Fly the missions on the board: recon, SEAD, CAP, CAS, anti-ship, strike, escort.
    Helicopter insertions take the enemy's cities and airfields.
    You are never killed or captured: after a sortie you are back at the board.
cm-section-new = New campaign
cm-new-line = { $theater }  -  { $situation }  -  opponents { $opponents }
cm-start = START
cm-section-continue = Continue
cm-no-saves = No saved campaigns
cm-played = { $hours } h played
cm-save-gone = The save is gone
cm-save-unreadable = The save could not be read: { $error }
cm-save-undeletable = The save could not be deleted: { $error }
cm-in-vr = IN VR
cm-delete =
    .facts = Delete this saved campaign

## War map

pic-friendly-aircraft = Friendly aircraft
pic-friendly-helicopter = Friendly helicopter
pic-enemy-fighter = Enemy fighter (on radar)
pic-sam-known = SAM site (known)
pic-guns-known = Guns / MANPADS (known)
pic-armour = Tank / IFV (enemy: in sight)
pic-patriot-reach = Patriot reach
pic-gepard-stinger = Gepard / Stinger
pic-missile-range = SAM missile range
pic-engagement-range = Engagement range
pic-reconned = Reconned
pic-you = You
pic-site = { $place } site
uv-live = { $unit } - LIVE
uv-contact-lost = CONTACT LOST
uv-hint = Drag: orbit  Wheel: zoom

## Multiplayer

mp-disconnected = Disconnected: { $why }
mp-connecting = Connecting to { $address }...
mp-joining = Connected to { $address }, joining...
mp-browser-public-only = only servers on the public list can be joined from the browser
mp-title-host = MULTIPLAYER: HOST A GAME
mp-subtitle = Coop in the dynamic campaign: fly the war together
mp-your-name = Your name
    .hint = Pilot name
mp-join-address = Join by address
    .hint = address or name[:port]
mp-connect = CONNECT
mp-server-name = Server name
mp-udp-port = UDP port
mp-address-note = Other players join with this computer's address; open the UDP port in your router to play over the internet
mp-list-public = List on the public server list (others find the game in their server list)
mp-rules = Rules for everybody
mp-time-of-day = Time of day (moonlit nights)
mp-realistic-weapons = Realistic F-117A weapons
mp-friendly-fire = Friendly fire
mp-day-night = Day and night:
mp-weather = Weather:
mp-host-new = HOST NEW
mp-host-saved = Host a saved campaign
mp-host = HOST
mp-host-game = HOST A GAME
mp-port-error = UDP port { $port }: { $error }
mp-default-server = { $pilot }'s campaign
mp-public-servers = Public servers
mp-refresh = REFRESH
mp-server-count = { $servers } servers, { $pilots } pilots
mp-list-error = The server list is not reachable: { $error }
mp-no-servers = No public servers right now
mp-col-server = SERVER
mp-col-pilots = PILOTS
mp-col-war = WAR
mp-dedicated = DEDICATED
mp-other-version = other version
mp-join = JOIN
