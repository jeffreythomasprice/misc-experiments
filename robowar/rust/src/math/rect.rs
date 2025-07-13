use std::ops::{Add, Sub};

use num::Zero;

use crate::math::Vec2;

#[derive(Debug, Clone)]
pub struct Rect<T> {
    min: Vec2<T>,
    max: Vec2<T>,
}

impl<T> Copy for Rect<T> where T: Copy {}

impl<T> Rect<T>
where
    T: Copy,
{
    pub fn new_with_origin_size(origin: Vec2<T>, size: Vec2<T>) -> Self
    where
        T: Add<Output = T> + PartialOrd,
    {
        let p1 = origin;
        let p2 = origin + size;
        let (x1, x2) = if p1.x < p2.x {
            (p1.x, p2.x)
        } else {
            (p2.x, p1.x)
        };
        let (y1, y2) = if p1.y < p2.y {
            (p1.y, p2.y)
        } else {
            (p2.y, p1.y)
        };
        Self {
            min: Vec2::new(x1, y1),
            max: Vec2::new(x2, y2),
        }
    }

    pub fn new_with_points(points: &[Vec2<T>]) -> Option<Rect<T>>
    where
        T: Zero + PartialOrd,
    {
        points.iter().fold(None, |result, point| match result {
            Some(result) => Some(Self {
                min: Vec2::new(
                    if result.min.x < point.x {
                        result.min.x
                    } else {
                        point.x
                    },
                    if result.min.y < point.y {
                        result.min.y
                    } else {
                        point.y
                    },
                ),
                max: Vec2::new(
                    if result.max.x > point.x {
                        result.max.x
                    } else {
                        point.x
                    },
                    if result.max.y > point.y {
                        result.max.y
                    } else {
                        point.y
                    },
                ),
            }),
            None => Some(Rect::new_with_origin_size(
                *point,
                Vec2::new(T::zero(), T::zero()),
            )),
        })
    }

    pub fn minimum(&self) -> Vec2<T> {
        self.min
    }

    pub fn maximum(&self) -> Vec2<T> {
        self.max
    }

    pub fn origin(&self) -> Vec2<T> {
        self.minimum()
    }

    pub fn size(&self) -> Vec2<T>
    where
        T: Sub<Output = T>,
    {
        self.max - self.min
    }

    pub fn width(&self) -> T
    where
        T: Sub<Output = T>,
    {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> T
    where
        T: Sub<Output = T>,
    {
        self.max.y - self.min.y
    }
}
