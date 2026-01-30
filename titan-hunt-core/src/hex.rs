//! Hexagonal grid coordinate system and utilities
//!
//! Uses axial coordinates (q, r) for flat-top hexagons with conversion
//! to cube coordinates for distance calculations.

use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// Axial hex coordinate using (q, r) system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HexCoord {
    pub q: i32,
    pub r: i32,
}

/// Cube coordinate for hex calculations (x + y + z = 0)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CubeCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

/// Direction offsets for the 6 hex directions (flat-top orientation)
/// Order: E, NE, NW, W, SW, SE
pub const AXIAL_DIRECTIONS: [(i32, i32); 6] = [
    (1, 0),   // East
    (1, -1),  // Northeast
    (0, -1),  // Northwest
    (-1, 0),  // West
    (-1, 1),  // Southwest
    (0, 1),   // Southeast
];

/// Facing direction for units on the hex grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Facing {
    East = 0,
    Northeast = 1,
    Northwest = 2,
    West = 3,
    Southwest = 4,
    Southeast = 5,
}

impl Facing {
    /// Get the opposite facing direction
    pub fn opposite(&self) -> Facing {
        match self {
            Facing::East => Facing::West,
            Facing::Northeast => Facing::Southwest,
            Facing::Northwest => Facing::Southeast,
            Facing::West => Facing::East,
            Facing::Southwest => Facing::Northeast,
            Facing::Southeast => Facing::Northwest,
        }
    }

    /// Get facing from index (0-5)
    pub fn from_index(index: u8) -> Option<Facing> {
        match index {
            0 => Some(Facing::East),
            1 => Some(Facing::Northeast),
            2 => Some(Facing::Northwest),
            3 => Some(Facing::West),
            4 => Some(Facing::Southwest),
            5 => Some(Facing::Southeast),
            _ => None,
        }
    }

    /// Get the index (0-5) for this facing
    pub fn index(&self) -> u8 {
        *self as u8
    }

    /// Get the angle in radians for this facing (0 = East, counter-clockwise)
    pub fn to_radians(&self) -> f64 {
        match self {
            Facing::East => 0.0,
            Facing::Northeast => PI / 3.0,
            Facing::Northwest => 2.0 * PI / 3.0,
            Facing::West => PI,
            Facing::Southwest => 4.0 * PI / 3.0,
            Facing::Southeast => 5.0 * PI / 3.0,
        }
    }

    /// Convert facing to sprite direction string (S, SW, W, NW, N, NE, E, SE)
    /// Note: Sprite directions use 8 directions, we map 6 hex facings to them
    pub fn to_sprite_direction(&self) -> &'static str {
        match self {
            Facing::East => "E",
            Facing::Northeast => "NE",
            Facing::Northwest => "NW",
            Facing::West => "W",
            Facing::Southwest => "SW",
            Facing::Southeast => "SE",
        }
    }

    /// Check if a target hex is in the front arc (3 hex sides in front)
    pub fn is_in_front_arc(&self, from: HexCoord, target: HexCoord) -> bool {
        let direction = from.direction_to(target);
        if let Some(dir) = direction {
            let diff = (dir.index() as i8 - self.index() as i8).rem_euclid(6);
            diff <= 1 || diff >= 5
        } else {
            true // Same hex counts as front
        }
    }

    /// Rotate clockwise by n steps
    pub fn rotate_cw(&self, steps: i32) -> Facing {
        let new_index = (self.index() as i32 - steps).rem_euclid(6) as u8;
        Facing::from_index(new_index).unwrap()
    }

    /// Rotate counter-clockwise by n steps
    pub fn rotate_ccw(&self, steps: i32) -> Facing {
        let new_index = (self.index() as i32 + steps).rem_euclid(6) as u8;
        Facing::from_index(new_index).unwrap()
    }
}

impl HexCoord {
    /// Create a new hex coordinate
    pub fn new(q: i32, r: i32) -> Self {
        HexCoord { q, r }
    }

    /// Origin hex at (0, 0)
    pub fn origin() -> Self {
        HexCoord { q: 0, r: 0 }
    }

    /// Convert to cube coordinates
    pub fn to_cube(&self) -> CubeCoord {
        CubeCoord {
            x: self.q,
            z: self.r,
            y: -self.q - self.r,
        }
    }

    /// Get all 6 neighboring hexes
    pub fn neighbors(&self) -> [HexCoord; 6] {
        [
            HexCoord::new(self.q + 1, self.r),      // East
            HexCoord::new(self.q + 1, self.r - 1),  // Northeast
            HexCoord::new(self.q, self.r - 1),      // Northwest
            HexCoord::new(self.q - 1, self.r),      // West
            HexCoord::new(self.q - 1, self.r + 1),  // Southwest
            HexCoord::new(self.q, self.r + 1),      // Southeast
        ]
    }

    /// Get neighbor in a specific direction
    pub fn neighbor(&self, facing: Facing) -> HexCoord {
        let (dq, dr) = AXIAL_DIRECTIONS[facing.index() as usize];
        HexCoord::new(self.q + dq, self.r + dr)
    }

    /// Calculate distance to another hex
    pub fn distance_to(&self, other: HexCoord) -> u32 {
        let a = self.to_cube();
        let b = other.to_cube();
        ((a.x - b.x).abs() + (a.y - b.y).abs() + (a.z - b.z).abs()) as u32 / 2
    }

    /// Get all hexes on a line to another hex
    pub fn line_to(&self, target: HexCoord) -> Vec<HexCoord> {
        let n = self.distance_to(target) as i32;
        if n == 0 {
            return vec![*self];
        }

        let mut results = Vec::with_capacity((n + 1) as usize);
        for i in 0..=n {
            let t = i as f64 / n as f64;
            let q = self.q as f64 + (target.q - self.q) as f64 * t;
            let r = self.r as f64 + (target.r - self.r) as f64 * t;
            results.push(hex_round(q, r));
        }
        results
    }

    /// Get the direction from this hex to another
    pub fn direction_to(&self, target: HexCoord) -> Option<Facing> {
        if *self == target {
            return None;
        }

        let dq = target.q - self.q;
        let dr = target.r - self.r;
        let angle = (dr as f64).atan2(dq as f64);

        // Convert angle to facing (0 = East, counter-clockwise)
        let normalized = (angle + 2.0 * PI) % (2.0 * PI);
        let index = ((normalized / (PI / 3.0) + 0.5) as i32).rem_euclid(6) as u8;
        Facing::from_index(index)
    }

    /// Convert hex coordinate to pixel position (flat-top orientation)
    pub fn to_pixel(&self, hex_size: f64) -> (f64, f64) {
        let x = hex_size * (3.0_f64.sqrt() * self.q as f64 + 3.0_f64.sqrt() / 2.0 * self.r as f64);
        let y = hex_size * (3.0 / 2.0 * self.r as f64);
        (x, y)
    }

    /// Convert pixel position to hex coordinate (flat-top orientation)
    pub fn from_pixel(x: f64, y: f64, hex_size: f64) -> HexCoord {
        let q = (3.0_f64.sqrt() / 3.0 * x - 1.0 / 3.0 * y) / hex_size;
        let r = (2.0 / 3.0 * y) / hex_size;
        hex_round(q, r)
    }
}

impl CubeCoord {
    /// Create a new cube coordinate
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        debug_assert!(x + y + z == 0, "Cube coordinates must sum to 0");
        CubeCoord { x, y, z }
    }

    /// Convert to axial coordinates
    pub fn to_axial(&self) -> HexCoord {
        HexCoord {
            q: self.x,
            r: self.z,
        }
    }
}

/// Round floating-point axial coordinates to nearest hex
fn hex_round(q: f64, r: f64) -> HexCoord {
    let s = -q - r;

    let mut rq = q.round();
    let mut rr = r.round();
    let rs = s.round();

    let q_diff = (rq - q).abs();
    let r_diff = (rr - r).abs();
    let s_diff = (rs - s).abs();

    if q_diff > r_diff && q_diff > s_diff {
        rq = -rr - rs;
    } else if r_diff > s_diff {
        rr = -rq - rs;
    }

    HexCoord::new(rq as i32, rr as i32)
}

/// Get the 6 corner points of a hex for rendering
pub fn hex_corners(center_x: f64, center_y: f64, size: f64) -> [(f64, f64); 6] {
    let mut corners = [(0.0, 0.0); 6];
    for i in 0..6 {
        let angle = PI / 3.0 * i as f64;
        corners[i] = (center_x + size * angle.cos(), center_y + size * angle.sin());
    }
    corners
}

/// Generate a rectangular map of hex coordinates
pub fn generate_rect_map(width: i32, height: i32) -> Vec<HexCoord> {
    let mut hexes = Vec::with_capacity((width * height) as usize);
    for r in 0..height {
        let r_offset = r / 2;
        for q in -r_offset..(width - r_offset) {
            hexes.push(HexCoord::new(q, r));
        }
    }
    hexes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_distance() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(2, -1);
        assert_eq!(a.distance_to(b), 2);
    }

    #[test]
    fn test_neighbors() {
        let center = HexCoord::new(0, 0);
        let neighbors = center.neighbors();
        assert_eq!(neighbors.len(), 6);
        for neighbor in &neighbors {
            assert_eq!(center.distance_to(*neighbor), 1);
        }
    }

    #[test]
    fn test_facing_opposite() {
        assert_eq!(Facing::East.opposite(), Facing::West);
        assert_eq!(Facing::Northeast.opposite(), Facing::Southwest);
    }

    #[test]
    fn test_cube_conversion() {
        let hex = HexCoord::new(3, -2);
        let cube = hex.to_cube();
        assert_eq!(cube.x + cube.y + cube.z, 0);
        assert_eq!(cube.to_axial(), hex);
    }

    #[test]
    fn test_line_to() {
        let start = HexCoord::new(0, 0);
        let end = HexCoord::new(3, 0);
        let line = start.line_to(end);
        assert_eq!(line.len(), 4);
        assert_eq!(line[0], start);
        assert_eq!(line[3], end);
    }

    #[test]
    fn test_pixel_conversion() {
        let hex = HexCoord::new(2, 1);
        let (px, py) = hex.to_pixel(60.0);
        let back = HexCoord::from_pixel(px, py, 60.0);
        assert_eq!(hex, back);
    }
}
