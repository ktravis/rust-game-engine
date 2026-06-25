use std::{
    collections::HashMap,
    ops::{Deref, DerefMut, Range},
    sync::Mutex,
};

use bytemuck::Zeroable;
use glam::{Mat4, Quat, Vec3, Vec4};
use shadertype_derive::shader_uniform_type;
use slotmap::SlotMap;

use super::{
    display::Display,
    instance::{InstanceRenderData, InstanceStorage},
    mesh::{LoadMesh, Mesh, RawMeshRef, UntypedMesh},
    shader_type::GlobalUniforms,
    text::{RenderableFont, TextDisplayOptions},
    texture::{Texture, TextureBuilder},
    BasicInstanceData, MeshRef, OffscreenFramebuffer, PipelineBuilder, PipelineRef, RawPipelineRef,
    RenderTarget, TextureRef, DEFAULT_TEXTURE_DATA,
};
use crate::{
    camera::Camera,
    color::Color,
    geom::{BasicVertexData, Point, Rect},
    renderer::{
        bindings::{
            create_uniform_bind_group, texture_bgl_entries, BindGroup, UniformBindGroup,
            UNIFORM_BGL_ENTRY,
        },
        shader_type::VertexInput,
    },
    transform::{Transform, Transform2D},
};

pub type BoundTexture = BindGroup<Texture>;

#[shader_uniform_type]
pub struct ViewProjectionUniforms {
    pub view: Mat4,
    pub projection: Mat4,
    pub camera_pos: Vec3,
    #[skip]
    pub _pad_camera_pos: [u8; 4u32 as usize],
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
    items: Vec<T>,
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
    pub fn get<'a>(&'a mut self, ctor: impl FnOnce() -> T) -> &'a T {
        if self.in_use >= self.items.len() {
            self.items.push(ctor());
        }
        let i = self.in_use;
        self.in_use += 1;
        &self.items[i]
    }

    pub fn reset(&mut self) {
        self.in_use = 0;
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

pub struct RenderState {
    pub global_uniforms: UniformBindGroup<GlobalUniforms>,
    quad_mesh: MeshRef<BasicVertexData>,

    instance_storage: InstanceStorage,
    view_proj_bind_groups: Mutex<CachePool<UniformBindGroup<ViewProjectionUniforms>>>,

    texture_bind_group_layouts: HashMap<wgpu::TextureFormat, wgpu::BindGroupLayout>,
    texture_manager: SlotMap<TextureRef, BoundTexture>,
    default_texture: TextureRef,

    mesh_manager: SlotMap<RawMeshRef, UntypedMesh>,
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

        let global_uniforms = create_uniform_bind_group(device, GlobalUniforms::zeroed());
        let mesh_manager = SlotMap::with_key();
        let instance_storage = InstanceStorage::new(display, 1024);

        let mut s = Self {
            texture_manager: SlotMap::with_key(),
            mesh_manager,
            pipelines: SlotMap::with_key(),
            global_uniforms,
            instance_storage,
            texture_bind_group_layouts: Default::default(),
            quad_mesh: Default::default(),
            view_proj_bind_groups: Default::default(),
            default_pipeline: Default::default(),
            text_pipeline: Default::default(),
            default_texture: Default::default(),
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
        let view_proj_uniform_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("view proj uniform bg layout"),
                entries: &[UNIFORM_BGL_ENTRY],
            });
        let main_texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("main texture"),
            entries: &texture_bgl_entries(TextureBuilder::DEFAULT_FORMAT),
        });
        s.default_pipeline = s
            .pipeline_builder()
            .with_label("Default Render Pipeline")
            .with_bind_group_layouts(vec![&main_texture_bgl, &view_proj_uniform_bgl])
            .build(display.device(), &default_shader);
        s.text_pipeline = s
            .pipeline_builder()
            .with_label("Text Render Pipeline")
            .with_bind_group_layouts(vec![&main_texture_bgl, &view_proj_uniform_bgl])
            .build(display.device(), &text_shader);
        s
    }

    pub fn quad_mesh(&self) -> MeshRef<BasicVertexData> {
        self.quad_mesh
    }

    pub fn pipeline_builder<'a>(&'a mut self) -> PipelineBuilder<'a> {
        PipelineBuilder::new(self)
    }

    pub(super) fn add_pipeline<V: VertexInput, I: VertexInput>(
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

        let mut encoder = display.command_encoder();
        {
            let color_attachments: [Option<wgpu::RenderPassColorAttachment>;
                Self::MAX_COLOR_ATTACHMENTS] = std::array::from_fn(|i| {
                color_targets.get(i).map(|target| {
                    let view = match target {
                        RenderTarget::TextureView(view) => *view,
                        RenderTarget::TextureRef(texture) => {
                            &self.texture_manager.get(*texture).unwrap().resource.view
                        }
                    };
                    wgpu::RenderPassColorAttachment {
                        view,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        resolve_target: None,
                        depth_slice: None,
                    }
                })
            });
            let raw_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(name),
                color_attachments: &color_attachments[..color_targets.len()],
                depth_stencil_attachment: depth_target.map(|target| {
                    let view = match target {
                        RenderTarget::TextureView(view) => view,
                        RenderTarget::TextureRef(texture) => {
                            &self.texture_manager.get(texture).unwrap().resource.view
                        }
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
                ..Default::default()
            });
            let mut render_pass = RenderPass::new(self, display, raw_pass);
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

    fn bgl_for_texture_format(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> &wgpu::BindGroupLayout {
        self.texture_bind_group_layouts
            .entry(format)
            .or_insert_with(|| {
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("texture({:?})", format)),
                    entries: &texture_bgl_entries(format),
                })
            })
    }

    pub fn load_texture(&mut self, display: &Display, t: Texture) -> TextureRef {
        let bt = BoundTexture::for_texture(
            display.device(),
            self.bgl_for_texture_format(display.device(), t.format()),
            t,
        );
        self.texture_manager.insert(bt)
    }

    pub fn get_texture(&self, texture: impl Into<Option<TextureRef>>) -> &wgpu::BindGroup {
        self.texture_manager
            .get(texture.into().unwrap_or(self.default_texture))
            .unwrap()
            .bind_group()
    }

    pub fn replace_texture(&mut self, display: &Display, texture_ref: TextureRef, value: Texture) {
        let bt = BoundTexture::for_texture(
            display.device(),
            self.bgl_for_texture_format(display.device(), value.format()),
            value,
        );
        *self.texture_manager.get_mut(texture_ref).unwrap() = bt;
    }

    pub fn prepare_mesh<V: VertexInput>(&mut self, mesh: Mesh<V>) -> MeshRef<V> {
        self.mesh_manager.insert(mesh.inner).into()
    }

    pub fn default_texture(&self) -> TextureRef {
        self.default_texture
    }
}

pub struct RenderPass<'a, 'p> {
    pub render_state: &'a mut RenderState,
    display: &'p Display,
    raw_pass: wgpu::RenderPass<'p>,

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

    fn new(
        render_state: &'a mut RenderState,
        display: &'p Display,
        raw_pass: wgpu::RenderPass<'p>,
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

    pub fn draw_mesh<V: VertexInput>(&mut self, mesh: MeshRef<V>) {
        self.draw_raw_mesh_ex(mesh.raw(), 0, None, 0..1)
    }

    #[inline]
    pub fn draw_instance<V: VertexInput, I: VertexInput>(
        &mut self,
        instance: &InstanceRenderData<V, I>,
    ) {
        let pipeline = instance.pipeline.map(|p| p.raw());
        if pipeline != self.active_pipeline {
            self.flush_draw_calls();
            self.active_pipeline = pipeline;
        }
        let mesh = instance.mesh.raw();
        if mesh != self.active_mesh.unwrap_or_default() {
            self.flush_draw_calls();
            self.active_mesh = Some(mesh);
        }
        if instance.texture != self.active_texture {
            self.flush_draw_calls();
            self.active_texture = instance.texture;
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
        let p = self
            .render_state
            .pipelines
            .get(
                self.active_pipeline
                    .unwrap_or(self.render_state.default_pipeline.raw()),
            )
            .unwrap();
        self.raw_pass.set_pipeline(p);
        self.render_state
            .instance_storage
            .update_buffer(self.display);
        self.raw_pass.set_bind_group(
            Self::TEXTURE_BIND_GROUP_INDEX,
            self.render_state.get_texture(self.active_texture),
            &[],
        );
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
