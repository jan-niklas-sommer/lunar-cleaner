//! Kleine deterministische Noise- und RNG-Helfer (portiert aus spcgame).
//!
//! Das f32-Value-Noise färbt Himmelskörper und erzeugt die Sternen-Cubemap;
//! das f64-Gradient-Noise unten baut das Mond-Terrain: Bei echtem Mond-Radius
//! (1 737 km) ist eine f32-Richtung nur auf ~2 cm genau, feine Details
//! würden Stufen bilden. Eigenbau statt Noise-Crate: wenige Dutzend Zeilen,
//! plattformübergreifend deterministisch, komplett f64.

use bevy::math::{DVec3, Vec3};

/// Integer-Hash (lowbias32 von Chris Wellons), gute Avalanche-Streuung.
pub fn hash_u32(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^= x >> 16;
    x
}

/// Hash nach `[0, 1]`.
pub fn hash_unit(x: u32) -> f32 {
    hash_u32(x) as f32 / u32::MAX as f32
}

fn lattice(x: i32, y: i32, z: i32, seed: u32) -> f32 {
    let h = (x as u32).wrapping_mul(0x8da6_b343)
        ^ (y as u32).wrapping_mul(0xd816_3841)
        ^ (z as u32).wrapping_mul(0xcb1a_b31f)
        ^ seed;
    hash_unit(h)
}

/// Glattes 3D-Value-Noise in `[0, 1]`.
pub fn value_noise(p: Vec3, seed: u32) -> f32 {
    let i = p.floor();
    let f = p - i;
    let u = f * f * (Vec3::splat(3.0) - 2.0 * f);
    let (x, y, z) = (i.x as i32, i.y as i32, i.z as i32);

    let corner = |dx, dy, dz| lattice(x + dx, y + dy, z + dz, seed);
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;

    let x00 = lerp(corner(0, 0, 0), corner(1, 0, 0), u.x);
    let x10 = lerp(corner(0, 1, 0), corner(1, 1, 0), u.x);
    let x01 = lerp(corner(0, 0, 1), corner(1, 0, 1), u.x);
    let x11 = lerp(corner(0, 1, 1), corner(1, 1, 1), u.x);
    lerp(lerp(x00, x10, u.y), lerp(x01, x11, u.y), u.z)
}

/// Fraktales (fBm) Value-Noise in `[0, 1]`.
pub fn fbm(p: Vec3, octaves: u32, seed: u32) -> f32 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for octave in 0..octaves {
        sum += amp * value_noise(p * freq, seed.wrapping_add(octave));
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

/// Minimaler Zähler-basierter RNG zum deterministischen Streuen.
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed)
    }

    /// Gleichverteilt in `[0, 1]`.
    pub fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9e37_79b9);
        hash_unit(self.0)
    }

    /// Gleichverteilte Richtung auf der Einheitskugel.
    pub fn unit_vector(&mut self) -> Vec3 {
        let z = self.next_f32() * 2.0 - 1.0;
        let phi = self.next_f32() * std::f32::consts::TAU;
        let r = (1.0 - z * z).max(0.0).sqrt();
        Vec3::new(r * phi.cos(), r * phi.sin(), z)
    }
}

fn lattice_hash(x: i64, y: i64, z: i64, seed: u32) -> u32 {
    hash_u32(
        (x as u32).wrapping_mul(0x8da6_b343)
            ^ (y as u32).wrapping_mul(0xd816_3841)
            ^ (z as u32).wrapping_mul(0xcb1a_b31f)
            ^ seed,
    )
}

/// Skalarprodukt des Versatzes mit einem von Perlins 12 Würfelkanten-Gradienten.
fn gradient(hash: u32, d: DVec3) -> f64 {
    match hash % 12 {
        0 => d.x + d.y,
        1 => -d.x + d.y,
        2 => d.x - d.y,
        3 => -d.x - d.y,
        4 => d.x + d.z,
        5 => -d.x + d.z,
        6 => d.x - d.z,
        7 => -d.x - d.z,
        8 => d.y + d.z,
        9 => -d.y + d.z,
        10 => d.y - d.z,
        _ => -d.y - d.z,
    }
}

/// 3D-Gradient-Noise (improved Perlin) in etwa `[-1, 1]`, komplett f64.
pub fn perlin(p: DVec3, seed: u32) -> f64 {
    let cell = p.floor();
    let f = p - cell;
    let u = f * f * f * (f * (f * 6.0 - 15.0) + 10.0);
    let (x, y, z) = (cell.x as i64, cell.y as i64, cell.z as i64);
    let corner = |dx: i64, dy: i64, dz: i64| {
        let offset = f - DVec3::new(dx as f64, dy as f64, dz as f64);
        gradient(lattice_hash(x + dx, y + dy, z + dz, seed), offset)
    };
    let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
    let x00 = lerp(corner(0, 0, 0), corner(1, 0, 0), u.x);
    let x10 = lerp(corner(0, 1, 0), corner(1, 1, 0), u.x);
    let x01 = lerp(corner(0, 0, 1), corner(1, 0, 1), u.x);
    let x11 = lerp(corner(0, 1, 1), corner(1, 1, 1), u.x);
    lerp(lerp(x00, x10, u.y), lerp(x01, x11, u.y), u.z)
}

/// Fraktales Gradient-Noise in etwa `[-1, 1]` (die meisten Werte in ±0,5).
pub fn fbm64(p: DVec3, octaves: u32, seed: u32) -> f64 {
    let (mut sum, mut amp, mut freq, mut norm) = (0.0, 0.5, 1.0, 0.0);
    for octave in 0..octaves {
        // Jede Oktave ist verschoben, damit Gitterpunkte sich nicht decken.
        let shift = DVec3::new(17.31, -5.17, 9.73) * octave as f64;
        sum += amp * perlin(p * freq + shift, seed.wrapping_add(octave));
        norm += amp;
        amp *= 0.5;
        freq *= 2.0;
    }
    sum / norm
}

/// Hash einer Ganzzahl-Zelle auf einen Punkt in `[0, 1)³`.
pub fn cell_point(x: i64, y: i64, z: i64, seed: u32) -> DVec3 {
    let h = lattice_hash(x, y, z, seed);
    let unit = |h: u32| hash_u32(h) as f64 / (u32::MAX as f64 + 1.0);
    DVec3::new(unit(h), unit(h ^ 0x68e3_1da4), unit(h ^ 0xb529_7a4d))
}

/// Zelluläres (Worley) Noise: Abstände zum nächsten und zweitnächsten
/// Merkmalspunkt und die Id der nächsten Zelle.
#[allow(dead_code)] // API für die kommenden Spielsysteme
pub fn worley(p: DVec3, seed: u32) -> (f64, f64, u32) {
    let cell = p.floor();
    let (x, y, z) = (cell.x as i64, cell.y as i64, cell.z as i64);
    let (mut f1, mut f2, mut id) = (f64::MAX, f64::MAX, 0);
    for dx in -1..=1 {
        for dy in -1..=1 {
            for dz in -1..=1 {
                let (cx, cy, cz) = (x + dx, y + dy, z + dz);
                let point =
                    DVec3::new(cx as f64, cy as f64, cz as f64) + cell_point(cx, cy, cz, seed);
                let d = point.distance(p);
                if d < f1 {
                    f2 = f1;
                    f1 = d;
                    id = lattice_hash(cx, cy, cz, seed ^ 0x51ed_270b);
                } else if d < f2 {
                    f2 = d;
                }
            }
        }
    }
    (f1, f2, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perlin_ist_deterministisch_begrenzt_und_null_auf_gitterpunkten() {
        let mut rng = Rng::new(3);
        for _ in 0..10_000 {
            let p = rng.unit_vector().as_dvec3() * 1_000.0;
            let n = perlin(p, 7);
            assert_eq!(n, perlin(p, 7));
            assert!(n.abs() <= 1.1, "{n}");
        }
        assert_eq!(perlin(DVec3::new(3.0, -2.0, 5.0), 1), 0.0);
    }

    #[test]
    fn perlin_ist_glatt_bei_grossen_koordinaten() {
        // Mondoberfläche in Metern, die Größenordnung, in der das Terrain sampelt.
        let p = DVec3::new(250_123.4, -312_456.7, 4_321.0);
        let a = perlin(p, 1);
        let b = perlin(p + DVec3::splat(1e-6), 1);
        assert!((a - b).abs() < 1e-5);
    }

    #[test]
    fn worley_abstaende_sind_geordnet() {
        let mut rng = Rng::new(9);
        for _ in 0..1_000 {
            let p = rng.unit_vector().as_dvec3() * 50.0;
            let (f1, f2, _) = worley(p, 4);
            assert!(f1 <= f2 && f1 < 1.8);
        }
    }
}
