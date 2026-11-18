//! The carrier's landing signal officer ("Paddles"): talks the player's jet down the last mile
//! to the deck, on the radio beside the controllers, with the landing lens (`meatball`) showing
//! the glide slope.
//!
//! - Three quarters of a mile out on the approach, gear down: "call the ball", the pilot's
//!   ball call with his fuel, "roger ball".
//! - Then a correction every second or two while the jet is off: low ("power", "you're low"),
//!   high, lineup, fast or slow. Quiet while it is on the glide slope and centre line.
//! - A wave-off (the lens's red lights flash): gear up, a foul deck (a jet on a waist
//!   catapult), too low close in, too far off the centre line or the deck's heading close in.
//!   It is advice: a jet that lands anyway still catches a wire (`flight::arrest`).
//!
//! Helicopters and the Harrier land as they like. The calls are in English, as the
//! controllers' are, and stay on this client (each player has his own Paddles).

use std::collections::VecDeque;

use bevy::prelude::*;
use stealth_sim::types::oflags;

use crate::atc::Call;
use crate::flight::{Aircraft, Player};
use crate::settings::AppState;
use crate::world::{WU, WorldPos, bevy_to_wu};

pub struct LsoPlugin;

impl Plugin for LsoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LsoState>()
            .add_systems(OnEnter(AppState::Flight), |mut s: ResMut<LsoState>| {
                *s = LsoState::default();
            })
            .add_systems(Update, control.run_if(in_state(AppState::Flight)));
    }
}

/// The LSO's station name on the radio.
pub const PADDLES: &str = "Paddles";
/// Approach window (m from the touchdown point, out along the approach): the ball is called
/// here, and the LSO works the jet from this far in to this close.
const BALL_CALL_M: f64 = 1300.0;
const APPROACH_FAR_M: f64 = 1600.0;
const APPROACH_NEAR_M: f64 = 30.0;
/// After a wave-off (or a pass the LSO gave up on) he is back once the jet is this far out
/// again, or this high above the deck (m).
const REARM_OUT_M: f64 = 2200.0;
const REARM_HIGH_M: f64 = 450.0;
/// Highest above the deck an approach is worked (m), and the widest off the heading (deg).
const APPROACH_HIGH_M: f64 = 460.0;
const APPROACH_HEADING_DEG: f64 = 30.0;
/// Seconds between corrections, and before the same one again.
const CORRECTION_GAP: f32 = 1.6;
const REPEAT_GAP: f32 = 3.5;
/// Glide slope errors (deg): a little / well low, high.
const LOW_DEG: f64 = -0.45;
const POWER_DEG: f64 = -0.8;
const HIGH_DEG: f64 = 0.6;
/// Wave-off: this far below the slope inside `WAVE_LOW_M`, this far off the centre line
/// inside `WAVE_LINEUP_M` (m; the deck's landing area), off the deck's heading inside 600 m.
const WAVE_LOW_DEG: f64 = -1.3;
const WAVE_LOW_M: f64 = 800.0;
const WAVE_LINEUP_OFF_M: f64 = 30.0;
const WAVE_LINEUP_M: f64 = 400.0;
const WAVE_HEADING_DEG: f64 = 20.0;
/// Lineup calls: off the centre line by this angle (deg) or at least `LINEUP_MIN_M`.
const LINEUP_DEG: f64 = 1.2;
const LINEUP_MIN_M: f64 = 5.0;
/// Angle of attack off the on-speed value (deg) that is fast or slow.
const AOA_OFF: f32 = 2.0;
/// The wave-off lights flash this long (s).
pub const WAVEOFF_LIGHTS_S: f32 = 4.0;

/// What the LSO sees of the jet this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LsoInput {
    pub dt: f32,
    /// Out from the touchdown point along the approach (m, positive behind the ship), right
    /// of the centre line (m), and height above the deck (m).
    pub out: f64,
    pub right: f64,
    pub height: f64,
    /// The jet's track off the landing heading (deg, positive right).
    pub track_off: f64,
    pub gear_down: bool,
    pub on_deck: bool,
    /// Angle of attack (deg; None in the classic flight model, which has none worth calling).
    pub aoa: Option<f32>,
    /// Fuel (lbs).
    pub fuel: f32,
    /// A jet on a waist catapult, in the landing area.
    pub foul_deck: bool,
}

impl LsoInput {
    /// The glide slope error (deg, positive high) and the lineup error (deg, positive right).
    fn errors(&self) -> (f64, f64) {
        let out = self.out.max(1.0);
        let gs = self.height.atan2(out).to_degrees() - stealth_sim::catapult::GLIDE_DEG;
        (gs, self.right.atan2(out).to_degrees())
    }

    fn on_approach(&self) -> bool {
        (APPROACH_NEAR_M..=APPROACH_FAR_M).contains(&self.out)
            && self.right.abs() < 0.3 * self.out + 60.0
            && self.height < APPROACH_HIGH_M
            && self.track_off.abs() < APPROACH_HEADING_DEG
    }
}

/// A call, and whether it cuts in over whatever the radio is saying (a wave-off).
#[derive(Debug, Clone, PartialEq)]
pub struct LsoCall {
    pub call: Call,
    pub urgent: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Phase {
    /// Waiting for a jet on the approach.
    #[default]
    Idle,
    /// The ball called; working the jet down.
    Ball,
    /// Done with this pass (a wave-off, the deck, or it left): quiet until re-armed.
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Correction {
    Power,
    Low,
    High,
    ComeLeft,
    RightForLineup,
    Fast,
    Slow,
}

impl Correction {
    fn text(self) -> &'static str {
        match self {
            Correction::Power => "Power.",
            Correction::Low => "You're low.",
            Correction::High => "You're high.",
            Correction::ComeLeft => "Come left.",
            Correction::RightForLineup => "Right for lineup.",
            Correction::Fast => "You're fast.",
            Correction::Slow => "You're slow.",
        }
    }
}

/// One jet's LSO.
#[derive(Debug, Clone)]
pub struct Lso {
    callsign: String,
    phase: Phase,
    /// Calls waiting their turn: (seconds to wait, call).
    queue: VecDeque<(f32, Call)>,
    /// Since the last correction, and the last one.
    since: f32,
    last: Option<Correction>,
    /// The ball is called and answered: corrections may start.
    rogered: bool,
    /// The wave-off lights' time left (s).
    pub waveoff: f32,
}

impl Lso {
    pub fn new(callsign: impl Into<String>) -> Self {
        Self {
            callsign: callsign.into(),
            phase: Phase::Idle,
            queue: VecDeque::new(),
            since: 0.0,
            last: None,
            rogered: false,
            waveoff: 0.0,
        }
    }

    fn say(&mut self, delay: f32, text: impl Into<String>, own: bool) {
        let from = if own { self.callsign.clone() } else { PADDLES.to_string() };
        self.queue.push_back((delay, Call { from, text: text.into(), own }));
    }

    /// One frame: the calls to make now.
    pub fn step(&mut self, i: &LsoInput) -> Vec<LsoCall> {
        let mut out = Vec::new();
        self.waveoff = (self.waveoff - i.dt).max(0.0);
        self.since += i.dt;
        match self.phase {
            Phase::Idle => {
                if i.on_approach() && i.out <= BALL_CALL_M && !i.on_deck {
                    self.phase = Phase::Ball;
                    self.rogered = false;
                    self.last = None;
                    let cs = self.callsign.clone();
                    self.say(0.0, format!("{cs}, Paddles contact, call the ball."), false);
                    let fuel = (i.fuel / 1000.0).max(0.0);
                    self.say(1.6, format!("{cs}, ball, {fuel:.1}."), true);
                    self.say(1.2, "Roger ball.", false);
                }
            }
            Phase::Ball => {
                if i.on_deck || i.out < APPROACH_NEAR_M {
                    // In the wires (or past the point the LSO can help): no grade.
                    self.end_pass();
                } else if !i.on_approach() {
                    self.end_pass();
                } else if let Some(why) = self.wave_off_reason(i) {
                    self.end_pass();
                    self.waveoff = WAVEOFF_LIGHTS_S;
                    let text = match why {
                        "" => "Wave off, wave off.".to_string(),
                        w => format!("Wave off, wave off. {w}."),
                    };
                    out.push(LsoCall {
                        call: Call { from: PADDLES.to_string(), text, own: false },
                        urgent: true,
                    });
                    return out;
                } else if self.rogered && self.queue.is_empty() {
                    self.correct(i, &mut out);
                }
            }
            Phase::Done => {
                if i.out > REARM_OUT_M || i.out < -REARM_OUT_M || i.height > REARM_HIGH_M {
                    self.phase = Phase::Idle;
                }
            }
        }
        if let Some((wait, _)) = self.queue.front_mut() {
            *wait -= i.dt;
            if *wait <= 0.0 {
                let (_, call) = self.queue.pop_front().unwrap();
                if self.queue.is_empty() && self.phase == Phase::Ball {
                    self.rogered = true;
                    self.since = 0.0;
                }
                out.push(LsoCall { call, urgent: false });
            }
        }
        out
    }

    fn end_pass(&mut self) {
        self.phase = Phase::Done;
        self.queue.clear();
        self.rogered = false;
    }

    /// Why the jet must go around (None while it may land on): "" for the approach itself.
    fn wave_off_reason(&self, i: &LsoInput) -> Option<&'static str> {
        let (gs, _) = i.errors();
        if !i.gear_down && i.out < 1000.0 {
            return Some("Gear");
        }
        if i.foul_deck && i.out < 1000.0 {
            return Some("Foul deck");
        }
        // Below the deck short of the ramp.
        let low = i.out < WAVE_LOW_M && (gs < WAVE_LOW_DEG || (i.height < 3.0 && i.out > 100.0));
        let lineup = i.out < WAVE_LINEUP_M && i.right.abs() > WAVE_LINEUP_OFF_M;
        let heading = i.out < 600.0 && i.track_off.abs() > WAVE_HEADING_DEG;
        (low || lineup || heading).then_some("")
    }

    /// The most pressing correction, if the jet needs one and the last was long enough ago.
    fn correct(&mut self, i: &LsoInput, out: &mut Vec<LsoCall>) {
        if self.since < CORRECTION_GAP {
            return;
        }
        let (gs, lineup) = i.errors();
        let lineup_off = |a: f64| a.abs() > LINEUP_DEG && i.right.abs() > LINEUP_MIN_M;
        let c = if gs < POWER_DEG {
            Some(Correction::Power)
        } else if gs < LOW_DEG {
            Some(Correction::Low)
        } else if gs > HIGH_DEG {
            Some(Correction::High)
        } else if lineup_off(lineup) && lineup > 0.0 {
            Some(Correction::ComeLeft)
        } else if lineup_off(lineup) {
            Some(Correction::RightForLineup)
        } else if let Some(aoa) = i.aoa {
            let on_speed = crate::hud::modern::APPROACH_AOA;
            if aoa < on_speed - AOA_OFF {
                Some(Correction::Fast)
            } else if aoa > on_speed + AOA_OFF {
                Some(Correction::Slow)
            } else {
                None
            }
        } else {
            None
        };
        let Some(c) = c else { return };
        // Not the same call again straight away (but "power" when it is getting worse).
        if self.last == Some(c) && self.since < REPEAT_GAP {
            return;
        }
        self.last = Some(c);
        self.since = 0.0;
        out.push(LsoCall {
            call: Call { from: PADDLES.to_string(), text: c.text().to_string(), own: false },
            urgent: false,
        });
    }
}

/// The LSO of this flight, and what the lens shows.
#[derive(Resource, Default)]
pub struct LsoState {
    lso: Option<Lso>,
    /// The wave-off lights flash on each carrier's lens (its world object index) while > 0.
    pub waveoff: Option<(usize, f32)>,
}

/// The carrier the player's jet is approaching (world object index, landing pose): the nearest
/// within a few miles.
pub fn approaching(
    sim: &stealth_sim::Sim,
    east: f64,
    north: f64,
) -> Option<(usize, stealth_sim::catapult::LandingPose)> {
    let range = 6000.0 / WU;
    sim.objects()
        .iter()
        .enumerate()
        .filter(|(_, o)| o.flags & oflags::CARRIER != 0 && o.flags & oflags::DESTROYED == 0)
        .map(|(i, o)| (i, stealth_sim::catapult::landing((o.x, o.y), sim.ns())))
        .map(|(i, l)| (i, l, (l.east - east).hypot(l.north - north)))
        .filter(|&(.., d)| d < range)
        .min_by(|a, b| a.2.total_cmp(&b.2))
        .map(|(i, l, _)| (i, l))
}

#[allow(clippy::too_many_arguments)]
fn control(
    settings: Res<crate::settings::Settings>,
    constants: Res<crate::flight::FlightConstants>,
    time: Res<Time>,
    combat: Option<Res<crate::combat::Combat>>,
    campaign: Option<Res<crate::campaign::ActiveCampaign>>,
    player: Query<(&Aircraft, &WorldPos), With<Player>>,
    atc: Res<crate::atc::AtcState>,
    mut state: ResMut<LsoState>,
    mut notices: ResMut<crate::player_notices::PlayerNotices>,
    mut sounds: MessageWriter<crate::sound::PlaySound>,
    mut tts: ResMut<crate::tts::Tts>,
) {
    let dt = time.delta_secs();
    if let Some((_, t)) = state.waveoff.as_mut() {
        *t -= dt;
        if *t <= 0.0 {
            state.waveoff = None;
        }
    }
    let (Some(combat), Ok((a, pos))) = (combat, player.single()) else { return };
    let spec = settings.aircraft.spec();
    if !settings.atc
        || a.ended.is_some()
        || dt <= 0.0
        || spec.helicopter
        || constants.heli.is_some()
        || constants.aero.vtol.is_some()
        || campaign.as_ref().is_some_and(|c| c.phase != crate::campaign::Phase::Sortie)
    {
        state.lso = None;
        return;
    }
    let sim = &combat.sim;
    let (east, north, _) = bevy_to_wu(pos.0);
    let Some((carrier, landing)) = approaching(sim, east, north) else {
        if let Some(l) = state.lso.as_mut() {
            l.end_pass();
        }
        return;
    };
    let (out, right) = landing.out_and_right(east, north);
    let track = (a.ground_vel.x as f64).atan2(-a.ground_vel.z as f64);
    let track_off = ((track - landing.heading).to_degrees() + 540.0).rem_euclid(360.0) - 180.0;
    let me = Some(sim.active_pilot());
    let foul_deck =
        (2..stealth_sim::catapult::CATAPULTS).any(|c| sim.catapult_in_use(carrier, c, me));
    let aoa = a.angle_of_attack();
    let input = LsoInput {
        dt,
        out: out * WU,
        right: right * WU,
        height: (a.altitude as f64 - stealth_sim::catapult::DECK_WU) * WU,
        track_off,
        gear_down: a.gear_down,
        on_deck: a.on_ground || a.trap.is_some(),
        aoa,
        fuel: a.fuel,
        foul_deck,
    };
    let lso = state
        .lso
        .get_or_insert_with(|| Lso::new(atc.callsign().unwrap_or("Viper 1-1").to_string()));
    let calls = lso.step(&input);
    if lso.waveoff > 0.0 {
        state.waveoff = Some((carrier, lso.waveoff));
    }
    for c in calls {
        info!("LSO {}: {}", c.call.from, c.call.text);
        notices.chat(&c.call.from, &c.call.text, c.call.own);
        if c.urgent {
            tts.stop();
        }
        if !crate::atc::speak(&mut tts, &settings, &c.call) {
            sounds.write(crate::sound::PlaySound {
                id: crate::atc::RADIO_CLICK,
                priority: crate::sound::DIRECT,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A jet on the glide slope and centre line, `out` m from the touchdown point.
    fn on_slope(out: f64) -> LsoInput {
        LsoInput {
            dt: 0.1,
            out,
            right: 0.0,
            height: out * stealth_sim::catapult::GLIDE_DEG.to_radians().tan(),
            track_off: 0.0,
            gear_down: true,
            on_deck: false,
            aoa: Some(crate::hud::modern::APPROACH_AOA),
            fuel: 5200.0,
            foul_deck: false,
        }
    }

    /// Flies in from 2 km at 70 m/s, `edit` changing each frame's input: every call (up to a
    /// wave-off).
    fn fly(lso: &mut Lso, edit: impl Fn(&mut LsoInput)) -> Vec<LsoCall> {
        let mut calls = Vec::new();
        let mut out = 2000.0;
        while out > 0.0 {
            let mut i = on_slope(out);
            edit(&mut i);
            let now = lso.step(&i);
            let urgent = now.iter().any(|c| c.urgent);
            calls.extend(now);
            if urgent {
                break;
            }
            out -= 7.0;
        }
        calls
    }

    fn texts(calls: &[LsoCall]) -> Vec<&str> {
        calls.iter().map(|c| c.call.text.as_str()).collect()
    }

    #[test]
    fn calls_the_ball_and_stays_quiet_on_the_slope() {
        let mut lso = Lso::new("Viper 1-1");
        let calls = fly(&mut lso, |_| {});
        assert_eq!(
            texts(&calls),
            ["Viper 1-1, Paddles contact, call the ball.", "Viper 1-1, ball, 5.2.", "Roger ball."]
        );
        assert!(calls[1].call.own && !calls[0].call.own);
        assert_eq!(calls[0].call.from, PADDLES);
        assert_eq!(lso.waveoff, 0.0);
    }

    #[test]
    fn corrects_low_high_lineup_and_speed() {
        let low = fly(&mut Lso::new("A"), |i| i.height *= 0.8);
        assert!(texts(&low).iter().any(|t| *t == "You're low." || *t == "Power."), "{low:?}");
        let high = fly(&mut Lso::new("A"), |i| i.height *= 1.4);
        assert!(texts(&high).contains(&"You're high."), "{high:?}");
        let left = fly(&mut Lso::new("A"), |i| i.right = -0.04 * i.out - 6.0);
        assert!(texts(&left).contains(&"Right for lineup."), "{left:?}");
        let right = fly(&mut Lso::new("A"), |i| i.right = 0.04 * i.out + 6.0);
        assert!(texts(&right).contains(&"Come left."), "{right:?}");
        let fast = fly(&mut Lso::new("A"), |i| i.aoa = Some(6.0));
        assert!(texts(&fast).contains(&"You're fast."), "{fast:?}");
        // Corrections come no faster than one every CORRECTION_GAP (7 m a frame at 0.1 s).
        let n = texts(&high).iter().filter(|t| **t == "You're high.").count();
        assert!((2..=8).contains(&n), "{n}");
    }

    #[test]
    fn waves_off_gear_up_foul_deck_and_low() {
        type Edit = Box<dyn Fn(&mut LsoInput)>;
        let cases: [(Edit, &str); 2] = [
            (Box::new(|i: &mut LsoInput| i.gear_down = false), "Gear"),
            (Box::new(|i: &mut LsoInput| i.foul_deck = i.out < 600.0), "Foul deck"),
        ];
        for (edit, reason) in cases {
            let mut lso = Lso::new("A");
            let calls = fly(&mut lso, edit);
            let last = calls.last().unwrap();
            assert_eq!(last.call.text, format!("Wave off, wave off. {reason}."));
            assert!(last.urgent);
        }
        let mut lso = Lso::new("A");
        let calls = fly(&mut lso, |i| {
            if i.out < 500.0 {
                i.height *= 0.4;
            }
        });
        assert_eq!(calls.last().unwrap().call.text, "Wave off, wave off.");
        assert!(lso.waveoff > 0.0);
    }

    #[test]
    fn quiet_after_a_wave_off_until_the_next_pass() {
        let mut lso = Lso::new("A");
        let calls = fly(&mut lso, |i| i.gear_down = i.out > 900.0);
        let n = calls.len();
        assert!(calls[n - 1].call.text.starts_with("Wave off"));
        // Around the pattern (out past 2.2 km), then a good pass: the ball again.
        let mut i = on_slope(2500.0);
        lso.step(&i);
        i.out = 3000.0;
        lso.step(&i);
        let calls = fly(&mut lso, |_| {});
        assert_eq!(calls[0].call.text, "A, Paddles contact, call the ball.");
    }

    #[test]
    fn landing_ends_the_pass_without_a_grade() {
        let mut lso = Lso::new("A");
        fly(&mut lso, |_| {});
        let mut i = on_slope(5.0);
        i.on_deck = true;
        for _ in 0..100 {
            assert!(lso.step(&i).is_empty());
        }
    }
}
