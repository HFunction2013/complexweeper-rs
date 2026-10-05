// Asset loading: decode the PNG atlas with the `image` crate, parse slot
// definitions from JSON, and provide sprite lookup by name / clue value.

use anyhow::{anyhow, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct AtlasJson {
    pub version: u32,
    pub width: u32,
    pub height: u32,
    pub slots: Vec<Slot>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Slot {
    pub name: String,
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub struct Atlas {
    pub width: u32,
    pub height: u32,
    /// RGBA8 pixel data, row-major, top-left origin.
    pub pixels: Vec<u8>,
    pub slots: HashMap<String, Slot>,
    /// Lookup from clue D value to sprite name (Complex mode).
    pub num_by_d: HashMap<i16, String>,
}

impl Atlas {
    pub fn from_json_png(json_bytes: &[u8], png_bytes: &[u8]) -> Result<Self> {
        let aj: AtlasJson = serde_json::from_slice(json_bytes)?;
        let img = image::load_from_memory(png_bytes)?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        if w != aj.width || h != aj.height {
            return Err(anyhow!(
                "Atlas dimensions mismatch: JSON says {}x{}, PNG is {}x{}",
                aj.width, aj.height, w, h
            ));
        }
        let mut slots = HashMap::new();
        for s in &aj.slots {
            slots.insert(s.name.clone(), s.clone());
        }
        let mut num_by_d = HashMap::new();
        for s in &aj.slots {
            if let Some(rest) = s.name.strip_prefix("num_") {
                if let Ok(d) = rest.parse::<i16>() {
                    num_by_d.insert(d, s.name.clone());
                }
            }
        }
        Ok(Atlas {
            width: w,
            height: h,
            pixels: rgba.into_raw(),
            slots,
            num_by_d,
        })
    }

    pub fn slot(&self, name: &str) -> Option<&Slot> {
        self.slots.get(name)
    }

    /// Get the RGBA sub-rectangle for a slot. Returns a Vec<u8> of RGBA data.
    pub fn slot_pixels(&self, name: &str) -> Option<Vec<u8>> {
        let s = self.slots.get(name)?;
        let mut out = Vec::with_capacity((s.w * s.h * 4) as usize);
        for row in 0..s.h {
            let src_y = s.y + row;
            let start = ((src_y * self.width + s.x) * 4) as usize;
            let end = start + (s.w * 4) as usize;
            out.extend_from_slice(&self.pixels[start..end]);
        }
        Some(out)
    }

    /// Sprite name for a clue value in Complex mode.
    pub fn num_sprite(&self, d: i16) -> Option<&str> {
        self.num_by_d.get(&d).map(|s| s.as_str())
    }
}

// Built-in atlas data (embedded at compile time).
pub const ATLAS_JSON: &[u8] = include_bytes!("../assets/atlas.json");
pub const ATLAS_PNG: &[u8] = include_bytes!("../assets/atlas.png");

pub fn load_builtin() -> Result<Atlas> {
    Atlas::from_json_png(ATLAS_JSON, ATLAS_PNG)
}
