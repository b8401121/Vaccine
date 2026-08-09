/* tslint:disable */
/* eslint-disable */

export function calculate_catch_up(vaccine_id: string, last_dose_num: number, year: number, month: number, day: number, is_roc: boolean): any;

export function calculate_growth_percentile(gender: string, age_months: number, height: number, weight: number, head: any): any;

export function get_all_vaccines(): any;

export function get_eligible_vaccines(year: number, month: number, day: number, is_roc: boolean, gender: string, location: string): any;

export function get_travel_advisory(destination: string, purpose: string): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly calculate_catch_up: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly calculate_growth_percentile: (a: number, b: number, c: number, d: number, e: number, f: any) => [number, number, number];
    readonly get_all_vaccines: () => [number, number, number];
    readonly get_eligible_vaccines: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => [number, number, number];
    readonly get_travel_advisory: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
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
