use glam::Vec2;

#[derive(Default, Debug)]
pub struct Cursor {
    position: Vec2,
    last_change: Option<Vec2>,
    delta_this_frame: Option<Vec2>,
}

impl Cursor {
    pub fn position(&self) -> Vec2 {
        self.position
    }

    pub fn update_position(&mut self, new_position: Vec2) -> Option<Vec2> {
        self.last_change = Some(match self.last_change {
            Some(_) => new_position - self.position,
            None => Vec2::ZERO,
        });
        self.delta_this_frame = self.last_change;
        self.position = new_position;
        self.last_change
    }

    pub fn delta(&self) -> Option<Vec2> {
        self.delta_this_frame
    }

    pub fn end_frame_update(&mut self) {
        self.delta_this_frame = None
    }
}
