//! The game's names in the player's language (the messages are in `assets/i18n`, the
//! translation system in `crates/stealth-i18n`).

use stealth_i18n::{t, tr, tr_lines};
use stealth_mission::{MissionKind, Sky};

use crate::aircraft::AircraftKind;
use crate::menu::THEATERS;

/// A theater's name ([`THEATERS`] index).
pub fn theater_name(i: usize) -> &'static str {
    THEATERS.get(i).map_or("", |t| tr(&format!("theater-{}", t.0)))
}

/// A theater's name from its code ("central_europe"); the code in capitals if unknown.
pub fn theater_name_of(code: &str) -> String {
    THEATERS
        .iter()
        .position(|t| t.0.eq_ignore_ascii_case(code))
        .map_or_else(|| code.to_ascii_uppercase(), |i| theater_name(i).to_string())
}

/// A theater's facts ([`THEATERS`] index).
pub fn theater_facts(i: usize) -> &'static [&'static str] {
    THEATERS.get(i).map_or(&[], |t| tr_lines(&format!("theater-{}.facts", t.0)))
}

/// The situation's name (0 Cold War, 1 Limited War, 2 Conventional War).
pub fn roe_name(roe: u8) -> &'static str {
    tr(&format!("roe-{}", roe.min(2)))
}

pub fn roe_facts(roe: u8) -> &'static [&'static str] {
    tr_lines(&format!("roe-{}.facts", roe.min(2)))
}

fn kind_id(k: MissionKind) -> &'static str {
    match k {
        MissionKind::Strike => "kind-strike",
        MissionKind::Air => "kind-air",
        MissionKind::StrikeTraining => "kind-strike-training",
        MissionKind::AirTraining => "kind-air-training",
        MissionKind::DroneDefense => "kind-drone-defense",
    }
}

pub fn kind_name(k: MissionKind) -> &'static str {
    tr(kind_id(k))
}

pub fn kind_facts(k: MissionKind) -> &'static [&'static str] {
    tr_lines(&format!("{}.facts", kind_id(k)))
}

/// The opponents' level (0 green to 3 elite).
pub fn opponents_name(o: u8) -> &'static str {
    tr(&format!("opponents-{}", o.min(3)))
}

pub fn opponents_facts(o: u8) -> &'static [&'static str] {
    tr_lines(&format!("opponents-{}.facts", o.min(3)))
}

fn sky_id(sky: Sky) -> String {
    format!("sky-{}", sky.name().to_ascii_lowercase())
}

/// A sky's name ("Scattered").
pub fn sky_name(sky: Sky) -> &'static str {
    tr(&sky_id(sky))
}

pub fn sky_facts(sky: Sky) -> &'static [&'static str] {
    tr_lines(&format!("{}.facts", sky_id(sky)))
}

/// The notes of a flyable aircraft (its name is the real one, untranslated).
pub fn aircraft_facts(a: AircraftKind) -> &'static [&'static str] {
    tr_lines(&format!("aircraft-{}.facts", a.id()))
}

/// "Day" or "Night".
pub fn daytime(night: bool) -> &'static str {
    if night { t!("summary-night") } else { t!("summary-day") }
}

/// How a flight ended.
pub fn outcome_name(o: stealth_sim::score::Outcome) -> &'static str {
    tr(o.id())
}

/// A mission rating as the report gives it in English ("Excellent").
pub fn rating_name(rating: &str) -> &'static str {
    tr(&format!("rating-{}", rating.to_ascii_lowercase()))
}

/// The font of the 2D overlays (the end of mission, the downed pilot's panel): egui's Hack, as
/// the menus and tooltips, which has the translations' letters (Bevy's built-in font has ASCII
/// only).
#[derive(bevy::prelude::Resource, Clone)]
pub struct UiFont(pub bevy::prelude::Handle<bevy::prelude::Font>);

impl bevy::prelude::FromWorld for UiFont {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        // Without text rendering (glvr's own renderer), Bevy's default handle.
        let font = world
            .get_resource_mut::<bevy::prelude::Assets<bevy::prelude::Font>>()
            .map(|mut fonts| {
                fonts.add(bevy::prelude::Font::from_bytes(
                    epaint_default_fonts::HACK_REGULAR.to_vec(),
                ))
            })
            .unwrap_or_default();
        UiFont(font)
    }
}

impl UiFont {
    /// The font at `px` pixels.
    pub fn at(&self, px: f32) -> bevy::prelude::TextFont {
        bevy::prelude::TextFont {
            font: self.0.clone().into(),
            font_size: bevy::text::FontSize::Px(px),
            ..Default::default()
        }
    }
}
