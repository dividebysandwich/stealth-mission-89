//! The night sky's stars: the Yale Bright Star Catalogue's 9096 stars down to magnitude 6.5
//! (`assets/sky/stars.bin`, made by `tools/gen_stars.py`), where they stand over the theater
//! at the mission's date and local time.
//!
//! The shaders find a star by the cell of the sky it lies in: rows of declination, each cut
//! into cells about as wide as they are high, in a texture ([`STAR_GRID`], [`grid`]). A cell
//! holds its brightest star: where in the cell it lies, its magnitude and its colour. The view
//! direction comes to the catalogue's frame (J2000 equatorial) by [`frame`]: the sidereal time
//! of the date and hour at the theater's longitude, its latitude, and the precession of the
//! equinoxes since 2000.

use std::f64::consts::{FRAC_PI_2, PI, TAU};

use bevy::asset::{RenderAssetUsages, uuid_handle};
use bevy::math::{DMat3, DVec3};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub struct StarsPlugin;

impl Plugin for StarsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, add_grid);
    }
}

/// The star cells' texture (`star_grid` in palette.wgsl).
pub const STAR_GRID: Handle<Image> = uuid_handle!("5b0e6f2c-6a8e-4d51-9a43-2f1d3c7e8b90");

/// The grid: rows of declination from the south pole up, the texture's width (a row's cells
/// from right ascension 0 eastwards, the rest unused) and the cells of the row at the equator.
/// Each row has as many cells as its middle is around ([`row_cells`]; must match palette.wgsl
/// `STAR_ROWS`, `STAR_COLS`).
pub const ROWS: u32 = 512;
pub const WIDTH: u32 = 1024;
const EQUATOR_CELLS: f32 = 1020.0;

const CATALOGUE: &[u8] = include_bytes!("../assets/sky/stars.bin");

/// A catalogue star: right ascension and declination (J2000, radians), V magnitude, B-V
/// colour index.
#[derive(Debug, Clone, Copy)]
pub struct Star {
    pub ra: f64,
    pub dec: f64,
    pub mag: f32,
    pub bv: f32,
}

/// The catalogue, brightest first.
pub fn catalogue() -> impl Iterator<Item = Star> {
    CATALOGUE.chunks_exact(6).map(|b| Star {
        ra: u16::from_le_bytes([b[0], b[1]]) as f64 / 65536.0 * TAU,
        dec: i16::from_le_bytes([b[2], b[3]]) as f64 / 32767.0 * FRAC_PI_2,
        mag: b[4] as i8 as f32 / 10.0,
        bv: b[5] as i8 as f32 / 40.0,
    })
}

/// Cells in grid row `row` (in f32 as the shaders work it out).
pub fn row_cells(row: u32) -> u32 {
    let dec = ((row as f32 + 0.5) / ROWS as f32 - 0.5) * std::f32::consts::PI;
    (EQUATOR_CELLS * dec.cos() + 0.5).floor().max(1.0) as u32
}

/// A star's magnitude and colour in a cell's blue and alpha bytes (decoded in palette.wgsl
/// `star_light`); blue 0 is an empty cell.
fn mag_code(mag: f32) -> u8 {
    (1.0 + (mag + 2.0) * 25.0).round().clamp(1.0, 255.0) as u8
}

fn bv_code(bv: f32) -> u8 {
    ((bv + 0.5) * 80.0).round().clamp(0.0, 255.0) as u8
}

/// Stars closer than this (radians, 5') are one to the eye.
const RESOLVED: f64 = 5.0 / 60.0 * PI / 180.0;

/// The grid's texels (RGBA8, `WIDTH` x `ROWS`): the place of the cell's star within it across
/// (red) and up (green), its magnitude (blue) and colour (alpha). A cell keeps the brightest of
/// its stars, brighter by the light of those too close to it to tell apart. Also the number of
/// stars left out (in a cell with another one).
pub fn grid() -> &'static (Vec<u8>, usize) {
    static GRID: std::sync::OnceLock<(Vec<u8>, usize)> = std::sync::OnceLock::new();
    GRID.get_or_init(|| {
        let mut texels = vec![0u8; (WIDTH * ROWS * 4) as usize];
        // Each cell's star so far: its direction and the light of it (magnitude).
        let mut kept: Vec<Option<(DVec3, f32)>> = vec![None; (WIDTH * ROWS) as usize];
        let mut dropped = 0;
        for s in catalogue() {
            let up = ((s.dec / PI + 0.5) * ROWS as f64).clamp(0.0, ROWS as f64 - 1e-6);
            let row = up.floor() as u32;
            let n = row_cells(row);
            let across = s.ra / TAU * n as f64;
            let cell = (row * WIDTH + across.floor() as u32 % n) as usize;
            let t = &mut texels[cell * 4..][..4];
            let dir = DVec3::new(s.dec.cos() * s.ra.cos(), s.dec.cos() * s.ra.sin(), s.dec.sin());
            match &mut kept[cell] {
                Some((d, mag)) if d.angle_between(dir) < RESOLVED as f64 => {
                    *mag = -2.5 * (10f32.powf(-0.4 * *mag) + 10f32.powf(-0.4 * s.mag)).log10();
                    t[2] = mag_code(*mag);
                }
                Some(_) => dropped += 1,
                None => {
                    kept[cell] = Some((dir, s.mag));
                    let byte = |f: f64| (f.fract() * 255.0).round() as u8;
                    t.copy_from_slice(&[byte(across), byte(up), mag_code(s.mag), bv_code(s.bv)]);
                }
            }
        }
        (texels, dropped)
    })
}

fn add_grid(mut images: ResMut<Assets<Image>>) {
    let image = Image::new(
        Extent3d { width: WIDTH, height: ROWS, depth_or_array_layers: 1 },
        TextureDimension::D2,
        grid().0.clone(),
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::RENDER_WORLD,
    );
    let _ = images.insert(STAR_GRID.id(), image);
}

/// Days from 1970-01-01 to the epoch J2000.0 (2000-01-01 12:00 UT).
const J2000: f64 = 10_957.5;

/// The precession of the equinoxes from J2000.0 to `t` Julian centuries after it (IAU 1976):
/// J2000 equatorial directions to those of the date.
fn precession(t: f64) -> DMat3 {
    let arcsec = |a: f64| (a / 3600.0).to_radians();
    let zeta = arcsec(2306.2181 * t + 0.30188 * t * t + 0.017998 * t * t * t);
    let z = arcsec(2306.2181 * t + 1.09468 * t * t + 0.018203 * t * t * t);
    let theta = arcsec(2004.3109 * t - 0.42665 * t * t - 0.041833 * t * t * t);
    DMat3::from_rotation_z(z) * DMat3::from_rotation_y(-theta) * DMat3::from_rotation_z(zeta)
}

/// The rotation from Bevy's frame (x east, y up, z south) to the catalogue's (J2000
/// equatorial: x to the March equinox, z to the north celestial pole) at `days` (since
/// 1970-01-01, its midnight) plus `hours` local mean solar time, at longitude `lon` and
/// latitude `lat` (degrees). The shaders get its columns (`Globals::star_frame`).
pub fn frame(days: f64, hours: f64, lon: f64, lat: f64) -> DMat3 {
    // Days since J2000.0 in UT.
    let d = days - J2000 + (hours - lon / 15.0) / 24.0;
    let gmst = 280.460_618_37 + 360.985_647_366_29 * d;
    let lst = (gmst + lon).to_radians();
    let phi = lat.to_radians();
    // The local directions in the equatorial frame of the date.
    let zenith = DVec3::new(phi.cos() * lst.cos(), phi.cos() * lst.sin(), phi.sin());
    let north = DVec3::new(-phi.sin() * lst.cos(), -phi.sin() * lst.sin(), phi.cos());
    let east = DVec3::new(-lst.sin(), lst.cos(), 0.0);
    precession(d / 36_525.0).transpose() * DMat3::from_cols(east, zenith, -north)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn day(y: i64, m: i64, d: i64) -> f64 {
        crate::tacview::days_from_civil(y, m, d) as f64
    }

    /// A catalogue direction from right ascension and declination (radians).
    fn equatorial(ra: f64, dec: f64) -> DVec3 {
        DVec3::new(dec.cos() * ra.cos(), dec.cos() * ra.sin(), dec.sin())
    }

    /// The Bevy direction of a catalogue star.
    fn local(f: DMat3, ra_h: f64, dec_deg: f64) -> DVec3 {
        f.transpose() * equatorial(ra_h / 24.0 * TAU, dec_deg.to_radians())
    }

    #[test]
    fn catalogue_reads() {
        let stars: Vec<Star> = catalogue().collect();
        assert_eq!(stars.len(), 9096);
        // Sirius first: 6h 45m, -16.7 degrees, magnitude -1.46.
        let s = stars[0];
        assert!(
            (s.ra / TAU * 24.0 - 6.752).abs() < 0.01 && (s.dec.to_degrees() + 16.72).abs() < 0.01
        );
        assert!((s.mag + 1.5).abs() < 0.06);
        assert!(stars.windows(2).all(|w| w[0].mag <= w[1].mag));
    }

    #[test]
    fn grid_rows_are_unambiguous() {
        // Each row's cell count is well clear of a rounding boundary, so the GPU's cosine
        // can't give a row another count than this one.
        for row in 0..ROWS {
            let dec = ((row as f64 + 0.5) / ROWS as f64 - 0.5) * PI;
            let x = EQUATOR_CELLS as f64 * dec.cos() + 0.5;
            let f = x.fract();
            assert!(x < 1.0 || (0.002..0.998).contains(&f), "row {row}: {x}");
            assert!(row_cells(row) < WIDTH);
        }
    }

    #[test]
    fn grid_keeps_the_bright_stars() {
        let (texels, dropped) = grid();
        let kept = texels.chunks_exact(4).filter(|t| t[2] != 0).count();
        eprintln!("kept {kept}, dropped {dropped}");
        assert!(kept + dropped < 9096 && *dropped < 200);
        // A cell's star is its brightest: every star brighter than magnitude 3.5 shows, the
        // close doubles as one (Alpha Centauri, -0.27 together).
        let bright = |m: f32| catalogue().filter(|s| s.mag < m).count();
        let shown =
            |m: f32| texels.chunks_exact(4).filter(|t| t[2] != 0 && t[2] < mag_code(m)).count();
        eprintln!("brighter than 3.5: {} shown of {}", shown(3.5), bright(3.5));
        assert!(shown(3.5) >= bright(3.5) - 3);
        assert_eq!(shown(-0.2), 3);
    }

    #[test]
    fn polaris_and_precession() {
        // Polaris (2h 31.8m, +89 15.85') stands at the latitude's height in the north.
        let f = frame(day(1991, 3, 1), 22.0, 10.0, 50.0);
        let p = local(f, 2.0 + 31.8 / 60.0, 89.264);
        let alt = p.y.asin().to_degrees();
        assert!((alt - 50.0).abs() < 1.0, "{alt}");
        assert!(p.z < 0.0);
        // Precession takes it towards the pole: +89 22' in 2026.
        let p = precession((day(2026, 1, 1) - J2000) / 36_525.0)
            * equatorial((2.0 + 31.8 / 60.0) / 24.0 * TAU, 89.264f64.to_radians());
        let dec = p.z.asin().to_degrees();
        assert!((dec - (89.0 + 22.0 / 60.0)).abs() < 0.03, "{dec}");
    }

    #[test]
    fn sirius_at_midnight_on_new_year() {
        // At local midnight at the turn of the year the sidereal time is about 6h 40m: Sirius
        // stands near the meridian in the south, 90 - 50 - 16.7 degrees high at 50 north.
        let f = frame(day(2000, 1, 1), 0.0, 0.0, 50.0);
        let s = local(f, 6.752, -16.716);
        let alt = s.y.asin().to_degrees();
        assert!((alt - 23.3).abs() < 0.5, "{alt}");
        assert!(s.z > 0.9 && s.x.abs() < 0.1, "{s}");
        // Six hours later it has set in the south west.
        let s = local(frame(day(2000, 1, 1), 6.0, 0.0, 50.0), 6.752, -16.716);
        assert!(s.y < 0.0 && s.x < 0.0);
        // The frame is a rotation.
        assert!((f.determinant() - 1.0).abs() < 1e-9);
        assert!(
            (f.transpose() * f - DMat3::IDENTITY).to_cols_array().iter().all(|v| v.abs() < 1e-9)
        );
    }

    #[test]
    fn orion_in_the_winter_evening_sky() {
        // Betelgeuse (5h 55m, +7.4) rises in the east in the early evening in late autumn.
        let f = frame(day(1990, 11, 15), 21.0, 10.0, 50.0);
        let b = local(f, 5.919, 7.407);
        assert!(b.y > 0.0 && b.y < 0.35 && b.x > 0.5, "{b}");
        // And is long down on a summer evening.
        let b = local(frame(day(1990, 7, 15), 22.0, 10.0, 50.0), 5.919, 7.407);
        assert!(b.y < -0.3, "{b}");
    }
}
