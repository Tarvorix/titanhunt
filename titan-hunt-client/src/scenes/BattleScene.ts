// Battle scene - main game view with hex grid and units

import Phaser from 'phaser';
import { Hex, Unit, AtlasManifest, Facing, facingToSpriteDirection } from '../types';
import { hexToPixel, pixelToHex, drawHex, hexKey, hexEquals, HEX_SIZE, hexNeighbors } from '../hex/HexUtils';

// Graphics layer depths
const DEPTH_GRID = 1;
const DEPTH_REACHABLE = 3;
const DEPTH_PATH = 4;
const DEPTH_UNITS = 5;
const DEPTH_SELECTION = 6;
const DEPTH_UI = 10;

// Colors
const COLOR_GRID_VALID = 0x666666;
const COLOR_SELECTED = 0x00ff00;
const COLOR_REACHABLE = 0x00ff00;
const COLOR_PATH = 0xffff00;

interface BattleSceneData {
  manifest: AtlasManifest;
}

export class BattleScene extends Phaser.Scene {
  // Game state
  private mapWidth = 15;
  private mapHeight = 10;
  private validHexes: Set<string> = new Set();
  private units: Unit[] = [];
  private selectedUnitId: number | null = null;
  private reachableHexes: Map<string, number> = new Map();
  private currentPath: Hex[] = [];
  private hoveredHex: Hex | null = null;

  // Graphics layers
  private gridGraphics!: Phaser.GameObjects.Graphics;
  private reachableGraphics!: Phaser.GameObjects.Graphics;
  private pathGraphics!: Phaser.GameObjects.Graphics;
  private selectionGraphics!: Phaser.GameObjects.Graphics;

  // Sprite container
  private unitSprites: Map<number, Phaser.GameObjects.Sprite> = new Map();

  // Camera controls
  private isDragging = false;
  private dragStartX = 0;
  private dragStartY = 0;
  private cameraStartX = 0;
  private cameraStartY = 0;

  // UI state
  private currentPhase: string = 'movement';
  private currentTurn = 1;
  private activePlayer = 1;

  // UI text elements
  private turnText: Phaser.GameObjects.Text | null = null;
  private phaseText: Phaser.GameObjects.Text | null = null;
  private playerText: Phaser.GameObjects.Text | null = null;

  // Origin offset for centering the map
  private hexOrigin = { x: 100, y: 100 };

  constructor() {
    super({ key: 'BattleScene' });
  }

  init(data: BattleSceneData): void {
    console.log('BattleScene init with manifest:', data.manifest);
  }

  create(): void {
    // Create graphics layers
    this.gridGraphics = this.add.graphics();
    this.gridGraphics.setDepth(DEPTH_GRID);

    this.reachableGraphics = this.add.graphics();
    this.reachableGraphics.setDepth(DEPTH_REACHABLE);

    this.pathGraphics = this.add.graphics();
    this.pathGraphics.setDepth(DEPTH_PATH);

    this.selectionGraphics = this.add.graphics();
    this.selectionGraphics.setDepth(DEPTH_SELECTION);

    // Generate map hexes
    this.generateMap();

    // Draw the hex grid
    this.drawGrid();

    // Setup camera
    this.setupCamera();

    // Setup input handlers
    this.setupInput();

    // Add test units
    this.addTestUnits();

    // Create UI
    this.createUI();

    // Initial render
    this.renderUnits();
  }

  private generateMap(): void {
    // Generate a rectangular hex map
    for (let r = 0; r < this.mapHeight; r++) {
      const rOffset = Math.floor(r / 2);
      for (let q = -rOffset; q < this.mapWidth - rOffset; q++) {
        this.validHexes.add(hexKey({ q, r }));
      }
    }
    console.log(`Generated map with ${this.validHexes.size} hexes`);
  }

  private drawGrid(): void {
    this.gridGraphics.clear();

    for (const key of this.validHexes) {
      const [q, r] = key.split(',').map(Number);
      const hex = { q, r };
      const pos = this.hexToWorld(hex);

      // Draw hex outline
      drawHex(
        this.gridGraphics,
        pos.x,
        pos.y,
        HEX_SIZE,
        0x1a1a2e, // Fill color (dark blue-gray)
        0.8,
        COLOR_GRID_VALID,
        0.5,
        1
      );
    }
  }

  private setupCamera(): void {
    const camera = this.cameras.main;

    // Calculate map bounds
    const minX = -100;
    const minY = -100;
    const maxX = this.mapWidth * HEX_SIZE * Math.sqrt(3) + 200;
    const maxY = this.mapHeight * HEX_SIZE * 1.5 + 200;

    camera.setBounds(minX, minY, maxX - minX, maxY - minY);
    camera.setZoom(1);

    // Center camera on map
    const centerX = (minX + maxX) / 2;
    const centerY = (minY + maxY) / 2;
    camera.centerOn(centerX, centerY);
  }

  private setupInput(): void {
    // Pointer down - start drag or select
    this.input.on('pointerdown', (pointer: Phaser.Input.Pointer) => {
      this.isDragging = false;
      this.dragStartX = pointer.x;
      this.dragStartY = pointer.y;
      this.cameraStartX = this.cameras.main.scrollX;
      this.cameraStartY = this.cameras.main.scrollY;
    });

    // Pointer move - drag camera or preview path
    this.input.on('pointermove', (pointer: Phaser.Input.Pointer) => {
      if (pointer.isDown) {
        const dx = pointer.x - this.dragStartX;
        const dy = pointer.y - this.dragStartY;

        // Only start dragging if moved more than threshold
        if (Math.abs(dx) > 8 || Math.abs(dy) > 8) {
          this.isDragging = true;
          this.cameras.main.scrollX = this.cameraStartX - dx;
          this.cameras.main.scrollY = this.cameraStartY - dy;
        }
      } else {
        // Update hovered hex for path preview
        const worldPos = this.cameras.main.getWorldPoint(pointer.x, pointer.y);
        const hex = this.worldToHex(worldPos.x, worldPos.y);

        if (!this.hoveredHex || !hexEquals(hex, this.hoveredHex)) {
          this.hoveredHex = hex;
          this.updatePathPreview();
        }
      }
    });

    // Pointer up - select hex or unit
    this.input.on('pointerup', (pointer: Phaser.Input.Pointer) => {
      if (!this.isDragging) {
        const worldPos = this.cameras.main.getWorldPoint(pointer.x, pointer.y);
        const hex = this.worldToHex(worldPos.x, worldPos.y);
        this.handleHexClick(hex);
      }
      this.isDragging = false;
    });

    // Mouse wheel - zoom
    this.input.on('wheel', (_pointer: Phaser.Input.Pointer, _gameObjects: unknown[], _dx: number, dy: number) => {
      const camera = this.cameras.main;
      const newZoom = Phaser.Math.Clamp(camera.zoom - dy * 0.001, 0.5, 2);
      camera.setZoom(newZoom);
    });

    // Keyboard shortcuts
    this.input.keyboard?.on('keydown-SPACE', () => {
      this.endTurn();
    });

    this.input.keyboard?.on('keydown-ESC', () => {
      this.deselectUnit();
    });

    // Facing rotation with Q/E
    this.input.keyboard?.on('keydown-Q', () => {
      this.rotateSelectedUnit(-1);
    });

    this.input.keyboard?.on('keydown-E', () => {
      this.rotateSelectedUnit(1);
    });
  }

  private hexToWorld(hex: Hex): { x: number; y: number } {
    const pos = hexToPixel(hex, HEX_SIZE);
    return {
      x: pos.x + this.hexOrigin.x,
      y: pos.y + this.hexOrigin.y,
    };
  }

  private worldToHex(x: number, y: number): Hex {
    return pixelToHex(
      x - this.hexOrigin.x,
      y - this.hexOrigin.y,
      HEX_SIZE
    );
  }

  private handleHexClick(hex: Hex): void {
    const key = hexKey(hex);

    // Check if clicking on valid hex
    if (!this.validHexes.has(key)) {
      return;
    }

    // Check if clicking on a unit
    const clickedUnit = this.units.find(
      (u) => u.q === hex.q && u.r === hex.r && !u.is_destroyed
    );

    if (clickedUnit) {
      // Select/deselect unit
      if (this.selectedUnitId === clickedUnit.id) {
        this.deselectUnit();
      } else {
        this.selectUnit(clickedUnit.id);
      }
    } else if (this.selectedUnitId !== null && this.reachableHexes.has(key)) {
      // Move selected unit to hex
      this.moveSelectedUnit(hex);
    } else {
      // Deselect if clicking empty hex
      this.deselectUnit();
    }
  }

  private selectUnit(unitId: number): void {
    this.selectedUnitId = unitId;
    const unit = this.units.find((u) => u.id === unitId);

    if (unit && unit.owner === this.activePlayer && !unit.has_moved) {
      // Calculate reachable hexes
      this.calculateReachableHexes(unit);
    } else {
      this.reachableHexes.clear();
    }

    this.renderSelection();
    this.renderReachable();
  }

  private deselectUnit(): void {
    this.selectedUnitId = null;
    this.reachableHexes.clear();
    this.currentPath = [];
    this.renderSelection();
    this.renderReachable();
    this.renderPath();
  }

  private calculateReachableHexes(unit: Unit): void {
    // Simple BFS for reachable hexes
    this.reachableHexes.clear();

    const start = { q: unit.q, r: unit.r };
    const budget = unit.movement_remaining;

    const queue: { hex: Hex; cost: number }[] = [{ hex: start, cost: 0 }];
    const visited = new Map<string, number>();
    visited.set(hexKey(start), 0);

    while (queue.length > 0) {
      const current = queue.shift()!;
      const remaining = budget - current.cost;

      // Can stop here if not occupied by another unit
      const occupyingUnit = this.units.find(
        (u) => u.q === current.hex.q && u.r === current.hex.r && u.id !== unit.id && !u.is_destroyed
      );
      if (!occupyingUnit || hexEquals(current.hex, start)) {
        this.reachableHexes.set(hexKey(current.hex), remaining);
      }

      // Explore neighbors
      for (const neighbor of hexNeighbors(current.hex)) {
        const neighborKey = hexKey(neighbor);

        // Check if valid hex
        if (!this.validHexes.has(neighborKey)) {
          continue;
        }

        // Movement cost (1 for clear terrain)
        const moveCost = 1;
        const newCost = current.cost + moveCost;

        if (newCost > budget) {
          continue;
        }

        // Check if we found a better path
        const existingCost = visited.get(neighborKey);
        if (existingCost === undefined || newCost < existingCost) {
          visited.set(neighborKey, newCost);

          // Check if blocked by enemy unit
          const blockingUnit = this.units.find(
            (u) => u.q === neighbor.q && u.r === neighbor.r && u.owner !== unit.owner && !u.is_destroyed
          );
          if (!blockingUnit) {
            queue.push({ hex: neighbor, cost: newCost });
          }
        }
      }
    }
  }

  private findPath(from: Hex, to: Hex): Hex[] {
    // A* pathfinding
    const startKey = hexKey(from);
    const endKey = hexKey(to);

    if (startKey === endKey) {
      return [from];
    }

    const openSet: { hex: Hex; cost: number; priority: number }[] = [
      { hex: from, cost: 0, priority: 0 },
    ];
    const cameFrom = new Map<string, Hex>();
    const gScore = new Map<string, number>();
    gScore.set(startKey, 0);

    while (openSet.length > 0) {
      // Get lowest priority
      openSet.sort((a, b) => a.priority - b.priority);
      const current = openSet.shift()!;
      const currentKey = hexKey(current.hex);

      if (currentKey === endKey) {
        // Reconstruct path
        const path: Hex[] = [to];
        let currentHex = to;
        while (cameFrom.has(hexKey(currentHex))) {
          currentHex = cameFrom.get(hexKey(currentHex))!;
          path.unshift(currentHex);
        }
        return path;
      }

      for (const neighbor of hexNeighbors(current.hex)) {
        const neighborKey = hexKey(neighbor);

        if (!this.validHexes.has(neighborKey)) {
          continue;
        }

        // Check if blocked
        const unit = this.units.find(
          (u) => u.q === neighbor.q && u.r === neighbor.r && !u.is_destroyed
        );
        const selectedUnit = this.units.find((u) => u.id === this.selectedUnitId);
        if (unit && unit.id !== this.selectedUnitId && selectedUnit && unit.owner !== selectedUnit.owner) {
          continue;
        }

        const tentativeG = (gScore.get(currentKey) ?? Infinity) + 1;

        if (tentativeG < (gScore.get(neighborKey) ?? Infinity)) {
          cameFrom.set(neighborKey, current.hex);
          gScore.set(neighborKey, tentativeG);

          const h = Math.abs(neighbor.q - to.q) + Math.abs(neighbor.r - to.r);
          const f = tentativeG + h;

          if (!openSet.find((n) => hexKey(n.hex) === neighborKey)) {
            openSet.push({ hex: neighbor, cost: tentativeG, priority: f });
          }
        }
      }
    }

    return []; // No path found
  }

  private updatePathPreview(): void {
    if (this.selectedUnitId === null || !this.hoveredHex) {
      this.currentPath = [];
      this.renderPath();
      return;
    }

    const hoveredKey = hexKey(this.hoveredHex);
    if (!this.reachableHexes.has(hoveredKey)) {
      this.currentPath = [];
      this.renderPath();
      return;
    }

    const unit = this.units.find((u) => u.id === this.selectedUnitId);
    if (!unit) {
      return;
    }

    this.currentPath = this.findPath({ q: unit.q, r: unit.r }, this.hoveredHex);
    this.renderPath();
  }

  private moveSelectedUnit(target: Hex): void {
    const unit = this.units.find((u) => u.id === this.selectedUnitId);
    if (!unit || this.currentPath.length === 0) {
      return;
    }

    // Calculate final facing based on movement direction
    let newFacing = unit.facing;
    if (this.currentPath.length >= 2) {
      const prevHex = this.currentPath[this.currentPath.length - 2];
      const dx = target.q - prevHex.q;
      const dy = target.r - prevHex.r;

      console.log(`Movement direction: dx=${dx}, dy=${dy}`);

      // Determine facing from direction (flat-top hex axial coordinates)
      if (dx === 1 && dy === 0) newFacing = Facing.East;
      else if (dx === 1 && dy === -1) newFacing = Facing.Northeast;
      else if (dx === 0 && dy === -1) newFacing = Facing.Northwest;
      else if (dx === -1 && dy === 0) newFacing = Facing.West;
      else if (dx === -1 && dy === 1) newFacing = Facing.Southwest;
      else if (dx === 0 && dy === 1) newFacing = Facing.Southeast;
      else {
        console.warn(`Unknown direction: dx=${dx}, dy=${dy}`);
      }
    }

    console.log(`Old facing: ${Facing[unit.facing]}, New facing: ${Facing[newFacing]}`);

    // Update unit position
    const pathCost = this.currentPath.length - 1;
    unit.q = target.q;
    unit.r = target.r;
    unit.facing = newFacing;
    unit.has_moved = true;
    unit.movement_remaining = Math.max(0, unit.movement_remaining - pathCost);

    // Update sprite
    this.updateUnitSprite(unit);

    // Clear selection state
    this.deselectUnit();

    console.log(`Unit ${unit.id} moved to (${target.q}, ${target.r}), facing ${Facing[newFacing]}`);
  }

  private rotateSelectedUnit(direction: number): void {
    const unit = this.units.find((u) => u.id === this.selectedUnitId);
    if (!unit) {
      return;
    }

    // Rotate facing (6 directions)
    unit.facing = ((unit.facing + direction + 6) % 6) as Facing;
    this.updateUnitSprite(unit);
  }

  private addTestUnits(): void {
    // Add Player 1 units (bottom left)
    this.units.push({
      id: 1,
      unit_type: 'Warlord_Titan',
      display_name: 'Warlord Titan',
      owner: 1,
      q: 1,
      r: 7,
      facing: Facing.East,
      sprite_frame: 'Warlord_Titan_E_0000',
      armor: 16,
      max_armor: 16,
      structure: 14,
      max_structure: 14,
      void_shields: 4,
      max_void_shields: 4,
      movement_remaining: 4,
      max_movement: 4,
      has_moved: false,
      has_attacked: false,
      is_destroyed: false,
      is_titan: true,
    });

    this.units.push({
      id: 2,
      unit_type: 'shadowsword',
      display_name: 'Shadowsword',
      owner: 1,
      q: 0,
      r: 8,
      facing: Facing.East,
      sprite_frame: 'shadowsword_E_0000',
      armor: 8,
      max_armor: 8,
      structure: 6,
      max_structure: 6,
      void_shields: 0,
      max_void_shields: 0,
      movement_remaining: 5,
      max_movement: 5,
      has_moved: false,
      has_attacked: false,
      is_destroyed: false,
      is_titan: false,
    });

    this.units.push({
      id: 3,
      unit_type: 'shadowsword2',
      display_name: 'Shadowsword Mk II',
      owner: 1,
      q: 2,
      r: 8,
      facing: Facing.Northeast,
      sprite_frame: 'shadowsword2_NE_0000',
      armor: 8,
      max_armor: 8,
      structure: 6,
      max_structure: 6,
      void_shields: 0,
      max_void_shields: 0,
      movement_remaining: 5,
      max_movement: 5,
      has_moved: false,
      has_attacked: false,
      is_destroyed: false,
      is_titan: false,
    });

    // Add Player 2 units (top right)
    this.units.push({
      id: 4,
      unit_type: 'Reaver_Titan',
      display_name: 'Reaver Titan',
      owner: 2,
      q: 10,
      r: 2,
      facing: Facing.West,
      sprite_frame: 'Reaver_Titan_W_0000',
      armor: 12,
      max_armor: 12,
      structure: 10,
      max_structure: 10,
      void_shields: 2,
      max_void_shields: 2,
      movement_remaining: 6,
      max_movement: 6,
      has_moved: false,
      has_attacked: false,
      is_destroyed: false,
      is_titan: true,
    });

    this.units.push({
      id: 5,
      unit_type: 'shadowsword3',
      display_name: 'Shadowsword Mk III',
      owner: 2,
      q: 11,
      r: 1,
      facing: Facing.Southwest,
      sprite_frame: 'shadowsword3_SW_0000',
      armor: 8,
      max_armor: 8,
      structure: 6,
      max_structure: 6,
      void_shields: 0,
      max_void_shields: 0,
      movement_remaining: 5,
      max_movement: 5,
      has_moved: false,
      has_attacked: false,
      is_destroyed: false,
      is_titan: false,
    });
  }

  private renderUnits(): void {
    // Create sprites for all units
    for (const unit of this.units) {
      this.createUnitSprite(unit);
    }
  }

  private createUnitSprite(unit: Unit): void {
    const pos = this.hexToWorld({ q: unit.q, r: unit.r });
    const spriteDir = facingToSpriteDirection(unit.facing);
    const frameKey = `${unit.unit_type}_${spriteDir}_0000`;

    const sprite = this.add.sprite(pos.x, pos.y, unit.unit_type, frameKey);
    sprite.setOrigin(0.5, 0.6); // Offset for better hex centering
    sprite.setDepth(DEPTH_UNITS);

    // Scale sprites to fit hex (512px sprites in 60px hexes)
    const scale = (HEX_SIZE * 1.8) / 512;
    sprite.setScale(scale);

    // Tint based on owner
    if (unit.owner === 1) {
      sprite.setTint(0xaaaaff); // Slight blue tint
    } else {
      sprite.setTint(0xffaaaa); // Slight red tint
    }

    this.unitSprites.set(unit.id, sprite);
  }

  private updateUnitSprite(unit: Unit): void {
    const sprite = this.unitSprites.get(unit.id);
    if (!sprite) {
      console.warn(`No sprite found for unit ${unit.id}`);
      return;
    }

    const pos = this.hexToWorld({ q: unit.q, r: unit.r });
    const spriteDir = facingToSpriteDirection(unit.facing);
    const frameKey = `${unit.unit_type}_${spriteDir}_0000`;

    console.log(`Updating sprite for unit ${unit.id}: facing=${unit.facing} (${Facing[unit.facing]}), spriteDir=${spriteDir}, frameKey=${frameKey}`);

    // Check if frame exists in texture
    const texture = this.textures.get(unit.unit_type);
    const frameNames = texture.getFrameNames();
    console.log(`Available frames for ${unit.unit_type}:`, frameNames);

    if (!frameNames.includes(frameKey)) {
      console.error(`Frame ${frameKey} not found in texture ${unit.unit_type}!`);
    }

    sprite.setPosition(pos.x, pos.y);
    sprite.setFrame(frameKey);

    // Verify the frame was set
    console.log(`Sprite frame after setFrame: ${sprite.frame.name}`);
  }

  private renderSelection(): void {
    this.selectionGraphics.clear();

    if (this.selectedUnitId === null) {
      return;
    }

    const unit = this.units.find((u) => u.id === this.selectedUnitId);
    if (!unit) {
      return;
    }

    const pos = this.hexToWorld({ q: unit.q, r: unit.r });

    // Draw selection ring
    drawHex(
      this.selectionGraphics,
      pos.x,
      pos.y,
      HEX_SIZE + 4,
      undefined,
      undefined,
      COLOR_SELECTED,
      1,
      3
    );
  }

  private renderReachable(): void {
    this.reachableGraphics.clear();

    for (const [key, remaining] of this.reachableHexes) {
      const [q, r] = key.split(',').map(Number);
      const pos = this.hexToWorld({ q, r });

      // Skip unit's current position
      const unit = this.units.find((u) => u.id === this.selectedUnitId);
      if (unit && unit.q === q && unit.r === r) {
        continue;
      }

      // Alpha based on remaining MP
      const alpha = 0.2 + (remaining / 6) * 0.2;

      drawHex(
        this.reachableGraphics,
        pos.x,
        pos.y,
        HEX_SIZE - 2,
        COLOR_REACHABLE,
        alpha,
        COLOR_REACHABLE,
        0.5,
        1
      );
    }
  }

  private renderPath(): void {
    this.pathGraphics.clear();

    if (this.currentPath.length <= 1) {
      return;
    }

    // Draw path line
    this.pathGraphics.lineStyle(4, COLOR_PATH, 0.8);
    this.pathGraphics.beginPath();

    const startPos = this.hexToWorld(this.currentPath[0]);
    this.pathGraphics.moveTo(startPos.x, startPos.y);

    for (let i = 1; i < this.currentPath.length; i++) {
      const pos = this.hexToWorld(this.currentPath[i]);
      this.pathGraphics.lineTo(pos.x, pos.y);
    }

    this.pathGraphics.strokePath();

    // Highlight destination hex
    const destHex = this.currentPath[this.currentPath.length - 1];
    const destPos = this.hexToWorld(destHex);

    drawHex(
      this.pathGraphics,
      destPos.x,
      destPos.y,
      HEX_SIZE - 2,
      COLOR_PATH,
      0.3,
      COLOR_PATH,
      1,
      2
    );
  }

  private createUI(): void {
    // Create UI container (fixed to camera)
    const uiContainer = this.add.container(10, 10);
    uiContainer.setScrollFactor(0);
    uiContainer.setDepth(DEPTH_UI);

    // Background panel
    const panel = this.add.graphics();
    panel.fillStyle(0x000000, 0.7);
    panel.fillRoundedRect(0, 0, 200, 100, 8);
    uiContainer.add(panel);

    // Turn text
    this.turnText = this.add.text(10, 10, `Turn: ${this.currentTurn}`, {
      font: '16px monospace',
      color: '#ffffff',
    });
    uiContainer.add(this.turnText);

    // Phase text
    this.phaseText = this.add.text(10, 30, `Phase: ${this.currentPhase}`, {
      font: '16px monospace',
      color: '#ffffff',
    });
    uiContainer.add(this.phaseText);

    // Active player text
    const playerColor = this.activePlayer === 1 ? '#6699ff' : '#ff6666';
    this.playerText = this.add.text(10, 50, `Player: ${this.activePlayer}`, {
      font: '16px monospace',
      color: playerColor,
    });
    uiContainer.add(this.playerText);

    // Instructions
    const instructionsText = this.add.text(10, 75, 'Click unit to select, Q/E rotate', {
      font: '12px monospace',
      color: '#888888',
    });
    uiContainer.add(instructionsText);
  }

  private endTurn(): void {
    // Reset all units for current player
    for (const unit of this.units) {
      if (unit.owner === this.activePlayer) {
        unit.has_moved = false;
        unit.has_attacked = false;
        unit.movement_remaining = unit.max_movement;
      }
    }

    // Switch players
    this.activePlayer = this.activePlayer === 1 ? 2 : 1;

    // Increment turn when returning to Player 1
    if (this.activePlayer === 1) {
      this.currentTurn++;
    }

    // Deselect any selected unit
    this.deselectUnit();

    console.log(`Turn ${this.currentTurn}, Player ${this.activePlayer}'s turn`);

    // Update UI (would need proper references)
    this.updateUI();
  }

  private updateUI(): void {
    if (this.turnText) {
      this.turnText.setText(`Turn: ${this.currentTurn}`);
    }
    if (this.phaseText) {
      this.phaseText.setText(`Phase: ${this.currentPhase}`);
    }
    if (this.playerText) {
      const color = this.activePlayer === 1 ? '#6699ff' : '#ff6666';
      this.playerText.setText(`Player: ${this.activePlayer}`);
      this.playerText.setColor(color);
    }
  }
}
