use std::{
    collections::HashMap,
    marker::PhantomData,
    num::NonZeroU32,
    ops::{Deref, DerefMut, Range},
};

use bytemuck::Zeroable;
use glam::{Mat3A, Mat4, Quat, Vec2, Vec4};
use shadertype_derive::shader_uniform_type;
use wgpu::util::DeviceExt;

use super::{
    display::Display,
    instance::InstanceStorage,
    shader_type::GlobalUniforms,
    text::{RenderableFont, TextDisplayOptions},
    texture::{Texture, TextureBuilder},
    Pipeline, PipelineBuilder, DEFAULT_TEXTURE_DATA,
};
use crate::{
    color::Color,
    geom::{quad, ModelVertexData, Point, Rect},
    renderer::{
        bindings::{
            create_uniform_bind_group, texture_bgl_entries, BindGroup, DepthTextureView,
            UniformBindGroup, UNIFORM_BGL_ENTRY,
        },
        instance::BasicInstanceData,
        shader_type::{create_shader, VertexInput, VertexInputs},
    },
    transform::Transform,
};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct MeshRef {
    pub base_vertex: i32,
    pub index_start: u32,
    pub index_end: u32,
}

impl MeshRef {
    pub fn indices(&self) -> Range<u32> {
        self.index_start..self.index_end
    }
}

pub type BoundTexture = BindGroup<Texture>;

#[derive(Debug, Default)]
#[shader_uniform_type]
pub struct ModelData {
    pub uv_scale: Vec2,
    pub uv_offset: Vec2,
    pub tint: Vec4,
    pub transform: Mat4,
    pub normal_matrix: Mat3A,
    pub material: u32,
}

const DEFAULT_SHADER: &'static str = crate::wgsl!(
    r#"
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> projection: mat4x4<f32>;

var<immediate> model_data: ModelData;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
}

@vertex
fn vs_main(
    vertex: ModelVertexData,
) -> VertexOutput {
    let model = model_data.transform * vertex.position;
    var out: VertexOutput;
    out.tex_coords = model_data.uv_offset + model_data.uv_scale * vertex.tex_coords;
    out.clip_position = projection * model;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return model_data.tint * textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
"#
);

const TEXT_SHADER: &'static str = crate::wgsl!(
    r#"
@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;

@group(1) @binding(0)
var<uniform> projection: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) screen_pos: vec2<f32>,
    @location(2) tint_color: vec4<f32>,
}

@vertex
fn vs_main(
    vertex: ModelVertexData,
    instance: BasicInstanceData,
) -> VertexOutput {
    let model_transform = mat4x4<f32>(
        instance.transform_1,
        instance.transform_2,
        instance.transform_3,
        instance.transform_4,
    );
    var out: VertexOutput;
    out.tex_coords = instance.subtexture_offset + instance.subtexture_scale * vertex.tex_coords;
    let model = model_transform * vertex.position;
    out.clip_position = projection * model;
    out.screen_pos = model.xy;
    out.tint_color = instance.tint;
    return out;
}

fn median(r: f32, g: f32, b: f32) -> f32 {
    return max(min(r, g), min(max(r, g), b));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let msd = textureSample(t_diffuse, s_diffuse, in.tex_coords);
    let sd = median(msd.r, msd.g, msd.b);
    let w = fwidth(sd) * 0.5;
    let opacity = smoothstep(0.5 - w, 0.5 + w, sd);
    // if (opacity == 0.0) {
    //     discard;
    // }
    return vec4(in.tint_color.x, in.tint_color.y, in.tint_color.z, opacity * in.tint_color.w);
}
"#
);

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
    quad_mesh: MeshRef,

    instance_storage: InstanceStorage,

    texture_bind_group_layouts: HashMap<wgpu::TextureFormat, wgpu::BindGroupLayout>,
    default_texture: BoundTexture,

    scene_verts: Vec<ModelVertexData>,
    scene_indices: Vec<u16>,
    scene_vertex_buffer: Option<wgpu::Buffer>,
    scene_index_buffer: Option<wgpu::Buffer>,

    default_pipeline: Pipeline<ModelVertexData>,
    text_pipeline: Pipeline<(ModelVertexData, BasicInstanceData)>,

    textures: Vec<Texture>,
    texture_array: Option<wgpu::BindGroup>,
    // TODO: annoying to make this pub
    pub texture_array_bgl: Option<wgpu::BindGroupLayout>,
}

impl RenderState {
    const MAX_COLOR_ATTACHMENTS: usize = 8;

    pub fn new(display: &Display) -> Self {
        let device = display.device();

        let global_uniforms = create_uniform_bind_group(device, GlobalUniforms::zeroed());
        let instance_storage = InstanceStorage::new(display, 1024);

        let view_proj_uniform_bgl =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("view proj uniform bg layout"),
                entries: &[UNIFORM_BGL_ENTRY],
            });
        let main_texture_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("main texture"),
            entries: &texture_bgl_entries(TextureBuilder::DEFAULT_FORMAT),
        });
        let default_shader = &create_shader::<ModelData, ModelVertexData>(
            display,
            "default",
            DEFAULT_SHADER.to_string(),
        );
        let default_pipeline = PipelineBuilder::new("default pipeline")
            .with_bind_group_layouts(vec![&main_texture_bgl, &view_proj_uniform_bgl])
            .with_immediate_size(std::mem::size_of::<ModelData>() as u32)
            .build(display.device(), &default_shader);
        let text_shader = &create_shader::<(), (ModelVertexData, BasicInstanceData)>(
            display,
            "text",
            TEXT_SHADER.to_string(),
        );
        let text_pipeline = PipelineBuilder::new("Text Render Pipeline")
            .with_bind_group_layouts(vec![&main_texture_bgl, &view_proj_uniform_bgl])
            .build(display.device(), &text_shader);

        let mut scene_verts = vec![];
        let mut scene_indices = vec![];
        let quad_mesh = MeshRef {
            base_vertex: 0,
            index_start: 0,
            index_end: quad::INDICES.len() as u32,
        };
        scene_verts.extend_from_slice(&quad::verts(0., 0., 1., 1., (0., 0.), (1., 1.)));
        scene_indices.extend_from_slice(&quad::INDICES);

        let t = TextureBuilder::default()
            .with_label("default_texture")
            .from_raw_bytes(
                display.device(),
                display.queue(),
                &DEFAULT_TEXTURE_DATA,
                Point::new(2, 2),
            );
        let default_texture = {
            let bgl = display
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some(&format!("texture({:?})", t.format())),
                    entries: &texture_bgl_entries(t.format()),
                });
            BoundTexture::for_texture(display.device(), &bgl, t.clone())
        };

        Self {
            global_uniforms,
            instance_storage,
            texture_bind_group_layouts: Default::default(),
            quad_mesh,
            scene_verts,
            scene_indices,
            scene_vertex_buffer: None,
            scene_index_buffer: None,
            default_pipeline,
            text_pipeline,
            default_texture,
            textures: vec![t],
            texture_array: None,
            texture_array_bgl: None,
        }
    }

    pub fn quad_mesh(&self) -> MeshRef {
        self.quad_mesh
    }

    #[must_use]
    pub fn render_pass<'a>(
        &mut self,
        display: &'a Display,
        name: &str,
        // TODO: this could be a tuple of typed TextureView's instead, which would allow them to be
        // different formats with validation on pass outputs?
        color_targets: &[&wgpu::TextureView],
        depth_target: Option<&DepthTextureView>,
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

        let mut encoder =
            display
                .device()
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some(&format!("Render Pass Encoder({name})")),
                });
        {
            let color_attachments: [Option<wgpu::RenderPassColorAttachment>;
                Self::MAX_COLOR_ATTACHMENTS] = std::array::from_fn(|i| {
                color_targets
                    .get(i)
                    .map(|view| wgpu::RenderPassColorAttachment {
                        view,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                        resolve_target: None,
                        depth_slice: None,
                    })
            });
            let raw_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some(name),
                color_attachments: &color_attachments[..color_targets.len()],
                depth_stencil_attachment: depth_target.map(|target| {
                    wgpu::RenderPassDepthStencilAttachment {
                        view: target.raw(),
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
        }
        PartialRenderPass { display, encoder }
    }

    pub fn add_texture(&mut self, t: Texture) -> u32 {
        if self.textures.len() > 0 {
            assert_eq!(self.textures[0].format(), t.format());
        }
        self.textures.push(t);
        (self.textures.len() - 1) as u32
    }

    pub fn upload_textures(&mut self, display: &Display) {
        assert_eq!(self.texture_array, None, "texture array already bound");
        assert!(self.textures.len() > 0, "no added textures to commit");
        let format = self.textures[0].format();
        let bgl_entries = {
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
                    count: NonZeroU32::new(self.textures.len() as _),
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
                    count: NonZeroU32::new(self.textures.len() as _),
                },
            ]
        };
        let layout = display
            .device()
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some(&format!("texture array({format:?})")),
                entries: &bgl_entries,
            });
        let (views, samplers): (Vec<_>, Vec<_>) =
            self.textures.iter().map(|t| (&t.view, &t.sampler)).unzip();
        let _ = self
            .texture_array
            .insert(
                display
                    .device()
                    .create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some(&format!("bind group (texture array({format:?})")),
                        layout: &layout,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: wgpu::BindingResource::TextureViewArray(&views),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: wgpu::BindingResource::SamplerArray(&samplers),
                            },
                        ],
                    }),
            );
        let _ = self.texture_array_bgl.insert(layout);
    }

    pub fn texture_array_bind_group(&self) -> &wgpu::BindGroup {
        self.texture_array
            .as_ref()
            .expect("texture array bindgroup not created")
    }

    pub fn bind_texture(&mut self, display: &Display, t: Texture) -> BoundTexture {
        let bgl = self
            .texture_bind_group_layouts
            .entry(t.format())
            .or_insert_with(|| {
                display
                    .device()
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some(&format!("texture({:?})", t.format())),
                        entries: &texture_bgl_entries(t.format()),
                    })
            });
        BoundTexture::for_texture(display.device(), bgl, t)
    }

    pub fn add_mesh_data(&mut self, vertices: &[ModelVertexData], indices: &[u16]) -> MeshRef {
        assert!(
            self.scene_vertex_buffer.is_none() && self.scene_index_buffer.is_none(),
            "scene mesh buffers already initialized"
        );
        let r = MeshRef {
            base_vertex: self.scene_verts.len() as i32,
            index_start: self.scene_indices.len() as u32,
            index_end: (self.scene_indices.len() + indices.len()) as u32,
        };
        self.scene_verts.extend_from_slice(&vertices);
        self.scene_indices.extend_from_slice(&indices);
        r
    }

    pub fn upload_meshes(&mut self, display: &Display) {
        assert!(
            self.scene_vertex_buffer.is_none(),
            "scene vertex buffer already initialized"
        );
        self.scene_vertex_buffer = Some(display.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&self.scene_verts),
                usage: wgpu::BufferUsages::VERTEX,
            },
        ));
        assert!(
            self.scene_index_buffer.is_none(),
            "scene index buffer already initialized"
        );
        self.scene_index_buffer = Some(display.device().create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&self.scene_indices),
                usage: wgpu::BufferUsages::INDEX,
            },
        ));
    }

    pub fn default_texture(&self) -> &wgpu::BindGroup {
        self.default_texture.bind_group()
    }
}

pub struct RenderPass<'a, 'p> {
    pub render_state: &'a mut RenderState,
    display: &'p Display,
    raw_pass: wgpu::RenderPass<'p>,

    current_draw_range: Range<u32>,
    scene_mesh_set: bool,
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
            current_draw_range: 0..0,
            scene_mesh_set: false,
        }
    }

    pub fn draw_scene_mesh(&mut self, mesh: MeshRef, instances: Range<u32>) {
        if !self.scene_mesh_set {
            self.raw_pass.set_vertex_buffer(
                0,
                self.render_state
                    .scene_vertex_buffer
                    .as_ref()
                    .unwrap()
                    .slice(..),
            );
            self.raw_pass.set_index_buffer(
                self.render_state
                    .scene_index_buffer
                    .as_ref()
                    .unwrap()
                    .slice(..),
                wgpu::IndexFormat::Uint16,
            );
            self.scene_mesh_set = true;
        }
        self.raw_pass
            .draw_indexed(mesh.indices(), mesh.base_vertex, instances);
    }

    pub fn set_pipeline<V: VertexInputs>(&mut self, pipeline: &Pipeline<V>) {
        self.raw_pass.set_pipeline(pipeline.raw());
    }

    #[inline]
    pub fn draw_quad<'b>(
        &mut self,
        texture: impl Into<Option<&'b wgpu::BindGroup>>,
        transform: impl Transform,
        c: Color,
        subtexture: Rect,
    ) {
        let texture = texture
            .into()
            .unwrap_or(self.render_state.default_texture());
        let transform = transform.as_mat4();
        let mesh = self.render_state.quad_mesh;
        self.raw_pass
            .set_pipeline(self.render_state.default_pipeline.raw());
        self.raw_pass.set_bind_group(0, texture, &[]);
        self.raw_pass.set_immediates(
            0,
            bytemuck::bytes_of(&ModelData {
                uv_scale: subtexture.dim,
                uv_offset: subtexture.pos,
                tint: c.into(),
                transform,
                normal_matrix: Mat3A::IDENTITY,
                material: 0,
            }),
        );
        self.draw_scene_mesh(mesh, 0..1);
    }

    #[inline]
    pub fn draw_text(
        &mut self,
        font: &RenderableFont,
        s: impl AsRef<str>,
        transform: impl Transform,
        opts: TextDisplayOptions,
    ) {
        self.raw_pass.set_bind_group(
            Self::TEXTURE_BIND_GROUP_INDEX,
            font.texture().bind_group(),
            &[],
        );
        let m = transform.as_mat4();
        let p = &self.render_state.text_pipeline.clone();
        let mut batch = self.draw_instanced_scene(self.render_state.quad_mesh, p);
        for glyph_data in font.layout_text(s.as_ref(), opts.layout) {
            let transform = m * Mat4::from_scale_rotation_translation(
                glyph_data.bounds.dim.extend(1.0),
                Quat::IDENTITY,
                glyph_data.bounds.pos.extend(0.0),
            );
            batch.add(&BasicInstanceData {
                subtexture: glyph_data.subtexture,
                tint: opts.color,
                transform,
                ..Default::default()
            });
        }
    }

    pub fn set_scene_buffers(&mut self) {
        // XXX: temp
        let b = self.render_state.scene_vertex_buffer.clone().unwrap();
        self.set_vertex_buffer(0, b.slice(..));
        let b = self.render_state.scene_index_buffer.clone().unwrap();
        self.set_index_buffer(b.slice(..), wgpu::IndexFormat::Uint16);
        self.scene_mesh_set = true;
    }

    // TODO: we could require I: InstanceInput or something and have that set step mode?
    // TODO: this doesn't really *need* to set the pipeline, it's mostly enforcing that V and I are
    // all aligned. Could potentially build the pipeline into the render_pass creation, so that
    // it's typed from the start (and then only something with (V, I: InstanceInput) would even
    // implement the instanced drawing functions
    pub fn draw_instanced_scene<'r, V: VertexInput, I: VertexInput>(
        &'r mut self,
        mesh: MeshRef,
        // TODO: split the association of the pipeline out into another wrapper method on
        // RenderPass (maybe ::use_pipeline)?
        pipeline: &Pipeline<(V, I)>,
    ) -> Batcher<'r, 'a, 'p, V, I> {
        self.raw_pass.set_pipeline(pipeline.raw());
        if !self.scene_mesh_set {
            self.set_scene_buffers();
        }
        let draw_range_start = self.current_draw_range.start;
        Batcher {
            render_pass: self,
            // instance_storage: &mut self.render_state.instance_storage,
            // display: self.display,
            // raw_pass: &mut self.raw_pass,
            draw_count: 0,
            draw_range_start,
            base_vertex: mesh.base_vertex,
            mesh_indices: mesh.indices(),
            _marker: PhantomData,
        }
    }
}

pub struct Batcher<'r, 'x: 'r, 'y: 'r, V: VertexInput, I: VertexInput> {
    render_pass: &'r mut RenderPass<'x, 'y>,
    draw_count: u32,
    draw_range_start: u32,
    base_vertex: i32,
    mesh_indices: Range<u32>,
    _marker: PhantomData<(V, I)>,
}

impl<'r, 'x: 'r, 'y: 'r, V: VertexInput, I: VertexInput> Batcher<'r, 'x, 'y, V, I> {
    pub fn add(&mut self, instance: &I) {
        self.render_pass.render_state.instance_storage.add(instance);
        self.draw_count += 1;
    }

    fn flush(&mut self) {
        if self.draw_count == 0 {
            return;
        }
        // TODO: make this instance range calculation internal to the instance storage
        self.render_pass.current_draw_range.start += self.draw_count;
        self.render_pass
            .render_state
            .instance_storage
            .update_buffer(self.render_pass.display);
        let buf = self
            .render_pass
            .render_state
            .instance_storage
            .buffer()
            .clone();
        // NOTE: this has to be done once we know the size of the buffer
        self.render_pass.set_vertex_buffer(1, buf.slice(..));
        self.render_pass.draw_indexed(
            self.mesh_indices.clone(),
            self.base_vertex,
            self.draw_range_start..(self.draw_range_start + self.draw_count),
        );
    }
}

impl<'r, 'x: 'r, 'y: 'r, V: VertexInput, I: VertexInput> Drop for Batcher<'r, 'x, 'y, V, I> {
    fn drop(&mut self) {
        self.flush();
    }
}
