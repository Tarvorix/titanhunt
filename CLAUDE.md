## Claude Code Guidelines

maxTokens: 128000

## Core Behavior

1. **Listen to all instructions** - Read the user's request carefully. Understand exactly what they are asking before doing anything.
2. **Only do what is asked** - Do NOT modify code that wasn't explicitly requested. Do NOT "fix" things that aren't broken. Do NOT make improvements or refactors unless specifically asked.
3. **Explain before making changes** - Before editing any file, explain what you plan to change and why. Wait for confirmation if the change is significant.
4. **Double check all work** - After making changes, verify they are correct. Re-read the original request to confirm you addressed what was actually asked, not what you assumed.
5. **Complete all tasks fully** - Do not simplify, Do not use placeholders, Complete all tasks fully and completely.
6. **When commiting and pushing to Github** - Commiter and Author should be Tarvorix...no mention of Claude anywhere in any commit message.
7. **No Placeholder Code** - Absolutely no placeholder code everything must be implemented fully
8. **No Simplification** - Absolutely no simplification of code everything must be implemented fully

## Prohibited Actions

- NEVER change working code without explicit permission
- NEVER assume something is broken without evidence
- NEVER make "while I'm here" improvements
- NEVER claim to fix something you weren't asked to fix
- NEVER touch deployment code if asked to fix movement code
- NEVER touch movement code if asked to fix deployment code
- NEVER add flags, options, or parameters that weren't in the original command
- NEVER start coding tasks without explicit user request
- NEVER create todo lists for tasks the user didn't ask for
- NEVER remove functionality from code - if adding features, preserve ALL existing behavior (e.g., gzip flush for live counts)
- Never simplify or use placeholder code

## File Deletion Rules

- NEVER delete any file or directory without explicit user confirmation
- Before ANY rm, rm -rf, or delete operation: list exactly what will be deleted and ask "Should I delete these? (yes/no)"
- Wait for explicit "yes" before proceeding
- No exceptions

## Before Any Code Change

Ask yourself:
1. Did the user explicitly ask for this change?
2. Is this code actually broken, or am I assuming?
3. Will this change affect other working functionality?
4. Have I explained what I'm about to do?

## If Uncertain

ASK. Do not guess. Do not assume. Ask the user to clarify.

## Git Author Configuration

All commits must use Tarvorix as author and committer. No mention of "Claude" anywhere in git commits.

Author name: Tarvorix
Committer name: Tarvorix
Never use "Claude" or any variation in commit author, committer, or commit messages or email
Configure git before committing:
git config user.name "Tarvorix"
git config user.email "Tarvorix@users.noreply.github.com"
No links to Claude Sessions in comments

## Project Overview

Titan Hunt is a 40K-inspired hex-based tactical game similar to Battletech/MekForce.

## Architecture

### Rust Game Engine (`titan-hunt-core/`)
Pure Rust crate compiled to WASM for browser integration.

```
titan-hunt-core/src/
├── lib.rs           # Module exports and re-exports
├── hex.rs           # Hex coordinate system (axial q,r coords)
├── rules.rs         # GameState, Unit, Phase, Command, UnitType
├── movement.rs      # A* pathfinding, reachable hex calculation
└── wasm_api.rs      # WASM bindings via wasm-bindgen
```

**Key Types:**
- `HexCoord` - Axial coordinates (q, r) with cube conversion for distance
- `Facing` - 6 hex directions (East, Northeast, Northwest, West, Southwest, Southeast)
- `Unit` - Position, facing, health (armor/structure/void shields), movement
- `UnitType` - ReaverTitan, WarlordTitan, Shadowsword, Shadowsword2, Shadowsword3
- `GameState` - Map, units, turn/phase tracking, commands
- `Command` - Move, EndPhase, EndTurn

### TypeScript Client (`titan-hunt-client/`)
Phaser 3 web client with Vite build system.

```
titan-hunt-client/src/
├── main.ts              # Phaser game config entry point
├── types.ts             # TypeScript type definitions
├── hex/HexUtils.ts      # Hex math, rendering, coordinate conversion
└── scenes/
    ├── BootScene.ts     # Asset loading (sprite atlases)
    └── BattleScene.ts   # Main game scene (grid, units, input)
```

**Phaser Configuration:**
- `Phaser.AUTO` renderer (WebGL with Canvas fallback)
- `Phaser.Scale.RESIZE` for responsive layout
- Camera pan (mouse drag) and zoom (scroll wheel)

### Sprite Atlases (`atlases/`)
TexturePacker JSON format with 8-directional isometric sprites.

**Frame naming:** `{unit_type}_{direction}_0000`
- Directions: S, SW, W, NW, N, NE, E, SE
- Frame size: 512x512 pixels
- Atlas size: 512x4096 pixels (vertical strip)

**Available units:**
- Reaver_Titan, Warlord_Titan (Titans with void shields)
- shadowsword, shadowsword2, shadowsword3 (Super-heavy tanks)

## Commands

```bash
# Rust
cargo check              # Type check
cargo test               # Run tests
cargo build --release    # Build release

# TypeScript
cd titan-hunt-client
npm install              # Install dependencies
npm run dev              # Start dev server (localhost:3000)
npm run typecheck        # TypeScript check
npm run build            # Production build

# WASM (requires wasm-pack)
cd titan-hunt-core
wasm-pack build --target web --out-dir ../titan-hunt-client/src/wasm
```

## Game Controls

- **Click unit** - Select/deselect
- **Click reachable hex** - Move selected unit
- **Q/E** - Rotate facing
- **Space** - End turn
- **Escape** - Deselect
- **Mouse drag** - Pan camera
- **Scroll wheel** - Zoom

## Hex Grid

- Flat-top hex orientation
- Axial coordinate system (q, r)
- Default hex size: 60 pixels
- Map origin offset: (100, 100) pixels

