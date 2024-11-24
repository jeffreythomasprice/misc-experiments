use std::fmt::Display;

use nalgebra_glm::{I32Vec2, U32Vec2};

#[derive(Debug, Clone)]
pub struct MousePressEvent {
    pub event: web_sys::MouseEvent,
}

#[derive(Debug)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(i16),
}

impl MousePressEvent {
    pub fn position(&self) -> U32Vec2 {
        U32Vec2::new(self.event.x() as u32, self.event.y() as u32)
    }

    pub fn button(&self) -> MouseButton {
        match self.event.button() {
            0 => MouseButton::Left,
            1 => MouseButton::Middle,
            2 => MouseButton::Right,
            x => MouseButton::Other(x),
        }
    }
}

impl Display for MousePressEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MousePressEvent(position=({}, {}), button={:?})",
            self.position().x,
            self.position().y,
            self.button()
        )
    }
}

#[derive(Debug, Clone)]
pub struct MouseMoveEvent {
    pub event: web_sys::MouseEvent,
}

impl MouseMoveEvent {
    pub fn position(&self) -> U32Vec2 {
        U32Vec2::new(self.event.x() as u32, self.event.y() as u32)
    }

    pub fn delta(&self) -> I32Vec2 {
        I32Vec2::new(self.event.movement_x(), self.event.movement_y())
    }
}

impl Display for MouseMoveEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MouseMoveEvent(position=({}, {}), delta={})",
            self.position().x,
            self.position().y,
            self.delta()
        )
    }
}

#[derive(Debug, Clone)]
pub struct KeyPressEvent {
    pub event: web_sys::KeyboardEvent,
}

impl KeyPressEvent {
    pub fn code(&self) -> String {
        self.event.code()
    }
}

impl Display for KeyPressEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "KeyPressEvent(code={})", self.code())
    }
}
