//! Lunar Cleaner — Mond-Vorschau (Grundgerüst aus spcgame-Technik).
//!
//! Zeigt die Startregion auf dem echten Mond (Apollo 11, Mare
//! Tranquillitatis, echte LRO-LOLA-Höhen + prozedurale Kleinkrater) unter
//! dem Himmel aus spcgame: Sonne mit Corona, volle Erde, Sternenfeld. Die
//! eigentlichen Spielsysteme (DustChunks, Putzen, Wirtschaft, Netzwerk)
//! baut die Spezifikation in `agents_md_lunar_cleaner_bevy_spezifikation.md`
//! auf dieser Grundlage auf.

mod fly_camera;
mod height_maps;
mod moon_data;
mod moon_surface;
mod noise;
mod palette;
mod sky;
mod starfield;
mod world_origin;

use bevy::prelude::*;
use bevy::window::PresentMode;

fn main() {
    App::new()
        .insert_resource(ClearColor(palette::SPACE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Lunar Cleaner — Mond-Vorschau".into(),
                resolution: (1600, 900).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(moon_surface::PlayRegion(moon_data::apollo_11(
            moon_surface::REGION_SIZE,
        )))
        .add_plugins((
            world_origin::WorldOriginPlugin,
            starfield::StarfieldPlugin,
            sky::SkyPlugin,
            moon_surface::MoonSurfacePlugin,
            fly_camera::FlyCameraPlugin,
        ))
        .add_systems(Startup, spawn_preview_camera)
        .run();
}

/// Kamera knapp über dem Boden am Regionsrand, Blick zum Mittelpunkt, wo
/// später das Start-Depot steht.
fn spawn_preview_camera(mut commands: Commands, region: Res<moon_surface::PlayRegion>) {
    let ground = moon_surface::ground_height(&region.0, 0.0, 0.0);
    fly_camera::spawn_camera(
        &mut commands,
        Vec3::new(-1_200.0, ground + 320.0, -1_200.0),
        Vec3::new(0.0, ground + 40.0, 0.0),
    );
}
