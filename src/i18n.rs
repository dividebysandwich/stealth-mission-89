//! The game's names in the player's language (the messages are in `assets/i18n`, the
//! translation system in `crates/stealth-i18n`).

use std::sync::Arc;

use bevy_egui::{EguiContexts, EguiPrimaryContextPass, egui};
use stealth_i18n::{Language, language, t, tr, tr_lines};
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

/// The languages with a font of their own besides Hack, which only draws the Latin, Greek and
/// Cyrillic letters: the Han characters, kana and hangul come from subsets of Noto Sans CJK
/// (SIL Open Font License, see `assets/fonts/README.md`).
pub const CJK: [Language; 3] = [Language::Zh, Language::Ja, Language::Ko];

/// A CJK language's font: a subset of Noto Sans CJK with the national standard's common
/// characters and the language's texts (`tools/i18n_fonts.py` makes it).
pub fn cjk_font(l: Language) -> Option<&'static [u8]> {
    match l {
        Language::Zh => Some(include_bytes!("../assets/fonts/NotoSansSC-Subset.otf")),
        Language::Ja => Some(include_bytes!("../assets/fonts/NotoSansJP-Subset.otf")),
        Language::Ko => Some(include_bytes!("../assets/fonts/NotoSansKR-Subset.otf")),
        _ => None,
    }
}

/// egui's fonts for language `l`: Hack, then the CJK fonts, `l`'s first (the same Han
/// characters are drawn differently in Chinese, Japanese and Korean). The others stay behind it
/// for the OPTIONS list, which names every language in itself.
fn egui_fonts(l: Language) -> egui::FontDefinitions {
    let mut defs = egui::FontDefinitions::default();
    let mut order = CJK;
    order.sort_by_key(|&c| c != l);
    for c in order {
        let Some(bytes) = cjk_font(c) else { continue };
        let name = format!("cjk-{}", c.code());
        defs.font_data.insert(name.clone(), Arc::new(egui::FontData::from_static(bytes)));
        for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
            if let Some(f) = defs.families.get_mut(&family) {
                f.push(name.clone());
            }
        }
    }
    defs
}

/// Gives egui the fonts of the language when it changes.
fn apply_egui_fonts(
    mut contexts: EguiContexts,
    mut applied: bevy::prelude::Local<Option<Language>>,
) {
    let l = language();
    if *applied == Some(l) {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };
    ctx.set_fonts(egui_fonts(l));
    *applied = Some(l);
}

/// Sets up the fonts: `UiFont` and egui's.
pub struct I18nPlugin;

impl bevy::prelude::Plugin for I18nPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<UiFont>().add_systems(EguiPrimaryContextPass, apply_egui_fonts);
        #[cfg(target_arch = "wasm32")]
        if let Some(offer) = language_offer() {
            use bevy::prelude::*;
            app.insert_resource(offer).add_systems(
                EguiPrimaryContextPass,
                language_offer_ui
                    .run_if(resource_exists::<LanguageOffer>)
                    .run_if(in_state(crate::settings::AppState::Menu)),
            );
        }
    }
}

/// `config/<this>` (the browser's local storage): true once the player has answered Yes or
/// ticked "Do not ask again".
#[cfg(target_arch = "wasm32")]
const OFFER_SETTLED: &str = "language-offer.json";

/// The browser build's offer to play in the browser's language, in that language.
#[cfg(target_arch = "wasm32")]
#[derive(bevy::prelude::Resource)]
struct LanguageOffer {
    language: Language,
    never: bool,
}

/// The offer for this visit: the browser's language is one of the game's, the player has not
/// chosen another language than English (OPTIONS, `--lang`) and has not settled the offer.
#[cfg(target_arch = "wasm32")]
fn language_offer() -> Option<LanguageOffer> {
    let language = crate::settings::browser_language()?;
    let chosen = crate::prefs::Prefs::load().language.is_some_and(|l| l != Language::En);
    if chosen
        || crate::settings::language_arg().is_some()
        || crate::prefs::load_json::<bool>(OFFER_SETTLED) == Some(true)
    {
        return None;
    }
    Some(LanguageOffer { language, never: false })
}

/// The window of the offer, over the menu. Yes switches the game to the language (the
/// preferences keep it); No keeps English, and the offer comes back on the next visit unless the
/// box is ticked.
#[cfg(target_arch = "wasm32")]
fn language_offer_ui(
    mut commands: bevy::prelude::Commands,
    mut contexts: EguiContexts,
    mut offer: bevy::prelude::ResMut<LanguageOffer>,
    mut settings: bevy::prelude::ResMut<crate::settings::Settings>,
) -> bevy::prelude::Result {
    use crate::menu::{BRIGHT, big_button, menu_window};
    let ctx = contexts.ctx_mut()?;
    let l = offer.language;
    let mut answer = None;
    menu_window(ctx, "language-offer", 460.0).order(egui::Order::Foreground).show(ctx, |ui| {
        ui.label(
            egui::RichText::new(stealth_i18n::t_in!(
                l,
                "language-offer-question",
                language = l.native_name()
            ))
            .color(BRIGHT),
        );
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            for (id, yes) in [("language-offer-yes", true), ("language-offer-no", false)] {
                if ui
                    .add(big_button(stealth_i18n::tr_in(l, id)).min_size(egui::vec2(120.0, 40.0)))
                    .clicked()
                {
                    answer = Some(yes);
                }
            }
        });
        ui.add_space(6.0);
        ui.checkbox(&mut offer.never, stealth_i18n::tr_in(l, "language-offer-never"));
    });
    let Some(yes) = answer else { return Ok(()) };
    if yes {
        settings.language = l;
        stealth_i18n::set_language(l);
    }
    // A Yes is not asked again either, even if the player goes back to English in OPTIONS.
    if (yes || offer.never)
        && let Err(e) = crate::prefs::save_json(OFFER_SETTLED, &true)
    {
        bevy::log::warn!("saving {OFFER_SETTLED}: {e}");
    }
    commands.remove_resource::<LanguageOffer>();
    Ok(())
}

/// The font of the 2D overlays and the tooltips (the end of mission, the downed pilot's
/// panel): egui's Hack, which has the translations' letters (Bevy's built-in font has ASCII
/// only), or the CJK language's.
#[derive(bevy::prelude::Resource, Clone)]
pub struct UiFont {
    latin: bevy::prelude::Handle<bevy::prelude::Font>,
    cjk: Vec<(Language, bevy::prelude::Handle<bevy::prelude::Font>)>,
}

impl bevy::prelude::FromWorld for UiFont {
    fn from_world(world: &mut bevy::prelude::World) -> Self {
        // Without text rendering (glvr's own renderer), Bevy's default handles.
        let Some(mut fonts) =
            world.get_resource_mut::<bevy::prelude::Assets<bevy::prelude::Font>>()
        else {
            return UiFont { latin: Default::default(), cjk: Vec::new() };
        };
        let latin =
            fonts.add(bevy::prelude::Font::from_bytes(epaint_default_fonts::HACK_REGULAR.to_vec()));
        let cjk = CJK
            .into_iter()
            .filter_map(|l| {
                let bytes = cjk_font(l)?;
                Some((l, fonts.add(bevy::prelude::Font::from_bytes(bytes.to_vec()))))
            })
            .collect();
        UiFont { latin, cjk }
    }
}

impl UiFont {
    /// The font of the current language.
    pub fn handle(&self) -> bevy::prelude::Handle<bevy::prelude::Font> {
        let l = language();
        self.cjk.iter().find(|(c, _)| *c == l).map_or(&self.latin, |(_, h)| h).clone()
    }

    /// The font at `px` pixels.
    pub fn at(&self, px: f32) -> bevy::prelude::TextFont {
        bevy::prelude::TextFont {
            font: self.handle().into(),
            font_size: bevy::text::FontSize::Px(px),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every character of the CJK languages' texts is in their font, or the player would see an
    /// empty box (`python tools/i18n_fonts.py` makes the fonts again).
    #[test]
    fn cjk_fonts_cover_the_texts() {
        for l in CJK {
            let bytes = cjk_font(l).expect("a font");
            let face = ttf_parser::Face::parse(bytes, 0).expect("a font file");
            let dir = format!("assets/i18n/{}", l.code());
            let mut missing = std::collections::BTreeSet::new();
            for file in std::fs::read_dir(&dir).expect("the language's texts") {
                let text = std::fs::read_to_string(file.unwrap().path()).unwrap();
                missing.extend(text.chars().filter(|&c| {
                    !c.is_control() && !c.is_whitespace() && face.glyph_index(c).is_none()
                }));
            }
            assert!(missing.is_empty(), "{}: not in the font: {missing:?}", l.code());
        }
    }

    /// The texts of the languages without a font of their own (Latin, Greek, Cyrillic) are in
    /// Hack, or egui, Bevy's text and the tooltips would draw empty boxes.
    #[test]
    fn hack_covers_the_texts() {
        let hack = ttf_parser::Face::parse(epaint_default_fonts::HACK_REGULAR, 0).unwrap();
        for l in Language::ALL.into_iter().filter(|l| !CJK.contains(l)) {
            let dir = format!("assets/i18n/{}", l.code());
            let Ok(files) = std::fs::read_dir(&dir) else { continue };
            let mut missing = std::collections::BTreeSet::new();
            for file in files {
                let path = file.unwrap().path();
                if path.extension().is_none_or(|e| e != "ftl") {
                    continue;
                }
                let text = std::fs::read_to_string(path).unwrap();
                missing.extend(text.chars().filter(|&c| {
                    !c.is_control() && !c.is_whitespace() && hack.glyph_index(c).is_none()
                }));
            }
            assert!(missing.is_empty(), "{}: not in Hack: {missing:?}", l.code());
        }
    }

    /// The names of the languages are drawn by the fonts egui has in every language.
    #[test]
    fn the_options_list_is_drawable() {
        for current in Language::ALL {
            for l in Language::ALL {
                let name = stealth_i18n::tr_in(current, &format!("lang-name-{}", l.code()));
                let text = format!("{} ({name})", l.native_name());
                let faces: Vec<_> = std::iter::once(epaint_default_fonts::HACK_REGULAR)
                    .chain(CJK.into_iter().filter_map(cjk_font))
                    .map(|bytes| ttf_parser::Face::parse(bytes, 0).unwrap())
                    .collect();
                // egui falls back from a font to the next one for each character.
                let missing: Vec<char> = text
                    .chars()
                    .filter(|&c| !faces.iter().any(|f| f.glyph_index(c).is_some()))
                    .collect();
                assert!(
                    missing.is_empty(),
                    "{} in {}: {text} {missing:?}",
                    l.code(),
                    current.code()
                );
            }
        }
    }
}
