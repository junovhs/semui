//! WGPU reference target — geometry + box rasterization (`RET-04`, `RET-05`).
//!
//! This is the non-browser reference renderer required by `DEC-04`. It stands up
//! a headless GPU device (a real adapter when present, otherwise a software
//! Vulkan/GL fallback), implements the [`TargetEmitter`] boundary from `RET-01`,
//! and renders a scene into an offscreen texture read back as exact RGBA bytes.
//!
//! `RET-04` rendered only the root background clear. `RET-05` adds the geometry
//! pass ([`crate::target::geometry`]) and a box rasterizer: every node whose
//! border box resolves is painted in pre-order (painter's order, so children
//! land over parents) with its background fill, a uniform solid border, and a
//! uniform `border-radius`, via a signed-distance rounded-rect shader.
//!
//! The supported capabilities are layout (block, flex, absolute) plus the box
//! paint families (background, border, radius). Text [`Capability::Typography`]
//! and [`Capability::ButtonControl`] remain explicit
//! [declared loss](TargetEmission::declared_loss) until later children land; a
//! box whose size is content-driven (needs text measurement) does not resolve
//! and is left unpainted rather than guessed.

use std::error::Error;
use std::fmt;

use wgpu::util::DeviceExt;

use crate::ir::SceneIr;
use crate::target::geometry::{BoxRect, canvas_extent, resolve_geometry};
use crate::target::{
    Capability, TargetCapabilities, TargetEmission, TargetEmitter, capability_gaps, preorder,
};

/// WGSL for the rounded-rect box rasterizer. The vertex stage expands a per-box
/// uniform rectangle into a screen quad; the fragment stage evaluates a rounded
/// rectangle signed-distance field to fill the interior, stroke a uniform
/// border, and discard outside the (possibly rounded) edge so the background
/// shows through.
const BOX_SHADER: &str = r#"
struct BoxUniform {
    rect: vec4<f32>,    // x, y, w, h in pixels, top-left origin
    fill: vec4<f32>,    // rgba 0..1; alpha 0 means no fill
    border: vec4<f32>,  // rgba 0..1
    params: vec4<f32>,  // radius, border_width, canvas_w, canvas_h
};
@group(0) @binding(0) var<uniform> u: BoxUniform;

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> @builtin(position) vec4<f32> {
    var corners = array<vec2<f32>, 6>(
        vec2<f32>(0.0, 0.0), vec2<f32>(1.0, 0.0), vec2<f32>(0.0, 1.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(1.0, 0.0), vec2<f32>(1.0, 1.0),
    );
    let px = u.rect.xy + corners[vi] * u.rect.zw;
    let ndc = vec2<f32>(px.x / u.params.z * 2.0 - 1.0, 1.0 - px.y / u.params.w * 2.0);
    return vec4<f32>(ndc, 0.0, 1.0);
}

fn rounded_sdf(p: vec2<f32>, half: vec2<f32>, r: f32) -> f32 {
    let q = abs(p) - (half - vec2<f32>(r, r));
    return length(max(q, vec2<f32>(0.0, 0.0))) + min(max(q.x, q.y), 0.0) - r;
}

@fragment
fn fs_main(@builtin(position) frag: vec4<f32>) -> @location(0) vec4<f32> {
    let half = u.rect.zw * 0.5;
    let center = u.rect.xy + half;
    let r = clamp(u.params.x, 0.0, min(half.x, half.y));
    let d = rounded_sdf(frag.xy - center, half, r);
    if (d > 0.0) {
        discard;
    }
    let border_width = u.params.y;
    if (border_width > 0.0 && d > -border_width) {
        return u.border;
    }
    if (u.fill.a <= 0.0) {
        discard;
    }
    return u.fill;
}
"#;

/// Edge length of the square offscreen canvas the scaffold renders into.
///
/// Chosen so `width * 4` is a multiple of [`wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`]
/// (256), but readback below still de-pads generically.
const CANVAS: u32 = 64;

/// A rendered RGBA frame read back from the GPU: tightly packed, row-major,
/// `width * height * 4` bytes with no row padding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedFrame {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

impl RenderedFrame {
    /// The RGBA pixel at `(x, y)`.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let base = ((y * self.width + x) * 4) as usize;
        [
            self.rgba[base],
            self.rgba[base + 1],
            self.rgba[base + 2],
            self.rgba[base + 3],
        ]
    }
}

/// Failure initializing the headless GPU device.
#[derive(Debug)]
pub enum WgpuTargetError {
    /// No GPU adapter (hardware or software) could be acquired.
    NoAdapter,
    /// An adapter was found but a device could not be requested.
    RequestDevice(wgpu::RequestDeviceError),
}

impl fmt::Display for WgpuTargetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WgpuTargetError::NoAdapter => {
                write!(f, "no GPU adapter available (hardware or software)")
            }
            WgpuTargetError::RequestDevice(err) => write!(f, "could not request device: {err}"),
        }
    }
}

impl Error for WgpuTargetError {}

/// The headless WGPU reference target.
pub struct WgpuTarget {
    device: wgpu::Device,
    queue: wgpu::Queue,
    backend: wgpu::Backend,
    adapter_name: String,
    box_pipeline: wgpu::RenderPipeline,
    box_bind_layout: wgpu::BindGroupLayout,
}

impl WgpuTarget {
    /// Initialize a headless device, preferring a real adapter and falling back
    /// to a software backend. Blocks on GPU initialization.
    pub fn new() -> Result<Self, WgpuTargetError> {
        pollster::block_on(Self::new_async())
    }

    async fn new_async() -> Result<Self, WgpuTargetError> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or(WgpuTargetError::NoAdapter)?;
        let info = adapter.get_info();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("semui-wgpu-target"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::default(),
                },
                None,
            )
            .await
            .map_err(WgpuTargetError::RequestDevice)?;
        let (box_pipeline, box_bind_layout) = build_box_pipeline(&device);
        Ok(Self {
            device,
            queue,
            backend: info.backend,
            adapter_name: info.name,
            box_pipeline,
            box_bind_layout,
        })
    }

    /// The graphics backend in use, e.g. `"Vulkan"`. Useful for run logs.
    pub fn backend(&self) -> &'static str {
        match self.backend {
            wgpu::Backend::Vulkan => "Vulkan",
            wgpu::Backend::Gl => "GL",
            wgpu::Backend::Metal => "Metal",
            wgpu::Backend::Dx12 => "DX12",
            wgpu::Backend::BrowserWebGpu => "WebGPU",
            wgpu::Backend::Empty => "Empty",
        }
    }

    /// The selected adapter's reported name, e.g. `"llvmpipe"` or a GPU model.
    pub fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    /// Render `scene` into the offscreen canvas and read it back as exact RGBA.
    ///
    /// The canvas is sized to the bounding extent of the resolved geometry (or a
    /// default square when nothing resolves, e.g. a background-only scene). The
    /// page is cleared to the root background, then every node with a resolvable
    /// border box and visible paint is drawn in pre-order. Deterministic: fixed
    /// geometry and a fixed pipeline yield identical bytes across runs.
    fn render(&self, scene: &SceneIr) -> RenderedFrame {
        let boxes = resolve_geometry(scene);
        let (width, height) = canvas_extent(&boxes).unwrap_or((CANVAS, CANVAS));
        let draws = self.box_draws(scene, &boxes, width, height);

        let clear = root_clear_color(scene);
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };
        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("semui-frame"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Non-sRGB so the clear color round-trips to exact bytes.
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("semui-encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("semui-boxes"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if !draws.is_empty() {
                pass.set_pipeline(&self.box_pipeline);
                for bind_group in &draws {
                    pass.set_bind_group(0, bind_group, &[]);
                    pass.draw(0..6, 0..1);
                }
            }
        }

        let rgba = self.read_back(&mut encoder, &texture, size);
        RenderedFrame {
            width,
            height,
            rgba,
        }
    }

    /// Build a per-box bind group (a uniform buffer of geometry + paint) for
    /// every node with a resolved border box and visible paint, in pre-order so
    /// the draw order is painter's order.
    fn box_draws(
        &self,
        scene: &SceneIr,
        boxes: &std::collections::BTreeMap<String, BoxRect>,
        canvas_w: u32,
        canvas_h: u32,
    ) -> Vec<wgpu::BindGroup> {
        let mut draws = Vec::new();
        for node in preorder(scene) {
            let Some(rect) = boxes.get(&node.id) else {
                continue;
            };
            let fill = node
                .paint
                .background_color
                .as_ref()
                .and_then(|c| parse_hex(&c.0));
            let border = node.paint.border.as_ref();
            let border_rgb = border.and_then(|b| parse_hex(&b.color.0));
            // Nothing visible to paint: skip (e.g. an invisible text-holder box).
            if fill.is_none() && border_rgb.is_none() {
                continue;
            }
            let uniform = box_uniform_bytes(
                *rect,
                fill,
                border_rgb,
                border.map(|b| b.width).unwrap_or(0.0),
                node.paint.border_radius.unwrap_or(0.0),
                canvas_w,
                canvas_h,
            );
            let buffer = self
                .device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("semui-box-uniform"),
                    contents: &uniform,
                    usage: wgpu::BufferUsages::UNIFORM,
                });
            let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("semui-box-bind"),
                layout: &self.box_bind_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }],
            });
            draws.push(bind_group);
        }
        draws
    }

    /// Copy `texture` to a mappable buffer, submit, and read tightly packed RGBA.
    fn read_back(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        texture: &wgpu::Texture,
        size: wgpu::Extent3d,
    ) -> Vec<u8> {
        let unpadded = size.width * 4;
        let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded = unpadded.div_ceil(align) * align;

        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("semui-readback"),
            size: (padded * size.height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded),
                    rows_per_image: Some(size.height),
                },
            },
            size,
        );

        // The encoder is consumed by replacing it with a fresh, empty one so the
        // caller's `&mut` stays valid after submit.
        let finished = std::mem::replace(
            encoder,
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None }),
        );
        self.queue.submit([finished.finish()]);

        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        self.device.poll(wgpu::Maintain::Wait);

        let mapped = slice.get_mapped_range();
        let mut rgba = Vec::with_capacity((unpadded * size.height) as usize);
        for row in 0..size.height {
            let start = (row * padded) as usize;
            rgba.extend_from_slice(&mapped[start..start + unpadded as usize]);
        }
        drop(mapped);
        buffer.unmap();
        rgba
    }
}

impl TargetEmitter for WgpuTarget {
    type Artifact = RenderedFrame;

    fn target_id(&self) -> &'static str {
        "wgpu"
    }

    fn capabilities(&self) -> TargetCapabilities {
        // Layout and box paint are rendered; text and native controls remain
        // declared loss until later RET-02 children implement them.
        TargetCapabilities::from_capabilities([
            Capability::BlockLayout,
            Capability::FlexLayout,
            Capability::AbsolutePositioning,
            Capability::Background,
            Capability::Border,
            Capability::BorderRadius,
        ])
    }

    fn emit(&self, scene: &SceneIr) -> TargetEmission<RenderedFrame> {
        TargetEmission {
            artifact: self.render(scene),
            declared_loss: capability_gaps(scene, &self.capabilities()),
        }
    }
}

/// Build the rounded-rect box pipeline and its single-uniform bind group layout.
fn build_box_pipeline(device: &wgpu::Device) -> (wgpu::RenderPipeline, wgpu::BindGroupLayout) {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("semui-box-shader"),
        source: wgpu::ShaderSource::Wgsl(BOX_SHADER.into()),
    });
    let bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("semui-box-bind-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    });
    let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("semui-box-pipeline-layout"),
        bind_group_layouts: &[&bind_layout],
        push_constant_ranges: &[],
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("semui-box-pipeline"),
        layout: Some(&layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                // Opaque writes; the shader discards rather than blends, so bytes
                // stay exact and deterministic.
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });
    (pipeline, bind_layout)
}

/// Pack one box's geometry and paint into the 64-byte uniform the shader reads.
/// Colors are normalized to `0..1`; a missing fill is alpha 0 (the shader
/// discards the interior), a missing border is width 0.
fn box_uniform_bytes(
    rect: BoxRect,
    fill: Option<[u8; 3]>,
    border: Option<[u8; 3]>,
    border_width: f32,
    radius: f32,
    canvas_w: u32,
    canvas_h: u32,
) -> [u8; 64] {
    let fill_rgba = match fill {
        Some([r, g, b]) => [norm(r), norm(g), norm(b), 1.0],
        None => [0.0, 0.0, 0.0, 0.0],
    };
    let border_rgba = match border {
        Some([r, g, b]) => [norm(r), norm(g), norm(b), 1.0],
        None => [0.0, 0.0, 0.0, 0.0],
    };
    let floats: [f32; 16] = [
        rect.x,
        rect.y,
        rect.width,
        rect.height,
        fill_rgba[0],
        fill_rgba[1],
        fill_rgba[2],
        fill_rgba[3],
        border_rgba[0],
        border_rgba[1],
        border_rgba[2],
        border_rgba[3],
        radius,
        border_width,
        canvas_w as f32,
        canvas_h as f32,
    ];
    let mut bytes = [0u8; 64];
    for (i, f) in floats.iter().enumerate() {
        bytes[i * 4..i * 4 + 4].copy_from_slice(&f.to_le_bytes());
    }
    bytes
}

/// A single color channel byte normalized to the `0.0..=1.0` shader range.
fn norm(channel: u8) -> f32 {
    channel as f32 / 255.0
}

/// The clear color for a scene: the root node's background, or opaque white when
/// the root declares none (the canonical page-frame normalization is white).
fn root_clear_color(scene: &SceneIr) -> wgpu::Color {
    let root_bg = scene
        .nodes
        .iter()
        .find(|node| node.parent_id.is_none())
        .and_then(|node| node.paint.background_color.as_ref());
    match root_bg.and_then(|color| parse_hex(&color.0)) {
        Some([r, g, b]) => wgpu::Color {
            r: r as f64 / 255.0,
            g: g as f64 / 255.0,
            b: b as f64 / 255.0,
            a: 1.0,
        },
        None => wgpu::Color::WHITE,
    }
}

/// Parse a `#rrggbb` hex color into RGB bytes. Returns `None` on any malformed
/// input.
fn parse_hex(hex: &str) -> Option<[u8; 3]> {
    let digits = hex.strip_prefix('#')?;
    if digits.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&digits[0..2], 16).ok()?;
    let g = u8::from_str_radix(&digits[2..4], 16).ok()?;
    let b = u8::from_str_radix(&digits[4..6], 16).ok()?;
    Some([r, g, b])
}

#[cfg(test)]
mod tests;
