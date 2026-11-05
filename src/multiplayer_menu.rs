//! The menu's MULTIPLAYER window: coop in the dynamic campaign. Join a server by its address,
//! or host a campaign here and fly in it (a listen server). A dedicated server is started
//! from the command line (`--dedicated`, `dedicated.rs`).
//!
//! The browser build only joins, and only servers on the public list: it has no UDP, so it goes
//! through the master server's relay (docs/web.md).
//!
//! The host's rules (realism, missile guidance, crashes, time of day, realistic weapons,
//! friendly fire) are
//! everybody's; each player keeps his own cockpit, flight model, avionics, HUD and graphics,
//! and picks his aircraft for each sortie.

use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};
use std::sync::{Arc, Mutex};
use stealth_campaign::{CampaignSetup, PlayerId};
use stealth_host::{Host, save};
use stealth_i18n::{t, t_lines};
use stealth_master_proto::ServerEntry;
use stealth_net::client::{Client, ClientEvent};
use stealth_net::server::Server;

use crate::campaign::{ActiveCampaign, Role};
use crate::layout::{Flow, Span, Split, fitted, screen_window};
use crate::menu::{
    AMBER, BRIGHT, DIM, GREEN, MenuState, THEATERS, big_button, dropdown, icon, icon_text, section,
};
use crate::mission::MissionData;
use crate::net::{NetLink, Notice};
use crate::settings::{AppState, MissionSetup};
use crate::text_entry::{Charset, text_entry};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Page {
    #[default]
    Join,
    Host,
}

#[derive(Resource, Default)]
pub struct MultiplayerWindow {
    pub open: bool,
    page: Page,
    /// Connecting to a server (until it takes the player in).
    joining: Option<(Client, String)>,
    status: Option<String>,
    /// What went wrong, for the popup (`multiplayer_window` stamps it with the time it came).
    error: Option<String>,
    /// The popup: the error and when it came (egui time, seconds).
    popup: Option<(String, f64)>,
    /// The name is missing or refused: the name area flashes (since when, egui time).
    flash_name: bool,
    flash_since: Option<f64>,
    saves: Vec<(String, String, f64)>,
    /// The save whose DELETE was clicked: (name, label), awaiting confirmation.
    confirm_delete: Option<(String, String)>,
    sky_ratio: f32,
    start_hour: f32,
    /// No weather: the classic clear sky.
    clear_sky: bool,
    /// `--host`: host a new campaign at once.
    auto_host: bool,
    /// `--host-save <name>`: host that saved campaign at once.
    auto_host_save: Option<String>,
    /// The public server list: as last fetched, being fetched, or why not.
    servers: Vec<ServerEntry>,
    fetching: Option<stealth_net::master::ListSlot>,
    list_error: Option<String>,
    listed_once: bool,
}

impl MultiplayerWindow {
    pub fn show(&mut self) {
        self.open = true;
        self.saves = save::list_saves();
        self.confirm_delete = None;
        self.error = None;
        self.status = None;
        if self.sky_ratio == 0.0 {
            self.sky_ratio = 24.0;
            self.start_hour = 8.0;
        }
        if !self.listed_once {
            self.listed_once = true;
            self.refresh_list();
        }
    }

    /// Fetches the public server list (in the background; `multiplayer_window` takes the
    /// answer).
    fn refresh_list(&mut self) {
        if self.fetching.is_some() {
            return;
        }
        let answer = Arc::new(Mutex::new(None));
        stealth_net::master::fetch_list_into(&master_url(), answer.clone());
        self.fetching = Some(answer);
        self.list_error = None;
    }
}

/// The public server list's master server (`--master <url>` for tests).
#[cfg(not(target_arch = "wasm32"))]
pub fn master_url() -> String {
    let args: Vec<String> = std::env::args().collect();
    args.iter()
        .position(|a| a == "--master")
        .and_then(|i| args.get(i + 1))
        .cloned()
        .unwrap_or_else(|| stealth_net::master::DEFAULT_MASTER.to_string())
}

/// The public server list's master server (`?master=<url>` on the page for tests).
#[cfg(target_arch = "wasm32")]
pub fn master_url() -> String {
    page_parameter("master").unwrap_or_else(|| stealth_net::master::DEFAULT_MASTER.to_string())
}

/// A parameter in the page's URL ("?master=...&relay=...").
#[cfg(target_arch = "wasm32")]
fn page_parameter(name: &str) -> Option<String> {
    let search = web_sys::window().and_then(|w| w.location().search().ok())?;
    let prefix = format!("{name}=");
    search
        .trim_start_matches('?')
        .split('&')
        .find_map(|kv| kv.strip_prefix(prefix.as_str()))
        .and_then(|v| js_sys::decode_uri_component(v).ok())
        .map(String::from)
}

pub struct MultiplayerMenuPlugin;

impl Plugin for MultiplayerMenuPlugin {
    fn build(&self, app: &mut App) {
        let open = std::env::args().any(|a| a == "--open-multiplayer");
        let mut w = MultiplayerWindow::default();
        if open {
            w.show();
        }
        app.insert_resource(w)
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                multiplayer_window.run_if(in_state(AppState::Menu)),
            )
            .add_systems(Update, test_args.run_if(in_state(AppState::Menu)));
    }
}

/// `--connect <host[:port]>` joins a server, `--host` serves a new campaign (in `--theater`,
/// with `--friendly-fire`, `--host-sky-ratio <r>`, `--host-clear-sky`), `--host-save <name>` a
/// saved one, all as `--name <pilot>` (else the saved name): test aids, and how JOIN IN VR and
/// HOST IN VR start the game again in the headset.
fn test_args(
    mut done: Local<bool>,
    mut w: ResMut<MultiplayerWindow>,
    mut state: ResMut<MenuState>,
) {
    if std::mem::replace(&mut *done, true) {
        return;
    }
    let args: Vec<String> = std::env::args().collect();
    let get = |f: &str| args.iter().position(|a| a == f).and_then(|i| args.get(i + 1)).cloned();
    if let Some(name) = get("--name") {
        state.player_name = name;
    }
    if let Some(addr) = get("--connect") {
        w.show();
        state.server_addr = addr.clone();
        connect(&mut w, &addr, &state.player_name);
    }
    if let Some(code) = get("--theater")
        && let Some(i) = THEATERS.iter().position(|t| t.0.eq_ignore_ascii_case(&code))
    {
        state.theater = i;
    }
    // `--open-host`: the host page, not hosted (screenshots).
    if args.iter().any(|a| a == "--open-host") {
        w.show();
        w.page = Page::Host;
    }
    if args.iter().any(|a| a == "--host") {
        state.friendly_fire |= args.iter().any(|a| a == "--friendly-fire");
        w.show();
        w.page = Page::Host;
        w.auto_host = true;
    }
    if let Some(name) = get("--host-save") {
        w.show();
        w.page = Page::Host;
        w.auto_host_save = Some(name);
    }
    if let Some(r) = get("--host-sky-ratio").and_then(|r| r.parse().ok()) {
        w.sky_ratio = r;
    }
    w.clear_sky |= args.iter().any(|a| a == "--host-clear-sky");
}

/// The rules of a campaign hosted here.
fn host_rules(settings: &crate::settings::Settings, state: &MenuState) -> stealth_host::Rules {
    stealth_host::Rules {
        friendly_fire: state.friendly_fire,
        airframe_losses: state.airframe_losses,
        no_ai_missions: !state.ai_missions,
        ..crate::campaign::rules_of(settings, state.lockheed)
    }
}

fn connect(w: &mut MultiplayerWindow, addr: &str, name: &str) {
    w.error = None;
    match open(w, addr, name.trim()) {
        Ok(c) => {
            w.status = Some(t!("mp-connecting", address = addr));
            w.joining = Some((c, addr.to_string()));
        }
        Err(e) => w.error = Some(format!("{addr}: {e}")),
    }
}

/// The server turned the pilot's name away (`stealth_net::server`: none, or already playing).
fn name_refused(why: &stealth_i18n::Text) -> bool {
    why.is("net-no-name") || why.is("net-name-taken") || {
        // An older server's reason, in English.
        let s = why.tr_in(stealth_i18n::Language::En);
        s == "No name" || s.ends_with(" is already playing")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn open(_w: &MultiplayerWindow, addr: &str, name: &str) -> std::io::Result<Client> {
    Client::connect(addr, name)
}

/// Through the master server's relay to a server on the list: by its address or name.
#[cfg(target_arch = "wasm32")]
fn open(w: &MultiplayerWindow, addr: &str, name: &str) -> std::io::Result<Client> {
    let want = addr.trim();
    let with_port = format!("{want}:{}", stealth_net::DEFAULT_PORT);
    let entry = w
        .servers
        .iter()
        .find(|s| s.addr == want || s.addr == with_port || s.report.name.eq_ignore_ascii_case(want))
        .ok_or_else(|| std::io::Error::other(t!("mp-browser-public-only")))?;
    // `?relay=ws://...` for a relay on a port of its own (tests without a proxy).
    let relay = page_parameter("relay").unwrap_or_else(master_url);
    Client::relayed(&stealth_net::master::relay_url(&relay, &entry.id), name)
}

#[allow(clippy::too_many_arguments)]
fn multiplayer_window(
    mut commands: Commands,
    mut contexts: EguiContexts,
    mut w: ResMut<MultiplayerWindow>,
    mut state: ResMut<MenuState>,
    data: Option<Res<MissionData>>,
    tables: Res<crate::flight::SimTables>,
    mut setup: ResMut<MissionSetup>,
    mut next: ResMut<NextState<AppState>>,
    mut settings: ResMut<crate::settings::Settings>,
    mut notice: ResMut<Notice>,
    vr: Res<crate::menu::VrAvailable>,
    mut exit: MessageWriter<AppExit>,
) -> Result {
    // A coop game that ended brings the menu back here, with why.
    if let Some(n) = notice.0.take() {
        w.show();
        w.error = Some(n);
    }
    if !w.open {
        return Ok(());
    }
    let Some(data) = data else { return Ok(()) };
    let ctx = contexts.ctx_mut()?;
    let escape = ctx.input(|i| i.key_pressed(egui::Key::Escape));
    // Escape cancels the delete confirmation first.
    let mut close = escape && w.joining.is_none() && w.confirm_delete.is_none();
    let mut join = false;
    let auto_host = std::mem::take(&mut w.auto_host);
    let auto_host_save = w.auto_host_save.take();
    // JOIN IN VR / HOST IN VR: not already in the headset, not in the browser or on Android.
    let can_vr = vr.0 && !settings.vr && !cfg!(any(target_arch = "wasm32", target_os = "android"));
    // The game started again in VR does what these say in its multiplayer menu.
    let mut relaunch: Option<Vec<String>> = None;
    let mut host: Option<Result<(String, Host), String>> = None;
    // The public server list, when it came.
    let came = w.fetching.as_ref().and_then(|a| a.lock().ok().and_then(|mut a| a.take()));
    match came {
        Some(Ok(list)) => {
            w.servers = list;
            w.fetching = None;
        }
        Some(Err(e)) => {
            w.list_error = Some(e);
            w.fetching = None;
        }
        None => {}
    }
    // Joining: what the server says.
    if let Some((client, addr)) = w.joining.as_mut() {
        let now = web_time::Instant::now();
        let events = client.poll(now);
        client.flush(now);
        let addr = addr.clone();
        let mut events = events.into_iter();
        while let Some(e) = events.next() {
            match e {
                ClientEvent::Connected => {
                    w.status = Some(t!("mp-joining", address = addr.as_str()))
                }
                ClientEvent::Welcome { id, server, rules, state: st, resume } => {
                    let Some((client, _)) = w.joining.take() else { break };
                    match ActiveCampaign::client(server, *st, &data, rules, id) {
                        Ok(mut c) => {
                            c.resume = resume;
                            // What came with the welcome (the first full world state) is the
                            // game's: lost, the client would never get it again.
                            let rest: Vec<ClientEvent> = events.by_ref().collect();
                            commands.queue(move |world: &mut World| {
                                let mut inbox = world.resource_mut::<crate::net::NetInbox>();
                                for e in rest {
                                    inbox.route(e);
                                }
                            });
                            commands.insert_resource(NetLink::Client(client));
                            crate::campaign::start(&mut commands, c, &mut setup, &tables.start_dir);
                            w.open = false;
                            w.status = None;
                            next.set(AppState::Flight);
                        }
                        Err(e) => w.error = Some(format!("{e:#}")),
                    }
                    break;
                }
                ClientEvent::Closed(why) => {
                    w.joining = None;
                    w.status = None;
                    w.flash_name |= name_refused(&why);
                    w.error = Some(format!("{addr}: {why}"));
                }
                _ => {}
            }
        }
    }
    let now = ctx.input(|i| i.time);
    if let Some(e) = w.error.take() {
        w.popup = Some((e, now));
    }
    if std::mem::take(&mut w.flash_name) {
        w.flash_since = Some(now);
    }
    // Three flashes a second, for a second.
    let flash = w.flash_since.is_some_and(|t| now - t < 1.0 && ((now - t) * 3.0).fract() < 0.5);
    let mut no_name = false;
    // The server list takes the room the window leaves.
    let list_height = (fitted(ctx, MULTIPLAYER_SIZE).y - 330.0).max(160.0);
    let title = t!(if w.page == Page::Host { "mp-title-host" } else { "menu-multiplayer" });
    let panel = crate::menu::Panel::new(title).note(t!("mp-note"));
    let window = screen_window(ctx, "Multiplayer", MULTIPLAYER_SIZE);
    panel.window(window, ctx, |ui| {
        let named = !state.player_name.trim().is_empty();
        let split = Split::new(ui, "multiplayer");
        let compact = split.compact();
        split.body(ui, |ui| {
            let mut flow = Flow::new(ui);
            flow.card(ui, Span::Fixed(NAME_WIDTH), |ui| {
                area(ui, flash, |ui| {
                    ui.set_max_width(340.0);
                    section(ui, t!("mp-your-name"));
                    text_entry(
                        ui,
                        "mp_name",
                        &mut state.player_name,
                        24,
                        Charset::Name,
                        settings.vr,
                        t!("mp-your-name.hint"),
                    );
                });
            });
            if w.page == Page::Host {
                flow.card(ui, Span::Fill(SERVER_LEAST), |ui| {
                    area(ui, false, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(t!("mp-server-name"));
                            let hint = server_title(&state);
                            text_entry(
                                ui,
                                "mp_server",
                                &mut state.server_name,
                                32,
                                Charset::Name,
                                settings.vr,
                                &hint,
                            );
                        });
                        ui.horizontal_wrapped(|ui| {
                            ui.label(t!("mp-udp-port"));
                            ui.add(egui::DragValue::new(&mut state.host_port).range(1024..=65535));
                            ui.add_space(20.0);
                            ui.label(t!("ops-section-players"));
                            ui.add(egui::Slider::new(
                                &mut state.max_players,
                                2..=stealth_net::protocol::MAX_PLAYERS as u8,
                            ));
                        });
                        ui.label(egui::RichText::new(t!("mp-address-note")).small().color(DIM));
                        ui.checkbox(&mut state.list_public, t!("mp-list-public"));
                    });
                });
            }
            if w.page == Page::Join {
                flow.card(ui, Span::Fill(JOIN_LEAST), |ui| {
                    area(ui, false, |ui| {
                        section(ui, t!("mp-join-address"));
                        // Wrapped: the buttons go under the address in a narrow card.
                        ui.horizontal_wrapped(|ui| {
                            text_entry(
                                ui,
                                "mp_server",
                                &mut state.server_addr,
                                64,
                                Charset::Address,
                                settings.vr,
                                t!("mp-join-address.hint"),
                            );
                            if w.joining.is_some() {
                                ui.add(egui::Spinner::new().color(GREEN));
                                if ui.button(icon_text(ui, icon::CANCEL, t!("cancel"))).clicked() {
                                    if let Some((mut c, _)) = w.joining.take() {
                                        c.leave();
                                    }
                                    w.status = None;
                                }
                            } else if ui
                                .add_enabled(
                                    !state.server_addr.trim().is_empty(),
                                    egui::Button::new(icon_text(ui, icon::JOIN, t!("mp-connect"))),
                                )
                                .clicked()
                            {
                                if named {
                                    join = true;
                                } else {
                                    no_name = true;
                                }
                            }
                            if can_vr
                                && w.joining.is_none()
                                && ui
                                    .add_enabled(
                                        !state.server_addr.trim().is_empty(),
                                        egui::Button::new(icon_text(
                                            ui,
                                            icon::VR,
                                            t!("mp-join-vr"),
                                        )),
                                    )
                                    .on_hover_text(t!("mp-join-vr.facts"))
                                    .clicked()
                            {
                                if named {
                                    let addr = state.server_addr.trim().to_string();
                                    relaunch = Some(vec!["--connect".into(), addr]);
                                } else {
                                    no_name = true;
                                }
                            }
                        });
                        if let Some(s) = &w.status {
                            ui.label(egui::RichText::new(s).small().color(BRIGHT));
                        }
                    });
                });
            }
            flow.finish(ui);
            ui.separator();
            match w.page {
                Page::Join => {
                    if let Some((addr, in_vr)) = server_list(ui, &mut w, list_height, can_vr) {
                        if !named {
                            no_name = true;
                        } else if in_vr {
                            relaunch = Some(vec!["--connect".into(), addr.clone()]);
                            state.server_addr = addr;
                        } else {
                            state.server_addr = addr;
                            join = true;
                        }
                    }
                }
                Page::Host => {
                    let mut flow = Flow::new(ui);
                    flow.card(ui, Span::Fixed(210.0), |ui| {
                        crate::menu::theater_options(ui, &mut state.theater, compact);
                    });
                    flow.card(ui, Span::Fixed(220.0), |ui| {
                        crate::menu::opponent_options(ui, &mut state.opponents);
                        crate::menu::situation_options(ui, &mut state.roe);
                        section(ui, t!("section-weather"));
                        dropdown(
                            ui,
                            "mp_weather",
                            &mut w.clear_sky,
                            [
                                (
                                    false,
                                    t!("cm-weather-dynamic"),
                                    t_lines!("cm-weather-dynamic.facts"),
                                ),
                                (true, t!("cm-weather-clear"), t_lines!("cm-weather-clear.facts")),
                            ],
                        );
                        section(ui, t!("cm-section-day-night"));
                        dropdown(
                            ui,
                            "mp_sky_ratio",
                            &mut w.sky_ratio,
                            [
                                (1.0, t!("cm-day-real"), t_lines!("cm-day-real.facts")),
                                (12.0, t!("cm-day-2h"), t_lines!("cm-day-2h.facts")),
                                (24.0, t!("cm-day-1h"), t_lines!("cm-day-1h.facts")),
                                (48.0, t!("cm-day-30min"), t_lines!("cm-day-30min.facts")),
                            ],
                        );
                    });
                    flow.card(ui, Span::Fixed(230.0), |ui| {
                        section(ui, t!("mp-rules"));
                        crate::menu::realism_options(ui, &mut settings);
                        ui.checkbox(&mut settings.continuous_time, t!("mp-time-of-day"));
                        ui.checkbox(&mut state.lockheed, t!("mp-realistic-weapons"));
                        ui.checkbox(&mut state.friendly_fire, t!("mp-friendly-fire"));
                        ui.checkbox(&mut state.airframe_losses, t!("mp-airframe-losses"));
                        ui.checkbox(&mut state.ai_missions, t!("mp-ai-missions"));
                    });
                    flow.card(ui, Span::Fill(NEW_LEAST), |ui| {
                        section(ui, t!("cm-section-new"));
                        ui.label(crate::i18n::theater_name(state.theater.min(THEATERS.len() - 1)));
                        // Wrapped: HOST IN VR goes under HOST NEW in a narrow card.
                        let (clicked, in_vr) = ui
                            .horizontal_wrapped(|ui| {
                                let clicked = ui
                                    .add(big_button(t!("mp-host-new")).icon(icon::HOST))
                                    .clicked();
                                let in_vr = can_vr
                                    && ui
                                        .add(big_button(t!("mp-host-vr")).icon(icon::VR))
                                        .on_hover_text(t!("mp-host-vr.facts"))
                                        .clicked();
                                (clicked, in_vr)
                            })
                            .inner;
                        no_name |= (clicked || in_vr) && !named;
                        if in_vr && named {
                            let mut args = vec!["--host".to_string(), "--host-sky-ratio".into()];
                            args.push(w.sky_ratio.to_string());
                            if w.clear_sky {
                                args.push("--host-clear-sky".into());
                            }
                            relaunch = Some(args);
                        }
                        if (clicked && named) || auto_host {
                            let (code, _) = THEATERS[state.theater.min(THEATERS.len() - 1)];
                            let secs = web_time::SystemTime::now()
                                .duration_since(web_time::UNIX_EPOCH)
                                .map_or(0, |d| d.as_secs());
                            let setup = CampaignSetup {
                                theater: state.theater,
                                theater_code: code.to_string(),
                                roe: state.roe,
                                opponents: state.opponents,
                                // Test aid: `--host --seed <n>`.
                                seed: std::env::args()
                                    .skip_while(|a| a != "--seed")
                                    .nth(1)
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or_else(rand::random),
                                sky_ratio: w.sky_ratio,
                                start_hour: w.start_hour,
                                dynamic_weather: !w.clear_sky,
                                crew_scaling: true,
                            };
                            let rules = host_rules(&settings, &state);
                            host = Some(
                                Host::new(setup, &data.0, rules)
                                    .map(|h| (format!("coop-{code}-{secs}"), h))
                                    .map_err(|e| format!("{e:#}")),
                            );
                        }
                        ui.add_space(6.0);
                        section(ui, t!("mp-host-saved"));
                        ui.label(egui::RichText::new(t!("cm-rules-note")).small().color(DIM));
                        if w.saves.is_empty() {
                            ui.label(egui::RichText::new(t!("cm-no-saves")).color(DIM));
                        }
                        let scroll = egui::ScrollArea::vertical()
                            .id_salt("mp_saves")
                            .max_height(crate::campaign::SAVES_HEIGHT);
                        scroll.show(ui, |ui| {
                            for (name, code, hours) in w.saves.clone() {
                                let theater = THEATERS
                                    .iter()
                                    .find(|t| t.0 == code)
                                    .map_or(code.as_str(), |t| t.1);
                                // Wrapped: the buttons go under the label in a narrow column.
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(format!("{theater:<22} {hours:>5.1} h"));
                                    let clicked = ui
                                        .button(icon_text(ui, icon::HOST, t!("mp-host")))
                                        .clicked();
                                    let in_vr = can_vr
                                        && ui
                                            .button(icon_text(ui, icon::VR, t!("cm-in-vr")))
                                            .on_hover_text(t!("mp-host-vr.facts"))
                                            .clicked();
                                    no_name |= (clicked || in_vr) && !named;
                                    if in_vr && named {
                                        relaunch = Some(vec!["--host-save".into(), name.clone()]);
                                    }
                                    let auto = auto_host_save.as_ref() == Some(&name);
                                    if (clicked && named) || auto {
                                        let rules = host_rules(&settings, &state);
                                        host = Some(match save::load(&name) {
                                            Ok(Some(st)) => Host::resume(st, &data.0, rules)
                                                .map(|h| (name.clone(), h))
                                                .map_err(|e| format!("{e:#}")),
                                            Ok(None) => Err(t!("cm-save-gone").into()),
                                            Err(e) => Err(format!("{e:#}")),
                                        });
                                    }
                                    if ui
                                        .button(icon_text(ui, icon::DELETE, t!("delete")))
                                        .on_hover_text(t!("cm-delete.facts"))
                                        .clicked()
                                    {
                                        let played = t!("cm-played", hours = format!("{hours:.1}"));
                                        w.confirm_delete =
                                            Some((name.clone(), format!("{theater}, {played}")));
                                    }
                                });
                            }
                        });
                    });
                    flow.finish(ui);
                }
            }
        });
        split.footer(ui, |ui| {
            crate::menu::footer_rule(ui);
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .add_enabled(w.joining.is_none(), big_button(t!("back")).icon(icon::BACK))
                    .clicked()
                {
                    close = true;
                }
                // A browser can't host (no UDP).
                if w.page == Page::Join && !cfg!(target_arch = "wasm32") {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_enabled(
                                w.joining.is_none(),
                                big_button(t!("mp-host-game")).icon(icon::HOST),
                            )
                            .clicked()
                        {
                            w.page = Page::Host;
                        }
                    });
                }
            });
        });
    });
    if no_name {
        w.flash_since = Some(now);
    }
    if let Some((name, label)) = w.confirm_delete.clone() {
        let answer = crate::campaign::delete_popup(ctx, "campaign", &label, escape);
        if answer == Some(true) {
            if let Err(e) = save::delete(&name) {
                w.error = Some(t!("cm-save-undeletable", error = format!("{e:#}")));
            }
            w.saves = save::list_saves();
        }
        if answer.is_some() {
            w.confirm_delete = None;
        }
    }
    if let Some((e, since)) = &w.popup {
        if now - since < POPUP_SECONDS {
            error_popup(ctx, e);
        } else {
            w.popup = None;
        }
    }
    // BACK from hosting goes back to the server list.
    if close && w.page == Page::Host {
        close = false;
        w.page = Page::Join;
    }
    if join {
        let (addr, name) = (state.server_addr.clone(), state.player_name.clone());
        connect(&mut w, &addr, &name);
    }
    if let Some(args) = relaunch {
        match crate::menu::relaunch_multiplayer_in_vr(&args, &state, &settings) {
            Ok(()) => {
                exit.write(AppExit::Success);
            }
            Err(e) => w.error = Some(t!("menu-vr-failed", error = e.to_string())),
        }
    }
    match host {
        Some(Ok((name, h))) => {
            let title = server_title(&state);
            match Server::new(
                &title,
                state.host_port,
                state.max_players as usize,
                Some(PlayerId(0)),
            ) {
                Ok(mut server) => {
                    if state.list_public {
                        server.list_on(&master_url());
                    }
                    let c = ActiveCampaign::with_host(name, h, Role::Host);
                    c.save();
                    commands.insert_resource(NetLink::Host(server));
                    crate::campaign::start(&mut commands, c, &mut setup, &tables.start_dir);
                    w.open = false;
                    next.set(AppState::Flight);
                }
                Err(e) => {
                    w.error =
                        Some(t!("mp-port-error", port = state.host_port, error = e.to_string()))
                }
            }
        }
        Some(Err(e)) => w.error = Some(e),
        None => {}
    }
    if close {
        w.open = false;
    }
    Ok(())
}

/// The name the server goes by: the one given, else "<player>'s campaign".
fn server_title(state: &crate::menu::MenuState) -> String {
    match state.server_name.trim() {
        "" => t!("mp-default-server", pilot = state.player_name.trim()),
        name => name.to_string(),
    }
}

/// How long an error shows in its popup (seconds).
const POPUP_SECONDS: f64 = 3.0;

/// The window's size, frame included, and the least width of the new campaign's column.
const MULTIPLAYER_SIZE: egui::Vec2 = egui::vec2(1132.0, 740.0);
const NEW_LEAST: f32 = 280.0;
/// The width of the name's frame, and the least width of the one to join by address.
const NAME_WIDTH: f32 = 372.0;
const JOIN_LEAST: f32 = 540.0;
/// The least width of the server's settings beside the name.
const SERVER_LEAST: f32 = 420.0;

/// An error in a popup at the top right of the screen.
fn error_popup(ctx: &egui::Context, text: &str) {
    egui::Area::new(egui::Id::new("mp_error"))
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-16.0, 16.0))
        .order(egui::Order::Foreground)
        .interactable(false)
        .show(ctx, |ui| {
            egui::Frame::window(ui.style())
                .stroke(egui::Stroke::new(2.0, AMBER))
                .inner_margin(12.0)
                .show(ui, |ui| {
                    ui.set_max_width(420.0);
                    ui.label(egui::RichText::new(text).color(AMBER));
                });
        });
}

/// A small framed area of the window, lit up when `flash`.
fn area(ui: &mut egui::Ui, flash: bool, add: impl FnOnce(&mut egui::Ui)) {
    let (stroke, fill) = if flash {
        (egui::Stroke::new(1.0, AMBER), AMBER.gamma_multiply(0.35))
    } else {
        (egui::Stroke::new(1.0, crate::menu::FRAME), egui::Color32::TRANSPARENT)
    };
    egui::Frame::group(ui.style()).stroke(stroke).fill(fill).inner_margin(10.0).show(ui, add);
}

/// The public server list on the Join page, `height` tall: a server's JOIN gives its address.
/// The public servers; the address of the one whose JOIN was clicked, and whether to join it
/// in VR (`vr`: the VR button is there).
fn server_list(
    ui: &mut egui::Ui,
    w: &mut MultiplayerWindow,
    height: f32,
    vr: bool,
) -> Option<(String, bool)> {
    let mut pick = None;
    ui.horizontal(|ui| {
        section(ui, t!("mp-public-servers"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if w.fetching.is_some() {
                ui.add(egui::Spinner::new().color(GREEN));
            } else if ui.button(icon_text(ui, icon::REFRESH, t!("mp-refresh"))).clicked() {
                w.refresh_list();
            }
            let n: usize = w.servers.iter().map(|s| s.report.players.len()).sum();
            if !w.servers.is_empty() {
                ui.label(
                    egui::RichText::new(t!(
                        "mp-server-count",
                        servers = w.servers.len(),
                        pilots = n
                    ))
                    .color(DIM),
                );
            }
        });
    });
    if let Some(e) = &w.list_error {
        ui.label(egui::RichText::new(t!("mp-list-error", error = e.as_str())).color(AMBER));
    } else if w.servers.is_empty() && w.fetching.is_none() {
        ui.label(egui::RichText::new(t!("mp-no-servers")).color(DIM));
    }
    let list = egui::ScrollArea::vertical().max_height(height).min_scrolled_height(height);
    list.auto_shrink([false, false]).show(ui, |ui| {
        // Two lines a server (its name and pilots, then its theater and war), the buttons at
        // the right: as narrow as the window, never wider.
        for (i, s) in w.servers.iter().enumerate() {
            let r = &s.report;
            let ours = r.protocol == stealth_net::PROTOCOL;
            let full = r.max_players > 0 && r.players.len() >= r.max_players as usize;
            let col = if ours { BRIGHT } else { DIM };
            let fill =
                if i % 2 == 0 { ui.visuals().faint_bg_color } else { egui::Color32::TRANSPARENT };
            egui::Frame::new().fill(fill).inner_margin(egui::Margin::symmetric(4, 3)).show(
                ui,
                |ui| {
                    ui.set_width(ui.available_width());
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        if !ours {
                            ui.label(
                                egui::RichText::new(t!("mp-other-version")).small().color(DIM),
                            );
                        } else {
                            let can = !full && w.joining.is_none();
                            // Right to left: VR right of JOIN.
                            if vr
                                && ui
                                    .add_enabled(
                                        can,
                                        egui::Button::new(icon_text(ui, icon::VR, "VR")),
                                    )
                                    .on_hover_text(t!("mp-join-vr.facts"))
                                    .clicked()
                            {
                                pick = Some((s.addr.clone(), true));
                            }
                            let join = egui::Button::new(icon_text(ui, icon::JOIN, t!("mp-join")));
                            if ui.add_enabled(can, join).clicked() {
                                pick = Some((s.addr.clone(), false));
                            }
                        }
                        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                            ui.horizontal_wrapped(|ui| {
                                ui.label(egui::RichText::new(&r.name).strong().color(col))
                                    .on_hover_text(s.addr.as_str());
                                if r.dedicated {
                                    ui.label(
                                        egui::RichText::new(t!("mp-dedicated")).small().color(DIM),
                                    );
                                }
                                ui.add_space(8.0);
                                ui.label(
                                    egui::RichText::new(t!("mp-col-pilots")).small().color(DIM),
                                );
                                let count = format!("{}/{}", r.players.len(), r.max_players);
                                let count_col = if full { AMBER } else { GREEN };
                                ui.label(egui::RichText::new(count).monospace().color(count_col));
                            });
                            ui.horizontal_wrapped(|ui| {
                                ui.label(egui::RichText::new(&r.theater).color(col));
                                ui.label(
                                    egui::RichText::new(format!("{}, {}", r.situation, r.clock))
                                        .color(DIM),
                                );
                            });
                        });
                    });
                },
            );
        }
    });
    ui.add_space(6.0);
    pick
}
