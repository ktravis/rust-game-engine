use glam::{vec2, Quat};

use crate::{
    sprite_manager::{SpriteManager, SpriteRef},
    transform::Transform,
};

use super::{instance::DrawInstance, ModelInstanceData, RenderData};

pub trait MakeSpriteRenderer: Sized + DrawInstance {
    fn sprite_renderer<'a>(
        &'a mut self,
        sprite_manager: &'a SpriteManager,
        render_data: RenderData,
    ) -> SpriteRenderer<'a, Self>;
}

impl<T: DrawInstance> MakeSpriteRenderer for T {
    fn sprite_renderer<'a>(
        &'a mut self,
        sprite_manager: &'a SpriteManager,
        render_data: RenderData,
    ) -> SpriteRenderer<'a, T> {
        SpriteRenderer {
            raw: self,
            sprite_manager,
            render_data,
        }
    }
}

pub struct SpriteRenderer<'a, T: DrawInstance> {
    raw: &'a mut T,
    sprite_manager: &'a SpriteManager,
    render_data: RenderData,
}

impl<'a, T: DrawInstance> SpriteRenderer<'a, T> {
    pub fn draw_sprite(&mut self, sprite: SpriteRef, frame: usize, transform: Transform) {
        let s = self.sprite_manager.get_sprite(sprite);
        let frame = &s.frames[frame];
        let scale = vec2(s.size.x as f32, s.size.y as f32);
        let origin = s.pivot.unwrap_or_default().as_vec2();
        let transform = Transform::from_matrix(
            transform.as_matrix()
                * Transform {
                    position: -1.0 * origin.extend(0.),
                    scale: scale.extend(1.),
                    rotation: Quat::IDENTITY,
                }
                .as_matrix(),
        );
        self.raw
            .draw_instance(&self.render_data.for_model_instance(ModelInstanceData {
                subtexture: frame.region,
                transform,
                ..Default::default()
            }));
    }
}
