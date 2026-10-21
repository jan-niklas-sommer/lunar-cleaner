//! Der echte Mond als Höhenfeld (Rezept aus spcgame portiert, auf echten
//! Mond-Radius umgerechnet): die großen Formen aus der echten LRO-LOLA-Karte
//! (`height_maps::moon`), die kleinen Krater prozedural obendrauf, denn die
//! Karte löst bei 0,125° Rasterweite (≈ 3,7 km/Pixel am Äquator) nur Krater
//! ab etlichen Kilometern Durchmesser auf.
//!
//! Für das Spiel wird eine [`Region`] auf einer Tangentialebene gesampelt:
//! `x` wächst nach Osten, `z` nach Norden, `height(x, z)` ist die Höhe über
//! der Ebene durch den Regionsmittelpunkt (Mondkrümmung herausgerechnet).
//! Das passt direkt auf das flache `DustChunk`-Raster aus der Spezifikation.

use bevy::math::DVec3;

use crate::height_maps::{self, dir_of, lon_lat};
use crate::noise;

/// Echter Mond-Radius in Metern.
pub const MOON_RADIUS: f64 = 1_737_400.0;

/// Welt-Seed für die prozeduralen Anteile (Krater, Detail).
const SEED: u32 = 0x600d_c0de;

/// Tiefenverhältnis der prozeduralen Krater (Randhöhe ≈ 0,3 der Tiefe).
const CRATER_DEPTH_RATIO: f64 = 0.07;

/// Kraterschichten: Zellen der Größe `cell` Meter, ein Krater in `density`
/// der Zellen. Zwei Größen, wie im spcgame-Terrain: ~1 km- und ~300 m-Krater,
/// die die LOLA-Karte nicht mehr hergibt.
const CRATER_LAYERS: [(f64, f64); 2] = [(2_400.0, 0.45), (700.0, 0.4)];

fn smoothstep(edge0: f64, edge1: f64, x: f64) -> f64 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn seed(n: u32) -> u32 {
    noise::hash_u32(SEED ^ n.wrapping_mul(0x9e37_79b9))
}

/// Höhenprofil eines Kraters: Senke mit Rand, in Einheiten des Kraterradius;
/// verschwindet außerhalb 1,7 Radien, damit Krater in der Nachbarzelle
/// bleiben (Voraussetzung für das 3×3×3-Zellfenster unten).
fn crater_profile(d: f64) -> f64 {
    const RIM: f64 = 0.3;
    if d < 1.0 {
        -1.0 + (RIM + 1.0) * smoothstep(0.6, 0.95, d)
    } else {
        RIM * (1.0 - smoothstep(1.15, 1.7, d))
    }
}

/// Runde Merkmale in einem `density`-Anteil der Zellen der Größe `cell`
/// Meter: `feature` erhält den Abstand in Merkmalsradien, den Radius in
/// Zellen (0,2–0,45) und einen Zufallswert.
fn cellular(
    dir: DVec3,
    cell: f64,
    density: f64,
    layer: u32,
    feature: impl Fn(f64, f64, f64) -> f64,
) -> f64 {
    let seed = seed(100 + layer);
    let p = dir * (MOON_RADIUS / cell);
    let base = p.floor();
    let mut sum = 0.0;
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let (x, y, z) = (base.x as i64 + dx, base.y as i64 + dy, base.z as i64 + dz);
                let traits = noise::cell_point(x, y, z, seed ^ 0x2545_f491);
                if traits.x > density {
                    continue;
                }
                let size = 0.2 + 0.25 * traits.y;
                let center =
                    DVec3::new(x as f64, y as f64, z as f64) + noise::cell_point(x, y, z, seed);
                sum += feature(center.distance(p) / size, size, traits.z);
            }
        }
    }
    sum
}

/// Eine Kraterschicht: Höhenversatz bei `dir`.
fn craters(dir: DVec3, cell: f64, density: f64, layer: u32) -> f64 {
    cellular(dir, cell, density, layer, |d, size, _| {
        crater_profile(d) * size * cell * CRATER_DEPTH_RATIO
    })
}

/// Kleines Feindetail (Wellen von ~200 m, wenige Meter Amplitude).
fn detail(dir: DVec3, amplitude: f64) -> f64 {
    noise::fbm64(dir * (MOON_RADIUS / 220.0), 2, seed(99)) * amplitude
}

/// Maria-Anteil bei `dir` (0–1): die echten, tiefen, ebenen Basalte.
/// Nicht das tiefste Becken (Südpol-Aitken) — wie im spcgame-Terrain.
pub fn mare(dir: DVec3) -> f64 {
    let real = height_maps::moon().height(dir);
    smoothstep(-800.0, -1_800.0, real) * (1.0 - smoothstep(-3_500.0, -4_500.0, real))
}

/// Höhe über dem Mond-Radius bei `dir` in Metern: echte LOLA-Höhe plus die
/// prozeduralen Krater (wenige auf den Maria) plus Feindetail. Realmaßstab,
/// also Faktor 1 statt spcgames komprimierter Skala.
pub fn elevation(dir: DVec3) -> f64 {
    let crater_amount = 1.0 - 0.75 * mare(dir);
    let craters: f64 = CRATER_LAYERS
        .iter()
        .enumerate()
        .map(|(i, &(cell, density))| craters(dir, cell, density, 3 + i as u32))
        .sum();
    height_maps::moon().height(dir) + craters * crater_amount + detail(dir, 2.0)
}

/// Eine rechteckige Spielregion auf der Mondoberfläche, in Metern.
#[derive(Clone, Copy, Debug)]
pub struct Region {
    /// Richtung des Mittelpunkts (Einheitskugel).
    pub center: DVec3,
    /// Kantenlänge in Metern.
    pub size: f64,
}

impl Region {
    /// Region um Längen-/Breitengrad (Grad) mit Kantenlänge in Metern.
    pub fn at(lon: f64, lat: f64, size: f64) -> Self {
        Region {
            center: dir_of(lon, lat),
            size,
        }
    }

    /// Ost- und Nordrichtung der Region (Tangentialebene).
    pub fn east_north(&self) -> (DVec3, DVec3) {
        let east = DVec3::Y.cross(self.center).normalize();
        let north = self.center.cross(east);
        (east, north)
    }

    /// Richtung des Punkts `x` Meter östlich, `z` Meter nördlich des
    /// Mittelpunkts.
    pub fn dir(&self, x: f64, z: f64) -> DVec3 {
        let (east, north) = self.east_north();
        let angle = 1.0 / MOON_RADIUS;
        (self.center + east * (x * angle) + north * (z * angle)).normalize()
    }

    /// Längen-/Breitengrad eines Regionpunkts (für die Anzeige).
    #[allow(dead_code)] // API für die kommenden Spielsysteme
    pub fn lon_lat(&self, x: f64, z: f64) -> (f64, f64) {
        lon_lat(self.dir(x, z))
    }

    /// Höhe in Metern über der Tangentialebene durch den Mittelpunkt
    /// (Mondkrümmung herausgerechnet, die Region liegt „flach“).
    pub fn height(&self, x: f64, z: f64) -> f64 {
        let dir = self.dir(x, z);
        elevation(dir) + MOON_RADIUS * (dir.dot(self.center) - 1.0)
    }
}

/// Startregion des Spiels: Apollo-11-Landestelle im Mare Tranquillitatis.
pub fn apollo_11(size: f64) -> Region {
    Region::at(23.473, 0.674, size)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apollo_11_liegt_im_mare_und_ist_eben() {
        let region = apollo_11(4_096.0);
        assert!(mare(region.center) > 0.8, "Mare Tranquillitatis");

        // Die Region liegt um die echte Höhe herum, ohne Ausreißer:
        // Krater bis ~75 m, LOLA-Variation im Mare gering.
        let real = height_maps::moon().height(region.center);
        for &(x, z) in &[(0.0, 0.0), (1_000.0, -500.0), (-2_000.0, 1_500.0)] {
            let h = region.height(x, z);
            assert!(
                (real - 120.0..real + 120.0).contains(&h),
                "({x}, {z}): {h} vs. echte Höhe {real}"
            );
        }
    }

    /// Das Terrain ist glatt: kein Schritt zwischen benachbarten Metern
    /// (f64-Rechnung, wie im spcgame-Kommentar begründet).
    #[test]
    fn region_ist_glatt() {
        let region = apollo_11(4_096.0);
        for &(x, z) in &[(0.0, 0.0), (512.5, -300.25), (-1_024.0, 2_048.0)] {
            let step = region.height(x + 0.25, z) - region.height(x, z);
            assert!(step.abs() < 3.0, "Schritt von {step} m bei ({x}, {z})");
        }
    }

    /// Hocheinden nebenan: deutlich mehr Krater als im Mare. Gesampelt wird
    /// ein 4x4-km-Gitter, denn ein einzelner Ring streut zu stark.
    #[test]
    fn hocheinden_haben_mehr_krater_als_maria() {
        let mare_dir = dir_of(23.473, 0.674);
        let highland_dir = dir_of(150.0, -30.0);
        assert!(mare(mare_dir) > 0.8);
        assert!(mare(highland_dir) < 0.1, "Hochebene auf der Fernseite");

        // Krateranteil (Höhe minus echte Karte) über je 4x4 km.
        let crater_relief = |dir| {
            let (east, north) = (
                DVec3::Y.cross(dir).normalize(),
                dir.cross(DVec3::Y.cross(dir).normalize()),
            );
            let mut lo = f64::MAX;
            let mut hi = f64::MIN;
            for iz in 0..41 {
                for ix in 0..41 {
                    let (x, z) = ((ix as f64 - 20.0) * 100.0, (iz as f64 - 20.0) * 100.0);
                    let d =
                        (dir + east * (x / MOON_RADIUS) + north * (z / MOON_RADIUS)).normalize();
                    let c = elevation(d) - height_maps::moon().height(d);
                    lo = lo.min(c);
                    hi = hi.max(c);
                }
            }
            hi - lo
        };
        let mare_relief = crater_relief(mare_dir);
        let highland_relief = crater_relief(highland_dir);
        assert!(
            highland_relief > 2.0 * mare_relief,
            "Hocheinde {highland_relief:.1} m vs. Mare {mare_relief:.1} m"
        );
    }
}
