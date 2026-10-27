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
use stealth_i18n::t;
use stealth_master_proto::ServerEntry;
use stealth_net::client::{Client, ClientEvent};
use stealth_net::server::Server;

use crate::campaign::{ActiveCampaign, Role};
use crate::menu::{AMBER, BRIGHT, DIM, GREEN, MenuState, THEATERS, big_button, section};
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

/// Test aids: `--connect <host[:port]>` joins a server, `--host` serves a new campaign (in
/// `--theater`, with `--friendly-fire`), both as `--name <pilot>` (else the saved name).
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
    if args.iter().any(|a| a == "--host") {
        state.friendly_fire |= args.iter().any(|a| a == "--friendly-fire");
        w.show();
        w.page = Page::Host;
        w.auto_host = true;
    }
}

/// The rules of a campaign hosted here.
fn host_rules(settings: &crate::settings::Settings, state: &MenuState) -> stealth_host::Rules {
    stealth_host::Rules {
        friendly_fire: state.friendly_fire,
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
    let width = crate::menu::wide_width(ctx).min(1100.0);
    // The server list takes the room the screen leaves.
    let list_height = (ctx.content_rect().height() - 330.0).max(160.0);
    crate::menu::menu_window(ctx, "Multiplayer", width).show(ctx, |ui| {
        let title = t!(if w.page == Page::Host { "mp-title-host" } else { "menu-multiplayer" });
        ui.label(egui::RichText::new(title).heading().color(BRIGHT));
        ui.label(egui::RichText::new(t!("mp-subtitle")).color(DIM));
        ui.add_space(6.0);
        let named = !state.player_name.trim().is_empty();
        ui.horizontal_top(|ui| {
            area(ui, flash, |ui| {
                ui.set_max_width(340.0);
                section(ui, t!("mp-your-name"));
                let hint = t!("mp-your-name.hint");
                text_entry(
                    ui,
                    "mp_name",
                    &mut state.player_name,
                    24,
                    Charset::Name,
                    settings.vr,
                    hint,
                );
            });
            if w.page != Page::Join {
                return;
            }
            area(ui, false, |ui| {
                section(ui, t!("mp-join-address"));
                ui.horizontal(|ui| {
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
                        if ui.button(t!("cancel")).clicked() {
                            if let Some((mut c, _)) = w.joining.take() {
                                c.leave();
                            }
                            w.status = None;
                        }
                    } else if ui
                        .add_enabled(
                            !state.server_addr.trim().is_empty(),
                            egui::Button::new(t!("mp-connect")),
                        )
                        .clicked()
                    {
                        if named {
                            join = true;
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
        ui.separator();
        match w.page {
            Page::Join => {
                if let Some(addr) = server_list(ui, &mut w, list_height) {
                    if named {
                        state.server_addr = addr;
                        join = true;
                    } else {
                        no_name = true;
                    }
                }
            }
            Page::Host => {
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
                ui.horizontal(|ui| {
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
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.set_width(210.0);
                        crate::menu::theater_options(ui, &mut state.theater);
                    });
                    ui.vertical(|ui| {
                        ui.set_width(200.0);
                        crate::menu::opponent_options(ui, &mut state.opponents);
                        crate::menu::situation_options(ui, &mut state.roe);
                    });
                    ui.vertical(|ui| {
                        ui.set_width(230.0);
                        section(ui, t!("mp-rules"));
                        crate::menu::realism_options(ui, &mut settings);
                        ui.checkbox(&mut settings.continuous_time, t!("mp-time-of-day"));
                        ui.checkbox(&mut state.lockheed, t!("mp-realistic-weapons"));
                        ui.checkbox(&mut state.friendly_fire, t!("mp-friendly-fire"));
                    });
                    ui.vertical(|ui| {
                        section(ui, t!("cm-section-new"));
                        ui.label(crate::i18n::theater_name(state.theater.min(THEATERS.len() - 1)));
                        ui.horizontal(|ui| {
                            ui.label(t!("mp-day-night"));
                            let ratios = [
                                (1.0, t!("cm-day-real")),
                                (24.0, t!("cm-day-1h")),
                                (48.0, t!("cm-day-30min")),
                            ];
                            for (ratio, label) in ratios {
                                if ui.selectable_label(w.sky_ratio == ratio, label).clicked() {
                                    w.sky_ratio = ratio;
                                }
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label(t!("mp-weather"));
                            for (clear, label) in
                                [(false, t!("cm-weather-dynamic")), (true, t!("cm-weather-clear"))]
                            {
                                if ui.selectable_label(w.clear_sky == clear, label).clicked() {
                                    w.clear_sky = clear;
                                }
                            }
                        });
                        let clicked = ui.add(big_button(t!("mp-host-new"))).clicked();
                        no_name |= clicked && !named;
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
                                seed: rand::random(),
                                sky_ratio: w.sky_ratio,
                                start_hour: w.start_hour,
                                dynamic_weather: !w.clear_sky,
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
                                let theater = crate::i18n::theater_name_of(&code);
                                ui.horizontal(|ui| {
                                    ui.label(format!("{theater:<22} {hours:>5.1} h"));
                                    let clicked = ui.button(t!("mp-host")).clicked();
                                    no_name |= clicked && !named;
                                    if clicked && named {
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
                                        .button(t!("delete"))
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
                });
            }
        }
        ui.separator();
        ui.horizontal(|ui| {
            if ui.add_enabled(w.joining.is_none(), big_button(t!("back"))).clicked() {
                close = true;
            }
            // A browser can't host (no UDP).
            if w.page == Page::Join && !cfg!(target_arch = "wasm32") {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add_enabled(w.joining.is_none(), big_button(t!("mp-host-game"))).clicked()
                    {
                        w.page = Page::Host;
                    }
                });
            }
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
        (egui::Stroke::new(1.0, DIM), egui::Color32::TRANSPARENT)
    };
    egui::Frame::group(ui.style()).stroke(stroke).fill(fill).inner_margin(10.0).show(ui, add);
}

/// The public server list on the Join page, `height` tall: a server's JOIN gives its address.
fn server_list(ui: &mut egui::Ui, w: &mut MultiplayerWindow, height: f32) -> Option<String> {
    let mut pick = None;
    ui.horizontal(|ui| {
        section(ui, t!("mp-public-servers"));
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if w.fetching.is_some() {
                ui.add(egui::Spinner::new().color(GREEN));
            } else if ui.button(t!("mp-refresh")).clicked() {
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
        if w.servers.is_empty() {
            return;
        }
        egui::Grid::new("mp_servers")
            .num_columns(5)
            .spacing(egui::vec2(18.0, 6.0))
            .striped(true)
            .show(ui, |ui| {
                for h in ["mp-col-server", "mp-col-pilots", "summary-theater", "mp-col-war"] {
                    ui.label(egui::RichText::new(t!(h)).color(DIM));
                }
                {
                    ui.label("");
                }
                ui.end_row();
                for s in &w.servers {
                    let r = &s.report;
                    let ours = r.protocol == stealth_net::PROTOCOL;
                    let full = r.max_players > 0 && r.players.len() >= r.max_players as usize;
                    let col = if ours { BRIGHT } else { DIM };
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(&r.name).strong().color(col));
                        if r.dedicated {
                            ui.label(egui::RichText::new(t!("mp-dedicated")).small().color(DIM));
                        }
                    })
                    .response
                    .on_hover_text(s.addr.as_str());
                    let count = format!("{}/{}", r.players.len(), r.max_players);
                    ui.label(egui::RichText::new(count).monospace().color(if full {
                        AMBER
                    } else {
                        GREEN
                    }));
                    ui.label(egui::RichText::new(&r.theater).color(col));
                    ui.label(
                        egui::RichText::new(format!("{}, {}", r.situation, r.clock)).color(DIM),
                    );
                    if !ours {
                        ui.label(egui::RichText::new(t!("mp-other-version")).small().color(DIM));
                    } else if ui
                        .add_enabled(!full && w.joining.is_none(), egui::Button::new(t!("mp-join")))
                        .clicked()
                    {
                        pick = Some(s.addr.clone());
                    }
                    ui.end_row();
                }
            });
    });
    ui.add_space(6.0);
    pick
}
