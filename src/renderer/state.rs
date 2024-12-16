use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut, Range},
    sync::{Arc, Mutex},
};

use glam::{Mat4, Vec3, Vec4};
use slotmap::SlotMap;
use wgpu::BindGroupLayout;

use super::{
    display::Display,
    instance::{InstanceRenderer, InstanceStorage},
    mesh::{LoadMesh, Mesh, RawMeshRef, UntypedMesh},
    texture::{Texture, TextureBuilder},
    BasicInstanceData, BindGroup, Bindable, InstanceData, MeshRef, OffscreenFramebuffer,
    PipelineBuilder, PipelineRef, RawPipelineRef, RenderTarget, TextureRef, UniformBindGroup,
    UniformBuffer, UniformData, DEFAULT_TEXTURE_DATA,
};
use crate::{
    camera::Camera,
    geom::{BasicVertexData, Point, VertexData},
};

pub type BoundTexture = BindGroup<Texture>;

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalUniforms {
    pub time: f32,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ViewProjectionUniforms {
    pub view: Mat4,
    pub projection: Mat4,
    pub pos: Vec3,
    pub inverse_view: Mat4,
}

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
            pos: camera.position(),
        }
    }
}

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
            .write_buffer(bg.buffer(), 0, bytemuck::bytes_of(&view_projection.raw()));
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
}

impl RenderState {
    const MAX_COLOR_ATTACHMENTS: usize = 8;

    pub fn new(display: &Display, default_shader: &wgpu::ShaderModule) -> Self {
        let device = display.device();

        let global_uniforms = UniformBindGroup::new(
            device,
            &BindingType::Uniform.create_layout(device, "global uniform"),
            UniformBuffer::new(device, Default::default()),
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
        pass: impl FnOnce(&mut InstanceRenderer),
    ) -> PartialRenderPass<'a> {
        if color_targets.len() > Self::MAX_COLOR_ATTACHMENTS {
            panic!(
                "too many color targets ({} > {})",
                color_targets.len(),
                Self::MAX_COLOR_ATTACHMENTS
            );
        }
        let alloc = BindGroupAllocator {
            display,
            layout: &self.bind_group_layout(display.device(), BindingType::Uniform),
            bind_groups: &self.view_proj_bind_groups,
        };
        let mut r = InstanceRenderer::new(
            &alloc,
            &mut self.instance_storage,
            self.quad_mesh,
            self.default_pipeline.raw(),
        );
        r.set_view_projection(view_projection);
        pass(&mut r);
        r.commit(display);

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
            // let color =
            //     render_target.color_attachment(&self, wgpu::LoadOp::Clear(wgpu::Color::BLACK));
            // let colors = [color];
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
            let mut pass = RenderPass {
                raw_pass,
                render_state: self,
                active_mesh: None,
                active_pipeline: None,
            };
            // pass.set_active_pipeline(self.default_pipeline);
            pass.bind_texture(self.default_texture);
            self.instance_storage.render_to(&mut pass);
        }
        self.view_proj_bind_groups.lock().unwrap().reset();
        self.instance_storage.clear();
        PartialRenderPass { display, encoder }
    }

    // #[must_use]
    // pub fn direct_render_pass<'a>(
    //     &mut self,
    //     display: &'a Display,
    //     name: &str,
    //     render_target: &RenderTarget,
    //     view_projection: &ViewProjectionUniforms,
    //     pass_func: impl FnOnce(&mut RenderPass<'_>),
    // ) -> PartialRenderPass<'a> {
    //     let bind_group = {
    //         let layout = self.bind_group_layout(display.device(), BindingType::Uniform);
    //         let mut x = self.view_proj_bind_groups.lock().unwrap();
    //         let bg = x.get(|| {
    //             BindGroup::new(
    //                 display.device(),
    //                 &layout,
    //                 UniformBuffer::<ViewProjectionUniforms>::new(
    //                     display.device(),
    //                     Default::default(),
    //                 ),
    //             )
    //         });
    //         display
    //             .queue()
    //             .write_buffer(bg.buffer(), 0, bytemuck::bytes_of(view_projection));
    //         bg.bind_group().clone()
    //     };
    //
    //     let mut encoder = display.command_encoder();
    //     {
    //         let color =
    //             render_target.color_attachment(&self, wgpu::LoadOp::Clear(wgpu::Color::BLACK));
    //         let colors = [color];
    //         let mut raw_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
    //             label: Some(name),
    //             color_attachments: if colors[0].is_some() { &colors } else { &[] },
    //             depth_stencil_attachment: render_target.depth_stencil_attachment(
    //                 &self,
    //                 wgpu::LoadOp::Clear(1.0),
    //                 None,
    //             ),
    //             ..Default::default()
    //         });
    //         raw_pass.set_bind_group(
    //             RenderPass::GLOBAL_UNIFORMS_BIND_GROUP_INDEX,
    //             self.global_uniforms.bind_group(),
    //             &[],
    //         );
    //         raw_pass.set_bind_group(
    //             RenderPass::VIEW_PROJECTION_UNIFORMS_BIND_GROUP_INDEX,
    //             &bind_group,
    //             &[],
    //         );
    //         let mut pass = RenderPass {
    //             raw_pass,
    //             render_state: self,
    //             active_mesh: None,
    //             active_pipeline: None,
    //         };
    //         pass.set_active_pipeline(self.default_pipeline);
    //         pass.bind_texture(self.default_texture);
    //         pass_func(&mut pass);
    //     }
    //     self.view_proj_bind_groups.lock().unwrap().reset();
    //     PartialRenderPass { display, encoder }
    // }

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

    pub fn draw_active_mesh_instanced(&mut self, instances: Range<u32>) {
        self.draw_raw_mesh_ex(
            self.active_mesh.expect("no active mesh"),
            0,
            None,
            instances,
        )
    }
}
