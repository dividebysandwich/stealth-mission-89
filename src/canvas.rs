//! Palette-indexed pixel canvases (a 320x200 mode 13h drawing model), uploaded to
//! textures through the active palette. Cockpit panel, HUD and MFDs draw into canvases; the
//! textures are shown on world-space quads (so they also work in VR).

use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::palette::Palette;

pub struct CanvasPlugin;

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CanvasMips>()
            .add_plugins(
                bevy::render::extract_resource::ExtractResourcePlugin::<CanvasMips>::default(),
            )
            .add_systems(
                PostUpdate,
                upload_canvases.in_set(CanvasUpload).run_if(resource_exists::<Palette>),
            );
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render.add_systems(
                bevy::render::Render,
                write_canvas_mips.in_set(bevy::render::RenderSystems::PrepareResources),
            );
        }
    }
}

/// The lower mip levels of the mipmapped canvases uploaded this frame: Bevy rewrites only the
/// first level of a texture it already has, so these are written after it (`write_canvas_mips`).
#[derive(Resource, Default, Clone, bevy::render::extract_resource::ExtractResource)]
struct CanvasMips(Vec<(AssetId<Image>, usize, usize, usize, Vec<u8>)>);

fn write_canvas_mips(
    mips: Res<CanvasMips>,
    images: Res<bevy::render::render_asset::RenderAssets<bevy::render::texture::GpuImage>>,
    queue: Res<bevy::render::renderer::RenderQueue>,
) {
    use bevy::render::render_resource::{TexelCopyBufferLayout, TexelCopyTextureInfo};
    for (id, w, h, levels, data) in &mips.0 {
        let Some(gpu) = images.get(*id) else { continue };
        let mut offset = 0;
        for (level, (mw, mh)) in mip_sizes(*w, *h, *levels).into_iter().enumerate().skip(1) {
            let len = mw * mh * 4;
            let Some(bytes) = data.get(offset..offset + len) else { break };
            queue.write_texture(
                TexelCopyTextureInfo { mip_level: level as u32, ..gpu.texture.as_image_copy() },
                bytes,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(mw as u32 * 4),
                    rows_per_image: Some(mh as u32),
                },
                Extent3d { width: mw as u32, height: mh as u32, depth_or_array_layers: 1 },
            );
            offset += len;
        }
    }
}

/// Palette index treated as transparent when `CanvasTexture::transparent` is set.
pub const TRANSPARENT: u8 = 0;

#[derive(Clone)]
pub struct Canvas {
    /// Size in drawing units (pixels unless `scale` > 1).
    pub w: usize,
    pub h: usize,
    /// `scale` x `scale` texels per drawing unit, row by row (`pw` x `ph`).
    pub px: Vec<u8>,
    /// Drawing clip rectangle (x0, y0, x1, y1), inclusive, like the sim's page
    /// descriptors. `set` (and everything built on it) only writes inside it.
    pub clip: [i32; 4],
    /// Texels per drawing unit: pixel-art drawing (`set`, `line`, fonts, sprites) fills blocks,
    /// the strokes, rings and triangles of the high-resolution displays use the finer texels.
    pub scale: usize,
}

impl Canvas {
    pub fn new(w: usize, h: usize) -> Self {
        Self::from_pixels(w, h, vec![0; w * h])
    }

    /// A canvas of `w` x `h` drawing units with `scale` x `scale` texels each.
    pub fn scaled(w: usize, h: usize, scale: usize) -> Self {
        let scale = scale.max(1);
        Self {
            w,
            h,
            px: vec![0; w * h * scale * scale],
            clip: [0, 0, w as i32 - 1, h as i32 - 1],
            scale,
        }
    }

    pub fn from_pixels(w: usize, h: usize, px: Vec<u8>) -> Self {
        assert_eq!(px.len(), w * h);
        Self { w, h, px, clip: [0, 0, w as i32 - 1, h as i32 - 1], scale: 1 }
    }

    /// Size in texels.
    pub fn pw(&self) -> usize {
        self.w * self.scale
    }

    pub fn ph(&self) -> usize {
        self.h * self.scale
    }

    /// Sets one texel (texel coordinates), inside the clip rectangle.
    #[inline]
    fn set_texel(&mut self, x: i32, y: i32, c: u8) {
        let s = self.scale as i32;
        let [x0, y0, x1, y1] = self.clip;
        if x >= x0 * s && y >= y0 * s && x < (x1 + 1) * s && y < (y1 + 1) * s {
            let pw = self.pw();
            self.px[y as usize * pw + x as usize] = c;
        }
    }

    pub fn clear(&mut self, c: u8) {
        self.px.fill(c);
    }

    /// Sets the clip rectangle (inclusive), limited to the canvas.
    pub fn set_clip(&mut self, x0: i32, y0: i32, x1: i32, y1: i32) {
        self.clip = [x0.max(0), y0.max(0), x1.min(self.w as i32 - 1), y1.min(self.h as i32 - 1)];
    }

    pub fn reset_clip(&mut self) {
        self.clip = [0, 0, self.w as i32 - 1, self.h as i32 - 1];
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, c: u8) {
        let [x0, y0, x1, y1] = self.clip;
        if x >= x0 && y >= y0 && x <= x1 && y <= y1 {
            if self.scale == 1 {
                self.px[y as usize * self.w + x as usize] = c;
            } else {
                let (s, pw) = (self.scale, self.pw());
                for row in self.px[y as usize * s * pw..].chunks_mut(pw).take(s) {
                    row[x as usize * s..(x as usize + 1) * s].fill(c);
                }
            }
        }
    }

    /// Copies a rectangle opaquely (the driver's copyRect).
    #[allow(clippy::too_many_arguments)]
    pub fn copy_rect(&mut self, src: &Canvas, sx: i32, sy: i32, w: i32, h: i32, dx: i32, dy: i32) {
        self.blit(src, sx, sy, w, h, dx, dy, None, None);
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && y >= 0 && (x as usize) < self.w && (y as usize) < self.h {
            self.px[y as usize * self.scale * self.pw() + x as usize * self.scale]
        } else {
            0
        }
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        self.hline(x, x + w - 1, y, c);
        self.hline(x, x + w - 1, y + h - 1, c);
        for yy in y..y + h {
            self.set(x, yy, c);
            self.set(x + w - 1, yy, c);
        }
    }

    pub fn hline(&mut self, x0: i32, x1: i32, y: i32, c: u8) {
        for x in x0.min(x1)..=x0.max(x1) {
            self.set(x, y, c);
        }
    }

    /// Bresenham line, as classic HUD lines are drawn.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u8) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        loop {
            self.set(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Copies a rectangle from another canvas (e.g. the cockpit sprite atlas). Pixels equal to
    /// `key` are skipped; `recolor` replaces every drawn pixel's colour if set.
    #[allow(clippy::too_many_arguments)]
    pub fn blit(
        &mut self,
        src: &Canvas,
        sx: i32,
        sy: i32,
        w: i32,
        h: i32,
        dx: i32,
        dy: i32,
        key: Option<u8>,
        recolor: Option<u8>,
    ) {
        if src.scale == self.scale && self.scale > 1 {
            // Texel by texel between canvases of the same resolution (pages drawn apart).
            let s = self.scale as i32;
            let (spw, sph) = (src.pw() as i32, src.ph() as i32);
            for y in 0..h * s {
                for x in 0..w * s {
                    let (tx, ty) = (sx * s + x, sy * s + y);
                    if tx < 0 || ty < 0 || tx >= spw || ty >= sph {
                        continue;
                    }
                    let c = src.px[(ty * spw + tx) as usize];
                    if Some(c) != key {
                        self.set_texel(dx * s + x, dy * s + y, recolor.unwrap_or(c));
                    }
                }
            }
            return;
        }
        for y in 0..h {
            for x in 0..w {
                let c = src.get(sx + x, sy + y);
                if Some(c) == key {
                    continue;
                }
                self.set(dx + x, dy + y, recolor.unwrap_or(c));
            }
        }
    }
}

impl Canvas {
    /// `blit` with every source pixel drawn as a `scale` x `scale` block.
    #[allow(clippy::too_many_arguments)]
    pub fn blit_scaled(
        &mut self,
        src: &Canvas,
        sx: i32,
        sy: i32,
        w: i32,
        h: i32,
        dx: i32,
        dy: i32,
        scale: i32,
        key: Option<u8>,
        recolor: Option<u8>,
    ) {
        for y in 0..h {
            for x in 0..w {
                let c = src.get(sx + x, sy + y);
                if Some(c) == key {
                    continue;
                }
                self.fill_rect(dx + x * scale, dy + y * scale, scale, scale, recolor.unwrap_or(c));
            }
        }
    }

    /// Filled triangle: every pixel whose centre is inside (or on an edge).
    pub fn triangle(&mut self, p: [Vec2; 3], c: u8) {
        let edge = |a: Vec2, b: Vec2, q: Vec2| (b - a).perp_dot(q - a);
        let area = edge(p[0], p[1], p[2]);
        if area == 0.0 {
            return;
        }
        let s = self.scale as f32;
        let lo = (p[0].min(p[1]).min(p[2]) * s).floor();
        let hi = (p[0].max(p[1]).max(p[2]) * s).ceil();
        for y in lo.y as i32..hi.y as i32 {
            for x in lo.x as i32..hi.x as i32 {
                let q = Vec2::new(x as f32 + 0.5, y as f32 + 0.5) / s;
                let w = [edge(p[1], p[2], q), edge(p[2], p[0], q), edge(p[0], p[1], q)];
                if w.iter().all(|&w| w * area >= 0.0) {
                    self.set_texel(x, y, c);
                }
            }
        }
    }

    /// Circle outline (polyline).
    pub fn circle(&mut self, cx: f32, cy: f32, r: f32, c: u8) {
        let steps = ((r * 1.5) as i32).clamp(12, 256);
        let mut prev = (cx + r, cy);
        for i in 1..=steps {
            let t = i as f32 / steps as f32 * std::f32::consts::TAU;
            let p = (cx + r * t.cos(), cy + r * t.sin());
            self.line(prev.0 as i32, prev.1 as i32, p.0 as i32, p.1 as i32, c);
            prev = p;
        }
    }
}

/// Thick strokes for the high-resolution displays (sub-pixel endpoints, round pens).
impl Canvas {
    /// Line of width `w` from a to b.
    pub fn stroke(&mut self, a: Vec2, b: Vec2, w: f32, c: u8) {
        let [x0, y0, x1, y1] = self.clip;
        let (lo, hi) = (a.min(b) - w, a.max(b) + w);
        if hi.x < x0 as f32 || hi.y < y0 as f32 || lo.x > x1 as f32 + 1.0 || lo.y > y1 as f32 + 1.0
        {
            return;
        }
        let len = a.distance(b);
        // Far off-canvas endpoints (projected cues): clip the segment to a padded canvas first.
        let (a, b) = if len > 4.0 * (self.w + self.h) as f32 {
            match clip_segment(
                a,
                b,
                Vec2::new(x0 as f32 - w, y0 as f32 - w),
                Vec2::new(x1 as f32 + w, y1 as f32 + w),
            ) {
                Some(s) => s,
                None => return,
            }
        } else {
            (a, b)
        };
        self.capsule(a, b, w, c);
    }

    /// Every texel whose centre is within the pen's reach of the segment a-b: row by row, the
    /// span where the row crosses the capsule (one span, the capsule being convex).
    fn capsule(&mut self, a: Vec2, b: Vec2, w: f32, c: u8) {
        let s = self.scale as f32;
        let (a, b) = (a * s, b * s);
        let r = w * s * 0.5;
        // As far as a round pen of width w reaches: half a texel past its radius.
        let rr = r * r + 0.25;
        let reach = rr.sqrt();
        let si = self.scale as i32;
        let [x0, y0, x1, y1] = self.clip;
        let (tx0, ty0, tx1, ty1) = (x0 * si, y0 * si, (x1 + 1) * si - 1, (y1 + 1) * si - 1);
        let top = ((a.y.min(b.y) - reach - 0.5).floor() as i32).max(ty0);
        let bottom = ((a.y.max(b.y) + reach - 0.5).ceil() as i32).min(ty1);
        let d = b - a;
        let l2 = d.length_squared();
        let span = reach * l2.sqrt();
        let pw = self.pw();
        for y in top..=bottom {
            let qy = y as f32 + 0.5;
            let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
            // The round ends.
            for p in [a, b] {
                let h = rr - (qy - p.y) * (qy - p.y);
                if h >= 0.0 {
                    let h = h.sqrt();
                    (lo, hi) = (lo.min(p.x - h), hi.max(p.x + h));
                }
            }
            // The band between them: within reach of the line, projecting inside the segment.
            if l2 > 1e-6 {
                let dy = qy - a.y;
                let (mut l, mut h) = (f32::NEG_INFINITY, f32::INFINITY);
                // |d.x * dy - d.y * (qx - a.x)| <= span
                if d.y.abs() > 1e-6 {
                    let (e0, e1) = ((d.x * dy - span) / d.y, (d.x * dy + span) / d.y);
                    (l, h) = (l.max(a.x + e0.min(e1)), h.min(a.x + e0.max(e1)));
                } else if (d.x * dy).abs() > span {
                    h = l;
                }
                // 0 <= d.x * (qx - a.x) + d.y * dy <= l2
                if d.x.abs() > 1e-6 {
                    let (e0, e1) = (-d.y * dy / d.x, (l2 - d.y * dy) / d.x);
                    (l, h) = (l.max(a.x + e0.min(e1)), h.min(a.x + e0.max(e1)));
                } else if !(0.0..=l2).contains(&(d.y * dy)) {
                    h = l;
                }
                if l <= h {
                    (lo, hi) = (lo.min(l), hi.max(h));
                }
            }
            let x0 = ((lo - 0.5).ceil() as i32).max(tx0);
            let x1 = ((hi - 0.5).floor() as i32).min(tx1);
            if x0 <= x1 {
                let row = y as usize * pw;
                self.px[row + x0 as usize..=row + x1 as usize].fill(c);
            }
        }
    }

    /// Polyline of width `w`.
    pub fn polyline(&mut self, pts: &[Vec2], w: f32, c: u8) {
        for p in pts.windows(2) {
            self.stroke(p[0], p[1], w, c);
        }
    }

    /// Dashed line: `dash` pixels drawn, `gap` pixels skipped.
    pub fn dashed(&mut self, a: Vec2, b: Vec2, dash: f32, gap: f32, w: f32, c: u8) {
        let len = a.distance(b);
        if len <= 0.0 {
            return;
        }
        let d = (b - a) / len;
        let mut t = 0.0;
        while t < len {
            let e = (t + dash).min(len);
            self.stroke(a + d * t, a + d * e, w, c);
            t = e + gap;
        }
    }

    /// Circle outline of radius r and width w.
    pub fn ring(&mut self, centre: Vec2, r: f32, w: f32, c: u8) {
        self.arc(centre, r, 0.0, std::f32::consts::TAU, w, c);
    }

    /// Arc from angle a0 to a1 (radians, 0 = right, positive = clockwise on screen).
    pub fn arc(&mut self, centre: Vec2, r: f32, a0: f32, a1: f32, w: f32, c: u8) {
        if !self.ring_shows(centre, r, w) {
            return;
        }
        // About 3 pixel chords, at least 8 a turn.
        let turn = (a1 - a0).abs() / std::f32::consts::TAU;
        let steps = ((r * (a1 - a0).abs() / 3.0).max(8.0 * turn).ceil() as i32).clamp(1, 256);
        let mut prev = centre + Vec2::new(a0.cos(), a0.sin()) * r;
        for i in 1..=steps {
            let t = a0 + (a1 - a0) * i as f32 / steps as f32;
            let p = centre + Vec2::new(t.cos(), t.sin()) * r;
            self.stroke(prev, p, w, c);
            prev = p;
        }
    }

    /// Whether a circle of radius r and pen width w crosses the clip rectangle (threat rings
    /// far larger than the screen, or wholly off it, are skipped).
    pub fn ring_shows(&self, centre: Vec2, r: f32, w: f32) -> bool {
        let (lo, hi) = self.clip_box(w);
        let near = centre.clamp(lo, hi).distance(centre);
        let far = (centre - lo).abs().max((centre - hi).abs()).length();
        near <= r + w && far >= r - w
    }

    /// Whether a point is within the clip rectangle grown by `pad`.
    pub fn shows(&self, p: Vec2, pad: f32) -> bool {
        let (lo, hi) = self.clip_box(pad);
        p.cmpge(lo).all() && p.cmple(hi).all()
    }

    /// The clip rectangle (drawing units, its far edges inclusive of their pixels) grown by
    /// `pad`.
    fn clip_box(&self, pad: f32) -> (Vec2, Vec2) {
        let [x0, y0, x1, y1] = self.clip;
        (Vec2::new(x0 as f32, y0 as f32) - pad, Vec2::new((x1 + 1) as f32, (y1 + 1) as f32) + pad)
    }

    /// Rectangle outline (corners a, b) of width w.
    pub fn frame(&mut self, a: Vec2, b: Vec2, w: f32, c: u8) {
        let (p, q) = (a.min(b), a.max(b));
        self.polyline(&[p, Vec2::new(q.x, p.y), q, Vec2::new(p.x, q.y), p], w, c);
    }
}

/// Liang-Barsky clip of segment a-b to the box lo..hi.
fn clip_segment(a: Vec2, b: Vec2, lo: Vec2, hi: Vec2) -> Option<(Vec2, Vec2)> {
    let d = b - a;
    let (mut t0, mut t1) = (0.0f32, 1.0f32);
    for (p, q) in [(-d.x, a.x - lo.x), (d.x, hi.x - a.x), (-d.y, a.y - lo.y), (d.y, hi.y - a.y)] {
        if p == 0.0 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
        }
    }
    (t0 <= t1).then(|| (a + d * t0, a + d * t1))
}

/// A true-colour picture (8-bit sRGB RGBA, row by row) shown over a canvas past its palette.
pub struct Picture {
    pub w: usize,
    pub h: usize,
    pub rgba: Vec<u8>,
}

/// A canvas shown through a texture. Set `dirty` after drawing.
#[derive(Component)]
pub struct CanvasTexture {
    pub canvas: Canvas,
    pub image: Handle<Image>,
    /// Index 0 becomes fully transparent (overlays such as the HUD or the panel's window area).
    pub transparent: bool,
    /// Optional per-pixel mask: index 0 is only transparent where the mask is true.
    pub mask: Option<Vec<bool>>,
    /// Pictures laid over the canvas at these texels (photos on the kneeboard).
    pub pictures: Vec<(UVec2, Arc<Picture>)>,
    pub dirty: bool,
    /// Mip levels of the texture (1: none; the others are built from the canvas on every upload).
    mips: usize,
    /// One channel, the ink's coverage (`coverage`).
    coverage: bool,
    /// The pixels, transparency and pictures last uploaded: a redraw that changed nothing (most
    /// frames for the instrument atlas) skips the conversion and the texture upload.
    shown: Option<(Vec<u8>, bool, Placed)>,
}

/// Picture positions and identities (`CanvasTexture::pictures`).
type Placed = Vec<(UVec2, usize)>;

impl CanvasTexture {
    /// Where the pictures are and which they are.
    fn placed(&self) -> Placed {
        self.pictures.iter().map(|(at, p)| (*at, Arc::as_ptr(p) as usize)).collect()
    }
}

impl CanvasTexture {
    pub fn new(canvas: Canvas, images: &mut Assets<Image>, transparent: bool) -> Self {
        let mut image = blank(&canvas, TextureFormat::Rgba8UnormSrgb);
        image.sampler = ImageSampler::nearest();
        Self::with_image(canvas, images.add(image), transparent, 1, false)
    }

    /// A one-channel texture of where the canvas is inked (any index but 0), linear filtered:
    /// the modern HUD and the helmet display, drawn in one colour, upload a quarter of the
    /// bytes of a palette picture every frame.
    pub fn coverage(canvas: Canvas, images: &mut Assets<Image>) -> Self {
        let mut image = blank(&canvas, TextureFormat::R8Unorm);
        image.sampler = ImageSampler::linear();
        Self::with_image(canvas, images.add(image), true, 1, true)
    }

    fn with_image(
        canvas: Canvas,
        image: Handle<Image>,
        transparent: bool,
        mips: usize,
        coverage: bool,
    ) -> Self {
        Self {
            canvas,
            image,
            transparent,
            mask: None,
            pictures: Vec::new(),
            dirty: true,
            mips,
            coverage,
            shown: None,
        }
    }

    /// A texture with mipmaps and trilinear, anisotropic filtering: thin lines stay readable
    /// on screens and dials seen from further away than one texel per pixel.
    pub fn mipmapped(canvas: Canvas, images: &mut Assets<Image>, transparent: bool) -> Self {
        Self::mipmapped_levels(canvas, images, transparent, usize::MAX)
    }

    /// `mipmapped` with at most `max_levels` levels (an atlas: the coarser ones would blend
    /// neighbouring faces).
    pub fn mipmapped_levels(
        canvas: Canvas,
        images: &mut Assets<Image>,
        transparent: bool,
        max_levels: usize,
    ) -> Self {
        let levels = mip_sizes(canvas.pw(), canvas.ph(), max_levels);
        let mut image = blank(&canvas, TextureFormat::Rgba8UnormSrgb);
        image.texture_descriptor.mip_level_count = levels.len() as u32;
        image.data = Some(vec![0; levels.iter().map(|(w, h)| w * h * 4).sum()]);
        image.sampler = ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
            anisotropy_clamp: 16,
            ..bevy::image::ImageSamplerDescriptor::linear()
        });
        Self::with_image(canvas, images.add(image), transparent, levels.len(), false)
    }
}

/// A zeroed texture the canvas' size, RGBA or (`R8Unorm`) one channel.
fn blank(canvas: &Canvas, format: TextureFormat) -> Image {
    let size = Extent3d {
        width: canvas.pw() as u32,
        height: canvas.ph() as u32,
        depth_or_array_layers: 1,
    };
    let pixel = if format == TextureFormat::R8Unorm { &[0][..] } else { &[0, 0, 0, 0] };
    Image::new_fill(size, TextureDimension::D2, pixel, format, RenderAssetUsages::default())
}

/// Mip level sizes down to 1x1, at most `max` of them.
fn mip_sizes(w: usize, h: usize, max: usize) -> Vec<(usize, usize)> {
    let mut out = vec![(w, h)];
    while out.len() < max && out.last().is_some_and(|&(w, h)| w > 1 || h > 1) {
        let (w, h) = *out.last().unwrap();
        out.push(((w / 2).max(1), (h / 2).max(1)));
    }
    out
}

/// Fills the mip levels after the first (RGBA, 2x2 box filter) in `data`.
fn build_mips(data: &mut [u8], w: usize, h: usize, levels: usize) {
    let mut start = 0;
    for pair in mip_sizes(w, h, levels).windows(2) {
        let ((sw, sh), (dw, dh)) = (pair[0], pair[1]);
        let (src, dst) = data[start..].split_at_mut(sw * sh * 4);
        let src = src.as_chunks::<4>().0;
        let dst = &mut dst.as_chunks_mut::<4>().0[..dw * dh];
        for (y, out) in dst.chunks_exact_mut(dw).enumerate() {
            let rows = [(2 * y).min(sh - 1), (2 * y + 1).min(sh - 1)].map(|r| &src[r * sw..][..sw]);
            for (x, o) in out.iter_mut().enumerate() {
                let (c0, c1) = ((2 * x).min(sw - 1), (2 * x + 1).min(sw - 1));
                *o = average([rows[0][c0], rows[0][c1], rows[1][c0], rows[1][c1]]);
            }
        }
        start += sw * sh * 4;
    }
}

/// The rounded average of four RGBA texels, channel by channel: two channels at a time in the
/// halves of a u32 (four bytes' sum fits in 16 bits).
#[inline]
fn average(px: [[u8; 4]; 4]) -> [u8; 4] {
    const LOW: u32 = 0x00ff_00ff;
    let (mut even, mut odd) = (0x0002_0002, 0x0002_0002);
    for p in px {
        let p = u32::from_le_bytes(p);
        even += p & LOW;
        odd += (p >> 8) & LOW;
    }
    (((even >> 2) & LOW) | (((odd >> 2) & LOW) << 8)).to_le_bytes()
}

/// Copies the drawn canvases into their textures (PostUpdate): canvases drawn in PostUpdate
/// go before it.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CanvasUpload;

fn upload_canvases(
    palette: Res<Palette>,
    mut images: ResMut<Assets<Image>>,
    mut q: Query<&mut CanvasTexture>,
    mut mips: ResMut<CanvasMips>,
) {
    if !mips.0.is_empty() {
        mips.0.clear();
    }
    let all = palette.is_changed();
    // Palette as 8-bit sRGB RGBA (in memory order), and with index 0 see-through.
    let mut lut = [0u32; 256];
    for (i, v) in lut.iter_mut().enumerate() {
        let c = palette.color(i as u8).to_srgba();
        *v = u32::from_le_bytes([
            (c.red * 255.0) as u8,
            (c.green * 255.0) as u8,
            (c.blue * 255.0) as u8,
            255,
        ]);
    }
    let mut see_lut = lut;
    see_lut[TRANSPARENT as usize] = 0;
    for mut ct in &mut q {
        if !(ct.dirty || all) {
            continue;
        }
        let ct = ct.bypass_change_detection();
        ct.dirty = false;
        let placed = ct.placed();
        let unchanged = ct.shown.as_ref().is_some_and(|(px, t, pics)| {
            *t == ct.transparent && *px == ct.canvas.px && *pics == placed
        });
        if !all && ct.mask.is_none() && unchanged {
            continue;
        }
        let Some(mut img) = images.get_mut(&ct.image) else { continue };
        let Some(data) = img.data.as_mut() else { continue };
        if ct.coverage {
            for (o, &c) in data.iter_mut().zip(&ct.canvas.px) {
                *o = if c == TRANSPARENT { 0 } else { 255 };
            }
            remember(ct, placed);
            continue;
        }
        let base = ct.canvas.px.len() * 4;
        let out = data[..base].as_chunks_mut::<4>().0.iter_mut().zip(&ct.canvas.px);
        match (&ct.mask, ct.transparent) {
            (Some(mask), true) => {
                for (i, (o, &c)) in out.enumerate() {
                    let l = if mask[i] { &see_lut } else { &lut };
                    *o = l[c as usize].to_le_bytes();
                }
            }
            (_, t) => {
                let l = if t { &see_lut } else { &lut };
                for (o, &c) in out {
                    *o = l[c as usize].to_le_bytes();
                }
            }
        }
        let (pw, ph) = (ct.canvas.pw(), ct.canvas.ph());
        for (at, pic) in &ct.pictures {
            let (x0, y0) = (at.x as usize, at.y as usize);
            let w = pic.w.min(pw.saturating_sub(x0));
            for y in 0..pic.h.min(ph.saturating_sub(y0)) {
                let o = ((y0 + y) * pw + x0) * 4;
                data[o..o + w * 4].copy_from_slice(&pic.rgba[y * pic.w * 4..][..w * 4]);
            }
        }
        if ct.mips > 1 {
            build_mips(data, ct.canvas.pw(), ct.canvas.ph(), ct.mips);
            mips.0.push((
                ct.image.id(),
                ct.canvas.pw(),
                ct.canvas.ph(),
                ct.mips,
                data[base..].to_vec(),
            ));
        }
        remember(ct, placed);
    }
}

/// Keeps what was uploaded, to skip uploading it again unchanged.
fn remember(ct: &mut CanvasTexture, placed: Placed) {
    match &mut ct.shown {
        Some((px, t, pics)) => {
            px.clone_from(&ct.canvas.px);
            *t = ct.transparent;
            *pics = placed;
        }
        None => ct.shown = Some((ct.canvas.px.clone(), ct.transparent, placed)),
    }
}
