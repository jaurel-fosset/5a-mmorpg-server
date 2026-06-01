use super::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct Position
{
    pub x: f32,
    pub y: f32,
}

impl Position
{
    pub fn new(x: f32, y: f32) -> Position
    {
        Position { x, y }
    }

    pub fn overlap_rect(&self, rect: Rect) -> bool
    {
        self.x >= rect.x && self.x <= rect.x + rect.width
            && self.y >= rect.y && self.y <= rect.y + rect.height
    }
}

pub fn distance_squared(a: Position, b: Position) -> f32
{
    let dx = a.x - b.x;
    let dy = a.y - b.y;

    dx * dx + dy * dy
}

pub fn distance(a: Position, b: Position) -> f32
{
    distance_squared(a, b).sqrt()
}