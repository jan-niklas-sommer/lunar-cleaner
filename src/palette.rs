//! Farb-Palette (Teilmenge aus spcgame übernommen).
//!
//! Alle Farben im Spiel kommen von hier, damit der Look konsistent bleibt.

use bevy::prelude::*;

/// Tiefes Navy als Weltraum-Hintergrund (nie rein schwarz).
pub const SPACE: Color = Color::srgb_u8(0x14, 0x1A, 0x33);

pub const SUN: Color = Color::srgb_u8(0xFF, 0xA5, 0x3D);
pub const SUN_CORE: Color = Color::srgb_u8(0xFF, 0xD2, 0x7A);

// Erde (Himmel-Asset): Ozean, Land, Eis.
pub const EARTH_OCEAN: Color = Color::srgb_u8(0x3D, 0x8F, 0xD9);
pub const EARTH_OCEAN_DEEP: Color = Color::srgb_u8(0x1E, 0x4E, 0x8C);
pub const EARTH_LAND: Color = Color::srgb_u8(0x6F, 0xAF, 0x4F);
pub const DESERT_GOLD: Color = Color::srgb_u8(0xE3, 0xB8, 0x65);
pub const EARTH_LAKE: Color = Color::srgb_u8(0x2E, 0x77, 0xB8);
pub const SNOW: Color = Color::srgb_u8(0xF2, 0xF6, 0xFA);
pub const CLOUD: Color = Color::srgb_u8(0xF5, 0xF7, 0xFA);

// Mond-Terrain.
pub const ROCK: Color = Color::srgb_u8(0x9C, 0x8A, 0x94);
pub const MOON_MARE: Color = Color::srgb_u8(0x66, 0x5E, 0x6E);
pub const ICE: Color = Color::srgb_u8(0xCF, 0xE3, 0xF2);

/// Teal-Akzent (glühendes Erz, UI-Highlights).
pub const ACCENT_TEAL: Color = Color::srgb_u8(0x5F, 0xE3, 0xC8);

/// Leicht bläuliches Umgebungslicht, damit Nachtseiten nie schwarz werden.
pub const AMBIENT: Color = Color::srgb_u8(0x8C, 0xA8, 0xF0);
