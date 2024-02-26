use glam::{vec2, vec3, Mat4, Quat};

use crate::{
    sprite_manager::{SpriteManager, SpriteRef},
    transform::{Transform, Transform3D},
};

use super::{
    instance::{DrawInstance, InstanceRenderData},
    ModelInstanceData, RenderData,
};

// pub trait MakeSpriteRenderer: Sized + DrawInstance {
//     fn sprite_renderer<'a>(
//         &'a mut self,
//         sprite_manager: &'a SpriteManager,
//         render_data: RenderData,
//     ) -> SpriteRenderer<'a, Self>;
// }
//
// impl<T: DrawInstance> MakeSpriteRenderer for T {
//     fn sprite_renderer<'a>(
//         &'a mut self,
//         sprite_manager: &'a SpriteManager,
//         render_data: RenderData,
//     ) -> SpriteRenderer<'a, T> {
//         SpriteRenderer {
//             raw: self,
//             sprite_manager,
//             render_data,
//         }
//     }
// }

pub struct SpriteRenderer<'a> {
    pub sprite_manager: &'a SpriteManager,
    pub render_data: RenderData,
}

impl<'a> SpriteRenderer<'a> {
    #[inline]
    pub fn draw_sprite(
        &self,
        sprite: SpriteRef,
        frame: usize,
        transform: impl Transform,
    ) -> InstanceRenderData<Mat4> {
        let s = self.sprite_manager.get_sprite(sprite);
        let frame = &s.frames[frame];
        let scale = vec3(s.size.x as f32, s.size.y as f32, 1.0);
        let origin = s.pivot.unwrap_or_default().as_vec2();
        let transform = transform.as_mat4()
            * Mat4::from_scale_rotation_translation(scale, Quat::IDENTITY, -origin.extend(0.0));
        self.render_data.for_model_instance(ModelInstanceData {
            subtexture: frame.region,
            transform,
            ..Default::default()
        })
    }
}
