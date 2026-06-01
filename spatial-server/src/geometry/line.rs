use super::prelude::*;

#[derive(Copy, Clone, Debug)]
pub struct Line
{
    a: f32,
    b: f32,
    c: f32,
}

impl Line
{
    pub fn new(p1: Position, p2: Position) -> Self
    {
        Self
        {
            a: p2.y - p1.y,
            b: p1.x - p2.x,
            c: p1.y * (p2.x - p1.x) - p1.x * (p2.y - p1.y),
        }
    }

    pub fn intersect(&self, other: Line) -> Option<Position>
    {
        let divider = self.a * other.b - other.a * self.b;
        if divider == 0.0
        {
            return None;
        }

        Some(Position
        {
            x: ( self.b * other.c - other.b * self.c) / divider,
            y: ( self.c * other.a - other.c * self.a) / divider,
        })
    }
}