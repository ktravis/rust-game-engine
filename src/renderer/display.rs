use std::{ops::Deref, sync::Arc};

use super::texture::{Texture, TextureBuilder};
use crate::geom::Point;

use glam::{vec3, Mat4, Quat, Vec2};
use image::ImageResult;
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug, Clone, Copy)]
pub enum ScalingMode {
    Stretch,
    Centered,
}

impl ScalingMode {
    pub fn view_matrix(self, actual: Vec2, target: Vec2) -> Mat4 {
        match self {
            ScalingMode::Stretch => Mat4::from_scale(target.extend(1.0)),
            ScalingMode::Centered => {
                let scale = (target.x / actual.x).min(target.y / actual.y);

                let scaled_target_size = scale * actual;
                Mat4::from_scale_rotation_translation(
                    scaled_target_size.extend(1.0),
                    Quat::IDENTITY,
                    vec3(
                        (target.x - scaled_target_size.x) / 2.0,
                        (target.y - scaled_target_size.y) / 2.0,
                        0.0,
                    ),
                )
            }
        }
    }
}

struct BufferUnmapper<'a>(&'a wgpu::Buffer);

impl Drop for BufferUnmapper<'_> {
    fn drop(&mut self) {
        self.0.unmap()
    }
}

pub struct MappedBufferView<'a> {
    buffer_view: wgpu::BufferView<'a>,
    _buffer: BufferUnmapper<'a>,
}

impl<'a> Deref for MappedBufferView<'a> {
    type Target = wgpu::BufferView<'a>;

    fn deref(&self) -> &Self::Target {
        &self.buffer_view
    }
}

pub struct Display {
    config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    depth_texture: Texture,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Arc<Window>,

    staging_buffer: Option<wgpu::Buffer>,
}

impl Display {
    pub async fn from_window(window: Window) -> Self {
        let size = window.inner_size();
        let window = Arc::new(window);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::from_build_config(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = instance.create_surface(window.clone()).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::POLYGON_MODE_LINE
                        | wgpu::Features::CLEAR_TEXTURE
                        | wgpu::Features::ADDRESS_MODE_CLAMP_TO_BORDER,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits {
                            max_bind_groups: 6,
                            ..wgpu::Limits::default()
                        }
                    },
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);
        let depth_texture = TextureBuilder::depth()
            .with_label("display_depth_texture")
            .build(&device, Point::new(size.width, size.height));
        Self {
            config,
            surface,
            device,
            queue,
            depth_texture,
            window,
            staging_buffer: None,
        }
    }

    pub fn reconfigure(&mut self) {
        self.resize(PhysicalSize {
            width: self.config.width,
            height: self.config.height,
        });
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.depth_texture = TextureBuilder::depth()
                .with_label("display_depth_texture")
                .build(self.device(), Point::new(new_size.width, new_size.height));
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    fn output_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    pub fn size_pixels(&self) -> Point<u32> {
        Point {
            x: self.config.width,
            y: self.config.height,
        }
    }

    pub fn depth_texture(&self) -> &Texture {
        &self.depth_texture
    }

    pub fn view(&self) -> Result<DisplayView, wgpu::SurfaceError> {
        let output_texture = self.output_texture()?;
        let view = output_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        Ok(DisplayView {
            display: self,
            output_texture,
            view,
        })
    }

    pub fn command_encoder(&self) -> wgpu::CommandEncoder {
        self.device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            })
    }

    pub fn depth_format(&self) -> wgpu::TextureFormat {
        TextureBuilder::DEFAULT_DEPTH_FORMAT
    }

    pub fn read_texture_data<'a>(&'a mut self, texture: &Texture) -> MappedBufferView<'a> {
        let dim = texture.size_pixels();
        let block_size = texture.format().block_copy_size(None).unwrap();
        let bytes_per_row = dim.x * block_size;
        let total_size = (dim.y * bytes_per_row) as u64;

        if let Some(buf) = &self.staging_buffer {
            if buf.size() < total_size {
                self.staging_buffer.take();
            }
        }
        let buffer = self.staging_buffer.get_or_insert_with(|| {
            self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("display staging buffer"),
                size: total_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        let mut enc = self.device.create_command_encoder(&Default::default());
        enc.copy_texture_to_buffer(
            texture.texture.as_image_copy(),
            wgpu::ImageCopyBuffer {
                buffer,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(dim.y),
                },
            },
            texture.texture.size(),
        );
        self.queue.submit([enc.finish()]);
        let slice = buffer.slice(..);
        slice.map_async(wgpu::MapMode::Read, |res| res.expect("buffer map failed"));
        self.device.poll(wgpu::Maintain::wait()).panic_on_timeout();
        MappedBufferView {
            _buffer: BufferUnmapper(buffer),
            buffer_view: slice.get_mapped_range(),
        }
    }

    pub fn load_texture_bytes(
        &self,
        buffer: &[u8],
        texture_builder: TextureBuilder,
    ) -> ImageResult<Texture> {
        Ok(texture_builder.from_image(
            &self.device,
            &self.queue,
            &image::load_from_memory(buffer)?.into_rgba8(),
        ))
    }
}

pub struct DisplayView<'a> {
    display: &'a Display,
    output_texture: wgpu::SurfaceTexture,
    pub(super) view: wgpu::TextureView,
}

impl DisplayView<'_> {
    pub fn present(self) {
        self.output_texture.present()
    }

    pub fn display(&self) -> &Display {
        self.display
    }

    pub fn orthographic_projection(&self) -> Mat4 {
        let target_size = self.size_pixels().as_vec2();
        Mat4::orthographic_rh(0.0, target_size.x, 0.0, target_size.y, 0.0, 1.0)
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }
}

impl<'a> Deref for DisplayView<'a> {
    type Target = Display;

    fn deref(&self) -> &Self::Target {
        self.display
    }
}
