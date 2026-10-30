//! Palette-indexed pixel canvases (a 320x200 mode 13h drawing model), uploaded to
//! textures through the active palette. Cockpit panel, HUD and MFDs draw into canvases; the
//! textures are shown on world-space quads (so they also work in VR).

use std::sync::Arc;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};

use crate::palette::Palette;

pub struct CanvasPlugin;

impl Plugin for CanvasPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            upload_canvases.in_set(CanvasUpload).run_if(resource_exists::<Palette>),
        );
        // With Bevy's renderer the render world writes the canvases' textures; without it
        // (`--gl-vr`) glvr uploads the changed image assets.
        if let Some(render) = app.get_sub_app_mut(bevy::render::RenderApp) {
            render
                .init_resource::<gpu::Pending>()
                .add_systems(bevy::render::ExtractSchedule, gpu::extract)
                .add_systems(
                    bevy::render::Render,
                    gpu::write.in_set(bevy::render::RenderSystems::PrepareResources),
                );
            app.init_resource::<CanvasWrites>();
        }
    }
}

/// The canvases' new pixels for the render world (`gpu`): with Bevy's renderer the image assets
/// stay as they were made, so Bevy doesn't copy each changed one whole (its mip levels too)
/// into the render world every frame.
#[derive(Resource, Default)]
struct CanvasWrites {
    queue: Vec<CanvasWrite>,
    /// Buffers back from the render world, to fill again (new ones of these sizes would cost
    /// the page faults of fresh memory every frame).
    spare: Vec<Vec<u8>>,
}

/// Mip levels of `CanvasTexture::mipmapped`.
const MIPS: usize = 4;

/// Buffers kept for reuse.
const SPARE: usize = 16;

struct CanvasWrite {
    image: AssetId<Image>,
    /// The texture's width, and the rows written: from `top`, as many as `data` holds.
    width: u32,
    top: u32,
    /// Bytes per texel: 4, or 1 for a coverage texture.
    texel: u32,
    /// Mip levels, made from the first on the GPU.
    mips: u32,
    /// The first level's rows.
    data: Vec<u8>,
}

/// Render world: the canvas writes into their textures, and the mip levels drawn from them.
mod gpu {
    use bevy::prelude::*;
    use bevy::render::MainWorld;
    use bevy::render::render_asset::RenderAssets;
    use bevy::render::render_resource::*;
    use bevy::render::renderer::{PendingCommandBuffers, RenderDevice, RenderQueue};
    use bevy::render::texture::GpuImage;

    use super::{CanvasWrite, CanvasWrites};

    /// Writes waiting for their textures (a canvas made this frame), in order, and how many
    /// frames they have waited.
    #[derive(Resource, Default)]
    pub struct Pending {
        writes: Vec<(CanvasWrite, u8)>,
        /// Written buffers, going back to the main world.
        done: Vec<Vec<u8>>,
    }

    /// Frames a write waits for its texture before it is dropped (the image is gone).
    const PATIENCE: u8 = 30;

    pub fn extract(mut main: ResMut<MainWorld>, mut pending: ResMut<Pending>) {
        let Some(mut writes) = main.get_resource_mut::<CanvasWrites>() else { return };
        let pending = &mut *pending;
        pending.writes.extend(writes.queue.drain(..).map(|w| (w, 0)));
        let room = super::SPARE.saturating_sub(writes.spare.len());
        writes.spare.extend(pending.done.drain(..).take(room));
        pending.done.clear();
    }

    pub fn write(
        mut pending: ResMut<Pending>,
        images: Res<RenderAssets<GpuImage>>,
        device: Res<RenderDevice>,
        queue: Res<RenderQueue>,
        mut submit: ResMut<PendingCommandBuffers>,
        mut mipper: Local<Option<Mipper>>,
    ) {
        if pending.writes.is_empty() {
            return;
        }
        let mut encoder = None;
        let Pending { writes, done } = &mut *pending;
        writes.retain_mut(|(w, age)| {
            let Some(gpu) = images.get(w.image) else {
                *age += 1;
                return *age < PATIENCE;
            };
            let (width, row) = (w.width, w.width * w.texel);
            let height = w.data.len() as u32 / row;
            queue.write_texture(
                TexelCopyTextureInfo {
                    origin: Origin3d { x: 0, y: w.top, z: 0 },
                    ..gpu.texture.as_image_copy()
                },
                &w.data,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(row),
                    rows_per_image: Some(height),
                },
                Extent3d { width, height, depth_or_array_layers: 1 },
            );
            if w.mips > 1 {
                let encoder = encoder.get_or_insert_with(|| {
                    device.create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("canvas_mips"),
                    })
                });
                mipper.get_or_insert_with(|| Mipper::new(&device)).levels(
                    &device,
                    encoder,
                    &gpu.texture,
                    w.mips,
                );
            }
            done.push(std::mem::take(&mut w.data));
            false
        });
        // In the frame's submission, ahead of the passes that show them (and after the writes:
        // a submission carries the queue's writes before its commands).
        if let Some(encoder) = encoder {
            submit.push([encoder.finish()]);
        }
    }

    /// Each mip level drawn from the one above, a texel from the bilinear sample between four:
    /// their average. Through views as plain RGBA, so the sRGB bytes average as they did on
    /// the CPU.
    pub struct Mipper {
        pipeline: RenderPipeline,
        layout: BindGroupLayout,
        sampler: Sampler,
    }

    /// The view format of the mip passes (`CanvasTexture::mipmapped_levels` allows it).
    pub const VIEW: TextureFormat = TextureFormat::Rgba8Unorm;

    const SHADER: &str = r"
        @group(0) @binding(0) var src: texture_2d<f32>;
        @group(0) @binding(1) var bilinear: sampler;

        struct Out {
            @builtin(position) position: vec4<f32>,
            @location(0) uv: vec2<f32>,
        };

        @vertex
        fn vertex(@builtin(vertex_index) i: u32) -> Out {
            let uv = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
            var out: Out;
            out.position = vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, 0.0, 1.0);
            out.uv = uv;
            return out;
        }

        @fragment
        fn fragment(in: Out) -> @location(0) vec4<f32> {
            return textureSampleLevel(src, bilinear, in.uv, 0.0);
        }
    ";

    impl Mipper {
        fn new(device: &RenderDevice) -> Self {
            let module = device.create_and_validate_shader_module(ShaderModuleDescriptor {
                label: Some("canvas_mips"),
                source: ShaderSource::Wgsl(SHADER.into()),
            });
            let layout = device.create_bind_group_layout(
                "canvas_mips",
                &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            );
            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("canvas_mips"),
                bind_group_layouts: &[Some(&layout)],
                immediate_size: 0,
            });
            let pipeline = device.create_render_pipeline(&RawRenderPipelineDescriptor {
                label: Some("canvas_mips"),
                layout: Some(&pipeline_layout),
                vertex: RawVertexState {
                    module: &module,
                    entry_point: Some("vertex"),
                    compilation_options: default(),
                    buffers: &[],
                },
                primitive: default(),
                depth_stencil: None,
                multisample: default(),
                fragment: Some(RawFragmentState {
                    module: &module,
                    entry_point: Some("fragment"),
                    compilation_options: default(),
                    targets: &[Some(ColorTargetState {
                        format: VIEW,
                        blend: None,
                        write_mask: ColorWrites::ALL,
                    })],
                }),
                multiview_mask: None,
                cache: None,
            });
            let sampler = device.create_sampler(&SamplerDescriptor {
                label: Some("canvas_mips"),
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                ..default()
            });
            Self { pipeline, layout, sampler }
        }

        fn levels(
            &self,
            device: &RenderDevice,
            encoder: &mut CommandEncoder,
            texture: &Texture,
            mips: u32,
        ) {
            let view = |level| {
                texture.create_view(&TextureViewDescriptor {
                    format: Some(VIEW),
                    base_mip_level: level,
                    mip_level_count: Some(1),
                    ..default()
                })
            };
            let mut src = view(0);
            for level in 1..mips {
                let dst = view(level);
                let bind = device.create_bind_group(
                    "canvas_mips",
                    &self.layout,
                    &BindGroupEntries::sequential((&src, &self.sampler)),
                );
                let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("canvas_mips"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &dst,
                        depth_slice: None,
                        resolve_target: None,
                        ops: Operations { load: LoadOp::Clear(default()), store: StoreOp::Store },
                    })],
                    ..default()
                });
                pass.set_pipeline(&self.pipeline);
                pass.set_bind_group(0, &bind, &[]);
                pass.draw(0..3, 0..1);
                drop(pass);
                src = dst;
            }
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

    /// Fills the shape outlined by closed `contours` (non-zero winding, so a glyph's holes stay
    /// open), each texel it covers set to `ink(the colour under it, the share covered)`: the
    /// edges anti-aliased where the palette ramps between the two colours.
    pub fn fill_contours(&mut self, contours: &[Vec<Vec2>], ink: impl Fn(u8, f32) -> u8) {
        // Sub-scanlines per texel row.
        const SUB: usize = 4;
        let s = self.scale as f32;
        let edges: Vec<(Vec2, Vec2)> = contours
            .iter()
            .flat_map(|c| c.iter().zip(c.iter().cycle().skip(1)).map(|(&a, &b)| (a * s, b * s)))
            .filter(|(a, b)| a.y != b.y)
            .collect();
        let Some((lo, hi)) = edges
            .iter()
            .map(|&(a, b)| (a.min(b), a.max(b)))
            .reduce(|(l0, h0), (l1, h1)| (l0.min(l1), h0.max(h1)))
        else {
            return;
        };
        let si = self.scale as i32;
        let [x0, y0, x1, y1] = self.clip;
        let left = (lo.x.floor() as i32).max(x0 * si);
        let right = (hi.x.ceil() as i32).min((x1 + 1) * si);
        let top = (lo.y.floor() as i32).max(y0 * si);
        let bottom = (hi.y.ceil() as i32).min((y1 + 1) * si);
        if left >= right {
            return;
        }
        let pw = self.pw();
        let mut cover = vec![0.0f32; (right - left) as usize];
        let mut xs: Vec<(f32, i32)> = Vec::new();
        for y in top..bottom {
            cover.fill(0.0);
            for k in 0..SUB {
                let sy = y as f32 + (k as f32 + 0.5) / SUB as f32;
                xs.clear();
                for &(a, b) in &edges {
                    let (p, q, dir) = if a.y < b.y { (a, b, 1) } else { (b, a, -1) };
                    if sy >= p.y && sy < q.y {
                        xs.push((p.x + (sy - p.y) / (q.y - p.y) * (q.x - p.x), dir));
                    }
                }
                xs.sort_by(|a, b| a.0.total_cmp(&b.0));
                let mut winding = 0;
                for pair in xs.windows(2) {
                    winding += pair[0].1;
                    if winding == 0 {
                        continue;
                    }
                    // The span's share of each texel it crosses.
                    let (a, b) = (pair[0].0.max(left as f32), pair[1].0.min(right as f32));
                    let mut x = a.floor();
                    while x < b {
                        let share = (b.min(x + 1.0) - a.max(x)) / SUB as f32;
                        cover[(x as i32 - left) as usize] += share;
                        x += 1.0;
                    }
                }
            }
            let row = y as usize * pw;
            for (i, &k) in cover.iter().enumerate() {
                if k > 0.01 {
                    let px = &mut self.px[row + left as usize + i];
                    *px = ink(*px, k.min(1.0));
                }
            }
        }
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
    /// Colours of its own (8-bit sRGB RGBA words, in memory order) instead of the live palette:
    /// the in-flight menu keeps its greens at every time of day.
    pub colours: Option<Arc<[u32; 256]>>,
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
            colours: None,
            dirty: true,
            mips,
            coverage,
            shown: None,
        }
    }

    /// A texture with mipmaps and trilinear, anisotropic filtering: thin lines stay readable
    /// on screens and dials seen from further away than one texel per pixel. Down to an eighth
    /// of the size: the cockpit's screens are never seen smaller, and each level is a render
    /// pass on every upload.
    pub fn mipmapped(canvas: Canvas, images: &mut Assets<Image>, transparent: bool) -> Self {
        Self::mipmapped_levels(canvas, images, transparent, MIPS)
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
        // The render world draws the levels (`gpu::Mipper`).
        image.texture_descriptor.usage |= TextureUsages::RENDER_ATTACHMENT;
        image.texture_descriptor.view_formats = &[gpu::VIEW];
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
    mut writes: Option<ResMut<CanvasWrites>>,
) {
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
        let (pw, ph) = (ct.canvas.pw(), ct.canvas.ph());
        // The rows that changed since the last upload; everything after a palette change, for
        // the masked canvases and those with pictures.
        let rows = match &ct.shown {
            Some((px, t, pics))
                if !all
                    && ct.mask.is_none()
                    && ct.pictures.is_empty()
                    && *t == ct.transparent
                    && *pics == placed
                    && px.len() == ct.canvas.px.len() =>
            {
                changed_rows(px, &ct.canvas.px, pw)
            }
            Some((px, t, pics))
                if !all
                    && ct.mask.is_none()
                    && *t == ct.transparent
                    && *pics == placed
                    && *px == ct.canvas.px =>
            {
                None
            }
            _ => Some(0..ph),
        };
        let Some(rows) = rows else { continue };
        let texel = if ct.coverage { 1 } else { 4 };
        let span = rows.start * pw * texel..rows.end * pw * texel;
        if let Some(writes) = writes.as_mut() {
            let len = span.len();
            let mut data = match writes.spare.iter().position(|b| b.capacity() >= len) {
                Some(i) => writes.spare.swap_remove(i),
                None => Vec::with_capacity(len),
            };
            data.clear();
            data.resize(len, 0);
            first_level(ct, &lut, &see_lut, rows.clone(), &mut data);
            writes.queue.push(CanvasWrite {
                image: ct.image.id(),
                width: pw as u32,
                top: rows.start as u32,
                texel: texel as u32,
                mips: ct.mips as u32,
                data,
            });
        } else {
            let Some(mut img) = images.get_mut(&ct.image) else { continue };
            let Some(data) = img.data.as_mut() else { continue };
            first_level(ct, &lut, &see_lut, rows, &mut data[span]);
            if ct.mips > 1 {
                build_mips(data, pw, ph, ct.mips);
            }
        }
        remember(ct, placed);
    }
}

/// The texel rows (first to past the last) where two pictures of a canvas `pw` texels wide
/// differ, if they do.
fn changed_rows(old: &[u8], new: &[u8], pw: usize) -> Option<std::ops::Range<usize>> {
    let pairs = || old.chunks_exact(pw).zip(new.chunks_exact(pw));
    let top = pairs().position(|(a, b)| a != b)?;
    let below = pairs().rev().position(|(a, b)| a != b).unwrap_or(0);
    Some(top..new.len() / pw - below)
}

/// Rows of the canvas through the palette (with its pictures), or its coverage, into `out`.
fn first_level(
    ct: &CanvasTexture,
    lut: &[u32; 256],
    see_lut: &[u32; 256],
    rows: std::ops::Range<usize>,
    out: &mut [u8],
) {
    let pw = ct.canvas.pw();
    let first = rows.start * pw;
    let px = &ct.canvas.px[first..rows.end * pw];
    let own_see;
    let (lut, see_lut) = match &ct.colours {
        Some(own) => {
            own_see = {
                let mut l = **own;
                l[TRANSPARENT as usize] = 0;
                l
            };
            (&**own, &own_see)
        }
        None => (lut, see_lut),
    };
    if ct.coverage {
        for (o, &c) in out.iter_mut().zip(px) {
            *o = if c == TRANSPARENT { 0 } else { 255 };
        }
        return;
    }
    let texels = out.as_chunks_mut::<4>().0.iter_mut().zip(px);
    match (&ct.mask, ct.transparent) {
        (Some(mask), true) => {
            for (i, (o, &c)) in texels.enumerate() {
                let l = if mask[first + i] { see_lut } else { lut };
                *o = l[c as usize].to_le_bytes();
            }
        }
        (_, t) => {
            let l = if t { see_lut } else { lut };
            for (o, &c) in texels {
                *o = l[c as usize].to_le_bytes();
            }
        }
    }
    for (at, pic) in &ct.pictures {
        let (x0, y0) = (at.x as usize, at.y as usize);
        let w = pic.w.min(pw.saturating_sub(x0));
        for y in 0..pic.h {
            let ty = y0 + y;
            if rows.contains(&ty) {
                let o = ((ty - rows.start) * pw + x0) * 4;
                out[o..o + w * 4].copy_from_slice(&pic.rgba[y * pic.w * 4..][..w * 4]);
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The band of rows to upload runs from the first changed row to the last.
    #[test]
    fn changed_rows_band() {
        let old = vec![0u8; 4 * 6];
        let mut new = old.clone();
        assert_eq!(changed_rows(&old, &new, 4), None);
        new[4 + 2] = 1;
        assert_eq!(changed_rows(&old, &new, 4), Some(1..2));
        new[4 * 4] = 1;
        assert_eq!(changed_rows(&old, &new, 4), Some(1..5));
        new[4 * 5 + 3] = 1;
        assert_eq!(changed_rows(&old, &new, 4), Some(1..6));
    }
}
