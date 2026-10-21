//! Echte Höhenkarten (portiert aus spcgame): Mond (LRO LOLA) und Erde
//! (NOAA ETOPO1), gemeinfrei, siehe `assets/<body>/README.md`.
//!
//! Beide Karten teilen ein Layout: equirektangulares Knoten-Raster, Zeile 0
//! bei 90° N, Spalte 0 bei 180° W, Längengrad wächst nach Osten.

use std::sync::OnceLock;

use bevy::asset::RenderAssetUsages;
use bevy::image::{CompressedImageFormats, Image, ImageSampler, ImageType};
use bevy::math::DVec3;
use bevy::render::render_resource::TextureFormat;

const MOON_HEIGHTS: &[u8] = include_bytes!("../assets/moon/heights.png");
const EARTH_HEIGHTS: &[u8] = include_bytes!("../assets/earth/heights.png");
const EARTH_CLIMATE: &[u8] = include_bytes!("../assets/earth/climate.png");
/// Wird zu den Höhen in den Dateien addiert, damit sie positiv bleiben
/// (siehe die READMEs in `assets/`).
const MOON_OFFSET: f64 = 32_768.0;
const EARTH_OFFSET: f64 = 11_000.0;

/// Längen- und Breitengrad in Grad einer Richtung in Körper-Koordinaten:
/// +Y ist Norden, Längengrad wächst nach Osten.
pub fn lon_lat(dir: DVec3) -> (f64, f64) {
    let lon = f64::atan2(-dir.z, dir.x).to_degrees();
    let lat = dir.y.clamp(-1.0, 1.0).asin().to_degrees();
    (lon, lat)
}

/// Richtung zu Längen- und Breitengrad in Grad (Inverse von [`lon_lat`]).
pub fn dir_of(lon: f64, lat: f64) -> DVec3 {
    let (lon, lat) = (lon.to_radians(), lat.to_radians());
    DVec3::new(lat.cos() * lon.cos(), lat.sin(), -lat.cos() * lon.sin())
}

/// Dekodiert eine eingebettete PNG-Karte.
pub fn decode(bytes: &[u8]) -> Image {
    Image::from_buffer(
        bytes,
        ImageType::Extension("png"),
        CompressedImageFormats::NONE,
        false,
        ImageSampler::Default,
        RenderAssetUsages::default(),
    )
    .expect("eingebettete Karte dekodiert")
}

/// Die Grauwerte einer 16-Bit-Graustufen-Karte, Zeile für Zeile.
pub fn gray16(image: &Image) -> Vec<u16> {
    let channels = match image.texture_descriptor.format {
        TextureFormat::R16Unorm => 1,
        TextureFormat::Rgba16Unorm => 4,
        other => panic!("unerwartetes Höhenkarten-Format {other:?}"),
    };
    image
        .data
        .as_deref()
        .expect("Kartendaten")
        .chunks_exact(2 * channels)
        .map(|p| u16::from_le_bytes([p[0], p[1]]))
        .collect()
}

/// Größe eines Karten-Rasters.
#[derive(Clone, Copy)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
}

impl Grid {
    /// Bilineare Interpolation eines Knotenwerts bei `dir`.
    pub fn bilinear(self, dir: DVec3, value: impl Fn(usize) -> f64) -> f64 {
        self.corners(dir)
            .iter()
            .map(|&(node, weight)| value(node) * weight)
            .sum()
    }

    /// Die vier Knoten um `dir` mit ihren bilinearen Gewichten.
    pub fn corners(self, dir: DVec3) -> [(usize, f64); 4] {
        let (lon, lat) = lon_lat(dir);
        let step = 360.0 / self.width as f64;
        let last = (self.height - 1) as f64;
        let x = (lon + 180.0) / step;
        let y = ((90.0 - lat) / step).clamp(0.0, last);
        let (x0, y0) = (x.floor(), y.floor().min(last - 1.0));
        let (fx, fy) = (x - x0, y - y0);
        let column = |c: f64| (c as i64).rem_euclid(self.width as i64) as usize;
        let (c0, c1, r0) = (column(x0), column(x0 + 1.0), y0 as usize);
        let at = |r: usize, c: usize| r * self.width + c;
        [
            (at(r0, c0), (1.0 - fx) * (1.0 - fy)),
            (at(r0, c1), fx * (1.0 - fy)),
            (at(r0 + 1, c0), (1.0 - fx) * fy),
            (at(r0 + 1, c1), fx * fy),
        ]
    }
}

/// Echte Höhen eines Körpers.
pub struct HeightMap {
    grid: Grid,
    /// Höhe in Einheiten von `meters_per_unit`, plus `offset`.
    values: Vec<u16>,
    offset: f64,
    meters_per_unit: f64,
}

impl HeightMap {
    pub fn decode(bytes: &[u8], offset: f64) -> Self {
        let image = decode(bytes);
        let grid = Grid {
            width: image.width() as usize,
            height: image.height() as usize,
        };
        let values = gray16(&image);
        assert_eq!(values.len(), grid.width * grid.height);
        HeightMap {
            grid,
            values,
            offset,
            meters_per_unit: 1.0,
        }
    }

    /// Raster der Karte.
    #[allow(dead_code)] // API für die kommenden Spielsysteme
    pub fn grid(&self) -> Grid {
        self.grid
    }

    /// Echte Höhe in Metern bei `dir`.
    pub fn height(&self, dir: DVec3) -> f64 {
        self.grid.bilinear(dir, |i| {
            (self.values[i] as f64 - self.offset) * self.meters_per_unit
        })
    }

    /// Echte Höhe in Metern des Knotens `index`.
    #[allow(dead_code)] // API für die kommenden Spielsysteme
    pub fn node(&self, index: usize) -> f64 {
        (self.values[index] as f64 - self.offset) * self.meters_per_unit
    }
}

/// Die echte Mond-Höhenkarte, beim ersten Gebrauch dekodiert.
pub fn moon() -> &'static HeightMap {
    static MOON: OnceLock<HeightMap> = OnceLock::new();
    MOON.get_or_init(|| HeightMap::decode(MOON_HEIGHTS, MOON_OFFSET))
}

/// Die echte Erd-Höhenkarte (ETOPO1, Meeresspiegel = 0).
pub fn earth_heights() -> &'static HeightMap {
    static EARTH: OnceLock<HeightMap> = OnceLock::new();
    EARTH.get_or_init(|| HeightMap::decode(EARTH_HEIGHTS, EARTH_OFFSET))
}

/// Rohdaten der Klima-Karte der Erde (RGBA: R = Wüste, G = Eis, B = See)
/// samt Raster; wird von `sky` zur Erd-Textur gelesen.
pub fn earth_climate() -> (Grid, Vec<[u8; 3]>) {
    static CLIMATE: OnceLock<(Grid, Vec<[u8; 3]>)> = OnceLock::new();
    CLIMATE.get_or_init(|| {
        let image = decode(EARTH_CLIMATE);
        let grid = Grid {
            width: image.width() as usize,
            height: image.height() as usize,
        };
        let climate = image
            .data
            .as_deref()
            .expect("Klimadaten")
            .as_chunks::<4>()
            .0
            .iter()
            .map(|p| [p[0], p[1], p[2]])
            .collect::<Vec<_>>();
        assert_eq!(climate.len(), grid.width * grid.height);
        (grid, climate)
    })
    .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn koordinaten_round_trip() {
        for (lon, lat) in [(0.0, 0.0), (13.4, 52.5), (-74.0, 40.7), (151.2, -33.9)] {
            let (lon2, lat2) = lon_lat(dir_of(lon, lat));
            assert!((lon - lon2).abs() < 1e-9 && (lat - lat2).abs() < 1e-9);
        }
    }

    /// Berühmte Orte haben ihre echten Höhen.
    #[test]
    fn landmarken_haben_ihre_echten_hoehen() {
        let moon_height = |lon, lat| moon().height(dir_of(lon, lat));
        assert!(
            moon_height(-11.36, -43.31) < -1_000.0,
            "Tycho-Boden"
        );
        assert!(
            moon_height(-170.0, -55.0) < -4_000.0,
            "Südpol-Aitken-Becken"
        );
        // Apollo 11 landete im Mare Tranquillitatis: tief und eben.
        assert!(
            (-4_000.0..-1_000.0).contains(&moon_height(23.473, 0.674)),
            "Mare Tranquillitatis"
        );

        let earth_height = |lon, lat| earth_heights().height(dir_of(lon, lat));
        assert!(earth_height(86.9, 27.9) > 5_000.0, "Himalaya");
        assert!(earth_height(-30.0, 30.0) < -3_000.0, "Atlantik");
        assert!(earth_height(10.0, 51.0) > 0.0, "Deutschland");
    }
}
