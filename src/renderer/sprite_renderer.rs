use crate::{
    geom::ModelVertexData,
    renderer::{
        bindings::{Bindable, UniformBuffer},
        instance::BasicInstanceData,
        shader_type::create_shader,
        state::BoundTexture,
        Display, Pipeline, PipelineBuilder, RenderPass, RenderState, Texture,
        ViewProjectionUniforms,
    },
};

pub struct SpriteRenderer {
    pipeline: Pipeline<(ModelVertexData, BasicInstanceData)>,
    sprite_atlas_texture: BoundTexture,
}

impl SpriteRenderer {
    pub fn new(state: &mut RenderState, display: &Display, sprite_atlas: Texture) -> Self {
        let sprite_shader = &create_shader::<(), (ModelVertexData, BasicInstanceData)>(
            display,
            "sprites instanced",
            SPRITE_SHADER.to_string(),
        );
        let sprite_atlas_texture = state.bind_texture(display, sprite_atlas);
        let view_proj_uniform_bgl = UniformBuffer::<ViewProjectionUniforms>::create_layout(
            display.device(),
            "view proj uniform bg layout",
        );
        let pipeline = PipelineBuilder::new("Default Render Pipeline")
            .with_bind_group_layouts(vec![sprite_atlas_texture.layout(), &view_proj_uniform_bgl])
            .build(display.device(), &sprite_shader);
        Self {
            pipeline,
            sprite_atlas_texture,
        }
    }

    pub fn render_batch<'a>(
        &self,
        r: &mut RenderPass<'_, '_>,
        sprites: impl Iterator<Item = &'a BasicInstanceData>,
    ) {
        r.set_bind_group(0, self.sprite_atlas_texture.bind_group(), &[]);
        let mut b = r.draw_instanced_scene(r.render_state.quad_mesh(), &self.pipeline);
        for s in sprites {
            b.add(s);
        }
    }

    pub fn update(&mut self, state: &mut RenderState, display: &Display, sprite_atlas: Texture) {
        self.sprite_atlas_texture = state.bind_texture(display, sprite_atlas)
    }
}

const SPRITE_SHADER: &'static str = crate::wgsl!(
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.tint_color * textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
"#
);
