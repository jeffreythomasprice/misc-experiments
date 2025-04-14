use bevy::{
    core::FrameCount,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    ecs::system::assert_system_does_not_conflict,
    math::VectorSpace,
    prelude::*,
    render::camera::{ScalingMode, Viewport},
    window::WindowResolution,
};

#[derive(Component)]
struct CameraMarker;

#[derive(Component)]
struct FPSCounterMarker;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Experiment".into(),
                resizable: true,
                resolution: WindowResolution::new(1024.0, 768.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, startup)
        .add_systems(PostStartup, log_camera)
        .add_systems(FixedUpdate, (update_fps_counter, handle_input, update))
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::FixedHorizontal {
                viewport_width: 500.0,
            },
            ..OrthographicProjection::default_2d()
        },
        CameraMarker,
    ));

    // TODO replace with moving sprites
    commands.spawn(Sprite::from_image(
        asset_server.load("rustacean-flat-happy.png"),
    ));

    // TODO put text in a corner to prove we know how ortho camera works
    let font = asset_server.load("calibri-font-family/calibri-regular.ttf");
    let text_font = TextFont {
        font: font.clone(),
        font_size: 50.0,
        ..default()
    };
    commands.spawn((
        Text2d::new("Hello, World!"),
        text_font,
        TextLayout::new_with_justify(JustifyText::Left),
        // ensure this is drawn on top
        Transform::from_translation(Vec3::Z),
    ));

    let root_uinode = commands
        .spawn(Node {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .id();
    let left_column = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Start,
            flex_grow: 1.,
            margin: UiRect::axes(Val::Px(5.), Val::Px(5.)),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((
                Text::new("This is\ntext with\nline breaks\nin the top left."),
                TextFont {
                    font: font.clone(),
                    font_size: 25.0,
                    ..default()
                },
                // TODO JEFF bg color?
                // BackgroundColor(background_color),
                FPSCounterMarker,
            ));
        })
        .id();
    let right_column = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::End,
            flex_grow: 1.,
            margin: UiRect::axes(Val::Px(5.), Val::Px(5.)),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((
                Text::new("This is\ntext with\nline breaks\nin the top right."),
                TextFont {
                    font: font.clone(),
                    font_size: 25.0,
                    ..default()
                },
                // TODO JEFF bg color?
                // BackgroundColor(background_color),
            ));
        })
        .id();
    commands
        .entity(root_uinode)
        .add_children(&[left_column, right_column]);
}

fn log_camera(
    mut camera_query: Query<
        (
            &mut Camera,
            &mut GlobalTransform,
            &mut OrthographicProjection,
        ),
        With<CameraMarker>,
    >,
    window: Query<&Window>,
) {
    let (camera, transform, mut projection) = camera_query.single_mut();
    info!("TODO camera = {:?}", camera);
    info!("TODO transform = {:?}", transform);
    info!("TODO projection = {:?}", projection);

    let window = window.single();
    info!("TODO window = {:?}", window);

    let upper_left = camera.viewport_to_world_2d(&transform, Vec2::ZERO);
    let lower_right = camera.viewport_to_world_2d(&transform, window.size());
    info!(
        "TODO window corners in world pos: {:?}, {:?}",
        upper_left, lower_right
    );
}

fn update_fps_counter(
    mut text: Query<&mut Text, With<FPSCounterMarker>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    if let Some(fps) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|fps| fps.smoothed())
    {
        let mut text = text.single_mut();
        *text = format!("FPS: {:.1}", fps).into();
    }
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
    for key in keys.get_just_released() {
        match key {
            KeyCode::Escape => {
                app_exit_events.send(AppExit::Success);
            }
            _ => (),
        };
    }
}

fn update() {
    // info!("TODO update");
}
