//! Minimale Freiflug-Kamera für die Vorschau (Maus umsehen, WASD fliegen,
//! Mausrad Geschwindigkeit). Wird später vom echten Spieler-Controller des
//! Spiels ersetzt.

use bevy::camera::Hdr;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

const START_SPEED: f32 = 60.0;
const SENSITIVITY: f32 = 0.0015;
const SPEED_STEP: f32 = 1.6;
const PIXELS_PER_TICK: f32 = 40.0;
const MAX_PITCH: f32 = 1.55;

pub struct FlyCameraPlugin;

impl Plugin for FlyCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (toggle_grab, look, fly).chain());
    }
}

#[derive(Component)]
pub struct FlyCam {
    speed: f32,
}

pub fn spawn_camera(commands: &mut Commands, position: Vec3, looking_at: Vec3) {
    commands.spawn((
        Name::new("Freiflug-Kamera"),
        Camera3d::default(),
        Hdr,
        Bloom {
            intensity: 0.35,
            ..Bloom::NATURAL
        },
        Projection::Perspective(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            // Bevy nutzt eine unendliche umgekehrte Z-Projektion; `far`
            // begrenzt nur das Culling. Muss die Erde (25 km) umfassen.
            near: 0.5,
            far: 50_000.0,
            ..default()
        }),
        Transform::from_translation(position).looking_at(looking_at, Vec3::Y),
        FlyCam { speed: START_SPEED },
    ));
}

fn toggle_grab(
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<(&mut Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let (mut window, mut cursor) = window.into_inner();
    if mouse.just_pressed(MouseButton::Left) {
        set_grab(&mut window, &mut cursor, true);
    }
    if keys.just_pressed(KeyCode::Escape) {
        set_grab(&mut window, &mut cursor, false);
    }
}

fn set_grab(window: &mut Window, cursor: &mut CursorOptions, grab: bool) {
    if grab {
        // Ein gelockter Cursor bleibt, wo er ist; außerhalb des Fensters
        // gingen Klicks an andere Programme. Also: in die Mitte parken.
        let center = window.size() / 2.0;
        window.set_cursor_position(Some(center));
    }
    cursor.grab_mode = if grab {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };
    cursor.visible = !grab;
}

fn look(
    motion: Res<AccumulatedMouseMotion>,
    cursor: Single<&CursorOptions, With<PrimaryWindow>>,
    camera: Single<(&mut Transform, &FlyCam), With<Camera3d>>,
) {
    if cursor.grab_mode == CursorGrabMode::None {
        return;
    }
    let (mut transform, _) = camera.into_inner();
    let (yaw, pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
    let yaw = yaw - motion.delta.x * SENSITIVITY;
    let pitch = (pitch - motion.delta.y * SENSITIVITY).clamp(-MAX_PITCH, MAX_PITCH);
    transform.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_x(pitch);
}

fn fly(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    scroll: Res<AccumulatedMouseScroll>,
    camera: Single<(&mut Transform, &mut FlyCam), With<Camera3d>>,
) {
    let (mut transform, mut cam) = camera.into_inner();
    let ticks = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_TICK,
    };
    if ticks != 0.0 {
        cam.speed = (cam.speed * SPEED_STEP.powf(ticks.signum())).clamp(2.0, 4_000.0);
    }
    let mut direction = Vec3::ZERO;
    let forward = transform.forward();
    let right = transform.right();
    if keys.pressed(KeyCode::KeyW) {
        direction += *forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= *forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += *right;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= *right;
    }
    if keys.pressed(KeyCode::Space) {
        direction += Vec3::Y;
    }
    if keys.pressed(KeyCode::ControlLeft) {
        direction -= Vec3::Y;
    }
    if direction != Vec3::ZERO {
        transform.translation += direction.normalize() * cam.speed * time.delta_secs();
    }
}
