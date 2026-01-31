// Type definitions for Titan Hunt

export interface Hex {
  q: number;
  r: number;
}

export interface PixelPos {
  x: number;
  y: number;
}

export interface ReachableHex extends Hex {
  remaining: number;
}

export interface PathResult {
  path: Hex[];
  cost: number;
  valid: boolean;
}

export interface Unit {
  id: number;
  unit_type: string;
  display_name: string;
  owner: number;
  q: number;
  r: number;
  facing: number;
  sprite_frame: string;
  armor: number;
  max_armor: number;
  structure: number;
  max_structure: number;
  void_shields: number;
  max_void_shields: number;
  movement_remaining: number;
  max_movement: number;
  has_moved: boolean;
  has_attacked: boolean;
  is_destroyed: boolean;
  is_titan: boolean;
}

export interface MapSize {
  width: number;
  height: number;
}

export interface GameEvent {
  type: string;
  unit_id?: number;
  from?: Hex;
  to?: Hex;
  facing?: number;
  turn?: number;
}

// Facing directions (matches Rust enum order)
export enum Facing {
  East = 0,
  Northeast = 1,
  Northwest = 2,
  West = 3,
  Southwest = 4,
  Southeast = 5,
}

// Sprite directions (8 directions from atlases)
export const SPRITE_DIRECTIONS = ['S', 'SW', 'W', 'NW', 'N', 'NE', 'E', 'SE'] as const;
export type SpriteDirection = typeof SPRITE_DIRECTIONS[number];

// Map facing to sprite direction
export function facingToSpriteDirection(facing: Facing): SpriteDirection {
  const mapping: Record<Facing, SpriteDirection> = {
    [Facing.East]: 'E',
    [Facing.Northeast]: 'NE',
    [Facing.Northwest]: 'NW',
    [Facing.West]: 'W',
    [Facing.Southwest]: 'SW',
    [Facing.Southeast]: 'SE',
  };
  return mapping[facing];
}

// Direction angles in radians for facing markers
export function facingToRadians(facing: Facing): number {
  const angles: Record<Facing, number> = {
    [Facing.East]: 0,
    [Facing.Northeast]: Math.PI / 3,
    [Facing.Northwest]: (2 * Math.PI) / 3,
    [Facing.West]: Math.PI,
    [Facing.Southwest]: (4 * Math.PI) / 3,
    [Facing.Southeast]: (5 * Math.PI) / 3,
  };
  return angles[facing];
}

// Atlas manifest structure
export interface AtlasSheet {
  name: string;
  base_animation: string;
  angles: number;
  frames: number;
  frame_range: [number, number];
  frame_size: { w: number; h: number };
  atlas_size: { w: number; h: number };
}

export interface AtlasManifest {
  sheets: AtlasSheet[];
  directions: string[];
  direction_angles: Record<string, number>;
  max_texture_width: number;
}

// Terrain types (matches StrikeMek/MegaMek)
export type TerrainType = 'CLEAR' | 'WOODS' | 'ROUGH' | 'WATER' | 'PAVEMENT' | 'BUILDING' | 'RUBBLE';

// Map tile from JSON
export interface MapTile {
  q: number;
  r: number;
  terrain: TerrainType;
  elevation: number;
  tileId: string;
  layers: string[];
  waterDepth: number;
  tags: string[];
}

// Map data from JSON
export interface MapData {
  id: string;
  name: string;
  layout: string;
  width: number;
  height: number;
  tiles: MapTile[];
}
