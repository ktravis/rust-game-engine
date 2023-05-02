use glam::{vec2, vec3, Mat4, Quat, Vec2};
use miniquad::*;
use ttf_parser::Face;

use crate::{
    atlas::Atlas,
    color::*,
    default_shader,
    font::{self, FontAtlas},
    geom::*,
    mesh::Mesh,
    resources,
    sprite::Sprite,
    text_shader,
    transform::*,
};

#[derive(Debug, Clone, Copy)]
pub struct InstanceData<T: ModelTransform> {
    pub tint: Color,
    pub subtexture: Rect,
    pub transform: T,
}

impl<T: ModelTransform> From<InstanceData<T>> for RawInstanceData {
    fn from(other: InstanceData<T>) -> Self {
        RawInstanceData {
            uv_scale: other.subtexture.dim,
            uv_offset: other.subtexture.pos,
            tint: other.tint.as_u8(),
            model: other.transform.model_transform(),
        }
    }
}

impl<T: ModelTransform> Default for InstanceData<T> {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            tint: Color::WHITE,
            subtexture: Rect::new(0., 0., 1., 1.),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct RawInstanceData {
    uv_scale: Vec2,
    uv_offset: Vec2,
    tint: [u8; 4],
    model: Mat4,
}

#[derive(Debug, Clone, Copy)]
pub enum DisplayMode {
    Stretch,
    Centered,
}

#[rustfmt::skip]
const DEFAULT_TEXTURE_DATA: [u8; 16] = [
    255, 0, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
    255, 0, 255, 255,
];

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct MeshRef(usize);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct MeshOffsets {
    offset: u16,
    count: u16,
}

#[derive(Default)]
pub struct MeshManager {
    mesh_refs: Vec<(String, MeshOffsets)>,
    vertices: Vec<VertexData>,
    indices: Vec<u16>,
    needs_rebuild: bool,
}

impl MeshManager {
    pub fn add(&mut self, name: impl Into<String>, mesh: Mesh) -> MeshRef {
        let mesh_ref = MeshOffsets {
            offset: self.indices.len() as _,
            count: mesh.indices.len() as _,
        };
        let vertex_offset: u16 = self.vertices.len() as _;
        self.vertices.extend(mesh.vertices.iter().cloned());
        self.indices
            .extend(mesh.indices.iter().map(|i| i + vertex_offset));
        let mesh_index = self.mesh_refs.len();
        self.mesh_refs.push((name.into(), mesh_ref));
        self.needs_rebuild = true;
        MeshRef(mesh_index)
    }

    pub fn get(&self, name: impl AsRef<str>) -> Option<MeshRef> {
        let name = name.as_ref();
        self.mesh_refs
            .iter()
            .enumerate()
            .find_map(|(i, (mesh_name, r))| (mesh_name == name).then_some(MeshRef(i)))
    }

    pub fn buffers(&mut self, ctx: &mut GraphicsContext) -> GeometryBuffers {
        self.needs_rebuild = false;
        GeometryBuffers::from_slices(ctx, &self.vertices, &self.indices)
    }

    fn offsets(&self, MeshRef(index): MeshRef) -> Option<MeshOffsets> {
        self.mesh_refs
            .iter()
            .enumerate()
            .find_map(|(i, (_, o))| (i == index).then_some(*o))
    }
}

pub struct GeometryBuffers {
    vertices: Buffer,
    indices: Buffer,
}

impl GeometryBuffers {
    pub fn from_meshes(ctx: &mut GraphicsContext, meshes: &[Mesh]) -> Self {
        let (vertices, indices): (Vec<VertexData>, Vec<u16>) =
            meshes
                .iter()
                .fold((vec![], vec![]), |(mut verts, mut inds), m| {
                    verts.extend(m.vertices.iter());
                    inds.extend(m.indices.iter());
                    (verts, inds)
                });
        Self::from_slices(ctx, &vertices, &indices)
    }

    pub fn from_slices(
        ctx: &mut GraphicsContext,
        vertices: &[VertexData],
        indices: &[u16],
    ) -> Self {
        GeometryBuffers {
            vertices: Buffer::immutable(ctx, BufferType::VertexBuffer, vertices),
            indices: Buffer::immutable(ctx, BufferType::IndexBuffer, indices),
        }
    }
}

impl Drop for GeometryBuffers {
    fn drop(&mut self) {
        self.vertices.delete();
        self.indices.delete();
    }
}

pub struct Renderer {
    pub mesh_manager: MeshManager,
    pub target_texture: Texture,
    pub depth_texture: Option<Texture>,
    textures: Vec<Texture>,
    geometry_buffers: GeometryBuffers,

    display_mode: DisplayMode,
    render_pass: miniquad::RenderPass,
    pipeline: Pipeline,
    bindings: Bindings,
    screen_bindings: Bindings,
    pub default_texture: Texture,
    instance_vertex_buffer: Buffer,
    batch: RenderBatch,
    pub font_atlas: FontAtlas,
    text_bindings: Bindings,
    text_pipeline: Pipeline,

    quad_mesh: MeshRef,
    cube_mesh: MeshRef,
}

impl Renderer {
    pub fn new(
        ctx: &mut GraphicsContext,
        target_texture: Texture,
        depth_texture: impl Into<Option<Texture>>,
        shader: miniquad::Shader,
        display_mode: DisplayMode,
    ) -> Self {
        let depth_texture = depth_texture.into();
        let render_pass = miniquad::RenderPass::new(ctx, target_texture, depth_texture);

        let default_texture = Texture::new(
            ctx,
            TextureAccess::Static,
            Some(&DEFAULT_TEXTURE_DATA),
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Repeat,
                filter: FilterMode::Nearest,
                width: 2,
                height: 2,
            },
        );

        let mut mesh_manager = MeshManager::default();
        let quad_mesh = mesh_manager.add("quad", quad::mesh());
        let cube_mesh = mesh_manager.add("cube", cube::mesh());
        let textures = vec![default_texture];
        let geometry_buffers = mesh_manager.buffers(ctx);

        let instance_vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            RenderBatch::MAX_INSTANCES * std::mem::size_of::<RawInstanceData>(),
        );
        let bindings = Bindings {
            vertex_buffers: vec![geometry_buffers.vertices, instance_vertex_buffer],
            index_buffer: geometry_buffers.indices,
            images: textures.clone(),
        };
        let screen_bindings = Bindings {
            images: vec![target_texture],
            ..bindings.clone()
        };
        let pipeline = Pipeline::with_params(
            ctx,
            &[
                BufferLayout::default(), // pos
                // instances
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                // vertex data
                VertexAttribute::with_buffer("position", VertexFormat::Float4, 0),
                VertexAttribute::with_buffer("uv", VertexFormat::Float2, 0),
                // instance data
                VertexAttribute::with_buffer("uv_scale", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("uv_offset", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("tint", VertexFormat::Byte4, 1),
                VertexAttribute::with_buffer("model", VertexFormat::Mat4, 1),
            ],
            shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                alpha_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Zero,
                    BlendFactor::One,
                )),
                // primitive_type: miniquad::PrimitiveType::Lines,
                ..Default::default()
            },
        );
        let text_shader = Shader::new(
            ctx,
            text_shader::VERTEX,
            text_shader::FRAGMENT,
            text_shader::meta(),
        )
        .expect("text shader creation failed");
        let text_pipeline = Pipeline::with_params(
            ctx,
            &[
                BufferLayout::default(), // pos
                // instances
                BufferLayout {
                    step_func: VertexStep::PerInstance,
                    ..Default::default()
                },
            ],
            &[
                // vertex data
                VertexAttribute::with_buffer("position", VertexFormat::Float4, 0),
                VertexAttribute::with_buffer("uv", VertexFormat::Float2, 0),
                // instance data
                VertexAttribute::with_buffer("uv_scale", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("uv_offset", VertexFormat::Float2, 1),
                VertexAttribute::with_buffer("tint", VertexFormat::Byte4, 1),
                VertexAttribute::with_buffer("model", VertexFormat::Mat4, 1),
            ],
            text_shader,
            PipelineParams {
                depth_test: Comparison::LessOrEqual,
                depth_write: true,
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                alpha_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Zero,
                    BlendFactor::One,
                )),
                // primitive_type: miniquad::PrimitiveType::Lines,
                ..Default::default()
            },
        );
        let b = std::fs::read("./res/fonts/Ubuntu-M.ttf").unwrap();
        let face = Face::parse(&b, 0).unwrap();
        let font_atlas = FontAtlas::new(face, Default::default()).unwrap();
        let font_atlas_texture = resources::texture_from_image_with_params(
            ctx,
            font_atlas.image(),
            TextureParams {
                format: TextureFormat::RGBA8,
                wrap: TextureWrap::Clamp,
                filter: FilterMode::Linear,
                width: font_atlas.image().width(),
                height: font_atlas.image().height(),
            },
        );
        let text_bindings = Bindings {
            images: vec![font_atlas_texture],
            ..bindings.clone()
        };
        Self {
            display_mode,
            target_texture,
            depth_texture,
            render_pass,
            bindings,
            screen_bindings,
            pipeline,
            default_texture,
            instance_vertex_buffer,
            batch: Default::default(),
            mesh_manager,
            textures,
            geometry_buffers,
            quad_mesh,
            cube_mesh,
            text_bindings,
            text_pipeline,
            font_atlas,
        }
    }

    fn rebuild_buffers(&mut self, ctx: &mut GraphicsContext) {
        self.geometry_buffers = self.mesh_manager.buffers(ctx);
        self.bindings = {
            Bindings {
                vertex_buffers: vec![self.geometry_buffers.vertices, self.instance_vertex_buffer],
                index_buffer: self.geometry_buffers.indices,
                images: self.textures.clone(),
            }
        };
        self.screen_bindings = Bindings {
            images: vec![self.target_texture],
            ..self.bindings.clone()
        };
    }

    pub fn draw_to_screen(&mut self, ctx: &mut GraphicsContext) {
        let (w, h) = ctx.screen_size();
        let modelview = match self.display_mode {
            DisplayMode::Stretch => Mat4::from_scale(vec3(w, h, 1.0)),
            DisplayMode::Centered => {
                let render_target_size = vec2(
                    self.target_texture.width as _,
                    self.target_texture.height as _,
                );
                let scale = (w / render_target_size.x).min(h / render_target_size.y);

                let scaled_target_size = scale * render_target_size;
                let display_region = Transform2D {
                    pos: vec2(
                        (w - scaled_target_size.x) / 2.0,
                        (h - scaled_target_size.y) / 2.0,
                    ),
                    scale: scaled_target_size,
                    ..Default::default()
                };
                display_region.model_transform()
            }
        };
        let mut render_pass = self.begin_screen_pass(
            ctx,
            RenderPassOptions {
                pass_action: PassAction::clear_color(0., 0., 0., 0.),
                view_transform: modelview,
                // projection: todo!(),
                projection: Some(glam::Mat4::orthographic_lh(0.0, w, 0.0, h, 1.0, -1.0)),
            },
        );
        render_pass.draw_quad(InstanceData::<Transform2D>::default());
    }

    fn begin_pass<'a>(
        &'a mut self,
        ctx: &'a mut GraphicsContext,
        pass_action: PassAction,
        view: Mat4,
        projection: Mat4,
        default: bool,
    ) -> RenderPass<'a> {
        if self.mesh_manager.needs_rebuild {
            self.rebuild_buffers(ctx);
        }
        ctx.begin_pass((!default).then_some(self.render_pass), pass_action);
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_uniforms(&default_shader::Uniforms { view, projection });
        let bindings = if default {
            &self.screen_bindings
        } else {
            &self.bindings
        };
        ctx.apply_bindings(bindings);
        RenderPass {
            ctx,
            current_batch: &mut self.batch,
            transform_stack: vec![],
            instance_buffer: &mut self.instance_vertex_buffer,
            mesh_manager: &mut self.mesh_manager,
            bindings,
            current_texture: None,
            current_mesh: None,
            quad_mesh: self.quad_mesh,
            cube_mesh: self.cube_mesh,
        }
    }

    pub fn begin_text_pass<'a>(
        &'a mut self,
        ctx: &'a mut GraphicsContext,
        pass_action: PassAction,
        view: Mat4,
        projection: Mat4,
    ) -> RenderPass<'a> {
        if self.mesh_manager.needs_rebuild {
            self.rebuild_buffers(ctx);
        }
        ctx.begin_pass(None, pass_action);
        ctx.apply_pipeline(&self.text_pipeline);
        ctx.apply_uniforms(&text_shader::Uniforms { view, projection });
        let bindings = &self.text_bindings;
        ctx.apply_bindings(bindings);
        RenderPass {
            ctx,
            current_batch: &mut self.batch,
            transform_stack: vec![],
            instance_buffer: &mut self.instance_vertex_buffer,
            mesh_manager: &mut self.mesh_manager,
            bindings,
            current_texture: None,
            current_mesh: None,
            quad_mesh: self.quad_mesh,
            cube_mesh: self.cube_mesh,
        }
    }

    pub fn begin_screen_pass<'a>(
        &'a mut self,
        ctx: &'a mut GraphicsContext,
        RenderPassOptions {
            pass_action,
            view_transform,
            projection,
        }: RenderPassOptions,
    ) -> RenderPass<'a> {
        self.begin_pass(ctx, pass_action, view_transform, projection.unwrap(), true)
    }

    pub fn begin_offscreen_pass<'a>(
        &'a mut self,
        ctx: &'a mut GraphicsContext,
        RenderPassOptions {
            pass_action,
            view_transform,
            projection,
        }: RenderPassOptions,
    ) -> RenderPass<'a> {
        let projection = projection.unwrap_or_else(|| {
            glam::Mat4::orthographic_lh(
                0.0,
                self.target_texture.width as _,
                self.target_texture.height as _,
                0.0,
                1.0,
                -1.0,
            )
        });
        self.begin_pass(ctx, pass_action, view_transform, projection, false)
    }
}

// pub trait RenderBatch<T> {
//     fn apply(&mut self);
//     fn add<T>(data: T)
//     fn clear(&mut self);
// }

// TODO add "render params" to compare against
struct RenderBatch {
    next_instance: usize,
    instances: [RawInstanceData; Self::MAX_INSTANCES],
}

impl Default for RenderBatch {
    fn default() -> Self {
        Self {
            next_instance: 0,
            instances: [Default::default(); Self::MAX_INSTANCES],
        }
    }
}

impl RenderBatch {
    pub const MAX_INSTANCES: usize = 128;

    pub fn add(&mut self, instance_data: RawInstanceData) {
        self.instances[self.next_instance] = instance_data;
        self.next_instance += 1;
    }

    #[inline]
    pub fn active_instances(&self) -> &[RawInstanceData] {
        &self.instances[..self.next_instance]
    }

    #[inline]
    pub fn instance_count(&self) -> usize {
        self.next_instance
    }

    #[inline]
    pub fn clear(&mut self) {
        self.next_instance = 0;
    }
}

pub struct RenderPassOptions {
    pass_action: PassAction,
    view_transform: Mat4,
    /// Defaults to orthographic projection with bounds the same as the render target size
    projection: Option<Mat4>,
}

impl Default for RenderPassOptions {
    fn default() -> Self {
        Self {
            pass_action: PassAction::Nothing,
            view_transform: Mat4::IDENTITY,
            projection: None,
        }
    }
}

impl RenderPassOptions {
    pub fn clear(c: Color) -> Self {
        Self::default().with_clear_color(c)
    }

    pub fn clear_depth(d: f32) -> Self {
        Self {
            pass_action: PassAction::Clear {
                color: None,
                depth: Some(d),
                stencil: None,
            },
            ..Default::default()
        }
    }

    pub fn with_clear_color(self, c: Color) -> Self {
        Self {
            pass_action: PassAction::clear_color(c.r, c.g, c.b, c.a),
            ..self
        }
    }

    pub fn with_pass_action(self, pass_action: PassAction) -> Self {
        Self {
            pass_action,
            ..self
        }
    }

    pub fn with_view_transform(self, view_transform: Mat4) -> Self {
        Self {
            view_transform,
            ..self
        }
    }

    pub fn orthographic(w: f32, h: f32) -> Self {
        Self {
            projection: Some(glam::Mat4::orthographic_lh(0.0, w, 0.0, h, 1.0, -1.0)),
            ..Default::default()
        }
    }

    pub fn with_projection(self, projection: Mat4) -> Self {
        Self {
            projection: Some(projection),
            ..self
        }
    }
}

pub struct RenderPass<'a> {
    instance_buffer: &'a mut Buffer,
    ctx: &'a mut GraphicsContext,
    current_batch: &'a mut RenderBatch,
    mesh_manager: &'a mut MeshManager,
    transform_stack: Vec<Mat4>,
    bindings: &'a Bindings,
    current_texture: Option<Texture>,
    current_mesh: Option<MeshRef>,
    // TODO put this in some abstraction layer?
    quad_mesh: MeshRef,
    cube_mesh: MeshRef,
}

impl<'a> Drop for RenderPass<'a> {
    fn drop(&mut self) {
        self.flush();
        self.ctx.end_render_pass();
    }
}

impl<'a> RenderPass<'a> {
    #[inline]
    pub fn flush(&mut self) {
        let mesh = self.current_mesh.unwrap();
        let offsets = self.mesh_manager.offsets(mesh).unwrap();
        self.instance_buffer
            .update(self.ctx, self.current_batch.active_instances());
        self.ctx.draw(
            offsets.offset as _,
            offsets.count as _,
            self.current_batch.instance_count() as _,
        );
        self.current_batch.clear();
    }

    pub fn push_transform(&mut self, m: Mat4) {
        self.transform_stack.push(m);
    }

    pub fn pop_transform(&mut self) {
        debug_assert!(self.transform_stack.pop().is_some());
    }

    pub fn set_texture(&mut self, t: Texture) {
        // TODO if current texture is set and different from t, flush first
        self.current_texture = Some(t);
        let bindings = Bindings {
            images: vec![t],
            ..self.bindings.clone()
        };
        self.ctx.apply_bindings(&bindings);
    }

    pub fn render_mesh<T: ModelTransform>(
        &mut self,
        mesh: MeshRef,
        instance_data: InstanceData<T>,
    ) {
        if self.current_batch.instance_count() == RenderBatch::MAX_INSTANCES
            || self.current_mesh.map_or(false, |m| m != mesh)
        {
            self.flush();
        }
        self.current_mesh = Some(mesh);
        self.current_batch.add(instance_data.into());
    }

    pub fn draw_quad<T: ModelTransform>(&mut self, quad: InstanceData<T>) {
        self.render_mesh(self.quad_mesh, quad);
    }

    pub fn draw_cube<T: ModelTransform>(&mut self, cube: InstanceData<T>) {
        self.render_mesh(self.cube_mesh, cube);
    }

    pub fn draw_sprite_frame(&mut self, pos: Point, s: &Sprite, i: usize) {
        let frame = &s.frames[i];
        let origin = pos - s.pivot.unwrap_or_default();
        let pos = vec2(origin.x as f32, origin.y as f32);
        let scale = vec2(s.size.x as f32, s.size.y as f32);
        self.draw_quad(InstanceData {
            tint: Color::WHITE,
            subtexture: frame.region,
            transform: Transform2D {
                pos,
                scale,
                angle: 0.,
            },
        });
    }
}
