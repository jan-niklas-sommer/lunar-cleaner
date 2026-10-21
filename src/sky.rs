//! Der Himmel über der Mond-Region (Technik aus spcgame): Sonne als
//! unbeleuchtete HDR-Kugel mit Corona-Billboard, Erde als Kugel mit einer
//! beim Start aus echten Daten (ETOPO1-Höhen, Blue-Marble-Klima) erzeugten
//! Textur, ein gerichtetes Sonnenlicht und Restlicht.
//!
//! Sonne und Erde hängen an einem [`SkyRig`], der der Kamera folgt: Sie
//! stehen wie wirklich ferne Objekte parallaxenfrei fest am Himmel. Vom
//! Mond aus gesehen steht die Erde (gebundene Rotation) immer an derselben
//! Stelle; ihre scheinbare Größe beträgt real ~1,9°, die der Sonne ~0,53°.

use bevy::asset::RenderAssetUsages;
use bevy::camera::visibility::NoFrustumCulling;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::transform::TransformSystems;

use crate::height_maps::{self, dir_of};
use crate::world_origin;
use crate::{noise, palette};

/// Entfernung, in der Sonne und Erde gerendert werden (muss < camera far bleiben).
const SKY_DISTANCE: f32 = 20_000.0;
/// Entfernung der Erde; etwas weiter weg als die Sonne, also steht die
/// Sonne bei Überdeckungen davor.
const EARTH_DISTANCE: f32 = 25_000.0;
/// Scheinbare Größen: Sonne 0,53°, Erde 1,9° Durchmesser (wie real vom Mond).
const SUN_ANGULAR_DIAMETER: f32 = 0.53_f32.to_radians();
const EARTH_ANGULAR_DIAMETER: f32 = 1.9_f32.to_radians();
/// Corona-Billboard: halbe Kantenlänge in Sonnenradien.
const CORONA_SCALE: f32 = 3.0;
/// HDR-Helligkeit von Sonnenkugel und Corona, damit Bloom sie aufgreift.
const SUN_GLOW: f32 = 2.2;
const CORONA_GLOW: f32 = 2.2;
const CORONA_TEXTURE_SIZE: u32 = 512;
/// Beleuchtungsstärke des Sonnenlichts in Lux (wie in spcgame abgestimmt).
const SUN_ILLUMINANCE: f32 = 14_000.0;
/// Restlicht (cd/m²): Der Mond hat keine Atmosphäre, Schatten wären sonst
/// bildschirm schwarz; leicht bläulich und bewusst schwach.
const AMBIENT_BRIGHTNESS: f32 = 150.0;
/// Erdtextur-Auflösung (halbe Auflösung der Höhenkarte reicht am Himmel).
const EARTH_TEXTURE_WIDTH: usize = 1_800;
const EARTH_TEXTURE_HEIGHT: usize = 900;
/// Umdrehung der Erde um sich selbst: 30 s pro Umdrehung (Vorschau-Tempo).
const EARTH_SPIN_SECONDS: f32 = 30.0;

/// Himmelsrichtung der Sonne (Einheitsvektor, horizontnah für lange Schatten).
fn sun_direction() -> Vec3 {
    let elevation = 28.0_f32.to_radians();
    let azimuth = 135.0_f32.to_radians();
    Vec3::new(
        azimuth.cos() * elevation.cos(),
        elevation.sin(),
        azimuth.sin() * elevation.cos(),
    )
    .normalize()
}

/// Himmelsrichtung der Erde: etwa gegenüber der Sonne — vom Mond aus sieht
/// man dann eine „volle Erde“.
fn earth_direction() -> Vec3 {
    let elevation = 42.0_f32.to_radians();
    let azimuth = 315.0_f32.to_radians();
    Vec3::new(
        azimuth.cos() * elevation.cos(),
        elevation.sin(),
        azimuth.sin() * elevation.cos(),
    )
    .normalize()
}

pub struct SkyPlugin;

impl Plugin for SkyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_sky).add_systems(
            PostUpdate,
            // Erst rezentrieren (Kamera nahe Null), dann folgt der Himmel der
            // Kamera, dann erst die Transform-Weitergabe an die Kinder.
            (follow_camera, spin_earth)
                .chain()
                .after(world_origin::RecenterWorld)
                .before(TransformSystems::Propagate),
        );
    }
}

/// Markiert das Gestell, das der Kamera folgt; Sonne und Erde sind Kinder.
#[derive(Component)]
struct SkyRig;

/// Die Erdkugel, langsam rotierend.
#[derive(Component)]
struct EarthGlobe;

fn spawn_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let sun_dir = sun_direction();
    let earth_dir = earth_direction();

    // Sonne: unbeleuchtet und überhell; Bloom macht daraus einen glühenden Ball.
    let sun_radius = SKY_DISTANCE * (SUN_ANGULAR_DIAMETER / 2.0).tan();
    let sun_material = materials.add(StandardMaterial {
        base_color: Color::LinearRgba(LinearRgba::rgb(SUN_GLOW, SUN_GLOW, SUN_GLOW)),
        unlit: true,
        ..default()
    });

    // Corona: kamerazugewandtes Billboard mit radialem Glühen und weichen
    // Strahlen; volle Stärke am Sonnenrand, ausblendend zur Kante.
    let corona_half = sun_radius * CORONA_SCALE;
    let corona_material = materials.add(StandardMaterial {
        base_color: Color::LinearRgba(LinearRgba::rgb(CORONA_GLOW, CORONA_GLOW, CORONA_GLOW)),
        base_color_texture: Some(images.add(corona_image(CORONA_TEXTURE_SIZE))),
        alpha_mode: AlphaMode::Add,
        unlit: true,
        ..default()
    });

    // Erde: Kugel mit einer Textur aus echten Höhen- und Klimadaten.
    let earth_radius = EARTH_DISTANCE * (EARTH_ANGULAR_DIAMETER / 2.0).tan();
    let earth_material = materials.add(StandardMaterial {
        base_color_texture: Some(images.add(earth_texture())),
        // Wolkenlos glatte Kugel aus der Ferne: keine Glanzpunkte.
        perceptual_roughness: 1.0,
        ..default()
    });

    commands
        .spawn((
            Name::new("Sky rig"),
            SkyRig,
            Transform::default(),
            Visibility::default(),
        ))
        .with_children(|rig| {
            rig.spawn((
                Name::new("Sonne"),
                Mesh3d(meshes.add(
                    Sphere::new(sun_radius)
                        .mesh()
                        .ico(4)
                        .expect("Icosphere-Unterteilung"),
                )),
                MeshMaterial3d(sun_material),
                Transform::from_translation(sun_dir * SKY_DISTANCE),
                NoFrustumCulling,
            ));
            rig.spawn((
                Name::new("Corona"),
                Mesh3d(meshes.add(Rectangle::new(2.0 * corona_half, 2.0 * corona_half))),
                MeshMaterial3d(corona_material),
                // Billboard: Frontseite (+Z) zeigt zur Kamera am Gestell-Ursprung.
                Transform::from_translation(sun_dir * SKY_DISTANCE)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                NoFrustumCulling,
            ));
            rig.spawn((
                Name::new("Erde"),
                EarthGlobe,
                Mesh3d(meshes.add(Sphere::new(earth_radius).mesh().uv(64, 32))),
                MeshMaterial3d(earth_material),
                Transform::from_translation(earth_dir * EARTH_DISTANCE),
                NoFrustumCulling,
            ));
        });

    // Sonnenlicht: scheint aus der Himmelsrichtung der Sonne in die Szene.
    commands.spawn((
        Name::new("Sonnenlicht"),
        DirectionalLight {
            color: palette::SUN_CORE.mix(&Color::WHITE, 0.6),
            illuminance: SUN_ILLUMINANCE,
            ..default()
        },
        Transform::from_translation(sun_dir).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    commands.spawn((
        Name::new("Restlicht"),
        AmbientLight {
            color: palette::AMBIENT,
            brightness: AMBIENT_BRIGHTNESS,
            ..default()
        },
    ));
}

/// Das Gestell (und damit Sonne, Corona und Erde) folgt der Kamera:
/// ferne Objekte zeigen keine Parallaxe.
fn follow_camera(
    camera: Single<&Transform, With<Camera3d>>,
    mut rig: Single<&mut Transform, (With<SkyRig>, Without<Camera3d>)>,
) {
    rig.translation = camera.translation;
}

/// Die Erde dreht sich langsam um sich selbst.
fn spin_earth(time: Res<Time>, mut globe: Single<&mut Transform, With<EarthGlobe>>) {
    globe.rotate_y(time.delta_secs() * std::f32::consts::TAU / EARTH_SPIN_SECONDS);
}

/// Radiales Glühen mit weichen Strahlen: volle Stärke am Sonnenrand
/// (Radius 1/CORONA_SCALE des Billboards), ausblendend zur Billboard-Kante.
/// Das Billboards ist kamerazugewandt; die Sonne liegt im Zentrum.
fn corona_image(size: u32) -> Image {
    let limb = 1.0 / CORONA_SCALE;
    let mut data = Vec::with_capacity((size * size * 4) as usize);
    for y in 0..size {
        for x in 0..size {
            let p = (Vec2::new(x as f32, y as f32) + 0.5) / size as f32 * 2.0 - 1.0;
            let r = p.length();
            let fade = ((1.0 - r) / (1.0 - limb)).clamp(0.0, 1.0);
            let angle = p.y.atan2(p.x);
            let rays = noise::fbm(Vec3::new(angle.cos(), angle.sin(), 0.0) * 6.0, 3, 7);
            let strength = fade.powf(3.0) * (0.75 + 0.5 * rays) + fade.powf(12.0);
            let color = palette::SUN.mix(&palette::SUN_CORE, fade).to_srgba();
            data.extend_from_slice(&[
                (color.red * 255.0) as u8,
                (color.green * 255.0) as u8,
                (color.blue * 255.0) as u8,
                (strength.clamp(0.0, 1.0) * 255.0) as u8,
            ]);
        }
    }
    Image::new(
        Extent3d {
            width: size,
            height: size,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Equirectangulare Erdtextur aus echten Daten: Meer nach Tiefe geschattet,
/// Land zwischen Grün und Wüstengold nach dem Wüstenanteil, Eis und Seen aus
/// der Klimakarte, dazu dezente statische Wolken.
fn earth_texture() -> Image {
    let heights = height_maps::earth_heights();
    let (grid, climate) = height_maps::earth_climate();
    let channel = |node: usize, k: usize| climate[node][k] as f64 / 255.0;
    let smoothstep = |a: f64, b: f64, x: f64| {
        let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    };
    let mix = |a: Color, b: Color, t: f64| a.mix(&b, t.clamp(0.0, 1.0) as f32);

    let mut data = Vec::with_capacity(EARTH_TEXTURE_WIDTH * EARTH_TEXTURE_HEIGHT * 4);
    for y in 0..EARTH_TEXTURE_HEIGHT {
        let lat = 90.0 - (y as f64 + 0.5) / EARTH_TEXTURE_HEIGHT as f64 * 180.0;
        for x in 0..EARTH_TEXTURE_WIDTH {
            let lon = (x as f64 + 0.5) / EARTH_TEXTURE_WIDTH as f64 * 360.0 - 180.0;
            let dir = dir_of(lon, lat);
            let height = heights.height(dir);
            let desert = grid.bilinear(dir, |i| channel(i, 0));
            let ice = grid.bilinear(dir, |i| channel(i, 1));
            let lake = grid.bilinear(dir, |i| channel(i, 2));

            let mut color = if height < 0.0 && lake < 0.3 {
                mix(
                    palette::EARTH_OCEAN,
                    palette::EARTH_OCEAN_DEEP,
                    smoothstep(0.0, 5_500.0, -height),
                )
            } else {
                let land = mix(palette::EARTH_LAND, palette::DESERT_GOLD, desert);
                mix(land, palette::SNOW, smoothstep(0.15, 0.75, ice))
            };
            color = mix(color, palette::EARTH_LAKE, smoothstep(0.3, 0.8, lake));

            // Dezente Wolkenbänder, zur Textur gebacken.
            let clouds = smoothstep(
                0.52,
                0.72,
                noise::fbm64(dir * 3.4, 4, 0xea27_7102) * 0.5 + 0.5,
            ) * 0.65
                * (1.0 - 0.5 * ice);
            color = mix(color, palette::CLOUD, clouds);

            let srgb = color.to_srgba();
            data.extend_from_slice(&[
                (srgb.red * 255.0) as u8,
                (srgb.green * 255.0) as u8,
                (srgb.blue * 255.0) as u8,
                255,
            ]);
        }
    }
    Image::new(
        Extent3d {
            width: EARTH_TEXTURE_WIDTH as u32,
            height: EARTH_TEXTURE_HEIGHT as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sonne_und_erde_liegen_gegeneinander() {
        let sun = sun_direction();
        let earth = earth_direction();
        assert!(sun.dot(earth) < 0.0, "Erde gegenüber der Sonne");
        assert!((sun.length() - 1.0).abs() < 1e-6);
        assert!((earth.length() - 1.0).abs() < 1e-6);
    }

    /// Die Größen am Himmel stimmen mit den Winkeln überein.
    #[test]
    fn scheinbare_groessen_stimmen() {
        let sun_radius = SKY_DISTANCE * (SUN_ANGULAR_DIAMETER / 2.0).tan();
        let earth_radius = EARTH_DISTANCE * (EARTH_ANGULAR_DIAMETER / 2.0).tan();
        assert!((sun_radius / SKY_DISTANCE).atan() * 2.0 - SUN_ANGULAR_DIAMETER < 1e-4);
        assert!((earth_radius / EARTH_DISTANCE).atan() * 2.0 - EARTH_ANGULAR_DIAMETER < 1e-4);
        // Sichtbar größer als die Sonne, wie real (~3,6-facher Durchmesser).
        assert!(earth_radius / EARTH_DISTANCE > 2.5 * sun_radius / SKY_DISTANCE);
    }
}
