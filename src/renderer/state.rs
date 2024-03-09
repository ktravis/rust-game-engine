use std::{
    ops::{Deref, DerefMut, Range},
    sync::{Arc, Mutex},
};

use glam::Mat4;
use slotmap::{Key, SlotMap};
use wgpu::{BufferDescriptor, BufferUsages};

use super::{
    display::Display,
    instance::{InstanceRenderer, InstanceStorage},
    mesh::{LoadMesh, Mesh},
    texture::{Texture, TextureBuilder},
    BindGroup, MeshRef, ModelInstanceData, OffscreenFramebuffer, PipelineRef, RenderTarget,
    TextureRef, UniformBuffer, UniformData, VertexLayout, DEFAULT_TEXTURE_DATA,
};
use crate::geom::{ModelVertexData, Point};

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
}

impl UniformData for ViewProjectionUniforms {}

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

pub struct BindGroupAllocator<'a> {
    display: &'a Display,
    layout: &'a wgpu::BindGroupLayout,
    bind_groups: &'a Mutex<CachePool<BindGroup<UniformBuffer<ViewProjectionUniforms>>>>,
}

impl<'a> BindGroupAllocator<'a> {
    pub fn get(&self, view_projection: &ViewProjectionUniforms) -> Arc<wgpu::BindGroup> {
        let display = self.display;
        let mut x = self.bind_groups.lock().unwrap();
        let bg = x.get(|| {
            BindGroup::new(
                display.device(),
                &self.layout,
                UniformBuffer::<ViewProjectionUniforms>::new(display.device(), Default::default()),
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

pub struct RenderState {
    pub global_uniforms: BindGroup<UniformBuffer<GlobalUniforms>>,
    quad_mesh: MeshRef,

    instance_storage: InstanceStorage,
    global_uniform_bind_group_layout: wgpu::BindGroupLayout,
    view_projection_uniform_bind_group_layout: wgpu::BindGroupLayout,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    depth_texture_bind_group_layout: wgpu::BindGroupLayout,
    view_proj_bind_groups: Mutex<CachePool<BindGroup<UniformBuffer<ViewProjectionUniforms>>>>,

    texture_manager: SlotMap<TextureRef, BoundTexture>,
    default_texture: BoundTexture,

    mesh_manager: SlotMap<MeshRef, Mesh>,
    pipelines: SlotMap<PipelineRef, wgpu::RenderPipeline>,
    default_pipeline: PipelineRef,
}

impl RenderState {
    pub fn new(display: &Display, default_shader: &wgpu::ShaderModule) -> Self {
        let device = display.device();
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        let depth_texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Depth,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                        count: None,
                    },
                ],
                label: Some("depth_texture_bind_group_layout"),
            });
        let global_uniform_bind_group_layout = UniformBuffer::<GlobalUniforms>::bind_group_layout(
            device,
            "global_uniform_bind_group_layout",
            wgpu::ShaderStages::VERTEX_FRAGMENT,
        );
        let global_uniforms = BindGroup::new(
            device,
            &global_uniform_bind_group_layout,
            UniformBuffer::new(device, Default::default()),
        );
        let view_projection_uniform_bind_group_layout =
            UniformBuffer::<ViewProjectionUniforms>::bind_group_layout(
                device,
                "view_projection_uniform_bind_group_layout",
                wgpu::ShaderStages::VERTEX_FRAGMENT,
            );

        let default_texture = BoundTexture::new(
            display.device(),
            &texture_bind_group_layout,
            TextureBuilder::render_target()
                .with_label("default_texture")
                .from_raw_bytes(
                    display.device(),
                    display.queue(),
                    &DEFAULT_TEXTURE_DATA,
                    Point::new(2, 2),
                ),
        );
        let mut mesh_manager = SlotMap::with_key();
        let quad_mesh = mesh_manager.insert(display.device().load_quad_mesh());
        let instance_storage = InstanceStorage::new(display, 1024);

        let mut s = Self {
            texture_bind_group_layout,
            depth_texture_bind_group_layout,
            global_uniform_bind_group_layout,
            view_projection_uniform_bind_group_layout,
            default_texture,
            texture_manager: SlotMap::with_key(),
            mesh_manager,
            pipelines: SlotMap::with_key(),
            global_uniforms,
            quad_mesh,
            instance_storage,
            view_proj_bind_groups: Default::default(),
            default_pipeline: Default::default(),
        };
        s.default_pipeline = s.create_pipeline_with_key(
            "Default Render Pipeline",
            display,
            default_shader,
            &[
                ModelVertexData::vertex_layout(),
                ModelInstanceData::vertex_layout(),
            ],
            None,
        );
        s
    }

    pub fn quad_mesh(&self) -> MeshRef {
        self.quad_mesh
    }

    pub fn default_pipeline(&self) -> PipelineRef {
        self.default_pipeline
    }

    pub fn create_pipeline<'a>(
        &mut self,
        name: &str,
        display: &Display,
        shader: &wgpu::ShaderModule,
        vertex_layouts: &[wgpu::VertexBufferLayout<'a>],
    ) -> PipelineRef {
        self.create_pipeline_with_key(name, display, shader, vertex_layouts, None)
    }

    pub fn replace_pipeline<'a>(
        &mut self,
        name: &str,
        display: &Display,
        shader: &wgpu::ShaderModule,
        vertex_layouts: &[wgpu::VertexBufferLayout<'a>],
        key: PipelineRef,
    ) -> PipelineRef {
        self.create_pipeline_with_key(name, display, shader, vertex_layouts, Some(key))
    }

    pub fn create_pipeline_with_key<'a>(
        &mut self,
        name: &str,
        display: &Display,
        shader: &wgpu::ShaderModule,
        vertex_layouts: &[wgpu::VertexBufferLayout<'a>],
        key: Option<PipelineRef>,
    ) -> PipelineRef {
        // TODO: do we need/want to dedupe or cache this?
        let layout = display
            .device()
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(&format!("{} Layout", name)),
                bind_group_layouts: &[
                    &self.texture_bind_group_layout,
                    &self.global_uniform_bind_group_layout,
                    &self.view_projection_uniform_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let pipeline = display
            .device()
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(name),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: shader,
                    entry_point: "vs_main",
                    buffers: vertex_layouts,
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
                    front_face: wgpu::FrontFace::Ccw,
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
                *self.pipelines.get_mut(key).unwrap() = pipeline;
                key
            }
            None => self.pipelines.insert(pipeline),
        }
    }

    pub fn create_vertex_buffers<'a, const N: usize>(
        &mut self,
        device: &wgpu::Device,
        vertex_layouts: [wgpu::VertexBufferLayout<'a>; N],
    ) -> [wgpu::Buffer; N] {
        std::array::from_fn(|i| {
            let layout = &vertex_layouts[i];
            device.create_buffer(&BufferDescriptor {
                label: None,                      // TODO: pass a name through
                size: 4096 * layout.array_stride, // TODO: pass the count through as well
                usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
                mapped_at_creation: false,
            })
        })
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
            layout: &self.view_projection_uniform_bind_group_layout,
            bind_groups: &self.view_proj_bind_groups,
        };
        let mut r = InstanceRenderer::new(&alloc, &mut self.instance_storage);
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
            pass.bind_texture_data(&self.default_texture);
            self.instance_storage.render_to(&mut pass);
        }
        self.view_proj_bind_groups.lock().unwrap().reset();
        self.instance_storage.clear();
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
        let layout = if t.is_depth() {
            &self.depth_texture_bind_group_layout
        } else {
            &self.texture_bind_group_layout
        };
        self.texture_manager
            .insert(BoundTexture::new(display.device(), layout, t))
    }

    pub fn get_texture(&self, texture: impl Into<Option<TextureRef>>) -> &BoundTexture {
        texture
            .into()
            .map(|t| self.texture_manager.get(t).unwrap())
            .unwrap_or(&self.default_texture)
    }

    pub fn replace_texture(&mut self, display: &Display, texture_ref: TextureRef, value: Texture) {
        *self.texture_manager.get_mut(texture_ref).unwrap() =
            BoundTexture::new(display.device(), &self.texture_bind_group_layout, value);
    }

    pub fn prepare_mesh(&mut self, mesh: Mesh) -> MeshRef {
        self.mesh_manager.insert(mesh)
    }
}

pub struct RenderPass<'a> {
    pub render_state: &'a RenderState,
    raw_pass: wgpu::RenderPass<'a>,
    active_mesh: Option<MeshRef>,
    active_pipeline: Option<PipelineRef>,
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

    pub fn set_active_pipeline(&mut self, pipeline: impl Into<Option<PipelineRef>>) {
        let pipeline = pipeline.into();
        if pipeline == self.active_pipeline {
            return;
        }
        let p = self
            .render_state
            .pipelines
            .get(pipeline.unwrap_or(self.render_state.default_pipeline))
            .unwrap();
        self.set_pipeline(p);
        self.active_pipeline = pipeline;
    }

    pub fn set_active_mesh(&mut self, mesh: MeshRef) {
        self.active_mesh = Some(mesh);
    }

    pub fn draw_active_mesh(&mut self, instances: Range<u32>) {
        let mesh = self
            .render_state
            .mesh_manager
            .get(self.active_mesh.expect("no active mesh"))
            .unwrap();
        self.raw_pass
            .set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.raw_pass
            .set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        self.raw_pass
            .draw_indexed(0..mesh.num_indices, 0, instances);
    }

    pub fn bind_texture_data(&mut self, texture_data: &'a BoundTexture) {
        self.raw_pass.set_bind_group(
            Self::TEXTURE_BIND_GROUP_INDEX,
            texture_data.bind_group(),
            &[],
        );
    }

    pub fn bind_texture(&mut self, texture: impl Into<Option<TextureRef>>) {
        self.bind_texture_data(self.render_state.get_texture(texture.into()))
    }
}

// pub async fn load_model(
//     file_name: &str,
//     device: &wgpu::Device,
//     queue: &wgpu::Queue,
//     material_layout: &wgpu::BindGroupLayout,
// ) -> anyhow::Result<model::Model> {
//     let obj_text = load_string(file_name).await?;
//     let obj_cursor = Cursor::new(obj_text);
//     let mut obj_reader = BufReader::new(obj_cursor);

//     let (models, obj_materials) = tobj::load_obj_buf_async(
//         &mut obj_reader,
//         &tobj::LoadOptions {
//             triangulate: true,
//             single_index: true,
//             ..Default::default()
//         },
//         |mat_name| async move {
//             let mat_text = load_string(&mat_name).await.unwrap();
//             tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
//         },
//     )
//     .await?;

//     let mut materials = Vec::new();
//     for m in obj_materials? {
//         let diffuse_texture = load_texture(&m.diffuse_texture, false, device, queue).await?;
//         let normal_texture = load_texture(&m.normal_texture, true, device, queue).await?;
//         materials.push(model::Material::new(
//             device,
//             &m.name,
//             diffuse_texture,
//             normal_texture,
//             material_layout,
//         ));
//     }

//     let meshes = models
//         .into_iter()
//         .map(|m| {
//             let mut vertices = (0..m.mesh.positions.len() / 3)
//                 .map(|i| model::ModelVertex {
//                     position: [
//                         m.mesh.positions[i * 3],
//                         m.mesh.positions[i * 3 + 1],
//                         m.mesh.positions[i * 3 + 2],
//                     ],
//                     tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
//                     normal: [
//                         m.mesh.normals[i * 3],
//                         m.mesh.normals[i * 3 + 1],
//                         m.mesh.normals[i * 3 + 2],
//                     ],
//                     tangent: [0.0; 3],
//                     bitangent: [0.0; 3],
//                 })
//                 .collect::<Vec<_>>();

//             let indices = &m.mesh.indices;
//             let mut triangles_included = vec![0; vertices.len()];

//             for c in indices.chunks(3) {
//                 let v0 = vertices[c[0] as usize];
//                 let v1 = vertices[c[1] as usize];
//                 let v2 = vertices[c[2] as usize];

//                 let pos0: cgmath::Vector3<_> = v0.position.into();
//                 let pos1: cgmath::Vector3<_> = v1.position.into();
//                 let pos2: cgmath::Vector3<_> = v2.position.into();

//                 let uv0: cgmath::Vector2<_> = v0.tex_coords.into();
//                 let uv1: cgmath::Vector2<_> = v1.tex_coords.into();
//                 let uv2: cgmath::Vector2<_> = v2.tex_coords.into();

//                 // triangle edges
//                 let delta_pos1 = pos1 - pos0;
//                 let delta_pos2 = pos2 - pos0;

//                 let delta_uv1 = uv1 - uv0;
//                 let delta_uv2 = uv2 - uv0;

//                 let r = 1.0 / (delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x);
//                 let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
//                 // We flip the bitangent to enable right-handed normal
//                 // maps with wgpu texture coordinate system
//                 let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * -r;

//                 // We'll use the same tangent/bitangent for each vertex in the triangle
//                 vertices[c[0] as usize].tangent =
//                     (tangent + cgmath::Vector3::from(vertices[c[0] as usize].tangent)).into();
//                 vertices[c[1] as usize].tangent =
//                     (tangent + cgmath::Vector3::from(vertices[c[1] as usize].tangent)).into();
//                 vertices[c[2] as usize].tangent =
//                     (tangent + cgmath::Vector3::from(vertices[c[2] as usize].tangent)).into();
//                 vertices[c[0] as usize].bitangent =
//                     (bitangent + cgmath::Vector3::from(vertices[c[0] as usize].bitangent)).into();
//                 vertices[c[1] as usize].bitangent =
//                     (bitangent + cgmath::Vector3::from(vertices[c[1] as usize].bitangent)).into();
//                 vertices[c[2] as usize].bitangent =
//                     (bitangent + cgmath::Vector3::from(vertices[c[2] as usize].bitangent)).into();

//                 // Used to average the tangents/bitangents
//                 triangles_included[c[0] as usize] += 1;
//                 triangles_included[c[1] as usize] += 1;
//                 triangles_included[c[2] as usize] += 1;
//             }

//             let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 label: Some(&format!("{:?} Vertex Buffer", file_name)),
//                 contents: bytemuck::cast_slice(&vertices),
//                 usage: wgpu::BufferUsages::VERTEX,
//             });
//             let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
//                 label: Some(&format!("{:?} Index Buffer", file_name)),
//                 contents: bytemuck::cast_slice(&m.mesh.indices),
//                 usage: wgpu::BufferUsages::INDEX,
//             });

//             model::Mesh {
//                 name: file_name.to_string(),
//                 vertex_buffer,
//                 index_buffer,
//                 num_elements: m.mesh.indices.len() as u32,
//                 material: m.mesh.material_id.unwrap_or(0),
//             }
//         })
//         .collect::<Vec<_>>();

//     Ok(model::Model { meshes, materials })
// }
