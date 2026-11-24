//! Every number that sets the pace of a campaign, in one place. Distances are map units
//! (15.625 m), times campaign seconds (real time while the game runs).

/// Cities and airfields of the same name this close are one place (8 km).
pub const PLACE_MERGE: i32 = 0x200;
/// A SAM site or radar belongs to the nearest place this close (12 km).
pub const SITE_ASSOCIATION: i32 = 0x300;
/// Enemy places this close to a friendly one are on the front (150 km).
pub const FRONT_RANGE: i32 = 0x2600;
/// Front places that get a full set of offers, nearest first.
pub const FRONT_PLACES: usize = 3;
/// Offers on the board at most.
pub const BOARD_SIZE: usize = 12;
/// Coop scaling: up to CREW_FREE players the enemy keeps its tempo (the players push it back);
/// beyond, it grows with the crew, n / CREW_FREE (1.5 for twelve, 2 for sixteen), at most
/// CREW_SCALE_MAX. The count it follows rises by one player in CREW_RISE seconds, falls by one
/// in CREW_FALL. Each player beyond the first adds CREW_OFFERS to the board, CREW_OFFERS_MAX at
/// most.
pub const CREW_FREE: f64 = 8.0;
pub const CREW_SCALE_MAX: f64 = 3.0;
pub const CREW_RISE: f64 = 3.0 * 60.0;
pub const CREW_FALL: f64 = 10.0 * 60.0;
pub const CREW_OFFERS: usize = 2;
pub const CREW_OFFERS_MAX: usize = 12;
/// Per step of the crew's pace beyond one: more enemy patrols up at once, more escorts with
/// a raid.
pub const CREW_PATROLS: f64 = 2.0;
pub const CREW_ESCORTS: f64 = 1.0;
/// Points of a planned route at most.
pub const ROUTE_POINTS: usize = 16;
/// The board is rebuilt this often (and after every capture).
pub const BOARD_REFRESH: f64 = 60.0;
/// Enemy ships this close to a friendly place are anti-ship targets (120 km).
pub const SHIP_RANGE: i32 = 0x1E00;
/// AAA within this distance of a place defends it (CAS targets, insertion threat; 6 km).
pub const AAA_RANGE: i32 = 0x180;
/// Enemy flights this close to a friendly place or an insertion call for CAP (60 km).
pub const CAP_TRIGGER: i32 = 0xF00;

/// Recon: intelligence stays valid this long; the defences' detection range with it, and the
/// player's spotting range.
pub const INTEL_SECONDS: f64 = 3600.0;
pub const INTEL_DETECTION: f32 = 0.7;
pub const INTEL_SPOTTING: f32 = 2.0;

/// The enemy repairs destroyed sites and airfields after this long.
pub const REPAIR_SITE: f64 = 45.0 * 60.0;
pub const REPAIR_AIRFIELD: f64 = 60.0 * 60.0;
/// A side repairs this many sites at once, one more for every REPAIR_CITIES_PER_CREW intact
/// cities, REPAIR_CREWS_MAX at most (the neutrals' all at once). A site destroyed again within
/// REPAIR_MEMORY of its last repair takes REPAIR_SLOWDOWN times longer each time, at most
/// REPAIR_SLOWDOWN_MAX times.
pub const REPAIR_CREWS: usize = 1;
pub const REPAIR_CITIES_PER_CREW: usize = 8;
pub const REPAIR_CREWS_MAX: usize = 4;
pub const REPAIR_MEMORY: f64 = 4.0 * 3600.0;
pub const REPAIR_SLOWDOWN: f64 = 1.5;
pub const REPAIR_SLOWDOWN_MAX: f64 = 3.0;
/// Insertions: helicopters per insertion, minutes from the order until they lift off (time to
/// take off for an escort), landing this close to the place takes it (1.5 km).
pub const INSERTION_HELOS: usize = 4;
pub const INSERTION_LEAD: f64 = 180.0;
pub const CAPTURE_RADIUS: i32 = 0x60;
/// Insertion helicopters cover this many map units a second on average, take-off and landing
/// included (~310 km/h; for the intercepts' arrival times).
pub const HELO_SPEED: f64 = 5.5;
/// Insertions and raids still in the air this long after take-off are called home.
pub const INSERTION_TIMEOUT: f64 = 90.0 * 60.0;
pub const RAID_TIMEOUT: f64 = 60.0 * 60.0;
/// The campaign sends an insertion on its own when a front place's threat is at most this, and
/// not more often than this.
pub const AUTO_INSERT_THREAT: i32 = 0;
pub const AUTO_INSERT_EVERY: f64 = 20.0 * 60.0;
/// A prepared operation's insertion goes in at once with this many guns left at most (and
/// not where one failed within FAILED_MEMORY).
pub const OPERATION_THREAT: i32 = 2;
/// Insertions fly from bases at most this far from the target (helicopter range, 300 km: every
/// theater's enemy airfields are in reach of one another's).
pub const INSERTION_RANGE: i32 = 0x4B00;
/// Weight of a live SAM site against one AAA unit in the threat estimate.
pub const SAM_THREAT: i32 = 3;
/// The enemy weighs a place where its insertions failed lately (this long) as if each failure
/// were one more SAM site, and picks among this many of the weakest.
pub const FAILED_MEMORY: f64 = 2.0 * 3600.0;
pub const ENEMY_TARGET_CHOICE: usize = 2;
/// What a friendly place is worth to the enemy's insertions against its defences (a SAM site
/// weighs `SAM_THREAT`, a gun 1): an intact airfield, every intact city.
pub const AIRFIELD_VALUE: i32 = 4;
pub const CITY_VALUE: i32 = 2;

/// Enemy air: a patrol takes off this often from a live airfield, at most this many fly
/// (plus one per ROE level); interceptors scramble against insertions this close to a base.
pub const PATROL_EVERY: f64 = 6.0 * 60.0;
pub const PATROLS: usize = 2;
/// The front's momentum counts what happened this long back: a place taken weighs
/// MOMENTUM_CAPTURE, a site destroyed 1; MOMENTUM_SHOWN either way shows as gaining.
pub const MOMENTUM_WINDOW: f64 = 2.0 * 3600.0;
pub const MOMENTUM_CAPTURE: u8 = 5;
pub const MOMENTUM_SHOWN: i32 = 3;
/// A place whose sites the other side destroyed draws its owner's patrols this long.
pub const PRESSURE_MEMORY: f64 = 30.0 * 60.0;
pub const SCRAMBLE_RANGE: i32 = 0x1400;

/// Friendly air: a CAP of one or two fighters scrambles from an airfield when enemy aircraft
/// come this close to it (40 km, the bubble its fighters engage in:
/// `stealth_sim::campaign::CAP_BUBBLE`) and orbits this far out towards them (8 km); after a
/// CAP fighter is shot down that airfield sends none for this long.
pub const AI_CAP_TRIGGER: i32 = 0xA00;
pub const AI_CAP_STATION: i32 = 0x200;
pub const AI_CAP_RESPAWN: f64 = 5.0 * 60.0;
/// A place under air attack (`Place::air_hits`) is guarded by the CAP of the nearest airfield
/// within this distance of it (120 km).
pub const AI_CAP_REACH: i32 = 0x1E00;
/// Patriot batteries defend friendly airfields at least this far behind the front (60 km), at
/// most this many (plus one per ROE level), standing this far from the airfield (4-6 km).
pub const PATRIOT_REAR: i32 = 0xF00;
pub const PATRIOT_BATTERIES: usize = 2;
pub const PATRIOT_OFFSET: i32 = 0x100;
/// A Patriot battery's launchers stand this close to its radar at most (map units).
pub const PATRIOT_SPREAD: i32 = 0x20;

/// The enemy's offensive, by the opponents' skill (0..3): an insertion against the weakest
/// friendly place on its front this often, announced this long before the helicopters lift
/// off (time to scramble an intercept).
pub const ENEMY_INSERT_EVERY: [f64; 4] = [75.0 * 60.0, 50.0 * 60.0, 35.0 * 60.0, 25.0 * 60.0];
pub const ENEMY_INSERTION_LEAD: f64 = 300.0;
/// Enemy raids on friendly airfields, by the opponents' skill: this often, this many strike
/// aircraft, announced this long before they take off, from a base at most this far from the
/// target (300 km).
pub const RAID_EVERY: [f64; 4] = [60.0 * 60.0, 40.0 * 60.0, 28.0 * 60.0, 18.0 * 60.0];
pub const RAID_AIRCRAFT: usize = 2;
/// Fighters escorting a raid: they take off this long before the strike aircraft and draw
/// the defenders' CAP.
pub const RAID_ESCORTS: usize = 2;
pub const RAID_ESCORT_LEAD: f64 = 90.0;
pub const RAID_LEAD: f64 = 120.0;
pub const RAID_RANGE: i32 = 0x4B00;
/// Finished raids are kept this long (the log and the debriefs have them).
pub const RAID_KEEP: f64 = 3600.0;
/// Enemy SEAD: a friendly SAM battery that shoots down an enemy aircraft draws a raid of
/// `RAID_AIRCRAFT` with anti-radiation missiles (escorted, paid like a raid), no sooner than
/// this after the last one, by the opponents' skill.
pub const SEAD_EVERY: [f64; 4] = [30.0 * 60.0, 20.0 * 60.0, 15.0 * 60.0, 10.0 * 60.0];

/// Guns and MANPADS teams of a place: one destroyed comes back this long after the last one
/// came back (or was lost).
pub const REPAIR_GUN: f64 = 20.0 * 60.0;

/// The campaign is over: the result stays on the screen this long before a dedicated server
/// starts the next one.
pub const END_SCREEN_SECONDS: f64 = 120.0;

/// Mission success: share of the defences destroyed (SEAD, CAS), time on station or kills
/// (CAP), within this radius of the CAP point (20 km).
pub const SUPPRESS_SHARE: f32 = 0.5;
pub const CAP_SECONDS: f64 = 600.0;
pub const CAP_KILLS: u32 = 2;
pub const CAP_RADIUS: i32 = 0x500;
/// A player on CAP keeps the enemy's new raids and insertions off places this close to his
/// CAP point (30 km).
pub const CAP_COVER: i32 = 0x780;
/// The AWACS diverts a player in the air to an urgent mission this close (120 km).
pub const DIVERT_RANGE: i32 = 0x1E00;
/// An enemy airfield struck holds back the enemy's next raid this long at least; a city its
/// next convoy and insertion (stores and transport gone).
pub const STRUCK_AIRFIELD_DELAY: f64 = 15.0 * 60.0;
pub const STRUCK_CITY_DELAY: f64 = 10.0 * 60.0;
/// Escorts, intercepts, airfield defences and rescues need the pilot there: this long airborne
/// within this radius (20 km) of the helicopters, raiders or pilot while it goes on (an air
/// kill counts too).
pub const PRESENCE_SECONDS: f64 = 120.0;
pub const PRESENCE_RADIUS: i32 = 0x500;

/// Campaign log entries kept.
pub const LOG_SIZE: usize = 200;

/// War materiel: what a side starts with and holds at most; what it gets an hour (a base
/// amount, for every place it holds, for every intact city of its places).
pub const MATERIEL_START: f64 = 150.0;
pub const MATERIEL_MAX: f64 = 400.0;
pub const INCOME_BASE: f64 = 20.0;
pub const INCOME_PLACE: f64 = 1.0;
pub const INCOME_CITY: f64 = 5.0;
/// The friendly side's income (its AI flies the war the players do not) is this many times
/// more with up to BLUE_FEW_PLAYERS_FULL players on, the bonus shrinking to nothing with
/// BLUE_FEW_PLAYERS_UNTIL (on top of the players' tempo, which it follows like the enemy's).
pub const BLUE_FEW_PLAYERS: f64 = 1.5;
pub const BLUE_FEW_PLAYERS_FULL: f64 = 2.0;
pub const BLUE_FEW_PLAYERS_UNTIL: f64 = 4.0;
/// A theater that starts the friendly side outnumbered (the enemy's places and cities weighed
/// as income against its own, beyond UNDERDOG_EVEN times as many) helps it by the ratio to the
/// power UNDERDOG_EXP, at most UNDERDOG_MAX: its income, its fighters and their purchases, and
/// the enemy's offensive slower by the root. Losing ground helps it again: COMEBACK for all its
/// places lost (in proportion), at most COMEBACK_MAX, on its income, its offensive and repairs.
pub const UNDERDOG_EVEN: f64 = 1.25;
pub const UNDERDOG_EXP: f64 = 0.5;
pub const UNDERDOG_MAX: f64 = 2.5;
/// Each friendly place's garrison has this many more tanks and MANPADS teams for every step of
/// that beyond 1 (rounded down).
pub const UNDERDOG_GARRISON: f64 = 1.4;
pub const COMEBACK: f64 = 2.5;
pub const COMEBACK_MAX: f64 = 2.0;
/// A destroyed city costs its side this much at once (stores and factories gone).
pub const CITY_LOSS: f64 = 30.0;
/// What the AI pays: repairs by kind of site, a gun or MANPADS team, a fighter sortie (patrol,
/// CAP), a raid, an insertion the campaign orders itself (the players' orders are free).
pub const REPAIR_COST_AIRFIELD: f64 = 25.0;
pub const REPAIR_COST_SAM: f64 = 15.0;
pub const REPAIR_COST_CITY: f64 = 10.0;
pub const REPAIR_COST_SHIP: f64 = 20.0;
pub const GUN_COST: f64 = 2.0;
pub const SORTIE_COST: f64 = 2.0;
pub const RAID_COST: f64 = 15.0;
/// One enemy raid in so many goes for a site the players built; one in so many airfield raids
/// comes with a feint at another airfield, which turns back this long after take-off.
pub const SITE_RAID_ONE_IN: i32 = 3;
pub const FEINT_ONE_IN: i32 = 4;
pub const FEINT_TURN: f64 = 4.0 * 60.0;
pub const INSERTION_COST: f64 = 20.0;

/// The AI's aircraft on the ground: each side starts with these fighters (the enemy more by
/// its skill) and helicopters, holds at most twice as many, and buys some every hour while
/// its materiel lasts. Taking an airfield brings its captor some of each.
pub const FIGHTERS_START: u32 = 16;
pub const ENEMY_FIGHTERS_PER_SKILL: u32 = 4;
pub const HELICOPTERS_START: u32 = 12;
pub const RESUPPLY_EVERY: f64 = 3600.0;
pub const RESUPPLY_FIGHTERS: u32 = 6;
pub const RESUPPLY_HELICOPTERS: u32 = 4;
/// The enemy buys this many an hour by the opponents' skill (the friendly side the above).
pub const ENEMY_RESUPPLY_FIGHTERS: [u32; 4] = [4, 6, 8, 10];
pub const ENEMY_RESUPPLY_HELICOPTERS: [u32; 4] = [3, 4, 6, 8];
/// Materiel above this is spent on aircraft (up to the limits) at the hourly purchase too.
pub const MATERIEL_RESERVE: f64 = 200.0;
pub const FIGHTER_COST: f64 = 15.0;
pub const HELICOPTER_COST: f64 = 4.0;
pub const CAPTURED_FIGHTERS: u32 = 2;
pub const CAPTURED_HELICOPTERS: u32 = 2;
/// Helicopters a player's insertion order always gets, whatever is left (the command keeps a
/// reserve for the players).
pub const RESERVE_HELICOPTERS: usize = 2;

/// Deep strike: enemy places off the front within this range of a friendly base (350 km) get
/// strike offers on their cities and airfields, this many of them.
pub const DEEP_RANGE: i32 = 0x5780;
pub const DEEP_TARGETS: usize = 2;

/// A pilot ejected over enemy territory waits this long for a rescue; it takes a player this
/// long airborne within this radius of him (5 km) to cover the helicopter in. Without one the
/// campaign's own helicopter gets him out at this chance. A pilot down there costs the friendly
/// side this much materiel and a fighter until he is rescued (lost for good if captured).
pub const RESCUE_WINDOW: f64 = 20.0 * 60.0;
pub const RESCUE_SECONDS: f64 = 120.0;
pub const RESCUE_RADIUS: i32 = 0x140;
pub const RESCUE_AI_CHANCE: i32 = 40;
pub const CAPTURE_COST: f64 = 30.0;

/// Flown in the helicopter: an insertion's troops get out in this many seconds landed within
/// `CAPTURE_RADIUS` of the target; a downed pilot boards in this many seconds landed within
/// this radius of him (about 200 m), and is home landed this near a friendly base.
pub const UNLOAD_SECONDS: f64 = 15.0;
pub const BOARD_SECONDS: f64 = 30.0;
pub const BOARD_RADIUS: i32 = 13;
pub const HOME_RADIUS: i32 = 0x40;

/// Points: each aircraft and ground kill, an aircraft brought home (on top of the objective's,
/// see `MissionKind::points`).
pub const POINTS_AIR_KILL: i32 = 50;
pub const POINTS_GROUND_KILL: i32 = 20;
pub const POINTS_LANDED: i32 = 25;
/// A package: objectives of two or more kinds achieved at one place within this time; each
/// sortie of it earns this bonus (once).
pub const PACKAGE_WINDOW: f64 = 15.0 * 60.0;
pub const PACKAGE_BONUS: i32 = 100;
/// AI SEAD support a player asks for: what it costs the friendly side, and how soon its strike
/// aircraft take off.
pub const SUPPORT_COST: f64 = 15.0;
pub const SUPPORT_LEAD: f64 = 60.0;

/// The enemy's AWACS: it goes up this often (when none is up) for this long, orbiting this far
/// behind its front (80 km); it costs this much to fly, and this much to lose.
pub const AWACS_EVERY: f64 = 90.0 * 60.0;
pub const AWACS_ENDURANCE: f64 = 60.0 * 60.0;
pub const AWACS_STANDOFF: i32 = 0x1400;
pub const AWACS_COST: f64 = 20.0;
pub const AWACS_LOSS: f64 = 60.0;

/// Enemy forward batteries: when no target on the board is within this of a player base
/// (50 km, a short helicopter flight), the enemy pushes a battery of guns forward this far from
/// the base nearest its front (15 to 35 km), on land and clear of every place, site and player
/// base (8 km). It looks into it this often, keeps this many at most, and pays this much for
/// one; the battery's guns stand within this radius of it (1.5 km).
pub const BATTERY_REACH: i32 = 0xC80;
pub const BATTERY_NEAR: i32 = 0x3C0;
pub const BATTERY_FAR: i32 = 0x8C0;
pub const BATTERY_CLEAR: i32 = 0x200;
pub const BATTERY_EVERY: f64 = 10.0 * 60.0;
pub const BATTERIES: usize = 2;
pub const BATTERY_COST: f64 = 6.0;
pub const BATTERY_RADIUS: i32 = 0x60;
/// The ground round a gun of a battery (map units, about 100 m) that must not be too steep.
pub const GUN_GROUND: i32 = 6;

/// Drone waves (the Ukraine theater): this often by the opponents' skill, this many drones,
/// announced this long before launch, at a target this far from their base at most (300 km),
/// at this cost.
pub const DRONES_EVERY: [f64; 4] = [80.0 * 60.0, 55.0 * 60.0, 40.0 * 60.0, 30.0 * 60.0];
pub const DRONE_WAVE: usize = 6;
pub const DRONE_LEAD: f64 = 180.0;
pub const DRONE_RANGE: i32 = 0x4B00;
pub const DRONE_COST: f64 = 12.0;

/// Resupply: an AI helicopter flies a request nobody has taken for this long (s), and a
/// player's drop takes this long hovering at the unit (s).
pub const SUPPLY_AI_WAIT: f64 = 5.0 * 60.0;
pub const SUPPLY_HOVER_SECONDS: f64 = 10.0;

/// New air defence sites the players designate: loads of cargo a Gepard, a Patriot battery, a
/// drone command center and a NASAMS battery (its Sentinel radar, then the launchers) need, how long the helipad waits for them (s), how many sites may wait at once, how long an
/// open load waits for a player before an AI helicopter brings it (s).
pub const DEPLOY_LOADS_GEPARD: u8 = 1;
pub const DEPLOY_LOADS_PATRIOT: u8 = 3;
pub const DEPLOY_LOADS_DRONES: u8 = 1;
pub const DEPLOY_LOADS_NASAMS: u8 = 2;
pub const DEPLOY_TIMEOUT: f64 = 24.0 * 3600.0;
pub const DEPLOY_OPEN_MAX: usize = 3;
pub const DEPLOY_AI_WAIT: f64 = 30.0 * 60.0;
/// A unit destroyed with an enemy aircraft this close (map units, 8 km) was hit from the air,
/// and so was its side's place within this distance of it (12 km).
pub const AIR_ATTACK_RANGE: i32 = 0x200;
pub const AIR_ATTACK_PLACE: i32 = 0x300;
/// The friendly command orders air defence sites of its own where enemy aircraft keep killing
/// its units: at a place that lost this many in a row to air attack, while it has this much
/// materiel (a NASAMS battery from the second amount on, a Gepard below) and pays the site's
/// cost; none within this distance (map units, 12 km) of another friendly missile battery or
/// new site, at most this many of its own standing or waiting, and always leaving the players
/// room to designate one.
pub const AUTO_SITE_HITS: u8 = 3;
pub const AUTO_SITE_MATERIEL: f64 = 100.0;
pub const AUTO_SITE_NASAMS_MATERIEL: f64 = 200.0;
pub const AUTO_SITE_COST_GEPARD: f64 = 20.0;
pub const AUTO_SITE_COST_NASAMS: f64 = 40.0;
pub const AUTO_SITE_COVER: i32 = 0x300;
pub const AUTO_SITES_MAX: usize = 4;
/// Cover for the friendly side's supply and cargo helicopters (`protect.rs`): enemy guns and
/// MANPADS this close to the way there (map units, 6 km: an S-60's reach) send a pair of
/// gunships ahead, the helicopter following this long after them (s); a live enemy SAM site
/// this close to the way (8 km: its reach at helicopters) holds it, and so do guns with no
/// gunships to send, at most this long (s); enemy fighters about the way send this many
/// fighters to fly with it.
pub const HELO_GUN_CLEAR: i32 = 0x180;
pub const HELO_SAM_CLEAR: i32 = 0x200;
pub const HELO_COVER_LEAD: f64 = 4.0 * 60.0;
pub const HELO_HOLD_MAX: f64 = 30.0 * 60.0;
pub const HELO_ESCORTS: usize = 2;
/// Materiel a load of cargo not in yet costs when a player fast-tracks a site's construction.
pub const DEPLOY_FAST_TRACK_COST: f64 = 20.0;
/// Map units: the unit stands this far north of its helipad; a Gepard's, a Patriot battery's
/// (its launchers stand around the radar) and a drone command center's ground, kept clear of
/// the other units.
pub const DEPLOY_UNIT_OFFSET: i32 = 3;
pub const DEPLOY_FOOTPRINT_GEPARD: i32 = 0x10;
pub const DEPLOY_FOOTPRINT_PATRIOT: i32 = 0x30;
pub const DEPLOY_FOOTPRINT_DRONES: i32 = 0x20;
pub const DEPLOY_FOOTPRINT_NASAMS: i32 = 0x30;
/// Map units: a site keeps this far from a place (its town or airfield) and from a world object.
pub const DEPLOY_PLACE_CLEAR: i32 = 0x80;
pub const DEPLOY_SITE_CLEAR: i32 = 0x40;
/// A load is in when its crate is set down this close to the helipad (map units, about 90 m;
/// and in metres).
pub const LOGISTICS_RADIUS: i32 = 6;
pub const CRATE_DROP_METRES: f32 = 90.0;
/// A crate is hooked by a helicopter this close to its base (map units, about 3 km: the client
/// saw the hover).
pub const CRATE_HOOK_RANGE: i32 = 0xC0;
/// Metres from the helipad where the AI's loads are set down.
pub const CRATE_SLOT_METRES: f32 = 9.0;

/// Convoys: the longest drive to an enemy place (map units, 120 km: some 100 min on the road).
pub const CONVOY_MAX_ROUTE: i32 = 0x1E00;
/// A side sends its next convoy this long (s) after the last one was over (the enemy's by
/// skill), the first this long into the campaign.
pub const CONVOY_EVERY: f64 = 25.0 * 60.0;
pub const ENEMY_CONVOY_EVERY: [f64; 4] = [45.0 * 60.0, 35.0 * 60.0, 25.0 * 60.0, 20.0 * 60.0];
/// What a convoy costs (materiel).
pub const CONVOY_COST: f64 = 15.0;
/// A place's garrison stands within this of it (map units, as its guns); a convoy whose head
/// is this close to a place without one takes it (2.5 km).
pub const GARRISON_RADIUS: i32 = 0x180;
pub const CONVOY_NEAR: i32 = 0xA0;
/// A convoy still on its way after this long (s) is called off.
pub const CONVOY_TIMEOUT: f64 = 4.0 * 3600.0;
/// A place's lost armour is replaced one vehicle at a time, this often (s), at this cost.
pub const REPAIR_ARMOUR: f64 = 30.0 * 60.0;
pub const ARMOUR_COST: f64 = 4.0;
/// A side whose convoy cleared a place without troops tries this long (s) to send helicopters.
pub const CONVOY_INSERTION_WAIT: f64 = 30.0 * 60.0;

/// AI missions (`ai_missions.rs`): the AI flies a mission of a side's board this long after it
/// was posted (and again this long after its last flight came back without it done), no more
/// than this many of a side's at once, each at this cost.
pub const AI_MISSION_WAIT: f64 = 5.0 * 60.0;
pub const AI_MISSIONS_MAX: usize = 4;
pub const AI_MISSION_COST: f64 = 10.0;
/// Close air support offers on the enemy's board at most (on friendly places and convoys).
pub const RED_CAS_OFFERS: usize = 2;
/// Aircraft of a flight: a recon flies alone, every other mission a pair.
pub const AI_RECON_AIRCRAFT: usize = 1;
pub const AI_FLIGHT_AIRCRAFT: usize = 2;
/// The helicopters go for targets this close to their base (40 km), and this far (80 km) while
/// the friendly fighters are out.
pub const AI_HELO_RANGE: i32 = 0xA00;
pub const AI_HELO_REACH: i32 = 0x1400;
/// Below this many fighters the friendly side buys fighters before helicopters (its air cover).
pub const BLUE_FIGHTER_FLOOR: u32 = 6;
/// An offer whose last AI flight was shot down to the last aircraft for nothing waits this
/// long (s) before the AI tries it again.
pub const AI_FAILED_WAIT: f64 = 60.0 * 60.0;
/// A heavy air defence (the offer's threat: a SAM site weighs `SAM_THREAT`, a gun 1), and
/// heavy fighter opposition (enemy aircraft in the air within `AIR_THREAT_RANGE`, 64 km).
pub const HEAVY_SAM_THREAT: i32 = 2 * SAM_THREAT;
pub const HEAVY_AIR_THREAT: usize = 2;
pub const AIR_THREAT_RANGE: i32 = 0x1000;
/// A flight still out this long is written off (its aircraft fly on without it).
pub const AI_MISSION_TIMEOUT: f64 = 45.0 * 60.0;
/// Done missions are forgotten this long after (a new flight may still go after `wait`).
pub const AI_MISSION_MEMORY: f64 = 30.0 * 60.0;
/// The enemy's recon of a friendly place is good this long.
pub const RED_INTEL_SECONDS: f64 = 3600.0;
/// A side's AI flights (and the enemy's raids) take off with no more than this many aircraft
/// in `AI_LAUNCH_WINDOW` (15 min), one at a time, `AI_STAGGER` apart.
pub const AI_LAUNCH_WINDOW: f64 = 15.0 * 60.0;
pub const AI_AIRCRAFT_PER_WINDOW: usize = 6;
pub const AI_STAGGER: f64 = 10.0;

/// Munitions (`munitions.rs`): a cargo flight takes off this long (s) after a base ran out of
/// a munition, the other side's interceptors this long after it was scheduled; it brings up to
/// this share of a base's full stock of each munition the production base has.
pub const CARGO_WAIT: f64 = 10.0 * 60.0;
pub const CARGO_INTERCEPT_WAIT: f64 = 2.0 * 60.0;
pub const CARGO_SHARE: f64 = 0.5;
/// A cargo flight saved in the air takes off again this long (s) after the campaign is loaded.
pub const CARGO_RELAUNCH: f64 = 60.0;
/// After a base's cargo aircraft was shot down, the next one goes this long (s) after.
pub const CARGO_LOST_COOLDOWN: f64 = 30.0 * 60.0;
/// The other side's SAM sites this close to where interceptors would wait for a cargo flight
/// (map units, ~64 km) make its threat: the AI does not fly a heavy one.
pub const CARGO_HUNT_SAM_RADIUS: i32 = 0x1000;
/// A cargo flight to a base this close to a place of the other side (map units, ~150 km) gets
/// an escort of this many fighters from that base.
pub const CARGO_ESCORT_RANGE: i32 = FRONT_RANGE;
pub const CARGO_ESCORTS: usize = 2;
/// A cargo route passing this close to a place of the other side (map units, ~40 km) crosses
/// its land: the production base is picked to avoid that.
pub const CARGO_ROUTE_CLEAR: i32 = 0xA00;
/// A cargo aircraft that comes down this close to a base of its side (map units, ~8 km)
/// delivers there.
pub const CARGO_LAND_RADIUS: i32 = 0x200;
/// Supply by road (`roads.rs`): a base within this many metres of a road's spur end is
/// supplied by truck; up to this many convoys on the road to it at once, a new one at least
/// this long (s) after the last and this long (s) before it sets off, when something there
/// (with what is coming) is below this share of a full stock. A convoy has this many trucks,
/// and a pickup with a machine gun at this chance.
pub const ROAD_BASE_REACH: f64 = 4000.0;
pub const SUPPLY_CONVOYS_MAX: usize = 3;
pub const SUPPLY_SPACING: f64 = 15.0 * 60.0;
pub const SUPPLY_WAIT: f64 = 5.0 * 60.0;
pub const SUPPLY_LOW: f64 = 0.6;
pub const SUPPLY_TRUCKS: usize = 3;
pub const TECHNICAL_CHANCE: f64 = 0.5;
/// Supply convoys set off at least this long (s) after each other, so that those to bases
/// along the same road do not drive in each other.
pub const SUPPLY_GAP_S: f64 = 45.0;
/// The enemy's fleet of supply trucks, and how many new ones an hour make up for those lost.
pub const TRUCK_FLEET: usize = 36;
pub const TRUCK_REPLACE: f64 = 0.5;
/// The production base keeps up to this many times a base's full stock.
pub const PRODUCTION_CAP: f64 = 3.0;
/// The enemy's SAM sites and guns refill their magazines this often (s), from the bases with
/// stock within this reach (map units, ~150 km), nearest first.
pub const MAGAZINE_REFILL_EVERY: f64 = 60.0;
pub const MAGAZINE_SUPPLY_RANGE: i32 = 0x2580;
/// A base keeps this many SAM missiles for each SAM site it supplies (two reloads), and gun
/// rounds for each gun.
pub const SAM_STOCK_PER_SITE: f64 = 8.0;
pub const AAA_STOCK_PER_GUN: f64 = 2400.0;
/// The production base makes this many an hour for each of the theater's sites (guns).
pub const SAM_LONG_PER_SITE_HOUR: f64 = 1.5;
pub const SAM_MEDIUM_PER_SITE_HOUR: f64 = 1.5;
pub const SAM_SHORT_PER_SITE_HOUR: f64 = 2.0;
pub const AAA_PER_GUN_HOUR: f64 = 300.0;

// --- Missile strikes (missile_strikes.rs) ---

/// Reach of the Iskander (450 km), the FP-7 (200 km) and the cruise missiles (1500 km), map
/// units.
pub const ISKANDER_RANGE: i32 = 28_800;
pub const FP7_RANGE: i32 = 12_800;
pub const CRUISE_RANGE: i32 = 96_000;
/// Campaign seconds between the enemy's Iskander strikes and Kh-101 strikes, by its skill.
pub const ISKANDER_EVERY: [f64; 4] = [5.0 * 3600.0, 4.0 * 3600.0, 3.0 * 3600.0, 2.5 * 3600.0];
pub const KH101_EVERY: [f64; 4] = [7.0 * 3600.0, 5.5 * 3600.0, 4.5 * 3600.0, 3.5 * 3600.0];
/// What a strike costs the enemy's operations.
pub const MISSILE_STRIKE_COST: f64 = 15.0;
/// A TEL drives this far out of its base (map units, 8 km) and sets up for this long (s); a
/// friendly launch site gets ready for this long.
pub const TEL_DRIVE: i32 = 512;
pub const TEL_PREPARE: f64 = 12.0 * 60.0;
pub const SITE_PREPARE: f64 = 15.0 * 60.0;
/// A launcher told to fire raises its erector first (s, about as the simulation does).
pub const LAUNCHER_ERECT_SECS: f64 = 25.0;
/// Missiles a TEL fires, and a Tu-95 (which launches a quarter of the way from its base).
pub const ISKANDER_SALVO: u8 = 2;
pub const KH101_SALVO: u8 = 6;
pub const KH101_LAUNCH_FRAC: f64 = 0.25;
/// Ended strikes kept (for the log).
pub const MISSILE_STRIKES_KEPT: usize = 20;
/// The friendly launch sites: the cargo loads one needs, how far (map units, 15 km) from any
/// airfield its spot is picked, how often (campaign s) a new one may be posted, and the rest
/// between a site's salvos.
pub const DEPLOY_LOADS_LAUNCH_SITE: u8 = 2;
pub const LAUNCH_SITE_AIRFIELD_CLEAR: i32 = 960;
pub const LAUNCH_SITE_EVERY: f64 = 2.0 * 3600.0;
pub const LAUNCH_SITE_REST: f64 = 40.0 * 60.0;
/// Missiles a friendly launch site fires in a salvo: FP-7s, Flamingos.
pub const FP7_SALVO: u8 = 2;
pub const FLAMINGO_SALVO: u8 = 1;
/// The enemy's Iskander sites: the cargo loads one needs (each flown in by a Mi-24, this long
/// after the last one left or was lost, campaign s).
pub const DEPLOY_LOADS_ISKANDER_SITE: u8 = 3;
pub const RED_CARGO_GAP: f64 = 10.0 * 60.0;

// ------------------------------------------------------------------- strategic facilities

/// In-game days each kind of facility is out of action once destroyed (`facilities.rs`).
pub const FACTORY_DAYS: f64 = 3.0;
pub const HEADQUARTERS_DAYS: f64 = 2.0;
pub const POWER_PLANT_DAYS: f64 = 2.0;
pub const FUEL_DEPOT_DAYS: f64 = 1.0;
pub const AMMO_STORAGE_DAYS: f64 = 1.0;
/// The sky's clock ratio an in-game day is measured with, at least and at most: a campaign with
/// the sky at real time (1) or stopped (0) still repairs its facilities within hours, not days.
pub const FACILITY_SKY_MIN: f64 = 6.0;
pub const FACILITY_SKY_MAX: f64 = 96.0;
/// How far from its anchor (map units) each kind is built: the factories and the headquarters
/// from the side's rearmost place, an ammo storage from its airfield, a fuel depot from its
/// city, a power plant from the place it is built at.
pub const FACTORY_FROM: (i32, i32) = (0x200, 0x500);
pub const AMMO_STORAGE_FROM: (i32, i32) = (0x50, 0x90);
pub const FUEL_DEPOT_FROM: (i32, i32) = (0x100, 0x180);
pub const POWER_PLANT_FROM: (i32, i32) = (0x180, 0x280);
/// Fuel depots a side has at most (one at each of its cities), and its power plants: one for
/// every `PLACES_PER_POWER_PLANT` places, at least and at most.
pub const FUEL_DEPOTS_MAX: usize = 6;
pub const PLACES_PER_POWER_PLANT: usize = 4;
pub const POWER_PLANTS: (usize, usize) = (2, 3);

// ------------------------------------------------------------------------- drone production

/// A drone command center's full load of Storks, FP-2s, interceptors and X-12s (the
/// simulation's `station::*_STOCK`), and what the friendly production base makes of each an
/// hour while the drone factory stands.
pub const DRONE_CENTER_STOCK: [u32; 4] = [4, 8, 6, 4];
pub const DRONES_PER_HOUR: [f64; 4] = [1.0, 3.0, 2.0, 1.0];
/// The enemy's Gerans: a base's full stock (one wave), and what its production base makes an
/// hour by skill (a little more than its waves use).
pub const GERAN_FULL: f64 = 6.0;
pub const GERAN_PER_HOUR: [f64; 4] = [6.0, 8.0, 11.0, 14.0];
