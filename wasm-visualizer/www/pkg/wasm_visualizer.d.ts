/* tslint:disable */
/* eslint-disable */
/**
 * A completed sort recording, exposed to JS as a class.
 */
export class SortRun {
  free(): void;
  /**
   * Number of recorded steps.
   */
  step_count(): number;
  /**
   * Replays the steps in Rust and returns per-kind counts — the same
   * replay the browser does, proving the recording is self-consistent.
   */
  replay_stats(): Uint32Array;
  /**
   * Human-readable algorithm name (String marshaling).
   */
  algorithm_name(): string;
  /**
   * Number of elements.
   */
  len(): number;
  /**
   * Creates a run: shuffles `n` values and records algorithm `kind`.
   *
   * # Errors
   *
   * `Err(String)` (thrown as a JS `Error`) if arguments are invalid.
   */
  constructor(kind: number, n: number);
  /**
   * The packed steps as a `Uint32Array` (a copy).
   */
  steps(): Uint32Array;
  /**
   * The initial (pre-sort) array as a typed array (a copy).
   */
  initial(): Int32Array;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly sort_new: (a: number, b: number) => number;
  readonly __wbg_sortrun_free: (a: number, b: number) => void;
  readonly sort_final_ptr: () => number;
  readonly sort_free: () => void;
  readonly sort_initial_ptr: () => number;
  readonly sort_len: () => number;
  readonly sort_steps_len: () => number;
  readonly sort_steps_ptr: () => number;
  readonly sortrun_algorithm_name: (a: number) => [number, number];
  readonly sortrun_initial: (a: number) => [number, number];
  readonly sortrun_len: (a: number) => number;
  readonly sortrun_new: (a: number, b: number) => [number, number, number];
  readonly sortrun_replay_stats: (a: number) => [number, number];
  readonly sortrun_step_count: (a: number) => number;
  readonly sortrun_steps: (a: number) => [number, number];
  readonly tree_free: () => void;
  readonly tree_init: (a: number, b: number) => number;
  readonly tree_nodes: () => number;
  readonly tree_steps_len: (a: number) => number;
  readonly tree_steps_ptr: (a: number) => number;
  readonly __wbindgen_export_0: WebAssembly.Table;
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
