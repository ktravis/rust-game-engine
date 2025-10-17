use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut, Range},
    sync::{Arc, Mutex},
};

use bytemuck::Zeroable;
use glam::{Mat4, Quat, Vec4};
use slotmap::SlotMap;
use wgpu::BindGroupLayout;

use super::{
    display::Display,
    instance::{InstanceRenderData, InstanceStorage},
    mesh::{LoadMesh, Mesh, RawMeshRef, UntypedMesh},
    shader_type::GlobalUniforms,
    shaders,
    text::{RenderableFont, TextDisplayOptions},
    texture::{Texture, TextureBuilder},
    BasicInstanceData, BindGroup, Bindable, InstanceData, MeshRef, OffscreenFramebuffer,
    PipelineBuilder, PipelineRef, RawPipelineRef, RenderTarget, TextureRef, UniformBindGroup,
    UniformBuffer, UniformData, DEFAULT_TEXTURE_DATA,
};
use crate::{
    camera::Camera,
    color::Color,
    geom::{BasicVertexData, Point, Rect, VertexData},
    transform::{Transform, Transform2D},
};

pub type BoundTexture = BindGroup<Texture>;

pub type ViewProjectionUniforms = shaders::global::types::ViewProjectionUniforms;

impl ViewProjectionUniforms {
    pub fn for_camera(camera: &Camera) -> Self {
        let view = camera.view_matrix();
        assert!(
            (view.inverse() * view * Vec4::ONE - Vec4::ONE)
                .abs()
                .length_squared()
                < 0.000001
        );
        Self {
            view,
            inverse_view: view.inverse(),
            projection: camera.perspective_matrix(),
            camera_pos: camera.position(),
            ..Default::default()
        }
    }
}

impl Default for ViewProjectionUniforms {
    fn default() -> Self {
        Self {
            view: Default::default(),
            projection: Default::default(),
            camera_pos: Default::default(),
            inverse_view: Default::default(),
            ..Zeroable::zeroed()
        }
    }
}

pub struct CachePool<T> {
    items: Vec<Arc<T>>,
    in_use: usize,
}

impl<T> Default for CachePool<T> {
    fn default() -> Self {
        Self {
            items: vec![],
            in_use: 0,
        }
    }
}

impl<T> CachePool<T> {
    pub fn get<'a>(&'a mut self, ctor: impl FnOnce() -> T) -> &'a Arc<T> {
        if self.in_use >= self.items.len() {
            self.items.push(Arc::new(ctor()));
        }
        let i = self.in_use;
        self.in_use += 1;
        &self.items[i]
    }

    pub fn reset(&mut self) {
        self.in_use = 0;
    }
}

// TODO: I think we could make this hold bind groups + buffers for multiple types by making an
// UntypedUniformBindGroup and keeping it in a map based on the size?

pub struct BindGroupAllocator<'a, U: UniformData> {
    display: &'a Display,
    layout: &'a wgpu::BindGroupLayout,
    bind_groups: &'a Mutex<CachePool<BindGroup<UniformBuffer<U>>>>,
}

impl<'a, U: UniformData + Default> BindGroupAllocator<'a, U> {
    pub fn get(&self, uniform: &U) -> Arc<wgpu::BindGroup> {
        let display = self.display;
        let mut x = self.bind_groups.lock().unwrap();
        let bg = x.get(|| {
            BindGroup::new(
                display.device(),
                &self.layout,
                UniformBuffer::<U>::new(display.device(), Default::default()),
            )
        });
        display
            .queue()
            .write_buffer(bg.buffer(), 0, bytemuck::bytes_of(&uniform.raw()));
        bg.bind_group().clone()
    }
}

pub struct PartialRenderPass<'a> {
    display: &'a Display,
    encoder: wgpu::CommandEncoder,
}

impl PartialRenderPass<'_> {
    pub fn encoder(self) -> wgpu::CommandEncoder {
        self.encoder
    }

    pub fn command_buffer(self) -> wgpu::CommandBuffer {
        self.encoder.finish()
    }

    pub fn submit(self) {
        self.display.queue().submit([self.command_buffer()]);
    }
}

pub trait BindingSlot {
    fn slot(&self) -> u32;
    fn value(&self) -> &Arc<wgpu::BindGroup>;
}

pub trait Bindings {
    fn types() -> Vec<BindingType>;
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum BindingType {
    Uniform,
    Texture { format: wgpu::TextureFormat },
    Direct(wgpu::ShaderStages, wgpu::BindingType),
}

impl BindingType {
    pub fn create_layout(&self, device: &wgpu::Device, name: &str) -> wgpu::BindGroupLayout {
        let entries = &match *self {
            BindingType::Uniform => vec![wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
            BindingType::Texture { format } => {
                let sample_type = format
                    .sample_type(Some(wgpu::TextureAspect::All), None)
                    .expect(&format!(
                        "non-sampleable texture format {:?} used in binding",
                        format
                    ));
                vec![
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(if format.has_depth_aspect() {
                            wgpu::SamplerBindingType::Comparison
                        } else {
                            match sample_type {
                                wgpu::TextureSampleType::Float { filterable: false } => {
                                    wgpu::SamplerBindingType::NonFiltering
                                }
                                _ => wgpu::SamplerBindingType::Filtering,
                            }
                        }),
                        count: None,
                    },
                ]
            }
            BindingType::Direct(visibility, ty) => vec![wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility,
                ty,
                count: None,
            }],
        };
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries,
            label: Some(name),
        })
    }
}

pub struct RenderState {
    pub global_uniforms: UniformBindGroup<GlobalUniforms>,
    quad_mesh: MeshRef<BasicVertexData>,

    bind_group_layouts: HashMap<BindingType, Arc<wgpu::BindGroupLayout>>,

    instance_storage: InstanceStorage,
    view_proj_bind_groups: Mutex<CachePool<BindGroup<UniformBuffer<ViewProjectionUniforms>>>>,

    texture_manager: SlotMap<TextureRef, BoundTexture>,
    default_texture: TextureRef,

    mesh_manager: SlotMap<RawMeshRef, UntypedMesh>,
    // pub(crate) pipeline_cache: wgpu::PipelineCache,
    pipelines: SlotMap<RawPipelineRef, wgpu::RenderPipeline>,
    default_pipeline: PipelineRef<BasicVertexData, BasicInstanceData>,
    text_pipeline: PipelineRef<BasicVertexData, BasicInstanceData>,
}

impl RenderState {
    const MAX_COLOR_ATTACHMENTS: usize = 8;

    pub fn new(
        display: &Display,
        default_shader: &wgpu::ShaderModule,
        text_shader: &wgpu::ShaderModule,
    ) -> Self {
        let device = display.device();

        let global_uniforms = UniformBindGroup::new(
            device,
            &BindingType::Uniform.create_layout(device, "global uniform"),
            UniformBuffer::new(device, Zeroable::zeroed()),
        );
        let mesh_manager = SlotMap::with_key();
        let instance_storage = InstanceStorage::new(display, 1024);
        // let pipeline_cache = unsafe {
        //     device.create_pipeline_cache(&wgpu::PipelineCacheDescriptor {
        //         label: Some("general pipeline cache"),
        //         data: None,
        //         fallback: true,
        //     })
        // };

        let mut s = Self {
            bind_group_layouts: HashMap::default(),
            texture_manager: SlotMap::with_key(),
            mesh_manager,
            pipelines: SlotMap::with_key(),
            global_uniforms,
            instance_storage,
            quad_mesh: Default::default(),
            view_proj_bind_groups: Default::default(),
            default_pipeline: Default::default(),
            text_pipeline: Default::default(),
            default_texture: Default::default(),
            // pipeline_cache,
        };

        s.default_texture = s.load_texture(
            display,
            TextureBuilder::render_target()
                .with_label("default_texture")
                .from_raw_bytes(
                    display.device(),
                    display.queue(),
                    &DEFAULT_TEXTURE_DATA,
                    Point::new(2, 2),
                ),
        );
        s.quad_mesh = s.prepare_mesh(display.device().load_quad_mesh());
        s.default_pipeline = s
            .pipeline_builder()
            .with_label("Default Render Pipeline")
            .build(display.device(), &default_shader);
        s.text_pipeline = s
            .pipeline_builder()
            .with_label("Text Render Pipeline")
            .build(display.device(), &text_shader);
        s
    }

    pub fn bind_group_layout(
        &mut self,
        device: &wgpu::Device,
        binding_type: BindingType,
    ) -> Arc<wgpu::BindGroupLayout> {
        self.bind_group_layouts
            .entry(binding_type)
            .or_insert_with(|| Arc::new(binding_type.create_layout(device, "TODO")))
            .clone()
    }

    pub fn create_bind_group<T: Bindable>(
        &mut self,
        device: &wgpu::Device,
        resource: T,
    ) -> BindGroup<T> {
        let layout = self.bind_group_layout(device, resource.binding_type());
        BindGroup::new(device, &layout, resource)
    }

    pub fn create_uniform_bind_group<U: UniformData>(
        &mut self,
        device: &wgpu::Device,
        uniform: U,
    ) -> (UniformBindGroup<U>, Arc<BindGroupLayout>) {
        let layout = self.bind_group_layout(device, BindingType::Uniform);
        let resource = UniformBuffer::new(device, uniform);
        (BindGroup::new(device, &layout, resource), layout)
    }

    pub fn quad_mesh(&self) -> MeshRef<BasicVertexData> {
        self.quad_mesh
    }

    pub fn default_pipeline(&self) -> PipelineRef<BasicVertexData, BasicInstanceData> {
        self.default_pipeline
    }

    pub fn pipeline_builder<'a>(&'a mut self) -> PipelineBuilder<'a> {
        PipelineBuilder::new(self)
    }

    pub(super) fn add_pipeline<V: VertexData, I: InstanceData>(
        &mut self,
        key: impl Into<Option<RawPipelineRef>>,
        pipeline: wgpu::RenderPipeline,
    ) -> PipelineRef<V, I> {
        match key.into() {
            Some(key) => {
                *self.pipelines.get_mut(key).unwrap() = pipeline;
                key.into()
            }
            None => self.pipelines.insert(pipeline).into(),
        }
    }

    #[must_use]
    pub fn render_pass<'a>(
        &mut self,
        display: &'a Display,
        name: &str,
        color_targets: &[RenderTarget],
        depth_target: Option<RenderTarget>,
        view_projection: &ViewProjectionUniforms,
        pass: impl FnOnce(&mut RenderPass<'_, '_>),
    ) -> PartialRenderPass<'a> {
        self.instance_storage.clear();
        if color_targets.len() > Self::MAX_COLOR_ATTACHMENTS {
            panic!(
                "too many color targets ({} > {})",
                color_targets.len(),
                Self::MAX_COLOR_ATTACHMENTS
            );
        }
        let bg = {
            let alloc = BindGroupAllocator {
                display,
                layout: &self.bind_group_layout(display.device(), BindingType::Uniform),
                bind_groups: &self.view_proj_bind_groups,
            };
            alloc.get(view_projection)
        };

        let mut encoder = display.command_encoder();
        {
            let color_attachments: [Option<wgpu::RenderPassColorAttachment>;
                Self::MAX_COLOR_ATTACHMENTS] = std::array::from_fn(|i| {
                color_targets.get(i).map(|target| {
                    let view = match target {
                        RenderTarget::TextureView(view) => *view,
                        RenderTarget::TextureRef(texture) => &self.get_texture(*texture).view,
                    };
                    wgpu::RenderPassColorAttachment {
                        view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    }
                })
            });
            let mut raw_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(name),
                color_attachments: &color_attachments[..color_targets.len()],
                depth_stencil_attachment: depth_target.map(|target| {
                    let view = match target {
                        RenderTarget::TextureView(view) => view,
                        RenderTarget::TextureRef(texture) => &self.get_texture(texture).view,
                    };
                    wgpu::RenderPassDepthStencilAttachment {
                        view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            raw_pass.set_bind_group(
                RenderPass::GLOBAL_UNIFORMS_BIND_GROUP_INDEX,
                self.global_uniforms.bind_group().deref(),
                &[],
            );

            raw_pass.set_bind_group(
                RenderPass::VIEW_PROJECTION_UNIFORMS_BIND_GROUP_INDEX,
                bg.deref(),
                &[],
            );
            let default_texture = self.default_texture;
            let mut render_pass = RenderPass::new(self, display, &mut raw_pass);
            render_pass.bind_texture(default_texture);
            pass(&mut render_pass);
            render_pass.flush_draw_calls();
        }
        PartialRenderPass { display, encoder }
    }

    pub fn after_frame(&mut self) {
        self.view_proj_bind_groups.lock().unwrap().reset();
    }

    pub fn create_offscreen_framebuffer(
        &mut self,
        display: &Display,
        size: Point<u32>,
        format: impl Into<Option<wgpu::TextureFormat>>,
    ) -> OffscreenFramebuffer {
        let format = format
            .into()
            .unwrap_or(TextureBuilder::DEFAULT_RENDER_FORMAT);
        let color = self.load_texture(
            display,
            TextureBuilder::labeled("offscreen_color_target")
                .with_format(format)
                .with_usage(
                    wgpu::TextureUsages::COPY_SRC
                        | wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .build(display.device(), size),
        );
        let depth = Some(
            self.load_texture(
                display,
                TextureBuilder::depth()
                    .with_label("offscreen_depth_target")
                    .with_usage(
                        wgpu::TextureUsages::COPY_SRC
                            | wgpu::TextureUsages::TEXTURE_BINDING
                            | wgpu::TextureUsages::RENDER_ATTACHMENT,
                    )
                    .build(display.device(), size),
            ),
        );
        OffscreenFramebuffer {
            color,
            depth,
            size,
            format,
        }
    }

    pub fn load_texture(&mut self, display: &Display, t: Texture) -> TextureRef {
        let layout = self.bind_group_layout(display.device(), t.binding_type());
        self.texture_manager
            .insert(BoundTexture::new(display.device(), &layout, t))
    }

    pub fn get_texture(&self, texture: impl Into<Option<TextureRef>>) -> &BoundTexture {
        self.texture_manager
            .get(texture.into().unwrap_or(self.default_texture))
            .unwrap()
    }

    pub fn replace_texture(&mut self, display: &Display, texture_ref: TextureRef, value: Texture) {
        let layout = self.bind_group_layout(display.device(), value.binding_type());
        *self.texture_manager.get_mut(texture_ref).unwrap() =
            BoundTexture::new(display.device(), &layout, value);
    }

    pub fn prepare_mesh<V: VertexData>(&mut self, mesh: Mesh<V>) -> MeshRef<V> {
        self.mesh_manager.insert(mesh.inner).into()
    }

    pub fn default_texture(&self) -> TextureRef {
        self.default_texture
    }
}

pub struct RenderPass<'a, 'p> {
    pub render_state: &'a mut RenderState,
    display: &'p Display,
    raw_pass: &'p mut wgpu::RenderPass<'p>,

    active_mesh: Option<RawMeshRef>,
    active_pipeline: Option<RawPipelineRef>,
    active_texture: Option<TextureRef>,
    current_draw_range: Range<u32>,
}

impl<'a, 'p> Deref for RenderPass<'a, 'p> {
    type Target = wgpu::RenderPass<'p>;

    fn deref(&self) -> &Self::Target {
        &self.raw_pass
    }
}

impl<'a, 'p> DerefMut for RenderPass<'a, 'p> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw_pass
    }
}

impl<'a, 'p> RenderPass<'a, 'p> {
    const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
    const GLOBAL_UNIFORMS_BIND_GROUP_INDEX: u32 = 1;
    pub const VIEW_PROJECTION_UNIFORMS_BIND_GROUP_INDEX: u32 = 2;

    pub fn new(
        render_state: &'a mut RenderState,
        display: &'p Display,
        raw_pass: &'p mut wgpu::RenderPass<'p>,
    ) -> Self {
        Self {
            display,
            render_state,
            raw_pass,
            active_mesh: None,
            active_pipeline: None,
            active_texture: None,
            current_draw_range: 0..0,
        }
    }

    pub fn set_active_pipeline<V, I>(&mut self, pipeline: impl Into<Option<PipelineRef<V, I>>>) {
        self.set_active_pipeline_raw(pipeline.into().map(PipelineRef::raw));
    }

    pub(super) fn set_active_pipeline_raw(&mut self, raw: Option<RawPipelineRef>) {
        if raw == self.active_pipeline {
            return;
        }

        let p = self
            .render_state
            .pipelines
            .get(raw.unwrap_or(self.render_state.default_pipeline.raw()))
            .unwrap();
        self.raw_pass.set_pipeline(p);
        self.active_pipeline = raw;
    }

    pub fn set_active_mesh<V: VertexData>(&mut self, mesh: MeshRef<V>) {
        self.set_active_mesh_raw(mesh.raw());
    }

    pub(super) fn set_active_mesh_raw(&mut self, raw: RawMeshRef) {
        self.active_mesh = Some(raw);
    }

    pub fn bind_texture(&mut self, texture: impl Into<Option<TextureRef>>) {
        self.raw_pass.set_bind_group(
            Self::TEXTURE_BIND_GROUP_INDEX,
            self.render_state
                .get_texture(texture.into())
                .bind_group()
                .deref(),
            &[],
        );
    }

    pub fn bind_texture_data(&mut self, texture_data: &BoundTexture) {
        self.raw_pass.set_bind_group(
            Self::TEXTURE_BIND_GROUP_INDEX,
            texture_data.bind_group().deref(),
            &[],
        );
    }

    pub fn draw_raw_mesh_ex(
        &mut self,
        raw_mesh: RawMeshRef,
        base_vertex: i32,
        mesh_indices: Option<Range<u32>>,
        instances: Range<u32>,
    ) {
        let mesh = self.render_state.mesh_manager.get(raw_mesh).unwrap();
        self.raw_pass
            .set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.raw_pass
            .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.raw_pass.draw_indexed(
            mesh_indices.unwrap_or(0..mesh.num_indices),
            base_vertex,
            instances,
        );
    }

    pub fn draw_mesh<V: VertexData>(&mut self, mesh: MeshRef<V>) {
        self.draw_raw_mesh_ex(mesh.raw(), 0, None, 0..1)
    }

    #[inline]
    pub fn draw_instance<V: VertexData, I: InstanceData>(
        &mut self,
        instance: &InstanceRenderData<V, I>,
    ) {
        let pipeline = instance.pipeline.map(|p| p.raw());
        if pipeline != self.active_pipeline {
            self.flush_draw_calls();
            self.set_active_pipeline_raw(pipeline);
        }
        let mesh = instance.mesh.raw();
        if mesh != self.active_mesh.unwrap_or_default() {
            self.flush_draw_calls();
            self.set_active_mesh_raw(mesh);
        }
        if instance.texture != self.active_texture {
            self.flush_draw_calls();
            self.active_texture = instance.texture;
            self.bind_texture(instance.texture);
        }
        self.current_draw_range.end += 1;
        self.render_state.instance_storage.add(&instance.instance);
    }

    #[inline]
    pub fn draw_quad(&mut self, texture: impl Into<Option<TextureRef>>, transform: impl Transform) {
        self.draw_quad_ex(texture.into(), transform, Color::WHITE, Rect::default())
    }

    #[inline]
    pub fn draw_quad_ex(
        &mut self,
        texture: Option<TextureRef>,
        transform: impl Transform,
        c: Color,
        subtexture: Rect,
    ) {
        let transform = transform.as_mat4();
        self.draw_instance(&InstanceRenderData {
            mesh: self.render_state.quad_mesh,
            instance: BasicInstanceData {
                transform,
                tint: c,
                subtexture,
            },
            texture,
            pipeline: None,
        });
    }

    #[inline]
    pub fn draw_rect(&mut self, rect: Rect, c: Color, texture: impl Into<Option<TextureRef>>) {
        self.draw_quad_ex(
            texture.into(),
            Transform2D {
                position: rect.pos,
                scale: rect.dim,
                rotation_rad: 0.0,
            },
            c,
            Rect::new(0.0, 0.0, 0.0, 0.0),
        );
    }

    #[inline]
    pub fn draw_text(
        &mut self,
        font: &RenderableFont,
        s: impl AsRef<str>,
        transform: impl Transform,
        opts: TextDisplayOptions,
    ) {
        let m = transform.as_mat4();
        for glyph_data in font.layout_text(s.as_ref(), opts.layout) {
            let transform = m * Mat4::from_scale_rotation_translation(
                glyph_data.bounds.dim.extend(1.0),
                Quat::IDENTITY,
                glyph_data.bounds.pos.extend(0.0),
            );
            self.draw_instance(&InstanceRenderData {
                instance: BasicInstanceData {
                    subtexture: glyph_data.subtexture,
                    tint: opts.color,
                    transform,
                    ..Default::default()
                },
                mesh: self.render_state.quad_mesh,
                texture: Some(font.texture()),
                pipeline: Some(self.render_state.text_pipeline),
            });
        }
    }

    fn flush_draw_calls(&mut self) {
        if self.current_draw_range.is_empty() {
            return;
        }
        if self.active_pipeline.is_none() {
            self.set_active_pipeline_raw(Some(self.render_state.default_pipeline.raw()));
        }
        self.render_state
            .instance_storage
            .update_buffer(self.display);
        self.raw_pass
            .set_vertex_buffer(1, self.render_state.instance_storage.buffer().slice(..));
        self.draw_raw_mesh_ex(
            self.active_mesh.expect("no active mesh"),
            0,
            None,
            self.current_draw_range.clone(),
        );
        self.current_draw_range.start = self.current_draw_range.end
    }
}
