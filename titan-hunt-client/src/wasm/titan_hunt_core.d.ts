/* tslint:disable */
/* eslint-disable */

/**
 * Game engine wrapper for WASM
 */
export class TitanHuntEngine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Add a unit to the game
     */
    addUnit(id: number, unit_type: string, player: number, q: number, r: number, facing: number): void;
    /**
     * End the current phase
     */
    endPhase(): any;
    /**
     * End the current turn
     */
    endTurn(): any;
    /**
     * Find path from a unit to a target hex
     */
    findPath(unit_id: number, target_q: number, target_r: number): any;
    /**
     * Get the active player (1 or 2)
     */
    getActivePlayer(): number;
    /**
     * Get the current phase
     */
    getCurrentPhase(): string;
    /**
     * Get the current turn number
     */
    getCurrentTurn(): number;
    /**
     * Get all valid hex coordinates on the map
     */
    getMapHexes(): any;
    /**
     * Get map dimensions
     */
    getMapSize(): any;
    /**
     * Get reachable hexes for a unit
     */
    getReachableHexes(unit_id: number): any;
    /**
     * Get the selected unit ID
     */
    getSelectedUnit(): number | undefined;
    /**
     * Get the current game state as JSON
     */
    getState(): any;
    /**
     * Get all units as JSON
     */
    getUnits(): any;
    /**
     * Convert hex to pixel coordinates
     */
    hexToPixel(q: number, r: number, hex_size: number): any;
    /**
     * Execute a move command
     */
    moveUnit(unit_id: number, path_json: any, final_facing: number): any;
    /**
     * Create a new game with the specified map dimensions
     */
    constructor(width: number, height: number);
    /**
     * Convert pixel coordinates to hex
     */
    pixelToHex(x: number, y: number, hex_size: number): any;
    /**
     * Select a unit
     */
    selectUnit(unit_id?: number | null): void;
    /**
     * Start the game (transition from deployment to movement)
     */
    startGame(): void;
}

/**
 * Initialize panic hook for better error messages in browser console
 */
export function init(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_titanhuntengine_free: (a: number, b: number) => void;
    readonly init: () => void;
    readonly titanhuntengine_addUnit: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number];
    readonly titanhuntengine_endPhase: (a: number) => [number, number, number];
    readonly titanhuntengine_endTurn: (a: number) => [number, number, number];
    readonly titanhuntengine_findPath: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly titanhuntengine_getActivePlayer: (a: number) => number;
    readonly titanhuntengine_getCurrentPhase: (a: number) => [number, number];
    readonly titanhuntengine_getCurrentTurn: (a: number) => number;
    readonly titanhuntengine_getMapHexes: (a: number) => [number, number, number];
    readonly titanhuntengine_getMapSize: (a: number) => [number, number, number];
    readonly titanhuntengine_getReachableHexes: (a: number, b: number) => [number, number, number];
    readonly titanhuntengine_getSelectedUnit: (a: number) => number;
    readonly titanhuntengine_getState: (a: number) => [number, number, number];
    readonly titanhuntengine_getUnits: (a: number) => [number, number, number];
    readonly titanhuntengine_hexToPixel: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly titanhuntengine_moveUnit: (a: number, b: number, c: any, d: number) => [number, number, number];
    readonly titanhuntengine_new: (a: number, b: number) => number;
    readonly titanhuntengine_pixelToHex: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly titanhuntengine_selectUnit: (a: number, b: number) => void;
    readonly titanhuntengine_startGame: (a: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
