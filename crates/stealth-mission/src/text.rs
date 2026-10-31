use serde::{Deserialize, Serialize};

use crate::generator::{GenResult, s16};
use crate::rng::MscRand;
use crate::tables::{MissionTables, Objective};

/// Briefing wording (`assets/mission/briefings.json`): every entry is a list of variants, one
/// of which is picked per mission. Templates use `{name}` placeholders. The intelligence
/// summaries are the mission templates' own (`missions.ron`).
///
/// This is the English; the other languages' are messages `bt-<entry>-<variant>` of
/// `assets/i18n/<language>/briefings.ftl` (written for English by `tools/i18n_briefings.py`),
/// the briefing being worded in the player's language when the mission is generated.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BriefingTexts {
    /// Rules of engagement per ROE level (0 clandestine, 1 limited, 2 full war).
    pub roe: Vec<Vec<String>>,
    pub headers: Vec<String>,
    /// Titles per mission kind (air combat, ground strike, air training, ground training).
    pub titles: Vec<Vec<String>>,
    pub labels: BriefingLabels,
    /// `{pilot}`.
    pub pilot: Vec<String>,
    /// `{base}`, `{time}`.
    pub depart: Vec<String>,
    /// `{base}`.
    pub land: Vec<String>,
    /// `{fuel}`.
    pub fuel: Vec<String>,
    pub objectives: ObjectiveTexts,
}

/// Section headings of the briefing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BriefingLabels {
    pub primary: String,
    pub secondary: String,
    pub objective: String,
    pub flight_plan: String,
    pub roe: String,
    pub armament: String,
}

/// Objective templates by mission type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectiveTexts {
    /// Photo / strike: `{what}` (and `{place}` for the `_at` forms), completed by a suffix.
    pub photo_at: Vec<String>,
    pub photo: Vec<String>,
    pub destroy_at: Vec<String>,
    pub destroy: Vec<String>,
    /// `{deadline}`.
    pub suffix_deadline: Vec<String>,
    /// `{deadline}`, the cargo is destroyed with the target while it is being unloaded.
    pub suffix_unloading: Vec<String>,
    pub suffix_none: Vec<String>,
    /// Supply drop: `{coord}`, `{deadline}`.
    pub drop: Vec<String>,
    /// Secret airstrip: `{coord}`, `{deadline}`.
    pub airstrip: Vec<String>,
    /// Transport on a known route: `{origin}`, `{depart}`, `{place}`, `{deadline}`.
    pub route: Vec<String>,
    /// Transport leaving a base: `{place}`, `{depart}`.
    pub leaving: Vec<String>,
    /// Transport last seen at a grid reference: `{coord}`, `{depart}`, `{place}`, `{deadline}`.
    pub last_seen: Vec<String>,
    /// Aircraft at a grid reference: `{target}`, `{coord}`.
    pub patrol: Vec<String>,
}

fn days_in_month(month: u32, year: u32) -> u32 {
    match month {
        2 if year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Fills `{key}` placeholders.
fn fill(template: &str, vars: &[(&str, &str)]) -> String {
    vars.iter().fold(template.to_string(), |s, (k, v)| s.replace(&format!("{{{k}}}"), v))
}

/// Message `id` in the player's language with `vars`; None in English or where the language
/// lacks it (the data files' English is used).
fn translated(id: &str, vars: &[(&str, &str)]) -> Option<String> {
    let l = stealth_i18n::language();
    if l == stealth_i18n::Language::En || !stealth_i18n::exists_in(l, id) {
        return None;
    }
    let mut args = stealth_i18n::Args::new();
    for (k, v) in vars {
        args.set(*k, *v);
    }
    Some(stealth_i18n::tr_args_in(l, id, Some(&args)))
}

/// A target type's name ("Bridge") in the player's language, with its article where the
/// language's objective templates need it.
fn object_name(name: &str) -> String {
    let id: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    let id = format!("obj-{}", id.trim_matches('-'));
    translated(&id, &[]).unwrap_or_else(|| name.to_string())
}

/// A briefing label in the player's language (`english` from the data files).
fn label(id: &str, english: &str) -> String {
    translated(id, &[]).unwrap_or_else(|| english.to_string())
}

/// The pre-flight briefing (paragraph layout is not reproduced).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Briefing {
    pub header: String,
    pub title: String,
    pub pilot_line: String,
    /// `"<Mon d, yyyy>"`.
    pub date: String,
    /// `"<takeoff base> - <date>"`.
    pub base_date_line: String,
    /// PRIMARY TARGET INTELLIGENCE SUMMARY paragraph (mission row text).
    pub primary_summary: String,
    pub primary_objective: String,
    pub secondary_summary: String,
    pub secondary_objective: String,
    pub depart_line: String,
    pub land_line: String,
    pub roe_text: String,
    /// One weapon name per bay (2 bays on the Lockheed).
    pub armament: Vec<String>,
    pub fuel_line: String,
    /// Not part of the classic briefing (and not in `lines`).
    #[serde(default)]
    pub weather_line: String,
    /// All non-empty lines in display order.
    pub lines: Vec<String>,
}

/// One line on the orders MFD page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrdersLine {
    pub text: String,
    /// Palette colour index of the line.
    pub color: u8,
}

/// The in-flight orders MFD page at mission start (no objective completed yet).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Orders {
    /// The lines as the orders page shows them (mixed case).
    pub lines: Vec<OrdersLine>,
    /// Primary order as one upper-case line, e.g. "STRIKE THE SS MISSILE AT LIBEREC".
    pub primary: String,
    pub secondary: String,
    /// "RETURN TO <landing base>".
    pub return_to: String,
}

/// Context for the text builders.
pub(crate) struct TextCtx<'a> {
    pub x: &'a MissionTables,
    pub texts: &'a BriefingTexts,
    /// Seed of the wording choice (the generator's entry `holdrand`).
    pub seed: u32,
    pub g: &'a GenResult,
    /// The theater's first mission date (month, day, year).
    pub date: (u8, u8, u16),
    pub kind: usize,
    pub roe: usize,
    pub lockheed: bool,
}

impl TextCtx<'_> {
    fn name(&self, i: i32) -> &str {
        self.g.world.name(i)
    }
    fn obj(&self, i: i32) -> &crate::world::WorldObject {
        let n = self.g.world.objects.len() as i32;
        &self.g.world.objects[i.rem_euclid(n) as usize]
    }
    /// Base/landmark display name: place name, or the shape name if unnamed.
    pub fn base_name(&self, i: i32) -> String {
        let o = self.obj(i);
        let k = if o.name != 0 { o.name as i32 } else { (o.object_idx & 0xFF) as i32 };
        self.name(k).to_string()
    }

    fn onc(b: &[u8; 4]) -> String {
        b.iter().map(|&c| c as char).collect()
    }

    /// One variant, the same for every call with this mission and `salt` (own random stream,
    /// so the generator's sequence is untouched).
    /// Which of `n` variants (one per mission and `salt`).
    fn pick_index(&self, n: usize, salt: u32) -> usize {
        let mut r = MscRand::new(self.seed ^ 0x4252_4946 ^ salt.wrapping_mul(0x9E37_79B9));
        r.rand();
        r.rand() as usize % n
    }

    /// The picked variant of entry `id` (message `<id>-<variant>`) in the player's language,
    /// else the English of `variants`, with its placeholders filled.
    fn text(&self, id: &str, variants: &[String], salt: u32, vars: &[(&str, &str)]) -> String {
        if variants.is_empty() {
            return String::new();
        }
        let i = self.pick_index(variants.len(), salt);
        translated(&format!("{id}-{i}"), vars).unwrap_or_else(|| fill(&variants[i], vars))
    }

    /// Intelligence paragraph of a target slot.
    pub fn summary(&self, slot: usize) -> String {
        let k = self.g.targets[slot].num as usize;
        self.x.templates.get(k).map_or(String::new(), |m| {
            let id = format!("bt-intel-{}", m.id.replace('_', "-"));
            self.text(&id, &m.briefings, 0x100 + slot as u32, &[])
        })
    }

    /// Objective text by mission type.
    pub fn objective(&self, slot: usize) -> String {
        let (g, x, o_t) = (self.g, self.x, &self.texts.objectives);
        let t = &g.targets[slot];
        let o = self.obj(t.idx);
        let place = if o.name != 0 { self.name(o.name as i32) } else { "" };
        let m = &x.templates[t.num as usize];
        let code = m.code();
        let coord = Self::onc(&t.coord);
        let salt = 0x200 + slot as u32;
        let text = |id: &str, v: &[String], vars: &[(&str, &str)]| self.text(id, v, salt, vars);
        let (deadline, depart) = (g.deadline.as_str(), g.depart.as_str());
        match m.objective {
            Objective::Photo | Objective::Destroy => {
                let what = object_name(self.name((o.object_idx & 0xFF) as i32));
                let photo = m.objective == Objective::Photo;
                let (id, forms) = match (photo, o.name != 0) {
                    (true, true) => ("bt-photo-at", &o_t.photo_at),
                    (true, false) => ("bt-photo", &o_t.photo),
                    (false, true) => ("bt-destroy-at", &o_t.destroy_at),
                    (false, false) => ("bt-destroy", &o_t.destroy),
                };
                let mut s = text(id, forms, &[("what", &what), ("place", place)]);
                let (id, suffix) = if code & 0x20 != 0 {
                    ("bt-suffix-unloading", &o_t.suffix_unloading)
                } else if code & 2 != 0 {
                    ("bt-suffix-deadline", &o_t.suffix_deadline)
                } else {
                    if s.ends_with(',') {
                        s.pop();
                    }
                    ("bt-suffix-none", &o_t.suffix_none)
                };
                s + &text(id, suffix, &[("deadline", deadline)])
            }
            Objective::SupplyDrop => {
                text("bt-drop", &o_t.drop, &[("coord", &coord), ("deadline", deadline)])
            }
            Objective::Airstrip => {
                text("bt-airstrip", &o_t.airstrip, &[("coord", &coord), ("deadline", deadline)])
            }
            Objective::InterceptRoute => text(
                "bt-route",
                &o_t.route,
                &[
                    ("origin", self.name(self.obj(g.special_start).name as i32)),
                    ("depart", depart),
                    ("place", place),
                    ("deadline", deadline),
                ],
            ),
            Objective::InterceptLeaving => {
                text("bt-leaving", &o_t.leaving, &[("place", place), ("depart", depart)])
            }
            Objective::InterceptHome => text(
                "bt-last-seen",
                &o_t.last_seen,
                &[
                    ("coord", &g.intercept_ref.as_ref().map(Self::onc).unwrap_or_default()),
                    ("depart", depart),
                    ("place", place),
                    ("deadline", deadline),
                ],
            ),
            Objective::Patrol => {
                let plane = &x.plane(g.world.flights[0].plane_type).name;
                let target = if m.fighters {
                    translated("bt-fighter-patrol", &[("plane", plane)])
                        .unwrap_or_else(|| format!("{plane} fighter patrol"))
                } else {
                    plane.to_string()
                };
                text("bt-patrol", &o_t.patrol, &[("target", &target), ("coord", &coord)])
            }
        }
    }

    /// Briefing date: the theater's start date, a day later for every mission flown.
    pub fn date(&self, sorties: u16) -> String {
        const MONTHS: [&str; 12] =
            ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        let (m, d, y) = self.date;
        let (mut mon, mut day, mut yr) =
            ((m as u32).clamp(1, 12), d as u32 + sorties as u32, y as u32);
        while day > days_in_month(mon, yr) {
            day -= days_in_month(mon, yr);
            mon += 1;
            if mon > 12 {
                mon = 1;
                yr += 1;
            }
        }
        let english = MONTHS[mon as usize - 1];
        let month = label(&format!("bt-month-{mon}"), english);
        let (day, year) = (day.to_string(), yr.to_string());
        let vars = [("month", month.as_str()), ("day", day.as_str()), ("year", year.as_str())];
        translated("bt-date", &vars).unwrap_or_else(|| format!("{english} {day}, {year}"))
    }

    pub fn briefing(&self, sorties: u16, pilot: &str) -> Briefing {
        let (g, x, tx) = (self.g, self.x, self.texts);
        let t = &g.targets;
        let date = self.date(sorties);
        let nbays = if self.lockheed { 2 } else { 4 };
        let takeoff = self.base_name(t[0].base);
        let fuel = g.fuel.to_string();
        let b = Briefing {
            header: self.text("bt-header", &tx.headers, 1, &[]),
            title: tx.titles.get(self.kind).map_or(String::new(), |v| {
                self.text(&format!("bt-title-{}", self.kind), v, 2, &[])
            }),
            pilot_line: self.text("bt-pilot", &tx.pilot, 3, &[("pilot", pilot)]),
            base_date_line: format!("{takeoff} - {date}"),
            date,
            primary_summary: self.summary(0),
            primary_objective: self.objective(0),
            secondary_summary: self.summary(1),
            secondary_objective: self.objective(1),
            depart_line: self.text(
                "bt-depart",
                &tx.depart,
                4,
                &[("base", &takeoff), ("time", &g.depart)],
            ),
            land_line: self.text("bt-land", &tx.land, 5, &[("base", &self.base_name(t[1].base))]),
            roe_text: tx
                .roe
                .get(self.roe)
                .map_or(String::new(), |v| self.text(&format!("bt-roe-{}", self.roe), v, 6, &[])),
            armament: g.weapons[..nbays]
                .iter()
                .map(|&w| x.weapon_name(w as i16).to_string())
                .collect(),
            fuel_line: self.text("bt-fuel", &tx.fuel, 7, &[("fuel", &fuel)]),
            weather_line: String::new(),
            lines: Vec::new(),
        };
        let l = &tx.labels;
        let objective = label("bt-label-objective", &l.objective);
        let mut lines: Vec<String> = vec![
            b.header.clone(),
            b.title.clone(),
            b.pilot_line.clone(),
            b.base_date_line.clone(),
            label("bt-label-primary", &l.primary),
            b.primary_summary.clone(),
            objective.clone(),
            b.primary_objective.clone(),
            label("bt-label-secondary", &l.secondary),
            b.secondary_summary.clone(),
            objective,
            b.secondary_objective.clone(),
            label("bt-label-flight-plan", &l.flight_plan),
            b.depart_line.clone(),
            b.land_line.clone(),
            label("bt-label-roe", &l.roe),
            b.roe_text.clone(),
            label("bt-label-armament", &l.armament),
        ];
        lines.extend(b.armament.iter().cloned());
        lines.push(b.fuel_line.clone());
        lines.retain(|l| !l.is_empty());
        Briefing { lines, ..b }
    }

    // --- in-flight orders page --------------------------------------------------------------

    /// Object display name: place name, or the shape name if unnamed; longer than 18
    /// characters is cut to 18 + ".".
    fn orders_obj_name(&self, i: i32) -> String {
        let o = self.obj(i);
        let place = self.name(o.name as i32);
        let mut s: String =
            if !place.is_empty() { place.into() } else { self.name(o.shape() as i32).into() };
        if s.chars().count() > 18 {
            s = s.chars().take(18).collect::<String>() + ".";
        }
        s
    }

    /// Mission clock: `t + tod` as "HH:MM:SS" from 06:00 / 18:00 (or the chosen start hour).
    pub fn mission_clock(&self, t: i32) -> String {
        fn two(buf: &mut Vec<u8>, v: i32) {
            let r = s16(v) % 60;
            if r < 10 {
                buf.push(b'0');
            }
            buf.extend(r.to_string().bytes());
        }
        let t = (t + self.g.tod) as u16;
        let mut h = self.g.base_hour + s16(t as i32 / 1800) % 60;
        if self.g.clock_wraps {
            h %= 24;
        }
        let mut buf = format!("{h:02}").into_bytes();
        buf.push(b':');
        two(&mut buf, (t / 30) as i32);
        buf.push(b':');
        two(&mut buf, (t as i32) << 1);
        buf.into_iter().map(|c| c as char).collect()
    }

    /// Orders MFD page.
    pub fn orders(&self) -> Orders {
        const VERBS: [&str; 8] = [
            "Recon photos of",
            "Strike the",
            "Supply drop near",
            "Land and load at",
            "Intercept An-72 to",
            "Intercept An-72 at",
            "Intercept plane to",
            "Shoot down over",
        ];
        let g = self.g;
        let mut lines = Vec::new();
        let mut summary = [String::new(), String::new()];
        for (k, t) in g.targets.iter().enumerate() {
            let color = if k == 0 { 0x11 } else { 0x29 };
            let verb = VERBS.get((t.ty - 1) as usize).copied().unwrap_or("");
            lines.push(OrdersLine { text: verb.into(), color });
            let o = self.obj(t.idx);
            let mut what = self.name(o.shape() as i32).to_string();
            let named = !self.name(o.name as i32).is_empty();
            let mut one = format!("{verb} {what}");
            if named {
                what += " at ";
                lines.push(OrdersLine { text: what, color });
                let place = self.orders_obj_name(t.idx);
                one = format!("{one} at {place}");
                lines.push(OrdersLine { text: place, color });
            } else {
                lines.push(OrdersLine { text: what, color });
            }
            if t.code & 2 != 0 {
                let before = format!("Before {}", self.mission_clock(g.mtime));
                one = format!("{one} {before}");
                lines.push(OrdersLine { text: before, color: 7 });
            }
            summary[k] = one.to_uppercase();
        }
        lines.push(OrdersLine { text: "Return to".into(), color: 0xA });
        let base = self.orders_obj_name(g.targets[1].base);
        let return_to = format!("RETURN TO {}", base.to_uppercase());
        lines.push(OrdersLine { text: base, color: 0xA });
        let [primary, secondary] = summary;
        Orders { lines, primary, secondary, return_to }
    }
}
