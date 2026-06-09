use super::prelude::*;

#[derive(Copy, Clone)]
pub struct Rect
{
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

const SUBSCRIBE_RANGE_PERCENTAGE: f32 = 0.1;

impl Rect
{
    pub fn divide(self) -> [Rect; 4]
    {
        let half_width = self.width / 2.0;
        let half_height = self.height / 2.0;
        [
            Rect { x: self.x, y: self.y, width: half_width, height: half_height },
            Rect { x: self.x + half_width, y: self.y, width: half_width, height: half_height },
            Rect { x: self.x + half_width, y: self.y + half_height, width: half_width, height: half_height },
            Rect { x: self.x, y: self.y + half_height, width: half_width, height: half_height },
        ]
    }
    
    pub fn subscribe_rect(&self) -> Self
    {
        Self
        {
            x: self.x * (1.0 - SUBSCRIBE_RANGE_PERCENTAGE),
            y: self.y * (1.0 - SUBSCRIBE_RANGE_PERCENTAGE),
            width: self.width * (1.0 + SUBSCRIBE_RANGE_PERCENTAGE),
            height: self.height * (1.0 + SUBSCRIBE_RANGE_PERCENTAGE),
        }
    }
    
    pub fn overlap_circle(self, circle: Circle) -> bool
    {
        if circle.center.overlap_rect(self)
        {
            return true;
        }
        
        let center = Position
        {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0
        };
        let line = Line::new(center, circle.center);

        let upper_left_point = Position::new(self.x, self.y);
        let upper_right_point = Position::new(self.x + self.width, self.y);
        let lower_left_point = Position::new(self.x, self.y + self.height);
        let lower_right_point = Position::new(self.x + self.width, self.y + self.height);

        let left_line = Line::new(upper_left_point, lower_left_point);
        let right_line = Line::new(upper_right_point, lower_right_point);
        let bottom_line = Line::new(lower_left_point, lower_right_point);
        let top_line = Line::new(upper_left_point, upper_right_point);

        let intersection = line.intersect(left_line)
            .and(line.intersect(right_line))
            .and(line.intersect(bottom_line))
            .and(line.intersect(top_line));

        let intersection = match intersection
        {
            Some(intersection) => intersection,
            None => return false,
        };

        circle.radius * circle.radius >= distance_squared(circle.center, intersection)
    }
}

