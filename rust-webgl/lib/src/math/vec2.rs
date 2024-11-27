use std::fmt::Display;

#[derive(Debug)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Clone for Vec2<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            x: self.x.clone(),
            y: self.y.clone(),
        }
    }
}

impl<T> Copy for Vec2<T> where T: Copy {}

impl<T> Display for Vec2<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl<T> From<nalgebra_glm::TVec2<T>> for Vec2<T>
where
    T: Copy,
{
    fn from(value: nalgebra_glm::TVec2<T>) -> Self {
        Self { x: value[0], y: value[1] }
    }
}

impl<T> From<rusttype::Point<T>> for Vec2<T> {
    fn from(value: rusttype::Point<T>) -> Self {
        Self { x: value.x, y: value.y }
    }
}
