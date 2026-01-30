//! Movement system with pathfinding and reachable hex calculations
//!
//! Implements A* pathfinding and movement cost calculations for the hex grid.

use crate::hex::{Facing, HexCoord};
use crate::rules::{GameMap, GameState, TerrainType, Unit};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

/// Result of a movement calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementResult {
    /// Hexes reachable with remaining MP
    pub reachable: HashMap<HexCoord, u32>,
    /// Path to a specific hex if requested
    pub path: Option<Vec<HexCoord>>,
    /// Cost of the path
    pub path_cost: u32,
}

/// Node for A* pathfinding
#[derive(Debug, Clone, Eq, PartialEq)]
struct PathNode {
    coord: HexCoord,
    cost: u32,
    priority: u32,
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap behavior
        other.priority.cmp(&self.priority)
            .then_with(|| self.coord.q.cmp(&other.coord.q))
            .then_with(|| self.coord.r.cmp(&other.coord.r))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Calculate movement cost between two adjacent hexes
pub fn movement_cost(map: &GameMap, _from: HexCoord, to: HexCoord) -> Option<u32> {
    map.get_tile(to)
        .and_then(|tile| tile.terrain.movement_cost())
}

/// Check if a hex is blocked (by terrain or unit)
pub fn is_blocked(state: &GameState, coord: HexCoord, moving_unit_id: u32) -> bool {
    // Check terrain
    if !state.map.is_valid(coord) {
        return true;
    }

    let terrain = state.map.terrain_at(coord);
    if terrain == TerrainType::Impassable {
        return true;
    }

    // Check for enemy units (friendly units can be moved through but not stopped on)
    if let Some(unit) = state.unit_at(coord) {
        if unit.id != moving_unit_id {
            // Can't stop on a hex with another unit
            return true;
        }
    }

    false
}

/// Check if a hex can be passed through (for pathfinding)
pub fn can_pass_through(state: &GameState, coord: HexCoord, moving_unit: &Unit) -> bool {
    // Check terrain
    if !state.map.is_valid(coord) {
        return false;
    }

    let terrain = state.map.terrain_at(coord);
    if terrain == TerrainType::Impassable {
        return false;
    }

    // Check for units
    if let Some(unit) = state.unit_at(coord) {
        if unit.id == moving_unit.id {
            return true; // Can always be at own position
        }
        // Can pass through friendly units but not enemy units
        if unit.owner == moving_unit.owner {
            return true;
        }
        return false;
    }

    true
}

/// Find all reachable hexes from a starting position within movement budget
pub fn find_reachable(state: &GameState, unit: &Unit) -> HashMap<HexCoord, u32> {
    let mut reachable = HashMap::new();
    let mut visited = HashSet::new();
    let mut frontier: BinaryHeap<PathNode> = BinaryHeap::new();

    let start = unit.position;
    let budget = unit.effective_movement();

    frontier.push(PathNode {
        coord: start,
        cost: 0,
        priority: 0,
    });

    while let Some(current) = frontier.pop() {
        if visited.contains(&current.coord) {
            continue;
        }
        visited.insert(current.coord);

        // Record remaining MP at this hex
        let remaining = budget.saturating_sub(current.cost);
        reachable.insert(current.coord, remaining);

        // Explore neighbors
        for neighbor in current.coord.neighbors() {
            if visited.contains(&neighbor) {
                continue;
            }

            if !can_pass_through(state, neighbor, unit) {
                continue;
            }

            if let Some(cost) = movement_cost(&state.map, current.coord, neighbor) {
                let new_cost = current.cost + cost;
                if new_cost <= budget {
                    // Check if we can stop here (not just pass through)
                    let can_stop = !is_blocked(state, neighbor, unit.id);
                    if can_stop {
                        frontier.push(PathNode {
                            coord: neighbor,
                            cost: new_cost,
                            priority: new_cost,
                        });
                    } else {
                        // Can pass through but not stop - still add to frontier for pathfinding
                        // but don't add to reachable
                        frontier.push(PathNode {
                            coord: neighbor,
                            cost: new_cost,
                            priority: new_cost,
                        });
                    }
                }
            }
        }
    }

    // Remove hexes where we can't actually stop
    reachable.retain(|coord, _| !is_blocked(state, *coord, unit.id) || *coord == start);

    reachable
}

/// Find the shortest path between two hexes using A*
pub fn find_path(
    state: &GameState,
    unit: &Unit,
    target: HexCoord,
    max_cost: Option<u32>,
) -> Option<(Vec<HexCoord>, u32)> {
    let start = unit.position;
    let budget = max_cost.unwrap_or(unit.effective_movement());

    if start == target {
        return Some((vec![start], 0));
    }

    if is_blocked(state, target, unit.id) {
        return None;
    }

    let mut open_set: BinaryHeap<PathNode> = BinaryHeap::new();
    let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();
    let mut g_score: HashMap<HexCoord, u32> = HashMap::new();

    g_score.insert(start, 0);

    open_set.push(PathNode {
        coord: start,
        cost: 0,
        priority: start.distance_to(target),
    });

    while let Some(current) = open_set.pop() {
        if current.coord == target {
            // Reconstruct path
            let mut path = vec![target];
            let mut current_coord = target;
            while let Some(&prev) = came_from.get(&current_coord) {
                path.push(prev);
                current_coord = prev;
            }
            path.reverse();
            return Some((path, *g_score.get(&target).unwrap()));
        }

        let current_g = *g_score.get(&current.coord).unwrap_or(&u32::MAX);

        for neighbor in current.coord.neighbors() {
            if !can_pass_through(state, neighbor, unit) {
                continue;
            }

            if let Some(cost) = movement_cost(&state.map, current.coord, neighbor) {
                let tentative_g = current_g + cost;

                if tentative_g > budget {
                    continue;
                }

                if tentative_g < *g_score.get(&neighbor).unwrap_or(&u32::MAX) {
                    came_from.insert(neighbor, current.coord);
                    g_score.insert(neighbor, tentative_g);

                    let f_score = tentative_g + neighbor.distance_to(target);
                    open_set.push(PathNode {
                        coord: neighbor,
                        cost: tentative_g,
                        priority: f_score,
                    });
                }
            }
        }
    }

    None
}

/// Determine the best facing for a unit after moving to a destination
pub fn suggest_facing(from: HexCoord, to: HexCoord) -> Facing {
    from.direction_to(to).unwrap_or(Facing::East)
}

/// Get movement path with facing changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovementPath {
    pub path: Vec<HexCoord>,
    pub final_facing: Facing,
    pub total_cost: u32,
}

impl MovementPath {
    /// Create a new movement path
    pub fn new(path: Vec<HexCoord>, final_facing: Facing, cost: u32) -> Self {
        MovementPath {
            path,
            final_facing,
            total_cost: cost,
        }
    }

    /// Check if the path is valid
    pub fn is_valid(&self) -> bool {
        !self.path.is_empty()
    }

    /// Get the starting position
    pub fn start(&self) -> Option<HexCoord> {
        self.path.first().copied()
    }

    /// Get the ending position
    pub fn end(&self) -> Option<HexCoord> {
        self.path.last().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::{GameMap, GameState, Player, UnitType};

    fn setup_test_state() -> GameState {
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

        state
    }

    #[test]
    fn test_find_reachable() {
        let state = setup_test_state();
        let unit = state.get_unit(1).unwrap();
        let reachable = find_reachable(&state, unit);

        // Should include starting position
        assert!(reachable.contains_key(&HexCoord::new(0, 0)));

        // Should include neighbors (cost 1 each for clear terrain)
        for neighbor in HexCoord::new(0, 0).neighbors() {
            if state.map.is_valid(neighbor) {
                assert!(reachable.contains_key(&neighbor));
            }
        }
    }

    #[test]
    fn test_find_path() {
        let state = setup_test_state();
        let unit = state.get_unit(1).unwrap();

        // Path to adjacent hex
        let result = find_path(&state, unit, HexCoord::new(1, 0), None);
        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(cost, 1);

        // Path to farther hex
        let result = find_path(&state, unit, HexCoord::new(3, 0), None);
        assert!(result.is_some());
        let (path, cost) = result.unwrap();
        assert_eq!(path.len(), 4);
        assert_eq!(cost, 3);
    }

    #[test]
    fn test_suggest_facing() {
        let facing = suggest_facing(HexCoord::new(0, 0), HexCoord::new(1, 0));
        assert_eq!(facing, Facing::East);
    }
}
