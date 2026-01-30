//! WASM API bindings for the TypeScript/JavaScript frontend
//!
//! Exposes game functions to the browser via wasm-bindgen.

use crate::hex::{Facing, HexCoord};
use crate::movement::{find_path, find_reachable};
use crate::rules::{Command, GameMap, GameState, Phase, Player, Unit, UnitType};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Game engine wrapper for WASM
#[wasm_bindgen]
pub struct TitanHuntEngine {
    state: GameState,
}

#[wasm_bindgen]
impl TitanHuntEngine {
    /// Create a new game with the specified map dimensions
    #[wasm_bindgen(constructor)]
    pub fn new(width: i32, height: i32) -> TitanHuntEngine {
        let map = GameMap::new(width, height);
        TitanHuntEngine {
            state: GameState::new(map),
        }
    }

    /// Get the current game state as JSON
    #[wasm_bindgen(js_name = getState)]
    pub fn get_state(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.state)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Add a unit to the game
    #[wasm_bindgen(js_name = addUnit)]
    pub fn add_unit(
        &mut self,
        id: u32,
        unit_type: &str,
        player: u32,
        q: i32,
        r: i32,
        facing: u8,
    ) -> Result<(), JsValue> {
        let unit_type = match unit_type {
            "Reaver_Titan" => UnitType::ReaverTitan,
            "Warlord_Titan" => UnitType::WarlordTitan,
            "shadowsword" => UnitType::Shadowsword,
            "shadowsword2" => UnitType::Shadowsword2,
            "shadowsword3" => UnitType::Shadowsword3,
            "krieg" => UnitType::KriegSquad,
            _ => return Err(JsValue::from_str(&format!("Unknown unit type: {}", unit_type))),
        };

        let owner = match player {
            1 => Player::Player1,
            2 => Player::Player2,
            _ => return Err(JsValue::from_str("Invalid player (must be 1 or 2)")),
        };

        let facing = Facing::from_index(facing)
            .ok_or_else(|| JsValue::from_str("Invalid facing (must be 0-5)"))?;

        let unit = Unit::new(id, unit_type, owner, HexCoord::new(q, r), facing);
        self.state.add_unit(unit);
        Ok(())
    }

    /// Get reachable hexes for a unit
    #[wasm_bindgen(js_name = getReachableHexes)]
    pub fn get_reachable_hexes(&self, unit_id: u32) -> Result<JsValue, JsValue> {
        let unit = self
            .state
            .get_unit(unit_id)
            .ok_or_else(|| JsValue::from_str("Unit not found"))?;

        let reachable = find_reachable(&self.state, unit);

        // Convert to array of {q, r, remaining} objects
        let result: Vec<ReachableHex> = reachable
            .into_iter()
            .map(|(coord, remaining)| ReachableHex {
                q: coord.q,
                r: coord.r,
                remaining,
            })
            .collect();

        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Find path from a unit to a target hex
    #[wasm_bindgen(js_name = findPath)]
    pub fn find_path_to(&self, unit_id: u32, target_q: i32, target_r: i32) -> Result<JsValue, JsValue> {
        let unit = self
            .state
            .get_unit(unit_id)
            .ok_or_else(|| JsValue::from_str("Unit not found"))?;

        let target = HexCoord::new(target_q, target_r);

        match find_path(&self.state, unit, target, None) {
            Some((path, cost)) => {
                let path_result: Vec<HexJson> = path
                    .into_iter()
                    .map(|coord| HexJson { q: coord.q, r: coord.r })
                    .collect();

                let result = PathResult {
                    path: path_result,
                    cost,
                    valid: true,
                };

                serde_wasm_bindgen::to_value(&result)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
            None => {
                let result = PathResult {
                    path: vec![],
                    cost: 0,
                    valid: false,
                };

                serde_wasm_bindgen::to_value(&result)
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }
    }

    /// Execute a move command
    #[wasm_bindgen(js_name = moveUnit)]
    pub fn move_unit(
        &mut self,
        unit_id: u32,
        path_json: JsValue,
        final_facing: u8,
    ) -> Result<JsValue, JsValue> {
        let path_data: Vec<HexJson> = serde_wasm_bindgen::from_value(path_json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        let path: Vec<HexCoord> = path_data
            .into_iter()
            .map(|h| HexCoord::new(h.q, h.r))
            .collect();

        let facing = Facing::from_index(final_facing)
            .ok_or_else(|| JsValue::from_str("Invalid facing"))?;

        let command = Command::Move {
            unit_id,
            path,
            final_facing: facing,
        };

        match self.state.process_command(command) {
            Ok(events) => serde_wasm_bindgen::to_value(&events)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// End the current phase
    #[wasm_bindgen(js_name = endPhase)]
    pub fn end_phase(&mut self) -> Result<JsValue, JsValue> {
        match self.state.process_command(Command::EndPhase) {
            Ok(events) => serde_wasm_bindgen::to_value(&events)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// End the current turn
    #[wasm_bindgen(js_name = endTurn)]
    pub fn end_turn(&mut self) -> Result<JsValue, JsValue> {
        match self.state.process_command(Command::EndTurn) {
            Ok(events) => serde_wasm_bindgen::to_value(&events)
                .map_err(|e| JsValue::from_str(&e.to_string())),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    /// Select a unit
    #[wasm_bindgen(js_name = selectUnit)]
    pub fn select_unit(&mut self, unit_id: Option<u32>) {
        self.state.select_unit(unit_id);
    }

    /// Get the selected unit ID
    #[wasm_bindgen(js_name = getSelectedUnit)]
    pub fn get_selected_unit(&self) -> Option<u32> {
        self.state.selected_unit
    }

    /// Get the current phase
    #[wasm_bindgen(js_name = getCurrentPhase)]
    pub fn get_current_phase(&self) -> String {
        match self.state.current_phase {
            Phase::Deployment => "deployment".to_string(),
            Phase::Movement => "movement".to_string(),
            Phase::Combat => "combat".to_string(),
            Phase::End => "end".to_string(),
        }
    }

    /// Get the active player (1 or 2)
    #[wasm_bindgen(js_name = getActivePlayer)]
    pub fn get_active_player(&self) -> u32 {
        match self.state.active_player {
            Player::Player1 => 1,
            Player::Player2 => 2,
        }
    }

    /// Get the current turn number
    #[wasm_bindgen(js_name = getCurrentTurn)]
    pub fn get_current_turn(&self) -> u32 {
        self.state.current_turn
    }

    /// Get all units as JSON
    #[wasm_bindgen(js_name = getUnits)]
    pub fn get_units(&self) -> Result<JsValue, JsValue> {
        let units: Vec<UnitJson> = self
            .state
            .units
            .iter()
            .map(|u| UnitJson {
                id: u.id,
                unit_type: u.unit_type.sprite_key().to_string(),
                display_name: u.unit_type.display_name().to_string(),
                owner: match u.owner {
                    Player::Player1 => 1,
                    Player::Player2 => 2,
                },
                q: u.position.q,
                r: u.position.r,
                facing: u.facing.index(),
                sprite_frame: u.sprite_frame(),
                armor: u.armor,
                max_armor: u.unit_type.base_armor(),
                structure: u.structure,
                max_structure: u.unit_type.base_structure(),
                void_shields: u.void_shields,
                max_void_shields: u.unit_type.void_shields(),
                movement_remaining: u.movement_remaining,
                max_movement: u.unit_type.base_movement(),
                has_moved: u.has_moved,
                has_attacked: u.has_attacked,
                is_destroyed: u.is_destroyed(),
                is_titan: u.unit_type.is_titan(),
            })
            .collect();

        serde_wasm_bindgen::to_value(&units)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get all valid hex coordinates on the map
    #[wasm_bindgen(js_name = getMapHexes)]
    pub fn get_map_hexes(&self) -> Result<JsValue, JsValue> {
        let hexes: Vec<HexJson> = self
            .state
            .map
            .all_hexes()
            .into_iter()
            .map(|coord| HexJson { q: coord.q, r: coord.r })
            .collect();

        serde_wasm_bindgen::to_value(&hexes)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Get map dimensions
    #[wasm_bindgen(js_name = getMapSize)]
    pub fn get_map_size(&self) -> Result<JsValue, JsValue> {
        let size = MapSize {
            width: self.state.map.width,
            height: self.state.map.height,
        };

        serde_wasm_bindgen::to_value(&size)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Start the game (transition from deployment to movement)
    #[wasm_bindgen(js_name = startGame)]
    pub fn start_game(&mut self) {
        if self.state.current_phase == Phase::Deployment {
            self.state.current_phase = Phase::Movement;
        }
    }

    /// Convert pixel coordinates to hex
    #[wasm_bindgen(js_name = pixelToHex)]
    pub fn pixel_to_hex(&self, x: f64, y: f64, hex_size: f64) -> Result<JsValue, JsValue> {
        let coord = HexCoord::from_pixel(x, y, hex_size);
        let hex = HexJson { q: coord.q, r: coord.r };
        serde_wasm_bindgen::to_value(&hex)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Convert hex to pixel coordinates
    #[wasm_bindgen(js_name = hexToPixel)]
    pub fn hex_to_pixel(&self, q: i32, r: i32, hex_size: f64) -> Result<JsValue, JsValue> {
        let (x, y) = HexCoord::new(q, r).to_pixel(hex_size);
        let pixel = PixelPos { x, y };
        serde_wasm_bindgen::to_value(&pixel)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

// JSON serialization helpers

#[derive(Serialize, Deserialize)]
struct HexJson {
    q: i32,
    r: i32,
}

#[derive(Serialize, Deserialize)]
struct PixelPos {
    x: f64,
    y: f64,
}

#[derive(Serialize, Deserialize)]
struct ReachableHex {
    q: i32,
    r: i32,
    remaining: u32,
}

#[derive(Serialize, Deserialize)]
struct PathResult {
    path: Vec<HexJson>,
    cost: u32,
    valid: bool,
}

#[derive(Serialize, Deserialize)]
struct UnitJson {
    id: u32,
    unit_type: String,
    display_name: String,
    owner: u32,
    q: i32,
    r: i32,
    facing: u8,
    sprite_frame: String,
    armor: u32,
    max_armor: u32,
    structure: u32,
    max_structure: u32,
    void_shields: u32,
    max_void_shields: u32,
    movement_remaining: u32,
    max_movement: u32,
    has_moved: bool,
    has_attacked: bool,
    is_destroyed: bool,
    is_titan: bool,
}

#[derive(Serialize, Deserialize)]
struct MapSize {
    width: i32,
    height: i32,
}
