use std::{collections::HashMap, hash::Hash};

use crate::glmath::vector2::Vector2;

pub struct EventState {
    mouse_location: Option<Vector2<i32>>,
    is_pointer_locked: bool,
    mouse_buttons: HashMap<i16, bool>,
    keyboard_keys: HashMap<String, bool>,
    keyboard_key_codes: HashMap<u32, bool>,
}

impl EventState {
    pub(crate) fn new() -> Self {
        Self {
            mouse_location: None,
            is_pointer_locked: false,
            mouse_buttons: HashMap::new(),
            keyboard_keys: HashMap::new(),
            keyboard_key_codes: HashMap::new(),
        }
    }

    pub(crate) fn mousemove(&mut self, location: Vector2<i32>, is_pointer_locked: bool) {
        self.mouse_location.replace(location);
        self.is_pointer_locked = is_pointer_locked;
    }

    pub(crate) fn mousedown(&mut self, button: i16, location: Vector2<i32>) {
        self.mouse_location.replace(location);
        self.mouse_buttons.insert(button, true);
    }

    pub(crate) fn mouseup(&mut self, button: i16, location: Vector2<i32>) {
        self.mouse_location.replace(location);
        self.mouse_buttons.insert(button, false);
    }

    pub(crate) fn keydown(&mut self, key: String, key_code: u32) {
        self.keyboard_keys.insert(key, true);
        self.keyboard_key_codes.insert(key_code, true);
    }

    pub(crate) fn keyup(&mut self, key: String, key_code: u32) {
        self.keyboard_keys.insert(key, false);
        self.keyboard_key_codes.insert(key_code, false);
    }

    pub fn mouse_location(&self) -> Option<Vector2<i32>> {
        self.mouse_location
    }

    pub fn is_pointer_locked(&self) -> bool {
        self.is_pointer_locked
    }

    pub fn is_mouse_button_pressed(&self, button: i16) -> bool {
        *self.mouse_buttons.get(&button).unwrap_or(&false)
    }

    pub fn is_keyboard_key_pressed(&self, key: String) -> bool {
        *self.keyboard_keys.get(&key).unwrap_or(&false)
    }

    pub fn is_keyboard_key_code_pressed(&self, key_code: u32) -> bool {
        *self.keyboard_key_codes.get(&key_code).unwrap_or(&false)
    }
}
