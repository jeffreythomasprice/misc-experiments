use std::{fmt::Display, ops};

use crate::Error;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl ops::Add<Point> for Point {
    type Output = Point;

    fn add(self, rhs: Point) -> Self::Output {
        Point {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    width: f64,
    height: f64,
}

impl Size {
    pub fn new(width: f64, height: f64) -> Result<Self, Error> {
        if width < 0.0 || height < 0.0 {
            Err(format!("invalid size {width} x {height}"))?
        } else {
            Ok(Self { width, height })
        }
    }

    pub fn width(&self) -> f64 {
        self.width
    }

    pub fn height(&self) -> f64 {
        self.height
    }
}

impl Display for Size {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} x {})", self.width, self.height)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Rectangle {
    origin: Point,
    size: Size,
}

impl Rectangle {
    pub fn from_origin_size(origin: Point, size: Size) -> Self {
        Self { origin, size }
    }

    pub fn from_two_points(p1: &Point, p2: &Point) -> Self {
        let x1 = p1.x.min(p2.x);
        let y1 = p1.y.min(p2.y);
        let x2 = p1.x.max(p2.x);
        let y2 = p1.y.max(p2.y);
        let size = Size::new(x2 - x1, y2 - y1)
            .expect("shouldn't be possible to get a negative size from two points");
        Self {
            origin: Point { x: x1, y: y1 },
            size,
        }
    }

    pub fn from_points<'a, I>(points: I) -> Result<Self, Error>
    where
        I: Iterator<Item = &'a Point>,
    {
        let mut x1: Option<f64> = None;
        let mut y1: Option<f64> = None;
        let mut x2: Option<f64> = None;
        let mut y2: Option<f64> = None;
        for p in points {
            x1 = match x1 {
                Some(cur) => Some(cur.min(p.x)),
                None => Some(p.x),
            };
            y1 = match y1 {
                Some(cur) => Some(cur.min(p.y)),
                None => Some(p.y),
            };
            x2 = match x2 {
                Some(cur) => Some(cur.max(p.x)),
                None => Some(p.x),
            };
            y2 = match y2 {
                Some(cur) => Some(cur.max(p.y)),
                None => Some(p.y),
            };
        }
        match (x1, y1, x2, y2) {
            (Some(x1), Some(y1), Some(x2), Some(y2)) => Ok(Self::from_two_points(
                &Point { x: x1, y: y1 },
                &Point { x: x2, y: y2 },
            )),
            _ => Err("not enough points")?,
        }
    }

    pub fn from_centered_size(other: &Rectangle, size: Size) -> Self {
        let offset = Point {
            x: (other.size.width() - size.width()) / 2.0,
            y: (other.size.height() - size.height()) / 2.0,
        };
        Self {
            origin: other.origin + offset,
            size,
        }
    }

    pub fn origin(&self) -> &Point {
        &self.origin
    }

    pub fn size(&self) -> &Size {
        &self.size
    }

    pub fn min(&self) -> &Point {
        &self.origin
    }

    pub fn max(&self) -> Point {
        Point {
            x: self.origin.x + self.size.width,
            y: self.origin.y + self.size.height,
        }
    }

    pub fn contains(&self, p: &Point) -> bool {
        p.x >= self.min().x && p.x <= self.max().x && p.y >= self.min().y && p.y <= self.max().y
    }
}

impl Display for Rectangle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rectangle(origin={}, size={})", self.origin, self.size)
    }
}

pub struct RGBColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Display for RGBColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RGB({}, {}, {})", self.red, self.green, self.blue)
    }
}
