//! The scenes behind the main menu take turns: each is shown for `SHOT_SECONDS`, the first the
//! night on Tabor's apron, the others in a shuffled order; every `LOOK_EVERY` scene changes the
//! weather, the time of day and the theater change. A scene that has to be somewhere else (the
//! apron, the sea by a carrier) moves to a theater of its own under the same sky. The scenes
//! play without sound.
//!
//! `SM89_MENU_SHOT=<name>[,<hours>[,<sky>[,<theater>]]]` holds one scene (screenshots; the
//! names are `Shot::name`'s, the sky one of clear, scattered, broken, overcast, storm);
//! `SM89_MENU_SNAP=<seconds>,...` with `SM89_MENU_SNAP_PNG=<path>` saves `<path>_N.png` at those
//! times of the held scene's first run, then quits. `SM89_MENU_ROLL=<n>` fixes its random choices (its aircraft, weapon, place);
//! `SM89_MENU_SHOT_AT=<start>[,<stop>]` starts it that far in and stops its clock at `stop`
//! (else it plays again from the start at its end).

use bevy::prelude::*;
use stealth_mission::weather::Sky as SkyKind;

/// How long each scene is shown (seconds).
pub const SHOT_SECONDS: f32 = 5.0;
/// The weather, the time of day and the theater change every so many scenes.
pub const LOOK_EVERY: usize = 4;

/// The scenes.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Shot {
    /// The F-117A on Tabor's apron at night, the tower beside it (the original menu scene).
    Apron,
    /// The carrier against the sun, aircraft and helicopters taking off and landing.
    Carrier,
    /// A red SAM site, its launchers and radar.
    SamSite,
    /// A blue aircraft firing one of its weapons, seen from outside.
    Strike,
    /// The AC-130 orbiting and firing its side guns.
    Gunship,
    /// The Patriot battery's console engaging a target.
    Patriot,
    /// The Gepard's console engaging a target.
    Gepard,
    /// The MH-60R hovering with its dipping sonar down.
    Dip,
    /// The MH-60R's sonar console, its waterfalls showing buoys' contacts.
    Sonar,
    /// The C-5 dropping cargo.
    Airdrop,
    /// A Black Hawk hooking up a crate on its sling and leaving with it.
    Sling,
    /// An interceptor drone, seen from outside.
    Interceptor,
    /// The drone command center's console: an interceptor chasing a Geran.
    DroneChase,
    /// The drone command center's console: an FP-2 diving at a tank column.
    DroneDive,
    /// The F/A-18E's cockpit, close on the MFD steering a SLAM-ER to its target.
    Slam,
    /// The F-16's cockpit, the pilot's head following an enemy fighter, an AIM-9 off the
    /// helmet sight.
    Helmet,
    /// The Tornado in low level flight, seen from outside.
    Tornado,
    /// The Comanche's cockpit, close on the MFD: the pod's picture aiming the gun at a target.
    PodGun,
    /// The Comanche's cockpit, close on the RCS display, maneuvering through SAM-covered land.
    Rcs,
    /// A random aircraft's forward cockpit view while it maneuvers.
    Cockpit,
    /// A random aircraft flying, the camera orbiting close.
    Orbit,
    /// An aircraft on final to a runway, followed by a chase plane.
    Approach,
    /// A Super Hornet on final to the carrier, followed by a chase plane.
    CarrierApproach,
}

/// Every scene, the apron first.
pub const SHOTS: [Shot; 23] = [
    Shot::Apron,
    Shot::Carrier,
    Shot::SamSite,
    Shot::Strike,
    Shot::Gunship,
    Shot::Patriot,
    Shot::Gepard,
    Shot::Dip,
    Shot::Sonar,
    Shot::Airdrop,
    Shot::Sling,
    Shot::Interceptor,
    Shot::DroneChase,
    Shot::DroneDive,
    Shot::Slam,
    Shot::Helmet,
    Shot::Tornado,
    Shot::PodGun,
    Shot::Rcs,
    Shot::Cockpit,
    Shot::Orbit,
    Shot::Approach,
    Shot::CarrierApproach,
];

/// Where a scene has to be: on Tabor's apron, at sea by the carrier, anywhere.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    Tabor,
    Sea,
    Any,
}

impl Shot {
    pub fn name(self) -> &'static str {
        match self {
            Shot::Apron => "apron",
            Shot::Carrier => "carrier",
            Shot::SamSite => "sam",
            Shot::Strike => "strike",
            Shot::Gunship => "gunship",
            Shot::Patriot => "patriot",
            Shot::Gepard => "gepard",
            Shot::Dip => "dip",
            Shot::Sonar => "sonar",
            Shot::Airdrop => "airdrop",
            Shot::Sling => "sling",
            Shot::Interceptor => "interceptor",
            Shot::DroneChase => "chase",
            Shot::DroneDive => "dive",
            Shot::Slam => "slam",
            Shot::Helmet => "helmet",
            Shot::Tornado => "tornado",
            Shot::PodGun => "podgun",
            Shot::Rcs => "rcs",
            Shot::Cockpit => "cockpit",
            Shot::Orbit => "orbit",
            Shot::Approach => "approach",
            Shot::CarrierApproach => "trap",
        }
    }

    pub fn place(self) -> Place {
        match self {
            Shot::Apron => Place::Tabor,
            Shot::Carrier | Shot::Dip | Shot::CarrierApproach => Place::Sea,
            _ => Place::Any,
        }
    }
}

/// The theater on which the apron stands.
pub const TABOR_THEATER: &str = "central_europe";
/// The theaters with a carrier.
pub const SEA_THEATERS: [&str; 8] = [
    "libya",
    "persian_gulf",
    "north_cape",
    "vietnam",
    "middle_east",
    "desert_storm",
    "cuba",
    "korea",
];

/// What a block of scenes shares: the theater, the local time and the moon's phase, the
/// weather. `classic` is the original night on the apron (the classic palette, held still).
#[derive(Clone, Debug)]
pub struct Look {
    pub theater: &'static str,
    pub hours: f64,
    pub phase: f64,
    pub weather: stealth_mission::Weather,
    pub classic: bool,
}

impl Look {
    /// The original scene's: a clear night with a full moon on Tabor's apron.
    fn classic() -> Self {
        Look {
            theater: TABOR_THEATER,
            hours: 23.0,
            phase: 0.5,
            weather: stealth_mission::Weather::default(),
            classic: true,
        }
    }

    /// A random time of day and weather on `theater`: mostly by day, now and then at dawn or
    /// dusk or by night (under a moon at least half full, so the scene shows).
    fn roll(theater: &'static str, sky: Option<SkyKind>, hours: Option<f64>) -> Self {
        let hours = hours.unwrap_or_else(|| match rand::random_range(0..10) {
            0..5 => rand::random_range(8.0..16.5),
            5 => rand::random_range(6.2..7.2),
            6 | 7 => rand::random_range(17.0..18.3),
            _ => rand::random_range(20.5..27.0) % 24.0,
        });
        let phase = rand::random_range(0.3..0.7);
        let index = stealth_formats::ALL_THEATERS.iter().position(|&t| t == theater).unwrap_or(0);
        // The rolled weather, less often the storms and overcasts that hide the scene.
        let sky = sky.unwrap_or_else(|| {
            [SkyKind::Clear, SkyKind::Clear, SkyKind::Scattered, SkyKind::Scattered]
                .into_iter()
                .chain(SkyKind::ALL)
                .nth(rand::random_range(0..4 + SkyKind::ALL.len()))
                .unwrap_or(SkyKind::Clear)
        });
        let weather = stealth_mission::Weather::roll(rand::random(), index, Some(sky), None);
        Look { theater, hours, phase, weather, classic: false }
    }
}

/// Which scene is on, for how long, what comes next and the block's look.
#[derive(Resource)]
pub struct Director {
    pub shot: Shot,
    /// Seconds into the scene.
    pub t: f32,
    /// The scene's things are spawned (its systems spawn them while this is false).
    pub spawned: bool,
    /// A random number for the scene's choices (aircraft, weapon, side).
    pub roll: u32,
    pub look: Look,
    /// The look changed: the theater, the palette and the sky follow it.
    pub look_changed: bool,
    order: Vec<Shot>,
    next: usize,
    /// Scene changes so far, and the looks rolled.
    changes: usize,
    pub looks: usize,
    /// One scene held (`SM89_MENU_SHOT`, played again and again; in the headset the apron).
    held: bool,
    /// The held scene's start and where its clock stops (`SM89_MENU_SHOT_AT`).
    from: f32,
    stop: Option<f32>,
}

impl Director {
    /// The desktop's scenes, in turn; in the headset only the apron.
    pub fn new(vr: bool) -> Self {
        if let Some(d) = Self::held() {
            return d;
        }
        let mut d = Director {
            shot: Shot::Apron,
            t: 0.0,
            spawned: false,
            roll: rand::random(),
            look: Look::classic(),
            look_changed: true,
            order: Vec::new(),
            next: 1,
            changes: 0,
            looks: 0,
            held: vr,
            from: 0.0,
            stop: None,
        };
        d.order = cycle();
        d
    }

    fn held() -> Option<Self> {
        let v = std::env::var("SM89_MENU_SHOT").ok()?;
        let mut it = v.split(',');
        let name = it.next()?.trim();
        let shot = SHOTS.into_iter().find(|s| s.name() == name)?;
        let hours = it.next().and_then(|h| h.trim().parse().ok());
        let sky = it.next().and_then(|s| {
            SkyKind::ALL.into_iter().find(|k| format!("{k:?}").eq_ignore_ascii_case(s.trim()))
        });
        let theater = it
            .next()
            .and_then(|t| stealth_formats::ALL_THEATERS.into_iter().find(|&x| x == t.trim()));
        let look = if shot == Shot::Apron && hours.is_none() {
            Look::classic()
        } else {
            let theater = theater.unwrap_or_else(|| theater_for(shot.place()));
            Look::roll(theater, sky, hours)
        };
        let at = std::env::var("SM89_MENU_SHOT_AT").unwrap_or_default();
        let mut at = at.split(',').map(|t| t.trim().parse::<f32>().ok());
        let from = at.next().flatten().unwrap_or(0.0);
        let stop = at.next().flatten();
        // `SM89_MENU_ROLL=<n>`: the scene's random choices.
        let roll = std::env::var("SM89_MENU_ROLL").ok().and_then(|r| r.parse().ok());
        Some(Director {
            shot,
            t: from,
            spawned: false,
            roll: roll.unwrap_or_else(rand::random),
            look,
            look_changed: true,
            order: vec![shot],
            next: 0,
            changes: 0,
            looks: 0,
            held: true,
            from,
            stop,
        })
    }

    /// The scene `shot` is on.
    pub fn on(&self, shot: Shot) -> bool {
        self.shot == shot
    }

    /// A random choice among `n`, the scene's own.
    pub fn pick(&self, n: usize) -> usize {
        (self.roll as usize) % n.max(1)
    }

    /// Steps the clock; at a scene's end the next one comes (and every `LOOK_EVERY` a new look).
    /// True when the scene changed.
    fn step(&mut self, dt: f32) -> bool {
        self.t += dt;
        if let Some(stop) = self.stop {
            self.t = self.t.min(stop);
        }
        if self.t < SHOT_SECONDS {
            return false;
        }
        if self.held && self.shot == Shot::Apron {
            // The apron's things live on (in the headset it is the only scene).
            return false;
        }
        if self.held {
            // Again from the start.
            self.t = self.from;
            self.spawned = false;
            if std::env::var("SM89_MENU_ROLL").is_err() {
                self.roll = rand::random();
            }
            return true;
        }
        self.t = 0.0;
        self.spawned = false;
        self.roll = rand::random();
        if self.next >= self.order.len() {
            self.order = cycle();
            self.next = 0;
        }
        self.shot = self.order[self.next];
        self.next += 1;
        self.changes += 1;
        let place = self.shot.place();
        if self.changes.is_multiple_of(LOOK_EVERY) {
            self.look = Look::roll(theater_for(place), None, None);
            self.looks += 1;
            self.look_changed = true;
        } else if !fits(place, self.look.theater) {
            self.look.theater = theater_for(place);
            self.look.classic = false;
            self.look_changed = true;
        }
        true
    }
}

/// One turn of every scene: the apron first, the others shuffled.
fn cycle() -> Vec<Shot> {
    let mut order: Vec<Shot> = SHOTS.to_vec();
    for i in (2..order.len()).rev() {
        order.swap(i, rand::random_range(1..=i));
    }
    order
}

/// A scene `place`d so can be shown on `theater`.
fn fits(place: Place, theater: &str) -> bool {
    match place {
        Place::Tabor => theater == TABOR_THEATER,
        Place::Sea => SEA_THEATERS.contains(&theater),
        Place::Any => true,
    }
}

/// A theater for a scene: Tabor's for the apron, one with a carrier for a scene at sea, else
/// any.
fn theater_for(place: Place) -> &'static str {
    match place {
        Place::Tabor => TABOR_THEATER,
        Place::Sea => SEA_THEATERS[rand::random_range(0..SEA_THEATERS.len())],
        Place::Any => {
            let all = stealth_formats::ALL_THEATERS;
            all[rand::random_range(0..all.len())]
        }
    }
}

/// Steps the director; at a change the last scene's things go.
pub fn direct(
    mut commands: Commands,
    time: Res<Time>,
    mut director: ResMut<Director>,
    things: Query<Entity, With<super::ShotEntity>>,
    mut trails: ResMut<crate::trails::Trails>,
    mut blasts: ResMut<crate::explosions::Explosions>,
) {
    if director.step(time.delta_secs().min(0.1)) {
        let l = &director.look;
        debug!(
            "menu scene: {} on {} at {:.1} h, {:?}",
            director.shot.name(),
            l.theater,
            l.hours,
            l.weather.sky
        );
        for e in &things {
            commands.entity(e).despawn();
        }
        trails.clear(&mut commands);
        blasts.clear = true;
    }
}

/// Run condition: the scene `shot` is on.
pub fn shot_on(shot: Shot) -> impl Fn(Option<Res<Director>>) -> bool + Clone {
    move |d: Option<Res<Director>>| d.is_some_and(|d| d.on(shot))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_scene_comes_once_a_cycle_the_apron_first() {
        for _ in 0..200 {
            let order = cycle();
            assert_eq!(order.len(), SHOTS.len());
            assert_eq!(order[0], Shot::Apron);
            for s in SHOTS {
                assert_eq!(order.iter().filter(|&&o| o == s).count(), 1);
            }
        }
    }

    #[test]
    fn the_weather_changes_every_four_scenes_and_each_scene_is_in_its_place() {
        let mut d = Director::new(false);
        d.held = false;
        for n in 1..=100 {
            let looks = d.looks;
            assert!(d.step(SHOT_SECONDS));
            assert_eq!(d.looks - looks, (n % LOOK_EVERY == 0) as usize, "scene {n}");
            assert!(fits(d.shot.place(), d.look.theater), "{:?} on {}", d.shot, d.look.theater);
        }
    }
}

/// The test aid's screenshots (`SM89_MENU_SNAP`): the times left and the next number.
pub fn snap(
    mut commands: Commands,
    director: Res<Director>,
    mut left: Local<Option<(Vec<f32>, usize, f32)>>,
    mut exit: MessageWriter<AppExit>,
) {
    let (times, n, wait) = left.get_or_insert_with(|| {
        let v = std::env::var("SM89_MENU_SNAP").unwrap_or_default();
        (v.split(',').filter_map(|t| t.trim().parse().ok()).collect(), 0, 0.0)
    });
    let Ok(path) = std::env::var("SM89_MENU_SNAP_PNG") else { return };
    if times.is_empty() {
        // A moment for the last one to be written.
        if *n > 0 {
            *wait += 1.0;
            if *wait > 30.0 {
                exit.write(AppExit::Success);
            }
        }
        return;
    }
    if director.t >= times[0] {
        times.remove(0);
        *n += 1;
        let file = format!("{}_{:02}.png", path.trim_end_matches(".png"), *n);
        commands
            .spawn(bevy::render::view::screenshot::Screenshot::primary_window())
            .observe(bevy::render::view::screenshot::save_to_disk(file));
    }
}
