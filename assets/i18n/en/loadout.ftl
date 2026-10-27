# The loadout editor (src/loadout.rs). Weapon names are the real ones, untranslated.

loadout-none = None
loadout-briefed = BRIEFED
    .facts = The stores the mission was briefed with
loadout-clear = CLEAR
    .facts = Every hardpoint empty
loadout-done = DONE
loadout-no-hardpoints = No hardpoints
loadout-no-stores = This aircraft carries no stores
loadout-fantasy = Not on the real aircraft
loadout-empty = Empty
loadout-out-of-stock = None in stock for this mission
loadout-mirror = Same on both sides
    .facts = Loads { $station } too
loadout-stores = Stores
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
