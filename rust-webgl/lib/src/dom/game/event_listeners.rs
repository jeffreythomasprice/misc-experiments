use std::time::Duration;

use crate::{errors::Result, glmath::vector2::Vector2};

use super::event_state::EventState;

pub trait EventListener {
    fn animate(&mut self, delta: Duration, event_state: &EventState) -> Result<()>;
    fn render(&mut self) -> Result<()>;
    fn resize(&mut self, size: Vector2<u32>) -> Result<()>;
    fn mousemove(
        &mut self,
        event_state: &EventState,
        location: Vector2<i32>,
        delta: Vector2<i32>,
        is_pointer_locked: bool,
    ) -> Result<()>;
    fn mousedown(
        &mut self,
        event_state: &EventState,
        button: i16,
        location: Vector2<i32>,
    ) -> Result<()>;
    fn mouseup(
        &mut self,
        event_state: &EventState,
        button: i16,
        location: Vector2<i32>,
    ) -> Result<()>;
    fn keydown(&mut self, event_state: &EventState, key: String, key_code: u32) -> Result<()>;
    fn keyup(&mut self, event_state: &EventState, key: String, key_code: u32) -> Result<()>;
}
