//! Game rules, state management, and unit definitions
//!
//! Contains the core game state, unit types, and command processing.

use crate::hex::{Facing, HexCoord};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Game phases in turn order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Phase {
    /// Initial deployment phase
    Deployment,
    /// Movement phase - units move
    Movement,
    /// Combat phase - units attack
    Combat,
    /// End of turn cleanup
    End,
}

impl Phase {
    /// Get the next phase in sequence
    pub fn next(&self) -> Phase {
        match self {
            Phase::Deployment => Phase::Movement,
            Phase::Movement => Phase::Combat,
            Phase::Combat => Phase::End,
            Phase::End => Phase::Movement, // New turn starts with movement
        }
    }

    /// Check if this is the movement phase
    pub fn is_movement(&self) -> bool {
        matches!(self, Phase::Movement)
    }

    /// Check if this is the combat phase
    pub fn is_combat(&self) -> bool {
        matches!(self, Phase::Combat)
    }
}

/// Player identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    /// Get the opposing player
    pub fn opponent(&self) -> Player {
        match self {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        }
    }
}

/// Unit type identifier (matches sprite atlas names)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    // Titans
    ReaverTitan,
    WarlordTitan,
    // Super-Heavy Tanks
    Shadowsword,
    Shadowsword2,
    Shadowsword3,
}

impl UnitType {
    /// Get the sprite atlas key for this unit type
    pub fn sprite_key(&self) -> &'static str {
        match self {
            UnitType::ReaverTitan => "Reaver_Titan",
            UnitType::WarlordTitan => "Warlord_Titan",
            UnitType::Shadowsword => "shadowsword",
            UnitType::Shadowsword2 => "shadowsword2",
            UnitType::Shadowsword3 => "shadowsword3",
        }
    }

    /// Get base movement points for this unit type
    pub fn base_movement(&self) -> u32 {
        match self {
            UnitType::ReaverTitan => 6,
            UnitType::WarlordTitan => 4,
            UnitType::Shadowsword | UnitType::Shadowsword2 | UnitType::Shadowsword3 => 5,
        }
    }

    /// Get base armor value
    pub fn base_armor(&self) -> u32 {
        match self {
            UnitType::ReaverTitan => 12,
            UnitType::WarlordTitan => 16,
            UnitType::Shadowsword | UnitType::Shadowsword2 | UnitType::Shadowsword3 => 8,
        }
    }

    /// Get base structure (health) value
    pub fn base_structure(&self) -> u32 {
        match self {
            UnitType::ReaverTitan => 10,
            UnitType::WarlordTitan => 14,
            UnitType::Shadowsword | UnitType::Shadowsword2 | UnitType::Shadowsword3 => 6,
        }
    }

    /// Get void shield count (0 for non-Titans)
    pub fn void_shields(&self) -> u32 {
        match self {
            UnitType::ReaverTitan => 2,
            UnitType::WarlordTitan => 4,
            UnitType::Shadowsword | UnitType::Shadowsword2 | UnitType::Shadowsword3 => 0,
        }
    }

    /// Check if this is a Titan
    pub fn is_titan(&self) -> bool {
        matches!(self, UnitType::ReaverTitan | UnitType::WarlordTitan)
    }

    /// Get the display name
    pub fn display_name(&self) -> &'static str {
        match self {
            UnitType::ReaverTitan => "Reaver Titan",
            UnitType::WarlordTitan => "Warlord Titan",
            UnitType::Shadowsword => "Shadowsword",
            UnitType::Shadowsword2 => "Shadowsword Mk II",
            UnitType::Shadowsword3 => "Shadowsword Mk III",
        }
    }
}

/// Terrain type for map hexes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TerrainType {
    #[default]
    Clear,
    Rough,
    Woods,
    Water,
    Ruins,
    Impassable,
}

impl TerrainType {
    /// Get the movement cost for this terrain (0 = impassable)
    pub fn movement_cost(&self) -> Option<u32> {
        match self {
            TerrainType::Clear => Some(1),
            TerrainType::Rough => Some(2),
            TerrainType::Woods => Some(2),
            TerrainType::Water => Some(3),
            TerrainType::Ruins => Some(2),
            TerrainType::Impassable => None,
        }
    }
}

/// A hex tile on the game map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: TerrainType,
    pub elevation: i32,
}

impl Default for Tile {
    fn default() -> Self {
        Tile {
            terrain: TerrainType::Clear,
            elevation: 0,
        }
    }
}

/// The game map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameMap {
    pub width: i32,
    pub height: i32,
    pub tiles: HashMap<(i32, i32), Tile>,
}

impl GameMap {
    /// Create a new empty map
    pub fn new(width: i32, height: i32) -> Self {
        let mut tiles = HashMap::new();
        for r in 0..height {
            let r_offset = r / 2;
            for q in -r_offset..(width - r_offset) {
                tiles.insert((q, r), Tile::default());
            }
        }
        GameMap {
            width,
            height,
            tiles,
        }
    }

    /// Get a tile at the given coordinate
    pub fn get_tile(&self, coord: HexCoord) -> Option<&Tile> {
        self.tiles.get(&(coord.q, coord.r))
    }

    /// Check if a coordinate is valid on this map
    pub fn is_valid(&self, coord: HexCoord) -> bool {
        self.tiles.contains_key(&(coord.q, coord.r))
    }

    /// Get all valid hex coordinates on this map
    pub fn all_hexes(&self) -> Vec<HexCoord> {
        self.tiles
            .keys()
            .map(|(q, r)| HexCoord::new(*q, *r))
            .collect()
    }

    /// Get the terrain at a coordinate
    pub fn terrain_at(&self, coord: HexCoord) -> TerrainType {
        self.get_tile(coord)
            .map(|t| t.terrain)
            .unwrap_or(TerrainType::Impassable)
    }
}

/// A unit on the battlefield
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Unit {
    pub id: u32,
    pub unit_type: UnitType,
    pub owner: Player,
    pub position: HexCoord,
    pub facing: Facing,

    // Health
    pub armor: u32,
    pub structure: u32,
    pub void_shields: u32,

    // Movement state
    pub movement_remaining: u32,
    pub has_moved: bool,
    pub has_attacked: bool,
}

impl Unit {
    /// Create a new unit
    pub fn new(id: u32, unit_type: UnitType, owner: Player, position: HexCoord, facing: Facing) -> Self {
        Unit {
            id,
            unit_type,
            owner,
            position,
            facing,
            armor: unit_type.base_armor(),
            structure: unit_type.base_structure(),
            void_shields: unit_type.void_shields(),
            movement_remaining: unit_type.base_movement(),
            has_moved: false,
            has_attacked: false,
        }
    }

    /// Check if the unit is destroyed
    pub fn is_destroyed(&self) -> bool {
        self.structure == 0
    }

    /// Get the sprite key for the current facing
    pub fn sprite_frame(&self) -> String {
        format!(
            "{}_{}_0000",
            self.unit_type.sprite_key(),
            self.facing.to_sprite_direction()
        )
    }

    /// Reset movement for a new turn
    pub fn reset_for_turn(&mut self) {
        self.movement_remaining = self.unit_type.base_movement();
        self.has_moved = false;
        self.has_attacked = false;
    }

    /// Get effective movement after damage
    pub fn effective_movement(&self) -> u32 {
        self.movement_remaining
    }
}

/// Player commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Move a unit along a path
    Move {
        unit_id: u32,
        path: Vec<HexCoord>,
        final_facing: Facing,
    },
    /// End the current phase
    EndPhase,
    /// End the current turn
    EndTurn,
}

/// Events generated by the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    /// Unit moved
    UnitMoved {
        unit_id: u32,
        from: HexCoord,
        to: HexCoord,
        facing: Facing,
    },
    /// Phase changed
    PhaseChanged {
        from: Phase,
        to: Phase,
    },
    /// Turn changed
    TurnChanged {
        turn: u32,
    },
    /// Unit destroyed
    UnitDestroyed {
        unit_id: u32,
    },
}

/// Complete game state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub map: GameMap,
    pub units: Vec<Unit>,
    pub current_turn: u32,
    pub current_phase: Phase,
    pub active_player: Player,
    pub selected_unit: Option<u32>,
    pub events: Vec<GameEvent>,
    pub game_over: bool,
    pub winner: Option<Player>,
}

impl GameState {
    /// Create a new game state with the given map
    pub fn new(map: GameMap) -> Self {
        GameState {
            map,
            units: Vec::new(),
            current_turn: 1,
            current_phase: Phase::Deployment,
            active_player: Player::Player1,
            selected_unit: None,
            events: Vec::new(),
            game_over: false,
            winner: None,
        }
    }

    /// Add a unit to the game
    pub fn add_unit(&mut self, unit: Unit) {
        self.units.push(unit);
    }

    /// Get a unit by ID
    pub fn get_unit(&self, id: u32) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id)
    }

    /// Get a mutable unit by ID
    pub fn get_unit_mut(&mut self, id: u32) -> Option<&mut Unit> {
        self.units.iter_mut().find(|u| u.id == id)
    }

    /// Get the unit at a position
    pub fn unit_at(&self, pos: HexCoord) -> Option<&Unit> {
        self.units.iter().find(|u| u.position == pos && !u.is_destroyed())
    }

    /// Get units owned by a player
    pub fn player_units(&self, player: Player) -> Vec<&Unit> {
        self.units
            .iter()
            .filter(|u| u.owner == player && !u.is_destroyed())
            .collect()
    }

    /// Process a command
    pub fn process_command(&mut self, command: Command) -> Result<Vec<GameEvent>, String> {
        let mut events = Vec::new();

        match command {
            Command::Move {
                unit_id,
                path,
                final_facing,
            } => {
                if self.current_phase != Phase::Movement {
                    return Err("Cannot move outside of movement phase".to_string());
                }

                let unit = self
                    .get_unit(unit_id)
                    .ok_or("Unit not found")?;

                if unit.owner != self.active_player {
                    return Err("Cannot move opponent's unit".to_string());
                }

                if unit.has_moved {
                    return Err("Unit has already moved this turn".to_string());
                }

                if path.is_empty() {
                    return Err("Path is empty".to_string());
                }

                let start = unit.position;
                let end = *path.last().unwrap();

                // Validate path (simplified - just check final position is valid)
                if !self.map.is_valid(end) {
                    return Err("Invalid destination".to_string());
                }

                if self.unit_at(end).is_some() && end != start {
                    return Err("Destination occupied".to_string());
                }

                // Apply movement
                let unit = self.get_unit_mut(unit_id).unwrap();
                unit.position = end;
                unit.facing = final_facing;
                unit.has_moved = true;
                unit.movement_remaining = 0;

                events.push(GameEvent::UnitMoved {
                    unit_id,
                    from: start,
                    to: end,
                    facing: final_facing,
                });
            }

            Command::EndPhase => {
                let old_phase = self.current_phase;
                self.current_phase = self.current_phase.next();

                if self.current_phase == Phase::End {
                    // End of turn, reset and go to next turn
                    self.end_turn();
                    events.push(GameEvent::TurnChanged {
                        turn: self.current_turn,
                    });
                }

                events.push(GameEvent::PhaseChanged {
                    from: old_phase,
                    to: self.current_phase,
                });
            }

            Command::EndTurn => {
                let old_phase = self.current_phase;
                self.end_turn();

                events.push(GameEvent::PhaseChanged {
                    from: old_phase,
                    to: Phase::Movement,
                });
                events.push(GameEvent::TurnChanged {
                    turn: self.current_turn,
                });
            }
        }

        self.events.extend(events.clone());
        Ok(events)
    }

    /// End the current turn
    fn end_turn(&mut self) {
        self.current_turn += 1;
        self.current_phase = Phase::Movement;
        self.active_player = self.active_player.opponent();

        // Reset all units
        for unit in &mut self.units {
            unit.reset_for_turn();
        }
    }

    /// Select a unit
    pub fn select_unit(&mut self, unit_id: Option<u32>) {
        self.selected_unit = unit_id;
    }

    /// Get the selected unit
    pub fn selected_unit(&self) -> Option<&Unit> {
        self.selected_unit.and_then(|id| self.get_unit(id))
    }

    /// Check if a player has won
    pub fn check_victory(&mut self) {
        let p1_alive = self.player_units(Player::Player1).len();
        let p2_alive = self.player_units(Player::Player2).len();

        if p1_alive == 0 && p2_alive > 0 {
            self.game_over = true;
            self.winner = Some(Player::Player2);
        } else if p2_alive == 0 && p1_alive > 0 {
            self.game_over = true;
            self.winner = Some(Player::Player1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_sequence() {
        assert_eq!(Phase::Deployment.next(), Phase::Movement);
        assert_eq!(Phase::Movement.next(), Phase::Combat);
        assert_eq!(Phase::Combat.next(), Phase::End);
        assert_eq!(Phase::End.next(), Phase::Movement);
    }

    #[test]
    fn test_unit_creation() {
        let unit = Unit::new(
            1,
            UnitType::ReaverTitan,
            Player::Player1,
            HexCoord::new(0, 0),
            Facing::East,
        );
        assert_eq!(unit.armor, 12);
        assert_eq!(unit.structure, 10);
        assert_eq!(unit.void_shields, 2);
        assert!(!unit.is_destroyed());
    }

    #[test]
    fn test_game_state() {
        let map = GameMap::new(10, 10);
        let mut state = GameState::new(map);

        let unit = Unit::new(
            1,
            UnitType::Shadowsword,
            Player::Player1,
            HexCoord::new(0, 0),
            Facing::East,
        );
        state.add_unit(unit);

        assert_eq!(state.units.len(), 1);
        assert!(state.get_unit(1).is_some());
        assert!(state.unit_at(HexCoord::new(0, 0)).is_some());
    }
}
