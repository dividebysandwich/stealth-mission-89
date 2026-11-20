//! The radio menu: the radio key (`\`) opens the calls to the AI wingman on the cockpit's
//! message panel (`player_notices.rs`, on the screen and in the headset); 1 to 7 make one,
//! the radio key again (or a while without a call) closes it. The pilot's call is spoken over
//! the radio (`atc.rs`'s voices), and the wingman's answer after it, and shown on the HUD.
//!
//! A single mission's (or a career's) wingman flies in this simulation; a campaign's call goes
//! to its host (`campaign::Action::Wing`).

use bevy::prelude::*;
use stealth_i18n::tf;
use stealth_sim::wingman::{WingCmd, WingReply};

use crate::combat::Combat;
use crate::input::{Action, ActionEvent, InputBlock, InputSystemsSet, RADIO_KEYS, RadioCapture};
use crate::settings::AppState;

/// Seconds the menu stays open without a call.
const OPEN_SECS: f64 = 12.0;

pub struct RadioMenuPlugin;

impl Plugin for RadioMenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RadioMenu>()
            .add_systems(
                PreUpdate,
                radio_menu
                    .after(InputSystemsSet)
                    .run_if(in_state(AppState::Flight))
                    .run_if(resource_exists::<Combat>),
            )
            .add_systems(
                Update,
                answers.run_if(in_state(AppState::Flight)).run_if(resource_exists::<Combat>),
            )
            .add_systems(OnExit(AppState::Flight), close);
    }
}

/// The radio menu: open since when (real seconds).
#[derive(Resource, Default)]
pub struct RadioMenu {
    open: Option<f64>,
}

/// A call's message id (radio.ftl): the menu's line and what the pilot says.
pub fn call_id(cmd: WingCmd) -> &'static str {
    match cmd {
        WingCmd::Engage => "radio-wing-engage",
        WingCmd::Sams => "radio-wing-sams",
        WingCmd::Cover => "radio-wing-cover",
        WingCmd::Rejoin => "radio-wing-rejoin",
        WingCmd::Rtb => "radio-wing-rtb",
        WingCmd::Trail => "radio-wing-trail",
        WingCmd::Spread => "radio-wing-spread",
    }
}

/// The menu's lines: its title, then each call on its key.
fn lines() -> Vec<String> {
    let mut out = vec![tf!("radio-wing-title").to_string()];
    for (k, &cmd) in WingCmd::ALL.iter().enumerate() {
        out.push(format!("{}  {}", k + 1, tf!(call_id(cmd))));
    }
    out
}

/// Opens and closes the menu, and makes the call picked on it.
#[allow(clippy::too_many_arguments)]
fn radio_menu(
    mut actions: MessageReader<ActionEvent>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<Real>>,
    block: Res<InputBlock>,
    mut menu: ResMut<RadioMenu>,
    mut capture: ResMut<RadioCapture>,
    mut notices: ResMut<crate::player_notices::PlayerNotices>,
    mut combat: ResMut<Combat>,
    mut campaign: Option<ResMut<crate::campaign::ActiveCampaign>>,
    mut atc: Option<ResMut<crate::atc::AtcState>>,
    mut msgs: ResMut<crate::hud::HudMessages>,
) {
    let now = time.elapsed_secs_f64();
    let pressed = actions.read().any(|a| a.0 == Action::RadioMenu);
    if block.0 {
        menu.open = None;
    } else if pressed {
        menu.open = if menu.open.is_some() { None } else { Some(now) };
    }
    if menu.open.is_some_and(|t| now - t > OPEN_SECS) {
        menu.open = None;
    }
    let picked = menu
        .open
        .and_then(|_| RADIO_KEYS.iter().position(|&k| keys.just_pressed(k)))
        .map(|k| WingCmd::ALL[k]);
    if let Some(cmd) = picked {
        menu.open = None;
        // The pilot's call over the radio, then the wingman's answer (`answers`).
        let call = tf!(call_id(cmd)).to_string();
        if !atc.as_mut().is_some_and(|a| a.radio("", String::new(), Some(call.clone()))) {
            msgs.show(call);
        }
        match campaign.as_deref_mut() {
            Some(c) => c.actions.push(crate::campaign::Action::Wing(cmd)),
            // Its answer comes with the simulation's next step (`answers`).
            None => {
                let me = combat.sim.active_pilot();
                combat.sim.wing_command(me, cmd);
            }
        }
    }
    let open = menu.open.is_some();
    if capture.0 != open {
        capture.0 = open;
    }
    let want = if open { lines() } else { Vec::new() };
    if notices.menu != want {
        notices.menu = want;
    }
}

/// The wingman's answers: spoken over the radio and shown on the HUD.
fn answers(
    mut combat: ResMut<Combat>,
    mut atc: Option<ResMut<crate::atc::AtcState>>,
    mut msgs: ResMut<crate::hud::HudMessages>,
) {
    let replies: Vec<WingReply> = std::mem::take(&mut combat.wing_replies);
    for reply in replies {
        let text = tf!(reply.id()).to_string();
        let from = tf!("radio-wing-callsign").to_string();
        if !atc.as_mut().is_some_and(|a| a.radio(&from, text.clone(), None)) {
            msgs.show(format!("{from}: {text}"));
        }
    }
}

fn close(
    mut menu: ResMut<RadioMenu>,
    mut capture: ResMut<RadioCapture>,
    mut notices: ResMut<crate::player_notices::PlayerNotices>,
) {
    menu.open = None;
    capture.0 = false;
    notices.menu.clear();
}
