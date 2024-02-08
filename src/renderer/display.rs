use std::ops::Deref;

use super::texture::{Texture, TextureBuilder};
use crate::{geom::Point, renderer::RenderTarget};

use glam::{vec3, Mat4, Quat, Vec2};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug, Clone, Copy)]
pub enum DisplayMode {
    Stretch,
    Centered,
}

impl DisplayMode {
    pub fn scaling_matrix(self, actual: Vec2, target: Vec2) -> Mat4 {
        match self {
            DisplayMode::Stretch => Mat4::from_scale(target.extend(1.0)),
            DisplayMode::Centered => {
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

pub struct Display {
    config: wgpu::SurfaceConfiguration,
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    depth_texture: Texture,
    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: Window,
}

impl Display {
    pub async fn from_window(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            flags: wgpu::InstanceFlags::from_build_config(),
            dx12_shader_compiler: Default::default(),
            gles_minor_version: wgpu::Gles3MinorVersion::Automatic,
        });

        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
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

    pub fn depth_stencil_attachment(
        &self,
        depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        Some(wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_texture.view,
            depth_ops: Some(wgpu::Operations {
                load: depth_load_op,
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: stencil_ops.into(),
        })
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
}

pub struct DisplayView<'a> {
    display: &'a Display,
    output_texture: wgpu::SurfaceTexture,
    view: wgpu::TextureView,
}

impl DisplayView<'_> {
    pub fn present(self) {
        self.output_texture.present()
    }
}

impl<'a> Deref for DisplayView<'a> {
    type Target = Display;

    fn deref(&self) -> &Self::Target {
        self.display
    }
}

impl<'a> RenderTarget for DisplayView<'a> {
    fn size_pixels(&self) -> Point<u32> {
        self.display.size_pixels()
    }

    fn color_attachment(
        &self,
        load_op: wgpu::LoadOp<wgpu::Color>,
    ) -> wgpu::RenderPassColorAttachment {
        wgpu::RenderPassColorAttachment {
            view: &self.view,
            resolve_target: None,
            ops: wgpu::Operations {
                load: load_op,
                store: wgpu::StoreOp::Store,
            },
        }
    }

    fn depth_stencil_attachment(
        &self,
        depth_load_op: wgpu::LoadOp<f32>,
        stencil_ops: impl Into<Option<wgpu::Operations<u32>>>,
    ) -> Option<wgpu::RenderPassDepthStencilAttachment> {
        self.display
            .depth_stencil_attachment(depth_load_op, stencil_ops)
    }
}
