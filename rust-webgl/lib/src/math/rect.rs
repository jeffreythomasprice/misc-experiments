use std::{
    fmt::Display,
    ops::{Add, Sub},
};

use super::{size::Size, vec2::Vec2};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rect<T> {
    pub min: Vec2<T>,
    pub max: Vec2<T>,
}

impl<T> Clone for Rect<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            min: self.min.clone(),
            max: self.max.clone(),
        }
    }
}

impl<T> Copy for Rect<T> where T: Copy {}

impl<T> Rect<T>
where
    T: Copy + Add<Output = T> + Ord,
{
    pub fn with_position_and_size(position: Vec2<T>, size: Size<T>) -> Self {
        let x1 = position.x;
        let x2 = position.x + size.width;
        let y1 = position.y;
        let y2 = position.y + size.height;
        Self {
            min: Vec2 {
                x: T::min(x1, x2),
                y: T::min(y1, y2),
            },
            max: Vec2 {
                x: T::max(x1, x2),
                y: T::max(y1, y2),
            },
        }
    }
}

impl<T> Rect<T> {
    pub fn origin(&self) -> &Vec2<T> {
        &self.min
    }
}

impl<T> Rect<T>
where
    T: Copy + Sub<Output = T>,
{
    pub fn size(&self) -> Size<T> {
        Size {
            width: self.max.x - self.min.x,
            height: self.max.y - self.min.y,
        }
    }
}

impl<T> Display for Rect<T>
where
    T: Copy + Sub<Output = T> + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rect(origin={}, size={})", self.origin(), self.size())
    }
}

impl<T> Rect<T>
where
    T: Copy + Ord,
{
    pub fn union(a: &Rect<T>, b: &Rect<T>) -> Rect<T> {
        Self {
            min: Vec2 {
                x: T::min(a.min.x, b.min.x),
                y: T::min(a.min.y, b.min.y),
            },
            max: Vec2 {
                x: T::max(a.max.x, b.max.x),
                y: T::max(a.max.y, b.max.y),
            },
        }
    }
}

impl<T> From<rusttype::Rect<T>> for Rect<T> {
    fn from(value: rusttype::Rect<T>) -> Self {
        Self {
            min: value.min.into(),
            max: value.max.into(),
        }
    }
}

impl<T> Rect<T>
where
    T: Copy + PartialOrd + Ord,
{
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        if other.max.x < self.min.x || other.min.x > self.max.x || other.max.y < self.min.y || other.min.y > self.max.y {
            None
        } else {
            Some(Rect {
                min: Vec2 {
                    x: self.min.x.max(other.min.x),
                    y: self.min.y.max(other.min.y),
                },
                max: Vec2 {
                    x: self.max.x.min(other.max.x),
                    y: self.max.y.min(other.max.y),
                },
            })
        }
    }
}
