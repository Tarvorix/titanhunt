// Hex coordinate utilities for flat-top hexagons

import { Hex, PixelPos } from '../types';

// Default hex size in pixels (100px gives ~173px hex width, good for 512px sprites)
export const HEX_SIZE = 100;

// Direction offsets for flat-top hexagons (matches Rust AXIAL_DIRECTIONS)
export const AXIAL_DIRECTIONS: [number, number][] = [
  [1, 0],   // East
  [1, -1],  // Northeast
  [0, -1],  // Northwest
  [-1, 0],  // West
  [-1, 1],  // Southwest
  [0, 1],   // Southeast
];

/**
 * Convert hex coordinates to pixel position (flat-top orientation)
 */
export function hexToPixel(hex: Hex, size: number = HEX_SIZE): PixelPos {
  const x = size * (Math.sqrt(3) * hex.q + (Math.sqrt(3) / 2) * hex.r);
  const y = size * ((3 / 2) * hex.r);
  return { x, y };
}

/**
 * Convert pixel position to hex coordinates (flat-top orientation)
 */
export function pixelToHex(x: number, y: number, size: number = HEX_SIZE): Hex {
  const q = ((Math.sqrt(3) / 3) * x - (1 / 3) * y) / size;
  const r = ((2 / 3) * y) / size;
  return hexRound(q, r);
}

/**
 * Round floating-point hex coordinates to nearest integer hex
 */
export function hexRound(q: number, r: number): Hex {
  const s = -q - r;

  let rq = Math.round(q);
  let rr = Math.round(r);
  const rs = Math.round(s);

  const qDiff = Math.abs(rq - q);
  const rDiff = Math.abs(rr - r);
  const sDiff = Math.abs(rs - s);

  if (qDiff > rDiff && qDiff > sDiff) {
    rq = -rr - rs;
  } else if (rDiff > sDiff) {
    rr = -rq - rs;
  }

  return { q: rq, r: rr };
}

/**
 * Calculate distance between two hexes
 */
export function hexDistance(a: Hex, b: Hex): number {
  // Convert to cube coordinates
  const ax = a.q;
  const az = a.r;
  const ay = -ax - az;

  const bx = b.q;
  const bz = b.r;
  const by = -bx - bz;

  return (Math.abs(ax - bx) + Math.abs(ay - by) + Math.abs(az - bz)) / 2;
}

/**
 * Get all 6 neighboring hexes
 */
export function hexNeighbors(hex: Hex): Hex[] {
  return AXIAL_DIRECTIONS.map(([dq, dr]) => ({
    q: hex.q + dq,
    r: hex.r + dr,
  }));
}

/**
 * Get neighbor in a specific direction (0-5)
 */
export function hexNeighbor(hex: Hex, direction: number): Hex {
  const [dq, dr] = AXIAL_DIRECTIONS[direction % 6];
  return { q: hex.q + dq, r: hex.r + dr };
}

/**
 * Create a unique key for a hex coordinate
 */
export function hexKey(hex: Hex): string {
  return `${hex.q},${hex.r}`;
}

/**
 * Parse a hex key back to coordinates
 */
export function parseHexKey(key: string): Hex {
  const [q, r] = key.split(',').map(Number);
  return { q, r };
}

/**
 * Check if two hexes are equal
 */
export function hexEquals(a: Hex, b: Hex): boolean {
  return a.q === b.q && a.r === b.r;
}

/**
 * Get the 6 corner points of a hex for rendering (flat-top orientation)
 * Corners start at 30° and go every 60°: 30°, 90°, 150°, 210°, 270°, 330°
 */
export function hexCorners(centerX: number, centerY: number, size: number): PixelPos[] {
  const corners: PixelPos[] = [];
  for (let i = 0; i < 6; i++) {
    // Flat-top: start at 30 degrees (PI/6) offset
    const angle = (Math.PI / 6) + (Math.PI / 3) * i;
    corners.push({
      x: centerX + size * Math.cos(angle),
      y: centerY + size * Math.sin(angle),
    });
  }
  return corners;
}

/**
 * Draw a hex shape on a Phaser Graphics object
 */
export function drawHex(
  graphics: Phaser.GameObjects.Graphics,
  centerX: number,
  centerY: number,
  size: number,
  fill?: number,
  fillAlpha?: number,
  stroke?: number,
  strokeAlpha?: number,
  lineWidth?: number
): void {
  const corners = hexCorners(centerX, centerY, size);
  const points = corners.map((c) => new Phaser.Math.Vector2(c.x, c.y));

  if (fill !== undefined) {
    graphics.fillStyle(fill, fillAlpha ?? 1);
    graphics.fillPoints(points, true);
  }

  if (stroke !== undefined) {
    graphics.lineStyle(lineWidth ?? 2, stroke, strokeAlpha ?? 1);
    graphics.strokePoints(points, true);
  }
}

/**
 * Generate a rectangular grid of hex coordinates
 */
export function generateRectMap(width: number, height: number): Hex[] {
  const hexes: Hex[] = [];
  for (let r = 0; r < height; r++) {
    const rOffset = Math.floor(r / 2);
    for (let q = -rOffset; q < width - rOffset; q++) {
      hexes.push({ q, r });
    }
  }
  return hexes;
}

/**
 * Get hexes along a line between two hexes
 */
export function hexLine(start: Hex, end: Hex): Hex[] {
  const n = hexDistance(start, end);
  if (n === 0) {
    return [start];
  }

  const results: Hex[] = [];
  for (let i = 0; i <= n; i++) {
    const t = i / n;
    const q = start.q + (end.q - start.q) * t;
    const r = start.r + (end.r - start.r) * t;
    results.push(hexRound(q, r));
  }
  return results;
}

/**
 * Calculate direction index (0-5) from one hex to another
 */
export function hexDirectionTo(from: Hex, to: Hex): number | null {
  if (hexEquals(from, to)) {
    return null;
  }

  const dx = to.q - from.q;
  const dy = to.r - from.r;
  const angle = Math.atan2(dy, dx);

  // Normalize angle and map to direction index
  const normalized = ((angle + 2 * Math.PI) % (2 * Math.PI));
  const index = Math.round((normalized / (Math.PI / 3))) % 6;
  return index;
}
