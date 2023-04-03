use std::time::Duration;

use crate::geom::{Point, Rect};

#[derive(Debug, Clone)]
pub struct Frame {
    pub index: usize,
    pub duration: Duration,
    pub region: Rect,
}

#[derive(Debug, Clone)]
pub struct Animation {
    pub name: String,
    pub frames: std::ops::Range<usize>,
}

#[derive(Debug, Clone, Default)]
pub struct Sprite {
    pub(crate) id: usize,
    pub name: String,
    pub size: Point<u32>,
    pub pivot: Option<Point>,
    pub frames: Vec<Frame>,
    pub animations: Vec<Animation>,
}
