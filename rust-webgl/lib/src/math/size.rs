use std::fmt::Display;

#[derive(Debug)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Clone for Size<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        Self {
            width: self.width.clone(),
            height: self.height.clone(),
        }
    }
}

impl<T> Copy for Size<T> where T: Copy {}

impl<T> Display for Size<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} x {})", self.width, self.height)
    }
}
