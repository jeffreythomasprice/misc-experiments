use nalgebra::{clamp, wrap, Matrix4};
use nalgebra_glm::{look_at, perspective_fov, rotate_vec3, I32Vec2, Vec2, Vec3};

use super::size::Size;

pub struct Camera {
    fov: f32,
    screen_size: Size<u32>,
    near: f32,
    far: f32,
    position: Vec3,
    look: Vec3,
    right: Vec3,
    up: Vec3,

    angle_right: f32,
    angle_up: f32,

    rotated_look: Vec3,
    rotated_right: Vec3,

    projection_matrix: Matrix4<f32>,
    model_view_matrix: Matrix4<f32>,
}

impl Camera {
    pub fn new(
        fov: f32,
        screen_size: Size<u32>,
        near: f32,
        far: f32,
        position: Vec3,
        target: Vec3,
        up: Vec3,
    ) -> Self {
        let look = (target - position).normalize();
        let up = up.normalize();
        let right = look.cross(&up);
        let mut result = Self {
            fov,
            screen_size,
            near,
            far,
            position,
            look,
            right,
            up,

            angle_right: 0.0,
            angle_up: 0.0,

            rotated_look: Vec3::zeros(),
            rotated_right: Vec3::zeros(),

            projection_matrix: Matrix4::identity(),
            model_view_matrix: Matrix4::identity(),
        };

        result.update_rotated_look();
        result.update_projection_matrix();
        result.update_model_view_matrix();

        result
    }

    pub fn field_of_view(&self) -> f32 {
        self.fov
    }

    pub fn get_screen_size(&self) -> &Size<u32> {
        &self.screen_size
    }

    pub fn set_screen_size(&mut self, s: Size<u32>) {
        self.screen_size = s;
        self.update_projection_matrix();
    }

    pub fn projection_matrix(&self) -> &Matrix4<f32> {
        &self.projection_matrix
    }

    pub fn model_view_matrix(&self) -> &Matrix4<f32> {
        &self.model_view_matrix
    }

    pub fn move_based_on_current_axes(&mut self, forward: f32, up: f32, right: f32) {
        let mut needs_update = false;
        if forward != 0.0f32 {
            self.position += forward * self.rotated_look;
            needs_update = true;
        }
        if up != 0.0f32 {
            self.position += up * self.up;
            needs_update = true;
        }
        if right != 0.0f32 {
            self.position += right * self.rotated_right;
            needs_update = true;
        }
        if needs_update {
            self.update_model_view_matrix();
        }
    }

    pub fn turn_based_on_mouse_delta(&mut self, delta: I32Vec2) {
        self.angle_right = wrap(
            self.angle_right - (delta.x as f32) * 0.1f32.to_radians(),
            0.0f32.to_radians(),
            360.0f32.to_radians(),
        );
        self.angle_up = clamp(
            self.angle_up - (delta.y as f32) * 0.1f32.to_radians(),
            -89.0f32.to_radians(),
            89.0f32.to_radians(),
        );
        self.update_rotated_look();
        self.update_model_view_matrix();
    }

    fn update_rotated_look(&mut self) {
        let look_with_only_right_angle = rotate_vec3(&self.look, self.angle_right, &self.up);
        let right_right_only_right_angle = look_with_only_right_angle.cross(&self.up);
        self.rotated_look = rotate_vec3(
            &look_with_only_right_angle,
            self.angle_up,
            &right_right_only_right_angle,
        );
        self.rotated_right = self.rotated_look.cross(&self.up);
    }

    fn update_projection_matrix(&mut self) -> &Matrix4<f32> {
        self.projection_matrix = if self.screen_size.width >= 1 && self.screen_size.height >= 1 {
            perspective_fov(
                self.fov,
                self.screen_size.width as f32,
                self.screen_size.height as f32,
                self.near,
                self.far,
            )
        } else {
            Matrix4::identity()
        };
        &self.projection_matrix
    }

    fn update_model_view_matrix(&mut self) -> &Matrix4<f32> {
        self.model_view_matrix = look_at(
            &self.position,
            &(self.position + self.rotated_look),
            &self.up,
        );
        &self.model_view_matrix
    }
}
