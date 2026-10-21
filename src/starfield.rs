//! Sternenhimmel (aus spcgame übernommen).
//!
//! Beim Start erzeugte Cubemap, angezeigt mit Bevys `Skybox`: Navy-Hintergrund
//! (nie schwarz), ein zarter Nebel und verstreute Sterne. Die Skybox wird im
//! Unendlichen gezeichnet — kein riesiges, kameragebundenes Mesh nötig.

use bevy::asset::RenderAssetUsages;
use bevy::light::Skybox;
use bevy::prelude::*;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

use crate::{noise, palette};

/// Kantenlänge einer Würfelfläche in Pixeln.
const FACE_SIZE: u32 = 1024;
const STAR_COUNT: usize = 9_000;
const SEED: u32 = 0x5eed_57a2;
/// Helligkeit der Skybox (cd/m²), auf die Standard-Kamerabelichtung abgestimmt.
pub const SKY_BRIGHTNESS: f32 = 1_000.0;

pub struct StarfieldPlugin;

impl Plugin for StarfieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, generate_sky)
            .add_systems(Update, attach_skybox);
    }
}

#[derive(Resource)]
struct StarCubemap(Handle<Image>);

fn generate_sky(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let face_bytes = (FACE_SIZE * FACE_SIZE * 4) as usize;
    let faces = generate_faces();
    let mut data = Vec::with_capacity(face_bytes * 6);
    for face in faces {
        data.extend(face);
    }

    // Sechs Flächen untereinander gestapelt, dann als Cube-Texture gelesen.
    let mut image = Image::new(
        Extent3d {
            width: FACE_SIZE,
            height: FACE_SIZE * 6,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image
        .reinterpret_stacked_2d_as_array(6)
        .expect("sechs quadratische Flächen untereinander");
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    commands.insert_resource(StarCubemap(images.add(image)));
}

fn attach_skybox(
    mut commands: Commands,
    sky: Option<Res<StarCubemap>>,
    cameras: Query<Entity, (With<Camera3d>, Without<Skybox>)>,
) {
    let Some(sky) = sky else { return };
    for camera in &cameras {
        commands.entity(camera).insert(Skybox {
            image: Some(sky.0.clone()),
            brightness: SKY_BRIGHTNESS,
            ..default()
        });
    }
}

/// Richtung für Pixel `(u, v)` in `[-1, 1]` auf Würfelfläche `face`
/// (Reihenfolge +X, -X, +Y, -Y, +Z, -Z).
fn face_direction(face: usize, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(1.0, -v, -u),
        1 => Vec3::new(-1.0, -v, u),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, -v),
        4 => Vec3::new(u, -v, 1.0),
        _ => Vec3::new(-u, -v, -1.0),
    }
}

/// Inverse von [`face_direction`]: Fläche und `(u, v)` in `[-1, 1]`.
fn direction_to_face(d: Vec3) -> (usize, f32, f32) {
    let a = d.abs();
    if a.x >= a.y && a.x >= a.z {
        if d.x > 0.0 {
            (0, -d.z / a.x, -d.y / a.x)
        } else {
            (1, d.z / a.x, -d.y / a.x)
        }
    } else if a.y >= a.z {
        if d.y > 0.0 {
            (2, d.x / a.y, d.z / a.y)
        } else {
            (3, d.x / a.y, -d.z / a.y)
        }
    } else if d.z > 0.0 {
        (4, d.x / a.z, -d.y / a.z)
    } else {
        (5, -d.x / a.z, -d.y / a.z)
    }
}

/// Lineares RGB pro Fläche, dann nach sRGB kodiert. Flächen parallel.
fn generate_faces() -> Vec<Vec<u8>> {
    let stars = scatter_stars();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..6)
            .map(|face| {
                let stars = &stars;
                scope.spawn(move || render_face(face, stars))
            })
            .collect();
        handles
            .into_iter()
            .map(|handle| handle.join().expect("Skybox-Flächen-Thread"))
            .collect()
    })
}

struct Star {
    face: usize,
    x: f32,
    y: f32,
    color: Vec3,
    /// Spitzenintensität; über 1 bekommt der Stern zusätzlich einen Halo.
    intensity: f32,
}

fn scatter_stars() -> Vec<Star> {
    let mut rng = noise::Rng::new(SEED);
    let tints = [
        Vec3::ONE,
        Vec3::new(0.75, 0.85, 1.0),
        Vec3::new(1.0, 0.85, 0.65),
        palette::ACCENT_TEAL.to_linear().to_vec3(),
    ];
    (0..STAR_COUNT)
        .map(|_| {
            let (face, u, v) = direction_to_face(rng.unit_vector());
            // Potenzgesetz: viele schwache, wenige helle Sterne.
            let intensity = 0.15 + 2.2 * rng.next_f32().powf(9.0);
            // Meist weiß, bläulich und warm; selten ein türkiser Akzentstern.
            let roll = rng.next_f32();
            let tint = if roll > 0.97 {
                tints[3]
            } else {
                tints[(roll * 3.0) as usize % 3]
            };
            Star {
                face,
                x: (u * 0.5 + 0.5) * FACE_SIZE as f32,
                y: (v * 0.5 + 0.5) * FACE_SIZE as f32,
                color: tint,
                intensity,
            }
        })
        .collect()
}

fn render_face(face: usize, stars: &[Star]) -> Vec<u8> {
    let size = FACE_SIZE as usize;
    let space = palette::SPACE.to_linear().to_vec3();
    let violet = Vec3::new(0.16, 0.08, 0.30);
    let teal = palette::ACCENT_TEAL.to_linear().to_vec3() * 0.12;

    // Hintergrund: Navy plus ein weiches Nebelband.
    let mut pixels = vec![Vec3::ZERO; size * size];
    for y in 0..size {
        for x in 0..size {
            let u = (x as f32 + 0.5) / size as f32 * 2.0 - 1.0;
            let v = (y as f32 + 0.5) / size as f32 * 2.0 - 1.0;
            let d = face_direction(face, u, v).normalize();

            // Nebel um einen geneigten Großkreis (eine „Milchstraße“).
            let band = (1.0 - d.dot(Vec3::new(0.3, 0.9, 0.3).normalize()).abs()).powf(3.0);
            let cloud = noise::fbm(d * 2.2, 5, SEED);
            let wisps = noise::fbm(d * 6.0 + Vec3::splat(7.0), 4, SEED ^ 0xabcd);
            let density = ((cloud - 0.45) * 2.2).clamp(0.0, 1.0) * (0.35 + 0.65 * band);
            let hue = noise::value_noise(d * 1.5, SEED ^ 0x1234);
            let nebula = violet.lerp(teal, hue) * density * (0.6 + 0.6 * wisps);

            pixels[y * size + x] = space + nebula;
        }
    }

    for star in stars.iter().filter(|star| star.face == face) {
        let radius = if star.intensity > 1.0 { 2 } else { 1 };
        let (cx, cy) = (star.x as i32, star.y as i32);
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                let (px, py) = (cx + dx, cy + dy);
                if px < 0 || py < 0 || px >= size as i32 || py >= size as i32 {
                    continue;
                }
                let dist2 = (px as f32 + 0.5 - star.x).powi(2) + (py as f32 + 0.5 - star.y).powi(2);
                let falloff = (-dist2 * 1.6).exp();
                pixels[py as usize * size + px as usize] += star.color * star.intensity * falloff;
            }
        }
    }

    pixels
        .into_iter()
        .flat_map(|linear| {
            let srgb = Color::linear_rgb(linear.x, linear.y, linear.z).to_srgba();
            [
                (srgb.red.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.green.clamp(0.0, 1.0) * 255.0).round() as u8,
                (srgb.blue.clamp(0.0, 1.0) * 255.0).round() as u8,
                255,
            ]
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flaeche_mapping_round_trip() {
        for face in 0..6 {
            for (u, v) in [(0.0, 0.0), (0.5, -0.25), (-0.9, 0.8)] {
                let (back_face, back_u, back_v) = direction_to_face(face_direction(face, u, v));
                assert_eq!(back_face, face);
                assert!((back_u - u).abs() < 1e-5 && (back_v - v).abs() < 1e-5);
            }
        }
    }
}
