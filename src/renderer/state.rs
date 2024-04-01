use std::{
    collections::HashMap,
    hash::Hash,
    marker::PhantomData,
    ops::{Deref, DerefMut, Range},
    sync::{Arc, Mutex},
};

use glam::{Mat4, Vec3};
use itertools::Itertools;
use slotmap::{Key, SlotMap};
use wgpu::{VertexAttribute, VertexBufferLayout};

use super::{
    display::Display,
    instance::{InstanceRenderer, InstanceStorage},
    mesh::{LoadMesh, Mesh, RawMeshRef, UntypedMesh},
    texture::{Texture, TextureBuilder},
    BasicInstanceData, BindGroup, Bindable, InstanceData, MeshRef, OffscreenFramebuffer,
    RenderTarget, TextureRef, UniformBindGroup, UniformBuffer, UniformData, DEFAULT_TEXTURE_DATA,
};
use crate::geom::{BasicVertexData, Point, VertexData};

pub type BoundTexture = BindGroup<Texture>;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {
    pub time: f32,
}

impl UniformData for GlobalUniforms {}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewProjectionUniforms {
    pub view: Mat4,
    pub projection: Mat4,
    pub pos: Vec3,
}

impl UniformData for ViewProjectionUniforms {}

impl UniformData for Vec3 {}

struct CachePool<T> {
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
    pub fn get(&self, view_projection: &U) -> Arc<wgpu::BindGroup> {
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
            .write_buffer(bg.buffer(), 0, bytemuck::bytes_of(view_projection));
        bg.bind_group().clone()
    }
}

pub struct PartialRenderPass<'a> {
    display: &'a Display,
    command_buffer: wgpu::CommandBuffer,
}

impl PartialRenderPass<'_> {
    pub fn command_buffer(self) -> wgpu::CommandBuffer {
        self.command_buffer
    }

    pub fn submit(self) {
        self.display.queue().submit([self.command_buffer]);
    }
}

pub trait BindingSlot {
    fn slot(&self) -> u32;
    fn value(&self) -> &Arc<wgpu::BindGroup>;
}

pub trait Bindings {
    fn types() -> Vec<BindingType>;
}

slotmap::new_key_type! {
    pub(super) struct RawPipelineRef;
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PipelineRef<V, I> {
    raw: RawPipelineRef,
    _marker: PhantomData<(V, I)>,
}

impl<V, I> PipelineRef<V, I> {
    pub(super) fn raw(self) -> RawPipelineRef {
        self.raw
    }

    pub fn is_null(&self) -> bool {
        self.raw.is_null()
    }
}

impl<V, I> From<RawPipelineRef> for PipelineRef<V, I> {
    fn from(raw: RawPipelineRef) -> Self {
        PipelineRef {
            raw,
            _marker: PhantomData,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum BindingType {
    Uniform,
    Texture { depth: bool },
    Direct(wgpu::BindingType),
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
            BindingType::Texture { depth } => vec![
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: if depth {
                            wgpu::TextureSampleType::Depth
                        } else {
                            wgpu::TextureSampleType::Float { filterable: true }
                        },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(if depth {
                        wgpu::SamplerBindingType::Comparison
                    } else {
                        wgpu::SamplerBindingType::Filtering
                    }),
                    count: None,
                },
            ],
            BindingType::Direct(ty) => vec![wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
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

    default_bindings: Vec<BindingType>,
    bind_group_layouts: HashMap<BindingType, Arc<wgpu::BindGroupLayout>>,

    instance_storage: InstanceStorage,
    view_proj_bind_groups: Mutex<CachePool<BindGroup<UniformBuffer<ViewProjectionUniforms>>>>,

    texture_manager: SlotMap<TextureRef, BoundTexture>,
    default_texture: TextureRef,

    mesh_manager: SlotMap<RawMeshRef, UntypedMesh>,
    pipelines: SlotMap<RawPipelineRef, wgpu::RenderPipeline>,
    default_pipeline: PipelineRef<BasicVertexData, BasicInstanceData>,
}

impl RenderState {
    pub fn new(display: &Display, default_shader: &wgpu::ShaderModule) -> Self {
        let device = display.device();

        let global_uniforms = UniformBindGroup::new(
            device,
            &BindingType::Uniform.create_layout(device, "global uniform"),
            UniformBuffer::new(device, Default::default()),
        );
        let mesh_manager = SlotMap::with_key();
        let instance_storage = InstanceStorage::new(display, 1024);

        let mut s = Self {
            default_bindings: vec![
                BindingType::Texture { depth: false }, // material texture
                BindingType::Uniform,                  // global
                BindingType::Uniform,                  // view/projection
            ],
            bind_group_layouts: HashMap::default(),
            texture_manager: SlotMap::with_key(),
            mesh_manager,
            pipelines: SlotMap::with_key(),
            global_uniforms,
            instance_storage,
            quad_mesh: Default::default(),
            view_proj_bind_groups: Default::default(),
            default_pipeline: Default::default(),
            default_texture: Default::default(),
        };
        s.bind_group_layout(device, BindingType::Uniform);

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
        s.default_pipeline = s.create_pipeline_with_key::<BasicVertexData, BasicInstanceData>(
            "Default Render Pipeline",
            display,
            default_shader,
            &[],
            None,
        );
        s
    }

    fn bind_group_layout(
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
    ) -> UniformBindGroup<U> {
        let layout = self.bind_group_layout(device, BindingType::Uniform);
        let resource = UniformBuffer::new(device, uniform);
        BindGroup::new(device, &layout, resource)
    }

    pub fn quad_mesh(&self) -> MeshRef<BasicVertexData> {
        self.quad_mesh
    }

    pub fn default_pipeline(&self) -> PipelineRef<BasicVertexData, BasicInstanceData> {
        self.default_pipeline
    }

    pub fn create_pipeline<V: VertexData, I: InstanceData>(
        &mut self,
        name: &str,
        display: &Display,
        shader: &wgpu::ShaderModule,
        extra_bindings: &[BindingType],
    ) -> PipelineRef<V, I> {
        self.create_pipeline_with_key::<V, I>(name, display, shader, extra_bindings, None)
    }

    pub fn replace_pipeline<V: VertexData, I: InstanceData>(
        &mut self,
        name: &str,
        display: &Display,
        shader: &wgpu::ShaderModule,
        extra_bindings: &[BindingType],
        key: PipelineRef<V, I>,
    ) -> PipelineRef<V, I> {
        self.create_pipeline_with_key::<V, I>(name, display, shader, extra_bindings, Some(key))
    }

    pub fn create_pipeline_with_key<V: VertexData, I: InstanceData>(
        &mut self,
        name: &str,
        display: &Display,
        shader: &wgpu::ShaderModule,
        extra_bindings: &[BindingType],
        key: Option<PipelineRef<V, I>>,
    ) -> PipelineRef<V, I> {
        let bind_group_layouts_vec = self
            .default_bindings
            .clone() // TODO: this clone seems unnecessary
            .iter()
            .chain(extra_bindings.iter())
            .map(|t| self.bind_group_layout(display.device(), *t))
            .collect_vec();

        let refs = bind_group_layouts_vec
            .iter()
            .map(|b| b.as_ref())
            .collect_vec();
        // TODO: do we need/want to dedupe or cache this?
        let layout = display
            .device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Layout", name)),
                bind_group_layouts: &refs,
                push_constant_ranges: &[],
            });
        let vv = V::vertex_layout();
        let ii = I::vertex_layout();
        let start_location = vv
            .attributes
            .last()
            .map(|a| a.shader_location + 1)
            .unwrap_or_default();
        let mut vertex_buffers = vec![vv];
        let offset_attributes = ii
            .attributes
            .iter()
            .map(|a| VertexAttribute {
                shader_location: start_location + a.shader_location,
                ..a.clone()
            })
            .collect_vec();
        if offset_attributes.len() > 0 {
            let ii = VertexBufferLayout {
                attributes: &offset_attributes,
                ..ii
            };
            vertex_buffers.push(ii);
        }

        let pipeline = display
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: "vs_main",
                    buffers: &vertex_buffers,
                },
                fragment: Some(wgpu::FragmentState {
                    module: shader,
                    entry_point: "fs_main",
                    // TODO: technically I think this should be some `for_render_targets` slice
                    // maybe `PipelineBuilder` eventually
                    targets: &[Some(wgpu::ColorTargetState {
                        format: display.format(),
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Cw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: TextureBuilder::DEFAULT_DEPTH_FORMAT,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
            });
        match key {
            Some(key) => {
                *self.pipelines.get_mut(key.raw()).unwrap() = pipeline;
                key
            }
            None => self.pipelines.insert(pipeline).into(),
        }
    }

    #[must_use]
    pub fn render_pass<'a>(
        &mut self,
        display: &'a Display,
        name: &str,
        render_target: &impl RenderTarget,
        view_projection: &ViewProjectionUniforms,
        pass: impl FnOnce(&mut InstanceRenderer),
    ) -> PartialRenderPass<'a> {
        let alloc = BindGroupAllocator {
            display,
            layout: &self.bind_group_layout(display.device(), BindingType::Uniform),
            bind_groups: &self.view_proj_bind_groups,
        };
        let mut r = InstanceRenderer::new(&alloc, &mut self.instance_storage, self.quad_mesh);
        r.set_view_projection(view_projection);
        pass(&mut r);
        r.commit(display);

        let mut encoder = display.command_encoder();
        {
            let mut raw_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(name),
                color_attachments: &[Some(
                    render_target.color_attachment(&self, wgpu::LoadOp::Clear(wgpu::Color::BLACK)),
                )],
                depth_stencil_attachment: render_target.depth_stencil_attachment(
                    &self,
                    wgpu::LoadOp::Clear(1.0),
                    None,
                ),
                ..Default::default()
            });
            raw_pass.set_bind_group(
                RenderPass::GLOBAL_UNIFORMS_BIND_GROUP_INDEX,
                self.global_uniforms.bind_group(),
                &[],
            );
            let mut pass = RenderPass {
                raw_pass,
                render_state: self,
                active_mesh: None,
                active_pipeline: None,
            };
            pass.set_active_pipeline(self.default_pipeline);
            pass.bind_texture(self.default_texture);
            self.instance_storage.render_to(&mut pass);
        }
        self.view_proj_bind_groups.lock().unwrap().reset();
        self.instance_storage.clear();
        PartialRenderPass {
            display,
            command_buffer: encoder.finish(),
        }
    }

    #[must_use]
    pub fn direct_render_pass<'a>(
        &mut self,
        display: &'a Display,
        name: &str,
        render_target: &impl RenderTarget,
        view_projection: &ViewProjectionUniforms,
        pass_func: impl FnOnce(&mut RenderPass<'_>),
    ) -> PartialRenderPass<'a> {
        let bind_group = {
            let layout = self.bind_group_layout(display.device(), BindingType::Uniform);
            let mut x = self.view_proj_bind_groups.lock().unwrap();
            let bg = x.get(|| {
                BindGroup::new(
                    display.device(),
                    &layout,
                    UniformBuffer::<ViewProjectionUniforms>::new(
                        display.device(),
                        Default::default(),
                    ),
                )
            });
            display
                .queue()
                .write_buffer(bg.buffer(), 0, bytemuck::bytes_of(view_projection));
            bg.bind_group().clone()
        };

        let mut encoder = display.command_encoder();
        {
            let mut raw_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(name),
                color_attachments: &[Some(
                    render_target.color_attachment(&self, wgpu::LoadOp::Clear(wgpu::Color::BLACK)),
                )],
                depth_stencil_attachment: render_target.depth_stencil_attachment(
                    &self,
                    wgpu::LoadOp::Clear(1.0),
                    None,
                ),
                ..Default::default()
            });
            raw_pass.set_bind_group(
                RenderPass::GLOBAL_UNIFORMS_BIND_GROUP_INDEX,
                self.global_uniforms.bind_group(),
                &[],
            );
            raw_pass.set_bind_group(
                RenderPass::VIEW_PROJECTION_UNIFORMS_BIND_GROUP_INDEX,
                &bind_group,
                &[],
            );
            let mut pass = RenderPass {
                raw_pass,
                render_state: self,
                active_mesh: None,
                active_pipeline: None,
            };
            pass.set_active_pipeline(self.default_pipeline);
            pass.bind_texture(self.default_texture);
            pass_func(&mut pass);
        }
        self.view_proj_bind_groups.lock().unwrap().reset();
        PartialRenderPass {
            display,
            command_buffer: encoder.finish(),
        }
    }

    pub fn create_offscreen_framebuffer(
        &mut self,
        display: &Display,
        size: Point<u32>,
    ) -> OffscreenFramebuffer {
        let color = self.load_texture(
            display,
            TextureBuilder::render_target()
                .with_label("offscreen_color_target")
                .build(display.device(), size),
        );
        let depth = Some(
            self.load_texture(
                display,
                TextureBuilder::depth()
                    .with_label("offscreen_depth_target")
                    .build(display.device(), size),
            ),
        );
        OffscreenFramebuffer { color, depth, size }
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
}

pub struct RenderPass<'a> {
    pub render_state: &'a RenderState,
    raw_pass: wgpu::RenderPass<'a>,
    active_mesh: Option<RawMeshRef>,
    active_pipeline: Option<RawPipelineRef>,
}

impl<'a> Deref for RenderPass<'a> {
    type Target = wgpu::RenderPass<'a>;

    fn deref(&self) -> &Self::Target {
        &self.raw_pass
    }
}

impl<'a> DerefMut for RenderPass<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.raw_pass
    }
}

impl<'a> RenderPass<'a> {
    // TODO: maybe we can put these in an enum wrapper or something?
    //   pass.set_bind_group_slot(BindGroupSlot::ViewProjection(view_proj))
    const TEXTURE_BIND_GROUP_INDEX: u32 = 0;
    const GLOBAL_UNIFORMS_BIND_GROUP_INDEX: u32 = 1;
    pub const VIEW_PROJECTION_UNIFORMS_BIND_GROUP_INDEX: u32 = 2;

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
        self.set_pipeline(p);
        self.active_pipeline = raw;
    }

    pub fn set_active_mesh<V: VertexData>(&mut self, mesh: MeshRef<V>) {
        self.set_active_mesh_raw(mesh.raw());
    }

    pub(super) fn set_active_mesh_raw(&mut self, raw: RawMeshRef) {
        self.active_mesh = Some(raw);
    }

    pub fn bind_texture(&mut self, texture: impl Into<Option<TextureRef>>) {
        self.bind_texture_data(self.render_state.get_texture(texture.into()))
    }

    fn bind_texture_data(&mut self, texture_data: &'a BoundTexture) {
        self.raw_pass.set_bind_group(
            Self::TEXTURE_BIND_GROUP_INDEX,
            texture_data.bind_group(),
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

    pub fn draw_active_mesh_instanced(&mut self, instances: Range<u32>) {
        self.draw_raw_mesh_ex(
            self.active_mesh.expect("no active mesh"),
            0,
            None,
            instances,
        )
    }
}
