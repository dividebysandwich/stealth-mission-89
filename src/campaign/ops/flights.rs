//! The FLIGHTS tab: every friendly flight in the war, the players' and the AI's, with who sits
//! where; a free seat (an AI aircraft's controls, a gunner's console) to hop into.

use bevy_egui::egui;
use stealth_campaign::{AiAirframe, Aircraft as Airframe, Side};
use stealth_i18n::{Text, t};
use stealth_sim::crew::Seat;

use super::{
    AI_GREY, ActiveCampaign, BRIGHT, DIM, Deed, GREEN, Phase, ROW, Todo, height_of, icon, kind_of,
    section, take_over_refusal,
};

/// A flight of the list: a player's aircraft or one of an AI flight's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlightKey {
    Player(u8),
    Ai(u32),
}

/// Who sits in a seat.
#[derive(Debug, Clone, PartialEq)]
enum Sitter {
    Player(String),
    Ai,
    Free,
}

impl Sitter {
    fn label(&self) -> String {
        match self {
            Sitter::Player(name) => name.clone(),
            Sitter::Ai => t!("ops-seat-ai").to_string(),
            Sitter::Free => t!("ops-seat-free").to_string(),
        }
    }

    fn colour(&self) -> egui::Color32 {
        match self {
            Sitter::Player(_) => BRIGHT,
            Sitter::Ai => AI_GREY,
            Sitter::Free => GREEN,
        }
    }
}

/// One aircraft of the list.
struct Row {
    key: FlightKey,
    title: Text,
    aircraft: Airframe,
    callsign: String,
    status: String,
    pilot: Sitter,
    /// None: it has no gunner's console.
    gunner: Option<Sitter>,
    /// An AI flight's (grey).
    ai: bool,
    /// What can be done with it: take its controls (an AI aircraft in the air), sit at its
    /// gunner's console.
    take: bool,
    join: bool,
}

/// The friendly flights now: the players' sorties, then the AI's missions' aircraft.
fn rows(c: &ActiveCampaign, sim: &stealth_sim::Sim) -> Vec<Row> {
    let st = c.state();
    let mut out = Vec::new();
    for p in &st.players {
        let Some(s) = &p.sortie else { continue };
        let id = p.id.0;
        let name = st.player_name(p.id).unwrap_or("?").to_string();
        let seat = sim.owner_seat(id);
        let pilot = if seat == Seat::Gunner { Sitter::Ai } else { Sitter::Player(name.clone()) };
        let gunner = s.aircraft.gunner_seat().then(|| {
            if seat == Seat::Gunner {
                Sitter::Player(name.clone())
            } else {
                match sim.gunner_of(id) {
                    Some(g) => Sitter::Player(
                        st.player_name(stealth_campaign::PlayerId(g)).unwrap_or("?").to_string(),
                    ),
                    None => Sitter::Free,
                }
            }
        });
        let status = match sim.pilot_flying(id) {
            None => t!("ops-flight-starting").to_string(),
            Some(true) => t!("ops-flight-airborne").to_string(),
            Some(false) => t!("ops-flight-on-ground").to_string(),
        };
        let join = id != c.me.0 && gunner == Some(Sitter::Free) && sim.gunner_seat_free(id);
        out.push(Row {
            key: FlightKey::Player(id),
            title: s.offer.title.clone(),
            aircraft: s.aircraft,
            callsign: if s.flight > 0 {
                format!("{}-{}", s.flight, s.position.max(1))
            } else {
                String::new()
            },
            status,
            pilot,
            gunner,
            ai: false,
            take: false,
            join,
        });
    }
    for m in st.ai_missions.iter().filter(|m| m.side == Side::Blue && m.open()) {
        let AiAirframe::Blue(aircraft) = m.airframe else { continue };
        for (n, &tag) in m.tags.iter().enumerate() {
            let Some((alive, controlled, waiting)) = m.tag_state(tag) else { continue };
            // Lost, or a player's now (his sortie's row).
            if !alive || controlled.is_some() {
                continue;
            }
            let status = if waiting {
                let due = m.due.get(n).copied().flatten().unwrap_or(st.clock);
                let left = (due - st.clock).max(0.0) as i64;
                t!("ops-ai-takes-off", time = format!("{}:{:02}", left / 60, left % 60)).to_string()
            } else {
                t!("ops-flight-airborne").to_string()
            };
            let takeable = !waiting && sim.takeable(tag);
            let gunner = aircraft.gunner_seat().then_some(Sitter::Free);
            out.push(Row {
                key: FlightKey::Ai(tag),
                title: m.offer.title.clone(),
                aircraft,
                callsign: format!("#{}", n + 1),
                status,
                pilot: Sitter::Ai,
                gunner,
                ai: true,
                take: takeable,
                join: takeable && aircraft.gunner_seat(),
            });
        }
    }
    out
}

/// Why this player can't sit at another's gunner's console now (None: they can).
fn crew_refusal(c: &ActiveCampaign) -> Option<String> {
    (c.phase != Phase::Ops || c.state().sortie(c.me).is_some())
        .then(|| t!("ce-finish-sortie").to_string())
}

/// The tab: the list, the selected flight's seats and what can be done with it.
pub(super) fn tab(
    ui: &mut egui::Ui,
    c: &mut ActiveCampaign,
    sim: &stealth_sim::Sim,
    _width: f32,
    deck: &mut Vec<Deed>,
    bar: &mut Option<(String, String)>,
) {
    let list = rows(c, sim);
    if c.ui.flight.is_none_or(|k| !list.iter().any(|r| r.key == k)) {
        c.ui.flight = list.first().map(|r| r.key);
    }
    let room = height_of(ui);
    // The list over the selected flight's seats.
    ui.vertical(|ui| {
        egui::ScrollArea::vertical().id_salt("flights").max_height((room * 0.6).max(120.0)).auto_shrink([false, true]).show(
            ui,
            |ui| {
                for r in &list {
                    let selected = c.ui.flight == Some(r.key);
                    let ink = |col| if selected { super::BG } else { col };
                    let base = if r.ai { AI_GREY } else { BRIGHT };
                    let frame = egui::Frame::new()
                        .fill(if selected { base } else { ROW })
                        .stroke(egui::Stroke::new(1.0, if selected { BRIGHT } else { DIM }))
                        .corner_radius(2.0)
                        .inner_margin(egui::Margin::symmetric(8, 5));
                    let inner = frame.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.spacing_mut().item_spacing.y = 3.0;
                        let main = format!(
                            "{} {:<5} {:<8} {:<32} {}",
                            if selected { '\u{25B6}' } else { ' ' },
                            r.callsign,
                            kind_of(r.aircraft).name(),
                            r.title.tr(),
                            r.status
                        );
                        let text = egui::RichText::new(main.trim_end()).monospace();
                        ui.add(egui::Label::new(text.color(ink(base))).selectable(false));
                        ui.horizontal_wrapped(|ui| {
                            let mut part = |s: String, col| {
                                let text = egui::RichText::new(s).small().color(ink(col));
                                ui.add(egui::Label::new(text).selectable(false));
                            };
                            part(t!("ops-flight-pilot").to_string(), DIM);
                            part(r.pilot.label(), r.pilot.colour());
                            if let Some(g) = &r.gunner {
                                part(t!("ops-flight-gunner").to_string(), DIM);
                                part(g.label(), g.colour());
                            }
                        });
                    });
                    let resp = ui
                        .interact(
                            inner.response.rect,
                            egui::Id::new(("flight", r.key)),
                            egui::Sense::click(),
                        )
                        .on_hover_cursor(egui::CursorIcon::PointingHand);
                    if resp.hovered() && !selected {
                        ui.painter().rect_stroke(
                            resp.rect,
                            2.0,
                            egui::Stroke::new(2.0, base),
                            egui::StrokeKind::Inside,
                        );
                    }
                    if resp.clicked() {
                        c.ui.flight = Some(r.key);
                        c.ui.error = None;
                    }
                    ui.add_space(2.0);
                }
                if list.is_empty() {
                    ui.label(egui::RichText::new(t!("ops-no-flights")).color(DIM));
                }
            },
        );
    });
    ui.add_space(8.0);
    ui.vertical(|ui| {
        let Some(r) = list.iter().find(|r| Some(r.key) == c.ui.flight) else { return };
        *bar = Some((format!("{} · {}", r.callsign, kind_of(r.aircraft).name()), format!("{} · {}", r.title.tr(), r.status)));
        section(ui, kind_of(r.aircraft).name());
        ui.label(egui::RichText::new(r.title.tr()).size(20.0).color(BRIGHT));
        ui.label(egui::RichText::new(&r.status).color(DIM));
        ui.add_space(6.0);
        let seat = |ui: &mut egui::Ui, label: String, who: &Sitter| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(label).color(DIM));
                ui.label(egui::RichText::new(who.label()).color(who.colour()));
            });
        };
        seat(ui, t!("ops-flight-pilot").to_string(), &r.pilot);
        if let Some(g) = &r.gunner {
            seat(ui, t!("ops-flight-gunner").to_string(), g);
        }
        if r.take
            && let FlightKey::Ai(tag) = r.key
        {
            let mut deed = Deed::new(
                Todo::TakeControl { tag, watched: false },
                t!("ops-take-control"),
                icon::JOIN,
                t!("ops-take-control.facts"),
            );
            deed.why = take_over_refusal(c);
            deck.push(deed);
        }
        if r.join {
            let todo = match r.key {
                FlightKey::Ai(tag) => Todo::GunnerAi { tag },
                FlightKey::Player(owner) => Todo::GunnerWith { owner },
            };
            let facts = match r.key {
                FlightKey::Ai(_) => t!("ops-join-gunner.ai"),
                FlightKey::Player(_) => t!("ops-join-gunner.facts"),
            };
            let mut deed = Deed::new(todo, t!("ops-join-gunner"), icon::JOIN, facts).primary();
            deed.why = match r.key {
                FlightKey::Ai(_) => take_over_refusal(c),
                FlightKey::Player(_) => crew_refusal(c),
            };
            deck.push(deed);
        }
    });
}
