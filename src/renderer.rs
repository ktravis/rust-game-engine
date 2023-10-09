use glam::{vec2, Mat4, Vec2};
use miniquad::*;
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::mesh_manager::MeshOffsets;
use crate::{
    color::*,
    font::{FontAtlas, LayoutOptions},
    geom::*,
    mesh_manager::GeometryBuffers,
    sprite::Sprite,
    text_shader,
    transform::*,
};

#[derive(Clone)]
pub struct RenderSet {
    shader: Shader,
    pipeline: Pipeline,
    bindings: Bindings,
}

pub trait VertexLayout {
    fn vertex_layout() -> Vec<VertexAttribute>;
}

pub trait InstanceData: Copy + Default + Sized + VertexLayout {}

impl VertexLayout for () {
    fn vertex_layout() -> Vec<VertexAttribute> {
        vec![]
    }
}

impl InstanceData for () {}

#[derive(Debug, Clone, Copy)]
pub struct ModelInstanceData<T: ModelTransform> {
    pub subtexture: Rect,
    pub tint: Color,
    pub transform: T,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct RawInstanceData {
    uv_scale: Vec2,
    uv_offset: Vec2,
    tint: [u8; 4],
    model: Mat4,
}

impl<T: ModelTransform> From<ModelInstanceData<T>> for RawInstanceData {
    fn from(other: ModelInstanceData<T>) -> Self {
        RawInstanceData {
            uv_scale: other.subtexture.dim,
            uv_offset: other.subtexture.pos,
            tint: other.tint.as_u8(),
            model: other.transform.model_transform(),
        }
    }
}

impl<T: ModelTransform> Default for ModelInstanceData<T> {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            tint: Color::WHITE,
            subtexture: Rect::new(0., 0., 1., 1.),
        }
    }
}

impl InstanceData for RawInstanceData {}

impl VertexLayout for RawInstanceData {
    fn vertex_layout() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute::with_buffer("uv_scale", VertexFormat::Float2, 1),
            VertexAttribute::with_buffer("uv_offset", VertexFormat::Float2, 1),
            VertexAttribute::with_buffer("tint", VertexFormat::Byte4, 1),
            VertexAttribute::with_buffer("model", VertexFormat::Mat4, 1),
        ]
    }
}

impl Default for RawInstanceData {
    fn default() -> Self {
        Self {
            uv_scale: Default::default(),
            uv_offset: Default::default(),
            tint: Color::WHITE.as_u8(),
            model: Default::default(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct InstancedModelVertexData {
    geometry_data: ModelVertexData,
    instance_data: RawInstanceData,
}

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
                let display_region = Transform2D {
                    pos: vec2(
                        (target.x - scaled_target_size.x) / 2.0,
                        (target.y - scaled_target_size.y) / 2.0,
                    ),
                    scale: scaled_target_size,
                    ..Default::default()
                };
                display_region.model_transform()
            }
        }
    }
}

#[rustfmt::skip]
const DEFAULT_TEXTURE_DATA: [u8; 16] = [
    255, 0, 255, 255,
    255, 255, 255, 255,
    255, 255, 255, 255,
    255, 0, 255, 255,
];

#[derive(Debug, Copy, Clone)]
pub struct OffscreenFramebuffer {
    pub color: Texture,
    pub depth: Option<Texture>,
    pass: miniquad::RenderPass,
}

impl OffscreenFramebuffer {
    pub fn new(
        ctx: &mut GraphicsContext,
        color: Texture,
        depth: impl Into<Option<Texture>>,
    ) -> Self {
        let depth = depth.into();
        let pass = miniquad::RenderPass::new(ctx, color, depth);
        Self { color, depth, pass }
    }

    pub fn render_pass(&self) -> miniquad::RenderPass {
        self.pass
    }

    pub fn draw_to_screen<I: InstanceData, B: Batcher>(
        &self,
        ctx: &mut GraphicsContext,
        // TODO: types here
        render_pipeline: &mut InstancedRenderPipeline<ModelVertexData, I, B>,
        display_mode: DisplayMode,
    ) {
        self.draw_to(ctx, render_pipeline, display_mode, RenderTarget::Default)
    }

    pub fn draw_to<I: InstanceData, B: Batcher>(
        &self,
        ctx: &mut GraphicsContext,
        // TODO: types here
        render_pipeline: &mut InstancedRenderPipeline<ModelVertexData, I, B>,
        display_mode: DisplayMode,
        render_target: RenderTarget,
    ) {
        let (w, h) = ctx.screen_size();

        let mut quad_mesh = quad::mesh();
        // we need a quad with flipped uv's, because the texture is top-down in memory
        quad_mesh
            .vertices
            .iter_mut()
            .for_each(|v| v.uv.y = 1.0 - v.uv.y);
        let quad_buffers = GeometryBuffers::from_meshes(ctx, &[quad_mesh]);
        let mut render_pass = render_pipeline.begin_pass(
            ctx,
            render_target,
            &quad_buffers,
            Some(vec![self.color]),
            Rc::new(Cell::new(MeshOffsets {
                offset: 0,
                count: quad::INDICES.len() as _,
            })),
            RenderPassOptions {
                pass_action: PassAction::clear_color(0., 0., 0., 0.),
                view_transform: display_mode.scaling_matrix(
                    vec2(self.color.width as f32, self.color.height as f32),
                    vec2(w, h),
                ),
                projection: Some(Mat4::orthographic_lh(0.0, w, h, 0.0, 1.0, -1.0)),
            },
        );
        render_pass.draw_quad(B::Item::default());
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RenderTarget {
    Offscreen(OffscreenFramebuffer),
    Default,
}

impl RenderTarget {
    pub fn render_pass(&self) -> Option<miniquad::RenderPass> {
        match self {
            RenderTarget::Offscreen(target) => Some(target.render_pass()),
            RenderTarget::Default => None,
        }
    }
}

impl Default for RenderTarget {
    fn default() -> Self {
        Self::Default
    }
}

impl From<OffscreenFramebuffer> for RenderTarget {
    fn from(fb: OffscreenFramebuffer) -> Self {
        Self::Offscreen(fb)
    }
}

#[derive(Clone)]
pub struct InstancedRenderPipeline<V: VertexData, I: InstanceData, B: Batcher = RenderBatch<I>> {
    // This will be a shader abstraction that shares the same vertex layout
    shader: Shader, /*<V, I>*/
    raw_pipeline: Pipeline,
    instance_vertex_buffer: Buffer,
    pipeline_params: PipelineParams,
    // TODO: remove this? reuse?
    batch: B,
    default_texture: Texture,
    _marker: PhantomData<V>,
    _marker2: PhantomData<I>,
}

pub type BasicRenderPipeline = InstancedRenderPipeline<ModelVertexData, (), NoopBatcher>;

impl<V: VertexData, I: InstanceData, B: Batcher> InstancedRenderPipeline<V, I, B> {
    pub fn new(ctx: &mut GraphicsContext, shader: Shader) -> Self {
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

        let instance_vertex_buffer = Buffer::stream(
            ctx,
            BufferType::VertexBuffer,
            128 * std::mem::size_of::<I>(),
        );

        let mut default_vertex_attributes = V::vertex_layout();
        default_vertex_attributes.extend(I::vertex_layout());

        // TODO: pass this in
        let pipeline_params = PipelineParams {
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
            cull_face: CullFace::Back,
            // primitive_type: miniquad::PrimitiveType::Lines,
            ..Default::default()
        };

        let mut buffer_layouts = vec![BufferLayout::default()];
        if I::vertex_layout().len() > 0 {
            buffer_layouts.push(BufferLayout {
                step_func: VertexStep::PerInstance,
                ..Default::default()
            });
        }

        let raw_pipeline = Pipeline::with_params(
            ctx,
            &buffer_layouts,
            &default_vertex_attributes,
            shader,
            pipeline_params,
        );

        Self {
            shader,
            raw_pipeline,
            instance_vertex_buffer,
            pipeline_params,
            batch: Default::default(),
            default_texture,
            _marker: Default::default(),
            _marker2: Default::default(),
        }
    }

    pub fn begin_pass<'a>(
        &'a mut self,
        ctx: &'a mut GraphicsContext,
        render_target: RenderTarget,
        geometry_buffers: &GeometryBuffers<V>,
        textures: Option<Vec<Texture>>,
        quad_mesh: Rc<Cell<MeshOffsets>>,
        RenderPassOptions {
            pass_action,
            view_transform,
            projection,
        }: RenderPassOptions,
    ) -> RenderPass<'a, B> {
        let projection = projection.unwrap_or_else(|| {
            let RenderTarget::Offscreen(o) = render_target else {
                panic!("what");
            };
            Mat4::orthographic_lh(0.0, o.color.width as _, o.color.height as _, 0.0, 1.0, -1.0)
        });
        ctx.begin_pass(render_target.render_pass(), pass_action);
        ctx.apply_pipeline(&self.raw_pipeline);
        ctx.apply_uniforms(&text_shader::Uniforms {
            view: view_transform,
            projection,
        });
        let images = textures.unwrap_or_else(|| vec![self.default_texture]);
        let mut vertex_buffers = vec![geometry_buffers.vertices];
        if I::vertex_layout().len() > 0 {
            vertex_buffers.push(self.instance_vertex_buffer);
        }

        let bindings = Bindings {
            vertex_buffers,
            index_buffer: geometry_buffers.indices,
            images,
        };
        ctx.apply_bindings(&bindings);
        RenderPass {
            ctx,
            current_batch: Default::default(),
            transform_stack: vec![],
            instance_buffer: self.instance_vertex_buffer,
            bindings,
            current_texture: None,
            current_mesh: None,
            quad_mesh,
        }
    }
}

pub trait Batcher: Default {
    type Item: InstanceData;
    fn add(&mut self, instance_data: Self::Item);
    fn active_instances(&self) -> &[Self::Item];
    fn instance_count(&self) -> usize;
    fn full(&self) -> bool;
    fn clear(&mut self);
}

#[derive(Debug, Copy, Clone, Default)]
pub struct NoopBatcher(bool);

impl Batcher for NoopBatcher {
    type Item = ();

    fn add(&mut self, _instance_data: Self::Item) {
        self.0 = true;
    }

    #[inline]
    fn active_instances(&self) -> &[Self::Item] {
        if self.0 {
            &[()]
        } else {
            &[]
        }
    }

    #[inline]
    fn instance_count(&self) -> usize {
        if self.0 {
            1
        } else {
            0
        }
    }

    #[inline]
    fn full(&self) -> bool {
        self.0
    }

    #[inline]
    fn clear(&mut self) {
        self.0 = false;
    }
}

// TODO add "render params" to compare against
#[derive(Clone)]
pub struct RenderBatch<I: InstanceData = RawInstanceData, const MAX_INSTANCES: usize = 128>
where
    I: Sized,
{
    next_instance: usize,
    instances: [I; MAX_INSTANCES],
}

impl<I: InstanceData, const N: usize> Default for RenderBatch<I, N> {
    fn default() -> Self {
        Self {
            next_instance: 0,
            instances: [Default::default(); N],
        }
    }
}

impl<I: InstanceData, const N: usize> Batcher for RenderBatch<I, N> {
    type Item = I;

    fn add(&mut self, instance_data: I) {
        self.instances[self.next_instance] = instance_data;
        self.next_instance += 1;
    }

    #[inline]
    fn active_instances(&self) -> &[I] {
        &self.instances[..self.next_instance]
    }

    #[inline]
    fn instance_count(&self) -> usize {
        self.next_instance
    }

    #[inline]
    fn full(&self) -> bool {
        self.next_instance == N
    }

    #[inline]
    fn clear(&mut self) {
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
            projection: Some(Mat4::orthographic_lh(0.0, w, 0.0, h, 1.0, -1.0)),
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

pub struct RenderPass<'a, B: Batcher = RenderBatch> {
    ctx: &'a mut GraphicsContext,
    instance_buffer: Buffer,
    current_batch: B,
    transform_stack: Vec<Mat4>,
    bindings: Bindings,
    current_texture: Option<Texture>,
    current_mesh: Option<Rc<Cell<MeshOffsets>>>,
    // TODO put this in some abstraction layer?
    quad_mesh: Rc<Cell<MeshOffsets>>,
}

impl<'a, B: Batcher> Drop for RenderPass<'a, B> {
    fn drop(&mut self) {
        self.flush();
        self.ctx.end_render_pass();
    }
}

impl<'a, B: Batcher> RenderPass<'a, B> {
    #[inline]
    pub fn flush(&mut self) {
        let mesh = self.current_mesh.clone().unwrap().get();
        let active_instances = self.current_batch.active_instances();
        self.instance_buffer.update(self.ctx, active_instances);
        self.ctx.draw(
            mesh.offset as _,
            mesh.count as _,
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
        if self.current_texture.is_some_and(|x| x != t) {
            self.flush();
        }
        self.current_texture = Some(t);
        let bindings = Bindings {
            images: vec![t],
            ..self.bindings.clone()
        };
        self.ctx.apply_bindings(&bindings);
    }

    pub fn render_mesh(&mut self, mesh: Rc<Cell<MeshOffsets>>, instance_data: impl Into<B::Item>) {
        if self.current_batch.full() || self.current_mesh.clone().map_or(false, |m| m != mesh) {
            self.flush();
        }
        self.current_mesh = Some(mesh);
        self.current_batch.add(instance_data.into());
    }

    pub fn draw_quad(&mut self, quad: impl Into<B::Item>) {
        self.render_mesh(self.quad_mesh.clone(), quad);
    }
}

impl<'a, B> RenderPass<'a, B>
where
    B: Batcher,
    B::Item: From<ModelInstanceData<Transform2D>>,
{
    pub fn draw_sprite_frame(&mut self, pos: Vec2, s: &Sprite, i: usize) {
        let frame = &s.frames[i];
        let origin = pos - s.pivot.unwrap_or_default().as_vec2();
        let pos = origin;
        let scale = vec2(s.size.x as f32, s.size.y as f32);
        self.draw_quad(ModelInstanceData {
            tint: Color::WHITE,
            subtexture: frame.region,
            transform: Transform2D {
                pos,
                scale,
                angle: 0.,
            },
        });
    }

    pub fn draw_text(&mut self, pos: Vec2, s: &str, font: &FontAtlas, opts: TextDisplayOptions) {
        for glyph_data in font.layout_text(&s, opts.layout) {
            self.draw_quad(ModelInstanceData::<Transform2D> {
                transform: Transform2D {
                    pos: glyph_data.bounds.pos + pos,
                    scale: glyph_data.bounds.dim,
                    angle: 0.,
                },
                subtexture: glyph_data.subtexture,
                tint: opts.color,
            });
        }
    }
}

#[derive(Clone, Copy, Default, Debug)]
pub struct TextDisplayOptions {
    pub color: Color,
    pub layout: LayoutOptions,
}
