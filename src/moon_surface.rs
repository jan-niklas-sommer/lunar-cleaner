//! Vorschau der Mond-Region als ein großes flaches Mesh, gesampelt aus den
//! echten LOLA-Höhen plus prozeduralen Kleinkratern (`moon_data`).
//!
//! Das ist noch nicht das `DustChunk`-System der Spezifikation — es beweist
//! die Datenquelle und liefert den Referenz-Boden, auf dem die Chunks dann
//! aufsetzen. `DustChunk`s sampeln dieselbe `Region` und übernehmen exakt
//! diese Höhen; nur die Auflösung wird feiner (0,25-m-Zellen statt 8 m).

use bevy::prelude::*;

use crate::moon_data::{self, Region};
use crate::palette;

/// Kantenlänge der Startregion in Metern: 7 km × 7 km ≈ 49 km² (Zielgröße
/// laut `design_wirtschaft_und_progression.md`, ~50 km²).
pub const REGION_SIZE: f64 = 7_000.0;
/// Auflösung des Vorschau-Meshes (Meter pro Quad). Grob gewählt, damit das
/// 49-km²-Mesh beim Start schnell steht; dieses Mesh dient später als
/// Fern-LOD, während die feinen `DustChunk`s (0,5 m) nahe an der Kamera
/// übernehmen.
const MESH_STEP: f64 = 16.0;

/// Die Spielregion; von `main` eingesetzt, überall lesbar.
#[derive(Resource)]
pub struct PlayRegion(pub Region);

pub struct MoonSurfacePlugin;

impl Plugin for MoonSurfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_surface);
    }
}

fn spawn_surface(
    region: Res<PlayRegion>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let size = region.0.size;
    let quads_per_side = (size / MESH_STEP) as usize;
    let verts_per_side = quads_per_side + 1;

    // Höhen einmal samplen; Mare-Anteil und Steilheit färben den Boden.
    let mut heights = vec![0.0f32; verts_per_side * verts_per_side];
    for iz in 0..verts_per_side {
        let z = -size / 2.0 + iz as f64 * MESH_STEP;
        for ix in 0..verts_per_side {
            let x = -size / 2.0 + ix as f64 * MESH_STEP;
            heights[iz * verts_per_side + ix] = region.0.height(x, z) as f32;
        }
    }

    let mut positions = Vec::with_capacity(heights.len());
    let mut colors = Vec::with_capacity(heights.len());
    for iz in 0..verts_per_side {
        let z = -size / 2.0 + iz as f64 * MESH_STEP;
        for ix in 0..verts_per_side {
            let x = -size / 2.0 + ix as f64 * MESH_STEP;
            let index = iz * verts_per_side + ix;
            positions.push([x as f32, heights[index], z as f32]);

            // Farbe wie im spcgame-Terrain: Grau im Hochland, dunkler auf
            // den Maria, helle Bruchkanten an Steilhängen.
            let dir = region.0.dir(x, z);
            let mare = moon_data::mare(dir) as f32;
            let base = palette::ROCK.mix(&palette::MOON_MARE, (mare * 1.4).clamp(0.0, 1.0));
            let slope = slope(&heights, verts_per_side, ix, iz, MESH_STEP as f32);
            let bright = palette::ICE;
            let color = base.mix(&bright, (slope / 35.0).clamp(0.0, 1.0) * 0.25);
            colors.push(color.to_linear().to_f32_array());
        }
    }

    let mut indices = Vec::with_capacity(quads_per_side * quads_per_side * 6);
    for iz in 0..quads_per_side {
        for ix in 0..quads_per_side {
            let (a, b, c, d) = (
                (iz * verts_per_side + ix) as u32,
                (iz * verts_per_side + ix + 1) as u32,
                ((iz + 1) * verts_per_side + ix) as u32,
                ((iz + 1) * verts_per_side + ix + 1) as u32,
            );
            indices.extend_from_slice(&[a, c, b, b, c, d]);
        }
    }

    let mut mesh = Mesh::new(
        bevy::render::mesh::PrimitiveTopology::TriangleList,
        bevy::asset::RenderAssetUsages::RENDER_WORLD,
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(bevy::render::mesh::Indices::U32(indices));
    mesh.compute_smooth_normals();

    commands.spawn((
        Name::new("Mondoberfläche (echte Höhen)"),
        Mesh3d(meshes.add(mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 0.95,
            reflectance: 0.05,
            ..default()
        })),
    ));
}

/// Steilheit in Grad an einem Gitterpunkt, aus den Nachbarhöhen.
fn slope(heights: &[f32], side: usize, ix: usize, iz: usize, step: f32) -> f32 {
    let at = |ix: usize, iz: usize| heights[iz * side + ix];
    let (left, right) = (ix.max(1) - 1, (ix + 1).min(side - 1));
    let (down, up) = (iz.max(1) - 1, (iz + 1).min(side - 1));
    let dx = (at(right, iz) - at(left, iz)) / (step * (right - left) as f32);
    let dz = (at(ix, up) - at(ix, down)) / (step * (up - down) as f32);
    dx.hypot(dz).atan().to_degrees()
}

/// Bodennähe über dem Terrain an `(x, z)` — Startpunkt für Kamera und später
/// die Spieler-Spawnhöhe.
pub fn ground_height(region: &Region, x: f64, z: f64) -> f32 {
    region.height(x, z) as f32
}
