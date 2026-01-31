// Map loader for terrain tiles and map data

import { MapData, MapTile } from '../types';

export class MapLoader {
  private scene: Phaser.Scene;
  private loadedImages: Set<string> = new Set();

  constructor(scene: Phaser.Scene) {
    this.scene = scene;
  }

  // Load a map JSON file
  async loadMap(mapId: string): Promise<MapData> {
    const response = await fetch(`terrain/maps/${mapId}.json`);
    if (!response.ok) {
      throw new Error(`Failed to load map: ${mapId}`);
    }
    return await response.json();
  }

  // Get all unique tile images from a map
  getRequiredImages(mapData: MapData): string[] {
    const images = new Set<string>();

    for (const tile of mapData.tiles) {
      if (tile.tileId) {
        images.add(tile.tileId);
      }
      for (const layer of tile.layers) {
        images.add(layer);
      }
    }

    return Array.from(images);
  }

  // Preload all images needed for a map
  preloadMapImages(mapData: MapData): void {
    const images = this.getRequiredImages(mapData);

    for (const imagePath of images) {
      if (!this.loadedImages.has(imagePath)) {
        const key = this.imagePathToKey(imagePath);
        this.scene.load.image(key, `terrain/hexes/${imagePath}`);
        this.loadedImages.add(imagePath);
      }
    }
  }

  // Convert image path to Phaser texture key
  imagePathToKey(imagePath: string): string {
    // Replace slashes and dots with underscores for a valid key
    return 'hex_' + imagePath.replace(/[\/\.]/g, '_');
  }

  // Check if an image is loaded
  isImageLoaded(imagePath: string): boolean {
    return this.loadedImages.has(imagePath);
  }

  // Get list of available maps
  async getMapList(): Promise<string[]> {
    // For now, return a static list - could be enhanced to load dynamically
    return [
      'grasslands_foothills_16x17',
      'grasslands_lakes_16x17',
      'grasslands_rolling_hills_16x17',
      'desert_hills_16x17',
      'desert_open_16x17',
      'badlands_1_16x17',
      'city_ruins_16x17',
    ];
  }
}

// Tile rendering helper
export function getTileMovementCost(tile: MapTile): number {
  switch (tile.terrain) {
    case 'CLEAR':
    case 'PAVEMENT':
      return 1;
    case 'ROUGH':
    case 'RUBBLE':
      return 2;
    case 'WOODS':
      return 2;
    case 'WATER':
      return tile.waterDepth > 1 ? 3 : 2;
    case 'BUILDING':
      return 999; // Impassable
    default:
      return 1;
  }
}

// Check if terrain blocks movement
export function isTileImpassable(tile: MapTile): boolean {
  return tile.terrain === 'BUILDING' || tile.waterDepth > 2;
}
