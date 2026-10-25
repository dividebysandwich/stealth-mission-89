//! Cockpit art and display drawing shared by the 3D cockpit.
//!
//! The classic view projects with `sx = cx + f*x/z`, `sy = cy - 0.75*f*y/z`, f = 256, so each pixel
//! of the 320x200 screen is a fixed direction from the eye. The 3D cockpit keeps that mapping
//! for its panel picture and HUD (`cockpit3d::geometry`).
//!
//! The instrument panel itself is built from meshes (`cockpit3d`); HUD symbols, LED digits, TAC
//! map icons and the weapon side views come from a sprite atlas, `images/sprites.png` (drawn by
//! `tools/gen_images.py`; the rectangles are in `sprites`).

use bevy::prelude::*;

use crate::canvas::Canvas;
use crate::font::Fonts;
use crate::hud::HudParams;
use crate::mfd::MfdParams;
use crate::panel::PanelParams;

pub struct CockpitPlugin;

impl Plugin for CockpitPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

/// Classic focal length (pixels) and vertical squash.
pub const FOCAL: f32 = 256.0;
pub const VSCALE: f32 = 0.75;

/// Where each sprite sits in the atlas (`images/sprites.png`). Mirrored in
/// `tools/gen_images.py`, which draws the atlas: keep the two in step.
pub mod sprites {
    /// A rectangle in the atlas.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Sprite {
        pub x: i32,
        pub y: i32,
        pub w: i32,
        pub h: i32,
    }

    const fn at(x: i32, y: i32, w: i32, h: i32) -> Sprite {
        Sprite { x, y, w, h }
    }

    /// Weapon side views, 77x15 with their frame: picture `p` is `ORDNANCE` moved by
    /// `ORDNANCE_STEP` (p % 4, p / 4).
    pub const ORDNANCE: Sprite = at(0, 0, 77, 15);
    pub const ORDNANCE_STEP: (i32, i32) = (80, 16);
    pub const ORDNANCE_COUNT: usize = 19;

    /// Flight path marker, drawn in the HUD colour, and its centre pixel.
    pub const FPM: Sprite = at(290, 65, 13, 9);
    pub const FPM_CENTRE: (i32, i32) = (6, 5);
    /// TAC/map icons, 7x7 around their middle pixel.
    pub const AIRBASE: Sprite = at(304, 66, 7, 7);
    pub const TARGET: Sprite = at(312, 66, 7, 7);

    /// Heading tape: one 10-pixel period wider than the 141 pixels shown; the scroll phase
    /// (0..10) is the x offset into it. Major ticks where (x + phase) % 10 == 9.
    pub const HEADING_TAPE: Sprite = at(0, 80, 151, 3);
    pub const HEADING_TAPE_SHOWN: i32 = 141;

    /// LED digits 0..9, 4x7 each, `LED_STEP` apart; lit segments 47 (the unlit "8" is drawn
    /// recoloured).
    pub const LED_DIGIT: Sprite = at(100, 83, 4, 7);

    /// Gun sight reticle, drawn recoloured, and its centre pixel.
    pub const GUN_SIGHT: Sprite = at(162, 80, 13, 11);
    pub const GUN_SIGHT_CENTRE: (i32, i32) = (6, 5);
    /// Boresight waterline (W) and the pixel on the nose line (the W's middle peak).
    pub const WATERLINE: Sprite = at(178, 80, 25, 4);
    pub const WATERLINE_CENTRE: (i32, i32) = (12, 0);
    pub const LED_STEP: i32 = 5;

    /// Weapon picture `p`, if the atlas has one.
    pub fn ordnance(p: usize) -> Option<Sprite> {
        (p < ORDNANCE_COUNT).then(|| Sprite {
            x: ORDNANCE.x + (p % 4) as i32 * ORDNANCE_STEP.0,
            y: ORDNANCE.y + (p / 4) as i32 * ORDNANCE_STEP.1,
            ..ORDNANCE
        })
    }

    /// LED digit `d` (0..=9).
    pub fn led_digit(d: i32) -> Sprite {
        Sprite { x: LED_DIGIT.x + LED_STEP * d, ..LED_DIGIT }
    }
}

/// The sprite atlas, the source of every sprite the displays draw.
#[derive(Resource)]
pub struct CockpitArt {
    pub sprites: Canvas,
}

/// Loads an indexed PNG as palette indices.
fn load_indexed_png(name: &str) -> Canvas {
    let rel = format!("images/{name}.png");
    let fail =
        |e: &dyn std::fmt::Display| -> ! { panic!("assets/{rel}: {e} (run tools/gen_images.py)") };
    let data = crate::asset_path::read(&rel).unwrap_or_else(|e| fail(&e));
    let mut reader =
        png::Decoder::new(std::io::Cursor::new(data)).read_info().unwrap_or_else(|e| fail(&e));
    let mut px = vec![0; reader.output_buffer_size().unwrap_or_else(|| fail(&"image too large"))];
    let frame = reader.next_frame(&mut px).unwrap_or_else(|e| fail(&e));
    if (frame.color_type, frame.bit_depth) != (png::ColorType::Indexed, png::BitDepth::Eight) {
        fail(&"not an 8-bit indexed PNG");
    }
    px.truncate(frame.buffer_size());
    Canvas::from_pixels(frame.width as usize, frame.height as usize, px)
}

fn setup(mut commands: Commands) {
    commands.insert_resource(CockpitArt { sprites: load_indexed_png("sprites") });
    commands.insert_resource(Fonts::builtin());
}

/// Everything the HUD, MFDs and panel instruments draw from.
#[derive(bevy::ecs::system::SystemParam)]
pub(crate) struct Displays<'w, 's> {
    pub hud: HudParams<'w, 's>,
    pub mfd: MfdParams<'w, 's>,
    pub panel: PanelParams<'w, 's>,
}

#[cfg(test)]
mod tests {
    use super::sprites::*;

    /// Every sprite rectangle lies inside the atlas and has something drawn in it (the atlas
    /// and `sprites` are kept in step by hand with tools/gen_images.py).
    #[test]
    fn sprites_in_atlas() {
        let atlas = super::load_indexed_png("sprites");
        let mut all = vec![FPM, AIRBASE, TARGET, HEADING_TAPE, GUN_SIGHT, WATERLINE];
        all.extend((0..10).map(led_digit));
        all.extend((0..ORDNANCE_COUNT).filter_map(ordnance));
        for s in all {
            assert!(s.x >= 0 && s.y >= 0, "{s:?}");
            assert!((s.x + s.w) as usize <= atlas.w && (s.y + s.h) as usize <= atlas.h, "{s:?}");
            let lit = (s.y..s.y + s.h)
                .flat_map(|y| (s.x..s.x + s.w).map(move |x| (x, y)))
                .any(|(x, y)| atlas.get(x, y) != 0);
            assert!(lit, "nothing drawn at {s:?}");
        }
    }
}
