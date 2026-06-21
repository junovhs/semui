//! WGPU reference target — headless scaffold (`RET-04`).
//!
//! This is the first slice of the non-browser reference renderer required by
//! `DEC-04`. It stands up a headless GPU device (a real adapter when present,
//! otherwise a software Vulkan/GL fallback), implements the [`TargetEmitter`]
//! boundary from `RET-01`, and renders a scene's root background into an
//! offscreen texture that is read back as exact RGBA bytes.
//!
//! Only [`Capability::Background`] is rendered here; every other capability a
//! scene needs is reported as explicit [declared loss](TargetEmission::declared_loss)
//! rather than silently dropped. Later children (`RET-05`, `RET-06`) raise the
//! supported set as geometry, primitives, and typography land.

use std::error::Error;
use std::fmt;

use crate::ir::SceneIr;
use crate::target::{
    Capability, TargetCapabilities, TargetEmission, TargetEmitter, capability_gaps,
};

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
        Ok(Self {
            device,
            queue,
            backend: info.backend,
            adapter_name: info.name,
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

    /// Render the scene's root background into the offscreen canvas and read it
    /// back as exact RGBA. Deterministic: a clear to a fixed color.
    fn render_background(&self, scene: &SceneIr) -> RenderedFrame {
        let clear = root_clear_color(scene);
        let size = wgpu::Extent3d {
            width: CANVAS,
            height: CANVAS,
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
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("semui-clear"),
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
        }

        let rgba = self.read_back(&mut encoder, &texture, size);
        RenderedFrame {
            width: CANVAS,
            height: CANVAS,
            rgba,
        }
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
        // The scaffold renders only the background clear; everything else is
        // declared loss until later RET-02 children implement it.
        TargetCapabilities::from_capabilities([Capability::Background])
    }

    fn emit(&self, scene: &SceneIr) -> TargetEmission<RenderedFrame> {
        TargetEmission {
            artifact: self.render_background(scene),
            declared_loss: capability_gaps(scene, &self.capabilities()),
        }
    }
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
