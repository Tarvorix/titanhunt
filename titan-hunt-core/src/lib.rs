//! Titan Hunt Core - Game logic for a 40K-inspired hex-based tactical game
//!
//! This crate contains pure Rust game logic that can be compiled to WASM
//! for use in a web-based frontend.

pub mod hex;
pub mod movement;
pub mod rules;
mod wasm_api;

// Re-export commonly used types
pub use hex::{CubeCoord, Facing, HexCoord, AXIAL_DIRECTIONS};
pub use movement::{find_path, find_reachable, MovementResult};
pub use rules::{Command, GameState, Phase, Player, Unit, UnitType};
