mod camera;

use bevy::{
    color::palettes::css::{GREEN, MEDIUM_VIOLET_RED},
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    math::VectorSpace,
    prelude::*,
    render::camera::ScalingMode,
    window::{PresentMode, WindowResolution},
};
use camera::Camera2dCentering;

#[derive(Component)]
struct FPSCounterMarker;

#[derive(Component)]
struct AnimatedSprite {
    rotation_speed: f32,
}

#[derive(Component)]
struct Player {
    movement_speed: Vec2,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Experiment".into(),
                resizable: true,
                resolution: WindowResolution::new(1024.0, 768.0),
                present_mode: PresentMode::AutoNoVsync,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_systems(Startup, startup)
        .add_systems(FixedUpdate, update_fps_counter)
        .add_systems(FixedUpdate, animated_sprite_update)
        .add_systems(FixedUpdate, camera::update)
        .add_systems(FixedUpdate, update_player)
        .add_systems(Update, handle_input)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: 500.0,
            },
            ..OrthographicProjection::default_2d()
        },
        Camera2dCentering {
            world_rect: Rect::from_center_size(Vec2::ZERO, Vec2::new(500.0, 500.0)),
        },
    ));

    // TODO multiple sprites
    commands.spawn((
        Sprite::from_image(asset_server.load("rustacean-flat-happy.png")),
        AnimatedSprite {
            rotation_speed: 90.0f32.to_radians(),
        },
    ));

    let font = asset_server.load("calibri-font-family/calibri-regular.ttf");
    let in_game_font = TextFont {
        font: font.clone(),
        font_size: 50.0,
        ..default()
    };
    let ui_font = TextFont {
        font: font.clone(),
        font_size: 25.0,
        ..default()
    };

    commands.spawn((
        Text2d::new("P"),
        in_game_font.clone(),
        TextLayout::new_with_justify(JustifyText::Left),
        // ensure this is drawn on top
        Transform::from_translation(Vec3::Z),
        Player {
            movement_speed: Vec2::ZERO,
        },
    ));
    commands.spawn((
        Text2d::new("1"),
        in_game_font.clone(),
        TextLayout::new_with_justify(JustifyText::Left),
        // z = 1 puts it on top of other stuff
        Transform::from_translation(Vec3::new(250.0, 250.0, 1.0)),
    ));
    commands.spawn((
        Text2d::new("2"),
        in_game_font.clone(),
        TextLayout::new_with_justify(JustifyText::Left),
        // z = 1 puts it on top of other stuff
        Transform::from_translation(Vec3::new(250.0, -250.0, 1.0)),
    ));
    commands.spawn((
        Text2d::new("3"),
        in_game_font.clone(),
        TextLayout::new_with_justify(JustifyText::Left),
        // z = 1 puts it on top of other stuff
        Transform::from_translation(Vec3::new(-250.0, -250.0, 1.0)),
    ));
    commands.spawn((
        Text2d::new("4"),
        in_game_font.clone(),
        TextLayout::new_with_justify(JustifyText::Left),
        // z = 1 puts it on top of other stuff
        Transform::from_translation(Vec3::new(-250.0, 250.0, 1.0)),
    ));

    let root_uinode = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceBetween,
            ..default()
        })
        .id();
    let left_column = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Start,
            flex_grow: 1.0,
            margin: UiRect::axes(Val::Px(5.0), Val::Px(5.0)),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((
                Text::new(""),
                ui_font.clone(),
                BackgroundColor(MEDIUM_VIOLET_RED.into()),
                FPSCounterMarker,
            ));
        })
        .id();
    let right_column = commands
        .spawn(Node {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::End,
            flex_grow: 1.0,
            margin: UiRect::axes(Val::Px(5.0), Val::Px(5.0)),
            ..default()
        })
        .with_children(|builder| {
            builder.spawn((
                Text::new("This is\ntext with\nline breaks\nin the top right."),
                ui_font.clone(),
                TextColor(GREEN.into()),
            ));
        })
        .id();
    commands
        .entity(root_uinode)
        .add_children(&[left_column, right_column]);
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
    mut player: Query<&mut Player>,
) {
    for key in keys.get_just_released() {
        if key == &KeyCode::Escape {
            app_exit_events.send(AppExit::Success);
        };
    }

    for (mut player) in player.iter_mut() {
        player.movement_speed = Vec2::ZERO;
        if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
            player.movement_speed.y += 1.0;
        }
        if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
            player.movement_speed.y -= 1.0;
        }
        if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
            player.movement_speed.x -= 1.0;
        }
        if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
            player.movement_speed.x += 1.0;
        }
    }
}

fn update_player(mut player: Query<(&mut Transform, &Player)>, time: Res<Time>) {
    for (mut player_transform, player) in player.iter_mut() {
        let delta = player.movement_speed * time.delta_secs() * 50.0;
        player_transform.translation += Vec3::new(delta.x, delta.y, 0.0);
    }
}

fn animated_sprite_update(mut sprites: Query<(&mut Transform, &AnimatedSprite)>, time: Res<Time>) {
    for (mut transform, anim) in &mut sprites {
        transform.rotate_z(anim.rotation_speed * time.delta_secs());
    }
}
