# Stealth Mission 89 – User Manual

This manual describes the basic operation of the game: the main menu, the input mapping, the
cockpit, the displays, targeting, radar detection and the aircraft, air defences and ships in the
theaters. A complete list of keys is in [keys.md](keys.md).

## Contents

- [Flyable aircraft](#flyable-aircraft)
1. [Main menu](#1-main-menu)
2. [Controls menu](#2-controls-menu)
3. [The cockpit](#3-the-cockpit)
4. [MFD pages](#4-mfd-pages)
5. [Targeting and weapons](#5-targeting-and-weapons)
6. [Targeting pod and helmet sights](#6-targeting-pod-and-helmet-sights)
7. [Weapon employment](#7-weapon-employment)
8. [Radar, detection and evasion](#8-radar-detection-and-evasion)
9. [Keyboard shortcuts](#9-keyboard-shortcuts)
10. [Aircraft, air defences and ships](#10-aircraft-air-defences-and-ships)
11. [Mission planner](#11-mission-planner)
12. [Debrief and career](#12-debrief-and-career)
13. [Dynamic campaign](#13-dynamic-campaign)
14. [Drone defense (Ukraine)](#14-drone-defense-ukraine)
15. [Ejection and rescue](#15-ejection-and-rescue)
16. [Multiplayer](#16-multiplayer)
17. [Your files](#17-your-files)

---

## Flyable aircraft

Pick the aircraft in the menu's *Aircraft* section (in a campaign, on the briefing's LOADOUT
page). Each has its own cockpit.

| | Aircraft | Special capabilities |
|---|---|---|
| <img src="images/units/fly_f117.png" width="200"> | **F-117A Nighthawk** | Stealth attack aircraft: radars see it only at short range, as long as the bay doors stay shut. Four internal weapon bays (two with *Realistic weapons*), subsonic, 7 g. Modern and classic cockpit. Made for strikes and recon deep in enemy country. |
| <img src="images/units/fly_f16.png" width="200"> | **F-16C Fighting Falcon** | Multirole fighter: Mach 2 class, 9 g, rolls twice as fast as the F-117A, with an RWR and a helmet sight (chapter 6.2). Its stores hang outside, so radars see it as an F-117A with its bay doors always open (+24 EMV). SEAD with HARMs, air-to-air. |
| <img src="images/units/fly_a10.png" width="200"> | **A-10C Thunderbolt II** | Close air support: slow and rugged, a 30 mm cannon and eleven stations for Mavericks, bombs and Sidewinders. |
| <img src="images/units/fly_tornado.png" width="200"> | **Tornado IDS** | Low-level strike aircraft with swing wings that sweep back with speed. Its **terrain following radar** flies it at 65 ft over hills, buildings, trees and power lines with the autopilot engaged (key 7), still steering to the waypoint; the cockpit's E-scope shows the terrain ahead. Three MFDs. |
| <img src="images/units/fly_uh60.png" width="200"> | **UH-60 Black Hawk** | Unarmed utility helicopter for transport: it inserts troops (land near the target), rescues downed pilots (CSAR: land beside the pilot) and carries cargo on a sling (hover low over a crate to hook it, set it down by the helipad of a new air defence site). In a campaign it is the only aircraft for INSERTION, CSAR, LOGISTICS and RESUPPLY missions (chapter 13.13). |
| <img src="images/units/fly_ah1z.png" width="200"> | **AH-1Z Viper** | Attack helicopter: turret gun, 16 Hellfires and Sidewinders on the wingtips. Its helmet display (chapter 6.3) points the targeting pod, gun and Hellfires wherever you look. |
| <img src="images/units/fly_rah66.png" width="200"> | **RAH-66 Comanche** | Stealth scout helicopter: radars barely see it hovering or flying slowly, and it shows a larger signature as it speeds up. Its weapons ride inside on the bay doors (three Hellfires or six Stingers per door), which open only to fire. |

## 1. Main menu

The main menu chooses what to play; each choice replaces it with its own screen, and BACK (or Esc)
returns to it.

| Button | Function |
|---|---|
| SINGLE MISSION | One mission with the options below. |
| CAREER | Pilot records and the tour of duty, with the tour's options (chapter 12). |
| DYNAMIC CAMPAIGN | A war that runs on in real time: take missions from a board, push the front forward with helicopter insertions and defend your own places until one side holds no airfield. Start a new campaign or continue a saved one (chapter 13). |
| MULTIPLAYER | Coop in the dynamic campaign: join a server or host a campaign and fly in it (chapter 16). |
| OPTIONS | Your own settings, saved for every later game: game, graphics, sound and controls (below). |
| QUIT | Exit to the desktop. |

### Single mission

![Single mission](images/menu.png)

The single mission screen generates a mission from the selected options. Every change creates a new
mission, and the summary on the right shows it. Hovering over an option shows a short description
(in the picture, the *Cockpit* switch).

| Section | Options |
|---|---|
| **Theater** | Libya (1986), Persian Gulf (1984), North Cape, Central Europe, Vietnam, Middle East, Desert Storm, Cuba, Korea, Ukraine (2026). |
| **Opponents** | Green, Regular, Veteran, Elite. Sets the skill of enemy pilots and SAM crews; Veteran and Elite add ships and patrols. Higher levels also increase radar sensitivity. |
| **Start time** | The takeoff hour; night from 18:00 to 06:00. *Random*: rolled with each new mission, 06:00-08:00 by day or 18:00-20:00 by night. |
| **Situation** | *Cold War*: covert peacetime operation, fewer enemy aircraft. *Limited War*: regional conflict, air defences on alert. *Conventional War*: full-scale war, all enemy forces active. The briefing states the exact rules of engagement. |
| **Mission** | *Strike*: ground target (destroy, photograph or reach), with a primary and a secondary objective. *Air to air*: intercept an enemy aircraft. *Strike training* and *Air training*: practice missions. *Drone defense* (Ukraine only): stop a wave of attack drones, then bomb their launch site (chapter 14). |
| **Aircraft** | *F-117A* or *F-16C*: the flown aircraft. *Realistic weapons* (F-117A only): two bays instead of four, no AIM-9/AIM-120, no carrier operations. |
| **Simulation** | The realism switches below and *Crash protection*: automatic pull-up near the ground with the gear up; ground contact and collisions are not fatal. They belong to each game (single mission, career, campaign), not to OPTIONS. |
| **Realism** | *Damage*: *Game*: hits damage systems but never bring the aircraft down. *Real*: a missile hit can destroy the aircraft outright, or damage several systems and start a fire; hits to the flight controls make it roll, pitch or wobble. *Guidance*: *Classic* keeps the DOS era missiles; *Real* missiles fly proportional navigation, glide after their motor burns out and loft against distant targets (chapter 7). *Low level*: *Real* adds anti-aircraft guns and MANPADS (chapter 10.3). |
| **Weather** | Random (based on the theater's climate), Clear, Scattered, Broken, Overcast, Storm. *Random cover* off: a slider sets the coverage of the main cloud layer. Clouds block optical seekers and visual identification for both sides; radar sees through clouds. |

The **mission summary** lists theater, situation, mission type, opponents, time of day and
departure time, weather, visibility and fuel. The **briefing** below it lists the objectives, the
departure and landing bases, the weather, the armament and intelligence notes.

The menu remembers its choices and switches for the next start.

### Options

OPTIONS (main menu) holds your own settings, saved for every later game; the in-flight menu has
the same pages (chapter 3.8).

| Tab | Settings |
|---|---|
| **Game** | *Cockpit*, *HUD*, *Avionics*: DOS era or modern (this manual describes the modern ones; avionics in chapter 4.1). *Flight model*: DOS era, or aerodynamic with fly-by-wire (the stick commands pitch and roll rate; released, the attitude holds). *Auto trim*: holds the attitude (helicopters) or trims out damage (jets); Alt + arrow keys trim by hand, Alt + Home centres. *Cockpit tooltips*, *VR cockpit size* (chapter 3.8), *Tacview export* and *real-time telemetry* (chapter 12). |
| **Graphics** | *Shaders*: flat DOS era, or anti-aliasing, shading and haze. *Shadows*. *Point lights*: cockpit lamps, explosions, flares and runway lights light the scene. *Time of day*: simplified, or the sun and moon moving with the mission clock (sunsets, stars, the moon's phase). Moonlight sets how far enemy pilots see you at night: about 2 km moonless, 4 km under a full moon, 8 km by day. *Automatic scenery detail (VR)*: lowers the scenery detail while frames run late, to hold the headset's frame rate. |
| **Sound** | *Master*, *Engine* (your engine or rotor, and the aircraft heard around you), *Effects* (weapons, explosions, gear, radio) and *Warnings* (stall horn, radar and missile warnings, fire siren, Sidewinder tones) volumes. *Engine low-pass*: inside the aircraft the engine sounds are muffled. |
| **Controls** | *Keys and joystick*: the input mapping (chapter 2). *Head/Eye Tracking* (desktop): your head turns and moves the view in the cockpit (chapter 3); below it the menu shows whether a tracker is heard. |

New players start with modern shaders, time of day, cockpit, HUD, avionics and flight model, and
classic shadows and lighting.

| Button | Function |
|---|---|
| BACK | Back to the main menu. |
| NEW MISSION | Generate different objectives with the same options and start time. |
| LOAD | Missions saved in flight (SAVE MISSION, chapter 3.8), the latest first. LOAD flies one on from where it was saved, with its aircraft and realism switches, starting paused; DELETE removes one. Not in career mode. |
| PLAN ROUTE | Below the mission summary: the mission planner (chapter 11). |
| LOADOUT | Below the mission summary: the stores on each hardpoint, seen from above. Click a bay or pylon and pick its load (*Same on both sides* mirrors it). Greyed out weapons are out of stock. BRIEFED restores the briefed stores, CLEAR empties every hardpoint. Which station takes what: chapter 7.2. |
| FLY IN VR | Restart the game in the VR headset with this mission (requires an OpenXR runtime). After the mission the debrief and the menu stay in the headset, on a panel in front of you: point a controller at it, pull the trigger to click, push the thumbstick to scroll; the left menu button puts the panel in front of you again. Starting the game with `--vr` opens the menu in the headset too. |
| FLY | Start the mission. |

## 2. Controls menu

![Input mapping](images/controls.png)

OPTIONS > CONTROLS > *Keys and joystick* opens the input mapping. The mapping is saved automatically and applies
to all later flights.

- **Search** (under the title): type to keep only the axes and actions whose name holds the
  text (any case), and the VR controller inputs whose name or action does. Backspace deletes,
  **Esc** or the field's **X** clears it. In VR, clicking the field opens the virtual keyboard
  under it.
- **Device list**: connected joysticks, throttles and gamepads.
- **Standard gamepad layout**: left stick for pitch and roll, right stick for rudder, triggers
  for throttle, face buttons for fire, gun, weapon select and bay doors.
- **Preview** (top right): an F-117 outline that moves with the mapped axes, with the throttle
  shown as the exhaust colour. If the aircraft moves the wrong way, enable *Invert* for that axis.
- **Axes**: Roll, Pitch, Rudder, Throttle, Sensor/cursor slew X and Y (modern avionics). Click **BIND**, then move the axis. *Invert* reverses
  the direction, *dead zone* sets the neutral range around the centre, and the bar shows the
  current value. **CLEAR** removes the binding.
- **Buttons**: every game action, with its keyboard key for reference. Click **BIND**, then press
  a button. A hat switch or an axis pushed past halfway can also be bound. **Esc** cancels the
  capture.
- **CLEAR ALL** removes all bindings; **DONE** closes the dialog.

![VR controller mapping](images/controls_vr.png)

**VR controllers**: the last section assigns an action to each VR controller input (trigger, A/X,
B/Y, stick click, stick directions) per hand. **DEFAULTS** restores the default assignment. A
trigger only counts while that hand holds the throttle (left) or the stick (right). The defaults
work like a HOTAS:

| | Right controller (stick) | Left controller (throttle) |
|---|---|---|
| Trigger | Fire weapon | Wheel brakes (held) |
| A / X | Gun (held) | Flare |
| B / Y | Select weapon | Chaff |
| Thumbstick | Targeting: up nearest target ahead, down clear target, right next target, left designate | Slews the displays' cursor |
| Stick click | Bay doors | Lock cursor target |

The left trigger's brakes are on while it is pulled, also with the hand off the throttle unless its
fingertip is on a control (any VR button set to *Brakes* works this way). The decoy is on the
cockpit's DECOY button. A mapping still on the old defaults is switched to these once; a
customised one is kept.

## 3. The cockpit

![Modern cockpit](images/cockpit.png)

The view shows the head-up display (HUD) at the top and the instrument panel below it. Hold the
**right mouse button** to look around; turn the **mouse wheel** while holding it to zoom in. **Click** a button on the panel to press it. Every button
has a keyboard equivalent (chapter 9). Rest the mouse on a button, display, instrument or warning
light and a **tooltip** names it, says what it does and shows its keys; parts hidden behind the
stick, the throttle or the glare shield can't be pointed at.

**Head/eye tracking.** With *Head/Eye Tracking* on in OPTIONS > CONTROLS, the cockpit view follows your head
movements, picked up by an infrared tracker (TrackIR, DelanClip, TrackHat, Grass Monkey
Simulations, ...), an eye tracker (Tobii Eye Tracker, Beam Eye Tracker, ...), a webcam or a smartphone. A tracking program such as OpenTrack sends them to the game (FreeTrack output, or UDP on port 4242).
**Home** recentres the view.

For comparison, the **classic cockpit** (menu: *Cockpit: Classic*) uses a DOS era panel with
the 77×60-pixel displays and lit lamps instead of labelled bezel buttons:

![Classic cockpit](images/cockpit_classic.png)

The rest of this manual describes the modern cockpit.

### 3.1 HUD and HUD mode buttons

The modern HUD is drawn in green in the style of the F-16's (in VR the F-117A's symbology is
drawn at half the size, about the F-16C's, so it stays in the middle of the large combiner):

![Modern HUD](images/hud_modern.png)

* **Top:** heading tape (tens of degrees) with the caret of the current steerpoint.
* **Top left:** g load, Mach number and the highest g of the flight.
* **Left / right:** airspeed scale (knots) and altitude scale ("1,5" = 1,500 ft) with the boxed
  readouts; climb rate in feet per minute above the altitude box, radar altitude ("R") below it
  under 5,000 ft.
* **Centre:** gun cross at the boresight and the flight path marker (the circle with wings: where
  the aircraft is going). The pitch ladder is centred on the flight path marker: a rung every
  5°, solid above the horizon, dashed and bent below it. With the gear down, the bracket left of
  the marker shows the approach angle of attack: marker in the middle of the bracket = on speed,
  high = fast, low = slow.
* **Bottom:** bank angle scale; mode, selected weapon and gun rounds (left); steerpoint number,
  distance and time to go (right). Messages such as "*Dessau detected*" appear at the top.

The modes add their own symbols:

* **NAV:** a diamond on the steerpoint on the ground; when the steerpoint is outside the HUD it
  waits at the HUD's edge in its direction, crossed out.

In every mode a small circle with a dot marks where the targeting pod looks while the TGP page is
up on an MFD, on the AH-1Z always (the classic HUD shows it and the steerpoint diamond too;
chapter 6).
* **AA:** the target designator box with the missile diamond once locked, the range below it;
  with an air-to-air missile selected the steering circle and dot, *SHOOT* in range, and the
  launch zone scale (maximum and minimum range, target range caret).
* **AG:** the target designator box (*IN RNG* in range); for guided bombs the azimuth steering
  line with the release cue that slides down to the flight path marker. With a free-fall or
  retarded bomb the bombsight replaces *AG* at the bottom left with its mode, **CCIP** or
  **CCRP** (chapter 5, *Bombing*), and the bottom right shows the slant range in nautical miles
  (*F004.2*) to the impact point (CCIP) or the target (CCRP), the time to release (CCRP,
  *REL 0:12*) and the bomb's time of fall (*TOF*):
  * **CCIP:** the bomb fall line runs down from the flight path marker to the pipper, a small
    circle with a dot where a bomb released now would land. While that point is below the HUD
    the pipper stays at the HUD's edge and the time delay cue crosses the line: dive steeper or
    wait.
  * **CCRP:** the azimuth steering line runs through the target designator box; the solution
    cue (a short bar on the line) slides down towards the flight path marker as the release
    point comes up and reaches it at release.
* **Gun** selected (in every mode): the gun pipper, where the target has to be for the rounds to
  hit (allowing for their flight time, the target's motion and drop). Without a lock it shows
  where the rounds hit the ground within 4 km; with a lock an arc around it unwinds with the
  range and *SHOOT* flashes within 2 km.

The **classic HUD** (menu: *HUD: Classic*) is a DOS era low-resolution HUD, as in the
classic cockpit picture above.

The HUD mode caps are the first four of the up-front display's top row (chapter 3.2):

| Button | Function |
|---|---|
| NAV | Navigation mode: steering to the current waypoint. |
| AIR | Air-to-air mode: aircraft targets. |
| GND | Air-to-ground mode: ground targets. |
| LOCK | Lamp: blinks while a target is designated, steady when the target is within weapon range. Pressing it designates a target (as N). |

F2 cycles through the modes; V declutters the HUD.

### 3.2 The F-117A's panel

The panel is laid out like the real F-117A's:

* **Up-front display** under the HUD: the **DED** (radio channels, steerpoint and target, clock,
  bearing, distance and ETA to the steerpoint, fuel) between two rows of caps (tables below).
* **Displays**: the large **centre display** (the right MFD's pages, chapter 4) between the
  **left MFD** and the **third MFD**, which starts on the **HSI** page (chapter 4.2). The warning
  lights sit over the side MFDs' inner corners.
* **Instruments**: airspeed, altimeter, AOA and vertical velocity tapes and the clock down the
  left edge; RPM, FTIT, oil pressure and nozzle position down the right. Under the left MFD the
  **attitude indicator** and the **BAY** cap; under the right MFD fuel, thrust and the radar bar
  graph.
* **Slopes and consoles**: the gear handle and aircraft systems on the left slope, the
  countermeasures on the right. Side stick on the right console, throttle on the left.

| Up-front display, top row | Key | Function |
|---|---|---|
| NAV, AIR, GND, LOCK | F2, N | HUD modes and designation (chapter 3.1). |
| DCLT | V | HUD declutter. |
| NVG | F4 | Night vision goggles. |
| ILS | F9 | Instrument landing system: the landing guidance on the centre display and, approaching a runway, the F-16-style ILS steering bars on the HUD (*ILS* at the bottom left): the localizer bar (upright) and glideslope bar (level) are referenced to the flight path marker; fly the marker to where they cross to be on the centreline and the 3° glide path. Each bar stops at full scale (2.5° off the centreline, 0.7° off the glide path). |

| Up-front display, bottom row | Key | Function |
|---|---|---|
| MENU | Esc | Open or close the in-flight menu (chapter 3.8). |
| VOL | Alt V | Sound level. |
| TIME | Shift Z / Shift X | Time acceleration on/off. Acceleration ends automatically when a radar detects the aircraft or a weapon is released. |
| STPT | F7 | Waypoints. |
| ORD | F10 | Mission orders. |
| COM, IFF | – | Not used. |

### 3.3 Warning lights

The lights are dark when inactive. Left: **MASTER CAUTION** (missile warning), **STALL**
(airspeed below stall speed, blinks), **TRACK** (an enemy radar tracks the aircraft), **IR**
(an infrared missile is in flight) and **LAUNCH** (a radar-guided missile is in flight).
Right: **ENG FIRE** (fire on board), **ENGINE** (engine damage), **FLCS** (fire control
damage), **JAM** (jammer damage: ECM and IRJ fail) and **AV** (avionics damage). The caution
lights at the top of the right slope: **FUEL** (fuel system damage), **BD** (bay door damage:
the doors stay open); the others are not used.

A fire on board also sounds the fire warning siren until the fire is out (with realistic damage
a fire burns out or blows the aircraft up; with game damage it burns on, leaking fuel). A damaged aircraft
trails smoke (grey, black when badly hit), and flames while it burns.

### 3.4 Gear, aircraft systems and targeting (left slope)

The gear handle (its arm shows the gear position: down = gear down) with the gear lights left
of it (an aircraft outline with a green lamp at each wheel while the gear is down), the caps
under them, and below them ahead of the throttle the target buttons. The **BAY** cap is under
the left MFD.

| Button | Key | Function |
|---|---|---|
| Gear handle | 6 | Landing gear. |
| FLAPS | 9 | Flaps: extend over about 3 s; more lift and a higher angle of attack limit, about 20 % lower stall speed. |
| BRAKE | 0 | Wheel brakes on the ground, speed brake in the air. |
| AUTO | 7 | Autopilot: flies to the current waypoint. |
| BAY | 8 | Weapon bay doors open/close. Weapons can only be released with open bays. |
| NEAR | T | Lock the nearest target ahead. AIR or GND mode depends on the selected weapon. |
| NEXT | B | Select the next target. |
| NONE | Shift B | Clear the target. No automatic acquisition until the next target or camera key. |
| WPN | Space | Select the next weapon, after the last one the gun. |
| DTL | Alt D | Detail level. |

The selected weapon is released with Enter, the gamepad's A button or the right VR trigger while
the hand holds the stick.

### 3.5 Countermeasures (right slope)

| Control | Key | Function |
|---|---|---|
| CHAFF | 2 | Release chaff. The counter shows the remaining stock; the lamp on the right lights on release. |
| FLARE | 1 | Release a flare. |
| DECOY | 5 | Release a decoy. |
| ECM | 4 | Radar jammer on/off (lit when on). |
| IRJ | 3 | IR jammer on/off (lit when on). Switches itself off after 20 s of continuous use and must cool down. |

### 3.6 Fuel, thrust and the radar bar graph

![Bottom strip](images/panel_bottom.png)

Under the right MFD, right of the fuel quantity: **FUEL** (fuel in lbs divided by 10) with the
**LOW FUEL** lamp (blinks below 1500 lbs of fuel) and **THRUST** (in percent); under them the
**EGR/EAR radar bar graph** (described in chapter 8.2) with the **EMV** warning lamp (lit for
2 s after each detection).

### 3.7 The F-16 cockpit

The F-16C has its own cockpit, laid out like the real Block 50's:

* **Up-front controls** under the HUD: **COM1** (volume), **MENU**, and the master modes **A-A**
  (again: NAV) and **A-G** (again: CCIP / CCRP). On the keypad **1 T-ILS** (ILS), **4 STPT**
  (WAY page), **6 TIME** (time acceleration), **7 MARK** (designate), **DCLT** and **NVG** work.
* **DED** (amber, right of them): radio channels, steerpoint and target, clock, bearing,
  distance and ETA to the steerpoint, time on target or fuel.
* **Flight instruments** in the middle: the attitude indicator with airspeed and altimeter, the
  AOA and vertical velocity tapes, the **HSI** below. An **MFD** on each side (the top and bottom
  button rows work).
* **Top left**: master caution, STALL, TRACK and IR lights, the **LAUNCH** light and the
  **RWR** scope. **Top right**: ENG FIRE, ENGINE and FLCS, with the engine instruments and fuel
  down the right edge. The other warning lights are on the right console.
* **Left slope**: gear handle and lights, **FLAPS**, **BRAKE**, **AUTO** (autopilot) and the
  target buttons (NEAR, NEXT, NONE, WPN, DTL). **Right slope**: clock and countermeasures (chaff,
  flare, decoy, ECM, IRJ). Side stick on the right console, throttle on the left.

The **AOA indexer** left of the HUD: red top chevron slow, green circle on speed, amber bottom
chevron fast.

It has no radar bar graph. The **RWR** shows every radar that sees the aircraft at its bearing
(up = ahead). It shows no range: the nearer the centre, the more dangerous (launching or tracking
SAMs innermost). Symbols: the SAM type (`2` SA-2, `10` SA-10, `N6` SA-N-6, `HK` Hawk, `RA`
Rapier, ...), aircraft radars with a hat (`29` MiG-29, `AW` AWACS). Dim = searching, bright =
tracking, a diamond = the most dangerous, a flashing circle = a missile in the air.

### 3.8 In-flight menu

**Esc** (also Pause, Alt P, the gamepad's Start button, the **MENU** cap of the F-117A's up-front
display or the F-16C's **MENU** key) opens the menu and pauses the flight: in VR a screen in front of the pilot, on
the desktop over the middle of the window. In VR, touch an
entry with a fingertip and pull the trigger, or use a thumbstick and A / X. On the desktop, click
an entry or use the arrow keys and Enter (gamepad: d-pad and A). Esc, BACK or the gamepad's B go
back a page; on the first page they close the menu.

| Entry | Function |
|---|---|
| Resume | Closes the menu (as does Esc again). |
| Save mission | Single missions only, while still in the aircraft: saves the flight as it is (the world, your aircraft, route, stores and the flight's record); LOAD in the single mission screen flies it on (chapter 1). Views, MFD pages and the kneeboard start afresh. |
| Options | The pages of the main menu's OPTIONS (chapter 1) that apply in flight, below. |
| Key guide | The keys as bound, the mouse or the VR controllers. Type to keep only the keys whose description holds the text (any case); Backspace deletes, the field's X or Esc clears it. In VR, a fingertip on the field and the trigger open a keyboard on the screen: touch a key and pull the trigger to type, DONE closes it. |
| End mission | Ends the flight and goes to the debrief. |

| Page | Entries |
|---|---|
| Game | *Cockpit* (F-117A only), *HUD*, *Avionics*, *Auto trim*, *Tooltips*, *Cockpit size* (VR only, 70–150 %: the size of the cockpit and the world in the headset). The flight model and Tacview are set in the main menu. |
| Graphics | *Shaders*, *Shadows*, *Point lights*, *Time of day*, *Sun and moon* (with the classic time of day), *Auto scenery detail* (in VR). |
| Sound | The *Master*, *Engine*, *Effects* and *Warnings* volumes in steps of 10 % (left / right or the < > halves), *Engine low-pass*. |
| Controls | *Head tracking* (desktop), *Keys and joystick*: the input mapping (chapter 2) over the view, in VR on a panel in front of you (point a controller, the trigger clicks); DONE returns to the menu. The key guide. |

Changes apply at once and are saved as the defaults for the next flight.

### 3.9 Kneeboard

A kneeboard lies on your left thigh: look down to read it. Its tabs: **BRIEF** (orders,
objectives, weather, armament, rules of engagement), **ROUTE** (each leg's bearing, distance and
time), **MAP** (the route and the SAM ranges) and **TARGET** (a close-up of the primary target and
the intelligence summary). Achieved objectives are ticked.

* Desktop: click a tab; **K** shows the board large over the view and turns its pages.
* VR: touch a tab with a fingertip; **K** or the *Kneeboard* action hides or shows it. Squeeze
  the grip with a hand on the board to pick it up and leave it floating in the cockpit.

## 4. MFD pages

The panel has two multi-function displays (MFDs), the F-117A's three: its right MFD is the large
centre display, its third MFD right of that shows the HSI (chapter 4.2; with the modern avionics
any format). With the **classic avionics** (menu: Classic /
Modern, Avionics) they show the low resolution pages described here; the **modern avionics** are in
chapter 4.1. The labelled buttons above and below each screen select the pages and their
functions. The button of the active page is lit.

| Left MFD buttons | Function |
|---|---|
| MAP / TAC | Select the page (key F3 toggles). |
| NUP / HUP | Map orientation: north up or heading up (Alt F3). |
| ZM- / ZM+ | Zoom out / in (Alt X / Alt Z). |

| Right MFD buttons | Function |
|---|---|
| TC | Tracking camera. |
| WAY | Waypoints (F7). |
| WPN | Weapons (F5). |
| ORD | Mission orders (F10). |
| FLIR | Thermal camera on/off (F6). |
| Lower row | Page functions, see below. |

<table>
<tr>
<td width="50%"><img src="images/mfd_map.png" alt="MAP"><br>
<b>MAP</b> – Top-down map of the theater with the own aircraft (triangle), the route and
waypoints, airbases and ground objects. A scale bar is shown at the bottom left. During a radar
sweep, the map shows the radar's coverage (chapter 8.3).</td>
<td width="50%"><img src="images/mfd_tac.png" alt="TAC"><br>
<b>TAC</b> – Heading-up tactical display with a grid (spacing shown at the bottom). Own aircraft
in blue. Other aircraft are shown as triangles pointing in their direction of flight: red at
the same altitude (±1000 ft), dark red below, yellow above. Missiles are shown as short lines;
radar sites, airbases and targets as icons.</td>
</tr>
<tr>
<td><img src="images/mfd_tc.png" alt="TC"><br>
<b>TC</b> – Tracking camera. Shows the current target with its name, range, bearing and
zoom factor. AHD / RGT / REAR / LFT (keys / , . M) point the camera ahead, right, rear or left;
each press also picks the nearest object in that direction. The last bottom button sets the zoom
(AUTO by range, then 1X..6X; also `[` / `]`).</td>
<td><img src="images/mfd_way.png" alt="WAY"><br>
<b>WAY</b> – Mission time and waypoint table: distance, flight time and ETA for each waypoint,
and a fuel bar (reserve green, needed white and blue, used black). WP- / WP+ (PgUp / PgDn)
select a waypoint. F8 edits it (red): keypad 8 / 2 / 4 / 6 move it over the MAP, 9 / 3 select
the last / next one; F7 leaves. RST (Shift F8) restores the original route.</td>
</tr>
<tr>
<td><img src="images/mfd_wpn.png" alt="WPN"><br>
<b>WPN</b> – The weapons carried and how many are left; the selected one is highlighted.
SEL (Space) selects the next weapon, after the last one the gun; the button next to it shows
the bombsight mode, CCIP or CCRP, and switches it (Alt F2).</td>
<td><img src="images/mfd_ord.png" alt="ORD"><br>
<b>ORD</b> – Mission orders: primary objective (yellow), secondary objective (red), landing
base (green).</td>
</tr>
<tr>
<td><img src="images/mfd_flir.png" alt="FLIR"><br>
<b>FLIR</b> – The tracking camera as a black-and-white thermal picture: aircraft, vehicles and
buildings appear bright.</td>
<td></td>
</tr>
</table>

### 4.1 Modern avionics

With the modern avionics each MFD can show any of five F-16 style formats, chosen with the
bottom row of bezel buttons: **HSD**, **HARM**, **TGP**, **MAP** and **SMS**; the F-117A's
third MFD offers **HSI** in place of TGP. The buttons' functions are written on the display next
to them (lit = selected); the bezel caps are blank. Choosing a format another MFD shows swaps
the two. F3 cycles the left MFD through HSD, MAP
and HARM; F5 / F10 bring up SMS, the camera keys the TGP (F6 too, switching its FLIR on or
off with each press), F7 the waypoint table.

| Format | Top row buttons | Shows |
|---|---|---|
| HSD | DEP / CEN, RNG-, range, RNG+, SOI | Horizontal situation display, heading up: range rings (the range in km is the outer ring), the route with the current steerpoint ringed, SAM sites with their missile's reach as red threat rings, hostile (red) and friendly (green) aircraft, ships (hull symbols, red or green; armed ones with their threat ring), missiles, the mission targets (cyan diamonds); the locked target is boxed. |
| HARM | ALL / SAM / AIR, SOI | The radars painting the aircraft, like the RWR: at their bearing, the most dangerous nearest the centre (chapter 3.7). |
| TGP | FLIR, zoom, SP, CZ, SOI | The targeting pod (tracking camera), see below. The pod's mode (SLV / AREA / PNT / SP) is shown bottom left, the zoom bottom right (AUTO: by range). |
| MAP | NUP / HUP, ZM-, ZM+, WAY | The moving map in muted colours; WAY switches to the waypoint table (WP-, WP+, RST, SOI): while it is the SOI, the keypad moves the waypoint being edited (F8). |
| SMS | WPN, ORD, SEL, CCIP / CCRP | Stores (WPN) and mission orders (ORD); on WPN the bombsight mode. |
| HSI | WP-, WP+ | Horizontal situation indicator (chapter 4.2). |

**Sensor of interest (SOI).** The HSD, HARM display or TGP with the white frame is the SOI (its
SOI button, or Alt S to move it). Its cursor moves with the **sensor/cursor slew** controls:
keypad 8 / 2 / 4 / 6, the left VR thumbstick or two bound joystick axes. **Lock cursor target**
(keypad Enter, left thumbstick click or a bound button) locks what is under the cursor: an
aircraft, a ground object, a ship or a radar. Held on the HSD or HARM display it opens a 4×
magnified window; letting go locks. Z / X change the HSD range. The cursor stays on its target
as you turn.

**Targeting pod.** The TGP format and how to work the pod (SLV, AREA, PNT, SP, CZ, zoom and
locking with it) are in chapter 6.1. The HSD shows where the pod looks: a green diamond on its
ground point with a dashed line from the aircraft.

### 4.2 The HSI page (F-117A third MFD)

A heading-up compass rose with the heading boxed above it. WP- / WP+ select the previous / next
steerpoint.

* Green **course arrow**: the leg to the current steerpoint. Its middle part, the **deviation
  bar**, shows how far you are off the course (one dot 2.5 km, two dots 5 km).
* Small green triangle: TO (along the arrow) or FROM (once the steerpoint is passed).
* Cyan **bearing pointer**: points at the steerpoint.
* Left: steerpoint number, distance (km), ETA. Right: bearing, course, ground speed (kt).

## 5. Targeting and weapons

![Ground target locked](images/targeting.png)

The picture shows a ground target locked in GND mode: the HUD box marks the target, the TC page
shows the target (here *Gutersloh Airbase*, 0.9 km), and the GND lamp is lit.

**Procedure**

1. Select the weapon with **WPN** (Space). The HUD shows it at the bottom left.
2. Lock a target: **NEAR** (T) for the nearest target ahead, **DESG** (N) to designate,
   **NEXT** (B) to step through targets. The camera keys (/ . , M) also select targets in
   their direction.
3. Open the bay doors (**BAY**, 8).
4. Release with Enter (gamepad A, VR: right trigger) when **LOCK** is steady. Releases fail with *Bay closed*,
   with a bank angle above about 67°, or on the ground (*Insufficient Altitude*).
5. Close the bay doors. Open doors increase the radar signature (chapter 8.1).

**Bombing:** free-fall and retarded bombs have two release modes; CCIP is the default. Switch
with Alt F2, the F-16's **A-G** button pressed again in AG mode, or the CCIP / CCRP button on the
weapons page. The HUD shows the mode at the bottom left (in AG mode).

* **CCIP** (continuously computed impact point): put the pipper at the end of the bomb fall line
  on the target and release with Enter. Free-fall bombs need a dive or some height to bring the
  pipper into the HUD; low and level, retarded bombs do.
* **CCRP** (continuously computed release point): lock the target (NEAR, DESG, NEXT or a display
  lock), steer the flight path marker onto the azimuth steering line and hold Enter (or the
  trigger): the bomb is released by itself when the solution cue reaches the flight path marker,
  i.e. when the target passes the release point. Pressing Enter does not release the bomb in
  CCRP; without a target the HUD says *No target designated*. Let go of the button to hold
  the release.

**Gun:** select it with **WPN** (Space) after the last weapon, or fire it with Backspace
whatever is selected. Hold the trigger (Enter, gamepad A, VR: right trigger) with the target in
the gun pipper (chapter 7.3). The real F-117A has no gun: with *Realistic weapons* there is none.

**Weapon view:** Shift F4 follows your newest missile or bomb in flight from behind (above a
falling bomb); pressed again it switches to your next weapon in flight. After the impact the camera
stays on the explosion and slowly pulls back for 10 seconds, then returns to the view you came
from. The right mouse button orbits around the weapon and the mouse wheel zooms.

Ground targets under cloud cannot be designated (*Target obscured*). IR and optical seekers
(AIM-9, Maverick, laser-guided bombs) need a clear line of sight.

With Sidewinders selected in the air-to-air HUD mode you hear the seeker: a low growl while it
searches, a higher growl once the locked target is in its view (the LOCK oval), and a
steady high tone when the target is in range: fire.

**Weapons**: the stores, their ranges and how to deliver each one are in chapter 7; locking
with the targeting pod and the helmet sights in chapter 6.

## 6. Targeting pod and helmet sights

### 6.1 The targeting pod

The F-16C and the F-117A carry a targeting pod (TGP) with the modern avionics; its picture is the
**TGP** format on an MFD (chapter 4.1). With the classic avionics the TC page shows the same
camera, pointed only by the camera keys. On the AH-1Z the pod is always working, with or without
its format on a display. The pod's picture shows in the forward cockpit view, not while you look
left, right or back.

**Bringing it up.** The TGP button under an MFD, or a camera key: `/` `.` `,` `M` point it
ahead, rear, right or left and pick the nearest object that way; T (nearest target) does the
same ahead. F6 brings up the TGP and switches its thermal picture (FLIR) on or off with each
press. B (next target) moves the pod onto the next target without bringing up the page; N
(designate) and Shift B (clear) leave it where it looks.

The format's top row: **FLIR** (black-and-white thermal picture), the **zoom** (AUTO or 1X..6X),
**SP** (snowplow), **CZ** (cursor zero, blank in A-A) and **SOI**. Bottom left the pod's mode,
bottom right the zoom; the cross in the middle is where the pod looks.

| Mode | The pod... | How you get there |
|---|---|---|
| **SLV** (slaved) | looks at the locked target. Without a lock it looks where the camera keys turned it, about 10° below the horizon (the AH-1Z's looks where you look, chapter 6.3). | Every target key (T, B, the camera keys), every lock on a display. |
| **AREA** | turns with the slew controls, steadied against the aircraft's motion. | Slew with the pod the SOI. |
| **PNT** (point) | holds the ground point it looks at, wherever the aircraft flies. | A moment (0.15 s) after the slew stops, on a ground point within 40 km. Above the horizon or farther, it stays in AREA. |
| **SP** (snowplow) | looks straight ahead and about 9° down, turning with the aircraft, holding nothing; the locks are dropped. | SP with the pod the SOI. Slew from there, or press a target key. |

**Finding and locking a target with the pod:**

1. Make the TGP the **SOI** (its SOI button, or Alt S until the white frame is on it). The pod
   only slews while its format is up and it is the SOI.
2. Point it near the target: **CZ** turns it to the current steerpoint (on a strike that is the
   target) and holds it there; a camera key turns it to a quarter; SP puts it ahead of the nose.
3. **Slew** onto the target: keypad 8 / 2 / 4 / 6 (and the diagonals 7 / 9 / 1 / 3), the left VR
   thumbstick, or the two joystick axes bound as *Sensor/cursor slew X/Y* (chapter 2). Full
   deflection turns the pod about 28° a second at 1X, half that at each zoom step (under 1° a
   second at 6X), so zoom in for the last bit. Beyond 3 km the response is finer around the
   centre and a held slew starts slowly, building up to full rate within a second: tap it for
   small corrections.
4. **Zoom**: the zoom button steps AUTO, 1X .. 6X; `[` / `]` zoom out and in, and so do Z / X
   while the pod is the SOI. 1X shows about 17°, each step half of that; AUTO zooms in one step
   each time the range doubles from 500 m.
5. **Let go.** The pod holds the point (PNT). Settled within 30 m of a vehicle, building or site
   it locks it by itself: the HUD goes to GND and the pod to SLV, following the lock. An
   anti-aircraft gun can't be locked; the pod just holds on it.
6. Otherwise press **Lock cursor target** (keypad Enter, the left thumbstick click in VR, or a
   bound button): it locks the object nearest the pod's point within about 125 m, else the
   ground point itself (*GROUND POINT*), and the pod follows the lock (SLV). In SP it locks the
   ground ahead of the snowplow line; in SLV the button does nothing. Under a cloud the HUD says
   *Target obscured* and nothing is locked.

The pod turns about 170° either side of the nose, 100° down and 29° up from the airframe. Where
it looks shows on the HUD and the helmet visor (a small circle with a dot) and on the HSD (a green
diamond with a dashed line from the aircraft).

**Which weapons use the pod.** The **Hellfire** flies to wherever the pod looks, lock or no lock
(chapter 7.5). The **Maverick**'s and the Hellfire's launch zones are measured to the pod's
point when you have slewed it off the lock. The laser-guided **GBU-12** does not need the pod: it
guides on the locked target however it was locked, as long as you can see it (chapter 7.6). Every
other weapon uses the lock, which the pod is one way to make.

### 6.2 The F-16C's helmet (HMCS)

The F-16C's pilot wears a helmet-mounted cueing system (HMCS), the AH-1Z's a full helmet-mounted
display (chapter 6.3). The F-117A and the UH-60 have neither. The symbology is drawn on the visor,
40° wide and 30° high, wherever you look in the cockpit (also in the look left, right and rear
views), and lies on the world like the HUD's.

**Where you look** is the helmet sight:

- **Desktop:** hold the right mouse button and move the mouse (up to about 150° either side);
  the wheel zooms meanwhile.
- **Head or eye tracking** (chapter 3): the tracker turns the view and the sight with it; Home
  recentres.
- **VR:** the headset.

The sight is smoothed, so a trembling head does not shake it, but follows a deliberate turn
quickly.

**The HMCS shows** the horizon, the boxed airspeed (left) and altitude (right), the route's
waypoints (the current steerpoint larger, with its number), the locked target (a box on a ground
target, a diamond on an aircraft; off the visor an arrow points towards it), where the targeting
pod looks, and with Sidewinders selected the seeker's circle. It has no aiming cross.

**Locking an aircraft by looking at it (Sidewinder):**

1. Select the **AIM-9** (Space) and the **AIR** mode (F2, or the F-16C's A-A button).
2. Look at the aircraft. The seeker's 6° circle follows your eyes up to 40° off the nose.
3. The seeker locks the aircraft nearest the middle of the circle by itself (no button): a
   diamond on it, and the growl rises. It must be in sight, not behind a cloud.
4. Keep looking at it. The steady high tone says it is in range: fire (Enter). Look away and the
   seeker loses it as it leaves the circle.

The seeker does not tell friend from foe: in multiplayer it locks other players as readily. A
lock you made yourself (T, B, the HSD cursor) is kept: the circle then sits on that target
instead of following your eyes, and the lock is not dropped when you look away.

The helmet only aims the Sidewinder. The AIM-120 locks with the HUD keys and the HSD, ground
targets with the target keys, the displays and the pod (6.1); the F-16C's pod does not follow the
helmet.

### 6.3 The AH-1Z's helmet (HMD)

The AH-1Z's visor shows everything the HMCS does except the waypoints, and:

- a **cross** where you look: the line of sight that a helmet-slaved gun follows;
- the turret gun's **pipper** while the gun is slaved to the helmet, where its rounds go; at the
  visor's edge it blinks, crossed out, when the target is beyond the turret's reach;
- the gun line under the cross while the gun is selected (`GUN 510 HMS` / `GUN 510 TGP`).

**The pod follows your eyes.** Without a lock the AH-1Z's pod (SLV) looks wherever you look;
with one it looks at the lock. So:

- **A Hellfire fired without a lock goes where you are looking** (and keeps following the pod,
  so keep looking at the target until it hits).
- **To lock a ground target by looking at it:** look at it, nudge the slew with the TGP the SOI
  (the pod goes to AREA from your line of sight), and let go: it settles on the point and locks
  what is there (6.1, step 5), or press **Lock cursor target**. T and B also pick targets in the
  pod's direction.
- **Aircraft** lock as on the F-16C: the Sidewinder's seeker follows your eyes (6.2).

**The turret gun** is two entries in the weapon cycle (Space): **GUN HMS**, the turret pointing
wherever you look, then **GUN TGP**, the turret following the pod; the next press goes on to the
missiles. The turret turns 110° either side, 50° down and 20° up.

## 7. Weapon employment

### 7.1 Rules for every weapon

- **HUD mode:** air-to-air weapons in AIR, all others in GND. T (nearest target) switches to
  the selected weapon's mode by itself.
- **Release:** the F-117A's bay doors open (*Bay closed*, except the fuel tank; the F-16C's and
  the AH-1Z's stores hang outside), bank under 67°, and in the air (*Insufficient Altitude* on
  the ground or the deck). At most four of your weapons fly at a time: a fifth press uses up a
  round without firing it. Each shot comes off the bay or station with the most rounds left.
- **Lock:** a guided air-to-ground missile is given its target at launch only while the LOCK
  lamp is on (the target in front of you and within its range); fired without, it flies on
  blind. The HARM only takes a radar, the Penguin and the Harpoon only a ship.
- **Clear sky:** the Sidewinder, the Maverick, the Hellfire and the laser-guided bombs need to
  see their target: none of them can be aimed at a target under a cloud. Radar weapons see
  through clouds.
- **Effect:** each weapon works better against some targets than others: a hit with too light
  a weapon may leave the target standing (*Ineffective*). The HARM only kills radars, the
  Durandal and the mines runways; the Maverick and the Hellfire are best against armour,
  launchers and radars; the GBU-12 and the CBU-72 do well against nearly everything.

**The launch zone.** With a missile selected and a target locked, the HUD's scale shows the
range at which it can reach the target: a line from the shortest to the longest range with a
caret at the target's range, and for air-to-air missiles a box for the no-escape range (the
target can't outrun the missile even if it turns away at launch). The zone is worked out by
flying the real missile from your present speed, altitude and pitch, so climbing, diving and
speeding up move it. **SHOOT** blinks while the caret is in the zone, **TOO CLOSE** inside the
shortest range. The guided bombs have no zone: their cue is *IN RNG*.

**Realistic and classic guidance** (the *Guidance* switch, chapter 1). Classic missiles turn in
fixed steps and fly until their fuel time runs out. Realistic missiles fly proportional
navigation to the collision point, burn their motor for the real one's burn time (the smoke trail
ends with it) and then glide, slowing with every turn; they must see the target within 30° of
their nose to lock on. In a campaign or a coop game the server's setting applies.

### 7.2 Ranges

The ranges are those the LOADOUT page gives for each store (in level flight at 450 kt, ballistic
bombs at 500 kt); a missile reaches farther from higher and faster, and an air-to-air missile
farther head-on.

| Weapon | Against | Range | Launch / release height | Carried by |
|---|---|---|---|---|
| AIM-9M Sidewinder | Aircraft (IR) | 1-12 km | Any | F-16C stations 1-3, 7-9; AH-1Z wingtips; F-117A* |
| AIM-120A AMRAAM | Aircraft (radar) | 1-38 km | Any | F-16C stations 1-3, 7-9; F-117A* |
| AGM-88A HARM | Radars | 9-50 km | 5,000-20,000 ft | F-117A, F-16C |
| Penguin 3 | Ships | 6-14 km (classic guidance 13-14) | 100-500 ft | F-117A, F-16C stations 3, 7 |
| AGM-84A Harpoon | Ships | 6-41 km | 2,000-5,000 ft | F-117A, F-16C stations 3, 7 |
| AGM-65D Maverick | Ground (TV) | 5-10 km | 2,000-5,000 ft | F-117A, F-16C |
| AGM-114 Hellfire | Ground (laser, the pod's spot) | 4-8 km | 2,000-3,000 ft | AH-1Z (16) |
| GBU-12 Paveway | Ground (laser) | 1.5 km to 2.5 × height (14 km from 15,000 ft) | 15,000-25,000 ft | F-117A, F-16C |
| Mk 20 Rockeye II, CBU-72 FAE | Ground (laser) | 1.5 km to 2.5 × height (3.7 km from 3,000 ft) | 3,000-5,000 ft | F-117A, F-16C |
| Mk 82 LDGP | Ground | 5-11 km short of the target | 3,000-20,000 ft | F-117A, F-16C |
| Mk 122 Fireye | Ground | 0.9-2.2 km short | 100-500 ft | F-117A, F-16C |
| Mk 20 Rockeye, Mk 35 | Ground | 1.1-1.6 km short | 300-1,000 ft | F-117A, F-16C |
| Mk 82 Snakeye, ISCB-1 mines | Ground | 0.9-1.3 km short | 200-500 ft | F-117A, F-16C |
| BLU-107 Durandal | Runways | 0.9-1.3 km short of the runway | 200-500 ft | F-117A, F-16C |
| IR/optical camera | Recon target | Photos from 2-8 km | Any | F-117A, F-16C centreline |
| 20 mm gun | Air and ground | Up to 2 km (ground: 4 km) | Any | All but the real F-117A and the UH-60 |

\* Not with *Realistic weapons*. On the F-16C the HARM, the Maverick and the bombs go on
stations 3-4 and 6-7, the fuel tank on 4, 5 or 6.

### 7.3 Air-to-air: Sidewinder, AMRAAM and the gun

**AIM-9M Sidewinder.** Infrared: it needs to see the target (no cloud between) and locks without
radar. Select it in AIR mode; the HUD locks the nearest aircraft within 4 km ahead by itself, T
locks the nearest one ahead, B steps outwards. With the F-16C's or the AH-1Z's helmet, look at
the target instead (chapter 6.2). Listen to the seeker: a low growl while it searches, a higher
growl once it sees the locked target, a steady high tone in range. Its motor burns 5 s; fire it
inside the launch zone.

**AIM-120A AMRAAM.** Radar guided: lock with T, B, NEXT or the HSD cursor (no line of sight
needed, clouds don't matter). It is the long-range weapon: fire from the launch zone's longest
range inwards, the no-escape box for a sure kill against a manoeuvring target.

With realistic guidance it **lofts** against targets beyond about 6 km: it climbs (14° to 34°,
steeper for farther targets), flies high where it loses little speed and dives on the target.
**Pull up 10-20° before a long-range shot**: the missile has less climbing to do and the zone on
the HUD grows. Its motor burns 8 s, then it glides.

**Gun.** Select it after the last weapon (Space) or fire it with Backspace whatever is selected,
and hold the trigger. 510 rounds. The rounds leave at 1050 m/s plus your airspeed and drop; with
an aircraft locked put it in the gun pipper (SHOOT flashes within 2 km, the arc around the pipper
unwinds with range, chapter 3.1). Against the ground the pipper shows where the rounds hit within
4 km. A round counts as a hit within 150 m of its target against green opponents, 60 m against
elite ones.

### 7.4 HARM and the anti-ship missiles

**AGM-88A HARM.** Kills only radars: SAM sites and the radar of a ZSU-23-4 Shilka.
Lock a radar on the **HARM** display (modern avionics): it shows the radars that are painting you,
the most dangerous nearest the centre; ALL / SAM / AIR filters them. Put the cursor on one and
press Lock cursor target: the HUD goes to GND with the site locked. T and B also lock radar sites.
Fire from 9-50 km, from 5,000 to 20,000 ft; a high, fast launch reaches farthest. A radar must be
on the air to show on the HARM display, so the HARM is fired at sites that are looking for you:
fire before they see you, from beyond their missile range (chapter 10.2).

**Penguin 3** and **AGM-84A Harpoon.** Ships only: lock the ship (T, B or the HSD cursor; ships
show from far out on the HSD) and fire inside the zone. The Penguin is a short-range weapon for a
low run, 6-14 km from 100-500 ft, its motor burning all the way. The Harpoon flies 6-41 km on its
turbojet, from 2,000-5,000 ft. Fire from outside the ship's SAM range where you can (chapter
10.4).

### 7.5 Maverick and Hellfire

**AGM-65D Maverick.** A TV guided missile against anything on the ground, best against armour,
launchers and radars. It needs a clear view of the target. Lock it in GND mode (T, B, the HSD
or the pod), and fire with the LOCK lamp on and the caret in the zone: 5-10 km, from 2,000-5,000
ft. With the pod slewed off the lock the zone is measured to the pod's point, but the missile
flies at the lock.

**AGM-114 Hellfire** (AH-1Z, RAH-66). Laser guided: it flies to wherever the targeting pod
looks and follows the pod as it moves, so **keep the pod on the target until it hits**. It needs a
line of sight to the spot. SHOOT shows when the spot is in the zone, with or without a lock. In
the AH-1Z the pod follows your eyes without a lock (chapter 6.3), so you can fire just by
looking at the target. Range 4-8 km, from 2,000-3,000 ft.

**Lobbing a Hellfire.** With realistic guidance it flies straight along your nose for up to 1 s
before turning to the spot, and its motor burns only 2.5 s. Against a distant target, raise the
nose until the caret is in the zone and SHOOT shows: the missile climbs, coasts farther and comes
down from above. A level shot from a low hover runs out of speed about 7 km out.

### 7.6 Laser-guided bombs: GBU-12, Rockeye II, CBU-72

The GBU-12 Paveway, the Mk 20 Rockeye II and the CBU-72 fuel-air bomb glide down to the
**locked ground target**, steering all the way. The laser is assumed to be yours: the bomb guides
as long as you can see the target, whichever way you locked it (T, B, N, the HSD or the pod; the
pod's picture is not needed), and a cloud between you and the target leaves it falling blind.
Changing the lock in flight sends the bomb to the new target.

1. Lock the target in GND mode. The HUD shows the azimuth steering line and the release cue.
2. Fly the flight path marker onto the steering line and bring the target into range: the LOCK
   lamp lights within 5 times your height, **IN RNG** shows within 2.5 times the height (at
   least 1.5 km).
3. Release with the LOCK lamp steady, and keep the lock (and the target clear of clouds) until
   the bomb hits.

The GBU-12 is dropped from high up (15,000-25,000 ft: 14 km from 15,000 ft, 18 km from 25,000 ft),
out of reach of the guns and MANPADS; the cluster bombs from 3,000-5,000 ft.

### 7.7 Unguided bombs

Free-fall bombs (Mk 82 LDGP, Mk 122 Fireye) and retarded and cluster bombs (Mk 82 Snakeye, Mk 20
Rockeye, Mk 35, ISCB-1 mines, Durandal) are aimed with the bombsight, CCIP or CCRP (chapter 5).
Free-fall bombs keep their speed and land far ahead (about 4,000 ft from 500 ft at 450 kt).
Retarded bombs slow down within seconds and fall steeply, landing behind you, so they can be
dropped low: 200-1,000 ft, about 1 km short (table in 7.2). Lofting a bomb throws it farther; the
CCIP pipper always shows where it would land.

**Durandal** and the **ISCB-1** mines are for runways: release on the runway's line, 0.9-1.3 km
short, from 200-500 ft. A bomb that goes off close under you damages your own aircraft: don't
release free-fall bombs low and slow.

### 7.8 Camera, fuel tank and cargo

**IR/optical camera.** For reconnaissance objectives: open the bay, bring up the camera page (TC
or TGP), lock the recon target and put the nose on it (the camera looks about 10° down). The HUD's cue walks you through it
(*SELECT CAMERA*, *OPEN BAY*, ...) and says *TARGET IN VIEW - SHOOT* when a photo counts: within
8 km against green opponents, down to 2 km against elite ones. Each press of Enter takes a frame.

**Fuel tank.** Releasing it transfers its 1,900 lb into the aircraft's tanks (no bay doors
needed).

**Special cargo.** Release on *Drop supplies now*, within 4 km of the drop zone.

## 8. Radar, detection and evasion

### 8.1 How detection works

Each radar sweep produces a **signal** against the aircraft. The aircraft itself has an
electromagnetic visibility (**EMV**). The aircraft is detected when **signal + EMV exceeds 100**.

**EMV** increases with:
- altitude (higher = larger),
- speed (500 kt adds about 13),
- bank angle (90° bank adds 32),
- open bay doors (+24),
- ECM switched on (+16).

**Radar signal** depends on:
- **Distance**: the signal decreases linearly with distance and reaches zero at the radar's
  maximum range.
- **Terrain under the aircraft**: open sea gives the strongest signal, normal land less, hills
  half of that. Over **mountains** no radar can detect the aircraft.
- **Height above the radar**: an aircraft high above a nearby radar is in the blind cone above
  it. Example: at 10,000 ft above the site, radars closer than about 10 km cannot see the aircraft.
- **Aspect and radar type** (see 8.4).
- **Difficulty**: higher opponent levels make radars more sensitive.

**Consequences of detection**: the site's alert level rises, the RWR tone sounds, the EMV lamp
lights and time acceleration stops. A site that stays alerted raises a theater-wide alarm, starts
tracking (**TRAK** lamp) and launches missiles when the aircraft is within its missile's lock
range. After repeated detections the HUD reports "*Radar ID*". Signals below the threshold only
produce a short tone, and the site's alert level decreases with each sweep that does not detect
the aircraft.

Enemy fighters that detect the aircraft by radar can identify it visually within about 8 km by
day and 4 km by night (not through clouds).

### 8.2 The radar bar graph (F-117A)

![Radar bar graph](images/panel_bottom.png)

The bar graph has two strips of LED segments:
- **EGR** (upper strip): signal of the last **ground** radar sweep.
- **EAR** (lower strip): signal of the last **airborne** radar sweep.

The radar signal lights segments from the **left in yellow**. The own **EMV** lights segments from
the **right in green**. Segments lit by both turn **red**: the two values add up to more than 100
and the **aircraft is detected**. A gap between the yellow and the green segments is the remaining
margin.

In the picture, an airborne radar (lower strip, yellow) is sweeping the aircraft; the EMV (green)
is low and the bars do not meet, so the aircraft is not detected.

The classic cockpit shows the same values in a single gauge: the EMV as a red bar from the right
(the blue remainder is the margin), with the EGR and EAR signal bars growing from the left over it.

### 8.3 Radar coverage on the map

![Radar coverage on the MAP page](images/radar_map.png)

During a sweep, the MAP page draws the radar's coverage as a circle around the radar. Its
radius is the distance at which that radar detects the aircraft **with its current EMV**. If
the own aircraft symbol is inside the circle, the aircraft is detected. A searching radar
shows a full circle; an alerted ground site shows a 90° sector pointing towards the aircraft.

Colours: ground radars dark red (weak), red (strong), yellow (detected); airborne radars cyan
(not detected) or white (detected). The picture shows an airborne radar (cyan circle); the
aircraft (white triangle, left) is outside the circle and not detected.

### 8.4 Radar types

| Type | Users | Strongest signal | Weakest signal |
|---|---|---|---|
| **Pulse radar** | SA-2, SA-5, SA-8B, SA-11, SA-13, SA-N-5, SA-N-7, Rapier, Tiger, Seacat; MiG-17, MiG-21, MiG-23, MiG-25, F-4E, F-5, Yak-38 | Aircraft seen from the side (beam aspect) | Aircraft flying directly towards or away from the radar |
| **Doppler radar** | SA-10, SA-12, SA-N-4, SA-N-6, Hawk, OTH radar; MiG-29, MiG-31, Su-27, F-14, Mirage F-1, IL-76 | Aircraft flying directly towards or away from the radar, at high speed | Aircraft flying at right angles to the radar (beam), at low speed |

Ground radars sweep all around. Fighter radars only see within about 67° either side of the
fighter's nose. The IL-76 airborne radar sees all around.

### 8.5 Evading radar

- **Keep EMV low**: fly low, at moderate speed and with small bank angles. Keep the bay doors
  closed until just before release. Switch ECM on only against missiles in flight, because it
  raises EMV.
- **Use the terrain**: fly over mountains and hills. Avoid flying over open sea near radars.
- **Mind the obstacles down low**: buildings, trees, pylons and power line wires are solid.
  Flying into one ends the flight like hitting a mountain (with crash protection it throws
  the aircraft back up).
- **Fly the right aspect**: point the nose towards or away from **pulse** radars. Fly at right
  angles to **Doppler** radars and reduce speed.
- **Watch the bar graph**: if the yellow and green bars approach each other, change heading,
  descend or slow down.
- **Watch the map**: keep the aircraft symbol outside the coverage circles. Stay out of the
  forward sector of enemy fighters.
- **Keep distance**: the signal drops with range. A site out of missile lock range can detect
  but not engage.

### 8.6 Defeating missiles

The RAD and IR lamps show which kind of missile is in flight; MW warns of a missile.

| Missile guidance | Examples | Countermeasures |
|---|---|---|
| Radar command guidance | SA-2, SA-5, Tiger, Seacat | Chaff; ECM (always effective). |
| Semi-active radar | SA-8B, SA-11, SA-N-4, SA-N-7, Hawk, Rapier, AA-6, AA-7, AA-9, AIM-7 | Chaff; ECM beyond a minimum distance. |
| Active radar | SA-10, SA-12, SA-N-6, AA-10 (MiG-29/IL-76 in Central Europe, North Cape, Ukraine) | ECM beyond a minimum distance; chaff only while flying at right angles to the missile. |
| Infrared, all-aspect | AA-2, AA-6 (IR), SA-N-5 | Flares; IR jammer (always effective). |
| Infrared, rear-aspect (can only lock on from behind the aircraft) | SA-13, AA-8, AA-10 (IR), AIM-9W | Flares; IR jammer beyond a minimum distance. |

The minimum distance for ECM and IR jammer is about 6 km (Green), 10 km (Regular), 14 km
(Veteran) and 18 km (Elite). A **decoy** draws missiles and the alarm position away from the
aircraft while it is active. A radar-guided missile more than about 8 km from the aircraft loses
guidance when its launcher stops tracking the aircraft. Bringing the signal + EMV below 100
therefore also defeats missiles at long range.

With realistic guidance a missile's energy is the other way out: once its smoke trail ends the
motor has burnt out, and a hard turn away (or a dive into thicker air while it is high up) makes
it bleed speed until it can no longer follow.

## 9. Keyboard shortcuts

The input mapping (chapter 2) shows all keys and can bind them to joystick buttons.

| Group | Action | Key |
|---|---|---|
| **Flight** | Stick (pitch / roll) | Arrow keys (rebindable); double-tap for full deflection |
| | Rudder (yaw) left / right | Q / E |
| | Stick sensitivity | Ins |
| | Throttle up / down | `=` / `-` |
| | Full / idle throttle | Shift `=` / Shift `-` |
| | Gear / autopilot / bay doors / flaps / brakes | 6 / 7 / 8 / 9 / 0 |
| | Eject | Shift F10 |
| **Weapons** | Select weapon / fire / gun | Space / Enter / Backspace |
| | Flare / chaff / IR jammer / ECM / decoy | 1 / 2 / 3 / 4 / 5 |
| **Targeting** | Nearest target ahead | T |
| | Slew the SOI's cursor / lock the target under it / SOI to the other MFD (modern avionics) | keypad 1–9 / keypad Enter / Alt S |
| | Designate new target / next target / clear target | N / B / Shift B |
| | Tracking camera ahead / rear / right / left | `/` / `.` / `,` / M |
| | Tracking camera zoom in / out | `]` / `[` |
| **Displays** | HUD mode / declutter | F2 / V |
| | Bombsight CCIP / CCRP | Alt F2 |
| | MAP / TAC (left MFD) | F3 |
| | Map zoom in / out, north up / heading up | Alt Z / Alt X / Alt F3 |
| | Night vision goggles | F4 |
| | Weapons page / FLIR / ILS / mission orders | F5 / F6 / F9 / F10 |
| **Navigation** | Waypoint page / edit waypoints / reset waypoints | F7 / F8 / Shift F8 |
| | Kneeboard: show, next page, hide (chapter 3.9) | K |
| | Previous / next waypoint | PgUp / PgDn |
| **Views** | Cockpit | F1 |
| | Look ahead / rear / left / right | Shift `/` / Shift `.` / Shift `,` / Shift M |
| | Slot / chase / side / weapon / tactical / inverse tactical | Shift F1 … Shift F6 |
| | Zoom / unzoom / wide angle | Z / X / C |
| | Look around (cockpit) / orbit (external views) | Right mouse button |
| | Zoom in and out (cockpit: up to 6x; external views: the camera distance) | Mouse wheel with the right mouse button held |
| | Recentre head tracking (chapter 3) | Home |
| **Time** | Time acceleration / normal | Shift Z / Shift X |
| **Game** | Settings menu (pauses) / detail / sound level | Esc, Pause, Alt P, gamepad Start / Alt D / Alt V |
| | Training / resupply / quit | Alt T / Alt R / Alt Q |
| | Chat (multiplayer, chapter 16) | Y |

## 10. Aircraft, air defences and ships

Ranges are the values the game uses, in km. **Radar range** is the distance at which the radar
signal reaches zero; **lock range** is the distance within which a missile is launched.
Theaters: LB Libya, PG Persian Gulf, NC North Cape, CE Central Europe, VN Vietnam, ME Middle
East, DS Desert Storm, CU Cuba, KO Korea, UA Ukraine. Enemy fighters also use their guns at
close range.

### 10.1 Aircraft

| | Aircraft | Description |
|---|---|---|
| <img src="images/units/ac_mig17.png" width="200"> | **MiG-17 Fresco** | Older fighter. Pulse radar, 50 km. Armament: AA-2 (IR, 14 km) in DS; guns only in CU and KO. |
| <img src="images/units/ac_mig21.png" width="200"> | **MiG-21 Fishbed** | Light fighter. Pulse radar, 40 km. Armament: AA-2 (IR, 14 km); AA-6 (IR, 50 km) in DS. Theaters: CU, KO, DS. |
| <img src="images/units/ac_mig23.png" width="200"> | **MiG-23 Flogger** | Fighter, the most common type. Pulse radar, 50 km. Armament: AA-2 (IR, 14 km) in CU, KO, DS, LB, ME; AA-8 (IR rear-aspect, 12 km) in CE; AA-6 (radar or IR, 50 km) in VN. |
| <img src="images/units/ac_mig25.png" width="200"> | **MiG-25 Foxbat / MiG-31 Foxhound** | Interceptors (same model). MiG-25: pulse radar 70 km, AA-6 (semi-active radar or IR, 50 km); CE, LB, NC. MiG-31: Doppler radar 100 km, AA-9 (semi-active radar, 82 km) in CE, NC, UA; AA-6 in ME, UA. |
| <img src="images/units/ac_mig29.png" width="200"> | **MiG-29 Fulcrum** | Fighter. Doppler radar, 80 km. Armament: AA-10 (active radar, 64 km) in CE, NC; AA-10 (IR rear-aspect, 64 km) in CU, KO, DS, LB, ME. |
| <img src="images/units/ac_su27.png" width="200"> | **Su-27 Flanker** | Fighter. Doppler radar, 80 km. Armament: AA-10 (IR rear-aspect, 64 km). Theaters: CE, KO, NC, UA. |
| <img src="images/units/ac_su27.png" width="200"> | **Su-34 Fullback / Su-35 Flanker-E** | Ukraine's strike fighter and fighter (drawn with the Su-27 model). Su-34: Doppler radar 80 km, AA-8 (IR rear-aspect, 12 km). Su-35: Doppler radar 100 km, AA-10 (active radar, 64 km). Theater: UA. |
| <img src="images/units/ac_yak38.png" width="200"> | **Yak-38 Forger** | Carrier-based fighter. Pulse radar, 50 km. Armament: AA-2 (IR, 14 km) in VN; AA-8 (IR rear-aspect, 12 km) in NC. |
| <img src="images/units/ac_mirage_f1.png" width="200"> | **Mirage F-1** | Fighter (Iraq). Doppler radar, 100 km. Armament: AA-6 (semi-active radar, 50 km). Theater: DS. |
| <img src="images/units/ac_f5.png" width="200"> | **F-5 Tiger** | Light fighter. Pulse radar, 40 km. Armament: AA-2 (IR, 14 km) in ME; AIM-7 (semi-active radar, 44 km) in PG. |
| <img src="images/units/ac_f4e.png" width="200"> | **F-4E Phantom** | Fighter. Enemy in PG: pulse radar 70 km, AIM-9 (IR rear-aspect, 12 km). Friendly and unarmed in CE, CU, KO, NC, UA, VN. |
| <img src="images/units/ac_f14.png" width="200"> | **F-14 Tomcat** | Fighter (Iran). Doppler radar, 80 km. Armament: AIM-9 (IR rear-aspect, 12 km). Theater: PG. |
| <img src="images/units/ac_il76.png" width="200"> | **IL-76 Mainstay** | Airborne early warning aircraft. Doppler radar, 200 km, sees all around. Armament listed: AA-10 (active radar, 64 km). Theaters: CE, NC, UA. |
| <img src="images/units/ac_an72.png" width="200"> | **An-72 Coaler** | Transport. No radar, unarmed. All theaters; a typical air-to-air mission target. |
| <img src="images/units/ac_mi24.png" width="200"> | **Mi-24 Hind** | Attack helicopter. Flies low, hovers, guns a low and slow player at close range; hunts a downed pilot (chapter 15). |
| <img src="images/units/ac_uh60.png" width="200"> | **UH-60 Black Hawk** | Friendly rescue helicopter (chapter 15). |
| <img src="images/units/ac_geran2.png" width="200"> | **Geran-2** (Shahed-136) | Attack drone (Ukraine). Piston engine and pusher propeller, about 185 km/h, low. No radar, unarmed: it flies straight at its target and dives into it. Shown on the TAC display as an open delta. |
| <img src="images/units/ac_geran3.png" width="200"> | **Geran-3** (Shahed-238) | The jet version: about 600 km/h, flies higher. Otherwise like the Geran-2. |
| <img src="images/units/ac_f18.png" width="200"> | **F/A-18 Hornet** | Friendly fighter. No threat. LB, ME, NC, PG, CE, UA. |
| <img src="images/units/ac_f15.png" width="200"> | **F-15 Eagle** | Friendly fighter. No threat. DS. |
| <img src="images/units/ac_a10.png" width="200"> | **A-10 Thunderbolt II** | Friendly ground attack aircraft. No threat. DS. |

### 10.2 Air defences

SAM sites appear on the map as radar sites. The site model differs by theater; the SAM type
determines radar and missile:

| CE / UA | LB | PG | KO | VN | DS |
|---|---|---|---|---|---|
| <img src="images/units/sam_ce.png" width="150"> | <img src="images/units/sam_lb.png" width="150"> | <img src="images/units/sam_pg.png" width="150"> | <img src="images/units/sam_ko.png" width="150"> | <img src="images/units/sam_vn.png" width="150"> | <img src="images/units/sam_ku.png" width="150"> |

| SAM | Radar | Radar range | Missile guidance | Lock range | Theaters |
|---|---|---|---|---|---|
| **SA-2** | Pulse | 200 | Radar command | 125 | CU, KO, DS, LB, ME, VN |
| **SA-5** | Pulse | 350 | Radar command | 150 | CE, CU, KO, DS, LB, ME, NC, VN |
| **SA-8B** | Pulse | 125 | Semi-active radar | 65 | KO |
| **SA-10** | Doppler | 320 | Active radar | 125 | CE, KO, NC, UA |
| **SA-11** | Pulse | 200 | Semi-active radar | 100 | CE |
| **SA-12** | Doppler | 250 | Active radar | 125 | CE, KO |
| **SA-13** | Pulse | 125 | IR (rear-aspect) | 65 | CE |
| **SA-21** (S-400) | Doppler | 400 | Active radar | 200 | UA |
| **SA-17** (Buk-M2) | Doppler | 250 | Semi-active radar | 110 | UA |
| **SA-22** (Pantsir) | Doppler | 130 | Semi-active radar | 60 | UA |
| **SA-15** (Tor-M2) | Doppler | 125 | Semi-active radar | 50 | UA |
| **Hawk** | Doppler | 175 | Semi-active radar | 125 | PG |
| **Rapier** | Pulse | 75 | Semi-active radar | 65 | PG |
| **Tiger** | Pulse | 65 | Radar command | 30 | PG |

| | Radar | Description |
|---|---|---|
| <img src="images/units/oth_ce.png" width="200"> | **OTH / LPAR radar** | Over-the-horizon and large phased-array early warning radars. Doppler, 500 km. No missiles; a detection raises the theater-wide alarm. CE, UA (OTH), NC (LPAR). |

### 10.3 Low-level air defences

With *Low level* set to *Real* in the menu's Realism section (on for new players), guns and
shoulder-launched missiles defend the enemy's airfields, the targets and the towns.

| Unit | Where | Sees you | Reach |
|---|---|---|---|
| **ZSU-23-4 Shilka** | 1-3 around each enemy airfield and target | Its Gun Dish radar (on the RWR as **A**: your EMV counts as against a SAM), or by eye | 2.5 km, below 2000 m. Fires bursts of 23 mm tracers |
| **S-60** | Batteries of 2-4 guns at airfields and targets (not in a cold war) | By eye only | 6 km, below 4500 m. Its 57 mm shells burst in black puffs around you |
| **MANPADS team** | In and around the towns | By eye only | 5 km, below 3500 m. Fires an SA-14 (IR) at most once a minute; no radar warning |

Seeing by eye takes daylight or moonlight (about 8 km by day, 4 km under a full moon, 2 km on a
moonless night with the *Time of day* switch on) and a clear line of sight through the clouds.
Bombs, missiles and the gun destroy the guns and the teams; a HARM fired at a Shilka's radar on
the HARM page flies at the spot where it stands.

### 10.4 Ships

Ships stand out on the open sea: they are drawn from about 25 km away and keep their detailed
models further out than other objects. Every ship shows on the HSD, where the cursor can lock it.

| | Ship | Description |
|---|---|---|
| <img src="images/units/ship_missileboat.png" width="200"> | **Missile boat / Patrol boat** | Armed with ship-based SAMs. Missile boat: SA-N-4 (Doppler 200 km, semi-active radar, 30 km), SA-N-5 (pulse 150 km, IR, 30 km) in LB, CU, KO; SA-N-6 (Doppler 320 km, active radar, 125 km) in KO; Seacat (pulse 200 km, radar command, 30 km) in PG. Patrol boat: SA-N-5, SA-N-7 (pulse 200 km, semi-active radar, 100 km) in CE, UA. |
| <img src="images/units/ship_destroyer.png" width="200"> | **Destroyer** | Warship. LB, PG, NC, VN, ME, DS, CU, KO. |
| <img src="images/units/ship_krivak.png" width="200"> | **Krivak** | Soviet frigate. NC. |
| <img src="images/units/ship_aegis.png" width="200"> | **Aegis cruiser** | Cruiser. PG. |
| <img src="images/units/ship_typhoon.png" width="200"> | **Typhoon** | Soviet ballistic missile submarine (surfaced). NC. |
| <img src="images/units/ship_tanker.png" width="200"> | **Oil tanker** | Tanker. LB, PG, CU. |
| <img src="images/units/ship_cargo.png" width="200"> | **Cargo ship** | Freighter. NC; merchant ships also appear in CU, ME, DS and KO. |
| <img src="images/units/ship_trawler.png" width="200"> | **Trawler** | Intelligence trawler. CU. |
| <img src="images/units/ship_fishing.png" width="200"> | **Armed fishing boat** | KO. |
| <img src="images/units/ship_carrier.png" width="200"> | **Aircraft carrier** | US carriers (USS America, Nimitz, Kennedy, ...). Friendly; takeoff and landing base in carrier missions (not with *Realistic weapons*). |

Only the missile boats and patrol boats carry air defences. The other ships have no air defences in
the game.

## 11. Mission planner

![Mission planner](images/planner.png)

*Plan route* in the menu opens the theater map with the mission's route. The targets and the
landing base stay where they are; between them you add, move and remove navigation points:

| Action | |
|---|---|
| Click a leg | Add a point there (up to 16 points). |
| Drag a point | Move it. |
| Right click a point | Remove it. |
| Drag the map, mouse wheel | Pan, zoom. |
| RESET | Back to the briefed route. |

Each SAM site shows its missile range as a dotted red circle and, filled in orange, where its radar would
detect you, flown at the selected profile (*Low* 500 ft, *Medium* 5000 ft or *High* 20,000 ft, at
480 kt with the bay doors shut and wings level; banking, opening the doors, the ECM jammer and
speed make the circles larger). Sites that are off the air are drawn faint. The panel lists the
legs with distance and time and how many km of the route a radar would see. In drone defense
missions the drones' starting points are marked.

In flight the INS and the autopilot follow the planned route. When a target is done, the next
waypoint is the point after it, so the planned way home is flown as well. The planned route also
goes along with *Fly in VR*.

## 12. Debrief and career

![Debrief](images/debrief.png)

After every flight the **debrief** shows the result, the statistics, the points and a rating, and
a timeline of the mission (kills, radar and visual identifications, objectives). Points: primary
objective 100, secondary 50, each aircraft or ground target 10 (+5 for an aircraft shot down
before it saw you), each drone 15; −5 for each identification, −10 for each missile hit; landing
+25, crash −50, captured −100. The sum is multiplied by the difficulty (opponents and situation;
training missions count half).

**Flight map.** *FLIGHT MAP* shows your track on the theater map, coloured by how close the
radars came to detecting you (green out of reach, amber close, red detectable), with the radars
that tracked you and markers where they did, where missiles were fired, where you were hit and
where you scored. Drag the slider or press *PLAY* to replay the flight. *Radar exposure* sums it
up. Drag to pan, wheel to zoom; *RESULTS* goes back.

**Tacview.** With *Tacview export* checked in OPTIONS > GAME, every flight is also saved as a Tacview
recording (`.acmi`) in the game's folder for your files under `tacview` (on Windows
`Documents\My Games\Stealth Mission 89\tacview`, on Linux `~/.local/share/stealth-mission-89/tacview`;
chapter 17):
your aircraft, the enemy aircraft and missiles, the radar sites and airfields. The theater is laid
on the globe around one of its real airbases at the game's scale, so the rest of it lies only
roughly where the real places are.

**Tacview real-time telemetry.** With it checked, the game hosts a Tacview session while you
fly: in Tacview choose *Record > Real-Time Telemetry*, address `127.0.0.1`, port `42674`, no
password. Tacview can join before or during the flight; the session lasts one flight (one sortie
in a campaign). For another computer, start the game with `--tacview-address 0.0.0.0` (or
`<ip>:<port>`, optionally `--tacview-password <text>`) and open the firewall.

**Career.** *Career* in the main menu keeps pilot records: enlist a pilot by name, then start a
**tour of duty**, 12 sorties in the theater, situation and opponents chosen beside the records
(fixed while the selected pilot is on a tour). The aircraft, realism and weather
options there apply to every sortie. *Start tour* or *Fly next sortie* shows the sortie's briefing
with PLAN ROUTE, LOADOUT and FLY; NEW SORTIE picks another one, BACK returns to the records. A
sortie starts at its rolled time. In a tour:

- What you destroy stays destroyed for the rest of the tour, and new targets are never ones you
  already hit.
- Points add up to promotions: First Lieutenant 300, Captain 800, Major 1800, Lieutenant Colonel
  3500, Colonel 6000. A finished tour earns 200 extra points.
- A mission can earn the Air Medal (150 points), Distinguished Flying Cross (300), Silver Star
  (500), Air Force Cross (800) or Medal of Honor (1200); hits taken earn the Purple Heart.
- A crash or failed ejection ends the career (killed in action); so does being captured after an
  ejection (prisoner of war).

![Pilot records](images/career.png)

Single missions from the menu do not count for any pilot.

## 13. Dynamic campaign

In the dynamic campaign a whole theater is at war, and the war runs on in real time while you
fly, plan or sit in the menu. You fly sortie after sortie from a mission board that follows the
situation, the front moves as places change hands, and the enemy attacks your side in turn. You
are never killed or lost for good: after every sortie you are back at the board.

### 13.1 Starting a campaign

DYNAMIC CAMPAIGN in the main menu lists the saved campaigns (theater and hours played) with
CONTINUE, IN VR and DELETE, and the options for a new one:

| Group | Options |
|---|---|
| **Campaign** | *Theater*, *Situation* (Cold War, Limited War, Conventional War: how many enemy aircraft fly) and *Opponents* (Green to Elite: the enemy's skill, and how often it attacks). |
| **Conditions** | *Weather*: *Dynamic* or *Clear sky*. *Start time* (default 08:00). *Day and night*: a day lasts 24 h (real time), 2 h, 1 h (the default) or 30 min. |
| **Simulation** | The realism switches (*Damage*, *Guidance*, *Air defence*) and *Crash protection* (chapter 1). |

The aircraft is chosen for each sortie in its briefing. START begins the campaign and saves it at
once. A campaign keeps the rules it was started with, whoever continues or serves it later; the
switches only apply to new campaigns. With dynamic weather the sky changes once or twice a day of
the campaign: mostly fair (clear, scattered or broken cumulus, sometimes an overcast), now and then
thunderstorms; each change takes 4 to 8 minutes, the clouds building up or thinning out and the
light following.

### 13.2 The Ops screen

The campaign opens on the **Ops screen**. Its header shows the score (*BLUE n RED n*: the places
each side holds), the campaign's day and time, the weather now and the next change, and **SAVE &
EXIT** and **OPTIONS**. The footer shows both sides' materiel and income.

| Tab | Shows |
|---|---|
| **MISSIONS** | The mission board (13.4), the details of the selected mission, and under the board the insertions and raids under way with their countdowns, the enemy AWACS's time left and the pilots down. |
| **WAR MAP** | The theater: airfields (squares) and cities (circles) in their side's colour, a white ring where you have fresh reconnaissance, the insertion routes, friendly aircraft and helicopters, the enemy fighters on radar, the known SAM sites, guns and MANPADS, and you. The selected mission's target is circled. Click a unit to watch it live; the wheel zooms, dragging pans. |
| **FORCES** | Both sides side by side: places held, materiel (up to 400) and income per hour and where it comes from, fighters and helicopters on the ground and in the air, where the materiel went since the start, and the prices (13.9). |
| **PLAYERS** | The pilots in the campaign (in coop all players, chapter 16), with what they do, their sorties, points and rank; under it the leaderboard of every pilot who flew in the campaign, and the campaign's rules. |
| **DEBRIEF** | The last sortie's result, kills, points, medal and rank, and what happened meanwhile. |
| **LOG** | The war's events, newest first. |
| **CHAT** | Coop games only. |

The clock does not stop for the in-flight menu. **SAVE & EXIT** saves and leaves: the war stops
until you continue it; aircraft in the air are lost and insertions under way start again from
their base. The campaign is also saved every minute.

### 13.3 Places and the front

The theater is made of **places**: cities and airfields, an airfield and the city beside it
counting as one. Each belongs to the friendly side (blue), the enemy (red) or neither (neutral
places never change hands). Every SAM site and radar belongs to the nearest place of its side.

The **front** is the up to three enemy places nearest to your own, within 150 km (else the
nearest one). The board offers the front places' missions, each with its **threat**: three per
live SAM site plus one per anti-aircraft gun.

**Places change hands only by helicopter insertion.** One helicopter landing within 1.5 km of
the place's airfield (else its city) takes it. A captured place has its cities and airfield
repaired and loses its old SAM sites for good; the new owner's guns move in, and an airfield taken
brings its captor two fighters and two helicopters. The front and the board change at once.

So the war goes: find out what defends a front place (RECON), take down its SAM sites and guns
(SEAD, CAS), and send the helicopters in (INSERTION, ESCORT). Command itself orders a free
insertion against a front place once it has no SAM sites and no guns left (at most every 20
minutes).

### 13.4 The mission board

The board lists up to twelve missions, the urgent ones first (interceptions, defence, rescue,
the AWACS), each with its kind, target, distance from the selected base and threat. It is
rebuilt every minute, after every capture and as soon as a target is destroyed. A mission stays
on the board as long as its situation lasts and is gone when it ends: the target destroyed, the
insertion or raid over, the intelligence gathered.

| Kind | Points | Offered | Achieved when |
|---|---|---|---|
| **RECON** | 100 | Each front place without fresh intelligence. | You photograph its main object. For an hour its defences see you at 0.7 times their range, you see them at twice yours, and the target keys pick its objects from twice as far. |
| **SEAD** | 150 | Front places with SAM sites or radars. | At least half of the listed sites are destroyed, by anyone (AI SEAD too). |
| **CAS** | 120 | Front places with guns and MANPADS; enemy batteries pushed forward near your bases. | You destroy at least half of the listed guns. |
| **STRIKE** | 150 | Front places with an intact airfield. | The airfield is destroyed. It launches no aircraft until it is repaired. |
| **INSERTION** | 150 | Front places within 300 km of a friendly base. | The insertion takes the place. Only in the UH-60 (13.6). |
| **ESCORT** | 120 | Each friendly insertion. | You were there and the place is taken. |
| **INTERCEPT** | 150 | Each enemy insertion. | You were there and it failed. You meet it halfway along its route. |
| **DEFEND** | 150 | Each enemy raid on an airfield, and each drone wave (DRONE DEFENSE). | You were there and every attacker was shot down (or it gave up after an hour). |
| **INTERDICT** | 200 | Up to two enemy places behind the front within 350 km. | The city is destroyed (*Deep strike*: the airfield, where there is no city). |
| **ANTI-SHIP** | 150 | Up to two enemy ships near friendly places. | The ship is sunk. |
| **CAP** | 100 | Where enemy aircraft are near a friendly place or an insertion's target; else over the friendly place nearest the front. | 10 minutes airborne within 20 km of the point; each kill there counts 5 minutes. A friendly aircraft lost there sets the clock back. |
| **CSAR** | 150 | Each pilot down and waiting (13.8). | 2 minutes airborne within 5 km of him, or there when he is rescued; in the UH-60, land by him, wait 30 s for him to board, and land at a friendly base. |
| **AWACS** | 250 | While the enemy AWACS is up. | It is shot down, by anyone. |
| **LOGISTICS** | 100 | Each cargo load for a new air defence site. | You set its crate down by the site's helipad. Only in the UH-60 (13.13). |
| **RESUPPLY** | 100 | Each air defence site that ran out or asked for a resupply. | You hover at the unit for 10 s. Only in the UH-60 (13.13). |

"You were there" means 2 minutes in the air within 20 km of the helicopters or attackers, their
target or the meeting point while it is going on, or an air kill. A sortie that ends while its
insertion, raid or rescue is still going on, with you there, is settled later, and the points come
then.

The mission's details show its description, the defences (SAM sites, threat, whether your
intelligence is fresh), the other missions it makes a package with (13.10), the base to take off
from (any friendly airfield or carrier, bombed or not) with its distance, and the aircraft and
stores. **REQUEST AI SEAD** sends two friendly strike aircraft against one of the place's SAM
sites a minute later (15 materiel and two friendly fighters; one request per place at a time);
their kills count for your SEAD. **ACCEPT MISSION** opens the briefing.

### 13.5 Flying a sortie

The **briefing** has two pages. ROUTE & THREATS plans the route as the mission planner does
(chapter 11), showing only the radars friendly aircraft have found. LOADOUT chooses the aircraft
and its stores (BRIEFED restores the mission's). TAKE OFF puts you on the chosen base's runway.

| Aircraft | Suits |
|---|---|
| **F-117A** | RECON, STRIKE, INTERDICT, ANTI-SHIP, SEAD; CAS. Unseen deep in enemy country. |
| **F-16C** | SEAD (HARM), CAP, DEFEND, INTERCEPT, the AWACS, ESCORT; anything else at a higher radar signature. |
| **AH-1Z** | CAS, ESCORT, INTERCEPT of helicopters: Hellfires and the turret gun (chapters 6.3, 7.5). |
| **UH-60** | INSERTION, CSAR, LOGISTICS and RESUPPLY only, and the only aircraft for them. |

The briefing offers the aircraft you last flew if it suits the mission. You may achieve more
than the mission you accepted: every kill counts, and objectives at the same place make a
package.

**Coming home.** Stop on a friendly runway and the ground crew comes out: **REARM & REFUEL**
fills you up for another mission in the same sortie, **END SORTIE** ends it as landed (+25
points). END SORTIE in the in-flight menu ends it where you are, as aborted (no landing points).
A crash ends the sortie a few seconds later. Either way the debrief follows and you are back at
the board.

### 13.6 Insertions

**ORDER INSERTION** on an INSERTION mission sends four helicopters from the friendly base nearest
the place (within 300 km); they lift off three minutes later. Your orders cost nothing, and you
always have helicopters for them. One insertion per place at a time, only against front places.
The board then offers its ESCORT: fly cover for them, or lead them yourself in the UH-60 (**FLY
THE UH-60**, which joins the insertion under way or starts one at once with your wingmen).

The insertion fails when all its helicopters are shot down, or when they are still in the air
after 90 minutes. When they come within 80 km of an enemy airfield the enemy sends a fighter
after them. Flying the UH-60 yourself, land within 1.5 km of the place and stay on the ground for
15 seconds; ending your sortie leaves the others flying on.

### 13.7 The enemy fights back

| Action | How often (green .. elite) | What happens |
|---|---|---|
| **Insertion** | Every 75 / 50 / 35 / 25 min, warned 5 min ahead | Four helicopters against one of the two weakest friendly places near the front (fewest SAM sites and guns). A place where its insertions failed in the last two hours counts as stronger. INTERCEPT it. |
| **Raid** | Every 60 / 40 / 28 / 18 min, warned 2 min ahead | Two strike aircraft, escorted by two fighters that take off 90 s earlier, against one of the two friendly airfields nearest it. A hit destroys the airfield: it launches no AI aircraft until repaired, though you can still take off from it. DEFEND it. |
| **Patrols** | Every 6 min (3 with the AWACS up) | Fighters from the airfields near the front, at most 2 to 4 in the air by the situation, sent at the insertions and raids, else at the friendly place nearest the front. |
| **AWACS** | Every 90 min for an hour | Orbits behind the front; its patrols go up twice as often and are sent straight at you. Shoot it down (AWACS mission). |
| **Forward batteries** | When nothing is on the board near your bases | Anti-aircraft guns 15-35 km from a friendly base near the front (at most two): a CAS mission. |
| **Drone waves** (Ukraine) | Every 80 / 55 / 40 / 30 min, warned 3 min ahead | Six Geran drones at a friendly city or airfield; the first one through destroys it. DEFEND (DRONE DEFENSE) it (chapter 14). |

Your own side defends too: an airfield scrambles one or two fighters when enemy aircraft come
within 40 km, and Patriot batteries stand at airfields well behind the front.

Everything destroyed is repaired by its owner, as long as it has the materiel: SAM sites, cities
and ships after 45 minutes, airfields after an hour. Guns and MANPADS teams destroyed are
replaced one per place every 20 minutes.

### 13.8 Pilots down (CSAR)

Ejecting over enemy country costs the friendly side 30 materiel and a fighter until you are out
(never your own next sortie). Once the parachute is down a UH-60 comes for you as in a single
mission (chapter 15), in real time: picked up, the materiel and the fighter come back; captured,
they stay lost.

**SKIP EXTRACTION** (or END SORTIE) does not wait for it: the sortie ends and the board posts a
CSAR mission instead. Any player who holds the air within 5 km of the pilot for 2 minutes, or
brings him home in the UH-60, gets him out. Otherwise, after 20 minutes, a rescue helicopter may
still get through (four times in ten); if it doesn't, the pilot is captured. Either rescue gives
back what he cost.

### 13.9 Materiel and aircraft

Each side's AI fights with the materiel its places bring in and with the aircraft it has left.
Each side starts with 150 materiel and holds at most 400. The income per hour is 20, plus 2 per
place, plus 3 per intact city.

| Costs | Materiel |
|---|---|
| Insertion (AI) | 20 |
| Raid | 15 |
| AI SEAD (yours) | 15 |
| AWACS sortie | 20 |
| Patrol or scramble, per fighter | 2 |
| Repair: airfield / ship / SAM site / city / gun | 25 / 20 / 15 / 10 / 2 |
| New fighter / helicopter | 15 / 4 |

Aircraft shot down are gone; every hour each side buys new ones while its materiel lasts
(helicopters first). An enemy that runs out can't repair, attack or replace its losses.
INTERDICT missions strike its income: a city destroyed costs it 20 materiel at once and its 3 an
hour until rebuilt. An AWACS shot down costs it 60. Only the AI runs short: your aircraft, bases
and insertion orders never do.

### 13.10 Points, ranks and packages

Every sortie earns points: the objective's (100 to 250 by kind, once it is settled), 50 per
aircraft and 20 per ground kill (SAM sites, guns, MANPADS), and 25 for landing at a friendly base
and ending the sortie there. They earn the career's ranks over the campaign, and its medals for
one sortie's points (chapter 12); the debrief shows them.

**Packages**: objectives of two different kinds achieved at one place within 15 minutes (SEAD and
STRIKE, say, or CAS and the ESCORT of an insertion) earn every sortie of the package 100 more
points. The mission's details name the other missions at the place. CAP, ANTI-SHIP, AWACS, CSAR
and battery CAS have no place and make no packages.

### 13.11 Winning and losing

The war is won when the enemy holds no airfield, and lost when the friendly side holds neither an
airfield nor a carrier. The end screen shows the result and every pilot's record; sorties in the
air may still finish. Then start a new campaign from the menu (a dedicated server starts the next
one by itself, chapter 16).

A campaign continued from its save goes on where it was, except that aircraft in the air are
back on the ground, insertions and raids under way wait to start again, and the AWACS is down.

### 13.12 Air defence operations

Your side can build its own air defence: **Gepard** anti-aircraft guns, **Patriot** batteries
and **drone command centers** (13.14). A player designates the spot, UH-60 pilots fly in the
cargo (13.13), and anyone can man the finished site.

| Site | Cargo loads | Armament |
|---|---|---|
| **Gepard** | 1 | Twin 35 mm guns firing airburst rounds, out to 2 km. 640 rounds ready, 640 in stock. |
| **Patriot** | 3 | A radar with launchers of four missiles each, out to 80 km. 8 missiles in stock. |
| **Drones** | 1 | 4 Stork, 8 FP-2 and 6 interceptor drones (13.14). |

**Designating a site**

1. On the Ops screen's WAR MAP or AIR DEFENSE tab, press **GEPARD**, **PATRIOT** or **DRONES**
   under *New site*.
2. Move the pointer over the map. A green circle shows where the site may go; a red one names
   the reason it can't (not friendly territory, water, too close to a town, airfield, site or
   other units, no open ground, no friendly airfield to fly the cargo from).
3. Click to designate it. Right click or Esc cancels.

A helipad appears with the loads delivered and the time left (`PATRIOT 1/3 23:41`). All loads
must arrive within 24 hours, or the helipad is gone. At most three sites can wait for cargo at
once. Once the last load is in, the unit stands beside the pad.

**Manning a site**

The AIR DEFENSE tab lists your side's sites: who mans each one (*auto* when nobody does), the
aircraft on its radar, what is ready and its stock. **ENTER** takes you to its console,
**LEAVE** back to the Ops screen. Unmanned Patriots and Gepards fight on their own.

![The Patriot console](images/patriot_console.png)

The Patriot and Gepard consoles share one layout:

* **Camera** (top): the tracking camera on the radar's or the Gepard's mast, following the
  lock. **FLIR** switches colour / thermal, **ZOOM+** / **ZOOM-** zoom.
* **Air picture** (bottom left): the tracks around the site, north up, friendly and hostile
  symbols with their type and a heading line. **RNG+** / **RNG-** set its range.
* **Board** (bottom right): **TRACKS** lists the radar's tracks (click one to select it);
  **SITE** shows the launchers, missiles ready, the stock, reloads and **REQUEST RESUPPLY**.
* **Engaging**: select a track, **LOCK** it (**DROP** lets go), then **FIRE**.
* **Signal monitor / target data** (top right): see below.

**Patriot.** Missiles go out to 80 km. **GUIDED** missiles are steered by the radar (hard to
decoy, one target at a time); **SEMI** missiles home on the radar's illumination (several targets
at once, easier to decoy). **L1**-**L3** pick the launcher to fire from; **RLD1**-**RLD3**
reload an empty one from the stock (4 minutes).

<table>
<tr>
<td width="50%"><img src="images/patriot_radar_signal.png" alt="Patriot radar signal monitor"></td>
<td>The Patriot's <b>radar signal monitor</b> shows what the radar receives:
<ul>
<li>Top: the echo strength against range, with the lock's echo standing out of the noise.</li>
<li>Bottom left: the locked target's Doppler spectrum: its speed line, engine or rotor
modulation beside it.</li>
<li>Bottom middle: the lock's angle error in the beam (it wanders more on a weak echo).</li>
<li>Right: signal to noise, range, closing (CLOS) or opening (OPEN) speed, Doppler shift,
guidance mode and the battery's missiles in the air.</li>
</ul></td>
</tr>
</table>

![The Gepard console](images/gepard_console.png)

**Gepard.** It fires bursts of airburst rounds out to 2 km. Without a lock the stick slews the
turret by hand and the camera looks along the guns, marking each aircraft in reach with its
lead. **RLD1** refills the guns from the stock.

<table>
<tr>
<td width="50%"><img src="images/gepard_target_data.png" alt="Gepard target data display"></td>
<td>The Gepard's <b>target data display</b> shows the locked track:
<ul>
<li>Top: the type, friend or foe, the bearing and the shells' time of flight.</li>
<li>Middle: the track number, range, altitude and speed.</li>
<li>Left: the state lamps: tracking (TRK), slewed by hand (MAN), firing, rounds ready (RDY),
refilling (RLD).</li>
<li>Bottom: whether the target is in reach (2,000 m).</li>
</ul></td>
</tr>
</table>

### 13.13 Logistics

UH-60 pilots keep the air defence going: they fly the cargo for new sites and the resupplies for
existing ones. Both are missions on the board (UH-60 only, 100 points). If no player takes one,
an AI helicopter flies it.

**Cargo for a new site (LOGISTICS, sling load)**

Each load of a designated site (13.12) is a **LOGISTICS** mission (`PATRIOT 2/3 - <site>`). Its
crate waits beside the runway of the nearest friendly airfield; after 30 minutes without a
player, an AI UH-60 brings it.

1. Take the mission and take off in the UH-60 from the crate's airfield.
2. Hover low over the crate: below 50 ft, slower than 15 kt, within 8 m of it. *Cargo hooked*:
   the winch lifts it on its rope under the helicopter.
3. Fly to the helipad. The crate swings on the rope: accelerate and turn gently, slow down
   early, and let the swing settle before you set it down.
4. Lower the crate onto the ground within 90 m of the helipad. *Cargo set down*: the load is
   delivered.

Any UH-60 can hook any waiting crate, and other players see it hanging under your helicopter.

**Resupply (RESUPPLY)**

A site that runs out of stock orders a resupply by itself; at a Patriot or Gepard console you
can also press **REQUEST RESUPPLY** on the SITE page. It becomes a **RESUPPLY** mission:

1. Take the mission and fly the UH-60 to the unit.
2. Hover within 50 m of it, under 100 ft and slower than 15 kt, for 10 seconds.
3. The load goes down and the site's stock is full again.

After 5 minutes without a player, an AI helicopter flies it from the nearest airfield.

### 13.14 Drone warfare

Players can build drone command centers and fly drones from their console, much like they man
the Patriot and Gepard sites. A center launches three types of drones, up to 5 in the air at
once. Waypoints are placed and moved by clicking on the map, also while the drone is flying;
some drone types have their own special waypoints. Any drone can be flown by hand at any time.

**Building and manning a center**: designate a DRONES site (13.12) and fly in its one load (13.13).
It starts with 4 Storks, 8 FP-2s and 6 interceptors and orders a resupply when it runs out. On
the AIR DEFENSE tab, **ENTER** takes you to its console, **LEAVE** back to the Ops screen.

![The drone command center's console](images/drone_console.png)

The desk screen is the ground control station: the drone list on the left, the map with the
drones, their routes, friendly units and the enemy air defences your drones have found. The
large screen shows the selected drone's camera after you press **MAIN**. The screen at the top
right shows each drone's battery.

**Launching a drone**:

1. Press **NEW DRONE** and pick the type: **STK**, **FP2** or **INT**.
2. Click the map to add waypoints (up to 12). Clicking a contact ends the route on it.
3. Select a waypoint to change it: **TYPE** changes its type, **ALT+**/**ALT-** its height,
   **DEL** deletes it. Drag it to move it.
4. Press **LAUNCH**. The drone leaves the rail and flies its route; the next drone can launch
   15 seconds later.

Click a drone or its row to select it. **NEXT** makes the selected waypoint the next one,
**MAIN** shows its camera on the large screen, **FWD** switches between the gimbaled camera and
the forward flight camera. **MANUAL** (or **MAN** on the panel) flies it with the stick and
throttle; **AUTO** sends it back on its route. The panel's **FLIR**, **ZOOM**, **TRACK** (hold
the camera on the point under its crosshair) and **SLAVE** (back to the next waypoint) work the
camera like a targeting pod. A drone with an empty battery goes down and is lost. Enemy air
defences only see a drone within 2 km.

<table>
<tr>
<td width="33%"><img src="images/units/ac_stork.png" alt="Stork-100"><br>
<b>Stork-100</b> surveillance drone</td>
<td width="33%"><img src="images/units/ac_fp2.png" alt="FP-2"><br>
<b>FP-2</b> attack drone</td>
<td width="33%"><img src="images/units/ac_interceptor.png" alt="Interceptor"><br>
<b>Interceptor</b> quadrotor</td>
</tr>
</table>

**Stork-100 (surveillance)**: slow, with about 2.5 hours of battery. It detects enemy air
defence units and SAM sites within 6 km and marks them on the map for follow-up missions,
including for the other players. Its special **LOITER** waypoint flies a wide circle, ideal for
continuous surveillance of an area. It has a forward flight camera and a gimbaled zoom camera.

**FP-2 (attack)**: a simple fixed-wing one-way attack drone that is flown into its target and
disables most targets with one hit. On its special **TERMINATE** waypoint it dives into the
ground point (put it on a contact the Stork found).

**Interceptor**: a rocket-shaped quadrotor, up to 600 km/h but with only about 8 minutes of
battery, less at high throttle. Thanks to its speed it easily catches Geran-2 drones; jet
Geran-3s **can** be intercepted, but this is very difficult. Plan a **LOITER** point in the
target's path at its height: the map shows aircraft and drones in the air with their heights.
It only has the forward camera.

**Auto lock-on**: an interceptor locks on by itself to the nearest aircraft or drone that comes
within 500 m and about 20° of its nose. The camera marks the locked target and the message
line reports it. Once locked, it steers itself; just open the throttle and let it close in. It
loses the lock beyond 1.5 km. It cannot tell friend from foe, so keep it away from friendly
aircraft. A crash into the ground also sets off its warhead, friendly units nearby included.

### 13.15 Tips

- Clear a front place's SAM sites and guns and Command sends the helicopters in for free; or
  order the insertion yourself and fly its escort.
- STRIKE the enemy's airfields near the front: for an hour they send no patrols or raids.
- The enemy goes for your weakest places. Keep the guns and SAM sites there standing, and
  intercept its insertions: a failed one keeps it away from that place for two hours.
- Fly RECON first on a well defended place: with fresh intelligence its defences see you later
  and you see them sooner.
- INTERDICT when the enemy's materiel runs high: without it, it can't repair or attack.
- Land and REARM & REFUEL rather than ending the sortie in the air: you keep the landing points
  and can fly the next mission at once.

## 14. Drone defense (Ukraine)

![Closing in on a Geran-3](images/geran_chase.png)

Russian Geran attack drones are on their way to a Ukrainian city or the South Ukraine nuclear power
plant. They start already in the air and arrive one after another, about 9 to 25 minutes after
take-off. You take off from the nearest airfield with air-to-air missiles, the gun and two
Paveways (with *Realistic weapons*: the Paveways and extra fuel).

- The **primary objective** is met when every drone is down or through and no more than the
  briefed number got through. A drone that hits the nuclear power plant fails the mission.
- The **secondary objective** is the launch site the drones came from.
- The early warning network shows every drone on the TAC display (open deltas) and on the map.
  They have no radar and do not fight back, but the Geran-2 is small, slow and low: slow down
  behind it and use the gun or a Sidewinder.
- Opponents and situation set the size of the wave (4 to 12) and the share of the faster jet
  Geran-3.

<table>
<tr>
<td width="40%"><img src="images/mfd_map_drones.png" alt="Drones on the MAP page"><br>
The drones (crosses) on the MAP page, on their way to Poltava.</td>
</tr>
</table>

In Ukraine strike missions a few drones may also cross the theater; shooting them down earns points.

## 15. Ejection and rescue

![Waiting for the rescue](images/csar.png)

After a good ejection over enemy territory (Shift-F10, or the ejection handle) the mission goes
on from the ground. The view follows the pilot down and on the ground, and a panel shows the
rescue's progress, with buttons for the signal flare and time acceleration (click them; in VR,
point and pull the trigger).

- A **UH-60** comes from the nearest friendly airfield's alert area. Close in it cannot find you
  without a **signal flare** (the flare key, 1): fire one when it is near (you have three).
- **Mi-24 Hinds** from the nearest enemy airfield search the area. A flare also shows them where you
  are, and a Hind passing close may spot you. If one gets to you first you are captured.
- Wait too long (15 minutes against green opponents, 9 against elite ones) and you are captured.
- Time acceleration (Shift-Z) runs four times faster while you wait. A dynamic campaign runs in
  real time: there the panel has **SKIP EXTRACTION** (Alt-Q) instead, which ends the sortie and
  leaves the rescue to the mission board (chapter 13.8).

Picked up: the mission ends like a parachute landing. Over friendly territory or the sea the
mission ends when the parachute lands, as before.

## 16. Multiplayer

Up to 64 players fly the dynamic campaign (chapter 13) together (coop: everybody is on the friendly side).
One computer runs the war, the **server**: either a player's game (HOST A GAME) or a dedicated
server without a game of its own. The others join it over UDP.

### Your name

The MULTIPLAYER screen asks for your name first; the other players know you by it, and the
campaign keeps your sorties under it: join again under the same name and they are yours again. In
the headset, click a text field with the laser pointer and a keyboard opens under it; press its
keys with the trigger and DONE when finished. Keys type at the blinking cursor: click in the text
to place it, or move it with ← and →. The career's pilot name uses the same field.

### Joining

JOIN A GAME lists the **public servers** at the top: each one's name, the pilots in it out of
how many it takes, the theater, the situation and the campaign's clock. JOIN on a row connects
to it (REFRESH fetches the list again; a server of another game version is greyed out). The
same list is on the web at https://sm-master.playspoon.com, where a server opens up for its
settings, the places each side holds and who flies what.

Or type the server's IP address or host name, with the port after a colon if it is not the
default 28989 (`192.168.1.20`, `sm89.example.org:29000`, `[2001:db8::1]:28989`), and CONNECT.
Either way you come to the campaign's Ops screen; your aircraft waits at a friendly base.

### Hosting

HOST A GAME serves a new campaign (theater, opponents, situation) or one of your saved ones, and
you fly in it. Give the server a name (the others see it when they join; without one it is
"<your name>'s campaign"), choose the UDP port and how many players may join, and whether the
game is on the public server list (on by default); to play over the internet,
forward that UDP port to your computer in your router. The campaign is saved on your computer as
when you play alone.

### Who decides what

The server decides how the war is fought, for everybody: realistic damage, realistic guidance, no
crashes, time of day (moonlit nights shorten the enemy's visual range), realistic F-117A
weapons and friendly fire, and of course the campaign's own options. The weather is the same for
everybody: the same clouds in the same places, and the same lightning strikes. Each player keeps his own cockpit (classic or
modern), flight model, avionics, HUD, shaders, shadows and lights, and picks his aircraft (F-117A
or F-16) for each sortie on the briefing's LOADOUT page.

### On the Ops screen

The Ops screen and the campaign itself work as when you play alone (chapter 13). In coop:

- **PLAYERS** lists everybody in the campaign: who is on the mission board, who flies what, the
  sorties flown and objectives achieved, points and rank, and each player's ping; under it the
  leaderboard keeps every pilot's record, also of those who have left (a newcomer taking a free
  slot does not take the record with it), and the line at the bottom names the campaign's rules.
- **CHAT** has every message of the game so far (newest at the bottom, the list scrolls) and a
  field to write one: Enter or SEND sends it. In the headset a keyboard opens under the field.
- **SAVE & EXIT** (top right, under the clock) leaves the campaign; as the host it also ends the
  game for the others. On a server that is not yours the button reads **DISCONNECT**: the war goes
  on on the server, and nothing is saved on your computer.
- Every player takes his own missions from the board and may order insertions. Kills and
  objectives count for the player whose weapon it was. Players taking off from the same base line
  up in a staggered formation on the runway: the first in the middle, far enough down the runway
  for the others, then rows of two behind him.

### Chat

Who joins and leaves, and what the players write, comes up on a panel next to the HUD's lower
right corner, with the sender's name; it grows with the messages (the last six, the newest at the
bottom) and fades out once they are a few seconds old. In flight, press **Y** to write: a field
opens under the panel, which stays up while you type, and every key types into it (none of the
other keys works meanwhile). Enter sends the message, Esc cancels it. What you send shows in the
panel too.

In the air the other players are friendly contacts on the MFD; their missiles and tracers show
like yours. Another player can be locked and shot at like an enemy: with the server's **friendly
fire** rule the hits damage him, without it missiles burst harmlessly and rounds just spark.

Your cannon hits are judged on your computer and checked by the server; missiles and bombs are
flown by the server.

If your connection drops during a sortie, your aircraft waits for you for 30 seconds: join again
under the same name and fly on. After that the sortie ends as aborted.

### Dedicated server

The game's executable runs a server without a window with `--dedicated`:

```
stealth-mission-89 --dedicated --theater ukraine --name "Friday night ops" --port 28989 --friendly-fire
```

| Switch | |
|---|---|
| `--name <text>` | Server name, shown to the players. |
| `--port <n>` | UDP port (default 28989). |
| `--max-players <n>` | 1 to 64 (default 64). |
| `--theater <id>` | central_europe, cuba, desert_storm, korea, libya, middle_east, north_cape, persian_gulf, ukraine, vietnam. Default: the saved campaign's, else central_europe. |
| `--roe <0..2>`, `--opponents <0..3>`, `--seed <n>`, `--start-hour <h>` | A new campaign's options. |
| `--sky-ratio <x>` | The sky runs x times faster than real time (default 24: a day per hour). |
| `--clear-sky` | A new campaign without weather (else it changes once or twice a day). |
| `--classic-damage`, `--classic-guidance`, `--classic-time` | Classic damage, guidance and a fixed time of day instead of the default realistic damage, realistic guidance and moving sun and moon. |
| `--no-crashes`, `--realistic-weapons`, `--friendly-fire` | The other rules. |
| `--save <file>` | The campaign file (default `campaigns/dedicated.json` in the game's folder for your files, chapter 17). |
| `--send-rate <hz>` | How often the moving world goes to the players, 1 to 15 times a second (default 7.5). More is smoother; less leaves bandwidth for more players. |
| `--master <url>` | The server list to be on (default `https://sm-master.playspoon.com`; reported every 30 seconds). |
| `--no-master` | Not on the server list: players join by address. |
| `--plain` | Log lines instead of the console screen. |
| `--no-rotate` | When the campaign is won or lost, keep it instead of starting the next one. |

The server saves the campaign (which side holds which place, what is destroyed, the players'
records, its rules) every minute and when it stops, and goes on from the save when it starts
again, under the rules it was started with: the rule switches apply to a new campaign. Started
with another `--theater` than the saved campaign's, it begins a new campaign and keeps the old
save next to it (`dedicated.json.<theater>.bak.json`).

While no player is connected the war waits: the clock stops, and the enemy takes nothing while
nobody can defend it. The first player to join sets it going again. The server list shows such
a server as *waiting for players*.

When the war is won or lost, every player sees the result with each pilot's record and a
countdown. Two minutes later (at once, if nobody is connected) the server keeps the finished
campaign as `dedicated.json.<theater>.won-<time>.json` (or `lost-`) and starts a new one in the
same theater. The players are disconnected and join the new campaign again.

In a terminal the server shows a console screen: its uptime, the places and airfields each side
holds, the players and what they do, and the recent events with their times (UTC). **q**, Esc or
Ctrl-C stops it (it saves first). Without a terminal (a service, `--plain`) it writes the events
as log lines; SIGTERM or Ctrl-C stops it the same way.

## 17. Your files

The game keeps everything it writes for you in one folder:

| System | Folder |
|---|---|
| Windows | `Documents\My Games\Stealth Mission 89` |
| macOS | `~/Library/Application Support/stealth-mission-89` |
| Linux, Steam Deck, Steam Frame | `~/.local/share/stealth-mission-89` |

In it: `saves` (single missions saved in flight), `pilots` (career), `campaigns` (dynamic
campaigns), `config` (your options and key bindings), `tacview` (Tacview recordings) and the log
of the last start. Copy the folder to keep your progress or to take it to another computer. With
Steam Cloud, the saves, pilots and campaigns follow you from one computer to another, also
between Windows, macOS and Linux; your options and bindings stay on each computer.
