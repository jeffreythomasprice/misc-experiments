use glam::Vec2;

pub fn circle(center: Vec2, radius: f32) -> Vec<Vec2> {
    let num_points = 32;
    let angle_step = std::f32::consts::TAU / (num_points as f32);
    let mut angle: f32 = 0.0;
    let mut results = Vec::with_capacity(num_points);
    for i in 0..num_points {
        results.push(Vec2::new(angle.cos(), angle.sin()) * radius);
        angle += angle_step;
    }
    results
}
