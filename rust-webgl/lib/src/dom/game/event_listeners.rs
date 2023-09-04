use std::time::Duration;

use crate::{errors::Result, glmath::vector2::Vector2};

pub trait EventListener {
    fn animate(&mut self, delta: Duration) -> Result<()>;
    fn render(&mut self) -> Result<()>;
    fn resize(&mut self, size: Vector2<u32>) -> Result<()>;
    fn mousemove(
        &mut self,
        location: Vector2<i32>,
        delta: Vector2<i32>,
        is_pointer_locked: bool,
    ) -> Result<()>;
    fn mousedown(&mut self, button: i16, location: Vector2<i32>) -> Result<()>;
    fn mouseup(&mut self, button: i16, location: Vector2<i32>) -> Result<()>;
}
