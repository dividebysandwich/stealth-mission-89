//! The settings that decide how the war is fought: the host's, for everybody in a session.

use serde::{Deserialize, Serialize};
use stealth_campaign::{Aircraft, CampaignRules};
use stealth_sim::{Airframe, LandingMode, SimOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct Rules {
    /// Missile hits can destroy the aircraft outright, fires, damaged control surfaces.
    pub realistic_damage: bool,
    /// Proportional navigation, burnt-out motors, lofted SAM shots.
    pub realistic_guidance: bool,
    /// Collisions bounce the aircraft off, crashes do not end the flight.
    pub no_crashes: bool,
    /// The sun and the moon move: moonlight sets how far enemy pilots see at night.
    pub continuous_time: bool,
    /// The F-117A as it was: no cannon, only two bays.
    pub realistic_weapons: bool,
    /// The players' missiles and gunfire damage each other.
    #[serde(default)]
    pub friendly_fire: bool,
    /// The classic flat ground, without the rolling hills.
    #[serde(default)]
    pub classic_terrain: bool,
    /// The AI's jets fly the classic moves: no flight model, no missile defence.
    #[serde(default)]
    pub classic_ai_flight: bool,
    /// A player's aircraft lost comes out of the friendly side's (the AI's) fighters or
    /// helicopters.
    #[serde(default)]
    pub airframe_losses: bool,
    /// The AI does not fly the missions on the board 5 minutes after they are posted.
    #[serde(default)]
    pub no_ai_missions: bool,
}

impl From<CampaignRules> for Rules {
    fn from(r: CampaignRules) -> Self {
        Self {
            realistic_damage: r.realistic_damage,
            realistic_guidance: r.realistic_guidance,
            no_crashes: r.no_crashes,
            continuous_time: r.continuous_time,
            realistic_weapons: r.realistic_weapons,
            friendly_fire: r.friendly_fire,
            classic_terrain: r.classic_terrain,
            classic_ai_flight: r.classic_ai_flight,
            airframe_losses: r.airframe_losses,
            no_ai_missions: r.no_ai_missions,
        }
    }
}

impl From<Rules> for CampaignRules {
    fn from(r: Rules) -> Self {
        Self {
            realistic_damage: r.realistic_damage,
            realistic_guidance: r.realistic_guidance,
            no_crashes: r.no_crashes,
            continuous_time: r.continuous_time,
            realistic_weapons: r.realistic_weapons,
            friendly_fire: r.friendly_fire,
            classic_terrain: r.classic_terrain,
            classic_ai_flight: r.classic_ai_flight,
            airframe_losses: r.airframe_losses,
            no_ai_missions: r.no_ai_missions,
        }
    }
}

impl Rules {
    /// The simulation's options for a campaign under these rules.
    pub fn sim_options(&self) -> SimOptions {
        SimOptions {
            landing: if self.no_crashes { LandingMode::NoCrashes } else { LandingMode::Easy },
            realistic_damage: self.realistic_damage,
            realistic_guidance: self.realistic_guidance,
            friendly_fire: self.friendly_fire,
            classic_terrain: self.classic_terrain,
            realistic_ai_flight: !self.classic_ai_flight,
            air_defence: true,
            campaign: true,
            ..Default::default()
        }
    }

    /// An aircraft's simulation traits under these rules (the real F-117A without a cannon).
    pub fn airframe(&self, aircraft: Aircraft) -> Airframe {
        match aircraft {
            Aircraft::F117 => Airframe { lockheed: self.realistic_weapons, ..Airframe::default() },
            Aircraft::F16 => Airframe { external_stores: true, ..Airframe::default() },
            // Weapons in bays as the F-117A's, a cannon, a lower radar return.
            Aircraft::F22 => Airframe { raptor: true, ..Airframe::default() },
            // No weapons, no cannon; cargo inside, dropped over the ramp (its "bay").
            Aircraft::C5 => Airframe { galaxy: true, ..Airframe::default() },
            Aircraft::Tornado => {
                Airframe { external_stores: true, tornado: true, ..Airframe::default() }
            }
            Aircraft::A10 => Airframe { external_stores: true, a10: true, ..Airframe::default() },
            Aircraft::AV8B => {
                Airframe { external_stores: true, harrier: true, ..Airframe::default() }
            }
            Aircraft::UH60 => {
                Airframe { external_stores: true, helicopter: true, ..Airframe::default() }
            }
            // Weapons on pylons, no turret: its gun is the machine gun pods.
            Aircraft::OH58 => Airframe {
                external_stores: true,
                helicopter: true,
                kiowa: true,
                ..Airframe::default()
            },
            Aircraft::CH47 => Airframe {
                external_stores: true,
                helicopter: true,
                chinook: true,
                ..Airframe::default()
            },
            Aircraft::AH1Z => Airframe {
                external_stores: true,
                helicopter: true,
                gunship: true,
                ..Airframe::default()
            },
            // Missiles in its bays: the doors open as it fires.
            Aircraft::RAH66 => {
                Airframe { helicopter: true, gunship: true, comanche: true, ..Airframe::default() }
            }
        }
    }
}
