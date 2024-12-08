use std::borrow::Cow;
use std::io::Read;

use glam::{vec2, vec3, vec4, Mat4, Quat, Vec2, Vec3, Vec4, Vec4Swizzles};
use itertools::Itertools;
use rust_game_engine::app::{AppState, Context};
use rust_game_engine::color::Color;
use rust_game_engine::renderer::geometry::GeometryPass;
use rust_game_engine::renderer::lighting::{Light, ShadowMappingPass, MAX_LIGHTS};
use rust_game_engine::renderer::model::LoadModel;
use rust_game_engine::renderer::state::BoundTexture;
use rust_game_engine::renderer::{MeshRef, RenderTarget, UniformBindGroup, UniformData};
use ttf_parser::Face;
use wgpu::include_wgsl;
use winit::dpi::{PhysicalSize, Size};
use winit::event_loop::EventLoop;
use winit::window::WindowBuilder;

use controlset_derive::ControlSet;
use rust_game_engine::assets::AssetManager;
use rust_game_engine::camera::Camera;
use rust_game_engine::font::FontAtlas;
use rust_game_engine::geom::{BasicVertexData, ModelVertexData, Point};
use rust_game_engine::input::{Axis, Button, ControlSet, Key, Toggle};
use rust_game_engine::renderer::{
    instance::InstanceRenderData,
    mesh::LoadMesh,
    state::{GlobalUniforms, ViewProjectionUniforms},
    text::TextDisplayOptions,
    BasicInstanceData, Display, OffscreenFramebuffer, PipelineRef, RenderData, ScalingMode,
    TextureBuilder, TextureRef,
};
use rust_game_engine::sprite_manager::SpriteManager;
use rust_game_engine::transform::{Transform, Transform2D, Transform3D};

const WINDOW_SIZE: PhysicalSize<u32> = PhysicalSize::new(960, 720);
const RENDER_SCALE: f32 = 1.0;

#[derive(ControlSet)]
struct GameControls {
    #[bind((Key::A, Key::D))]
    x: Axis,
    #[bind((Key::W, Key::S))]
    y: Axis,
    #[bind((Key::F, Key::R))]
    scale: Axis,
    #[bind(Key::Escape)]
    quit: Button,
    #[bind(Key::P)]
    next: Button,
    #[bind(Key::O)]
    add: Button,
    #[bind(Key::Slash)] // aka question mark
    show_help: Toggle,
    #[bind(Key::GraveAccent)]
    debug: Toggle,
}

#[derive(Debug, Default, Clone)]
struct ShaderSource {
    dirty: bool,

    text: String,
}

#[derive(Default)]
struct RenderPipelines {
    text: PipelineRef<BasicVertexData, BasicInstanceData>,
    ssao: PipelineRef<BasicVertexData, BasicInstanceData>,
    ssao_blur: PipelineRef<BasicVertexData, BasicInstanceData>,
    lighting: PipelineRef<BasicVertexData, BasicInstanceData>,
}

#[derive(Default)]
struct GameAssets {
    shader_sources: ShaderSource,
    font_atlas: FontAtlas,
    sprites: SpriteManager,
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct SSAOKernel {
    items: [Vec4; 64],
    count: u32,
    radius: f32,
    bias: f32,
    noise_texture_scale: Vec2,
}

#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct BlurUniforms {
    half_kernel_size: i32,
}

impl Default for BlurUniforms {
    fn default() -> Self {
        Self {
            half_kernel_size: 2,
        }
    }
}

struct State {
    asset_manager: AssetManager<GameAssets>,

    render_pipelines: RenderPipelines,

    // TODO: BitmapFontRenderer
    font_render_data: RenderData<BasicVertexData, BasicInstanceData>,
    sprite_render_data: RenderData<BasicVertexData, BasicInstanceData>,
    offscreen_framebuffer: OffscreenFramebuffer,
    shadow_mapping_pass: ShadowMappingPass,
    geometry_pass: GeometryPass,
    ssao_buffer: TextureRef,
    ssao_buffer_2: TextureRef,
    ssao_kernel: UniformBindGroup<SSAOKernel>,
    ssao_noise_texture_bgl: wgpu::BindGroupLayout,
    ssao_noise_texture: BoundTexture,
    ssao_blur_settings: UniformBindGroup<BlurUniforms>,
    fb_size: Point<u32>,

    // "game" state
    camera: Camera,
    lights: Vec<Light>,
    sprite_instances: Vec<InstanceRenderData>,
    crate_texture: TextureRef,
    cat_texture: TextureRef,
    cube_mesh: MeshRef<ModelVertexData>,

    model_meshes: Vec<(MeshRef<ModelVertexData>, Option<tobj::Material>)>,
}

impl State {
    fn add_sprites(&mut self, display: &Display) {
        let size = display.size_pixels();
        let sprite_ref = self.asset_manager.sprites.get_sprite_ref("guy").unwrap();
        let sprite = self.asset_manager.sprites.get_sprite(sprite_ref);
        for _ in 0..100 {
            self.sprite_instances.push(
                self.sprite_render_data.for_instance(BasicInstanceData {
                    subtexture: sprite.frames[0].region,
                    transform: Transform2D {
                        position: vec2(
                            (rand::random::<u32>() % size.x) as f32,
                            (rand::random::<u32>() % size.y) as f32,
                        ),
                        scale: 4.0 * sprite.size.as_vec2(),
                        ..Default::default()
                    }
                    .as_mat4(),
                    ..Default::default()
                }),
            );
        }
    }

    fn create_or_update_render_pipelines<C: ControlSet>(&mut self, ctx: &mut Context<C>) {
        // TODO: this should probably just move into the renderer, or maybe have a separate text
        // renderer layer
        self.render_pipelines.text = ctx
            .render_state
            .pipeline_builder()
            .with_label("Text Render Pipeline")
            .with_key(self.render_pipelines.text)
            .build(
                ctx.display.device(),
                &ctx.display
                    .device()
                    .create_shader_module(wgpu::ShaderModuleDescriptor {
                        label: Some("text"),
                        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(
                            &self.asset_manager.shader_sources.text,
                        )),
                    }),
            );
        let uniform_bgl = &ctx.render_state.bind_group_layout(
            ctx.display.device(),
            rust_game_engine::renderer::state::BindingType::Uniform,
        );
        self.render_pipelines.ssao = ctx
            .render_state
            .pipeline_builder()
            .with_label("SSAO Pipeline")
            .with_key(self.render_pipelines.ssao)
            .with_extra_bind_group_layouts(vec![
                self.geometry_pass.bind_group_layout(),
                uniform_bgl,
                &self.ssao_noise_texture_bgl,
            ])
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                blend: None,
                format: wgpu::TextureFormat::R16Float,
                write_mask: wgpu::ColorWrites::RED,
            })])
            .with_depth_stencil_state(None)
            .build(
                ctx.display.device(),
                &ctx.display
                    .device()
                    .create_shader_module(include_wgsl!("../../../res/shaders/ssao.wgsl")),
            );
        self.render_pipelines.lighting = ctx
            .render_state
            .pipeline_builder()
            .with_label("Lighting Render Pipeline")
            .with_key(self.render_pipelines.lighting)
            .with_extra_bind_group_layouts(vec![
                self.geometry_pass.bind_group_layout(),
                self.shadow_mapping_pass.bind_group_layout(),
            ])
            .build(
                ctx.display.device(),
                &ctx.display
                    .device()
                    .create_shader_module(include_wgsl!("../../../res/shaders/lighting.wgsl")),
            );
        self.render_pipelines.ssao_blur = ctx
            .render_state
            .pipeline_builder()
            .with_label("SSAO Blur Pipeline")
            .with_key(self.render_pipelines.ssao_blur)
            .with_color_target_states(vec![Some(wgpu::ColorTargetState {
                blend: None,
                format: wgpu::TextureFormat::R16Float,
                write_mask: wgpu::ColorWrites::RED,
            })])
            .with_extra_bind_group_layouts(vec![uniform_bgl])
            .with_depth_stencil_state(None)
            .build(
                ctx.display.device(),
                &ctx.display
                    .device()
                    .create_shader_module(include_wgsl!("../../../res/shaders/blur.wgsl")),
            );
    }
}

impl AppState for State {
    type Controls = GameControls;

    fn new(ctx: &mut Context<Self::Controls>) -> Self {
        let mut asset_manager = AssetManager::new(GameAssets::default(), "./res/");

        let cube_mesh = ctx
            .render_state
            .prepare_mesh(ctx.display.device().load_cube_mesh());

        let crate_texture = ctx.render_state.load_texture(
            &ctx.display,
            ctx.display
                .load_texture_bytes(
                    include_bytes!("../../../res/images/crate.png"),
                    TextureBuilder::labeled("crate"),
                )
                .unwrap(),
        );
        let cat_texture = ctx.render_state.load_texture(
            &ctx.display,
            ctx.display
                .load_texture_bytes(
                    include_bytes!("../../../res/images/sample.png"),
                    TextureBuilder::labeled("sample"),
                )
                .unwrap(),
        );

        asset_manager.track_glob("./res/sprites/*.aseprite", |state, path, f| {
            state.sprites.add_sprite_file(path.to_path_buf(), f);
        });
        let sprite_atlas = ctx.render_state.load_texture(
            &ctx.display,
            TextureBuilder::labeled("sprite_atlas").from_image(
                ctx.display.device(),
                ctx.display.queue(),
                asset_manager.sprites.atlas_image(),
            ),
        );

        // TODO: these callbacks should be able to return an error, optionally
        asset_manager.track_file("./res/fonts/Ubuntu-M.ttf", |state, _, mut f| {
            let mut b = vec![];
            f.read_to_end(&mut b).unwrap();
            let face = Face::parse(&b, 0).unwrap();
            state.font_atlas = FontAtlas::new(face, Default::default()).unwrap();
        });
        let font_atlas_texture = ctx.render_state.load_texture(
            &ctx.display,
            TextureBuilder::labeled("font_atlas")
                .with_filter_mode(wgpu::FilterMode::Linear)
                .from_image(
                    ctx.display.device(),
                    ctx.display.queue(),
                    asset_manager.font_atlas.image(),
                ),
        );

        asset_manager.track_file("./res/shaders/text.wgsl", |state, _, f| {
            state.shader_sources.dirty = true;
            state.shader_sources.text = std::io::read_to_string(f).unwrap();
        });

        // asset_manager.track_file("./res/shaders/model.wgsl", |state, _, f| {
        //     state.shader_sources.dirty = true;
        //     state.shader_sources.model_with_normals = std::io::read_to_string(f).unwrap();
        // });
        //
        let model = ctx
            .display
            .device()
            .load_model("./res/models/astronautB.obj")
            .unwrap();

        let model_meshes = model
            .meshes
            .into_iter()
            .map(|m| (ctx.render_state.prepare_mesh(m.mesh), m.material))
            .collect();

        let fb_size = Point::from((
            (WINDOW_SIZE.width as f32 / RENDER_SCALE) as u32,
            (WINDOW_SIZE.height as f32 / RENDER_SCALE) as u32,
        ));
        let offscreen_framebuffer =
            ctx.render_state
                .create_offscreen_framebuffer(&ctx.display, fb_size, None);

        let sprite_render_data = RenderData {
            pipeline: None,
            texture: sprite_atlas,
            mesh: ctx.render_state.quad_mesh(),
        };
        ctx.set_cursor_captured(true);

        let lighting_pass = ShadowMappingPass::new(&mut ctx.render_state, &ctx.display);
        let geometry_pass = GeometryPass::new(&mut ctx.render_state, &ctx.display, fb_size);

        let mut ssao_kernel = ctx.render_state.create_uniform_bind_group(
            ctx.display.device(),
            SSAOKernel {
                items: std::array::from_fn(|i| {
                    let v = rand::random::<f32>()
                        * vec4(
                            2.0 * rand::random::<f32>() - 1.0,
                            2.0 * rand::random::<f32>() - 1.0,
                            0.85 * rand::random::<f32>() + 0.15,
                            1.0,
                        )
                        .normalize();
                    let scale = rand::random::<f32>() * i as f32 / 64.0;
                    let scale = egui::lerp(0.05f32..=1.0f32, scale * scale);
                    scale * v
                }),
                count: 64,
                radius: 0.5,
                bias: 0.025,
                noise_texture_scale: fb_size.as_vec2() / 4.0,
            },
        );
        let u = *ssao_kernel.uniform();
        ssao_kernel.update(ctx.display.queue(), u);
        let ssao_buffer = ctx.render_state.load_texture(
            &ctx.display,
            TextureBuilder::render_target()
                .with_label("ssao")
                .with_format(wgpu::TextureFormat::R16Float)
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .build(ctx.display.device(), fb_size),
        );
        let ssao_buffer_2 = ctx.render_state.load_texture(
            &ctx.display,
            TextureBuilder::render_target()
                .with_label("ssao")
                .with_format(wgpu::TextureFormat::R16Float)
                .with_usage(
                    wgpu::TextureUsages::TEXTURE_BINDING
                        | wgpu::TextureUsages::COPY_DST
                        | wgpu::TextureUsages::RENDER_ATTACHMENT,
                )
                .build(ctx.display.device(), fb_size),
        );
        let ssao_noise: [Vec4; 16] = std::array::from_fn(|_| {
            vec4(
                2.0 * rand::random::<f32>() - 1.0,
                2.0 * rand::random::<f32>() - 1.0,
                0.0,
                1.0,
            )
        });
        let ssao_noise_texture = TextureBuilder::labeled("ssao_noise")
            .with_usage(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST)
            .with_format(wgpu::TextureFormat::Rgba32Float)
            .with_address_mode(wgpu::AddressMode::Repeat)
            .build(ctx.display.device(), Point::new(4, 4));
        ctx.display.queue().write_texture(
            ssao_noise_texture.texture.as_image_copy(),
            bytemuck::bytes_of(&ssao_noise),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * 4 * 4),
                rows_per_image: Some(4),
            },
            wgpu::Extent3d {
                width: 4,
                height: 4,
                depth_or_array_layers: 1,
            },
        );
        let ssao_noise_texture_bgl =
            ctx.display
                .device()
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: false },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            visibility: wgpu::ShaderStages::FRAGMENT,
                            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                            count: None,
                        },
                    ],
                });
        let ssao_noise_texture = BoundTexture::new(
            ctx.display.device(),
            &ssao_noise_texture_bgl,
            ssao_noise_texture,
        );

        let ssao_blur_settings = ctx
            .render_state
            .create_uniform_bind_group(ctx.display.device(), Default::default());

        let mut state = Self {
            asset_manager,
            crate_texture,
            cat_texture,
            cube_mesh,
            sprite_instances: vec![],
            camera: Camera::new(vec3(0.0, 2.3, 6.0), 960.0 / 720.0),
            lights: vec![],
            // lights: vec![Light {
            //     position: vec4(5.0, 2.5, 7.0, 1.0),
            //     color: Color::WHITE.into(),
            //     proj: Mat4::orthographic_rh(-20.0, 20.0, -20.0, 20.0, 1.0, 24.0),
            //     view: Mat4::look_at_rh(vec3(0.0, 5.0, 0.0), Vec3::ZERO, Vec3::X),
            // }],
            shadow_mapping_pass: lighting_pass,
            geometry_pass,
            ssao_buffer,
            ssao_buffer_2,
            ssao_kernel,
            ssao_blur_settings,
            ssao_noise_texture,
            ssao_noise_texture_bgl,
            font_render_data: Default::default(),
            sprite_render_data,
            offscreen_framebuffer,
            fb_size,
            render_pipelines: Default::default(),
            model_meshes,
        };
        state.create_or_update_render_pipelines(ctx);
        state.font_render_data = RenderData {
            pipeline: Some(state.render_pipelines.text),
            texture: font_atlas_texture,
            mesh: ctx.render_state.quad_mesh(),
        };
        state
    }

    fn update(&mut self, ctx: &mut Context<GameControls>) -> bool {
        if self.asset_manager.check_for_updates() {
            if self.asset_manager.shader_sources.dirty {
                self.asset_manager.shader_sources.dirty = false;
                self.create_or_update_render_pipelines(ctx);
            }
            if self.asset_manager.sprites.maybe_rebuild() {
                ctx.render_state.replace_texture(
                    &ctx.display,
                    self.sprite_render_data.texture,
                    TextureBuilder::labeled("sprite_atlas").from_image(
                        ctx.display.device(),
                        ctx.display.queue(),
                        self.asset_manager.sprites.atlas_image(),
                    ),
                );
            }
        }
        if !ctx.input.debug.on {
            if let Some(delta) = ctx.input.mouse_delta {
                let delta = 0.8 * ctx.frame_timing.delta().as_secs_f32() * delta;
                self.camera.update_angle(delta.x, delta.y);
            }
        }
        self.camera.update_position(
            ctx.frame_timing.delta().as_secs_f32()
                * 10.0
                * vec3(
                    ctx.input.x.value(),
                    ctx.input.scale.value(),
                    ctx.input.y.value(),
                )
                .normalize_or_zero(),
        );
        if ctx.input.add.is_down() {
            self.add_sprites(&ctx.display);
            println!("{}", self.sprite_instances.len());
        }
        if ctx.input.quit.is_down() {
            return false;
        }
        if ctx.input.debug.just_pressed() {
            ctx.set_cursor_captured(!ctx.input.debug.on);
        }
        true
    }

    fn render(&mut self, ctx: &mut Context<GameControls>) -> Result<(), wgpu::SurfaceError> {
        ctx.render_state.global_uniforms.update(
            ctx.display.queue(),
            GlobalUniforms {
                time: ctx.frame_timing.time(),
            },
        );

        let mut scene = vec![
            InstanceRenderData {
                texture: Some(self.crate_texture),
                mesh: self.cube_mesh,
                instance: Default::default(),
                pipeline: None,
            },
            InstanceRenderData {
                mesh: self.cube_mesh,
                instance: BasicInstanceData {
                    tint: vec4(1.0, 0.9, 0.9, 1.0).into(),
                    transform: Transform3D {
                        position: vec3(-1.0, 1.5, -0.4),
                        ..Default::default()
                    }
                    .as_mat4(),
                    ..Default::default()
                },
                texture: None,
                pipeline: None,
            },
            InstanceRenderData {
                mesh: self.cube_mesh,
                instance: BasicInstanceData {
                    tint: vec4(0.88, 0.82, 0.8, 1.0).into(),
                    transform: Transform3D {
                        position: vec3(0.0, -1.5, 0.0),
                        scale: vec3(30.0, 0.5, 30.0),
                        ..Default::default()
                    }
                    .as_mat4(),
                    ..Default::default()
                },
                texture: None,
                pipeline: None,
            },
        ];
        for (mesh, mat) in &self.model_meshes {
            scene.push(InstanceRenderData {
                mesh: *mesh,
                instance: BasicInstanceData {
                    tint: mat.as_ref().map(|m| m.diffuse.into()).unwrap_or_default(),
                    transform: Transform3D {
                        position: vec3(0.0, 2.0, 0.0),
                        ..Default::default()
                    }
                    .as_mat4(),
                    ..Default::default()
                },
                texture: None,
                pipeline: None,
            });
        }

        self.geometry_pass.run(
            &mut ctx.render_state,
            &ctx.display,
            &ViewProjectionUniforms::for_camera(&self.camera),
            &scene,
        );

        let quad = ctx.render_state.quad_mesh();
        ctx.render_state
            .render_pass(
                &ctx.display,
                "SSAO Pass",
                &[RenderTarget::TextureRef(self.ssao_buffer)],
                None,
                &ViewProjectionUniforms::for_camera(&self.camera),
                |r| {
                    r.set_bind_group(3, self.geometry_pass.bind_group().clone());
                    r.set_bind_group(4, self.ssao_kernel.bind_group().clone());
                    r.set_bind_group(5, self.ssao_noise_texture.bind_group().clone());
                    r.draw_instance(&InstanceRenderData {
                        mesh: quad,
                        instance: BasicInstanceData {
                            ..Default::default()
                        },
                        texture: None,
                        pipeline: Some(self.render_pipelines.ssao),
                    });
                },
            )
            .submit();

        ctx.render_state
            .render_pass(
                &ctx.display,
                "SSAO Blur Pass",
                &[RenderTarget::TextureRef(self.ssao_buffer_2)],
                None,
                &ViewProjectionUniforms {
                    // projection: display_view.orthographic_projection(),
                    ..Default::default()
                },
                |r| {
                    r.set_bind_group(3, self.ssao_blur_settings.bind_group().clone());
                    r.draw_instance(&InstanceRenderData {
                        mesh: quad,
                        instance: BasicInstanceData {
                            ..Default::default()
                        },
                        texture: Some(self.ssao_buffer),
                        pipeline: Some(self.render_pipelines.ssao_blur),
                    });
                },
            )
            .submit();

        self.shadow_mapping_pass
            .run(&mut ctx.render_state, &ctx.display, &self.lights, &scene);

        ctx.render_state
            .render_pass(
                &ctx.display,
                "Lighting Pass",
                &[RenderTarget::TextureRef(self.offscreen_framebuffer.color)],
                self.offscreen_framebuffer
                    .depth
                    .map(RenderTarget::TextureRef),
                &ViewProjectionUniforms::for_camera(&self.camera),
                |r| {
                    r.set_bind_group(3, self.geometry_pass.bind_group().clone());
                    r.set_bind_group(4, self.shadow_mapping_pass.bind_group().clone());
                    r.draw_instance(&InstanceRenderData {
                        mesh: quad,
                        instance: Default::default(),
                        texture: Some(self.ssao_buffer_2),
                        pipeline: Some(self.render_pipelines.lighting),
                    });
                },
            )
            .submit();

        let display_view = ctx.display.view()?;
        let mut enc = ctx
            .render_state
            .render_pass(
                &ctx.display,
                "Default Pass",
                &[RenderTarget::TextureView(display_view.view())],
                Some(RenderTarget::TextureView(
                    &display_view.display().depth_texture().view,
                )),
                &ViewProjectionUniforms {
                    projection: display_view.orthographic_projection(),
                    ..Default::default()
                },
                |r| {
                    r.draw_quad(
                        self.offscreen_framebuffer.color,
                        ScalingMode::Centered.view_matrix(
                            self.offscreen_framebuffer.size_pixels().as_vec2(),
                            ctx.display.size_pixels().as_vec2(),
                        ),
                    );
                    for instance in &self.sprite_instances {
                        r.draw_instance(instance);
                    }
                    let text = if ctx.input.show_help.on {
                        ctx.input
                            .status()
                            .map(|(c, bindings, state)| {
                                format!("{:?} : {} = {}", c, bindings.iter().join(", "), state)
                            })
                            .join("\n")
                    } else {
                        format!("{:.2}", ctx.frame_timing.fps())
                    };
                    r.draw_text(
                        &text,
                        &self.asset_manager.font_atlas,
                        &self.font_render_data,
                        Transform2D {
                            position: vec2(20.0, 40.0),
                            ..Default::default()
                        },
                        TextDisplayOptions::default(),
                    );
                },
            )
            .encoder();

        if ctx.input.debug.on {
            ctx.egui.draw(
                &ctx.display,
                &mut enc,
                wgpu::RenderPassColorAttachment {
                    view: display_view.view(),
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                },
                Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &display_view.display().depth_texture().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                |ui| {
                    egui::Window::new("Debug")
                        // .vscroll(true)
                        .default_open(true)
                        // .max_width(2000.0)
                        // .max_height(1200.0)
                        // .default_width(1000.0)
                        .resizable(true)
                        .anchor(egui::Align2::LEFT_TOP, [0.0, 0.0])
                        .show(&ui, |ui| {
                            ui.spacing_mut().slider_width = 200.0;
                            ui.label("Lights");
                            for mut i in 0..(self.lights.len() as usize) {
                                if i > 0 {
                                    ui.separator();
                                }
                                let removed = ui
                                    .horizontal(|ui| {
                                        ui.label(&format!("Light {}", i));
                                        if ui.button("remove").clicked() {
                                            self.lights.remove(i);
                                            i = i.saturating_sub(1);
                                            true
                                        } else {
                                            false
                                        }
                                    })
                                    .inner;
                                if removed {
                                    continue;
                                }
                                let light = &mut self.lights[i];
                                ui.add(
                                    egui::Slider::new(&mut light.position.x, -20.0..=20.0)
                                        .text("x"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut light.position.y, -20.0..=20.0)
                                        .text("y"),
                                );
                                ui.add(
                                    egui::Slider::new(&mut light.position.z, -20.0..=20.0)
                                        .text("z"),
                                );
                                let mut c = egui::Rgba::from_rgba_premultiplied(
                                    light.color.x,
                                    light.color.y,
                                    light.color.z,
                                    light.color.w,
                                );
                                ui.horizontal(|ui| {
                                    ui.label("Color: ");
                                    egui::color_picker::color_edit_button_rgba(
                                        ui,
                                        &mut c,
                                        egui::color_picker::Alpha::OnlyBlend,
                                    );
                                });
                                light.color = c.to_array().into();
                                light.view =
                                    Mat4::look_at_rh(light.position.xyz(), Vec3::ZERO, Vec3::X);
                            }

                            if self.lights.len() < MAX_LIGHTS {
                                if ui.add(egui::Button::new("Add Light")).clicked() {
                                    self.lights.push(Light {
                                        position: vec4(0.0, 5.0, 0.0, 1.0),
                                        color: Color::WHITE.into(),
                                        proj: Mat4::orthographic_rh(
                                            -20.0, 20.0, -20.0, 20.0, 1.0, 24.0,
                                        ),
                                        view: Mat4::look_at_rh(
                                            vec3(0.0, 5.0, 0.0),
                                            Vec3::ZERO,
                                            Vec3::X,
                                        ),
                                    });
                                }
                            }

                            ui.separator();
                            ui.label("SSAO");
                            ui.add(
                                egui::Slider::new(&mut self.ssao_kernel.radius, 0.0..=5.0)
                                    .text("radius"),
                            );
                            ui.add(
                                egui::Slider::new(&mut self.ssao_kernel.bias, 0.0..=2.0)
                                    .text("bias"),
                            );
                            let u = *self.ssao_kernel.uniform();
                            self.ssao_kernel.update(ctx.display.queue(), u);

                            ui.separator();
                            ui.label("Blur");
                            ui.add(
                                egui::Slider::new(
                                    &mut self.ssao_blur_settings.half_kernel_size,
                                    0..=10,
                                )
                                .text("half kernel size"),
                            );
                            let u = *self.ssao_blur_settings.uniform();
                            self.ssao_blur_settings.update(ctx.display.queue(), u);
                        });
                },
            );
        }
        ctx.display.queue().submit([enc.finish()]);

        display_view.present();

        Ok(())
    }
}

fn main() {
    pollster::block_on(async {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_inner_size(Size::new(WINDOW_SIZE))
            .build(&event_loop)?;

        let display = Display::from_window(window).await;
        State::create_app(display).run(event_loop)
    })
    .expect("game crashed");
}
