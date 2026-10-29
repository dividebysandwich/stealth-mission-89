//! APP-6 (NATO joint military symbology) unit symbols for the maps: the frame by affiliation
//! and battle dimension (a rectangle, diamond or square on the ground, their upper halves open
//! below in the air), filled with the affiliation's colour, the black icon inside, and the
//! modifiers the maps use (installation, echelon, tracked mobility, direction of movement).

use bevy_egui::egui;

/// APP-6's light fill colours: friend cyan, hostile red, neutral green.
pub const FRIEND: egui::Color32 = egui::Color32::from_rgb(128, 224, 255);
pub const HOSTILE: egui::Color32 = egui::Color32::from_rgb(255, 128, 128);
pub const NEUTRAL: egui::Color32 = egui::Color32::from_rgb(170, 255, 170);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Affiliation {
    Friend,
    Hostile,
    Neutral,
}

impl Affiliation {
    pub fn colour(self) -> egui::Color32 {
        match self {
            Affiliation::Friend => FRIEND,
            Affiliation::Hostile => HOSTILE,
            Affiliation::Neutral => NEUTRAL,
        }
    }
}

/// The battle dimension: the frame's shape. An installation is a ground frame with its bar.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dimension {
    Air,
    Ground,
    Installation,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    None,
    FixedWing,
    RotaryWing,
    Uav,
    /// Air defence, missile (a SAM site, a Patriot battery, MANPADS).
    AirDefenceMissile,
    /// Air defence, gun (AAA, a Gepard).
    AirDefenceGun,
}

/// The echelon above the frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Echelon {
    None,
    Team,
    Battery,
}

#[derive(Clone, Copy, Debug)]
pub struct Symbol {
    pub affiliation: Affiliation,
    pub dimension: Dimension,
    pub icon: Icon,
    pub echelon: Echelon,
    /// Tracked mobility (an oval under the frame).
    pub tracked: bool,
    /// Direction of movement (radians, 0 north, clockwise): a staff from the frame's centre.
    pub heading: Option<f32>,
}

impl Symbol {
    pub fn new(affiliation: Affiliation, dimension: Dimension, icon: Icon) -> Self {
        Self { affiliation, dimension, icon, echelon: Echelon::None, tracked: false, heading: None }
    }

    pub fn echelon(self, echelon: Echelon) -> Self {
        Self { echelon, ..self }
    }

    pub fn tracked(self) -> Self {
        Self { tracked: true, ..self }
    }

    pub fn heading(self, heading: f32) -> Self {
        Self { heading: Some(heading), ..self }
    }

    /// Draws the symbol centred on `c`; `k` scales it (1: a friendly ground frame 15 by 10
    /// points).
    pub fn draw(&self, painter: &egui::Painter, c: egui::Pos2, k: f32, alpha: f32) {
        let ink = egui::Color32::BLACK.gamma_multiply(alpha);
        let line = egui::Stroke::new((1.2 * k).max(0.8), ink);
        let at = |x: f32, y: f32| c + egui::vec2(x, y) * k;
        let frame = self.frame();
        let pts: Vec<egui::Pos2> = frame.outline.iter().map(|&(x, y)| at(x, y)).collect();
        if let Some(h) = self.heading {
            // Under the frame: the staff shows past its edge.
            let fwd = egui::vec2(h.sin(), -h.cos());
            let side = egui::vec2(-fwd.y, fwd.x);
            let tip = c + fwd * 14.0 * k;
            let staff = egui::Stroke::new(
                (1.5 * k).max(1.0),
                self.affiliation.colour().gamma_multiply(alpha),
            );
            painter.line_segment([c, tip], staff);
            painter.line_segment([tip, tip - fwd * 3.0 * k + side * 2.0 * k], staff);
            painter.line_segment([tip, tip - fwd * 3.0 * k - side * 2.0 * k], staff);
        }
        let fill = self.affiliation.colour().gamma_multiply(alpha);
        painter.add(egui::Shape::convex_polygon(pts.clone(), fill, egui::Stroke::NONE));
        let outline = if frame.open {
            egui::Shape::line(pts, line)
        } else {
            egui::Shape::closed_line(pts, line)
        };
        painter.add(outline);
        let (bx, by, hw, hh) = frame.icon;
        icon(painter, self.icon, at(bx, by), hw * k, hh * k, line);
        let top = at(0.0, frame.top);
        if self.dimension == Dimension::Installation {
            let w = frame.width * 0.3 * k;
            let r =
                egui::Rect::from_min_max(top + egui::vec2(-w, -2.5 * k), top + egui::vec2(w, 0.0));
            painter.rect_filled(r, 0.0, ink);
        }
        match self.echelon {
            Echelon::None => {}
            Echelon::Battery => {
                painter.line_segment(
                    [top - egui::vec2(0.0, 1.5 * k), top - egui::vec2(0.0, 5.0 * k)],
                    line,
                );
            }
            Echelon::Team => {
                let o = top - egui::vec2(0.0, 3.5 * k);
                painter.circle_stroke(o, 1.8 * k, line);
                painter.line_segment(
                    [o + egui::vec2(-2.5, 2.5) * k, o + egui::vec2(2.5, -2.5) * k],
                    line,
                );
            }
        }
        if self.tracked {
            let r = egui::Rect::from_min_max(
                at(-frame.width * 0.4, frame.bottom + 1.0),
                at(frame.width * 0.4, frame.bottom + 3.0),
            );
            painter.rect_stroke(r, r.height() / 2.0, line, egui::StrokeKind::Middle);
        }
    }

    fn frame(&self) -> Frame {
        let ground = |outline, icon, top, bottom, width| Frame {
            outline,
            open: false,
            icon,
            top,
            bottom,
            width,
        };
        let air = |outline, icon, width| Frame {
            outline,
            open: true,
            icon,
            top: -6.0,
            bottom: 4.0,
            width,
        };
        match (self.dimension, self.affiliation) {
            (Dimension::Air, Affiliation::Friend) => {
                // A dome: the upper half of an ellipse.
                let outline = (0..=16)
                    .map(|i| {
                        let t = std::f32::consts::PI * (1.0 + i as f32 / 16.0);
                        (7.0 * t.cos(), 4.0 + 10.0 * t.sin())
                    })
                    .collect();
                air(outline, (0.0, 0.5, 4.5, 3.0), 14.0)
            }
            (Dimension::Air, Affiliation::Hostile) => air(
                vec![(-6.5, 4.0), (-6.5, -0.5), (0.0, -6.5), (6.5, -0.5), (6.5, 4.0)],
                (0.0, 1.0, 4.5, 3.0),
                13.0,
            ),
            (Dimension::Air, Affiliation::Neutral) => air(
                vec![(-5.5, 4.0), (-5.5, -6.0), (5.5, -6.0), (5.5, 4.0)],
                (0.0, -0.5, 4.0, 3.5),
                11.0,
            ),
            (_, Affiliation::Friend) => ground(
                vec![(-7.5, -5.0), (7.5, -5.0), (7.5, 5.0), (-7.5, 5.0)],
                (0.0, 0.0, 6.0, 3.8),
                -5.0,
                5.0,
                15.0,
            ),
            (_, Affiliation::Hostile) => ground(
                vec![(0.0, -7.5), (7.5, 0.0), (0.0, 7.5), (-7.5, 0.0)],
                (0.0, 0.0, 3.8, 3.8),
                -7.5,
                7.5,
                15.0,
            ),
            (_, Affiliation::Neutral) => ground(
                vec![(-5.5, -5.5), (5.5, -5.5), (5.5, 5.5), (-5.5, 5.5)],
                (0.0, 0.0, 4.5, 4.5),
                -5.5,
                5.5,
                11.0,
            ),
        }
    }
}

/// A frame's outline (symbol units, y down), open below (air), the icon's box (centre and
/// half extents), its top and bottom and width.
struct Frame {
    outline: Vec<(f32, f32)>,
    open: bool,
    icon: (f32, f32, f32, f32),
    top: f32,
    bottom: f32,
    width: f32,
}

/// The icon in the box centred on `c`, `hw` by `hh` either way.
fn icon(painter: &egui::Painter, icon: Icon, c: egui::Pos2, hw: f32, hh: f32, line: egui::Stroke) {
    let at = |x: f32, y: f32| c + egui::vec2(x * hw, y * hh);
    match icon {
        Icon::None => {}
        Icon::FixedWing => {
            painter.line_segment([at(0.0, -1.0), at(0.0, 1.0)], line);
            painter.add(egui::Shape::line(vec![at(-1.0, 0.3), at(0.0, -0.3), at(1.0, 0.3)], line));
            painter.line_segment([at(-0.4, 1.0), at(0.4, 1.0)], line);
        }
        Icon::RotaryWing => {
            for s in [-1.0, 1.0] {
                let pts = vec![at(0.0, 0.0), at(s, -0.7), at(s, 0.7)];
                painter.add(egui::Shape::convex_polygon(pts, line.color, egui::Stroke::NONE));
            }
        }
        Icon::Uav => {
            let wide = egui::Stroke::new(line.width * 1.4, line.color);
            painter.add(egui::Shape::line(vec![at(-1.0, -0.5), at(0.0, 0.5), at(1.0, -0.5)], wide));
        }
        Icon::AirDefenceMissile | Icon::AirDefenceGun => {
            // The air defence dome on the box's floor, the weapon rising from it.
            let dome = (0..=12)
                .map(|i| {
                    let t = std::f32::consts::PI * (1.0 + i as f32 / 12.0);
                    at(t.cos(), 1.0 + 0.8 * t.sin())
                })
                .collect();
            painter.add(egui::Shape::line(dome, line));
            if icon == Icon::AirDefenceGun {
                painter.line_segment([at(0.0, 0.2), at(0.0, -1.0)], line);
            } else {
                let body = (0..12)
                    .map(|i| {
                        let t = i as f32 / 12.0 * std::f32::consts::TAU;
                        c + egui::vec2(0.16 * hw * t.cos(), (-0.35 + 0.6 * t.sin()) * hh)
                    })
                    .collect();
                painter.add(egui::Shape::convex_polygon(body, line.color, egui::Stroke::NONE));
            }
        }
    }
}
