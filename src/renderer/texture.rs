use crate::geom::Point;
use crate::renderer::Bindable;
use image::{EncodableLayout, RgbaImage};

slotmap::new_key_type! {
    pub struct TextureRef;
}

#[derive(Clone, Default)]
pub struct TextureBuilder<'a> {
    label: Option<&'a str>,
    format: Option<wgpu::TextureFormat>,
    address_mode: Option<wgpu::AddressMode>,
    min_filter: Option<wgpu::FilterMode>,
    mag_filter: Option<wgpu::FilterMode>,
    mipmap_filter: Option<wgpu::FilterMode>,
    compare_func: Option<wgpu::CompareFunction>,
    usage: Option<wgpu::TextureUsages>,
    // TODO: more
}

impl<'a> TextureBuilder<'a> {
    pub const DEFAULT_ADDRESS_MODE: wgpu::AddressMode = wgpu::AddressMode::ClampToEdge;
    pub const DEFAULT_FILTER_MODE: wgpu::FilterMode = wgpu::FilterMode::Nearest;
    pub const DEFAULT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
    pub const DEFAULT_RENDER_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8UnormSrgb;
    pub const DEFAULT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn labeled(label: &'a str) -> Self {
        Self {
            label: Some(label),
            ..Default::default()
        }
    }

    pub fn depth() -> Self {
        Self::default().with_format(Self::DEFAULT_DEPTH_FORMAT)
    }

    pub fn render_target() -> Self {
        Self::default().with_format(Self::DEFAULT_RENDER_FORMAT)
    }

    pub fn with_label(self, label: &'a str) -> Self {
        let label = Some(label);
        Self { label, ..self }
    }

    pub fn with_format(self, format: wgpu::TextureFormat) -> Self {
        let format = Some(format);
        Self { format, ..self }
    }

    pub fn with_usage(self, usage: wgpu::TextureUsages) -> Self {
        let usage = Some(usage);
        Self { usage, ..self }
    }

    pub fn with_address_mode(self, address_mode: wgpu::AddressMode) -> Self {
        let address_mode = Some(address_mode);
        Self {
            address_mode,
            ..self
        }
    }

    pub fn with_filter_mode(self, filter_mode: wgpu::FilterMode) -> Self {
        let filter_mode = Some(filter_mode);
        Self {
            min_filter: filter_mode,
            mag_filter: filter_mode,
            mipmap_filter: filter_mode,
            ..self
        }
    }

    pub fn from_raw_bytes(
        mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        size: Point<u32>,
    ) -> Texture {
        if let Some(usage) = self.usage {
            if !usage.contains(wgpu::TextureUsages::COPY_DST) {
                panic!(
                    "Given usage {:?} requires COPY_DST to be initialized.",
                    self.usage
                );
            }
        } else {
            self.usage = Some(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST);
        }
        let texture = self.build(device, size);
        queue.write_texture(
            wgpu::ImageCopyTexture {
                aspect: wgpu::TextureAspect::All,
                texture: &texture.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * size.x), // assume 32-bit pixels
                rows_per_image: Some(size.y),
            },
            texture.texture.size(),
        );
        texture
    }

    pub fn from_image(
        self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        // TODO: make this generic over image types that can convert to Rgba8
        image: &RgbaImage,
    ) -> Texture {
        let (width, height) = image.dimensions();
        self.from_raw_bytes(device, queue, image.as_bytes(), Point::new(width, height))
    }

    pub fn build(mut self, device: &wgpu::Device, size: Point<u32>) -> Texture {
        let format = self.format.unwrap_or(Self::DEFAULT_FORMAT);
        let mut view_formats = vec![format];
        let usage = self.usage.unwrap_or(
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        if format.has_depth_aspect() {
            if !usage.contains(wgpu::TextureUsages::RENDER_ATTACHMENT) {
                panic!("Creating a depth texture without render attachment usage, is that really your intention?");
            }
            self.mag_filter.get_or_insert(wgpu::FilterMode::Linear);
            self.min_filter.get_or_insert(wgpu::FilterMode::Linear);
            self.mipmap_filter.get_or_insert(wgpu::FilterMode::Nearest);
            self.compare_func
                .get_or_insert(wgpu::CompareFunction::LessEqual);
        } else {
            let other = if format.is_srgb() {
                format.remove_srgb_suffix()
            } else {
                format.add_srgb_suffix()
            };
            if format != other {
                view_formats.push(other);
            }
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: self.label,
            size: wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage,
            view_formats: &view_formats,
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let address_mode = self.address_mode.unwrap_or(Self::DEFAULT_ADDRESS_MODE);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: address_mode,
            address_mode_v: address_mode,
            address_mode_w: address_mode,
            mag_filter: self.mag_filter.unwrap_or(Self::DEFAULT_FILTER_MODE),
            min_filter: self.min_filter.unwrap_or(Self::DEFAULT_FILTER_MODE),
            mipmap_filter: self.mipmap_filter.unwrap_or(Self::DEFAULT_FILTER_MODE),
            compare: self.compare_func,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            ..Default::default()
        });
        Texture {
            texture,
            view,
            sampler,
        }
    }
}

#[derive(Debug)]
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn size_pixels(&self) -> Point<u32> {
        let fb_size = self.texture.size();
        Point::new(fb_size.width as _, fb_size.height as _)
    }
}

impl Bindable for Texture {
    fn bind_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
            label: Some("texture_bind_group"),
        })
    }
}
