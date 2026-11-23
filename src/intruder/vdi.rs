//! The pilot's vertical display indicator (VDI): the contact analog picture. A sky with clouds
//! over a ground of dark ellipses that streams past with the speed, the flight path a bright
//! road running to the commanded course's point on the horizon, the impact point's ball, the
//! steering symbol's hollow square, the target's square, and the release and pull-up markers.
//! In terrain clearance the ground is the track radar's: the ridges ahead as silhouettes,
//! the nearer the brighter. With LAND in the road is the glide path to the deck or runway.

use bevy::prelude::*;

use super::computer::Computer;
use super::terrain::{self, TerrainScan};
use super::{IntruderPanel, VdiDisplay, VdiRange, disc, ink};
use crate::canvas::Canvas;
use crate::font::{Align, Fonts, stroke_text_at};

/// The canvas (drawing units, 4:3) and its texels per unit.
pub const CANVAS: (usize, usize) = (320, 240);
pub const SCALE: usize = 2;
/// Drawing units per radian: the picture is 64 degrees wide and 48 high.
const FOCAL: f32 = 5.0 * 180.0 / std::f32::consts::PI;

/// The picture's shades.
const SKY: f32 = 0.42;
const CLOUD: f32 = 0.60;
const GROUND: f32 = 0.20;
const STRIPE: f32 = 0.27;
const BLOB: f32 = 0.05;
const LINE: f32 = 1.0;

/// The ground's pattern lies this far under the eye in its own units, whatever the height:
/// a unit is the height over the ground, no less than `LOWEST` metres.
const LOWEST: f32 = 60.0;
/// The pattern: rows `ROW` apart with a blob every `PITCH` across, every other row half a
/// pitch aside, the blobs' radius, and the rows shown.
const ROW: f32 = 1.5;
const PITCH: f32 = 3.2;
const BLOB_RADIUS: f32 = 0.5;
const ROWS: i32 = 8;
const CLOUD_ROWS: i32 = 4;
/// The landing road in metres: its depth under the eye on the glide path and its half width.
const GLIDE_DEPTH: f32 = 45.0;
const GLIDE_HALF: f32 = 32.0;

/// The tube's colours: a red picture (the display is read through a red filter), from the
/// dark glass up to a bright orange red.
pub fn colours() -> std::sync::Arc<[u32; 256]> {
    let (dark, bright, top) = ([22.0, 1.0, 1.0], [255.0, 42.0, 30.0], [255.0, 120.0, 90.0]);
    let mut lut = [u32::from_le_bytes([22, 1, 1, 255]); 256];
    for (i, v) in lut.iter_mut().enumerate().skip(1).take(100) {
        let t = i as f32 / 100.0;
        let (a, b, f): ([f32; 3], [f32; 3], f32) =
            if t < 0.85 { (dark, bright, t / 0.85) } else { (bright, top, (t - 0.85) / 0.15) };
        let [r, g, b] = std::array::from_fn(|c| (a[c] + (b[c] - a[c]) * f).round() as u8);
        *v = u32::from_le_bytes([r, g, b, 255]);
    }
    std::sync::Arc::new(lut)
}

/// The raster's lines: every other row of texels dimmer.
fn raster(c: &mut Canvas) {
    let pw = c.pw();
    for row in c.px.chunks_mut(pw).skip(1).step_by(2) {
        for p in row.iter_mut().filter(|p| **p > 1) {
            *p = dim(*p);
        }
    }
}

/// A shade on the raster's dimmer rows.
fn dim(p: u8) -> u8 {
    ((p as f32 * 0.72) as u8).max(1)
}

/// How far the ground's pattern has streamed (pattern units east, north), kept between frames.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Stream(pub Vec2);

impl Stream {
    /// Moves on with the velocity over the ground (m/s east, north) at `height` metres.
    pub fn advance(&mut self, velocity: Vec2, height: f32, dt: f32) {
        self.0 += velocity * dt / height.max(LOWEST);
        // Both periods of the pattern, so it never grows large.
        let period = Vec2::new(PITCH * 2.0, ROW * 2.0) * 8.0;
        self.0 = Vec2::new(self.0.x.rem_euclid(period.x), self.0.y.rem_euclid(period.y));
    }
}

/// What the display shows of the flight.
#[derive(Debug, Clone, Copy, Default)]
pub struct Flight {
    /// Heading, pitch and roll (rad; roll right positive).
    pub heading: f32,
    pub pitch: f32,
    pub roll: f32,
    /// Where the aircraft goes: right of the nose and above the level (rad).
    pub drift: f32,
    pub climb: f32,
    /// Height over the ground and above the sea (m).
    pub height: f32,
    pub alt: f32,
    pub stream: Vec2,
    /// Blinking symbols show.
    pub blink: bool,
}

/// The view: directions in the level frame of the nose's heading (x right, y up, -z ahead) to
/// the picture.
struct View {
    turn: Quat,
    mid: Vec2,
}

impl View {
    fn new(f: &Flight, c: &Canvas) -> Self {
        let attitude = Quat::from_euler(EulerRot::YXZ, 0.0, f.pitch, -f.roll);
        Self { turn: attitude.inverse(), mid: Vec2::new(c.w as f32 / 2.0, c.h as f32 / 2.0) }
    }

    /// In the display's own frame: x right, y up, z the depth ahead.
    fn eye(&self, d: Vec3) -> Vec3 {
        let v = self.turn * d;
        Vec3::new(v.x, v.y, -v.z)
    }

    fn flat(&self, v: Vec3) -> Vec2 {
        self.mid + Vec2::new(v.x, -v.y) * FOCAL / v.z
    }

    fn project(&self, d: Vec3) -> Option<Vec2> {
        let v = self.eye(d);
        (v.z > 0.02 * v.length()).then(|| self.flat(v))
    }

    /// A polygon of points in the level frame, cut where it passes behind the eye.
    fn polygon(&self, pts: &[Vec3]) -> Vec<Vec2> {
        const NEAR: f32 = 0.05;
        let eye: Vec<Vec3> = pts.iter().map(|p| self.eye(*p)).collect();
        let mut out = Vec::with_capacity(eye.len() + 2);
        for (i, a) in eye.iter().enumerate() {
            let b = eye[(i + 1) % eye.len()];
            if a.z >= NEAR {
                out.push(self.flat(*a));
            }
            if (a.z >= NEAR) != (b.z >= NEAR) {
                let t = (NEAR - a.z) / (b.z - a.z);
                out.push(self.flat(a.lerp(b, t)));
            }
        }
        out
    }
}

/// The direction `az` right of the nose's heading and `el` above the level.
fn dir(az: f32, el: f32) -> Vec3 {
    let (s, c) = az.sin_cos();
    let (se, ce) = el.sin_cos();
    Vec3::new(s * ce, se, -c * ce)
}

/// A convex polygon filled.
fn fill(c: &mut Canvas, pts: &[Vec2], colour: u8) {
    for i in 1..pts.len().saturating_sub(1) {
        c.triangle([pts[0], pts[i], pts[i + 1]], colour);
    }
}

fn outline(c: &mut Canvas, pts: &[Vec2], w: f32, colour: u8) {
    for (i, a) in pts.iter().enumerate() {
        c.stroke(*a, pts[(i + 1) % pts.len()], w, colour);
    }
}

fn square(mid: Vec2, half: f32) -> [Vec2; 4] {
    [
        mid + Vec2::new(-half, -half),
        mid + Vec2::new(half, -half),
        mid + Vec2::new(half, half),
        mid + Vec2::new(-half, half),
    ]
}

/// The sky over the ground: the horizon's point ahead and the picture's directions along it
/// and down from it. None looking so steeply that it is all sky (false) or ground (true).
fn horizon(v: &View, f: &Flight) -> Result<(Vec2, Vec2, Vec2), bool> {
    let (Some(mid), Some(right)) = (v.project(dir(0.0, 0.0)), v.project(dir(0.2, 0.0))) else {
        return Err(f.pitch < 0.0);
    };
    let along = (right - mid).normalize_or(Vec2::X);
    Ok((mid, along, Vec2::new(-along.y, along.x)))
}

/// The pattern's blobs of a plane `depth` under the eye (above it if negative), `rows` rows
/// out, streamed by `stream` and turned to the heading.
fn blobs(c: &mut Canvas, v: &View, f: &Flight, depth: f32, rows: i32, colour: u8) {
    let (sh, ch) = f.heading.sin_cos();
    // The eye's place in the pattern, and the pattern's axes in the nose's frame.
    let own = f.stream * if depth < 0.0 { 0.25 } else { 1.0 };
    let row0 = (own.y / ROW).floor() as i32;
    let col0 = (own.x / PITCH).floor() as i32;
    let reach = rows as f32 * ROW + 2.5;
    let span = (reach / ROW).ceil() as i32 + 1;
    for row in row0 - span..=row0 + span {
        let aside = if row.rem_euclid(2) == 1 { PITCH / 2.0 } else { 0.0 };
        for col in col0 - span..=col0 + span {
            // The blob's place east and north of the eye, then right and ahead of the nose.
            let e = col as f32 * PITCH + aside - own.x;
            let n = row as f32 * ROW - own.y;
            let (right, ahead) = (e * ch - n * sh, e * sh + n * ch);
            if ahead < 1.2 || ahead > reach || right.abs() > ahead * 1.2 + 2.0 {
                continue;
            }
            let ring: Vec<Vec3> = (0..16)
                .map(|k| {
                    let a = k as f32 * std::f32::consts::TAU / 16.0;
                    Vec3::new(
                        right + a.cos() * BLOB_RADIUS,
                        -depth,
                        -(ahead + a.sin() * BLOB_RADIUS * 0.8),
                    )
                })
                .collect();
            let pts = v.polygon(&ring);
            if pts.len() >= 3 {
                fill(c, &pts, colour);
            }
        }
    }
}

/// A road on a plane: from `near` to `far` ahead along the course `az` right of the nose, its
/// centre line `aside` right of the eye, `depth` under it at the eye and sinking `slope` (rad)
/// along the way.
#[allow(clippy::too_many_arguments)]
fn road(
    v: &View,
    az: f32,
    aside: f32,
    depth: f32,
    slope: f32,
    half: f32,
    near: f32,
    far: f32,
) -> (Vec<Vec2>, Option<(Vec2, Vec2)>) {
    let (s, c) = az.sin_cos();
    let at = |along: f32, across: f32| {
        let (x, z) = (along * s + (across + aside) * c, along * c - (across + aside) * s);
        Vec3::new(x, -depth - along * slope.tan(), -z)
    };
    let pts = v.polygon(&[at(near, -half), at(far, -half), at(far, half), at(near, half)]);
    let centre = v.polygon(&[at(near, 0.0), at(far, 0.0)]);
    (pts, (centre.len() >= 2).then(|| (centre[0], centre[1])))
}

/// The terrain clearance picture's ground: the ridges of four range bands, far to near.
fn ridges(c: &mut Canvas, v: &View, f: &Flight, scan: &TerrainScan, miles: f32, down: Vec2) {
    let reach = miles * 1852.0;
    let tops: [[Option<Vec2>; terrain::BEARINGS]; 4] = std::array::from_fn(|band| {
        let (near, far) = (reach * band as f32 / 4.0, reach * (band + 1) as f32 / 4.0);
        std::array::from_fn(|b| {
            scan.elevation(b, near, far, f.alt)
                .and_then(|el| v.project(dir(terrain::bearing_at(b), el)))
        })
    });
    for band in (0..4).rev() {
        let shade = ink(0.85 - band as f32 * 0.2);
        for b in 1..terrain::BEARINGS {
            if let (Some(a), Some(p)) = (tops[band][b - 1], tops[band][b]) {
                // Its silhouette reaches down to the nearer band's ridge (which covers the rest,
                // drawn after it) where that has this segment, else off the picture.
                let nearer = band.checked_sub(1).and_then(|n| tops[n][b - 1].zip(tops[n][b]));
                let (fa, fp) = nearer.map_or((400.0, 400.0), |(na, np)| {
                    let to =
                        |near: Vec2, top: Vec2| (near - top).dot(down).max(0.0) + RIDGE_OVERLAP;
                    (to(na, a), to(np, p))
                });
                fill(c, &[a, p, p + down * fp, a + down * fa], shade);
                c.stroke(a, p, 1.0, ink(LINE));
            }
        }
    }
}

/// A far band's silhouette overlaps the nearer band's ridge by this much (drawing units).
const RIDGE_OVERLAP: f32 = 3.0;

/// The contact analog's sky and ground.
fn world(c: &mut Canvas, v: &View, f: &Flight, analog: bool) -> Option<(Vec2, Vec2, Vec2)> {
    let h = match horizon(v, f) {
        Ok(h) => h,
        Err(ground) => {
            c.clear(ink(if ground && analog { GROUND } else { SKY }));
            return None;
        }
    };
    let (mid, along, down) = h;
    c.clear(ink(SKY));
    if analog {
        blobs(c, v, f, -1.6, CLOUD_ROWS, ink(CLOUD));
    }
    let (a, b) = (mid - along * 600.0, mid + along * 600.0);
    fill(c, &[a, b, b + down * 900.0, a + down * 900.0], ink(if analog { GROUND } else { 0.1 }));
    if analog {
        // The raster's lines on the ground, streaming towards the eye, and the blobs.
        let (sh, ch) = f.heading.sin_cos();
        let ahead = f.stream.x * sh + f.stream.y * ch;
        let phase = (ahead / (ROW / 2.0)).rem_euclid(1.0);
        for i in 0..ROWS * 2 + 4 {
            let d = (i as f32 + 1.0 - phase) * ROW / 2.0 + 1.0;
            if let Some(p) = v.project(dir(0.0, -(1.0 / d).atan())) {
                let off = (p - mid).dot(down);
                c.stroke(a + down * off, b + down * off, 0.8, ink(STRIPE));
            }
        }
        blobs(c, v, f, 1.0, ROWS, ink(BLOB));
    }
    Some(h)
}

/// The fixed marks: the heading index at the top and the wings' level marks at the sides, with
/// the pitch trim knob's offset.
fn marks(c: &mut Canvas, panel: &IntruderPanel) {
    let (w, h) = (c.w as f32, c.h as f32);
    let y = h / 2.0 - panel.pitch_trim as f32 * 5.0;
    for x in [18.0, w - 18.0] {
        c.stroke(Vec2::new(x - 9.0, y), Vec2::new(x + 9.0, y), 2.2, ink(LINE));
        c.stroke(Vec2::new(x - 9.0, y), Vec2::new(x + 9.0, y), 0.8, ink(0.0));
    }
    c.stroke(Vec2::new(w / 2.0, 10.0), Vec2::new(w / 2.0, 24.0), 2.0, ink(LINE));
}

/// The picture.
pub fn draw(
    c: &mut Canvas,
    _fonts: &Fonts,
    panel: &IntruderPanel,
    f: &Flight,
    computer: &Computer,
    scan: &TerrainScan,
) {
    c.reset_clip();
    c.clear(0);
    let (w, h) = (c.w as f32, c.h as f32);
    let centre = Vec2::new(w / 2.0, h / 2.0);
    match panel.vdi {
        VdiDisplay::Off => return,
        VdiDisplay::Standby => {
            c.clear(ink(0.04));
            return;
        }
        _ => {}
    }
    if panel.range == VdiRange::Test {
        // The test picture: a grid of the shades.
        for i in 0..8 {
            let x = (i * 40) as i32;
            c.fill_rect(x, 0, 40, c.h as i32, ink(i as f32 / 7.0));
        }
        for i in 1..6 {
            let y = i as f32 * 40.0;
            c.stroke(Vec2::new(0.0, y), Vec2::new(w, y), 1.0, ink(0.5));
        }
        c.ring(centre, 60.0, 1.5, ink(LINE));
        return;
    }
    let v = View::new(f, c);
    let analog = panel.vdi == VdiDisplay::ContactAnalog;
    let hor = world(c, &v, f, analog);
    if let (false, Some((_, _, down)), true) = (analog, hor, scan.on) {
        ridges(c, &v, f, scan, panel.range.miles().unwrap_or(2) as f32, down);
    }
    if let Some((mid, along, down)) = hor {
        c.stroke(mid - along * 600.0, mid + along * 600.0, 1.6, ink(LINE));
        // Pitch lines every ten degrees, short, the climb's above the horizon.
        for deg in [-60, -30, -20, -10, 10, 20, 30, 60] {
            if let Some(p) = v.project(dir(0.0, (deg as f32).to_radians())) {
                let half = if deg > 0 { 26.0 } else { 16.0 };
                c.stroke(p - along * half, p + along * half, 1.0, ink(0.7));
            }
        }
        let _ = down;
    }
    // The road: the glide path with LAND in, else the commanded course's.
    let bright = ink(if panel.path_bright { 0.88 } else { 0.62 });
    let (pts, line) = match computer.landing {
        Some(l) => {
            let unit = GLIDE_DEPTH;
            // The touchdown's depth under the eye, and the road's here: under the glide path,
            // so it is narrow from above the path and wide from below it.
            let touchdown = l.range * (l.glide + l.slope).tan();
            let above = touchdown - l.range * l.slope.tan();
            let depth = (GLIDE_DEPTH + above).clamp(0.25 * unit, 8.0 * unit);
            let aside = -(l.range * l.lineup.tan()) / unit;
            let az = computer_wrap(l.course - f.heading);
            let far = (l.range / unit).max(1.0);
            // Its far end sinks (or rises) to the touchdown.
            let slope = ((touchdown - depth) / l.range.max(1.0)).atan();
            road(&v, az, aside, depth / unit, slope, GLIDE_HALF / unit, 0.4, far)
        }
        None => {
            // Cruising the road stands on the picture's foot and its far end goes to the
            // commanded course's point on the horizon: all the way aside for a turn of 26
            // degrees or more.
            let az = if computer.target.is_some() { computer.bearing } else { f.drift };
            let az = az.clamp(-super::computer::BANK_LIMIT, super::computer::BANK_LIMIT);
            match (v.project(dir(az, 0.0)), v.project(dir(0.0, -0.6)), hor) {
                (Some(apex), Some(foot), Some((_, along, _))) => {
                    let half = along * 62.0;
                    (vec![foot - half, apex, foot + half], Some((foot, apex)))
                }
                _ => (Vec::new(), None),
            }
        }
    };
    if pts.len() >= 3 {
        fill(c, &pts, bright);
        if let Some((a, b)) = line {
            c.stroke(a, b, 1.2, ink(BLOB));
        }
    }
    // The target's square: dark in a bright frame.
    let target = match computer.landing {
        Some(l) => {
            let az = computer_wrap(l.course - f.heading) - l.lineup;
            v.project(dir(az, -(l.glide + l.slope)))
        }
        None => computer.target.filter(|_| computer.designated).and_then(|_| {
            v.project(dir(computer.bearing, -(f.height / computer.range.max(1.0)).atan()))
        }),
    };
    if let Some(p) = target.filter(|p| c.shows(*p, 0.0)) {
        fill(c, &square(p, 6.0), ink(LINE));
        fill(c, &square(p, 3.5), ink(BLOB));
    }
    // The impact point: a ball on a stem, where the aircraft goes.
    let impact = v.project(dir(f.drift, f.climb)).unwrap_or(centre);
    let impact = impact.clamp(Vec2::splat(8.0), Vec2::new(w - 8.0, h - 8.0));
    // The steering symbol: the hollow square to fly the ball into, aside by the bank still to
    // take up and up or down by the glide path's error.
    let bank_error = computer.bank - f.roll;
    let rise = computer.landing.map_or(0.0, |l| (l.glide * 6.0).clamp(-0.2, 0.2));
    if computer.target.is_some() || computer.landing.is_some() {
        let p = impact + Vec2::new(bank_error * 70.0, rise * FOCAL);
        let p = p.clamp(Vec2::splat(10.0), Vec2::new(w - 10.0, h - 10.0));
        outline(c, &square(p, 7.5), 2.0, ink(LINE));
    }
    c.stroke(impact, impact - Vec2::Y * 12.0, 1.6, ink(BLOB));
    disc(c, impact, 5.0, ink(LINE));
    disc(c, impact, 3.8, ink(BLOB));
    // The release marker comes down the picture to the ball as the release nears; the pull-up
    // marker blinks over it.
    if let Some(secs) = computer.release.filter(|s| (0.0..30.0).contains(s)) {
        let y = (impact.y - secs * 5.0).max(14.0);
        for side in [-1.0, 1.0] {
            let x = impact.x + side * 16.0;
            c.stroke(Vec2::new(x, y), Vec2::new(x + side * 16.0, y), 2.4, ink(LINE));
        }
    }
    if computer.pull_up && f.blink {
        let top = impact - Vec2::Y * 44.0;
        c.stroke(top, top + Vec2::new(-16.0, 16.0), 2.6, ink(LINE));
        c.stroke(top, top + Vec2::new(16.0, 16.0), 2.6, ink(LINE));
    }
    if !analog {
        // Terrain clearance: the impact bar, offset under the ball by the clearance set.
        let y = impact.y + 14.0;
        c.stroke(Vec2::new(impact.x - 30.0, y), Vec2::new(impact.x + 30.0, y), 2.0, ink(LINE));
        if let Some(mi) = panel.range.miles() {
            stroke_text_at(c, w - 40.0, 30.0, &format!("{mi}"), 9.0, 1.3, ink(LINE), Align::Centre);
        }
    }
    marks(c, panel);
    raster(c);
}

fn computer_wrap(a: f32) -> f32 {
    super::computer::wrap(a)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn count(c: &Canvas, colour: u8) -> usize {
        c.px.iter().filter(|p| **p == colour).count()
    }

    fn shot(f: &Flight, computer: &Computer, panel: &IntruderPanel) -> Canvas {
        let mut c = Canvas::scaled(CANVAS.0, CANVAS.1, SCALE);
        let fonts = Fonts::builtin();
        draw(&mut c, &fonts, panel, f, computer, &TerrainScan::default());
        c
    }

    /// Level, half the picture is ground; nose up, more of it is sky; in a right bank the
    /// ground stands on the right.
    #[test]
    fn horizon_follows_the_attitude() {
        let panel = IntruderPanel::default();
        let computer = Computer::default();
        let level = Flight { height: 300.0, ..default() };
        let shades = [ink(SKY), ink(CLOUD), dim(ink(SKY)), dim(ink(CLOUD))];
        let sky = |c: &Canvas| c.px.iter().filter(|p| shades.contains(p)).count();
        let ground = |c: &Canvas| c.px.len() - sky(c);
        let a = shot(&level, &computer, &panel);
        let up = shot(&Flight { pitch: 0.2, ..level }, &computer, &panel);
        assert!(ground(&up) < ground(&a) * 2 / 3);
        let bank = shot(&Flight { roll: 0.6, ..level }, &computer, &panel);
        let sky = ink(SKY);
        let (pw, ph) = (bank.pw(), bank.ph());
        // Even rows: the raster's brighter ones.
        let at = |x: usize, y: usize| bank.px[(y & !1) * pw + x];
        assert_eq!(at(20, ph / 2 + 60), sky, "sky low on the left in a right bank");
        assert_ne!(at(pw - 20, ph / 2 - 60), sky, "ground high on the right");
    }

    /// The road's far end goes to the side the target is on.
    #[test]
    fn road_points_at_the_target() {
        let panel = IntruderPanel::default();
        let f = Flight { height: 300.0, ..default() };
        let side = |bearing: f32| {
            let computer = Computer {
                target: Some(bevy::math::DVec2::ZERO),
                bearing,
                range: 9000.0,
                ..default()
            };
            let c = shot(&f, &computer, &panel);
            let road = ink(0.88);
            let (pw, row) = (c.pw(), (c.ph() / 2 + 20) & !1);
            let xs: Vec<usize> = (0..pw).filter(|x| c.px[row * pw + x] == road).collect();
            xs.iter().sum::<usize>() as f32 / xs.len().max(1) as f32 - pw as f32 / 2.0
        };
        assert!(side(0.3) > 40.0);
        assert!(side(-0.3) < -40.0);
        assert!(side(0.0).abs() < 6.0);
    }

    /// The ground streams: the pattern is another after moving on.
    #[test]
    fn ground_streams() {
        let panel = IntruderPanel::default();
        let computer = Computer::default();
        let mut s = Stream::default();
        let a = shot(&Flight { height: 300.0, stream: s.0, ..default() }, &computer, &panel);
        s.advance(Vec2::new(0.0, 200.0), 300.0, 0.5);
        let b = shot(&Flight { height: 300.0, stream: s.0, ..default() }, &computer, &panel);
        assert!(count(&a, ink(BLOB)) > 2000);
        assert!(a.px != b.px);
    }
}
