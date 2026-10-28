//! Animated GIF recording (F12, `Action::RecordGif`, in any screen): the window is captured at
//! the frame rate and size chosen in OPTIONS > GAME and saved in the platform's Pictures folder.
//!
//! The frames come from the GPU readback of Bevy's screenshots; scaling, colour quantisation and
//! writing run on worker threads behind a bounded queue. A frame that finds the queue full is
//! dropped (the GIF's frame times come from the capture times, so it keeps its pace) rather than
//! held up. A REC mark shows in the top right corner while recording; it is hidden on the
//! frames that are captured, and two frames in a row are never captured, so it stays on screen
//! but never gets into the GIF.

use std::path::PathBuf;
use std::sync::mpsc::{Receiver, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
use bevy::window::PrimaryWindow;

use crate::input::{Action, ActionEvent};
use crate::settings::Settings;

/// The sizes offered (percent of the window), with their names.
pub const SCALES: [(u8, &str, &[&str]); 5] = [
    (100, "100%", &["The window's full resolution: large files"]),
    (75, "75%", &[]),
    (50, "50%", &[]),
    (33, "33%", &[]),
    (25, "25%", &["Small files"]),
];
pub const DEFAULT_SCALE: u8 = 50;

/// The frame rates offered (frames per second; GIF times are in 1/100 s, players slow anything
/// under 2/100 s down).
pub const FRAME_RATES: [(u8, &str, &[&str]); 6] = [
    (10, "10 fps", &[]),
    (15, "15 fps", &[]),
    (20, "20 fps", &[]),
    (25, "25 fps", &[]),
    (30, "30 fps", &[]),
    (50, "50 fps", &["Only when the game runs at 100 fps or more (every other frame is captured)"]),
];
pub const DEFAULT_FRAME_RATE: u8 = 20;

/// NeuQuant sampling (1 best .. 30 fastest).
const QUANT_SPEED: i32 = 10;
/// Frames waiting for the encoders, in bytes of window pixels.
const QUEUE_BYTES: usize = 256 << 20;

pub struct GifRecorderPlugin;

impl Plugin for GifRecorderPlugin {
    fn build(&self, app: &mut App) {
        // The browser build writes no files.
        if cfg!(target_arch = "wasm32") {
            return;
        }
        app.init_resource::<GifRecorder>()
            .add_systems(Update, (toggle, capture, finish_saving, show_mark).chain())
            .add_systems(Last, finish_on_exit)
            .add_observer(captured);
    }
}

/// The GIF size for a window: `percent` of it, at least a pixel and at most the format's limit.
pub fn gif_size(window: UVec2, percent: u8) -> UVec2 {
    let scale = |v: u32| (v * percent as u32).div_ceil(100).clamp(1, u16::MAX as u32);
    UVec2::new(scale(window.x), scale(window.y))
}

#[derive(Resource, Default)]
struct GifRecorder {
    session: Option<Session>,
    /// The recordings' numbers (late frames of a stopped one are dropped).
    sessions: u32,
    /// Encoders still writing their files.
    saving: Vec<JoinHandle<()>>,
    /// The REC mark while recording or saving.
    mark: Option<Entity>,
}

struct Session {
    id: u32,
    tx: SyncSender<Shot>,
    path: PathBuf,
    started: f64,
    /// Real time of the next capture, and the time between captures.
    next: f64,
    period: f64,
    /// The last frame was captured (the next one is not: it shows the REC mark).
    captured_last: bool,
    /// Frames handed to the encoders, and frames dropped with the queue full.
    sent: u32,
    dropped: u32,
}

/// A captured frame for the encoders.
struct Shot {
    index: u32,
    /// Seconds since the recording started.
    t: f64,
    size: UVec2,
    /// RGBA (BGRA if `bgra`) pixels, the alpha meaningless.
    data: Vec<u8>,
    bgra: bool,
}

/// On a screenshot entity of a recording.
#[derive(Component)]
struct GifShot {
    session: u32,
    t: f64,
}

#[derive(Component)]
struct RecMark;

/// `<Pictures>/stealth-mission-89_<date>_<time>.gif` (UTC).
fn new_path() -> PathBuf {
    #[cfg(not(target_arch = "wasm32"))]
    let dir = dirs::picture_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join("Pictures")))
        .unwrap_or_else(|| PathBuf::from("."));
    #[cfg(target_arch = "wasm32")]
    let dir = PathBuf::from(".");
    let now = web_time::SystemTime::now()
        .duration_since(web_time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64);
    let (y, mo, d) = crate::tacview::civil_from_days(now.div_euclid(86_400));
    let t = now.rem_euclid(86_400);
    let stem = format!(
        "stealth-mission-89_{y:04}-{mo:02}-{d:02}_{:02}-{:02}-{:02}",
        t / 3600,
        t / 60 % 60,
        t % 60
    );
    let mut path = dir.join(format!("{stem}.gif"));
    for n in 2.. {
        if !path.exists() {
            break;
        }
        path = dir.join(format!("{stem}_{n}.gif"));
    }
    path
}

fn toggle(
    mut actions: MessageReader<ActionEvent>,
    mut rec: ResMut<GifRecorder>,
    settings: Res<Settings>,
    time: Res<Time<Real>>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    if !actions.read().any(|a| a.0 == Action::RecordGif) {
        return;
    }
    if let Some(s) = rec.session.take() {
        stop(s);
        return;
    }
    let Ok(window) = window.single() else { return };
    let size = gif_size(window.physical_size(), settings.gif_scale);
    let path = new_path();
    if let Some(dir) = path.parent()
        && let Err(e) = std::fs::create_dir_all(dir)
    {
        warn!("GIF recording: {}: {e}", dir.display());
        return;
    }
    let frame_bytes = (window.physical_width() * window.physical_height() * 4).max(1) as usize;
    let (tx, rx) = std::sync::mpsc::sync_channel((QUEUE_BYTES / frame_bytes).clamp(4, 120));
    let fps = settings.gif_fps.max(1);
    let file = path.clone();
    let writer =
        std::thread::Builder::new().name("gif writer".into()).spawn(move || {
            match encode(rx, &file, size, fps) {
                Ok(n) => info!("GIF saved: {} ({n} frames)", file.display()),
                Err(e) => warn!("GIF recording {}: {e:#}", file.display()),
            }
        });
    let writer = match writer {
        Ok(w) => w,
        Err(e) => {
            warn!("GIF recording: {e}");
            return;
        }
    };
    info!("GIF recording to {} ({}x{}, {fps} fps)", path.display(), size.x, size.y);
    rec.sessions += 1;
    let now = time.elapsed_secs_f64();
    rec.session = Some(Session {
        id: rec.sessions,
        tx,
        path,
        started: now,
        next: now,
        period: 1.0 / fps as f64,
        captured_last: false,
        sent: 0,
        dropped: 0,
    });
    rec.saving.push(writer);
}

/// Ends a recording: the encoders write what is queued and close the file.
fn stop(s: Session) {
    if s.dropped > 0 {
        warn!(
            "GIF recording {}: {} of {} frames dropped (the encoders fell behind: a smaller size or frame rate keeps up)",
            s.path.display(),
            s.dropped,
            s.sent + s.dropped
        );
    }
}

/// Takes a screenshot when one is due (never two frames in a row) and hides the REC mark on it.
fn capture(
    mut commands: Commands,
    mut rec: ResMut<GifRecorder>,
    time: Res<Time<Real>>,
    mut mark: Query<&mut Visibility, With<RecMark>>,
) {
    let now = time.elapsed_secs_f64();
    let shoot = rec.session.as_mut().is_some_and(|s| {
        let due = !s.captured_last && now >= s.next;
        if due {
            s.next = (s.next + s.period).max(now);
            commands.spawn((
                Screenshot::primary_window(),
                GifShot { session: s.id, t: now - s.started },
            ));
        }
        s.captured_last = due;
        due
    });
    for mut v in &mut mark {
        v.set_if_neq(if shoot { Visibility::Hidden } else { Visibility::Inherited });
    }
}

/// A recording's screenshot: its pixels go to the encoders.
fn captured(
    mut shot: On<ScreenshotCaptured>,
    shots: Query<&GifShot>,
    mut rec: ResMut<GifRecorder>,
) {
    let Ok(&GifShot { session, t }) = shots.get(shot.entity) else { return };
    let Some(s) = rec.session.as_mut().filter(|s| s.id == session) else { return };
    let image = &mut shot.event_mut().image;
    let bgra = match image.texture_descriptor.format {
        TextureFormat::Bgra8UnormSrgb | TextureFormat::Bgra8Unorm => true,
        TextureFormat::Rgba8UnormSrgb | TextureFormat::Rgba8Unorm => false,
        f => {
            warn!("GIF recording: the window's format {f:?} is not supported");
            if let Some(s) = rec.session.take() {
                stop(s);
            }
            return;
        }
    };
    let size = image.size();
    let Some(data) = image.data.take() else { return };
    match s.tx.try_send(Shot { index: s.sent, t, size, data, bgra }) {
        Ok(()) => s.sent += 1,
        Err(TrySendError::Full(_)) => s.dropped += 1,
        // The writer gave up (logged).
        Err(TrySendError::Disconnected(_)) => rec.session = None,
    }
}

/// Collects the encoders that have finished.
fn finish_saving(mut rec: ResMut<GifRecorder>) {
    let (done, busy) = std::mem::take(&mut rec.saving).into_iter().partition(|w| w.is_finished());
    rec.saving = busy;
    for w in done {
        let _ = w.join();
    }
}

/// The game is closing: a recording stops, and the files are finished before it does.
fn finish_on_exit(mut exit: MessageReader<AppExit>, mut rec: ResMut<GifRecorder>) {
    if exit.read().last().is_none() {
        return;
    }
    if let Some(s) = rec.session.take() {
        stop(s);
    }
    for w in std::mem::take(&mut rec.saving) {
        let _ = w.join();
    }
}

/// The REC mark while recording, "SAVING GIF" while the encoders finish.
fn show_mark(
    mut commands: Commands,
    mut rec: ResMut<GifRecorder>,
    mut text: Query<&mut Text>,
    children: Query<&Children>,
) {
    let recording = rec.session.is_some();
    let label = if recording { "REC" } else { "SAVING GIF" };
    match rec.mark {
        Some(mark) if !recording && rec.saving.is_empty() => {
            commands.entity(mark).despawn();
            rec.mark = None;
        }
        Some(mark) => {
            for c in children.iter_descendants(mark) {
                if let Ok(mut t) = text.get_mut(c)
                    && t.0 != label
                {
                    t.0 = label.into();
                }
            }
        }
        None if recording => {
            let red = Color::srgb(0.95, 0.1, 0.1);
            let mark = commands
                .spawn((
                    RecMark,
                    Node {
                        position_type: PositionType::Absolute,
                        top: px(12),
                        right: px(12),
                        padding: UiRect::axes(px(8), px(4)),
                        column_gap: px(6),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                    GlobalZIndex(i32::MAX - 1),
                ))
                .with_children(|p| {
                    p.spawn((
                        Node {
                            width: px(12),
                            height: px(12),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(red),
                    ));
                    p.spawn((Text::new(label), TextFont::from_font_size(16.0), TextColor(red)));
                })
                .id();
            rec.mark = Some(mark);
        }
        None => {}
    }
}

/// Encodes the frames from `rx` into the GIF at `path`, `size` pixels, until the recording
/// stops (the sender is dropped). Returns the number of frames written.
fn encode(rx: Receiver<Shot>, path: &std::path::Path, size: UVec2, fps: u8) -> anyhow::Result<u32> {
    use anyhow::Context;
    let rx = Arc::new(Mutex::new(rx));
    let (out_tx, out_rx) = std::sync::mpsc::channel();
    let cores = std::thread::available_parallelism().map_or(2, |n| n.get());
    for i in 0..(cores / 4).clamp(1, 4) {
        let (rx, out) = (rx.clone(), out_tx.clone());
        std::thread::Builder::new().name(format!("gif quantise {i}")).spawn(move || {
            loop {
                let Ok(shot) = rx.lock().unwrap().recv() else { break };
                let (index, t) = (shot.index, shot.t);
                if out.send((index, t, quantise(shot, size))).is_err() {
                    break;
                }
            }
        })?;
    }
    drop(out_tx);
    let file = std::fs::File::create(path).context("creating the file")?;
    let mut gif =
        gif::Encoder::new(std::io::BufWriter::new(file), size.x as u16, size.y as u16, &[])?;
    gif.set_repeat(gif::Repeat::Infinite)?;
    // The workers finish out of order; the frames are written in order, each held until the
    // next one's time gives its delay.
    let mut waiting = std::collections::BTreeMap::new();
    let (mut next, mut written) = (0, 0);
    let mut held: Option<(f64, gif::Frame)> = None;
    // Centiseconds from the start, rounded: the delays add up to the capture times.
    let cs = |t: f64| (t * 100.0).round() as i64;
    for (index, t, frame) in out_rx {
        waiting.insert(index, (t, frame));
        while let Some((t, frame)) = waiting.remove(&next) {
            next += 1;
            if let Some((t0, mut f)) = held.take() {
                f.delay = (cs(t) - cs(t0)).clamp(2, u16::MAX as i64) as u16;
                gif.write_frame(&f)?;
                written += 1;
            }
            held = Some((t, frame));
        }
    }
    if let Some((_, mut f)) = held {
        f.delay = (100 / fps as u16).max(2);
        gif.write_frame(&f)?;
        written += 1;
    }
    gif.into_inner()?;
    Ok(written)
}

/// A captured frame scaled to `size` and reduced to 256 colours.
fn quantise(shot: Shot, size: UVec2) -> gif::Frame<'static> {
    use image::imageops;
    let mut data = shot.data;
    for p in data.as_chunks_mut::<4>().0 {
        if shot.bgra {
            p.swap(0, 2);
        }
        p[3] = 255;
    }
    let mut pixels = match image::RgbaImage::from_raw(shot.size.x, shot.size.y, data) {
        Some(img) if shot.size == size => img.into_raw(),
        // Down: averaged; up (the window grew while recording): interpolated.
        Some(img) if shot.size.cmpge(size).all() => {
            imageops::thumbnail(&img, size.x, size.y).into_raw()
        }
        Some(img) => {
            imageops::resize(&img, size.x, size.y, imageops::FilterType::Triangle).into_raw()
        }
        None => vec![0; (size.x * size.y * 4) as usize],
    };
    gif::Frame::from_rgba_speed(size.x as u16, size.y as u16, &mut pixels, QUANT_SPEED)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes() {
        assert_eq!(gif_size(UVec2::new(1920, 1080), 50), UVec2::new(960, 540));
        assert_eq!(gif_size(UVec2::new(1920, 1080), 33), UVec2::new(634, 357));
        assert_eq!(gif_size(UVec2::new(1, 1), 25), UVec2::ONE);
        assert!(SCALES.iter().any(|s| s.0 == DEFAULT_SCALE));
        assert!(FRAME_RATES.iter().any(|f| f.0 == DEFAULT_FRAME_RATE));
    }

    /// Frames handed over out of order and at uneven times come out in order, timed by their
    /// capture times, in the scaled size.
    #[test]
    fn encodes_in_order() {
        let dir = std::env::temp_dir().join(format!("sm89-gif-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("t.gif");
        let (tx, rx) = std::sync::mpsc::sync_channel(8);
        let size = UVec2::new(8, 4);
        for (index, t) in [(0, 0.0), (1, 0.05), (2, 0.2)] {
            let mut data = vec![0u8; 16 * 8 * 4];
            data.iter_mut().step_by(4).for_each(|r| *r = 80 * index as u8);
            tx.send(Shot { index, t, size: UVec2::new(16, 8), data, bgra: false }).unwrap();
        }
        drop(tx);
        assert_eq!(encode(rx, &path, size, 20).unwrap(), 3);
        let mut opts = gif::DecodeOptions::new();
        opts.set_color_output(gif::ColorOutput::RGBA);
        let mut d = opts.read_info(std::fs::File::open(&path).unwrap()).unwrap();
        assert_eq!((d.width(), d.height()), (8, 4));
        let mut frames = vec![];
        while let Some(f) = d.read_next_frame().unwrap() {
            frames.push((f.delay, f.buffer[0]));
        }
        assert_eq!(frames.len(), 3);
        assert_eq!(frames.iter().map(|f| f.0).collect::<Vec<_>>(), [5, 15, 5]);
        assert!(frames[0].1 < frames[1].1 && frames[1].1 < frames[2].1, "{frames:?}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
