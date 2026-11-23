# The loadout editor (src/loadout.rs). Weapon names are the real ones, untranslated.

loadout-none = None
loadout-briefed = BRIEFED
    .facts = The stores the mission was briefed with
loadout-own = Your own stores: they come again on the next missions of this kind with this aircraft. { loadout-briefed } gives back the mission's.
loadout-clear = CLEAR
    .facts = Every hardpoint empty
loadout-done = DONE
loadout-no-hardpoints = No hardpoints
loadout-no-stores = This aircraft carries no stores
loadout-fantasy = Not on the real aircraft
loadout-empty = Empty
loadout-out-of-stock = None in stock for this mission
loadout-none-in-stock = (none in stock)
loadout-empties = Empties station { $stations } when loaded
loadout-empties-two = Empties stations { $first } and { $second } when loaded
loadout-mirror = Same on both sides
    .facts = Loads { $station } too
loadout-stores = Stores
loadout-realistic-f117 = Realistic F-117A loadout
loadout-stations = Stations
loadout-click-drawing = Or click a station on the drawing
loadout-needs-equipment = The mission needs its camera or special equipment

## What each weapon is for (weapon types 0 to 20)

# AIM-9M Sidewinder
weapon-role-0 = Air-to-air, infrared homing, short range
# AIM-120A AMRAAM
weapon-role-1 = Air-to-air, radar homing, medium range
# AGM-88A HARM
weapon-role-2 = Anti-radiation missile: radars and SAM sites
# Penguin 3 ASM
weapon-role-3 = Anti-ship missile
# AGM-84A Harpoon
weapon-role-4 = Long range anti-ship missile
# AGM-65 Maverick
weapon-role-5 = TV guided missile: vehicles, bunkers, buildings
# GBU-12 Paveway
weapon-role-6 = Laser guided bomb: hard targets
# Mk 20 Rockeye
weapon-role-7 = Cluster bomb: soft and area targets
# Durandal
weapon-role-8 = Runway cratering bomb
# Mk 82 LDGP
weapon-role-9 = 500 lb free-fall bomb
# Mk 82 Snakeye
weapon-role-10 = 500 lb retarded bomb for low releases
# Mk 20 Rockeye II
weapon-role-11 = Cluster bomb: armour and area targets
# Mk 122 Fireye
weapon-role-12 = Incendiary bomb
# CBU-72 FAE
weapon-role-13 = Fuel-air explosive: area targets
# Mk 35 incendiary
weapon-role-14 = Incendiary cluster bomb
# ISCB-1 mines
weapon-role-15 = Area denial mines
# IR/optical camera
weapon-role-16 = Reconnaissance photos of the target
# 1900 lb fuel tank
weapon-role-17 = Extra fuel, transferred when released
# Special cargo
weapon-role-18 = Supplies or cargo for the mission
# 20 mm cannon
weapon-role-19 = The gun (not a store)
# AGM-114 Hellfire
weapon-role-20 = Laser guided anti-armour missile: flies to where the targeting pod looks

## How each weapon is used (ranges in the HUD's km, heights in feet)

employment-0 = Range: 1-12 km, farther head-on
employment-1 = Range: 1-38 km, farther head-on
employment-2 =
    Range: 9-50 km
    Launch: 5,000-20,000 ft
employment-3 =
    Range: 6-14 km (classic guidance: 13-14 km)
    Launch: 100-500 ft
employment-4 =
    Range: 6-41 km
    Launch: 2,000-5,000 ft
employment-5 =
    Range: 5-10 km
    Launch: 2,000-5,000 ft
employment-6 =
    Range: 1.5 km to 2.5 times the height, LOCK steady (14 km from 15,000 ft, 18 km from 25,000 ft)
    Release: 15,000-25,000 ft
employment-8 =
    Range: 0.9-1.3 km short of the runway
    Release: 200-500 ft
employment-9 =
    Range: 5-11 km short of the target
    Release: 3,000-20,000 ft
employment-12 =
    Range: 0.9-2.2 km short of the target
    Release: 100-500 ft
employment-16 = Range: photos from 2-8 km (the shorter, the more opponents), nose on the target
employment-18 = Range: release on "Drop supplies now", within 4 km of the drop zone
employment-20 =
    Range: 4-8 km
    Launch: 2,000-3,000 ft
employment-guided-cluster =
    Range: 1.5 km to 2.5 times the height, LOCK steady (3.7 km from 3,000 ft, 6 km from 5,000 ft)
    Release: 3,000-5,000 ft
employment-medium-retarded =
    Range: 1.1-1.6 km short of the target
    Release: 300-1,000 ft
employment-low-retarded =
    Range: 0.9-1.3 km short of the target
    Release: 200-500 ft

## Hardpoints

hardpoint-left-bay = Left bay
hardpoint-right-bay = Right bay
hardpoint-left-wing-bay = Left wing bay
hardpoint-right-wing-bay = Right wing bay
hardpoint-station-1-left-wingtip = Station 1, left wingtip
hardpoint-station-2-left-outer-wing = Station 2, left outer wing
hardpoint-station-3-left-wing = Station 3, left wing
hardpoint-station-4-left-inner-wing = Station 4, left inner wing
hardpoint-station-5-centreline = Station 5, centreline
hardpoint-station-6-right-inner-wing = Station 6, right inner wing
hardpoint-station-7-right-wing = Station 7, right wing
hardpoint-station-8-right-outer-wing = Station 8, right outer wing
hardpoint-station-9-right-wingtip = Station 9, right wingtip
hardpoint-station-1-left-wingtip-rail = Station 1, left wingtip rail
hardpoint-station-2-left-outer-pylon = Station 2, left outer pylon
hardpoint-station-3-left-inner-pylon = Station 3, left inner pylon
hardpoint-station-4-right-inner-pylon = Station 4, right inner pylon
hardpoint-station-5-right-outer-pylon = Station 5, right outer pylon
hardpoint-station-6-right-wingtip-rail = Station 6, right wingtip rail
hardpoint-tornado-left-wing-rail = Left wing, Sidewinder rail
hardpoint-tornado-left-wing-pylon = Left wing pylon
hardpoint-tornado-left-shoulder-forward = Left shoulder, forward
hardpoint-tornado-left-shoulder-aft = Left shoulder, aft
hardpoint-tornado-right-shoulder-aft = Right shoulder, aft
hardpoint-tornado-right-shoulder-forward = Right shoulder, forward
hardpoint-tornado-right-wing-pylon = Right wing pylon
hardpoint-tornado-right-wing-rail = Right wing, Sidewinder rail
hardpoint-oh58-left-pylon = Left weapons pylon
hardpoint-oh58-right-pylon = Right weapons pylon
hardpoint-mh60r-left-outer = Left outer station (Hellfire launcher)
hardpoint-mh60r-left-inner = Left inner station
hardpoint-mh60r-right-inner = Right inner station
hardpoint-a10-1-left-outer-wing = Station 1, left outer wing
hardpoint-a10-2-left-wing = Station 2, left wing
hardpoint-a10-3-left-wing = Station 3, left wing
hardpoint-a10-4-left-centre-wing = Station 4, left centre wing
hardpoint-a10-5-left-fuselage = Station 5, left fuselage
hardpoint-a10-6-centreline = Station 6, centreline
hardpoint-a10-7-right-fuselage = Station 7, right fuselage
hardpoint-a10-8-right-centre-wing = Station 8, right centre wing
hardpoint-a10-9-right-wing = Station 9, right wing
hardpoint-a10-10-right-wing = Station 10, right wing
hardpoint-a10-11-right-outer-wing = Station 11, right outer wing
hardpoint-av8b-1-left-rail = Station 1, left Sidewinder rail
hardpoint-av8b-3-left-middle-wing = Station 3, left intermediate wing
hardpoint-av8b-4-left-inboard-wing = Station 4, left inboard wing
hardpoint-av8b-6-right-inboard-wing = Station 6, right inboard wing
hardpoint-av8b-7-right-middle-wing = Station 7, right intermediate wing
hardpoint-av8b-9-right-rail = Station 9, right Sidewinder rail
hardpoint-f22-left-side-bay = Left side bay (Sidewinder)
hardpoint-f22-left-main-bay = Main bay, left half
hardpoint-f22-right-main-bay = Main bay, right half
hardpoint-f22-right-side-bay = Right side bay (Sidewinder)
hardpoint-fa18e-8-right-inner-wing = Station 8, right inner wing
hardpoint-fa18e-10-right-outer-wing = Station 10, right outer wing
hardpoint-fa18e-11-right-wingtip = Station 11, right wingtip
hardpoint-a6-1-left-outer-wing = Station 1, left outer wing
hardpoint-a6-2-left-inner-wing = Station 2, left inner wing
hardpoint-a6-4-right-inner-wing = Station 4, right inner wing
hardpoint-a6-5-right-outer-wing = Station 5, right outer wing
hardpoint-c5-hold = Cargo hold (airdrop pallets)
hardpoint-ch47-cabin = Cabin (paratroopers)
weapon-role-21 = Air-to-air, infrared homing, very short range
employment-21 = Range: 1-6 km, farther head-on
weapon-role-22 = Machine gun pod: fired as the gun, along the nose
employment-22 = Range: up to 1.5 km, 500 rounds a pod
weapon-role-23 = Sensor fuzed weapon: forty skeets fire down at the vehicles under its pattern
employment-23 =
    Guided to the point marked with the targeting pod at release
    Release: above the burst altitude set on the SMS page (500-3,000 ft)
weapon-role-24 = Lightweight torpedo: drops on a chute, searches under water and homes on a submarine
employment-24 =
    Programmed on the sensor operator's WPN page: heading, enable delay, search pattern
    Release: over water, below 300 ft and 100 kt
weapon-role-25 = Stand-off land attack missile: cruises to its target, then the pilot steers it by its seeker picture
employment-25 =
    Programmed on the SLAM page: cruise altitude, seeker range, primary or secondary target
    Range: up to 250 km, four in flight at once
weapon-role-26 = Glide bomb: glides to the designated point, its reach growing with release height and speed
employment-26 =
    Aimed at the pod's point or the ground lock, else the primary target
    Range: some 20 km low, up to 130 km from high; air defences can shoot it down
weapon-role-27 = Air-launched decoy: flies an aircraft's signature into the air defences and draws their missiles
employment-27 =
    Flies to the designated point, then straight on for up to 10 minutes
    Harmless to targets
weapon-role-28 = Paratroopers: a stick of soldiers to drop on a place they take
employment-28 =
    Out of the C-5's doors (two sticks) or the CH-47's ramp (one stick) over the drop zone
    Jump between 500 and 1500 ft over the ground, below 150 kt; most of a stick must land in the zone
weapon-role-29 = Tactical jamming pod: cuts the radars it is set on down to their burn-through range
employment-29 =
    Set on the JAM page: a radar or a sector each pod, on or off
    Jamming makes you seen from further off
weapon-role-30 = Airfield attack dispenser: lays cratering bomblets and anti-armour mines on chutes along a 500 m strip
employment-30 =
    Stays on the pylon and dispenses for two seconds; fly level and low over the target
    From no higher than 1000 ft over the ground; three craters close a runway, the mines wait for vehicles
weapon-role-31 = Unguided 70 mm rockets in a pod: pairs or ripples against vehicles and troops in the open
employment-31 =
    Aim with the rocket reticle (CCIP): it shows where a rocket fired now lands
    Press for a pair, hold for a ripple; best from a shallow dive inside 4 km
weapon-role-32 = Laser-guided 70 mm rockets: each homes on the targeting pod's laser spot
employment-32 =
    Put the targeting pod on the target and fire along the nose; they turn onto the spot
    Up to 6 km; keep the pod on the target until they hit
loadout-wingman = WINGMAN
    .facts = An AI wingman in the same aircraft with the same stores joins you once you are up; the radio menu (\) gives it its orders
