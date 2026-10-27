# The main menu and the single mission screen (src/menu.rs).

## Theaters (stealth_host::theater::THEATERS)

theater-libya = Libya (1986)
    .facts =
        Gulf of Sidra and the Libyan coast
        Era of Operation El Dorado Canyon
        Clear desert skies most days
theater-persian_gulf = Persian Gulf (1984)
    .facts =
        Iran-Iraq Tanker War era
        Over water and coastal desert
        Clearest weather of all theaters
theater-north_cape = North Cape
    .facts =
        Arctic Norway and the Kola Peninsula
        Home of the Soviet Northern Fleet
        Frequent low overcast
theater-central_europe = Central Europe
    .facts =
        NATO vs Warsaw Pact over Germany
        Clear skies only 15% of the time
theater-vietnam = Vietnam
    .facts =
        Hanoi and Haiphong region
        Jungle and mountain terrain
        Thunderstorms in 30% of missions
theater-middle_east = Middle East
    .facts =
        Eastern Mediterranean and the Levant
        Mostly clear skies
theater-desert_storm = Desert Storm
    .facts =
        Kuwait and Iraq, 1991
        The F-117A's real combat debut
        Mostly clear desert skies
theater-cuba = Cuba
    .facts =
        Caribbean island theater
        Sea and land targets
        Tropical: storms in 15% of missions
theater-korea = Korea
    .facts =
        Korean peninsula north of the DMZ
        Mountainous terrain
        Changeable weather, any sky is likely
theater-ukraine = Ukraine (2026)
    .facts =
        Eastern Ukraine, Crimea and southern Russia
        Front line from Kharkiv to Kherson burns
        Open farmland, ruined towns at the front

## Situations (rules of engagement)

roe-0 = Cold War
    .facts =
        Covert peacetime operation
        Staying undetected matters most
        Fewer enemy aircraft on patrol
        Exact rules of engagement are in the briefing
roe-1 = Limited War
    .facts =
        Regional shooting conflict
        Enemy air defenses on alert
        More enemy aircraft than in Cold War
roe-2 = Conventional War
    .facts =
        Full-scale war
        All enemy forces active
        More enemy aircraft than in Cold War

## Mission kinds

kind-strike = Strike
    .facts =
        Destroy, photograph or reach a ground target
        Primary and secondary objective
        Air-to-ground loadout
kind-air = Air to air
    .facts =
        Intercept an enemy aircraft
        Transports, bombers or fighter patrols
        Air-to-air missile loadout
kind-strike-training = Strike training
    .facts =
        Practice strike mission
        Learn weapons and navigation
kind-air-training = Air training
    .facts =
        Practice air-to-air mission
        Learn intercepts and missiles
kind-drone-defense = Drone defense
    .facts =
        Ukraine only
        Stop a wave of Geran attack drones before it hits a city or a nuclear plant
        Then bomb their launch site

## Opponents

opponents-0 = Green
    .facts =
        Inexperienced enemy crews
        Easiest setting
opponents-1 = Regular
    .facts =
        Average enemy crews
        Default setting
opponents-2 = Veteran
    .facts =
        Skilled enemy crews
        More ships and patrols
opponents-3 = Elite
    .facts =
        Best enemy pilots and SAM crews
        Hardest setting

## Weather

sky-random = Random
    .facts =
        Rolled from the theater's climate
        Changes with each new mission
        Clouds block line of sight for both sides
sky-clear = Clear
    .facts =
        No clouds
        Visibility 40 km
        The classic DOS era sky
sky-scattered = Scattered
    .facts =
        Cumulus 20-45%, base 2500-6000 ft
        Sometimes a thin altostratus high above
        Visibility 30 km
sky-broken = Broken
    .facts =
        Cumulus 45-70%, base 2000-5000 ft
        Often a stratocumulus layer above
        Visibility 25 km
sky-overcast = Overcast
    .facts =
        Stratus 80-100%, base 1500-5000 ft
        A few cumulus fragments below
        Visibility 20 km
sky-storm = Storm
    .facts =
        Cumulonimbus towers up to 40000 ft
        Low stratus and cumulus from 1000 ft
        Visibility 8 km
menu-random-cover = Random cover
    .facts =
        Main cloud layer cover rolled with the weather
        Uncheck to set it yourself
        Main layer: stratus for overcast and storm, cumulus otherwise
menu-cover-slider =
    .facts =
        Share of the sky the main layer covers
        No effect on a clear sky

## Moon

moon-date = Mission date
    .facts =
        The real moon of the briefing date
        Needs the time of day switch (modern)
moon-0 = New moon
    .facts =
        No moonlight: a black night, lights and stars only
        Enemy pilots see you at 2 km
moon-1 = Waxing crescent
    .facts = A thin moon in the evening sky, set before midnight
moon-2 = First quarter
    .facts = Half lit, up in the evening, sets around midnight
moon-3 = Waxing gibbous
    .facts = Bright, up from the afternoon until after midnight
moon-4 = Full moon
    .facts =
        Up all night, high at midnight
        Enemy pilots see you at 4 km
moon-5 = Waning gibbous
    .facts = Bright, rises in the evening, up until the morning
moon-6 = Last quarter
    .facts = Half lit, rises around midnight
moon-7 = Waning crescent
    .facts = A thin moon rising before dawn

## Aircraft

aircraft-f117 =
    .facts =
        Stealth attack aircraft
        Subsonic, 7 g
        Modern and classic cockpit
aircraft-f16 =
    .facts =
        Multirole fighter
        Mach 2 class, 9 g, quick roll
        F-16 cockpit with RWR
aircraft-tornado =
    .facts =
        Low-level strike aircraft
        Swing wings sweep with speed
        Modern cockpit, three MFDs
aircraft-a10 =
    .facts =
        Close air support aircraft
        Slow and rugged, 30 mm cannon
        A-10C cockpit, two MFDs
aircraft-uh60 =
    .facts =
        Utility helicopter
        Inserts troops: land near the target
        Cyclic and collective
aircraft-ah1z =
    .facts =
        Attack helicopter
        Turret gun, Hellfires, AIM-9
        Gun and Hellfires follow the pod

## Sections and groups

section-theater = Theater
section-opponents = Opponents
section-situation = Situation
section-mission = Mission
section-moon = Moon
section-start-time = Start time
section-weather = Weather
section-realism = Realism
section-tacview = Tacview
section-mission-summary = Mission summary
section-briefing = Briefing
group-mission = Mission
group-conditions = Conditions
group-vehicle = Vehicle
group-simulation = Simulation

## Options of the mission

menu-start-random = Random
    .facts =
        Rolled with each new mission, as in classic mode
        06:00-08:00 by day or 18:00-20:00 by night
        Uncheck to choose the hour
menu-start-slider =
    .facts =
        Takeoff time
        Night from 18:00 to 06:00
        Kept by NEW MISSION
menu-realistic-weapons = Realistic weapons
    .facts =
        The real aircraft's limits
        2 weapon bays instead of 4
        No AIM-9 or AIM-120 missiles
        No carrier takeoffs or landings
menu-crash-protection = Crash protection
    .facts =
        Pulls up automatically near the ground (gear up)
        Ground contact and collisions are not fatal
        Off: the ground can always be hit
menu-tacview-export = Export
    .facts =
        Saves each flight as a Tacview recording (.acmi)
        In the game's folder for your files, under tacview
menu-tacview-real-time = Real-time telemetry
    .facts =
        Sends the flight to Tacview while you fly
        In Tacview: Record > Real-Time Telemetry
        Address 127.0.0.1, port 42674 (--tacview-address)
switch-game = GAME
switch-real = REAL
menu-damage = Damage
    .facts =
        Weapon damage
        Game: classic damage; hits add up but never bring the aircraft down
        Realistic: a missile hit can destroy the aircraft (eject!), damages more systems and often starts a fire, which burns out or blows the aircraft up
        Realistic: damaged control surfaces put the aircraft out of trim or make it oscillate
menu-guidance = Guidance
    .facts =
        Missile guidance
        Classic: missiles flying straight at their target
        Realistic: proportional navigation to the collision point; motors burn out and the missiles glide on, losing speed with every turn (smoke only while the motor burns)
        Realistic: SAMs loft against distant targets; the AIM-120 flies straight for 3 s (pull up to lob it)
menu-air-defence = Air defence
    .facts =
        Low-level air defences
        Classic: only the SAM sites and fighters
        Realistic: radar-laid ZSU-23-4 guns (their radar sees you like a SAM's) and S-60 flak batteries guard the airfields and the targets; MANPADS teams in the towns fire SA-14s at what they see
        Flying low over defended places becomes dangerous; bombs and guns destroy the guns

## Main screen

menu-title = STEALTH MISSION 89
menu-subtitle = MISSION PLANNING SYSTEM
menu-single-mission = SINGLE MISSION
    .facts =
        One mission in the theater of your choice
        Choose the options, plan the route and the loadout
menu-career = CAREER
    .facts =
        Pilot records: rank, decorations, sorties
        Tour of duty: 12 sorties in one theater
        What you destroy stays destroyed
menu-campaign = DYNAMIC CAMPAIGN
    .facts =
        Dynamic campaign in real time
        Take the enemy's cities and airfields
        Missions from the live situation
menu-multiplayer = MULTIPLAYER
    .facts = Coop in the dynamic campaign
    .join = Join a server on the public list
    .host = Join a server, or host a campaign and fly in it
menu-options = OPTIONS
    .facts =
        Game: cockpit, HUD and avionics style, auto trim
        Graphics, sound volumes, key and joystick bindings
        Saved for the next flights
    .short = Game, graphics, sound and controls
menu-download = DOWNLOAD
    .facts =
        The desktop game for Windows, Linux and macOS
        With VR and multiplayer
        Steam Deck and Steam Frame: the Linux AppImage
web-play = PLAY IN BROWSER
    .facts = Right here, nothing to install
web-download-platforms = Windows, Linux and macOS, Steam Deck and Steam Frame
web-download-version = The download version
web-gives-vr = VR
    .why = Fly in VR on any OpenXR-compatible headset and the Steam Frame
web-gives-hotas = HOTAS
    .why = Full support for gamepads, joysticks, throttles and pedals
web-gives-multiplayer = Better multiplayer
    .why = Direct UDP connection to servers for less lag
web-gives-server = Host your own server
    .why = Host a coop campaign, or run a dedicated server
web-gives-head-tracking = Head tracking
    .why = TrackIR-style trackers through OpenTrack
web-gives-tacview = Tacview
    .why = Flight recordings and real-time telemetry
web-gives-keep = Yours to keep
    .why = If the website goes down you still have your own copy
menu-discord = DISCORD
    .facts =
        Fly together, report bugs and share missions
        Follow development
menu-quit = QUIT
    .facts = Exit to the desktop
menu-version = VERSION { $version }

## Single mission screen

menu-career-sortie = CAREER SORTIE
menu-tour-sortie = { $pilot }  //  TOUR SORTIE { $sortie } OF { $length }
menu-msn = MISSION PLANNING SYSTEM  //  MSN { $seed }
menu-no-targets = No targets left standing for this tour
menu-plan-route = PLAN ROUTE
    .facts =
        The theater map with the enemy radar and SAM coverage
        Add, move and remove navigation points
        Targets and the landing base stay fixed
menu-loadout = LOADOUT
    .facts =
        Choose the stores on each bay or pylon
        Starts with the briefed weapons
menu-generating = GENERATING...
menu-back-records = Back to the pilot records
menu-back-main = Back to the main menu
menu-new-sortie = NEW SORTIE
menu-new-mission = NEW MISSION
    .facts =
        Rolls different objectives
        Keeps all the options and the start time
menu-load = LOAD
    .facts =
        Missions saved in flight (menu: SAVE MISSION)
        Fly on from where they were saved
menu-fly = FLY
    .facts = Take off on the briefed mission
menu-fly-vr = FLY IN VR
    .facts =
        Restarts the game in the VR headset
        Flies this exact mission
    .no-mission = Generate a mission first
    .no-runtime = No OpenXR runtime (Monado, WiVRn, SteamVR) is active
menu-vr-failed = could not start in VR: { $error }

## Mission summary

summary-theater = THEATER
summary-situation = SITUATION
summary-mission = MISSION
summary-opponents = OPPONENTS
summary-time = TIME
summary-weather = WEATHER
summary-fuel = FUEL
summary-night = Night
summary-day = Day
summary-time-value = { $daytime }  dep { $depart }
summary-weather-value = { $sky }  vis { $vis } km
summary-fuel-value = { $fuel } lbs
summary-armament = Armament: { $stores }
summary-intelligence = INTELLIGENCE
