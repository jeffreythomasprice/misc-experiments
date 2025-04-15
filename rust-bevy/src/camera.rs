use bevy::{
    color::palettes::css::{GREEN, MEDIUM_VIOLET_RED},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
    render::camera::ScalingMode,
    window::WindowResolution,
};

#[derive(Component, Debug)]
pub struct Camera2dCentering {
    pub world_rect: Rect,
}

pub fn update(
    mut camera_query: Query<(
        &mut Camera,
        &mut GlobalTransform,
        &mut OrthographicProjection,
        &mut Camera2dCentering,
    )>,
    window: Query<&Window>,
) {
    // let (camera, transform, projection, centering) = camera_query.single_mut();
    // info!("TODO camera = {:?}", camera);
    // info!("TODO transform = {:?}", transform);
    // info!("TODO projection = {:?}", projection);
    // info!("TODO centering = {:?}", centering);

    // let window = window.single();
    // info!("TODO window = {:?}", window);

    // let upper_left = camera.viewport_to_world_2d(&transform, Vec2::ZERO);
    // let lower_right = camera.viewport_to_world_2d(&transform, window.size());
    // info!(
    //     "TODO window corners in world pos: {:?}, {:?}",
    //     upper_left, lower_right
    // );

    /*
    TODO impl camera centering

    centering.world_rect = the coordinate space to look at
    screen_rect = rect formed by upper_left and lower_right = world rect

    scale such that the one dimension of world_rect = equivalent on screen_rect, and on the other dimension screen_rect is bigger
    transform such that world_ret is centered
    */
}
